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

pub type CardFactory = fn() -> CardDefinition;

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

    // Every seat carries the standard Lessons sideboard so Learn abilities
    // (Eyetwitch, Field Trip, Igneous Inspiration, …) can fetch a Lesson
    // rather than falling back to the Draw 1 approximation.
    for p in 0..2 {
        for &f in &lessons_sideboard() {
            state.add_card_to_sideboard(p, f());
        }
    }

    state.players[0].wants_ui = true;
    state.players[1].wants_ui = true;
    state
}

/// The standard Strixhaven Lessons every cube seat carries in its sideboard
/// ("outside the game"). A Learn ability may reveal one of these into hand.
/// The set spans several colors so any Learn deck has a useful fetch target.
pub fn lessons_sideboard() -> Vec<CardFactory> {
    vec![
        environmental_sciences,
        introduction_to_prophecy,
        introduction_to_annihilation,
        spirit_summoning,
        pest_summoning,
        mascot_exhibition,
    ]
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
        // ── modern_decks: draw-engine artifact ──
        the_endstone,
        // ── modern_decks: Battle-cry artifact creature ──
        signal_pest,
        // ── Eldrazi (colorless {C} cost) ──
        reality_smasher,
        // ── colorless utility ──
        pilgrims_eye,
        filigree_familiar,
        // ── classic colorless core-set bodies (claude/modern_decks) ──
        obsianus_golem,
        yotian_soldier,
        cathodion,
        bottle_gnomes,
        universal_automaton,
        frogmite,
        myr_enforcer,
        chief_of_the_foundry,
        sol_ring,
        ornithopter,
        ornithopter_of_paradise,
        mind_stone,
        coldsteel_heart,
        // fellwar_stone / star_compass read the relevant side's basic-land
        // types via `AnyColorOpponentCouldProduce` / `AnyColorYouCouldProduce`.
        fellwar_stone,
        star_compass,
        millstone,
        aether_spellbomb,
        nihil_spellbomb,
        pyrite_spellbomb,
        sylvan_spellbomb,
        horizon_spellbomb,
        expedition_map,
        executioners_capsule,
        damping_sphere,
        zuran_orb,
        soul_conduit,
        chromatic_star,
        soul_guide_lantern,
        wasteland,
        strip_mine,
        // ── Modern Locus / fetch / utility lands ──
        evolving_wilds,
        glimmerpost,
        cloudpost,
        lotus_field,
        // ── modern_decks: mana-fixing artifact land ──
        power_depot,
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
        // ── modern_decks (claude/modern_decks) — new colorless ──
        howling_mine,
        // ── modern_decks batch 102: cube expansion ──
        trinisphere,
        ravens_crime,
        pithing_needle,
        juggernaut,
        trenchpost,
        three_tree_city,
        // ── modern_decks batch 103: colorless cube expansion ──
        glaring_fleshraker,
        brightglass_gearhulk,
        // ── SOS colorless lands / artifacts / creatures ──
        skycoach_waypoint,
        biblioplex_tomekeeper,
        strixhaven_skycoach,
        the_dawning_archaic,
        // ── claude/modern_decks push: new colorless cards ──
        portal_to_phyrexia,
        mesmeric_orb,
        chalice_of_the_void,
        candelabra_of_tawnos,
        rishadan_port,
        monument_to_endurance,
        exotic_orchard,
        golos_tireless_pilgrim,
        ramos_dragon_engine,
        maelstrom_archangel,
        maelstrom_nexus,
        leyline_of_the_guildpact,
        coveted_jewel,
        the_mightstone_and_weakstone,
        kozileks_command,
        eldrazi_confluence,
        planar_nexus,
        maze_of_ith,
        // ── modern_decks-17 ──
        lightning_greaves,
        stonecoil_serpent,
        // ── equip-granted dies trigger (CR 702.6e) ──
        skullclamp,
        // ── modern_decks: living weapon + board-scaled equip bonus ──
        nettlecyst,
        // ── modern_decks: begin-combat token-copy equipment ──
        helm_of_the_host,
        // ── modern_decks: +2/+2 double-protection sword cycle ──
        sword_of_body_and_mind,
        sword_of_feast_and_famine,
        sword_of_war_and_peace,
        // ── Push XXIV (session 8) — colorless cube additions ──
        phyrexian_revoker,
        solemn_simulacrum,
        inquisitive_puppet,
        // ── Modular (CR 702.43) artifact creatures ──
        arcbound_worker,
        arcbound_stinger,
        arcbound_ravager,
        arcbound_hybrid,
        arcbound_bruiser,
        arcbound_slith,
        // ── claude/modern_decks: mana rocks, utility artifacts, rainbow lands ──
        thran_dynamo,
        ur_golems_eye,
        dreamstone_hedron,
        gilded_lotus,
        arcane_signet,
        commanders_sphere,
        prismatic_lens,
        guardian_idol,
        temple_bell,
        wayfarers_bauble,
        batterskull,
        city_of_brass,
        mana_confluence,
        ghost_quarter,
        swiftfoot_boots,
        vulshok_morningstar,
        bone_saw,
        accorders_shield,
        strider_harness,
        loxodon_warhammer,
        sword_of_vengeance,
        fireshrieker,
        whispersilk_cloak,
        darksteel_plate,
        rogues_gloves,
        specters_shroud,
        mask_of_memory,
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
        // ── modern_decks: white value/aggro creatures ──
        solemn_recruit,
        fairgrounds_warden,
        knight_exemplar,
        archangel_of_thune,
        wingmate_roc,
        boros_elite,
        brimaz_king_of_oreskos,
        adeline_resplendent_cathar,
        bygone_bishop,
        sram_senior_edificer,
        // ── Soulshift (CR 702.46) ──
        hundred_talon_kami,
        // ── Soulbond (CR 702.95) ──
        silverblade_paladin,
        nearheath_pilgrim,
        // ── simple keyword bodies ──
        mardu_hateblade,
        war_falcon,
        boros_recruit,
        // ── modern_decks: energy O-Ring + card-advantage vehicle ──
        static_prison,
        shorikai_genesis_engine,
        // ── modern_decks: graveyard-hate equipment (counter-scaled) ──
        lion_sash,
        // ── modern_decks: Adventure enchantment (anthem + token) ──
        virtue_of_loyalty,
        // ── claude/modern_decks: Extort (CR 702.99) ──
        syndic_of_tithes,
        // ── Backup that grants a triggered ability (CR 702.164) ──
        bola_slinger,
        // ── Embalm / Eternalize (CR 702.88 / 702.91) ──
        sacred_cat,
        adorned_pouncer,
        unwavering_initiate,
        steadfast_sentinel,
        sunscourge_champion,
        oketras_attendant,
        anointer_priest,
        angel_of_sanctions,
        // ── Exert (CR 702.137) ──
        tah_crop_elite,
        glory_bound_initiate,
        // ── flash O-Ring + combat removal ──
        cast_out,
        gideons_reproach,
        // ── white value/aggro bodies ──
        palace_sentinels,
        knight_of_the_white_orchid,
        adanto_vanguard,
        court_homunculus,
        // ── claude/modern_decks: white value/keyword bodies ──
        kor_skyfisher,
        whitemane_lion,
        stormfront_pegasus,
        suntail_hawk,
        pillarfield_ox,
        skyknight_legionnaire,
        healers_hawk,
        // ── Investigate (CR 701.13) ──
        thraben_inspector,
        selfless_spirit,
        champion_of_the_parish,
        soltari_priest,
        // ── claude/modern_decks: white Ixalan dinosaurs ──
        kinjallis_caller,
        territorial_hammerskull,
        // ── training body (claude/modern_decks) ──
        pridemalkin,
        // ── classic core-set bodies (claude/modern_decks) ──
        eager_cadet,
        youthful_knight,
        standing_troops,
        benalish_hero,
        skyhunter_skirmisher,
        knight_errant,
        venerable_monk,
        elite_vanguard,
        devoted_hero,
        pearled_unicorn,
        tundra_wolves,
        mesa_pegasus,
        wall_of_swords,
        // ── modern_decks (cascade/aura) ──
        ardent_plea,
        cloudgoat_ranger,
        holy_strength,
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
        descendant_of_storms,
        intervention_pact,
        thalia_guardian_of_thraben,
        // ranger_captain_of_eos 🟡 — sac-for-no-noncreature-spells static
        // omitted, but ETB tutor-for-MV≤1 is the marquee effect and
        // works end-to-end. Activated for cube pool.
        ranger_captain_of_eos,
        wrath_of_god,
        // heliod_sun_crowned 🟡 — devotion-based creature/enchantment
        // toggle omitted (we leave Heliod as a permanent creature), but
        // body + lifegain → +1/+1 trigger is the marquee. Activated.
        heliod_sun_crowned,
        // containment_priest 🟡 — ETB-replacement effect for non-cast
        // creatures omitted (the trigger doesn't fire today against
        // Reanimate/Sneak Attack-style cheats). The 2/2 Flash body
        // still functions; activate the card for the cube pool with a
        // best-effort body.
        containment_priest,
        // Clone / Phantasmal Image — CR-707 enter-as-a-copy via the
        // `enters_as_copy` hook (Clone is fully faithful; Phantasmal
        // Image carries its Illusion type + sacrifice-when-targeted rider).
        clone_card,
        phantasmal_image,
        mirror_image,
        stunt_double,
        mockingbird,
        cackling_counterpart,
        sunhome_stalwart,
        // Proliferate (CR 701.27) suite — grows the controller's counters
        // + poisons opponents via `Effect::Proliferate`.
        steady_progress,
        volt_charge,
        karns_bastion,
        contagion_clasp,
        throne_of_geth,
        inexorable_tide,
        thrummingbird,
        spike_feeder,
        grim_affliction,
        // Counter-synergy creatures (combo with proliferate / Heliod).
        walking_ballista,
        triskelion,
        hangarback_walker,
        sea_gate_oracle,
        fertilid,
        spark_double,
        reflector_mage,
        man_o_war,
        siege_gang_commander,
        flame_javelin,
        pongify,
        arc_trail,
        prey_upon,
        hedron_archive,
        soul_warden,
        essence_warden,
        llanowar_visionary,
        // ── modern_decks: block-restriction evasion (CR 509.1b) ──
        silhana_ledgewalker,
        steel_leaf_champion,
        thalia_heretic_cathar,
        // ── modern_decks: high-confidence value bodies ──
        cloudblazer,
        phantom_warrior,
        invisible_stalker,
        slither_blade,
        mistral_charger,
        vorstclaw,
        shadowmage_infiltrator,
        lilianas_specter,
        bone_shredder,
        goldnight_commander,
        elvish_archdruid,
        priest_of_titania,
        elvish_warrior,
        sylvan_ranger,
        civic_wayfinder,
        welkin_tern,
        charging_rhino,
        indrik_stomphowler,
        ambassador_oak,
        nessian_asp,
        aerial_responder,
        knight_of_meadowgrain,
        skyhunter_patrol,
        veteran_armorer,
        attended_knight,
        kor_hookmaster,
        dusk_legion_zealot,
        phyrexian_gargantua,
        frost_lynx,
        dark_banishing,
        inspired_charge,
        servo_exhibition,
        fire_ambush,
        trumpet_blast,
        augur_of_bolas,
        pestermite,
        suture_priest,
        knight_of_autumn,
        journey_to_nowhere,
        banishing_light,
        seal_of_cleansing,
        dissolve,
        day_of_judgment,
        enlightened_tutor,
        healing_salve,
        intervention_pact,
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
        // ── modern_decks (claude/modern_decks) — new W cards ──
        wall_of_omens,
        white_suns_zenith,
        // ── modern_decks batch 102 (mono-white cube expansion) ──
        generous_gift,
        oust,
        soul_snare,
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
        // ── Cube expansion: body-only stubs ──
        enduring_innocence,
        thundertrap_trainer,
        // ── modern_decks-16 ──
        wall_of_omens,
        lingering_souls,
        decree_of_justice,
        guardian_scalelord,
        intervention_pact,
        // ── New cube cards ──
        guardian_scalelord,
        descendant_of_storms,
        elite_spellbinder,
        // ── Linked exile (CR 603.6e) ──
        banisher_priest,
        oblivion_ring,
        fiend_hunter,
        // ── Exalted (CR 702.83) ──
        akrasan_squire,
        aven_squire,
        // ── Renown (CR 702.111) / Outlast (CR 702.97) ──
        topan_freeblade,
        stalwart_aven,
        ainok_bond_kin,
        abzan_falconer,
        knight_of_the_pilgrims_road,
        consuls_lieutenant,
        abzan_battle_priest,
    ];
    if pair_contains(pair, Color::Green) {
        v.push(watchwolf);
        v.push(thornglint_bridge);
        v.push(lush_portico);
        // ── Cube expansion: GW cards ──
        v.push(growing_ranks);
        v.push(citadel_castellan);
    }
    if pair_contains(pair, Color::Red) {
        v.push(lightning_helix);
        v.push(onward_victory);
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
        // ── claude/modern_decks push: RW horizon land ──
        v.push(sunbaked_canyon);
    }
    if pair_contains(pair, Color::Black) {
        v.push(mourning_thrull);
        v.push(goldmire_bridge);
        v.push(mortify);
        v.push(silent_clearing);
        v.push(talisman_of_hierarchy);
        v.push(orzhov_signet);
        v.push(brightclimb_pathway);
        v.push(bleachbone_verge);
        // ── modern_decks (aura): Gift of Orzhova ──
        v.push(gift_of_orzhova);
        // ── modern_decks-14 (WB cross-pool removal) ──
        v.push(vindicate);
        v.push(anguished_unmaking);
        v.push(despark);
        // ── modern_decks batch 102 (WB Stillmoon Cavalier) ──
        v.push(stillmoon_cavalier);
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
        v.push(fix_whats_broken);
        v.push(awaken_the_honored_dead);
        v.push(dakkon_shadow_slayer);
        // ── Cube expansion: WB cards ──
        v.push(elite_spellbinder);
        // ── modern_decks-16 ──
        v.push(lingering_souls);
    }
    if pair_contains(pair, Color::Blue) {
        v.push(teferi_time_raveler);
        v.push(dovins_veto);
        v.push(stand_deliver);
        v.push(razortide_bridge);
        v.push(floodfarm_verge);
        v.push(hengegate_pathway);
        v.push(azorius_signet);
        // ── modern_decks-16 ──
        v.push(spell_queller);
    }
    v
}

fn blue_pool(pair: [Color; 2]) -> Vec<CardFactory> {
    let mut v: Vec<CardFactory> = vec![
        // ── modern_decks: Persist counter-mage ──
        glen_elendra_archmage,
        // ── Embalm / Eternalize (CR 702.88 / 702.91) ──
        aven_initiate,
        proven_combatant,
        tah_crop_skirmisher,
        sinuous_striker,
        champion_of_wits,
        // ── Soulbond (CR 702.95) ──
        wingcrafter,
        deadeye_navigator,
        tandem_lookout,
        // ── modern_decks: cube planeswalkers ──
        narset_parter_of_veils,
        teferi_hero_of_dominaria,
        // ── modern_decks: land-scaled dig + storm payoff ──
        consult_the_star_charts,
        minds_desire,
        // ── modern_decks: draw-matters +1/+1 payoff ──
        profts_eidetic_memory,
        // ── modern_decks: artifacts-matter token maker ──
        pinnacle_emissary,
        // ── claude/modern_decks: flash flyer ──
        spectral_sailor,
        thieving_magpie,
        rootwater_hunter,
        // ── ETB-value + adapt/connive creatures (claude/modern_decks) ──
        aether_adept,
        augury_owl,
        cloudkin_seer,
        merfolk_skydiver,
        benthic_biomancer,
        pteramander,
        quandrix_cryptomancer,
        // ── Kicker (CR 702.32) ──
        into_the_roil,
        aether_figment,
        glint_nest_crane,
        cloud_of_faeries,
        merfolk_looter,
        think_twice,
        forbidden_alchemy,
        phantom_monster,
        // ── Delve dragon (MH2) ──
        murktide_regent,
        // ── classic core-set bodies (claude/modern_decks) ──
        air_elemental,
        snapping_drake,
        phantom_warrior,
        merfolk_of_the_pearl_trident,
        vodalian_soldiers,
        sea_eagle,
        wind_spirit,
        wall_of_air,
        essence_scatter,
        // ── modern_decks (auras) ──
        spectral_flight,
        wind_drake,
        flight,
        counterspell,
        mana_leak,
        spell_pierce,
        // Change-target (CR 115.7)
        redirect,
        negate,
        dispel,
        daze,
        // swan_song 🟡 — Bird token currently goes to caster instead of
        // the countered spell's controller in multiplayer; gameplay-
        // equivalent in 2-player (the only matters-case is who owns the
        // bird, which the existing wiring resolves correctly via
        // PlayerRef::ControllerOf the targeted spell). Activate.
        swan_song,
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
        gush,
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
        // ── modern_decks (claude/modern_decks) — new U cards ──
        snapcaster_mage,
        hydroblast,
        blue_elemental_blast,
        tales_end,
        stroke_of_genius,
        // ── modern_decks batch 103 (mono-blue cube expansion) ──
        tempest_angler,
        seal_of_removal,
        rapid_hybridization,
        impulse,
        serum_visions,
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
        // ── Cube expansion ──
        back_to_basics,
        opposition,
        omniscience,
        blustersquall,
        // ── modern_decks-16 ──
        mulldrifter,
        deep_analysis,
        vapor_snag,
        // ── modern_decks-17 ──
        snapcaster_mage,
        // ── cube instants ──
        gush,
    ];
    if pair_contains(pair, Color::Green) {
        v.push(gaeas_skyfolk);
        v.push(talisman_of_curiosity);
        v.push(spring_mind);
        v.push(give_take);
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
        // ── modern_decks-16 ──
        v.push(oko_thief_of_crowns);
    }
    if pair_contains(pair, Color::Red) {
        v.push(stormchaser_mage);
        v.push(talisman_of_creativity);
        v.push(fire_ice);
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
        // ── modern_decks batch 102 (UR cube expansion) ──
        v.push(saheeli_rai);
        // ── modern_decks-16 ──
        v.push(electrolyze);
        v.push(expressive_iteration);
    }
    if pair_contains(pair, Color::White) {
        v.push(teferi_time_raveler);
        v.push(dovins_veto);
        v.push(razortide_bridge);
        // ── modern_decks-16 ──
        v.push(spell_queller);
        // ── modern_decks: Jeskai Flurry (copy-second-spell) ──
        v.push(shiko_and_narset_unified);
    }
    if pair_contains(pair, Color::Black) {
        v.push(marauding_mako);
        v.push(mistvault_bridge);
        v.push(glimpse_the_unthinkable);
        v.push(far_away);
        v.push(consign_oblivion);
        v.push(spite_malice);
        v.push(profit_loss);
        v.push(gloomlake_verge);
        v.push(dimir_signet);
        v.push(clearwater_pathway);
        v.push(crabomination);
        v.push(cruel_somnophage);
        // ── modern_decks-14 (UB cross-pool) ──
        v.push(drown_in_the_loch);
        // ── modern_decks batch 102 (UB cube expansion) ──
        v.push(ashiok_nightmare_weaver);
        // ── Cube expansion: UB cards ──
        v.push(master_of_death);
        v.push(fallen_shinobi);
        // ── modern_decks-16 ──
        v.push(baleful_strix);
        // ── modern_decks-17 ──
        v.push(thought_erasure);
    }
    v
}

fn black_pool(pair: [Color; 2]) -> Vec<CardFactory> {
    let mut v: Vec<CardFactory> = vec![
        // ── modern_decks: cube planeswalkers ──
        liliana_of_the_veil,
        liliana_the_last_hope,
        // ── Eternalize (CR 702.91) ──
        dreamstealer,
        // ── Afflict (CR 702.130) ──
        khenra_eternal,
        // ── Frenzy (CR 702.68) ──
        frenzy_sliver,
        // ── Gravestorm (CR 702.69) ──
        ominous_harvest,
        // ── modern_decks: black value creatures ──
        midnight_reaper,
        grim_haruspex,
        kitesail_freebooter,
        tormented_soul,
        // ── Fabricate (CR 702.122) ──
        weaponcraft_enthusiast,
        // ── combat-damage value body ──
        stromkirk_patrol,
        // ── modern_decks: ETB reanimator with a lifelink counter ──
        metamorphosis_fanatic,
        // ── modern_decks: artifacts-matter Esper commander ──
        urza_chief_artificer,
        // ── modern_decks: Jund sacrifice payoff ──
        korvold_fae_cursed_king,
        // ── claude/modern_decks: Extort (CR 702.99) ──
        basilica_screecher,
        tithe_drinker,
        kingpins_pet,
        // ── claude/modern_decks: aristocrats / sac-fodder ──
        zulaport_cutthroat,
        doomed_dissenter,
        nantuko_husk,
        bloodthrone_vampire,
        fleshbag_marauder,
        typhoid_rats,
        abyssal_specter,
        bloodgift_demon,
        // ── claude/modern_decks: Explore (CR 701.40) ──
        seekers_squire,
        // ── classic core-set bodies (claude/modern_decks) ──
        royal_assassin,
        scathe_zombies,
        walking_corpse,
        bog_imp,
        severed_legion,
        looming_shade,
        terror,
        doom_blade,
        ravenous_chupacabra,
        vampire_nighthawk,
        nekrataal,
        skinrender,
        fatal_push,
        // ── Aristocrats / recursion ──
        blood_artist,
        carrion_feeder,
        unearth,
        // ── Evoke Incarnation (MH2) ──
        grief,
        // ── modern_decks (dredge): Stinkweed Imp ──
        stinkweed_imp,
        // ── modern_decks (dredge/aura) ──
        darkblast,
        unholy_strength,
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
        tidehollow_sculler,
        phyrexian_arena,
        bloodchiefs_thirst,
        collective_brutality,
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
        vampire_hexmage,
        plagued_rusalka,
        gnarled_scarhide,
        cordial_vampire,
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
        // ── modern_decks (claude/modern_decks) — new B cards ──
        toxic_deluge,
        demonic_consultation,
        phyrexian_reclamation,
        ophiomancer,
        black_suns_zenith,
        // ── modern_decks batch 103 (mono-black cube expansion) ──
        mai_scornful_striker,
        vendetta,
        ultimate_price,
        walk_the_plank,
        // ── modern_decks batch 102: cube expansion ──
        // murderous_cut — ✅ Delve (CR 702.66) wired via Keyword::Delve +
        //                  GameAction::CastSpellDelve; {4}{B} destroy.
        murderous_cut,
        wishclaw_talisman,
        // ── Cube expansion: body-only stubs ──
        corpse_dance,
        moonshadow,
        doomsday_excruciator,
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
        leech_collector,
        scathing_shadelock,
        scheming_silvertongue,
        // ── modern cube supplement ──
        baleful_mastery,
        parallax_nexus,
        parallax_tide,
        parallax_dementia,
        teval_arbiter_of_virtue,
        sab_sunen_luxa_embodied,
        // ── modern_decks-16 ──
        shriekmaw,
        collective_brutality,
        murderous_cut,
        chainers_edict,
        toxic_deluge,
        sinkhole,
        baleful_mastery,
        corpse_dance,
        // ── modern_decks-17 ──
        tasigur_the_golden_fang,
        // ── Outlast (CR 702.97) / Renown (CR 702.111) ──
        mer_ek_nightblade,
        disowned_ancestor,
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
        // ── modern_decks batch 102 (BR cube expansion) ──
        v.push(kolaghans_command);
        v.push(master_of_cruelties);
        v.push(geyadrone_dihada);
        // ── modern_decks batch 103 (BR cube expansion) ──
        v.push(carnage_interpreter);
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
        v.push(fix_whats_broken);
    }
    if pair_contains(pair, Color::Blue) {
        v.push(marauding_mako);
        v.push(mistvault_bridge);
        v.push(glimpse_the_unthinkable);
        v.push(crabomination);
        v.push(cruel_somnophage);
        // ── modern_decks-14 (UB cross-pool) ──
        v.push(drown_in_the_loch);
        v.push(master_of_death);
        // ── modern_decks-17 ──
        v.push(thought_erasure);
    }
    if pair_contains(pair, Color::Green) {
        v.push(darkmoss_bridge);
        v.push(underground_mortuary);
        v.push(tear_asunder);
        v.push(assassins_trophy);
        v.push(maelstrom_pulse);
        // ── modern_decks-16 ──
        v.push(putrefy);
        v.push(pernicious_deed);
        // ── modern_decks (dredge): Golgari graveyard engine ──
        v.push(golgari_thug);
        v.push(life_from_the_loam);
        v.push(golgari_brownscale);
        v.push(golgari_grave_troll);
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
        v.push(lluwen_exchange_student);
        // ── modern_decks batch 102 (BG cube expansion) ──
        v.push(putrefy_modern);
        // ── modern_decks-17 ──
        v.push(grim_flayer);
        v.push(grisly_salvage);
    }
    if pair_contains(pair, Color::White) {
        // ── modern_decks-14 (WB cross-pool removal) ──
        v.push(vindicate);
        v.push(anguished_unmaking);
        v.push(despark);
        // ── modern_decks batch 102 (WB cube expansion) ──
        v.push(sorin_grim_nemesis);
        v.push(stillmoon_cavalier);
        // ── Cube expansion: WB cards ──
        v.push(elite_spellbinder);
    }
    v
}

fn red_pool(pair: [Color; 2]) -> Vec<CardFactory> {
    let mut v: Vec<CardFactory> = vec![
        // ── modern_decks: red legends ──
        pia_and_kiran_nalaar,
        zo_zu_the_punisher,
        frontline_devastator,
        rough_tumble,
        cut_ribbons,
        toil_trouble,
        dead_gone,
        pure_simple,
        boom_bust,
        // ── modern_decks: devotion payoff ──
        fanatic_of_mogis,
        // ── modern_decks: red utility creatures ──
        manic_vandal,
        goblin_cratermaker,
        spikeshot_elder,
        bloodcrazed_neonate,
        stormblood_berserker,
        // ── modern_decks: artifacts-matter aggro ──
        inventors_apprentice,
        // ── Eternalize (CR 702.91) ──
        earthshaker_khenra,
        // ── aggressive red bodies + burn ──
        bloodrage_brawler,
        nimble_blade_khenra,
        defiant_khenra,
        open_fire,
        // ── Soulbond (CR 702.95) ──
        hanweir_lancer,
        // ── Mentor (CR 702.134) ──
        goblin_banneret,
        hammer_dropper,
        // ── Provoke (CR 702.39) ──
        crested_craghorn,
        // ── simple keyword body ──
        bloodrock_cyclops,
        // ── claude/modern_decks: Riot (CR 702.137) ──
        zhur_taa_goblin,
        frenzied_arynx,
        // ── claude/modern_decks: red value/keyword bodies ──
        mogg_fanatic,
        lightning_elemental,
        skyknight_legionnaire,
        viashino_pyromancer,
        ember_hauler,
        fire_imp,
        goblin_balloon_brigade,
        thundering_giant,
        // ── Kicker (CR 702.32) ──
        goblin_bushwhacker,
        goblin_chainwhirler,
        seasoned_pyromancer,
        goblin_king,
        goblin_chieftain,
        krenko_mob_boss,
        goblin_instigator,
        goblin_trashmaster,
        beetleback_chief,
        goblin_warchief,
        skirk_prospector,
        goblin_sledder,
        mogg_raider,
        bloodlust_inciter,
        foundry_street_denizen,
        midnight_haunting,
        gather_the_townsfolk,
        captain_of_the_watch,
        // ── UR spellslinger ──
        goblin_electromancer,
        wee_dragonauts,
        // ── Evoke Incarnation (MH2) ──
        fury,
        // ── divided-damage burn (claude/modern_decks) ──
        arc_lightning,
        forked_lightning,
        chandras_pyrohelix,
        // ── claude/modern_decks: Goad (CR 701.38) ──
        disrupt_decorum,
        // ── claude/modern_decks: Monstrosity (CR 701.31) ──
        ember_swallower,
        ill_tempered_cyclops,
        charging_monstrosaur,
        // ── claude/modern_decks: enrage dinosaurs ──
        frilled_deathspitter,
        raptor_hatchling,
        otepec_huntmaster,
        // ── classic core-set bodies (claude/modern_decks) ──
        gray_ogre,
        mons_goblin_raiders,
        raging_goblin,
        goblin_piker,
        goblin_chariot,
        mountain_goat,
        dragon_hatchling,
        hurloon_minotaur,
        wall_of_stone,
        wall_of_fire,
        flame_spirit,
        goblin_balloon_brigade,
        lightning_bolt,
        shock,
        // Copy-with-new-targets (CR 707.12 / 115.7)
        reverberate,
        fork,
        // Battle cry (CR 702.92)
        goblin_wardriver,
        sweltering_suns,
        fanatical_firebrand,
        blazing_rootwalla,
        cunning_sparkmage,
        hill_giant,
        anjes_ravager,
        reckless_wurm,
        reckless_abandon,
        fiery_temper,
        big_game_hunter,
        // tarfire 🟡 — Kindred/Tribal type fully omitted from engine
        // (the printed Kindred Goblin subtype doesn't grant any tribal
        // payoff today). The 2-damage-to-any-target body works end-to-
        // end. Activated.
        tarfire,
        shivan_dragon,
        goblin_guide,
        pact_of_the_titan,
        // vandalblast 🟡 — Overload mode omitted, but the base
        // "destroy target opponent's artifact" is the gameplay-relevant
        // payoff in most casts. Activated.
        vandalblast,
        big_score,
        callous_sell_sword,
        blasphemous_act,
        anger_of_the_gods,
        // goldspan_dragon 🟡 — "becomes target" trigger and Treasure-
        // double static omitted; body + attack-trigger Treasure still
        // works.
        goldspan_dragon,
        sundering_eruption,
        // grim_lavamancer 🟡 — the "exile two cards from your gy" cost
        // is approximated by `exile_other_filter: Any` (exiles one card).
        // The 2-damage-to-any-target body fires. Activated.
        grim_lavamancer,
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
        cam_and_farrik,
        magda_brazen_outlaw,
        stone_rain,
        earthquake,
        // chaos_warp      — 🟡 "reveal top card; cast if permanent" half omitted
        // balefire_dragon — 🟡 "that much damage" collapsed to fixed 6
        // ── modern_decks-14 ──
        magma_spray,
        skullcrack,
        fiery_impulse,
        searing_blood,
        flame_rift,
        dragon_fodder,
        krenkos_command,
        hordeling_outburst,
        grapeshot,
        ahn_crop_crasher,
        servant_of_tymaret,
        unholy_heat,
        cut_down,
        galvanic_blast,
        seal_of_fire,
        abrade,
        boros_charm,
        sprite_dragon,
        kiln_fiend,
        soul_scar_mage,
        temur_battle_rage,
        mutagenic_growth,
        brute_force,
        titans_strength,
        crash_through,
        fling,
        supreme_verdict,
        stubborn_denial,
        archmages_charm,
        snakeskin_veil,
        murmuring_mystic,
        werewolf_pack_leader,
        infernal_grasp,
        village_rites,
        power_word_kill,
        crumble_to_dust,
        // ── modern_decks (claude/modern_decks) — new R cards ──
        pyroblast,
        red_elemental_blast,
        red_suns_zenith,
        // ── modern_decks batch 102 (mono-red cube expansion) ──
        hellrider,
        etali_primal_storm,
        goblin_rabblemaster,
        // ── modern_decks batch 103 (cube expansion) ──
        death_greeters_champion,
        robber_of_the_rich,
        detectives_phoenix,
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
        // ── Cube expansion ──
        arclight_phoenix,
        electrickery,
        street_spasm,
        // ── modern cube supplement ──
        dreadhorde_arcanist,
        // ── Cube expansion: body-only stubs ──
        amped_raptor,
        // ── modern_decks-16 ──
        firebolt,
        fiery_confluence,
        arclight_phoenix,
        // ── modern_decks-17 ──
        young_pyromancer,
        dragonmaster_outcast,
        spikeshot_goblin,
        zealous_conscripts,
        pia_nalaar,
        borderland_marauder,
        monastery_swiftspear,
        // ── modern_decks-18 ──
        chain_lightning,
        rift_bolt,
        exquisite_firecraft,
        sulfuric_vortex,
        kari_zev_skyship_raider,
        // ── New cube cards ──
        greasewrench_goblin,
        // ── Renown (CR 702.111) ──
        skyraker_giant,
        // ── Bloodthirst (CR 702.54) ──
        scab_clan_mauler,
        gorehorn_minotaurs,
        bloodfray_giant,
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
        // ── modern_decks-16 ──
        v.push(kolaghans_command);
        v.push(blazemire_verge);
        v.push(talisman_of_indulgence);
        v.push(rakdos_signet);
    }
    if pair_contains(pair, Color::Green) {
        v.push(ghor_clan_rampager);
        v.push(slagwoods_bridge);
        v.push(elegant_parlor);
        v.push(assault_battery);
    }
    if pair_contains(pair, Color::Blue) {
        v.push(stormchaser_mage);
        v.push(talisman_of_creativity);
        v.push(silverbluff_bridge);
        v.push(thundering_falls);
        v.push(fiery_islet);
        v.push(riverpyre_verge);
        v.push(riverglide_pathway);
        v.push(izzet_signet);
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
        v.push(teleportal);
        // ── Cube expansion: UR cards ──
        v.push(expressive_iteration);
    }
    if pair_contains(pair, Color::White) {
        v.push(rustvale_bridge);
        v.push(commercial_district);
        // ── SOS push XI: Lorehold (R/W) MDFC ──
        v.push(kirol_history_buff);
        // ── modern_decks batch 102 (RW cube expansion) ──
        // ── modern_decks-16 ──
        v.push(wear_tear);
        v.push(needleverge_pathway);
        v.push(boros_signet);
    }
    v
}

fn green_pool(pair: [Color; 2]) -> Vec<CardFactory> {
    let mut v: Vec<CardFactory> = vec![
        // ── Poisonous (CR 702.70) ──
        marsh_viper,
        mouth_feed,
        supply_demand,
        // ── modern_decks: +1/+1-counter green creatures ──
        avatar_of_the_resolute,
        pelt_collector,
        // ── Revolt (CR 702.139) ──
        narnam_renegade,
        greenwheel_liberator,
        hidden_herbalists,
        ridgescale_tusker,
        // ── modern_decks: green beaters / landfall ──
        plated_crusher,
        terra_stomper,
        rumbling_baloth,
        charging_badger,
        bellowing_tanglewurm,
        vinelasher_kudzu,
        // ── Embalm (CR 702.88) ──
        honored_hydra,
        timeless_witness,
        // ── AKH green bodies (Exert, Flash, ramp, big cycler) ──
        hooded_brawler,
        greater_sandwurm,
        pouncing_cheetah,
        naga_vitalist,
        // ── Soulbond (CR 702.95) ──
        wolfir_silverheart,
        nightshade_peddler,
        trusted_forcemage,
        // ── modern_decks: begin-combat counter scaling + Plot ──
        ouroboroid,
        // ── modern_decks: Changeling + token Mutavault ──
        mutable_explorer,
        railway_brawler,
        baloth_prime,
        icetill_explorer,
        mightform_harmonizer,
        springleaf_parade,
        kestia_the_cultivator,
        // ── claude/modern_decks: deathtouch body ──
        gnarlwood_dryad,
        gladecover_scout,
        deadly_recluse,
        sporemound,
        centaur_courser,
        borderland_ranger,
        penumbra_spider,
        // ── Lure (CR 509.1c all-must-block) ──
        lure,
        // ── Undying aggro ──
        strangleroot_geist,
        // ── classic core-set bodies (claude/modern_decks) ──
        spined_wurm,
        panther_warriors,
        redwood_treefolk,
        gorilla_chieftain,
        trained_armodon,
        giant_spider,
        scryb_sprites,
        wall_of_wood,
        llanowar_elves,
        giant_growth,
        grizzly_bears,
        elvish_archers,
        craw_wurm,
        arrogant_wurm,
        cryptolith_rite,
        call_of_the_herd,
        springbloom_druid,
        pelakka_wurm,
        brindle_boar,
        sentinel_spider,
        // ── modern_decks (cascade): Apex Devastator (cascade ×4) ──
        apex_devastator,
        // ── modern_decks (cascade/aura): Shardless Agent (GU) + Rancor ──
        shardless_agent,
        rancor,
        // ── modern_decks (cascade RG): Violent Outburst ──
        violent_outburst,
        birds_of_paradise,
        sylvan_caryatid,
        summoners_pact,
        natures_claim,
        natures_lore,
        blossoming_defense,
        // tireless_tracker 🟡 — "sac Clue: +1/+1 counter" activated
        // ability omitted, but the landfall → investigate trigger is
        // the marquee value engine in green ramp shells and fires.
        // Activate.
        tireless_tracker,
        // ── claude/modern_decks: Explore (CR 701.40) ──
        merfolk_branchwalker,
        jadelight_ranger,
        wildgrowth_walker,
        tishanas_wayfinder,
        emperors_vanguard,
        path_of_discovery,
        // ── claude/modern_decks: Monstrosity (CR 701.31) ──
        nessian_wilds_ravager,
        arbor_colossus,
        // ── claude/modern_decks: Ixalan dinosaurs / green value ──
        ripjaw_raptor,
        thrashing_brontodon,
        farhaven_elf,
        regisaur_alpha,
        grazing_whiptail,
        pounce,
        atzocan_archer,
        ranging_raptors,
        // sentinel_of_the_nameless_city 🟡 — Ward 2 not enforced;
        // Plant subtype dropped (Merfolk Warrior Scout is preserved).
        // The ETB/attack-trigger Map token still works.
        sentinel_of_the_nameless_city,
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
        viridian_emissary,
        werebear,
        thornweald_archer,
        wild_nacatl,
        skyshroud_elite,
        elvish_mystic,
        keen_eyed_curator,
        harmonize,
        wild_mongrel,
        carnage_tyrant,
        // elvish_reclaimer         — 🟡 Threshold pump (+3/+2 with 7+ in graveyard) omitted
        // rofellos_llanowar_emissary ✅ (was 🟡) — push (claude/modern_decks):
        // Forest-count multiplier now wired via
        // `ManaPayload::OfColor(Green, Value::Times(2, CountOf(Forest)))`.
        rofellos_llanowar_emissary,
        // biorhythm                — 🟡 "set life total to" collapsed to fixed drain
        lumra_bellow_of_the_woods,
        // ── modern_decks (claude/modern_decks) — new G cards ──
        three_visits,
        wall_of_roots,
        pernicious_deed,
        yavimaya_elder,
        channel,
        sylvan_library,
        green_suns_zenith,
        // ── modern_decks batch 103 (mono-green cube expansion) ──
        mossborn_hydra,
        rabid_bite,
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
        // ── Cube expansion ──
        collector_ouphe,
        // ── Cube expansion: body-only stubs ──
        basking_rootwalla,
        // ── Push XIX: cube expansion ──
        elder_gargaroth,
        // ── claude/modern_decks push: new green cards ──
        vengevine,
        finale_of_devastation,
        archdruids_charm,
        kodamas_reach,
        greater_good,
        qasali_pridemage,
        basking_broodscale,
        sowing_mycospawn,
        ursine_monstrosity,
        conclave_sledge_captain,
        zopandrel_hunger_dominus,
        // ── modern_decks-16 ──
        wall_of_blossoms,
        thragtusk,
        tireless_provisioner,
        courser_of_kruphix,
        explore,
        elder_gargaroth,
        vengevine,
        // ── modern_decks-18 ──
        scavenging_ooze,
        // ── New cube cards ──
        esikas_chariot,
        // ── Outlast (CR 702.97) / Graft (CR 702.57) ──
        tuskguard_captain,
        aquastrand_spider,
        plaxcaster_frogling,
        cytoplast_root_kin,
        simic_initiate,
        vigean_graftmage,
        helium_squirter,
    ];
    if pair_contains(pair, Color::White) {
        v.push(watchwolf);
        v.push(thornglint_bridge);
        v.push(lush_portico);
        v.push(torsten_founder_of_benalia);
        v.push(wax_wane);
        v.push(alive_well);
        v.push(ready_willing);
        v.push(horizon_canopy);
        v.push(hushwood_verge);
        v.push(talisman_of_unity);
        v.push(selesnya_signet);
        v.push(branchloft_pathway);
        v.push(growing_ranks);
        // ── modern_decks (cascade): Enlisted Wurm (GW cascade) ──
        v.push(enlisted_wurm);
        // ── modern_decks batch 102 (GW cube expansion) ──
        v.push(heroic_intervention);
        v.push(knight_of_the_reliquary);
        // ── modern_decks batch 103 (GW cube expansion) ──
        v.push(loot_the_pathfinder);
        v.push(messenger_falcons);
        // ── modern_decks-16 ──
        v.push(kitchen_finks);
    }
    if pair_contains(pair, Color::Red) {
        v.push(ghor_clan_rampager);
        v.push(slagwoods_bridge);
        v.push(elegant_parlor);
        // ── modern_decks batch 102 (RG cube expansion) ──
        v.push(territorial_kavu);
        v.push(omnath_locus_of_rage);
        // ── modern_decks-16 ──
        v.push(bloodbraid_elf);
        v.push(thornspire_verge);
        v.push(talisman_of_impulse);
        v.push(gruul_signet);
        v.push(cragcrown_pathway);
        // ── modern_decks: cube maybeboard additions ──
        v.push(bloodbraid_challenger);
        v.push(legion_extruder);
        v.push(dragonback_assault);
        v.push(twisted_landscape);
        v.push(sheltering_landscape);
        v.push(bountiful_landscape);
        v.push(enduring_vitality);
        v.push(fangkeepers_familiar);
    }
    if pair_contains(pair, Color::Blue) {
        v.push(gaeas_skyfolk);
        v.push(talisman_of_curiosity);
        v.push(tanglepool_bridge);
        v.push(hedge_maze);
        v.push(barkchannel_pathway);
        v.push(simic_signet);
        // ── SOS Quandrix (G/U) ──
        v.push(pterafractyl);
        v.push(fractal_mascot);
        v.push(mind_into_matter);
        v.push(growth_curve);
        v.push(quandrix_charm);
        v.push(fractal_anomaly);
        v.push(proctors_gaze);
        v.push(tam_observant_sequencer);
        // ── modern_decks batch 102 (GU cube expansion) ──
        v.push(tamiyo_collector_of_tales);
        // ── modern_decks batch 103 (GU cube expansion) ──
        v.push(lonis_genetics_expert);
        // ── claude/modern_decks push: new GU cards ──
        v.push(koma_cosmos_serpent);
        v.push(waterlogged_grove);
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
        v.push(wight_of_the_reliquary);
        // ── modern_decks-17 ──
        v.push(grim_flayer);
        v.push(grisly_salvage);
        v.push(nurturing_peatland);
        v.push(talisman_of_resilience);
        v.push(golgari_signet);
        v.push(wastewood_verge);
        v.push(broodspinner);
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
            // modern_decks batch 103: cube expansion. Listed so the
            // prefetch test catches any that slip out of the pool.
            "Glaring Fleshraker",
            "Brightglass Gearhulk",
            "Death-Greeter's Champion",
            "Detective's Phoenix",
            "Mossborn Hydra",
            "Mai, Scornful Striker",
            "Tempest Angler",
            "Carnage Interpreter",
            "Lonis, Genetics Expert",
            "Loot, the Pathfinder",
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

    #[test]
    fn build_cube_state_gives_each_seat_a_lessons_sideboard() {
        use crate::card::SpellSubtype;
        let state = build_cube_state();
        for p in 0..2 {
            assert_eq!(
                state.players[p].sideboard.len(),
                lessons_sideboard().len(),
                "seat {p} carries the full Lessons sideboard",
            );
            // Every sideboard card must be a Lesson, or a Learn ability
            // (which filters on `SpellSubtype::Lesson`) couldn't fetch it.
            assert!(
                state.players[p].sideboard.iter().all(|c| c
                    .definition
                    .subtypes
                    .spell_subtypes
                    .contains(&SpellSubtype::Lesson)),
                "seat {p}'s sideboard is all Lessons",
            );
        }
    }
}
