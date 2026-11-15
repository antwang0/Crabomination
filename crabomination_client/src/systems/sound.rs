//! Sound: a short cue for the moments a player should hear — a spell cast,
//! a land played, damage, a creature dying, life gained, a card drawn, your
//! turn starting, the game ending.
//!
//! The cues are **synthesized at startup** ([`synth`]): a few sine /
//! triangle tones with pitch sweeps and a noise burst, a couple of hundred
//! milliseconds each. No asset files, nothing to license, nothing for the
//! art prefetch to fetch, and the web build gets the same sounds. They play
//! through a custom [`Decodable`] source, since the client's Bevy carries
//! only the Vorbis decoder.
//!
//! A batch of events plays each cue at most once and at most
//! [`MAX_CUES_PER_BATCH`] cues, loudest-meaning first, so a board wipe is
//! one crash rather than twelve. Settings → Sound sets the volume (Off
//! mutes); `GameplayConfig::sound_volume` persists it.

use std::sync::Arc;
use std::time::Duration;

use bevy::audio::{AddAudioSource, AudioPlayer, Decodable, PlaybackSettings, Sample, Source, Volume};
use bevy::prelude::*;
use crabomination::net::GameEventWire;

use crate::net_plugin::LatestServerEvents;

const RATE: u32 = 44_100;

/// At most this many different cues sound for one batch of events.
pub const MAX_CUES_PER_BATCH: usize = 3;

/// A synthesized sound, played through Bevy's audio as its own source type.
#[derive(Asset, TypePath, Clone)]
pub struct Synth {
    samples: Arc<[Sample]>,
}

pub struct SynthDecoder {
    samples: Arc<[Sample]>,
    at: usize,
}

impl Iterator for SynthDecoder {
    type Item = Sample;
    fn next(&mut self) -> Option<Sample> {
        let s = self.samples.get(self.at).copied();
        self.at += 1;
        s
    }
}

impl Source for SynthDecoder {
    fn current_span_len(&self) -> Option<usize> {
        Some(self.samples.len().saturating_sub(self.at))
    }
    fn channels(&self) -> bevy::audio::ChannelCount {
        std::num::NonZero::new(1).expect("one channel")
    }
    fn sample_rate(&self) -> bevy::audio::SampleRate {
        std::num::NonZero::new(RATE).expect("a sample rate")
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(self.samples.len() as f64 / RATE as f64))
    }
}

impl Decodable for Synth {
    type Decoder = SynthDecoder;
    fn decoder(&self) -> SynthDecoder {
        SynthDecoder { samples: self.samples.clone(), at: 0 }
    }
}

/// What a cue says. Ordered by how much it matters when several land in one
/// batch: the first [`MAX_CUES_PER_BATCH`] of a batch's cues, in this order,
/// are the ones that sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cue {
    Win,
    Lose,
    YourTurn,
    Death,
    Countered,
    Damage,
    Cast,
    LifeGain,
    Land,
    Draw,
}

impl Cue {
    pub const ALL: [Cue; 10] = [
        Cue::Win,
        Cue::Lose,
        Cue::YourTurn,
        Cue::Death,
        Cue::Countered,
        Cue::Damage,
        Cue::Cast,
        Cue::LifeGain,
        Cue::Land,
        Cue::Draw,
    ];
}

/// One voice of a cue: a tone gliding from `from` to `to` Hz over `secs`,
/// starting `at` seconds in, with a quick attack and an exponential decay.
struct Voice {
    at: f32,
    secs: f32,
    from: f32,
    to: f32,
    gain: f32,
    /// 0 = sine, 1 = triangle (brighter); noise is its own field.
    tri: f32,
    /// Share of white noise mixed in (a hit, a crash).
    noise: f32,
}

const fn v(at: f32, secs: f32, from: f32, to: f32, gain: f32) -> Voice {
    Voice { at, secs, from, to, gain, tri: 0.0, noise: 0.0 }
}

fn voices(cue: Cue) -> Vec<Voice> {
    match cue {
        // A rising two-note chime.
        Cue::Cast => vec![
            Voice { tri: 0.6, ..v(0.0, 0.12, 880.0, 880.0, 0.35) },
            Voice { tri: 0.6, ..v(0.07, 0.16, 1318.5, 1318.5, 0.3) },
        ],
        // A soft low thump.
        Cue::Land => vec![v(0.0, 0.14, 150.0, 85.0, 0.6)],
        // A short noisy hit over a low knock.
        Cue::Damage => vec![v(0.0, 0.1, 180.0, 90.0, 0.45), Voice { noise: 1.0, ..v(0.0, 0.07, 0.0, 0.0, 0.25) }],
        // A falling tone with a crash.
        Cue::Death => vec![
            Voice { tri: 0.4, ..v(0.0, 0.38, 420.0, 110.0, 0.4) },
            Voice { noise: 1.0, ..v(0.0, 0.18, 0.0, 0.0, 0.18) },
        ],
        // A bright rising arpeggio.
        Cue::LifeGain => vec![
            v(0.0, 0.12, 1046.5, 1046.5, 0.22),
            v(0.06, 0.12, 1318.5, 1318.5, 0.22),
            v(0.12, 0.18, 1568.0, 1568.0, 0.22),
        ],
        // A quiet brush: the most frequent cue, so the least of one.
        Cue::Draw => vec![Voice { noise: 1.0, ..v(0.0, 0.06, 0.0, 0.0, 0.08) }, v(0.0, 0.05, 2400.0, 1800.0, 0.05)],
        // Two bells: your turn.
        Cue::YourTurn => vec![
            Voice { tri: 0.3, ..v(0.0, 0.35, 783.99, 783.99, 0.3) },
            Voice { tri: 0.3, ..v(0.14, 0.45, 1046.5, 1046.5, 0.3) },
        ],
        // A dissonant drop.
        Cue::Countered => vec![
            Voice { tri: 0.8, ..v(0.0, 0.22, 740.0, 330.0, 0.25) },
            Voice { tri: 0.8, ..v(0.0, 0.22, 784.0, 349.0, 0.2) },
        ],
        // A major triad, rising.
        Cue::Win => vec![
            Voice { tri: 0.5, ..v(0.0, 0.3, 523.25, 523.25, 0.3) },
            Voice { tri: 0.5, ..v(0.15, 0.3, 659.25, 659.25, 0.3) },
            Voice { tri: 0.5, ..v(0.3, 0.6, 783.99, 783.99, 0.3) },
        ],
        // A minor fall.
        Cue::Lose => vec![
            Voice { tri: 0.3, ..v(0.0, 0.35, 440.0, 440.0, 0.3) },
            Voice { tri: 0.3, ..v(0.2, 0.35, 349.23, 349.23, 0.3) },
            Voice { tri: 0.3, ..v(0.4, 0.7, 293.66, 280.0, 0.3) },
        ],
    }
}

/// The cue's samples, mono at [`RATE`], peaking at most at 0.9.
pub fn synth(cue: Cue) -> Vec<f32> {
    let voices = voices(cue);
    let secs = voices.iter().map(|v| v.at + v.secs).fold(0.0, f32::max);
    let len = (secs * RATE as f32) as usize + 1;
    let mut out = vec![0.0f32; len];
    // A fixed LCG, so the noise is the same every launch.
    let mut seed: u32 = 0x9E37_79B9;
    let mut noise = || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0
    };
    for voice in &voices {
        let start = (voice.at * RATE as f32) as usize;
        let n = (voice.secs * RATE as f32) as usize;
        let mut phase = 0.0f32;
        for i in 0..n {
            let t = i as f32 / n as f32;
            let freq = voice.from + (voice.to - voice.from) * t;
            phase = (phase + freq / RATE as f32).fract();
            let sine = (phase * std::f32::consts::TAU).sin();
            let tri = 1.0 - 4.0 * (phase - 0.5).abs();
            let tone = sine * (1.0 - voice.tri) + tri * voice.tri;
            let s = tone * (1.0 - voice.noise) + noise() * voice.noise;
            // 4 ms attack, exponential decay to the end.
            let attack = (i as f32 / (0.004 * RATE as f32)).min(1.0);
            let env = attack * (-4.0 * t).exp() * (1.0 - t);
            if let Some(o) = out.get_mut(start + i) {
                *o += s * env * voice.gain;
            }
        }
    }
    let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak > 0.9 {
        out.iter_mut().for_each(|s| *s *= 0.9 / peak);
    }
    out
}

/// Every cue's sound, built at startup.
#[derive(Resource)]
pub struct SoundBank(Vec<(Cue, Handle<Synth>)>);

pub fn build_sound_bank(mut commands: Commands, mut synths: ResMut<Assets<Synth>>) {
    let bank = Cue::ALL
        .iter()
        .map(|&cue| {
            let samples: Arc<[Sample]> = synth(cue).into_iter().map(Sample::from).collect();
            (cue, synths.add(Synth { samples }))
        })
        .collect();
    commands.insert_resource(SoundBank(bank));
}

/// The cues a batch of events calls for, each once, most important first,
/// at most [`MAX_CUES_PER_BATCH`].
pub fn cues_for(events: &[GameEventWire], viewer: usize) -> Vec<Cue> {
    let mut cues: Vec<Cue> = events
        .iter()
        .filter_map(|ev| match ev {
            GameEventWire::GameOver { winner } => Some(if *winner == Some(viewer) { Cue::Win } else { Cue::Lose }),
            GameEventWire::TurnStarted { player, .. } if *player == viewer => Some(Cue::YourTurn),
            GameEventWire::CreatureDied { .. } => Some(Cue::Death),
            GameEventWire::SpellCountered { .. } => Some(Cue::Countered),
            GameEventWire::DamageDealt { amount, .. } if *amount > 0 => Some(Cue::Damage),
            GameEventWire::SpellCast { .. } => Some(Cue::Cast),
            GameEventWire::LifeGained { amount, .. } if *amount > 0 => Some(Cue::LifeGain),
            GameEventWire::LandPlayed { .. } => Some(Cue::Land),
            GameEventWire::CardDrawn { player, .. } if *player == viewer => Some(Cue::Draw),
            _ => None,
        })
        .collect();
    cues.sort();
    cues.dedup();
    cues.truncate(MAX_CUES_PER_BATCH);
    cues
}

/// Play the cues the latest batch of events calls for, at the set volume.
pub fn play_event_sounds(
    mut commands: Commands,
    events: Res<LatestServerEvents>,
    view: Res<crate::net_plugin::CurrentView>,
    bank: Option<Res<SoundBank>>,
    gameplay: Res<crate::config::GameplayConfig>,
) {
    if events.0.is_empty() || gameplay.sound_volume <= 0.0 {
        return;
    }
    let (Some(bank), Some(cv)) = (bank, view.0.as_ref()) else { return };
    for cue in cues_for(&events.0, cv.your_seat) {
        let Some((_, handle)) = bank.0.iter().find(|(c, _)| *c == cue) else { continue };
        commands.spawn((
            AudioPlayer::<Synth>(handle.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gameplay.sound_volume)),
        ));
    }
}

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<Synth>()
            .add_systems(Startup, build_sound_bank)
            .add_systems(Update, play_event_sounds.run_if(in_state(crate::menu::AppState::InGame)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination::card::CardId;

    #[test]
    fn every_cue_is_short_audible_and_unclipped() {
        for cue in Cue::ALL {
            let s = synth(cue);
            let secs = s.len() as f32 / RATE as f32;
            assert!((0.04..1.5).contains(&secs), "{cue:?} lasts {secs} s");
            let peak = s.iter().fold(0.0f32, |m, x| m.max(x.abs()));
            assert!(peak > 0.02 && peak <= 0.9 + 1e-6, "{cue:?} peaks at {peak}");
        }
    }

    /// A board wipe is one death cue, not twelve; the cues come most
    /// important first and never more than the cap.
    #[test]
    fn a_batch_plays_each_cue_once_most_important_first() {
        let mut events: Vec<GameEventWire> =
            (0..12).map(|i| GameEventWire::CreatureDied { card_id: CardId(i) }).collect();
        events.push(GameEventWire::LandPlayed { player: 0, card_id: CardId(99), played: true });
        events.push(GameEventWire::TurnStarted { player: 0, turn: 3 });
        events.push(GameEventWire::LifeGained { player: 1, amount: 2 });
        events.push(GameEventWire::CardDrawn { player: 1, card_id: CardId(5) });
        assert_eq!(cues_for(&events, 0), vec![Cue::YourTurn, Cue::Death, Cue::LifeGain]);
        assert_eq!(cues_for(&[GameEventWire::TurnStarted { player: 1, turn: 4 }], 0), vec![], "not your turn");
        assert_eq!(cues_for(&[GameEventWire::GameOver { winner: Some(1) }], 0), vec![Cue::Lose]);
    }
}
