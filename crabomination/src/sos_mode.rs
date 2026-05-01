//! Secrets of Strixhaven (SoS) format: each player rolls a Strixhaven
//! college (Lorehold, Prismari, Quandrix, Silverquill, or Witherbloom)
//! and gets a 60-card two-color deck built from that college's cards.
//!
//! Deck recipe (mirrors `cube_deck`):
//!
//! - 22 basic lands (11 of each college color)
//! - 4 "colorless" picks from the college's school land
//! - 17 cards drawn from college color A's pool
//! - 17 cards drawn from college color B's pool
//!
//! Each color pool contains:
//!
//! - Mono-color SoS cards in that color (✅-status only)
//! - The college's multi-color SoS cards (also ✅-status only)
//! - The college's school land
//!
//! Only fully-implemented (✅) SoS cards are included — see
//! `STRIXHAVEN2.md` for status tracking. The card list here is generated
//! from that file's ✅ rows by `scripts/list_sos_ok.py` /
//! `scripts/sos_ok_factory_map.py`.
//!
//! Pool sizes are smaller than the cube's, especially for Prismari
//! (12 unique cards). `sample_with_cap` may run out of legal picks
//! before hitting the requested count; the assembler tops up shortfalls
//! with extra basic lands so every deck reaches 60 cards.

use std::collections::HashMap;

use rand::seq::SliceRandom;
use rand::{Rng, RngExt};

use crate::card::CardDefinition;
use crate::catalog::*;
use crate::game::GameState;
use crate::mana::Color;
use crate::player::Player;

type CardFactory = fn() -> CardDefinition;

const COPY_CAP: u32 = 4;
const BASICS_PER_COLOR: usize = 11;
const COLORLESS_COUNT: usize = 4;
const CARDS_PER_COLOR: usize = 17;

/// One of the five Strixhaven schools. Each has a fixed color pair, a
/// signature multi-color card pool, and a school land that taps for either
/// of its colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum College {
    Lorehold,    // Red / White
    Prismari,    // Blue / Red
    Quandrix,    // Green / Blue
    Silverquill, // White / Black
    Witherbloom, // Black / Green
}

impl College {
    pub fn colors(self) -> [Color; 2] {
        match self {
            College::Lorehold => [Color::Red, Color::White],
            College::Prismari => [Color::Blue, Color::Red],
            College::Quandrix => [Color::Green, Color::Blue],
            College::Silverquill => [Color::White, Color::Black],
            College::Witherbloom => [Color::Black, Color::Green],
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            College::Lorehold => "Lorehold",
            College::Prismari => "Prismari",
            College::Quandrix => "Quandrix",
            College::Silverquill => "Silverquill",
            College::Witherbloom => "Witherbloom",
        }
    }

    /// The school land for this college. Taps for either of the college's
    /// two colors and has a `{2}{c1}{c2},{T}: Surveil 1` activated ability.
    pub fn school_land(self) -> CardFactory {
        match self {
            College::Lorehold => fields_of_strife,
            College::Prismari => spectacle_summit,
            College::Quandrix => paradox_gardens,
            College::Silverquill => forum_of_amity,
            College::Witherbloom => titans_grave,
        }
    }

    /// All ✅ multi-color cards belonging to this college (the cards whose
    /// color identity is exactly this college's two-color pair).
    pub fn multi_pool(self) -> Vec<CardFactory> {
        match self {
            // Lorehold (R/W). + Hardened Academic, Spirit Mascot,
            // Garrison Excavator now ✅ via CardLeftGraveyard event.
            // + Ark of Hunger (CardLeftGraveyard drain + Mill activation),
            // + Wilt in the Heat (5-to-creature removal).
            College::Lorehold => vec![
                lorehold_charm,
                lorehold_the_historian,
                startled_relic_sloth,
                hardened_academic,
                spirit_mascot,
                ark_of_hunger,
                wilt_in_the_heat,
            ],
            // ↑ Fields of Strife is the school land, included via school_land();
            //   keep it out of the multi pool to avoid double-counting at cap time.

            // Prismari (U/R). + Spectacular Skywhale (1/4 Flying body),
            // Resonating Lute (gated draw activation; lands-grant
            // omitted), + Prismari Charm (3-mode surveil/burn/bounce).
            College::Prismari => vec![
                prismari_charm,
                rapturous_moment,
                resonating_lute,
                spectacular_skywhale,
                splatter_technique,
                traumatic_critique,
            ],
            // ↑ Spectacle Summit is the school land.

            // Quandrix (G/U). + Berta, Wise Extrapolator (counter-add
            // mana ability + X-cost Fractal token activation), Paradox
            // Surveyor (3/3 reach + ETB reveal-5).
            College::Quandrix => vec![
                berta_wise_extrapolator,
                fractal_mascot,
                fractal_tender,
                growth_curve,
                paradox_surveyor,
                proctors_gaze,
            ],
            // ↑ Paradox Gardens is the school land.

            // Silverquill (W/B).
            College::Silverquill => vec![
                imperious_inkmage,
                inkling_mascot,
                silverquill_charm,
                snooping_page,
                stirring_honormancer,
            ],
            // ↑ Forum of Amity is the school land.

            // Witherbloom (B/G). + Witherbloom, the Balancer (body-only,
            // late-game finisher), + Essenceknit Scholar (Pest token +
            // creature-died end-step draw), + Professor Dellian Fel
            // (planeswalker — 3 abilities; ult emblem omitted).
            College::Witherbloom => vec![
                blech_loafing_pest,
                bogwater_lumaret,
                cauldron_of_essence,
                essenceknit_scholar,
                grapple_with_death,
                lluwen_exchange_student,
                old_growth_educator,
                pest_mascot,
                professor_dellian_fel,
                vicious_rivalry,
                witherbloom_the_balancer,
            ],
            // ↑ Titan's Grave is the school land.
        }
    }
}

/// Build a fresh SoS match: two seats, each rolled to a random college
/// with a 60-card deck and 7 cards drawn into the opening hand. Both seats
/// are flagged `wants_ui` so all decisions surface as `pending_decision`
/// for UI/bot handling.
pub fn build_sos_state() -> GameState {
    let mut rng = rand::rng();
    let mut state = GameState::new(vec![Player::new(0, "P0"), Player::new(1, "P1")]);

    let p0_college = random_college(&mut rng);
    let p1_college = random_college(&mut rng);

    state.players[0].name = format!("P0 ({})", p0_college.name());
    state.players[1].name = format!("P1 ({})", p1_college.name());

    let p0_deck = sos_deck(p0_college, &mut rng);
    let p1_deck = sos_deck(p1_college, &mut rng);

    for &f in &p0_deck {
        state.add_card_to_library(0, f());
    }
    state.players[0].library.shuffle(&mut rng);
    for &f in &p1_deck {
        state.add_card_to_library(1, f());
    }
    state.players[1].library.shuffle(&mut rng);

    state.players[0].wants_ui = true;
    state.players[1].wants_ui = true;
    state
}

/// Pick one of the five colleges uniformly at random.
pub fn random_college<R: Rng>(rng: &mut R) -> College {
    const ALL: [College; 5] = [
        College::Lorehold,
        College::Prismari,
        College::Quandrix,
        College::Silverquill,
        College::Witherbloom,
    ];
    ALL[rng.random_range(0..ALL.len())]
}

/// Assemble a 60-card SoS deck for the given college.
pub fn sos_deck<R: Rng>(college: College, rng: &mut R) -> Vec<CardFactory> {
    let mut deck: Vec<CardFactory> = Vec::with_capacity(60);
    let mut counts: HashMap<usize, u32> = HashMap::new();

    let [c0, c1] = college.colors();

    // 22 basic lands.
    for &c in &[c0, c1] {
        let basic = basic_factory(c);
        for _ in 0..BASICS_PER_COLOR {
            deck.push(basic);
        }
    }

    // 4 "colorless" slots filled with the college's school land. The school
    // land is colorless for cost-purposes (no mana cost) so it slots into
    // the cube's colorless bucket.
    let school_pool = vec![college.school_land()];
    sample_with_cap(
        &mut deck,
        &mut counts,
        &school_pool,
        COLORLESS_COUNT,
        rng,
    );

    // 17 cards from each color's SoS pool.
    let pool0 = sos_color_pool(c0, college);
    let pool1 = sos_color_pool(c1, college);
    sample_with_cap(&mut deck, &mut counts, &pool0, CARDS_PER_COLOR, rng);
    sample_with_cap(&mut deck, &mut counts, &pool1, CARDS_PER_COLOR, rng);

    // Top up any shortfall with extra basics (Prismari's 12-unique pool can
    // run out of legal picks before reaching 17 per color when the multi-
    // college cards hit the global cap on the first sample).
    let target = 22 + COLORLESS_COUNT + 2 * CARDS_PER_COLOR;
    if deck.len() < target {
        let shortfall = target - deck.len();
        let basic_a = basic_factory(c0);
        let basic_b = basic_factory(c1);
        for i in 0..shortfall {
            deck.push(if i % 2 == 0 { basic_a } else { basic_b });
        }
    }

    deck
}

fn basic_factory(c: Color) -> CardFactory {
    match c {
        Color::White => plains,
        Color::Blue => island,
        Color::Black => swamp,
        Color::Red => mountain,
        Color::Green => forest,
    }
}

/// Sample `count` cards from `pool` (with replacement, capped at
/// `COPY_CAP` per card globally) and append them to `deck`. The shared
/// `counts` map is keyed by factory function-pointer address so a card
/// sampled out of two different pools (multi-college cards) still counts
/// once toward the cap.
fn sample_with_cap<R: Rng>(
    deck: &mut Vec<CardFactory>,
    counts: &mut HashMap<usize, u32>,
    pool: &[CardFactory],
    count: usize,
    rng: &mut R,
) {
    if pool.is_empty() {
        return;
    }
    let mut picks = 0;
    let max_attempts = count.saturating_mul(20).max(40);
    let mut attempts = 0;
    while picks < count && attempts < max_attempts {
        attempts += 1;
        let idx = rng.random_range(0..pool.len());
        let factory_addr = pool[idx] as usize;
        let entry = counts.entry(factory_addr).or_insert(0);
        if *entry >= COPY_CAP {
            continue;
        }
        *entry += 1;
        deck.push(pool[idx]);
        picks += 1;
    }
}

/// Every card factory that can appear in any SoS deck. Used by the client
/// at startup to prefetch Scryfall art for the full SoS card universe (the
/// per-match deck is randomly rolled after assets are loaded).
pub fn all_sos_cards() -> Vec<CardFactory> {
    use std::collections::HashSet;
    let mut all: Vec<CardFactory> = vec![plains, island, swamp, mountain, forest];
    for college in [
        College::Lorehold,
        College::Prismari,
        College::Quandrix,
        College::Silverquill,
        College::Witherbloom,
    ] {
        all.push(college.school_land());
        all.extend(college.multi_pool());
        for &c in &college.colors() {
            all.extend(mono_color_pool(c));
        }
    }
    let mut seen: HashSet<usize> = HashSet::new();
    all.retain(|f| seen.insert(*f as usize));
    all
}

/// Build the per-color SoS pool for a given college. Combines the
/// mono-color ✅ cards in `target` with the college's multi-color ✅ cards
/// and the college's school land.
fn sos_color_pool(target: Color, college: College) -> Vec<CardFactory> {
    let mut v = mono_color_pool(target);
    v.extend(college.multi_pool());
    v.push(college.school_land());
    v
}

/// Mono-color ✅ SoS cards in `c`. Sourced from `STRIXHAVEN2.md`'s ✅ rows
/// in the matching color section.
fn mono_color_pool(c: Color) -> Vec<CardFactory> {
    match c {
        // White — added Practiced Offense (a +1/+1 fan-out + double-strike pump),
        // Daydream (exile + return with +1/+1 counter), Soaring Stoneglider
        // (4/3 flying-vigilance Elephant Cleric).
        Color::White => vec![
            ascendant_dustspeaker,
            daydream,
            eager_glyphmage,
            ennis_debate_moderator,
            graduation_day,
            informed_inkwright,
            interjection,
            practiced_offense,
            primary_research,
            rapier_wit,
            rehearsed_debater,
            restoration_seminar,
            shattered_acolyte,
            soaring_stoneglider,
            stand_up_for_yourself,
            stirring_hopesinger,
            // Push IX additions:
            stone_docent,
            // Push X additions:
            inkshape_demonstrator,
        ],
        // Blue (8) — added Mana Sculpt (counter + mana refund),
        // Homesickness (6-mana draw 2 + tap+stun), Fractalize (X-cost
        // pump-up to base (X+1)/(X+1)), Divergent Equation (X-cost gy
        // recursion).
        Color::Blue => vec![
            banishing_betrayal,
            chase_inspiration,
            divergent_equation,
            fractal_anomaly,
            fractalize,
            homesickness,
            mana_sculpt,
            procrastinate,
            // Push IX additions:
            deluge_virtuoso,
            flow_state,
            muses_encouragement,
            textbook_tabulator,
        ],
        // Black — added Rabid Attack (+1/+0 friendly pump),
        // Decorum Dissertation (Lesson: draw 2 lose 2),
        // Tragedy Feaster (7/6 trample body), Forum Necroscribe
        // (5/4 Repartee gy-recursion body).
        Color::Black => vec![
            arcane_omens,
            arnyn_deathbloom_botanist,
            burrog_banemaker,
            decorum_dissertation,
            foolish_fate,
            forum_necroscribe,
            lecturing_scornmage,
            masterful_flourish,
            melancholic_poet,
            rabid_attack,
            sneering_shadewriter,
            tragedy_feaster,
            wander_off,
            withering_curse,
            // Push IX additions:
            moseo_veins_new_dean,
            ral_zarek_guest_lecturer,
        ],
        // Red — added Tablet of Discovery (mill+R mana), Garrison
        // Excavator (Spirit-token graveyard payoff), Living History
        // (Spirit-token + on-attack pump), Steal the Show (modal
        // discard/draw or IS-graveyard damage), Tome Blast (2-to-any-
        // target burn), Duel Tactics (1-to-creature + grants CantBlock
        // until EOT via the new keyword), Rubble Rouser (rummage ETB).
        Color::Red => vec![
            artistic_process,
            charging_strifeknight,
            duel_tactics,
            garrison_excavator,
            living_history,
            magmablood_archaic,
            rearing_embermare,
            rubble_rouser,
            steal_the_show,
            tablet_of_discovery,
            tome_blast,
            zealous_lorecaster,
            // Push IX additions:
            unsubtle_mockery,
        ],
        // Green — added Burrog Barrage (conditional pump + power damage),
        // Chelonian Tackle (+0/+10 + power damage), Snarl Song (Converge
        // Fractal tokens + life), Wild Hypothesis (X+1/+1 Fractal +
        // Surveil 2), Topiary Lecturer (now uses ManaPayload::OfColor),
        // Additive Evolution (Fractal-with-counters ETB + combat pump),
        // Zimone's Experiment (creature-or-land smoothing).
        Color::Green => vec![
            additive_evolution,
            ambitious_augmenter,
            burrog_barrage,
            chelonian_tackle,
            efflorescence,
            environmental_scientist,
            germination_practicum,
            glorious_decay,
            mindful_biomancer,
            noxious_newt,
            oracles_restoration,
            planar_engineering,
            shopkeepers_bane,
            slumbering_trudge,
            snarl_song,
            topiary_lecturer,
            wild_hypothesis,
            wildgrowth_archaic,
            zimones_experiment,
            // Push X additions:
            studious_first_year,
            thornfist_striker,
            lumarets_favor,
            // Push XV additions:
            follow_the_lumarets,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_college_is_one_of_five() {
        let mut rng = rand::rng();
        for _ in 0..50 {
            let college = random_college(&mut rng);
            // Just exercise the .colors() / .name() / .school_land() path.
            let [a, b] = college.colors();
            assert_ne!(a, b);
            assert!(!college.name().is_empty());
            let _ = college.school_land()();
        }
    }

    #[test]
    fn sos_deck_is_always_sixty_cards_for_every_college() {
        let mut rng = rand::rng();
        for college in [
            College::Lorehold,
            College::Prismari,
            College::Quandrix,
            College::Silverquill,
            College::Witherbloom,
        ] {
            for _ in 0..10 {
                let deck = sos_deck(college, &mut rng);
                assert_eq!(deck.len(), 60, "{:?} deck must be 60 cards", college);
            }
        }
    }

    #[test]
    fn sos_deck_obeys_copy_cap_for_non_basics() {
        // Each individual non-basic card capped at COPY_CAP. Basics are
        // exempt (decks are heavily basic-padded for small colleges).
        let basics = [plains, island, swamp, mountain, forest];
        let basic_addrs: std::collections::HashSet<usize> =
            basics.iter().map(|f| *f as usize).collect();
        let mut rng = rand::rng();
        for college in [
            College::Lorehold,
            College::Prismari,
            College::Quandrix,
            College::Silverquill,
            College::Witherbloom,
        ] {
            for _ in 0..10 {
                let deck = sos_deck(college, &mut rng);
                let mut counts: HashMap<usize, u32> = HashMap::new();
                for &f in &deck {
                    *counts.entry(f as usize).or_insert(0) += 1;
                }
                for (addr, n) in counts {
                    if basic_addrs.contains(&addr) {
                        continue;
                    }
                    assert!(
                        n <= COPY_CAP,
                        "{:?}: card at {:#x} has {} copies (cap {})",
                        college,
                        addr,
                        n,
                        COPY_CAP
                    );
                }
            }
        }
    }

    #[test]
    fn sos_deck_only_uses_chosen_college_colors_and_basics() {
        // Every non-basic card in a Prismari deck should be either mono
        // U / mono R / multi UR / Spectacle Summit. Verify by checking
        // that the deck is a subset of the union of Prismari's pools +
        // basics.
        let mut rng = rand::rng();
        for college in [
            College::Lorehold,
            College::Prismari,
            College::Quandrix,
            College::Silverquill,
            College::Witherbloom,
        ] {
            let [c0, c1] = college.colors();
            let mut allowed: std::collections::HashSet<usize> = Default::default();
            for &c in &[c0, c1] {
                for f in mono_color_pool(c) {
                    allowed.insert(f as usize);
                }
            }
            for f in college.multi_pool() {
                allowed.insert(f as usize);
            }
            allowed.insert(college.school_land() as usize);
            for &f in &[plains, island, swamp, mountain, forest] {
                allowed.insert(f as usize);
            }

            let deck = sos_deck(college, &mut rng);
            for &f in &deck {
                assert!(
                    allowed.contains(&(f as usize)),
                    "{:?}: deck contains an out-of-pool card",
                    college
                );
            }
        }
    }

    #[test]
    fn build_sos_state_succeeds() {
        let state = build_sos_state();
        assert_eq!(state.players.len(), 2);
        // Both players have a full deck minus the opening 7.
        for p in &state.players {
            assert!(p.wants_ui);
            assert_eq!(p.library.len() + p.hand.len(), 60);
        }
    }

    #[test]
    fn all_sos_cards_includes_every_factory_used_by_the_pools() {
        let all: std::collections::HashSet<usize> =
            all_sos_cards().into_iter().map(|f| f as usize).collect();
        // Spot-check: school lands + a handful of pool cards.
        let must_include: Vec<CardFactory> = vec![
            // Each school land
            fields_of_strife, spectacle_summit, paradox_gardens, forum_of_amity, titans_grave,
            // One ✅ card from each color pool
            ascendant_dustspeaker,        // White
            chase_inspiration,            // Blue
            withering_curse,              // Black
            charging_strifeknight,        // Red
            efflorescence,                // Green
            // One multi-college card from each college
            startled_relic_sloth,         // Lorehold
            traumatic_critique,           // Prismari
            growth_curve,                 // Quandrix
            silverquill_charm,            // Silverquill
            cauldron_of_essence,          // Witherbloom
        ];
        for f in must_include {
            assert!(
                all.contains(&(f as usize)),
                "all_sos_cards missed a factory used by the pools"
            );
        }
    }
}
