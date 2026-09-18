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
//! Each list is ~37 lands (10 nonbasic), the format's colorless staples, then
//! ramp / removal / draw / a creature curve, so bots finish games with them.

use crate::catalog::*;
use crate::cube::CardFactory;

/// Sigarda, Host of Herons — GW. Commander, then the 99.
pub const SIGARDA_COMMANDERS: &[CardFactory] = &[sigarda_host_of_herons];

/// Sigarda selfless GW: 70 nonbasic cards + 29 basics = 99.
pub const SIGARDA_MAIN: &[CardFactory] = &[
    akrasan_squire, arcane_signet, avacyns_pilgrim, boreal_druid, boseiju_who_endures,
    chromatic_sphere, chromatic_star, command_tower, commanders_sphere, containment_construct,
    daru_warchief, delighted_halfling, demystify, eager_glyphmage, eiganjo_seat_of_the_empire,
    elite_interceptor, elvish_mystic, enduring_innocence, ephemerate, erode, esper_sentinel,
    everflowing_chalice, flourishing_fox, gaeas_cradle, ghost_vacuum, greater_sandwurm,
    haywire_mite, horizon_spellbomb, hornet_sting, icatian_javelineers, immaculate_magistrate,
    isolate, keen_sense, lay_of_the_land, llanowar_elves, loran_of_the_third_path,
    lumra_bellow_of_the_woods, mana_vault, mind_stone, mosquito_guard, mosswort_bridge,
    narnam_renegade, ohran_frostfang, oracles_restoration, pearled_unicorn, railway_brawler,
    restless_prairie, rumbling_baloth, seedborn_muse, selfless_spirit, senseis_divining_top,
    sentinel_of_the_nameless_city, shifting_woodland, skullclamp, sol_ring, sowing_mycospawn,
    sram_senior_edificer, suture_priest, tenured_concocter, thalia_guardian_of_thraben,
    tishanas_wayfinder, tocatli_honor_guard, unwavering_initiate, vengevine, voracious_varmint,
    walking_ballista, war_falcon, wayfarers_bauble, windbrisk_heights, yavimaya_elder,
    // Basics: 15 forest, 14 plains
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains, plains,
];

/// Judith, the Scourge Diva — BR. Commander, then the 99.
pub const JUDITH_COMMANDERS: &[CardFactory] = &[judith_the_scourge_diva];

/// Judith aristocrats BR: 70 nonbasic cards + 29 basics = 99.
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
    marauding_mako, mind_stone, mortuary_mire, nekrataal, nihil_spellbomb, pia_nalaar,
    reckless_wurm, restless_vents, royal_assassin, scathing_shadelock, shivan_dragon,
    simian_spirit_guide, skirk_prospector, slaughter_pact, sokenzan_crucible_of_defiance,
    sol_ring, spinerock_knoll, takenuma_abandoned_mire, torbran_thane_of_red_fell,
    universal_automaton, walking_ballista, wayfarers_bauble, weaponcraft_enthusiast,
    // Basics: 14 mountain, 15 swamp
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
];

/// Hanna, Ship's Navigator — UW. Commander, then the 99.
pub const HANNA_COMMANDERS: &[CardFactory] = &[hanna_ships_navigator];

/// Hanna artifact UW: 68 nonbasic cards + 31 basics = 99.
pub const HANNA_MAIN: &[CardFactory] = &[
    aether_spellbomb, air_servant, arcane_signet, archangel_of_tithes, archon_of_justice,
    beloved_beggar, benthic_biomancer, blue_elemental_blast, brainstorm, bubble_smuggler,
    careful_study, chromatic_sphere, chromatic_star, command_tower, commanders_sphere,
    consider, curiosity, daru_lancer, daru_warchief, demystify, eiganjo_seat_of_the_empire,
    elite_interceptor, encouraging_aviator, enduring_innocence, ephemerate, erode,
    everflowing_chalice, ghost_vacuum, goldnight_commander, grim_monolith, guardian_idol,
    healers_hawk, hedron_crab, hydro_channeler, hydroblast, icatian_javelineers,
    imposing_vantasaur, isolate, kozileks_command, lodestone_golem, mana_vault, mind_stone,
    myr_retriever, orysa_tide_choreographer, otawara_soaring_city, phelia_exuberant_shepherd,
    prismatic_lens, rancorous_archaic, ranger_of_eos, restless_anchorage, sanctifier_en_vec,
    shelldock_isle, sol_ring, solemn_recruit, souls_attendant, spiketail_hatchling,
    spiritcall_enthusiast, sram_senior_edificer, stirring_hopesinger, thalia_heretic_cathar,
    thieving_magpie, thought_vessel, tideshaper_mystic, vodalian_arcanist, walking_ballista,
    wayfarers_bauble, windbrisk_heights, yotian_soldier,
    // Basics: 16 island, 15 plains
    island, island, island, island, island, island, island, island, island, island, island,
    island, island, island, island, island, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains, plains, plains, plains,
];

/// Tatyova, Benthic Druid — GU. Commander, then the 99.
pub const TATYOVA_COMMANDERS: &[CardFactory] = &[tatyova_benthic_druid];

/// Tatyova landfall GU: 70 nonbasic cards + 29 basics = 99.
pub const TATYOVA_MAIN: &[CardFactory] = &[
    aether_spellbomb, arcane_signet, basking_broodscale, basking_rootwalla, benthic_biomancer,
    blue_elemental_blast, boreal_druid, boseiju_who_endures, brainstorm, careful_study,
    chromatic_sphere, chromatic_star, command_tower, commanders_sphere, consider, curiosity,
    delighted_halfling, delver_of_secrets, elvish_mystic, elvish_piper, elvish_warrior,
    emeritus_of_abundance, endurance, everflowing_chalice, fertilid, gaeas_cradle,
    ghost_vacuum, glistener_elf, haywire_mite, horizon_spellbomb, hornet_sting, hydroblast,
    invisible_stalker, lay_of_the_land, llanowar_elves, mana_vault, marsh_viper, mind_stone,
    mossborn_hydra, mosswort_bridge, natures_claim, nessian_demolok, obstinate_baloth,
    orysa_tide_choreographer, otawara_soaring_city, penumbra_wurm, phantasmal_bear,
    phantasmal_image, pongify, predatory_sliver, rapid_hybridization, restless_vinestalk,
    runeclaw_bear, sakura_tribe_elder, shelldock_isle, shifting_woodland, sol_ring,
    somber_hoverguard, spark_double, spellseeker, spike_feeder, spined_wurm, thragtusk,
    thrun_the_last_troll, vigean_graftmage, walking_ballista, wall_of_roots, wayfarers_bauble,
    wickerbough_elder, wild_nacatl,
    // Basics: 15 forest, 14 island
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest, island, island, island, island, island, island, island,
    island, island, island, island, island, island, island,
];
