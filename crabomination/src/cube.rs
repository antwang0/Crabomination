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
    format!("{}{}", colors[0].short_name(), colors[1].short_name())
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
    let mut all: Vec<CardFactory> = vec![plains, island, swamp, mountain, forest];
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
/// that don't care about the deck's color identity. Wasteland and Strip
/// Mine are colorless utility *lands* but slot here since they're
/// universally useful and the deck-builder treats `colorless_pool` as the
/// "always-available" bucket.
fn colorless_pool() -> Vec<CardFactory> {
    vec![
        sol_ring,
        ornithopter,
        ornithopter_of_paradise,
        mind_stone,
        // fellwar_stone — 🟡 "matches opponent's lands" restriction omitted
        millstone,
        aether_spellbomb,
        damping_sphere,
        zuran_orb,
        chromatic_star,
        soul_guide_lantern,
        wasteland,
        strip_mine,
        // ── Modern Locus / fetch / utility lands ──
        evolving_wilds,
        glimmerpost,
        cloudpost,
        lotus_field,
        // ── Modern utility artifacts ──
        // coalition_relic — 🟡 charge-counter burst ability omitted
        ghost_vacuum,
        krark_clan_ironworks,
        // karn_scion_of_urza — 🟡 artifact-count scaling and ult omitted
        // pentad_prism     — 🟡 Sunburst collapses to flat 2 counters
        // ── modern_decks-14 ──
        mortuary_mire,
        geier_reach_sanitarium,
        // ── SOS (Secrets of Strixhaven) — colorless ──
        rancorous_archaic,
        together_as_one,
        diary_of_dreams,
        potioners_trove,
        // ── SOS push XV ──
        great_hall_of_the_biblioplex,
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
        // ranger_captain_of_eos — 🟡 sac-for-no-noncreature-spells static omitted
        wrath_of_god,
        // heliod_sun_crowned  — 🟡 devotion-based creature/enchantment toggle omitted
        // containment_priest  — 🟡 ETB-replacement effect for non-cast creatures omitted
        // static_prison       — 🟡 "while it has stun counters don't untap" suppression omitted
        day_of_judgment,
        enlightened_tutor,
        healing_salve,
        raise_the_alarm,
        spectral_procession,
        lay_down_arms,
        holy_light,
        mana_tithe,
        path_of_peace,
        vryn_wingmare,
        rout,
        celestial_purge,
        // ── modern_decks-15 ──
        bond_of_discipline,
        // ── SOS (Secrets of Strixhaven) ──
        eager_glyphmage,
        erode,
        harsh_annotation,
        interjection,
        stand_up_for_yourself,
        rapier_wit,
        stirring_hopesinger,
        rehearsed_debater,
        informed_inkwright,
        ascendant_dustspeaker,
        shattered_acolyte,
        summoned_dromedary,
        dig_site_inventory,
        group_project,
        owlin_historian,
        graduation_day,
        antiquities_on_the_loose,
        ajanis_response,
        // ── SOS push XI: White MDFCs ──
        elite_interceptor,
        emeritus_of_truce,
        honorbound_page,
        joined_researchers,
        quill_blade_laureate,
        spiritcall_enthusiast,
    ];
    if pair_contains(pair, Color::Green) {
        v.push(watchwolf);
        v.push(thornglint_bridge);
        v.push(lush_portico);
    }
    if pair_contains(pair, Color::Red) {
        v.push(lightning_helix);
        v.push(talisman_of_conviction);
        v.push(rustvale_bridge);
        v.push(commercial_district);
        v.push(fields_of_strife);
        v.push(pursue_the_past);
        // ── SOS Lorehold (R/W) ──
        v.push(startled_relic_sloth);
        v.push(hardened_academic);
        v.push(borrowed_knowledge);
        v.push(lorehold_charm);
        v.push(kirol_history_buff);
    }
    if pair_contains(pair, Color::Black) {
        v.push(mourning_thrull);
        v.push(goldmire_bridge);
        v.push(mortify);
        // ── modern_decks-14 (WB cross-pool removal) ──
        v.push(vindicate);
        v.push(anguished_unmaking);
        v.push(despark);
        // ── SOS Silverquill (W/B) ──
        v.push(silverquill_charm);
        v.push(killians_confidence);
        v.push(imperious_inkmage);
        v.push(forum_of_amity);
        v.push(render_speechless);
        v.push(snooping_page);
        v.push(inkling_mascot);
        v.push(moment_of_reckoning);
        v.push(stirring_honormancer);
        v.push(conciliators_duelist);
        v.push(scolding_administrator);
        v.push(abigale_poet_laureate);
    }
    if pair_contains(pair, Color::Blue) {
        v.push(teferi_time_raveler);
        v.push(dovins_veto);
        v.push(razortide_bridge);
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
        // daze           — 🟡 "return an Island" alt-cost omitted
        // swan_song      — 🟡 Bird token goes to caster not correct player in 2-player
        spell_snare,
        mystical_dispute,
        brainstorm,
        preordain,
        opt,
        consider,
        thought_scour,
        ancestral_recall,
        // frantic_search — ✅ "up to three" untap honored via
        // `Effect::Untap.up_to`. Disabled in cube pending color-balance
        // pass on the new 3-cap version.
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
        // cryptic_command — 🟡 "choose two" collapsed to bundled pairs
        mystical_tutor,
        // dandan         — 🟡 "can attack only if defending player controls an Island" omitted
        turnabout,
        remand,
        storm_crow,
        cancel,
        annul,
        unsummon,
        boomerang,
        cyclonic_rift,
        repeal,
        anticipate,
        divination,
        concentrate,
        telling_time,
        read_the_tides,
        hieroglyphic_illumination,
        // ── modern_decks-15 ──
        tome_scour,
        repulse,
        visions_of_beyond,
        strategic_planning,
        echoing_truth,
        // ── SOS (Secrets of Strixhaven) ──
        chase_inspiration,
        banishing_betrayal,
        procrastinate,
        wisdom_of_ages,
        brush_off,
        run_behind,
        // body-only batch
        pensive_professor,
        tester_of_the_tangential,
        muse_seeker,
        // ── 2026-04-30 push III ──
        hydro_channeler,
        // ── modern_decks (post-push III) ──
        mathemagics,
        matterbending_mage,
        orysa_tide_choreographer,
        exhibition_tidecaller,
        // ── SOS push XI: Blue MDFCs ──
        encouraging_aviator,
        harmonized_trio,
        jadzi_steward_of_fate,
        landscape_painter,
        skycoach_conductor,
        spellbook_seeker,
    ];
    if pair_contains(pair, Color::Green) {
        v.push(gaeas_skyfolk);
        v.push(talisman_of_curiosity);
        v.push(tanglepool_bridge);
        v.push(hedge_maze);
        // ── SOS Quandrix (G/U) ──
        v.push(pterafractyl);
        v.push(fractal_mascot);
        v.push(mind_into_matter);
        v.push(growth_curve);
        v.push(quandrix_charm);
        v.push(fractal_anomaly);
        v.push(proctors_gaze);
        v.push(cuboid_colony);
        v.push(tam_observant_sequencer);
    }
    if pair_contains(pair, Color::Red) {
        v.push(stormchaser_mage);
        v.push(talisman_of_creativity);
        v.push(silverbluff_bridge);
        v.push(thundering_falls);
        // ── SOS Prismari (U/R) ──
        v.push(stadium_tidalmage);
        v.push(vibrant_outburst);
        v.push(stress_dream);
        v.push(traumatic_critique);
        v.push(rapturous_moment);
        v.push(splatter_technique);
        v.push(spectacle_summit);
        v.push(visionarys_dance);
        v.push(abstract_paintmage);
        v.push(sanar_unfinished_genius);
    }
    if pair_contains(pair, Color::White) {
        v.push(teferi_time_raveler);
        v.push(dovins_veto);
        v.push(razortide_bridge);
    }
    if pair_contains(pair, Color::Black) {
        v.push(marauding_mako);
        v.push(mistvault_bridge);
        v.push(glimpse_the_unthinkable);
        v.push(crabomination);
        v.push(cruel_somnophage);
        // ── modern_decks-14 (UB cross-pool) ──
        v.push(drown_in_the_loch);
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
        // ichorid           — 🟡 "opponent has a black creature in graveyard" gate omitted
        silversmote_ghoul,
        bitterbloom_bearer,
        // dread_return      — 🟡 flashback sac-3-creatures additional cost omitted
        // tidehollow_sculler — 🟡 exile-until-LTB replaced by permanent discard
        phyrexian_arena,
        // bloodchiefs_thirst — 🟡 kicker mode (mana value ≤ 6) omitted
        deadly_dispute,
        // indulgent_tormentor — 🟡 opponent choice collapsed to always-drain-3
        damnation,
        diabolic_tutor,
        imperial_seal,
        snuff_out,
        innocent_blood,
        diabolic_edict,
        geths_verdict,
        read_the_bones,
        tragic_slip,
        heros_downfall,
        cast_down,
        mind_rot,
        raise_dead,
        murder,
        go_for_the_throat,
        disfigure,
        languish,
        grasp_of_darkness,
        smother,
        final_reward,
        ambitions_cost,
        severed_strands,
        despise,
        distress,
        last_gasp,
        plague_wind,
        bone_splinters,
        mind_twist,
        dismember,
        cling_to_dust,
        // tezzeret_cruel_captain — 🟡 static (+1/+1 to artifact creatures) and ult omitted
        // ── modern_decks-14 ──
        cremate,
        // ── modern_decks-15 ──
        ravenous_rats,
        brain_maggot,
        sudden_edict,
        // ── SOS (Secrets of Strixhaven) ──
        sneering_shadewriter,
        burrog_banemaker,
        masterful_flourish,
        wander_off,
        send_in_the_pest,
        pull_from_the_grave,
        lecturing_scornmage,
        melancholic_poet,
        foolish_fate,
        cost_of_brilliance,
        arnyn_deathbloom_botanist,
        arcane_omens,
        withering_curse,
        dissection_practice,
        end_of_the_hunt,
        // body-only batch
        eternal_student,
        postmortem_professor,
        // ── 2026-04-30 push III ──
        poisoners_apprentice,
        ulna_alley_shopkeep,
        // ── modern_decks (post-push III) ──
        pox_plague,
        // ── SOS push XI: Black MDFCs ──
        adventurous_eater,
        cheerful_osteomancer,
        emeritus_of_woe,
        grave_researcher,
        leech_collector,
        scathing_shadelock,
        scheming_silvertongue,
    ];
    if pair_contains(pair, Color::Red) {
        v.push(terminate);
        v.push(voldaren_epicure);
        // bloodtithe_harvester — 🟡 sac-Blood ping ability omitted
        v.push(drossforge_bridge);
        v.push(raucous_theater);
        // ── modern_decks-15 (BR cross-pool removal) ──
        v.push(dreadbore);
        v.push(bedevil);
    }
    if pair_contains(pair, Color::White) {
        v.push(mourning_thrull);
        v.push(goldmire_bridge);
        v.push(mortify);
        // ── SOS Silverquill (W/B) ──
        v.push(silverquill_charm);
        v.push(killians_confidence);
        v.push(imperious_inkmage);
        v.push(forum_of_amity);
        v.push(render_speechless);
        v.push(snooping_page);
        v.push(inkling_mascot);
        v.push(moment_of_reckoning);
        v.push(stirring_honormancer);
        v.push(conciliators_duelist);
        v.push(scolding_administrator);
        v.push(abigale_poet_laureate);
    }
    if pair_contains(pair, Color::Blue) {
        v.push(marauding_mako);
        v.push(mistvault_bridge);
        v.push(glimpse_the_unthinkable);
        v.push(crabomination);
        v.push(cruel_somnophage);
        // ── modern_decks-14 (UB cross-pool) ──
        v.push(drown_in_the_loch);
    }
    if pair_contains(pair, Color::Green) {
        v.push(darkmoss_bridge);
        v.push(underground_mortuary);
        v.push(tear_asunder);
        v.push(assassins_trophy);
        v.push(maelstrom_pulse);
        // ── SOS Witherbloom (B/G) ──
        v.push(witherbloom_charm);
        v.push(bogwater_lumaret);
        v.push(pest_mascot);
        v.push(grapple_with_death);
        v.push(titans_grave);
        v.push(dinas_guidance);
        v.push(teachers_pest);
        v.push(old_growth_educator);
        v.push(mind_roots);
        v.push(root_manipulation);
        v.push(blech_loafing_pest);
        v.push(cauldron_of_essence);
        v.push(vicious_rivalry);
        // Push XV: Lluwen MDFC closes out the Witherbloom school.
        v.push(lluwen_exchange_student);
    }
    if pair_contains(pair, Color::White) {
        // ── modern_decks-14 (WB cross-pool removal) ──
        v.push(vindicate);
        v.push(anguished_unmaking);
        v.push(despark);
    }
    v
}

fn red_pool(pair: [Color; 2]) -> Vec<CardFactory> {
    let mut v: Vec<CardFactory> = vec![
        lightning_bolt,
        shock,
        // tarfire          — 🟡 Kindred/Tribal type fully omitted from engine
        shivan_dragon,
        goblin_guide,
        pact_of_the_titan,
        // vandalblast      — 🟡 Overload mode omitted
        big_score,
        callous_sell_sword,
        blasphemous_act,
        anger_of_the_gods,
        // goldspan_dragon  — 🟡 "becomes target" trigger and Treasure-double static omitted
        sundering_eruption,
        // grim_lavamancer  — 🟡 exile-two-from-graveyard cost approximated away
        // pyrokinesis      — 🟡 "divide 4 damage among any number" collapsed to single target
        // simian_spirit_guide — 🟡 "exile from hand: add {R}" alt-mana ability missing
        pyroclasm,
        lightning_strike,
        goblin_bombardment,
        magma_jet,
        ancient_grudge,
        tormenting_voice,
        wild_guess,
        thrill_of_possibility,
        volcanic_hammer,
        slagstorm,
        stoke_the_flames,
        smelt,
        banefire,
        shatter,
        incinerate,
        searing_spear,
        flame_slash,
        roast,
        lava_coil,
        jayas_greeting,
        volcanic_fallout,
        stone_rain,
        earthquake,
        // chaos_warp      — 🟡 "reveal top card; cast if permanent" half omitted
        // balefire_dragon — 🟡 "that much damage" collapsed to fixed 6
        // ── modern_decks-14 ──
        magma_spray,
        skullcrack,
        fiery_impulse,
        searing_blood,
        crumble_to_dust,
        // ── modern_decks-15 ──
        strangle,
        // ── SOS (Secrets of Strixhaven) ──
        impractical_joke,
        rearing_embermare,
        charging_strifeknight,
        zealous_lorecaster,
        heated_argument,
        // body-only batch
        tackle_artist,
        thunderdrum_soloist,
        molten_core_maestro,
        expressive_firedancer,
        // ── SOS push XI: Red MDFCs ──
        blazing_firesinger,
        emeritus_of_conflict,
        goblin_glasswright,
        maelstrom_artisan,
        pigment_wrangler,
    ];
    if pair_contains(pair, Color::White) {
        v.push(lightning_helix);
        v.push(talisman_of_conviction);
        v.push(fields_of_strife);
        v.push(pursue_the_past);
        // ── SOS Lorehold (R/W) ──
        v.push(startled_relic_sloth);
        v.push(hardened_academic);
        v.push(borrowed_knowledge);
        v.push(lorehold_charm);
        // ── modern_decks (post-push III) ──
        v.push(colossus_of_the_blood_age);
    }
    if pair_contains(pair, Color::Black) {
        v.push(terminate);
        v.push(voldaren_epicure);
        // bloodtithe_harvester — 🟡 sac-Blood ping ability omitted
        v.push(drossforge_bridge);
        v.push(raucous_theater);
        // ── modern_decks-15 (BR cross-pool removal) ──
        v.push(dreadbore);
        v.push(bedevil);
    }
    if pair_contains(pair, Color::Green) {
        v.push(ghor_clan_rampager);
        v.push(slagwoods_bridge);
        v.push(elegant_parlor);
    }
    if pair_contains(pair, Color::Blue) {
        v.push(stormchaser_mage);
        v.push(talisman_of_creativity);
        v.push(silverbluff_bridge);
        v.push(thundering_falls);
        // ── SOS Prismari (U/R) ──
        v.push(stadium_tidalmage);
        v.push(vibrant_outburst);
        v.push(stress_dream);
        v.push(traumatic_critique);
        v.push(rapturous_moment);
        v.push(splatter_technique);
        v.push(spectacle_summit);
        v.push(visionarys_dance);
        v.push(abstract_paintmage);
        v.push(sanar_unfinished_genius);
    }
    if pair_contains(pair, Color::White) {
        v.push(rustvale_bridge);
        v.push(commercial_district);
        // ── SOS push XI: Lorehold (R/W) MDFC ──
        v.push(kirol_history_buff);
    }
    v
}

fn green_pool(pair: [Color; 2]) -> Vec<CardFactory> {
    let mut v: Vec<CardFactory> = vec![
        llanowar_elves,
        giant_growth,
        grizzly_bears,
        elvish_archers,
        craw_wurm,
        birds_of_paradise,
        sylvan_caryatid,
        summoners_pact,
        natures_claim,
        natures_lore,
        blossoming_defense,
        // tireless_tracker         — 🟡 "sac Clue: +1/+1 counter" activated ability omitted
        // sentinel_of_the_nameless_city — 🟡 Ward 2 not enforced; Plant subtype dropped
        haywire_mite,
        up_the_beanstalk,
        cosmogoyf,
        naturalize,
        sylvan_safekeeper,
        cankerbloom,
        eternal_witness,
        worldly_tutor,
        reclamation_sage,
        acidic_slime,
        regrowth,
        beast_within,
        rampant_growth,
        cultivate,
        farseek,
        sakura_tribe_elder,
        wood_elves,
        elvish_mystic,
        harmonize,
        wild_mongrel,
        carnage_tyrant,
        // elvish_reclaimer         — 🟡 Threshold pump (+3/+2 with 7+ in graveyard) omitted
        // rofellos_llanowar_emissary — 🟡 Forest-count multiplier collapsed to flat {G}{G}
        // biorhythm                — 🟡 "set life total to" collapsed to fixed drain
        lumra_bellow_of_the_woods,
        // ── modern_decks-14 ──
        harrow,
        // ── modern_decks-15 ──
        plummet,
        // ── SOS (Secrets of Strixhaven) ──
        noxious_newt,
        mindful_biomancer,
        shopkeepers_bane,
        oracles_restoration,
        glorious_decay,
        environmental_scientist,
        pestbrood_sloth,
        efflorescence,
        slumbering_trudge,
        tenured_concocter,
        comforting_counsel,
        planar_engineering,
        // ── SOS push XV ──
        follow_the_lumarets,
        // body-only batch
        aberrant_manawurm,
        hungry_graffalon,
        // ── modern_decks (post-push III) ──
        emil_vastlands_roamer,
        // ── SOS push XI: Green MDFCs ──
        emeritus_of_abundance,
        infirmary_healer,
        vastlands_scavenger,
    ];
    if pair_contains(pair, Color::White) {
        v.push(watchwolf);
        v.push(thornglint_bridge);
        v.push(lush_portico);
    }
    if pair_contains(pair, Color::Red) {
        v.push(ghor_clan_rampager);
        v.push(slagwoods_bridge);
        v.push(elegant_parlor);
    }
    if pair_contains(pair, Color::Blue) {
        v.push(gaeas_skyfolk);
        v.push(talisman_of_curiosity);
        v.push(tanglepool_bridge);
        v.push(hedge_maze);
        // ── SOS Quandrix (G/U) ──
        v.push(pterafractyl);
        v.push(fractal_mascot);
        v.push(mind_into_matter);
        v.push(growth_curve);
        v.push(quandrix_charm);
        v.push(fractal_anomaly);
        v.push(proctors_gaze);
        v.push(tam_observant_sequencer);
    }
    if pair_contains(pair, Color::Black) {
        v.push(darkmoss_bridge);
        v.push(underground_mortuary);
        v.push(tear_asunder);
        v.push(assassins_trophy);
        v.push(maelstrom_pulse);
        // ── SOS Witherbloom (B/G) ──
        v.push(witherbloom_charm);
        v.push(bogwater_lumaret);
        v.push(pest_mascot);
        v.push(grapple_with_death);
        v.push(titans_grave);
        v.push(dinas_guidance);
        v.push(teachers_pest);
        v.push(old_growth_educator);
        v.push(mind_roots);
        v.push(vicious_rivalry);
        v.push(lluwen_exchange_student);
    }
    if pair_contains(pair, Color::Blue) {
        // ── SOS Quandrix (G/U) ──
        v.push(paradox_gardens);
        v.push(embrace_the_paradox);
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
        let mut missing: Vec<&'static str> = Vec::new();
        for name in [
            "Disentomb",
            // (Tireless Tracker / Goldspan Dragon are intentionally not in
            // the cube pool — their respective sac-Clue / Treasure-double
            // statics aren't supported yet, so they ship as 🟡 stubs in
            // the catalog but stay out of the pool. Restore as those
            // engine features land.)
            "Cathar Commando",
            "Mind Stone",
            "Up the Beanstalk",
            // modern_decks-11: surveil land cycle + multi-color removal +
            // sweepers + body + mana engine. Checked here so the Scryfall
            // prefetch picks them up automatically.
            "Underground Mortuary",
            "Lush Portico",
            "Hedge Maze",
            "Thundering Falls",
            "Commercial District",
            "Raucous Theater",
            "Elegant Parlor",
            "Tear Asunder",
            "Assassin's Trophy",
            "Volcanic Fallout",
            "Rout",
            "Plague Wind",
            "Carnage Tyrant",
            "Krark-Clan Ironworks",
            // modern_decks-12: 12 new cards across all five colors —
            // listed here so the prefetch test fails fast if any of
            // them slips out of the pool.
            "Stone Rain",
            "Bone Splinters",
            "Hieroglyphic Illumination",
            "Mortify",
            "Maelstrom Pulse",
            "Mind Twist",
            "Dismember",
            "Echoing Truth",
            "Celestial Purge",
            "Earthquake",
            "Glimpse the Unthinkable",
            "Cling to Dust",
            // modern_decks-13: 12 new cards across all five colors —
            // listed here so the prefetch test fails fast if any of
            // them slips out of the pool.
            "Lumra, Bellow of the Woods",
            "Crabomination",
            // Cards present in catalog but intentionally not in the cube
            // pool (each ships as a 🟡 stub due to a missing engine
            // primitive — see modern.rs comments). Restore as the matching
            // primitive lands.
            //   Chaos Warp                 — top-card reveal-and-cast
            //   Elvish Reclaimer           — graveyard-count threshold pump
            //   Rofellos, Llanowar Emissary — forest-count mana scaling
            //   Biorhythm                  — set-life-total mechanic
            //   Karn, Scion of Urza        — artifact-count construct token
            //   Tezzeret, Cruel Captain    — artifact-creature pump static
            //   Balefire Dragon            — variable-damage replication
            //   Goldspan Dragon            — Treasure-double static
            //   Pentad Prism               — Sunburst counter-on-cast
            "Cruel Somnophage",
            // modern_decks-14: 13 new cards. Listed here so prefetch
            // catches any that slip out of the pool.
            "Vindicate",
            "Anguished Unmaking",
            "Despark",
            "Magma Spray",
            "Skullcrack",
            "Fiery Impulse",
            "Searing Blood",
            "Crumble to Dust",
            "Drown in the Loch",
            "Harrow",
            "Cremate",
            "Mortuary Mire",
            "Geier Reach Sanitarium",
            // modern_decks-15: 12 new cards. Listed here so prefetch
            // catches any that slip out of the pool.
            "Strangle",
            "Dreadbore",
            "Bedevil",
            "Tome Scour",
            "Repulse",
            "Visions of Beyond",
            "Plummet",
            "Strategic Planning",
            "Ravenous Rats",
            "Brain Maggot",
            "Bond of Discipline",
            "Sudden Edict",
        ] {
            if !names.contains(&name) {
                missing.push(name);
            }
        }
        assert!(missing.is_empty(),
            "expected the following cards in the cube prefetch pool but they are missing: {missing:?}");
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
