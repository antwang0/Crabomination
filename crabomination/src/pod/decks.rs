//! The Commander pod's target decks.
//!
//! Seven hand-picked commanders, each with a 99 drawn from cards this engine
//! already implements: legal under CR 903 (100 cards including the commander,
//! singleton outside basics, every card inside the commander's CR 903.4 color
//! identity) and Commander-legal per Scryfall's ban list, both of which
//! `pod::tests` asserts rather than trusts.
//!
//! They are not official preconstructed lists: a precon's partition into four
//! decks is not derivable from the offline Scryfall cache this repo carries,
//! and a list nobody can verify is worse than one the suite checks every run.
//! What they keep from the precon idea is what the pod needs — fixed, legal,
//! seven different color identities, and built to play against each other.
//! One is a Partner pair (CR 702.124b) and one is a **planeswalker**
//! (CR 903.3a); both sit past the fourth slot, because `pod_field(n)` takes
//! the first `n` and appending is what keeps the committed outcome table
//! from moving.
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
    everflowing_chalice, flourishing_fox, fumigate, gaeas_cradle, ghost_vacuum,
    greater_sandwurm, haywire_mite, horizon_spellbomb, hornet_sting, icatian_javelineers,
    immaculate_magistrate, isolate, keen_sense, lay_of_the_land, llanowar_elves,
    loran_of_the_third_path, lumra_bellow_of_the_woods, mana_vault, mind_stone, mosswort_bridge,
    narnam_renegade, ohran_frostfang, opal_palace, oracles_restoration, path_of_ancestry,
    railway_brawler, restless_prairie, seedborn_muse, selesnya_signet, selfless_spirit,
    senseis_divining_top, sentinel_of_the_nameless_city, shifting_woodland, skullclamp,
    sol_ring, sowing_mycospawn, sram_senior_edificer, suture_priest, talisman_of_unity,
    tenured_concocter, thalia_guardian_of_thraben, tishanas_wayfinder, tocatli_honor_guard,
    unwavering_initiate, vengevine, voracious_varmint, walking_ballista, wayfarers_bauble,
    windbrisk_heights, wrath_of_god, yavimaya_elder,
    // Basics: 14 forest, 13 plains
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains,
];

/// Judith, the Scourge Diva — BR. Commander, then the 99.
pub const JUDITH_COMMANDERS: &[CardFactory] = &[judith_the_scourge_diva];

/// Judith BR: 72 nonbasic cards + 27 basics = 99.
///
/// Retuned 2026-09-19 — see PERF/TODO for the measurement that chose this
/// shape. The first list was cheap cube filler; the obvious repair (a real
/// aristocrats engine — free sacrifice outlets, drain payoffs, recursive
/// fodder) measured **worse**, because a one-ply material evaluator cannot
/// see a value engine: sacrificing a body for one life off each opponent
/// is material-negative every single time it is asked. What the bot does
/// pilot is removal, card advantage and fat, so that is what this is — with
/// Sepulchral Primordial and Gray Merchant for the per-opponent scaling and
/// Command Beacon for the command zone.
pub const JUDITH_MAIN: &[CardFactory] = &[
    arcane_signet, balefire_dragon, bedevil, blasphemous_act, bloodchiefs_thirst, bone_shards,
    bone_splinters, burnished_hart, cabal_coffers, cabal_ritual, carrier_thrall, cathodion,
    chaos_warp, command_beacon, command_tower, commanders_sphere, cryptbreaker, dark_ritual,
    dauthi_voidwalker, den_of_the_bugbear, desperate_ritual, earthshaker_khenra,
    etali_primal_storm, everflowing_chalice, feed_the_swarm, glorybringer, goblin_rabblemaster,
    grave_pact, gray_merchant_of_asphodel, grim_monolith, guardian_idol, hellrider,
    indulgent_tormentor, kokusho_the_evening_star, mana_vault, massacre_wurm, midnight_reaper,
    mind_stone, mortuary_mire, nekrataal, nights_whisper, nihil_spellbomb, noxious_gearhulk,
    opal_palace, path_of_ancestry, phyrexian_arena, pia_nalaar, rakdos_signet,
    ravenous_chupacabra, read_the_bones, reckless_wurm, restless_vents, royal_assassin,
    sepulchral_primordial, sheoldred_whispering_one, shivan_dragon, sign_in_blood,
    skirk_prospector, skullclamp, slaughter_pact, sokenzan_crucible_of_defiance, sol_ring,
    solemn_simulacrum, spinerock_knoll, takenuma_abandoned_mire, talisman_of_indulgence,
    terminate, torbran_thane_of_red_fell, toxic_deluge, walking_ballista, wayfarers_bauble,
    weaponcraft_enthusiast,
    // Basics: 13 mountain, 14 swamp
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, swamp, swamp, swamp,
];

/// Hanna, Ship's Navigator — UW. Commander, then the 99.
pub const HANNA_COMMANDERS: &[CardFactory] = &[hanna_ships_navigator];

/// Hanna artifact UW: 70 nonbasic cards + 29 basics = 99.
pub const HANNA_MAIN: &[CardFactory] = &[
    aether_spellbomb, air_servant, arcane_signet, archangel_of_tithes, archon_of_justice,
    azorius_signet, beloved_beggar, benthic_biomancer, brainstorm, bubble_smuggler,
    careful_study, chromatic_sphere, chromatic_star, command_tower, commanders_sphere, consider,
    curiosity, daru_warchief, demystify, eiganjo_seat_of_the_empire, elite_interceptor,
    encouraging_aviator, enduring_innocence, ephemerate, erode, everflowing_chalice,
    ghost_vacuum, goldnight_commander, grim_monolith, guardian_idol, healers_hawk, hedron_crab,
    hydro_channeler, hydroblast, icatian_javelineers, isolate, kozileks_command,
    lodestone_golem, mana_vault, mind_stone, myr_retriever, opal_palace,
    orysa_tide_choreographer, otawara_soaring_city, path_of_ancestry, phelia_exuberant_shepherd,
    prismatic_lens, rancorous_archaic, ranger_of_eos, restless_anchorage, rivers_rebuke,
    sanctifier_en_vec, shelldock_isle, sol_ring, solemn_recruit, souls_attendant,
    spiketail_hatchling, spiritcall_enthusiast, sram_senior_edificer, stirring_hopesinger,
    talisman_of_progress, thalia_heretic_cathar, thieving_magpie, thought_vessel,
    tideshaper_mystic, vodalian_arcanist, walking_ballista, wayfarers_bauble, windbrisk_heights,
    yotian_soldier,
    // Basics: 15 island, 14 plains
    island, island, island, island, island, island, island, island, island, island, island,
    island, island, island, island, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains, plains,
];

/// Tatyova, Benthic Druid — GU. Commander, then the 99.
pub const TATYOVA_COMMANDERS: &[CardFactory] = &[tatyova_benthic_druid];

/// Tatyova landfall GU: 72 nonbasic cards + 27 basics = 99.
pub const TATYOVA_MAIN: &[CardFactory] = &[
    aether_spellbomb, arcane_signet, bane_of_progress, basking_broodscale, basking_rootwalla,
    benthic_biomancer, blue_elemental_blast, boreal_druid, boseiju_who_endures, brainstorm,
    careful_study, chromatic_sphere, chromatic_star, command_tower, commanders_sphere, consider,
    curiosity, delighted_halfling, delver_of_secrets, elvish_mystic, elvish_piper,
    emeritus_of_abundance, endurance, evacuation, everflowing_chalice, fertilid, gaeas_cradle,
    ghost_vacuum, glistener_elf, haywire_mite, horizon_spellbomb, hornet_sting, hydroblast,
    invisible_stalker, lay_of_the_land, llanowar_elves, mana_vault, marsh_viper, mind_stone,
    mossborn_hydra, mosswort_bridge, natures_claim, nessian_demolok, obstinate_baloth,
    opal_palace, orysa_tide_choreographer, otawara_soaring_city, path_of_ancestry,
    penumbra_wurm, phantasmal_image, pongify, predatory_sliver, rapid_hybridization,
    restless_vinestalk, sakura_tribe_elder, shelldock_isle, shifting_woodland, simic_signet,
    sol_ring, somber_hoverguard, spark_double, spellseeker, spike_feeder, talisman_of_curiosity,
    thragtusk, thrun_the_last_troll, vigean_graftmage, walking_ballista, wall_of_roots,
    wayfarers_bauble, wickerbough_elder, wild_nacatl,
    // Basics: 14 forest, 13 island
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, island, island, island, island, island, island, island, island,
    island, island, island, island, island,
];

/// Krark + Rograkh — mono-red, and the pod's only seat with **two**
/// commanders. CR 702.124b puts both in the command zone at the start, CR
/// 702.124d keeps their CR 903.8 tax and their 21-damage tallies separate,
/// and Rograkh's {0} cost makes every recast of him pure tax — which is the
/// point of running the pair here rather than only in a fixture.
pub const KRARK_COMMANDERS: &[CardFactory] = &[krark_the_thumbless, rograkh_son_of_rohgahh];

/// Krark/Rograkh goblins R: 73 nonbasic cards + 25 basics = 98, plus the two
/// commanders.
///
/// ⚠ **No unbounded doubler.** Krenko, Mob Boss ("{T}: create X Goblins, where
/// X is the number of Goblins you control") was in the first draft and one
/// five-seat game reached **1,486 Goblins** before the board cap ended it
/// undecided — the run's only stall. A smoke-test deck wants boards a pod can
/// finish: an engine that doubles every turn makes one game dominate the run's
/// wall clock and decide by cap rather than by play. Goblin King is the linear
/// lord in its slot.
pub const KRARK_MAIN: &[CardFactory] = &[
    abrade, ancient_tomb, arc_trail, arcane_signet, balefire_dragon, beetleback_chief,
    blasphemous_act, blast_zone, bloodrage_brawler, buried_ruin, burnished_hart, cathodion,
    chain_lightning, chandra_torch_of_defiance, chaos_warp, command_tower,
    commanders_sphere, crush, crystalline_crawler, den_of_the_bugbear, desperate_ritual,
    dragon_fodder, etali_primal_storm, everflowing_chalice, evolving_wilds,
    faithless_looting, fanatical_firebrand, flame_slash, flametongue_kavu,
    goblin_bushwhacker, goblin_chieftain, goblin_matron, goblin_rabblemaster,
    goblin_warchief, goblin_welder, guardian_idol, hedron_archive, hellrider,
    impact_tremors, inferno_titan, inventors_fair, goblin_king, krenkos_command,
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

/// Edgar Markov — BRW (Mardu). Commander, then the 99.
///
/// The pod's **sixth** deck and its first three-colour identity, which is what
/// it is here for: CR 903.4 over three colours (Command Tower, Arcane Signet,
/// Commander's Sphere, Path of Ancestry and Opal Palace all read it), and an
/// **Eminence** commander (CR 113.6b) whose ability runs from the command zone
/// every game rather than only in a unit test. It is last in `target_decks`, so
/// `pod_field(4)` and `pod_field(5)` — and the committed outcome table — are
/// unchanged by its existence; `--seats 6` is what reaches it.
pub const EDGAR_COMMANDERS: &[CardFactory] = &[edgar_markov];

/// Edgar Markov Vampire tribal BRW: 76 nonbasic cards + 23 basics = 99.
pub const EDGAR_MAIN: &[CardFactory] = &[
    accursed_duneyard, anguished_unmaking, arcane_signet, bedevil, blasphemous_act,
    blood_artist, bloodghast, bloodline_bidding, bloodline_keeper, bloodline_recollector,
    bloodtithe_harvester, boros_signet, burnished_hart, captivating_vampire, champion_of_dusk,
    chaos_warp, charismatic_conqueror, clavileno_first_of_the_blessed, command_tower,
    commanders_sphere, cordial_vampire, cruel_celebrant, crux_of_fate, damn, drana_and_linvala,
    drana_liberator_of_malakir, edgar_ancient_bloodlord, edgar_charmed_groom,
    elenda_the_dusk_rose, elendas_hierophant, farewell, feed_the_swarm, fetid_heath,
    florian_voldaren_scion, foreboding_ruins, forerunner_of_the_legion, haunted_ridge,
    heirloom_blade, kindred_dominance, luxury_suite, markov_baron, master_of_dark_rites,
    mavren_fein_dusk_apostle, minas_tirith, mind_stone, necropotence, new_blood, nights_whisper,
    oathsworn_vampire, olivia_voldaren, olivias_wrath, opal_palace, orzhov_signet,
    path_of_ancestry, patron_of_the_vein, phyrexian_arena, rakdos_signet, rakish_heir,
    read_the_bones, sign_in_blood, skullclamp, sol_ring, solemn_simulacrum,
    sorin_lord_of_innistrad, sorin_solemn_visitor, spectator_seating, swords_to_plowshares,
    talisman_of_indulgence, terminate, toxic_deluge, vault_of_champions, vault_of_the_archangel,
    village_rites, wayfarers_bauble, westvale_abbey, wrath_of_god,
    // Basics: 10 swamp, 7 plains, 6 mountain
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, plains, plains,
    plains, plains, plains, plains, plains, mountain, mountain, mountain, mountain, mountain,
    mountain,
];
/// Freyalise, Llanowar's Fury — mono-green, and the pod's only seat led by a
/// **non-creature**. Seventh in `target_decks`, so `--seats 7` is what runs
/// it and every committed 2..6-seat number is untouched. CR 903.3a lets a card say "[this] can be your commander";
/// `CardDefinition::can_be_commander` is that line and `validate_commander_deck`
/// reads it, but until this list existed the rule was only ever unit-tested.
/// A planeswalker commander also exercises the rest of CR 903 from an unusual
/// angle: it is recast from the command zone under the CR 903.8 tax like any
/// other commander, it cannot deal commander damage (CR 903.10a is combat
/// damage, and a planeswalker never deals any), and a seat whose commander is
/// a loyalty engine keeps it on the battlefield rather than replaying it.
pub const FREYALISE_COMMANDERS: &[CardFactory] = &[freyalise_llanowars_fury];

/// Freyalise mono-G: 72 nonbasic cards + 27 basics = 99.
///
/// Built from the measurement in `DECK_FEATURES.md` rather than from taste: a
/// one-ply material evaluator prices removal, card advantage and fat, and
/// cannot price a value engine — so this is ramp into big bodies, with the
/// removal green actually has (Beast Within, Krosan Grip, Song of the Dryads)
/// and **one** board wipe's worth of interaction rather than two, since the
/// board-wipe measurement showed a second one only lengthens games.
///
/// Mono-green takes neither the guild Signet nor the guild Talisman, for the
/// same CR 903.4 reason mono-red Krark does not: every card in both cycles is
/// two-colour in identity because the pips are in the rules text.
pub const FREYALISE_MAIN: &[CardFactory] = &[
    // Lands (12 nonbasic + 25 Forest = 37)
    command_tower, path_of_ancestry, opal_palace, gaeas_cradle, boseiju_who_endures,
    mosswort_bridge, shifting_woodland, nykthos_shrine_to_nyx, castle_garenbrig,
    ancient_tomb, blast_zone, rogues_passage,
    // Colorless ramp and utility
    sol_ring, arcane_signet, commanders_sphere, mind_stone, everflowing_chalice,
    mana_vault, wayfarers_bauble, skullclamp, senseis_divining_top, walking_ballista,
    // Green ramp — the half the evaluator does price, because a land is material
    llanowar_elves, elvish_mystic, boreal_druid, delighted_halfling, sakura_tribe_elder,
    rampant_growth, cultivate, kodamas_reach, natures_lore, three_visits, farseek,
    wall_of_roots, fertilid, yavimaya_elder,
    // Card advantage
    harmonize, return_of_the_wildspeaker, shamanic_revelation, regal_force,
    sylvan_library, eternal_witness, tishanas_wayfinder, keen_sense,
    // Interaction
    beast_within, krosan_grip, natures_claim, song_of_the_dryads, reclamation_sage,
    acidic_slime, wickerbough_elder, nessian_demolok, heroic_intervention,
    // Bodies and finishers
    craterhoof_behemoth, avenger_of_zendikar, vorinclex_voice_of_hunger, thragtusk,
    obstinate_baloth, penumbra_wurm, greater_sandwurm, vengevine, ohran_frostfang,
    emeritus_of_abundance, mossborn_hydra, lumra_bellow_of_the_woods, seedborn_muse,
    thrun_the_last_troll, immaculate_magistrate, elvish_piper, spike_feeder,
    sentinel_of_the_nameless_city, garruk_wildspeaker,
    // Basics: 27 forest
    forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest, forest, forest, forest, forest, forest,
];
