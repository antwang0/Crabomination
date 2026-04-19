use bevy::time::Timer;
use bevy::prelude::*;
use crabomination::{
    card::{CardDefinition, CardId},
    catalog::*,
    game::{GameEvent, GameState},
    mana::{Color as ManaColor, ManaCost},
    player::Player,
};
use rand::seq::SliceRandom;

pub const PLAYER_0: usize = 0;
pub const PLAYER_1: usize = 1;

#[derive(Resource)]
pub struct GameResource {
    pub state: GameState,
}

#[derive(Resource, Default)]
pub struct GameLog {
    pub entries: Vec<String>,
}

impl GameLog {
    pub fn push(&mut self, msg: impl Into<String>) {
        self.entries.push(msg.into());
        if self.entries.len() > 16 {
            self.entries.remove(0);
        }
    }

    pub fn apply_events(&mut self, events: &[GameEvent]) {
        let mut pending_step: Option<String> = None;
        for event in events {
            if matches!(event, GameEvent::StepChanged(_)) {
                // Buffer the step label; only flush it if something happens in this step
                pending_step = Some(describe_event(event));
            } else {
                if let Some(step_msg) = pending_step.take() {
                    self.push(step_msg);
                }
                self.push(describe_event(event));
            }
        }
        // Trailing StepChanged with nothing following = empty phase, discard it
    }
}

fn player_label(idx: usize) -> &'static str {
    if idx == PLAYER_0 { "Player 0" } else { "Player 1" }
}

fn describe_event(event: &GameEvent) -> String {
    match event {
        GameEvent::StepChanged(step) => format!(">>> {step:?}"),
        GameEvent::TurnStarted { player, turn } => {
            format!("Turn {turn}: {}'s turn", player_label(*player))
        }
        GameEvent::CardDrawn { player, .. } => format!("{} draws", player_label(*player)),
        GameEvent::CardDiscarded { player, .. } => format!("{} discards", player_label(*player)),
        GameEvent::LandPlayed { player, .. } => format!("{} plays land", player_label(*player)),
        GameEvent::SpellCast { player, .. } => format!("{} casts spell", player_label(*player)),
        GameEvent::AbilityActivated { .. } => "Ability activated".into(),
        GameEvent::ManaAdded { player, color } => {
            format!("{} adds {:?}", player_label(*player), color)
        }
        GameEvent::ColorlessManaAdded { player } => {
            format!("{} adds {{C}}", player_label(*player))
        }
        GameEvent::PermanentEntered { .. } => "Permanent enters".into(),
        GameEvent::DamageDealt { amount, to_player, .. } => {
            if let Some(p) = to_player {
                format!("{amount} dmg to {}", player_label(*p))
            } else {
                format!("{amount} dmg to creature")
            }
        }
        GameEvent::LifeLost { player, amount } => {
            format!("{} loses {amount} life", player_label(*player))
        }
        GameEvent::LifeGained { player, amount } => {
            format!("{} gains {amount} life", player_label(*player))
        }
        GameEvent::CreatureDied { .. } => "Creature died".into(),
        GameEvent::PumpApplied { power, toughness, .. } => {
            format!("+{power}/+{toughness} until EOT")
        }
        GameEvent::AttackerDeclared(_) => "Attacker declared".into(),
        GameEvent::BlockerDeclared { .. } => "Blocker declared".into(),
        GameEvent::CombatResolved => "Combat resolved".into(),
        GameEvent::PermanentExiled { .. } => "Permanent exiled".into(),
        GameEvent::CounterAdded { count, counter_type, .. } => {
            format!("{count} {:?} counter(s) added", counter_type)
        }
        GameEvent::CounterRemoved { count, counter_type, .. } => {
            format!("{count} {:?} counter(s) removed", counter_type)
        }
        GameEvent::PermanentTapped { .. } => "Permanent tapped".into(),
        GameEvent::PermanentUntapped { .. } => "Permanent untapped".into(),
        GameEvent::TokenCreated { .. } => "Token created".into(),
        GameEvent::CardMilled { player, .. } => format!("{} milled a card", player_label(*player)),
        GameEvent::ScryPerformed { player, looked_at, bottomed } => {
            format!(
                "{} scry {looked_at}: kept {} on top, sent {bottomed} to bottom",
                player_label(*player),
                looked_at - bottomed,
            )
        }
        GameEvent::TopCardRevealed { player, card_name, is_land } => {
            let suffix = if *is_land { " (land — drawn!)" } else { " (not a land)" };
            format!("Revealed: {} for {}{}", card_name, player_label(*player), suffix)
        }
        GameEvent::GameOver { winner } => match winner {
            Some(p) => format!("GAME OVER: {} wins!", player_label(*p)),
            None => "GAME OVER: Draw!".into(),
        },
        GameEvent::FirstStrikeDamageResolved => "First-strike damage resolved".into(),
        GameEvent::AttachmentMoved { .. } => "Attachment moved".into(),
        GameEvent::PoisonAdded { player, amount } => {
            format!("{} gets {amount} poison counter(s)", player_label(*player))
        }
        GameEvent::LoyaltyAbilityActivated { .. } => "Loyalty ability activated".into(),
        GameEvent::LoyaltyChanged { new_loyalty, .. } => {
            format!("Loyalty changed to {new_loyalty}")
        }
        GameEvent::PlaneswalkerDied { .. } => "Planeswalker died".into(),
        GameEvent::SpellsCopied { .. } => "Spells copied".into(),
        GameEvent::SurveilPerformed { player, looked_at, graveyarded } => {
            format!(
                "{} surveils {looked_at}: kept {}, sent {graveyarded} to graveyard",
                player_label(*player),
                looked_at - graveyarded,
            )
        }
    }
}

/// Format a player's mana pool as a compact string, e.g. "R:3".
pub fn format_mana_pool(state: &GameState, player_idx: usize) -> String {
    let pool = &state.players[player_idx].mana_pool;
    let colors = [
        (ManaColor::White, 'W'),
        (ManaColor::Blue, 'U'),
        (ManaColor::Black, 'B'),
        (ManaColor::Red, 'R'),
        (ManaColor::Green, 'G'),
    ];
    let mut parts: Vec<String> = colors
        .iter()
        .filter_map(|(c, sym)| {
            let n = pool.amount(*c);
            if n > 0 { Some(format!("{sym}:{n}")) } else { None }
        })
        .collect();
    let cl = pool.colorless_amount();
    if cl > 0 { parts.push(format!("C:{cl}")); }
    if parts.is_empty() { "0".into() } else { parts.join(" ") }
}

#[derive(Resource)]
pub struct P1Timer(pub Timer);

/// Tracks the "targeting mode" UI state when a spell needs a player-chosen target.
#[derive(Resource, Default)]
pub struct TargetingState {
    /// Whether we're currently waiting for the player to pick a target.
    pub active: bool,
    /// The spell card the player is trying to cast.
    pub pending_card_id: Option<CardId>,
    /// Full mana cost of the pending spell (for color-aware auto-tapping).
    pub pending_cost: ManaCost,
}

/// Tracks the opening-hand mulligan phase (before the game starts).
#[derive(Resource)]
pub struct MulliganState {
    /// Whether the mulligan phase is currently active.
    pub active: bool,
    /// Whether initial 7-card hands have been dealt yet.
    pub initial_deal_done: bool,
    /// Whether player 0 has decided to keep their current hand.
    pub p0_kept: bool,
    /// Whether player 1 has decided to keep their current hand.
    pub p1_kept: bool,
    /// Number of times player 0 has mulliganed.
    pub p0_mulligans: usize,
    /// Number of times player 1 has mulliganed.
    pub p1_mulligans: usize,
    /// Timer controlling when the bot (P1) makes its mulligan decision.
    pub p1_timer: Timer,
    /// London mulligan: how many cards P0 still needs to click to send to the bottom.
    /// Non-zero means the bottoming phase is active.
    pub p0_cards_to_bottom: usize,
}

impl Default for MulliganState {
    fn default() -> Self {
        Self {
            active: true,
            initial_deal_done: false,
            p0_kept: false,
            p1_kept: false,
            p0_mulligans: 0,
            p1_mulligans: 0,
            p1_timer: Timer::from_seconds(2.0, bevy::time::TimerMode::Once),
            p0_cards_to_bottom: 0,
        }
    }
}

/// State for the graveyard card browser popup.
#[derive(Resource, Default)]
pub struct GraveyardBrowserState {
    pub open: bool,
    pub owner: usize,
}

/// Tracks player 0's blocker assignments during the DeclareBlockers step.
#[derive(Resource, Default)]
pub struct BlockingState {
    /// The player 0 creature the player clicked to block with.
    pub selected_blocker: Option<CardId>,
    /// Confirmed (blocker_id, attacker_id) assignments to submit on Pass.
    pub assignments: Vec<(CardId, CardId)>,
}

type CardFactory = fn() -> CardDefinition;

/// Build a fresh game with two players, each with a shuffled deck and 7-card opening hand.
///
/// Player 0 — Blue/White control (60 cards): Power Nine, Counterspell, Force of Will, Swords to Plowshares.
/// Player 1 — Black combo (60 cards): Dark Ritual, Demonic Tutor, Juzám Djinn, Reanimate, discard.
pub fn build_game() -> GameResource {
    let mut state = GameState::new(vec![Player::new(PLAYER_0, "Player 0"), Player::new(PLAYER_1, "Player 1")]);

    // ── Player 0: Blue/White Vintage Control (60 cards) ──────────────────────
    // Power Nine, hard permission, the best removal, and Serra Angel finishers.
    let p0_deck: &[CardFactory] = &[
        // Lands (20)
        plains, plains, plains, plains, plains, plains,
        plains, plains, plains, plains,
        island, island, island, island, island, island,
        island, island, island, island,
        // Power (6)
        black_lotus, sol_ring, mox_pearl, mox_sapphire, mox_ruby, mox_emerald,
        // Draw (6)
        ancestral_recall, ancestral_recall, ancestral_recall, ancestral_recall,
        brainstorm, brainstorm,
        // Cantrips — exercise the Scry decider (4)
        opt, opt, preordain, preordain,
        // Permission (8)
        counterspell, counterspell, counterspell, counterspell,
        force_of_will, force_of_will, force_of_will, force_of_will,
        // Removal (6)
        swords_to_plowshares, swords_to_plowshares, swords_to_plowshares, swords_to_plowshares,
        wrath_of_god, wrath_of_god,
        // Mana fixing — exercises the AddManaAnyColor decider (2)
        birds_of_paradise, birds_of_paradise,
        // Creatures (4)
        white_knight, white_knight,
        mahamoti_djinn, mahamoti_djinn,
        // Finishers (4)
        serra_angel, serra_angel, serra_angel, serra_angel,
    ];

    // ── Player 1: Black Vintage Combo (60 cards) ──────────────────────────────
    // Dark Ritual into threats, Demonic Tutor for silver bullets, reanimation package.
    let p1_deck: &[CardFactory] = &[
        // Lands (20)
        swamp, swamp, swamp, swamp, swamp, swamp,
        swamp, swamp, swamp, swamp, swamp, swamp,
        mountain, mountain, mountain, mountain,
        mountain, mountain, mountain, mountain,
        // Power (5)
        black_lotus, sol_ring, mox_jet, mox_ruby, mox_emerald,
        // Acceleration (4)
        dark_ritual, dark_ritual, dark_ritual, dark_ritual,
        // Tutors + draw (6)
        demonic_tutor, demonic_tutor, demonic_tutor, demonic_tutor,
        wheel_of_fortune, wheel_of_fortune,
        // Discard (4)
        hymn_to_tourach, hymn_to_tourach, hymn_to_tourach, hymn_to_tourach,
        // Reanimation (3)
        reanimate, reanimate, reanimate,
        // Creatures (10)
        black_knight, black_knight, black_knight, black_knight,
        hypnotic_specter, hypnotic_specter,
        juzam_djinn, juzam_djinn, juzam_djinn, juzam_djinn,
        // Removal (8)
        terror, terror, terror, terror,
        terminate, terminate, terminate, terminate,
    ];

    let mut rng = rand::rng();

    for &f in p0_deck { state.add_card_to_library(PLAYER_0, f()); }
    state.players[PLAYER_0].library.shuffle(&mut rng);

    for &f in p1_deck { state.add_card_to_library(PLAYER_1, f()); }
    state.players[PLAYER_1].library.shuffle(&mut rng);

    // Initial hands are dealt during the mulligan phase by mulligan_system.

    GameResource { state }
}
