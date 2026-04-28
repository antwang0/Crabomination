//! Cube format: random two-color decks built from a curated card pool.
//!
//! `build_cube_state()` rolls a random color pair per seat, then assembles
//! a 60-card deck with the recipe:
//!
//! - 22 basic lands (11 of each color)
//! - 4 colorless utility artifacts
//! - 17 cards drawn from the first color's pool
//! - 17 cards drawn from the second color's pool
//!
//! Each color pool includes mono-color cube staples plus any two-color
//! cards whose other color is also part of the chosen pair (so a UR pair
//! gets `stormchaser_mage`, an RB pair gets `terminate`, etc.). Sampling
//! is with replacement but capped at four copies per card, matching
//! Modern's deck-construction rule.
//!
//! Card pools are hand-curated below — no attempt at "balance" beyond
//! making sure each color has enough cards to fill 17 picks. The point
//! is variety, not tournament-grade decks.

use std::collections::HashMap;

use rand::{Rng, RngExt};
use rand::seq::SliceRandom;

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

/// Build a fresh cube match: two seats, each with a random 2-color deck
/// and 7 cards drawn into the opening hand. Both seats are flagged
/// `wants_ui` so all decisions surface as `pending_decision` for UI/bot
/// handling.
pub fn build_cube_state() -> GameState {
    let mut rng = rand::rng();
    let mut state = GameState::new(vec![Player::new(0, "P0"), Player::new(1, "P1")]);

    let p0_colors = random_color_pair(&mut rng);
    let p1_colors = random_color_pair(&mut rng);

    state.players[0].name = format!("P0 ({})", color_pair_name(p0_colors));
    state.players[1].name = format!("P1 ({})", color_pair_name(p1_colors));

    let p0_deck = cube_deck(p0_colors, &mut rng);
    let p1_deck = cube_deck(p1_colors, &mut rng);

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

/// Pick two distinct colors uniformly at random from {W, U, B, R, G}.
pub fn random_color_pair<R: Rng>(rng: &mut R) -> [Color; 2] {
    let colors = [
        Color::White, Color::Blue, Color::Black, Color::Red, Color::Green,
    ];
    let i = rng.random_range(0..colors.len());
    let mut j = rng.random_range(0..colors.len() - 1);
    if j >= i {
        j += 1;
    }
    [colors[i], colors[j]]
}

/// Two-letter guild-style abbreviation for a color pair (UW, UB, etc.).
/// Used in player names so the UI can show "P0 (UR)" at a glance.
pub fn color_pair_name(colors: [Color; 2]) -> String {
    fn ch(c: Color) -> char {
        match c {
            Color::White => 'W',
            Color::Blue => 'U',
            Color::Black => 'B',
            Color::Red => 'R',
            Color::Green => 'G',
        }
    }
    format!("{}{}", ch(colors[0]), ch(colors[1]))
}

/// Assemble a 60-card cube deck for the given color pair.
pub fn cube_deck<R: Rng>(colors: [Color; 2], rng: &mut R) -> Vec<CardFactory> {
    let mut deck: Vec<CardFactory> = Vec::with_capacity(60);
    // Single counts map shared across all `sample_with_cap` calls so a
    // two-color card that appears in both color pools (e.g. `terminate`
    // for a Black-Red pair) is capped at four total — not four per pool.
    let mut counts: HashMap<usize, u32> = HashMap::new();

    // 22 basic lands.
    for &c in &colors {
        let basic = basic_factory(c);
        for _ in 0..BASICS_PER_COLOR {
            deck.push(basic);
        }
    }

    // Colorless utility artifacts — capped to one or two copies typically
    // because the pool is small.
    sample_with_cap(&mut deck, &mut counts, &colorless_pool(), COLORLESS_COUNT, rng);

    // Per-color picks.
    sample_with_cap(&mut deck, &mut counts, &color_pool(colors[0], colors), CARDS_PER_COLOR, rng);
    sample_with_cap(&mut deck, &mut counts, &color_pool(colors[1], colors), CARDS_PER_COLOR, rng);

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
/// `counts` map is keyed by factory function-pointer address so the same
/// card sampled out of two different color pools still counts once
/// toward the cap.
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
    // Worst case: every roll lands on a maxed-out card. Cap attempts so a
    // tiny pool already at the cap doesn't loop forever.
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

/// Every card factory that can appear in any cube deck across any color
/// pair. Used by the client at startup to prefetch Scryfall art for the
/// full cube card universe (since the per-match deck is randomly rolled
/// after assets are loaded). The union covers basics + colorless +
/// each color's pool with every other color paired in.
pub fn all_cube_cards() -> Vec<CardFactory> {
    use std::collections::HashSet;
    let mut all: Vec<CardFactory> = Vec::new();
    all.push(plains);
    all.push(island);
    all.push(swamp);
    all.push(mountain);
    all.push(forest);
    all.extend(colorless_pool());
    let colors = [
        Color::White, Color::Blue, Color::Black, Color::Red, Color::Green,
    ];
    for &a in &colors {
        for &b in &colors {
            if a == b {
                continue;
            }
            all.extend(color_pool(a, [a, b]));
        }
    }
    // Dedupe by function-pointer address — same card may appear in
    // multiple pools (two-color cards) and the loop above hits each
    // mono-color list once per partner.
    let mut seen: HashSet<usize> = HashSet::new();
    all.retain(|f| seen.insert(*f as usize));
    all
}

/// Cards usable in any color combination — colorless utility artifacts
/// that don't care about the deck's color identity.
fn colorless_pool() -> Vec<CardFactory> {
    vec![
        sol_ring,
        ornithopter,
        ornithopter_of_paradise,
        mind_stone,
        fellwar_stone,
        millstone,
        aether_spellbomb,
        damping_sphere,
        zuran_orb,
        chromatic_star,
        soul_guide_lantern,
    ]
}

/// Cards available for a given color, including any two-color cards
/// whose second color is also in the chosen pair.
fn color_pool(target: Color, pair: [Color; 2]) -> Vec<CardFactory> {
    match target {
        Color::White => white_pool(pair),
        Color::Blue => blue_pool(pair),
        Color::Black => black_pool(pair),
        Color::Red => red_pool(pair),
        Color::Green => green_pool(pair),
    }
}

fn pair_contains(pair: [Color; 2], c: Color) -> bool {
    pair[0] == c || pair[1] == c
}

fn white_pool(pair: [Color; 2]) -> Vec<CardFactory> {
    let mut v: Vec<CardFactory> = vec![
        savannah_lions,
        white_knight,
        swords_to_plowshares,
        path_to_exile,
        disenchant,
        isolate,
        restoration_angel,
        flickerwisp,
        loran_of_the_third_path,
        cathar_commando,
        ephemerate,
        glorious_anthem,
        serra_angel,
        thalia_guardian_of_thraben,
        ranger_captain_of_eos,
        wrath_of_god,
    ];
    if pair_contains(pair, Color::Green) {
        v.push(watchwolf);
    }
    if pair_contains(pair, Color::Red) {
        v.push(lightning_helix);
    }
    if pair_contains(pair, Color::Black) {
        v.push(mourning_thrull);
    }
    if pair_contains(pair, Color::Blue) {
        v.push(teferi_time_raveler);
        v.push(dovins_veto);
    }
    v
}

fn blue_pool(pair: [Color; 2]) -> Vec<CardFactory> {
    let mut v: Vec<CardFactory> = vec![
        counterspell,
        mana_leak,
        spell_pierce,
        negate,
        dispel,
        daze,
        swan_song,
        spell_snare,
        mystical_dispute,
        brainstorm,
        preordain,
        opt,
        consider,
        thought_scour,
        ancestral_recall,
        frantic_search,
        paradoxical_outcome,
        upheaval,
        force_of_will,
        force_of_negation,
        consign_to_memory,
        pact_of_negation,
        mahamoti_djinn,
        prodigal_sorcerer,
        quantum_riddler,
        tishanas_tidebinder,
    ];
    if pair_contains(pair, Color::Green) {
        v.push(gaeas_skyfolk);
    }
    if pair_contains(pair, Color::Red) {
        v.push(stormchaser_mage);
    }
    if pair_contains(pair, Color::White) {
        v.push(teferi_time_raveler);
        v.push(dovins_veto);
    }
    v
}

fn black_pool(pair: [Color; 2]) -> Vec<CardFactory> {
    let mut v: Vec<CardFactory> = vec![
        terror,
        doom_blade,
        fatal_push,
        dark_ritual,
        demonic_tutor,
        thoughtseize,
        inquisition_of_kozilek,
        nights_whisper,
        spoils_of_the_vault,
        rakshasas_bargain,
        disentomb,
        reanimate,
        bone_shards,
        animate_dead,
        hymn_to_tourach,
        hypnotic_specter,
        sengir_vampire,
        juzam_djinn,
        black_knight,
        slaughter_pact,
        drown_in_ichor,
        fell,
        blasphemous_edict,
        griselbrand,
        dark_confidant,
        bloodghast,
        phyrexian_arena,
    ];
    if pair_contains(pair, Color::Red) {
        v.push(terminate);
        v.push(voldaren_epicure);
        v.push(bloodtithe_harvester);
    }
    if pair_contains(pair, Color::White) {
        v.push(mourning_thrull);
    }
    v
}

fn red_pool(pair: [Color; 2]) -> Vec<CardFactory> {
    let mut v: Vec<CardFactory> = vec![
        lightning_bolt,
        shock,
        tarfire,
        shivan_dragon,
        goblin_guide,
        pact_of_the_titan,
        vandalblast,
        big_score,
        callous_sell_sword,
        blasphemous_act,
        anger_of_the_gods,
        goldspan_dragon,
        sundering_eruption,
        grim_lavamancer,
        pyrokinesis,
    ];
    if pair_contains(pair, Color::White) {
        v.push(lightning_helix);
    }
    if pair_contains(pair, Color::Black) {
        v.push(terminate);
        v.push(voldaren_epicure);
        v.push(bloodtithe_harvester);
    }
    if pair_contains(pair, Color::Green) {
        v.push(ghor_clan_rampager);
    }
    if pair_contains(pair, Color::Blue) {
        v.push(stormchaser_mage);
    }
    v
}

fn green_pool(pair: [Color; 2]) -> Vec<CardFactory> {
    let mut v: Vec<CardFactory> = vec![
        llanowar_elves,
        giant_growth,
        grizzly_bears,
        elvish_archer,
        craw_wurm,
        birds_of_paradise,
        sylvan_caryatid,
        summoners_pact,
        natures_claim,
        natures_lore,
        blossoming_defense,
        tireless_tracker,
        sentinel_of_the_nameless_city,
        haywire_mite,
        up_the_beanstalk,
        cosmogoyf,
        naturalize,
        sylvan_safekeeper,
        cankerbloom,
    ];
    if pair_contains(pair, Color::White) {
        v.push(watchwolf);
    }
    if pair_contains(pair, Color::Red) {
        v.push(ghor_clan_rampager);
    }
    if pair_contains(pair, Color::Blue) {
        v.push(gaeas_skyfolk);
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_color_pair_yields_two_distinct_colors() {
        let mut rng = rand::rng();
        for _ in 0..100 {
            let pair = random_color_pair(&mut rng);
            assert_ne!(pair[0], pair[1]);
        }
    }

    #[test]
    fn color_pair_name_is_two_chars() {
        let n = color_pair_name([Color::Blue, Color::Red]);
        assert_eq!(n, "UR");
        let n = color_pair_name([Color::Green, Color::White]);
        assert_eq!(n, "GW");
    }

    #[test]
    fn cube_deck_has_60_cards_with_basics_for_both_colors() {
        let mut rng = rand::rng();
        for _ in 0..20 {
            let pair = random_color_pair(&mut rng);
            let deck = cube_deck(pair, &mut rng);
            assert_eq!(deck.len(), 60, "cube deck must be 60 cards");
            // Eleven basics of each color present.
            let basic_a = basic_factory(pair[0]);
            let basic_b = basic_factory(pair[1]);
            let count_a = deck.iter().filter(|f| **f as usize == basic_a as usize).count();
            let count_b = deck.iter().filter(|f| **f as usize == basic_b as usize).count();
            assert_eq!(count_a, BASICS_PER_COLOR, "{:?} basics missing", pair[0]);
            assert_eq!(count_b, BASICS_PER_COLOR, "{:?} basics missing", pair[1]);
        }
    }

    #[test]
    fn cube_deck_respects_four_copy_cap_for_non_basics() {
        let mut rng = rand::rng();
        for _ in 0..20 {
            let pair = random_color_pair(&mut rng);
            let deck = cube_deck(pair, &mut rng);
            // Group factory pointers by address and verify non-basic counts ≤ 4.
            let basic_a = basic_factory(pair[0]) as usize;
            let basic_b = basic_factory(pair[1]) as usize;
            let mut counts: HashMap<usize, u32> = HashMap::new();
            for &f in &deck {
                *counts.entry(f as usize).or_insert(0) += 1;
            }
            for (&addr, &count) in &counts {
                if addr == basic_a || addr == basic_b {
                    continue;
                }
                assert!(count <= COPY_CAP,
                    "non-basic card exceeds 4-copy cap: count={count}");
            }
        }
    }

    #[test]
    fn all_cube_cards_includes_basics_and_representative_color_picks() {
        let names: Vec<&'static str> =
            all_cube_cards().into_iter().map(|f| f().name).collect();
        // Five basics (one of each color).
        for basic in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            assert!(names.contains(&basic), "missing basic: {basic}");
        }
        // A few representative cube cards across colors. If any of these
        // disappears from the pool, the asset-prefetch will silently skip
        // its art.
        for name in [
            "Disentomb",
            "Tireless Tracker",
            "Goldspan Dragon",
            "Cathar Commando",
            "Mind Stone",
            "Up the Beanstalk",
        ] {
            assert!(names.contains(&name),
                "expected `{name}` in cube prefetch pool");
        }
    }

    #[test]
    fn all_cube_cards_is_deduplicated() {
        let cards = all_cube_cards();
        let mut seen: HashMap<usize, u32> = HashMap::new();
        for f in &cards {
            *seen.entry(*f as usize).or_insert(0) += 1;
        }
        for (addr, n) in &seen {
            assert_eq!(*n, 1, "factory {addr:#x} duplicated {n} times in pool");
        }
    }

    #[test]
    fn build_cube_state_seats_two_players_with_libraries() {
        let state = build_cube_state();
        assert_eq!(state.players.len(), 2);
        // Each library holds 60 cards.
        assert_eq!(state.players[0].library.len(), 60);
        assert_eq!(state.players[1].library.len(), 60);
        // Player names carry their color pair tag.
        assert!(state.players[0].name.contains('('));
        assert!(state.players[1].name.contains('('));
    }
}
