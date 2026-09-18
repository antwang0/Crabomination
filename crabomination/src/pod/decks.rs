//! The Commander pod's target decks.
//!
//! Four hand-picked commanders, each with a 99 drawn from cards this engine
//! already implements: legal under CR 903 (100 cards including the commander,
//! singleton outside basics, every card inside the commander's CR 903.4 color
//! identity) and Commander-legal per Scryfall's ban list, both of which
//! `pod::tests` asserts rather than trusts.
//!
//! They are not official preconstructed lists: a precon's partition into four
//! decks is not derivable from the offline Scryfall cache this repo carries,
//! and a list nobody can verify is worse than one the suite checks every run.
//! What they keep from the precon idea is what the pod needs — fixed, legal,
//! four different color identities, and built to play against each other.
//!
//! Each list is ~37 lands (12 nonbasic), the format's colorless staples, then
//! ramp / removal / draw / a creature curve, so bots finish games with them.

use crate::catalog::*;
use crate::cube::CardFactory;

/// Sigarda, Host of Herons — GW. Commander, then the 99.
pub const SIGARDA_COMMANDERS: &[CardFactory] = &[sigarda_host_of_herons];

/// Sigarda selfless GW: 72 nonbasic cards + 27 basics = 99.
pub const SIGARDA_MAIN: &[CardFactory] = &[
    akrasan_squire, arcane_signet, avacyns_pilgrim, boreal_druid, boseiju_who_endures,
    chromatic_sphere, chromatic_star, command_tower, commanders_sphere, containment_construct,
    daru_warchief, delighted_halfling, demystify, eager_glyphmage, eiganjo_seat_of_the_empire,
    elite_interceptor, elvish_mystic, enduring_innocence, ephemerate, erode, esper_sentinel,
    everflowing_chalice, flourishing_fox, gaeas_cradle, ghost_vacuum, greater_sandwurm,
    haywire_mite, horizon_spellbomb, hornet_sting, icatian_javelineers, immaculate_magistrate,
    isolate, keen_sense, lay_of_the_land, llanowar_elves, loran_of_the_third_path,
    lumra_bellow_of_the_woods, mana_vault, mind_stone, mosquito_guard, mosswort_bridge,
    narnam_renegade, ohran_frostfang, opal_palace, oracles_restoration, path_of_ancestry,
    pearled_unicorn, railway_brawler,
    restless_prairie, rumbling_baloth, seedborn_muse, selfless_spirit, senseis_divining_top,
    sentinel_of_the_nameless_city, shifting_woodland, skullclamp, sol_ring, sowing_mycospawn,
    sram_senior_edificer, suture_priest, tenured_concocter, thalia_guardian_of_thraben,
    tishanas_wayfinder, tocatli_honor_guard, unwavering_initiate, vengevine, voracious_varmint,
    walking_ballista, war_falcon, wayfarers_bauble, windbrisk_heights, yavimaya_elder,
    // Basics: 14 forest, 13 plains
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains,
];

/// Judith, the Scourge Diva — BR. Commander, then the 99.
pub const JUDITH_COMMANDERS: &[CardFactory] = &[judith_the_scourge_diva];

/// Judith aristocrats BR: 72 nonbasic cards + 27 basics = 99.
pub const JUDITH_MAIN: &[CardFactory] = &[
    arcane_signet, bartizan_bats, bloodchiefs_thirst, bloodrage_brawler, boggart_harbinger,
    bone_shards, bone_splinters, brain_maggot, cabal_coffers, cabal_ritual, canyon_minotaur,
    carrier_thrall, cathodion, chain_lightning, chromatic_sphere, chromatic_star,
    cling_to_dust, command_tower, commanders_sphere, crash_through, cremate, crimson_wisps,
    crush, cryptbreaker, crystalline_crawler, dark_ritual, darkblast, darksteel_colossus,
    dauthi_voidwalker, den_of_the_bugbear, desperate_ritual, earthshaker_khenra,
    embereth_shieldbreaker, everflowing_chalice, fanatical_firebrand, festering_mummy,
    furnace_whelp, goblin_banneret, grim_monolith, guardian_idol, highborn_ghoul, hollow_one,
    hurloon_minotaur, indulgent_tormentor, lecturing_scornmage, lightning_mauler, mana_vault,
    marauding_mako, mind_stone, mortuary_mire, nekrataal, nihil_spellbomb, opal_palace,
    path_of_ancestry, pia_nalaar,
    reckless_wurm, restless_vents, royal_assassin, scathing_shadelock, shivan_dragon,
    simian_spirit_guide, skirk_prospector, slaughter_pact, sokenzan_crucible_of_defiance,
    sol_ring, spinerock_knoll, takenuma_abandoned_mire, torbran_thane_of_red_fell,
    universal_automaton, walking_ballista, wayfarers_bauble, weaponcraft_enthusiast,
    // Basics: 13 mountain, 14 swamp
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
];

/// Hanna, Ship's Navigator — UW. Commander, then the 99.
pub const HANNA_COMMANDERS: &[CardFactory] = &[hanna_ships_navigator];

/// Hanna artifact UW: 70 nonbasic cards + 29 basics = 99.
pub const HANNA_MAIN: &[CardFactory] = &[
    aether_spellbomb, air_servant, arcane_signet, archangel_of_tithes, archon_of_justice,
    beloved_beggar, benthic_biomancer, blue_elemental_blast, brainstorm, bubble_smuggler,
    careful_study, chromatic_sphere, chromatic_star, command_tower, commanders_sphere,
    consider, curiosity, daru_lancer, daru_warchief, demystify, eiganjo_seat_of_the_empire,
    elite_interceptor, encouraging_aviator, enduring_innocence, ephemerate, erode,
    everflowing_chalice, ghost_vacuum, goldnight_commander, grim_monolith, guardian_idol,
    healers_hawk, hedron_crab, hydro_channeler, hydroblast, icatian_javelineers,
    imposing_vantasaur, isolate, kozileks_command, lodestone_golem, mana_vault, mind_stone,
    myr_retriever, opal_palace, orysa_tide_choreographer, otawara_soaring_city,
    path_of_ancestry, phelia_exuberant_shepherd,
    prismatic_lens, rancorous_archaic, ranger_of_eos, restless_anchorage, sanctifier_en_vec,
    shelldock_isle, sol_ring, solemn_recruit, souls_attendant, spiketail_hatchling,
    spiritcall_enthusiast, sram_senior_edificer, stirring_hopesinger, thalia_heretic_cathar,
    thieving_magpie, thought_vessel, tideshaper_mystic, vodalian_arcanist, walking_ballista,
    wayfarers_bauble, windbrisk_heights, yotian_soldier,
    // Basics: 15 island, 14 plains
    island, island, island, island, island, island, island, island, island, island, island,
    island, island, island, island, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains, plains, plains,
];

/// Tatyova, Benthic Druid — GU. Commander, then the 99.
pub const TATYOVA_COMMANDERS: &[CardFactory] = &[tatyova_benthic_druid];

/// Tatyova landfall GU: 72 nonbasic cards + 27 basics = 99.
pub const TATYOVA_MAIN: &[CardFactory] = &[
    aether_spellbomb, arcane_signet, basking_broodscale, basking_rootwalla, benthic_biomancer,
    blue_elemental_blast, boreal_druid, boseiju_who_endures, brainstorm, careful_study,
    chromatic_sphere, chromatic_star, command_tower, commanders_sphere, consider, curiosity,
    delighted_halfling, delver_of_secrets, elvish_mystic, elvish_piper, elvish_warrior,
    emeritus_of_abundance, endurance, everflowing_chalice, fertilid, gaeas_cradle,
    ghost_vacuum, glistener_elf, haywire_mite, horizon_spellbomb, hornet_sting, hydroblast,
    invisible_stalker, lay_of_the_land, llanowar_elves, mana_vault, marsh_viper, mind_stone,
    mossborn_hydra, mosswort_bridge, natures_claim, nessian_demolok, obstinate_baloth,
    opal_palace, orysa_tide_choreographer, otawara_soaring_city, path_of_ancestry,
    penumbra_wurm, phantasmal_bear,
    phantasmal_image, pongify, predatory_sliver, rapid_hybridization, restless_vinestalk,
    runeclaw_bear, sakura_tribe_elder, shelldock_isle, shifting_woodland, sol_ring,
    somber_hoverguard, spark_double, spellseeker, spike_feeder, spined_wurm, thragtusk,
    thrun_the_last_troll, vigean_graftmage, walking_ballista, wall_of_roots, wayfarers_bauble,
    wickerbough_elder, wild_nacatl,
    // Basics: 14 forest, 13 island
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, island, island, island, island, island, island, island,
    island, island, island, island, island, island,
];

/// Krark + Rograkh — mono-red, and the pod's only seat with **two**
/// commanders. CR 702.124b puts both in the command zone at the start, CR
/// 702.124d keeps their CR 903.8 tax and their 21-damage tallies separate,
/// and Rograkh's {0} cost makes every recast of him pure tax — which is the
/// point of running the pair here rather than only in a fixture.
pub const KRARK_COMMANDERS: &[CardFactory] = &[krark_the_thumbless, rograkh_son_of_rohgahh];

/// Krark/Rograkh goblins R: 73 nonbasic cards + 25 basics = 98, plus the two
/// commanders.
pub const KRARK_MAIN: &[CardFactory] = &[
    abrade, ancient_tomb, arc_trail, arcane_signet, balefire_dragon, beetleback_chief,
    blasphemous_act, blast_zone, bloodrage_brawler, buried_ruin, burnished_hart, cathodion,
    chain_lightning, chandra_torch_of_defiance, chaos_warp, command_tower,
    commanders_sphere, crush, crystalline_crawler, den_of_the_bugbear, desperate_ritual,
    dragon_fodder, etali_primal_storm, everflowing_chalice, evolving_wilds,
    faithless_looting, fanatical_firebrand, flame_slash, flametongue_kavu,
    goblin_bushwhacker, goblin_chieftain, goblin_matron, goblin_rabblemaster,
    goblin_warchief, goblin_welder, guardian_idol, hedron_archive, hellrider,
    impact_tremors, inferno_titan, inventors_fair, krenko_mob_boss, krenkos_command,
    light_up_the_stage, lightning_bolt, lightning_mauler, mana_vault, mind_stone,
    mizzium_mortars, opal_palace, palladium_myr, path_of_ancestry, pyretic_ritual,
    ramunap_ruins, reckless_bushwhacker, reckless_wurm, rogues_passage, shivan_dragon,
    shock, skirk_prospector, sneak_attack, sol_ring, solemn_simulacrum,
    terramorphic_expanse, terror_of_the_peaks, torbran_thane_of_red_fell, tormenting_voice,
    vandalblast, walking_ballista, warstorm_surge, wayfarers_bauble, wheel_of_fortune,
    zealous_conscripts,
    // Basics: 25 mountain
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain,
];
