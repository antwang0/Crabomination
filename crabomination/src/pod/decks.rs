//! The Commander pod's target decks.
//!
//! Ten hand-picked commanders plus fourteen official preconstructed lists, each a
//! 99 of cards this engine implements: legal under CR 903 (100 cards
//! including the commander, singleton outside basics, every card inside the
//! commander's CR 903.4 color identity) and Commander-legal per Scryfall's
//! ban list, both of which `pod::tests` asserts rather than trusts.
//!
//! The ten were built by hand because the offline Scryfall cache cannot say
//! how a precon splits into decks; MTGJSON's deck files can, and the eleventh
//! through twenty-fourth (Sultai Arisen, Mind Flayarrrs, Blood Rites, Heads I
//! Win, Tails You Lose, Goblin Storm, Foundations Commander's Wretched Ranks,
//! Tramplesaurus Rex and Keen Engineering, Commander Legends' Reap the Tides,
//! Phyrexia: All Will Be One's Corrupting Influence, Zendikar Rising's Sneak
//! Attack, Tarkir: Dragonstorm's Jeskai Striker, Commander Masters' Sliver
//! Swarm and Outlaws of Thunder Junction's Quick Draw) are taken from one card for card. What all of them keep
//! from the precon idea is what the pod needs — fixed, legal, varied color
//! identities, and built to play against each other.
//! One is a Partner pair (CR 702.124b), one is a **planeswalker**
//! (CR 903.3a) and one is a **Choose a Background** pair (CR 702.124k); all
//! three sit past the fourth slot, because `pod_field(n)` takes the first `n`
//! and appending is what keeps the committed outcome table from moving.
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

/// Zellix, Sanity Flayer + Passionate Archaeologist — UR, and the pod's only
/// **Choose a Background** pair (CR 702.124k). Eighth in `target_decks`, so
/// `--seats 8` is what runs it and every committed 2..7-seat number is
/// untouched.
///
/// It is here for the same reason the Partner seat and the planeswalker seat
/// are: `Keyword::ChooseABackground` and `format::is_background_pair` had been
/// validated since they shipped and never *piloted*. Three things fall out of
/// a Background second commander and none of them is the Partner pair's:
///
/// * CR 702.124k's second commander is a legendary **enchantment**, not a
///   creature — `is_legal_commander` rejects it on its own, and only the pair
///   check in `validate_commander_deck` lets it in;
/// * CR 702.124c combines the identities across a creature and an enchantment
///   ({U} + {R} = Izzet), which is what every "any colour in your commander's
///   identity" source in the list reads;
/// * CR 702.124d still keeps the tax and the damage tallies separate, and the
///   Background's 21-damage tally stays at zero all game for CR 903.10a's
///   reason — an enchantment deals no combat damage. Same shape as the
///   planeswalker seat, reached from the other direction.
///
/// Zellix's own trigger is the pod half: "whenever **a player** mills one or
/// more creature cards" is `EventScope::AnyPlayer` with `once_per_batch`
/// (CR 603.2c), so it reads the whole table rather than its controller.
pub const ZELLIX_COMMANDERS: &[CardFactory] =
    &[zellix_sanity_flayer, passionate_archaeologist];

/// Zellix/Archaeologist Izzet: 74 nonbasic cards + 24 basics = 98, plus the
/// two commanders.
///
/// Built to the same measured shape as the Judith and Freyalise retunes —
/// removal, card advantage and fat, which is what a one-ply material
/// evaluator can price — rather than to the mill theme Zellix suggests. The
/// self-mill that is here (Hedron Crab, Careful Study, Faithless Looting)
/// feeds the Hive Mind trigger without asking the bot to value a graveyard.
pub const ZELLIX_MAIN: &[CardFactory] = &[
    // Lands (12 nonbasic + 24 basics = 36)
    command_tower, path_of_ancestry, opal_palace, otawara_soaring_city, shelldock_isle,
    den_of_the_bugbear, spinerock_knoll, buried_ruin, inventors_fair, rogues_passage,
    // The bond cycle's Izzet member — a tapland in a duel and a dual in a pod,
    // which is the only seat in the field that runs one from the start
    training_center, evolving_wilds,
    // Colorless ramp and utility
    sol_ring, arcane_signet, commanders_sphere, mind_stone, thought_vessel,
    everflowing_chalice, guardian_idol, mana_vault, grim_monolith, hedron_archive,
    wayfarers_bauble, palladium_myr, solemn_simulacrum, skullclamp, senseis_divining_top,
    // The guild pair — both are two-colour in CR 903.4 identity, so they are
    // legal here where mono-red Krark and mono-green Freyalise take neither
    izzet_signet, talisman_of_creativity,
    // Interaction
    lightning_bolt, shock, abrade, chain_lightning, flame_slash, arc_trail,
    mizzium_mortars, chaos_warp, blasphemous_act, crush, vandalblast, pongify,
    rapid_hybridization, rivers_rebuke,
    // Card advantage and selection
    brainstorm, consider, careful_study, faithless_looting, light_up_the_stage,
    tormenting_voice, wheel_of_fortune, curiosity, thieving_magpie,
    // Bodies and finishers
    fanatical_firebrand, bloodrage_brawler, goblin_rabblemaster, flametongue_kavu,
    hedron_crab, benthic_biomancer, tideshaper_mystic, spiketail_hatchling,
    vodalian_arcanist, myr_retriever, lodestone_golem, air_servant, reckless_wurm,
    inferno_titan, balefire_dragon, shivan_dragon, terror_of_the_peaks,
    etali_primal_storm, walking_ballista, cathodion, zealous_conscripts,
    phantasmal_image,
    // Basics: 12 island, 12 mountain
    island, island, island, island, island, island, island, island, island, island,
    island, island, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain,
];

/// Yuriko, the Tiger's Shadow — UB, and the pod's first seat led by a
/// commander whose printed ability is used **from the command zone by an
/// action rather than by a cast**. Ninth in `target_decks`, so `--seats 9` is
/// what runs it and every committed 2..8-seat number is untouched.
///
/// It is here for the reason the Partner, planeswalker and Background seats
/// are: `Keyword::CommanderNinjutsu` (CR 702.49d) had been validated since it
/// shipped and never *piloted*. Three things fall out of it and none is any
/// other seat's:
///
/// * CR 702.49d is the one command-zone route that is **not** a cast, so the
///   CR 903.8 tax never applies to it — `commander_cast_count` stays where it
///   was and the ninja is free after the first, which is the whole reason the
///   variant exists;
/// * it needs an *unblocked attacker* to return, so the deck is built around
///   creatures that cannot be blocked rather than around big ones, and the
///   bot's `pick_ninjutsu` only offers the swap when the ninja hits at least
///   as hard or carries an attack/combat-damage trigger;
/// * Yuriko's own trigger — "whenever a Ninja you control deals combat damage
///   to a player, reveal the top card of your library … each opponent loses
///   life equal to that card's mana value" — is a **pod** clause: one
///   connection drains the whole table, so its rate scales with the seat
///   count where every other seat's damage does not.
///
/// The 99 is the field's shape: ~36 lands, the colorless staples, the Dimir
/// Signet and Talisman, then removal, card advantage and a few finishers,
/// which is what a one-ply material evaluator can price (DECK_FEATURES has
/// the measurement that settled that).
pub const YURIKO_COMMANDERS: &[CardFactory] = &[yuriko_the_tigers_shadow];

/// Yuriko Dimir: 76 nonbasic cards + 23 basics = 99.
pub const YURIKO_MAIN: &[CardFactory] = &[
    // Lands (13 nonbasic + 23 basics = 36)
    command_tower, path_of_ancestry, opal_palace, otawara_soaring_city,
    takenuma_abandoned_mire, drowned_catacomb, watery_grave, darkslick_shores,
    underground_river, dimir_guildgate, shelldock_isle, evolving_wilds,
    // The one land that turns a stalled board back into a ninjutsu trigger
    rogues_passage,
    // Colorless ramp and utility
    sol_ring, arcane_signet, commanders_sphere, mind_stone, thought_vessel,
    everflowing_chalice, wayfarers_bauble, senseis_divining_top, skullclamp, mana_vault,
    // The guild pair — both two-colour in CR 903.4 identity
    dimir_signet, talisman_of_dominance,
    // Ninjas: the payoff half of the deck, and what Yuriko's trigger counts
    dokuchi_silencer, inkrise_infiltrator, moon_circuit_hacker, nashi_searcher_in_the_dark,
    nezumi_prowler, satoru_the_infiltrator, silver_fur_master, skullsnatcher,
    biting_palm_ninja, mistblade_shinobi, prosperous_thief, ninja_of_the_deep_hours,
    higure_the_still_wind, ink_eyes_servant_of_oni,
    // Enablers: cheap bodies that connect, which is what ninjutsu returns
    changeling_outcast, tormented_soul, slither_blade, triton_shorestalker,
    invisible_stalker, gingerbrute, faerie_seer, spectral_sailor, vault_skirge,
    baleful_strix, dimir_infiltrator, looter_il_kor,
    // Interaction, including the two sweepers that break a stalled pod
    pongify, rapid_hybridization, bloodchiefs_thirst, feed_the_swarm, slaughter_pact,
    go_for_the_throat, cast_down, murder, bone_shards, agony_warp, toxic_deluge, damnation,
    // Card advantage and one counter
    counterspell, brainstorm, consider, nights_whisper, sign_in_blood, careful_study,
    phyrexian_arena, read_the_bones,
    // Finishers
    gray_merchant_of_asphodel, massacre_wurm, sheoldred_whispering_one,
    consecrated_sphinx, hypnotic_specter,
    // Basics: 12 island, 11 swamp
    island, island, island, island, island, island, island, island, island, island,
    island, island, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp,
];

/// Adriana, Captain of the Guard — RW. Commander, then the 99.
pub const ADRIANA_COMMANDERS: &[CardFactory] = &[adriana_captain_of_the_guard];

/// Adriana Boros: 74 nonbasic cards + 25 basics = 99.
///
/// The field's tenth deck and the first built around what only happens at
/// three seats or more. Every other list wins by killing the seat across the
/// table faster; this one is 27 cards whose text has no meaning in a duel —
/// **the monarch** (CR 725), **goad** (CR 701.15), **melee** (CR 702.121),
/// **will of the council** and **council's dilemma** (CR 701.38), **tempting
/// offer** and **join forces** (ability words, CR 207.2c), **the initiative**
/// (CR 726) and **myriad** (CR 702.116). Nine implemented
/// mechanics that no pod deck had ever piloted, which is why they are here:
/// a mechanic the field never plays is a mechanic self-play never crashes on.
///
/// Adriana herself is the thesis — melee on every creature she controls, so
/// the pump scales with the number of *distinct opponents attacked*, not with
/// the number of attackers. Goad supplies those opponents by forcing the
/// table to swing at each other, and the monarch supplies the cards while
/// daring them to swing back.
pub const ADRIANA_MAIN: &[CardFactory] = &[
    // Lands: 11 nonbasic, one of which is the monarch's own
    command_tower, path_of_ancestry, sacred_foundry, battlefield_forge, inspiring_vantage,
    arid_mesa, evolving_wilds, temple_of_triumph, myriad_landscape, throne_of_the_high_city,
    spectator_seating,
    // Rocks
    sol_ring, arcane_signet, boros_signet, commanders_sphere, mind_stone, fellwar_stone,
    wayfarers_bauble,
    // The monarch (CR 725): the card engine, and a standing invitation to attack you
    palace_sentinels, palace_jailer, throne_warden, protector_of_the_crown,
    crown_hunter_hireling,
    // Goad (CR 701.15): the opponents Adriana's melee wants attacked
    grenzo_havoc_raiser, gloin_dwarf_emissary, goblin_racketeer, besmirch, disrupt_decorum,
    // Melee (CR 702.121) and the rest of the attacks-are-mandatory package.
    // ⚠ Grand Melee was cut from this slot: its second line ("all creatures
    // block each combat if able") is symmetric and cancels exactly the attacks
    // goad and Fumiko force, and the tenth seat cost +26.5 turns/game against
    // a ~13-turn baseline per added seat while it was in.
    custodi_soulcaller, deputized_protester, wings_of_the_guard, grenzos_ruffians, impact_tremors,
    fumiko_the_lowblood, hellraiser_goblin, the_akroan_war,
    // Voting (CR 701.38) and the two offers (ability words, CR 207.2c), plus
    // the initiative (CR 726) and myriad (CR 702.116)
    councils_judgment, custodi_squire, lieutenants_of_the_guard, coercive_portal,
    tempt_with_glory, tempt_with_vengeance, mana_charged_dragon, caves_of_chaos_adventurer,
    blade_of_selves,
    // Removal, including the two sweepers a stalled pod needs
    swords_to_plowshares, path_to_exile, lightning_bolt, abrade, chaos_warp, vandalblast,
    wrath_of_god, blasphemous_act,
    // Card advantage
    skullclamp, faithless_looting, light_up_the_stage, wheel_of_fortune, senseis_divining_top,
    // The curve: bodies to attack three ways at once with
    dragon_fodder, krenkos_command, goblin_rabblemaster, beetleback_chief, goblin_chieftain,
    goblin_warchief, skirk_prospector, mavren_fein_dusk_apostle, goldnight_commander, hellrider,
    ranger_of_eos, solemn_simulacrum, archangel_of_tithes,
    // Finishers
    warstorm_surge, torbran_thane_of_red_fell, glorybringer,
    // Basics: 13 mountain, 12 plains
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains,
];

/// Teval, the Balanced Scale — BGU. Commander, then the 99.
pub const TEVAL_COMMANDERS: &[CardFactory] = &[teval_the_balanced_scale];

/// **Sultai Arisen**, the Tarkir: Dragonstorm Commander precon (TDC,
/// 2025-04-11), exactly as printed — MTGJSON's `SultaiArisen_TDC` list:
/// 84 nonbasic cards + 15 basics = 99. The pod's first official list, and
/// its graveyard seat: self-mill, "cards leave your graveyard" payoffs,
/// graveyard casts (Kotis, Lord of the Forsaken's graveyard-only mana).
pub const TEVAL_MAIN: &[CardFactory] = &[
    kotis_sibsig_champion, diviner_of_mist, afterlife_from_the_loam, tevals_judgment,
    welcome_the_dead, floral_evoker, steward_of_the_harvest, will_of_the_sultai,
    colossal_grave_reaver, gravecrawler, life_from_the_loam, casualties_of_war, amphin_mutineer,
    river_kelpie, dauthi_voidwalker, disciple_of_bolas, junji_the_midnight_sky, lethal_scheme,
    living_death, lord_of_the_forsaken, necromantic_selection, necropolis_fiend,
    noxious_gearhulk, ob_nixilis_the_fallen, tasigur_the_golden_fang, woe_strider,
    avenger_of_zendikar, conduit_of_worlds, multani_yavimayas_avatar, shigeki_jukai_visionary,
    consuming_aberration, jarad_golgari_lich_lord, lord_of_extinction, meren_of_clan_nel_toth,
    command_beacon, crypt_of_agadeem, darkwater_catacombs, dreamroot_cascade, drownyard_temple,
    exotic_orchard, fetid_pools, hinterland_harbor, llanowar_wastes, sunken_hollow,
    temple_of_malady, woodland_cemetery, essence_anchor, arcane_signet, sol_ring, command_tower,
    forbidden_alchemy, hedron_crab, treasure_cruise, wonder, phyrexian_reclamation,
    reassembling_skeleton, stitchers_supplier, victimize, kishla_skimmer, crawling_sensation,
    cultivate, farseek, grapple_with_the_past, harrow, opulent_palace, rampant_growth,
    sakura_tribe_elder, satyr_wayfinder, springbloom_druid, tear_asunder, timeless_witness,
    grisly_salvage, nyx_weaver, putrefy, skull_prophet, millikin, cephalid_coliseum,
    contaminated_aquifer, foreboding_landscape, golgari_rot_farm, haunted_mire,
    memorial_to_folly, myriad_landscape, terramorphic_expanse,
    // Basics: 4 island, 5 swamp, 6 forest
    island, island, island, island, swamp, swamp, swamp, swamp, swamp, forest, forest, forest,
    forest, forest, forest,
];

/// Captain N'ghathrod — UB. Commander, then the 99.
pub const NGHATHROD_COMMANDERS: &[CardFactory] = &[captain_nghathrod];

/// **Mind Flayarrrs**, the Commander Legends: Battle for Baldur's Gate precon
/// (CLB, 2022-06-10), exactly as MTGJSON's `MindFlayarrrs_CLB` prints it:
/// 79 nonbasic cards + 20 basics = 99. Horror tribal that mills opponents and
/// takes what lands in their graveyards; the pod's second official list.
pub const NGHATHROD_MAIN: &[CardFactory] = &[
    chasm_skulker, forgotten_creation, grazilaxx_illithid_scholar, hullbreaker_horror,
    mind_flayer, overcharged_amalgam, sludge_monster, wharf_infiltrator, dark_hatchling,
    dross_harvester, guiltfeeder, hunted_horror, nighthowler, nihilith, sewer_nemesis,
    woe_strider, consuming_aberration, nemesis_of_reason, phyrexian_revoker, psychosis_crawler,
    spellskite, dauthi_horror, dusk_mangler, phyrexian_rager, plague_spitter,
    ravenous_chupacabra, fractured_sanity, crippling_fear, hex, in_garruks_wake, feed_the_swarm,
    syphon_mind, extract_from_darkness, pull_from_tomorrow, curtains_call, memory_plunder,
    fact_or_fiction, drown_in_the_loch, arcane_signet, mind_stone, dimir_keyrune, dimir_signet,
    everflowing_chalice, heralds_horn, lightning_greaves, mindcrank, sol_ring,
    talisman_of_dominance, thought_vessel, leyline_of_anticipation, reflections_of_littjara,
    black_market, choked_estuary, creeping_tar_pit, darkwater_catacombs, drownyard_temple,
    exotic_orchard, nephalia_drownyard, river_of_tears, sunken_hollow, temple_of_deceit,
    command_tower, ash_barrens, dimir_aqueduct, myriad_landscape, path_of_ancestry,
    port_of_karfell, rogues_passage, tainted_isle, temple_of_the_false_god,
    zellix_sanity_flayer, haunted_one, aboleth_spawn, endless_evil, grell_philosopher,
    psionic_ritual, brainstealer_dragon, from_the_catacombs, uchuulon,
    // Basics: 9 island, 11 swamp
    island, island, island, island, island, island, island, island, island, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
];

/// Clavileño, First of the Blessed — WB. Commander, then the 99.
pub const CLAVILENO_COMMANDERS: &[CardFactory] = &[clavileno_first_of_the_blessed];

/// **Blood Rites**, The Lost Caverns of Ixalan Commander precon (LCC,
/// 2023-11-17), exactly as MTGJSON's `BloodRites_LCC` prints it: 78 nonbasic
/// cards + 21 basics = 99. Vampire aristocrats that turn Vampires into
/// Demons; the pod's third official list.
pub const CLAVILENO_MAIN: &[CardFactory] = &[
    carmen_cruel_skymarcher, charismatic_conqueror, elendas_hierophant, march_of_the_canonized,
    redemption_choir, dusk_legion_sergeant, master_of_dark_rites, promise_of_aclazotz,
    order_of_sacred_dusk, austere_command, kindred_boon, mavren_fein_dusk_apostle,
    radiant_destiny, welcoming_vampire, bloodghast, bloodtracker, butcher_of_malakir,
    champion_of_dusk, cordial_vampire, crossway_troublemakers, damn, drana_liberator_of_malakir,
    exquisite_blood, glass_cast_heart, new_blood, nighthawk_scavenger, olivias_wrath,
    pact_of_the_serpent, patron_of_the_vein, sanctum_seeker, timothar_baron_of_bats,
    twilight_prophet, yahenni_undying_partisan, elenda_the_dusk_rose, sorin_lord_of_innistrad,
    utter_end, vona_butcher_of_magan, blade_of_the_bloodchief, isolated_chapel,
    shineshadow_snarl, temple_of_silence, vault_of_the_archangel, voldaren_estate,
    windbrisk_heights, martyr_of_dusk, return_to_dust, swords_to_plowshares, blood_artist,
    bloodline_necromancer, dusk_legion_zealot, falkenrath_noble, indulgent_aristocrat,
    oathsworn_vampire, village_rites, viscera_seer, bartolome_del_presidio, cruel_celebrant,
    etchings_of_the_chosen, legion_lieutenant, arcane_signet, commanders_sphere, heirloom_blade,
    mind_stone, orzhov_signet, sol_ring, swiftfoot_boots, talisman_of_hierarchy,
    wayfarers_bauble, bojuka_bog, command_tower, myriad_landscape, orzhov_basilica,
    path_of_ancestry, rogues_passage, secluded_courtyard, tainted_field,
    temple_of_the_false_god, unclaimed_territory,
    // Basics: 8 plains, 13 swamp
    plains, plains, plains, plains, plains, plains, plains, plains, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
];

/// Zndrsplt, Eye of Wisdom **+** Okaun, Eye of Chaos — UR, a "Partner with"
/// pair (CR 702.124j). Both commanders, then the 98.
pub const ZNDRSPLT_COMMANDERS: &[CardFactory] = &[zndrsplt_eye_of_wisdom, okaun_eye_of_chaos];

/// **Heads I Win, Tails You Lose**, the Secret Lair Commander deck (SLD,
/// 2022-04-22), exactly as MTGJSON's `HeadsIWinTailsYouLose_SLD` prints it:
/// 98 cards. Coin flips; the pod's fourth official list and its first
/// "Partner with" pair.
pub const ZNDRSPLT_MAIN: &[CardFactory] = &[
    daretti_scrap_savant, ral_zarek, goblin_kaboomist, krark_the_thumbless, niv_mizzet_parun,
    spark_double, frenetic_sliver, goblin_archaeologist, karplusan_minotaur, tavern_scoundrel,
    tribute_mage, bloodsworn_steward, goblin_engineer, the_locust_god, yusri_fortunes_flame,
    mogg_assassin, sakashima_the_impostor, stitch_in_time, blasphemous_act, fabricate,
    fiery_gambit, chandras_ignition, gamble, reshape, seize_the_day, ponder, preordain,
    serum_visions, slip_through_space, squees_revenge, vandalblast, negate, chaos_warp,
    whir_of_invention, counterspell, long_term_plans, muddle_the_mixture, temur_battle_rage,
    krarks_thumb, izzet_signet, lightning_greaves, mind_stone, swiftfoot_boots,
    sword_of_vengeance, thought_vessel, whispersilk_cloak, boompile, commanders_plate,
    embercleave, shadowspear, arcane_signet, crooked_scales, sol_ring, talisman_of_creativity,
    propaganda, mirror_march, chance_encounter, footfall_crater, impulsive_maneuvers,
    planar_chaos, risky_move, temple_of_epiphany, wandering_fumarole, buried_ruin,
    great_furnace, izzet_boilerworks, myriad_landscape, path_of_ancestry, rogues_passage,
    academy_ruins, cascade_bluffs, desolate_lighthouse, exotic_orchard, flamekin_village,
    inventors_fair, shivan_reef, spinerock_knoll, sulfur_falls, tolaria_west, training_center,
    command_tower, reliquary_tower, temple_of_the_false_god,
    // Basics
    island, island, island, island, island, island, island, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain,
];

/// Zada, Hedron Grinder — mono-R. Commander, then the 99.
pub const ZADA_COMMANDERS: &[CardFactory] = &[zada_hedron_grinder];

/// **Goblin Storm**, the Secret Lair Commander deck (SLD, 2026-05-18), exactly
/// as MTGJSON's `GoblinStorm_SLD` prints it: 77 nonbasic cards + 22 Mountains
/// = 99. Goblins, rituals and spells aimed at Zada that copy onto the team.
pub const ZADA_MAIN: &[CardFactory] = &[
    krenko_mob_boss, pashalik_mons, brightstone_ritual, broadside_bombardiers,
    conspicuous_snoop, empty_the_warrens, grapeshot, skirk_prospector, roaming_throne,
    skullclamp, sol_ring, blasphemous_act, chaos_warp, frontline_heroism, goblin_bombardment,
    goblin_chieftain, goblin_dark_dwellers, goblin_lackey, goblin_trashmaster,
    great_train_heist, grenzo_havoc_raiser, howlsquad_heavy, past_in_flames,
    redcap_gutter_dweller, rundvelt_hordemaster, searslicer_goblin, siege_gang_commander,
    siege_gang_lieutenant, idol_of_oblivion, ruby_medallion, throne_of_eldraine, arena_of_glory,
    castle_embereth, den_of_the_bugbear, fountainport, kher_keep, spinerock_knoll, war_room,
    shinka_the_bloodsoaked_keep, ancestors_aid, battle_hymn, boggart_shenanigans, crimson_wisps,
    daring_discovery, dragon_fodder, expedite, faithless_looting, fists_of_flame,
    gempalm_incinerator, general_kreat_the_boltbringer, glimpse_the_impossible,
    goblin_bushwhacker, goblin_matron, goblin_negotiation, goblin_warchief, haze_of_rage,
    impact_tremors, impulsive_pilferer, krenkos_command, mana_geyser, mogg_war_marshal,
    quest_for_the_goblin_lord, renegade_tactics, sazacaps_brew, seething_song,
    spreading_insurrection, storm_kiln_artist, vandalblast, wild_ride, witchs_mark,
    swiftfoot_boots, dwarven_mine, forgotten_cave, goblin_burrows, hidden_volcano,
    reliquary_tower, smoldering_crater,
    // Basics: 22 mountain
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain,
];

pub const GISA_COMMANDERS: &[CardFactory] = &[ghoulcaller_gisa];

/// **Wretched Ranks**, the Foundations Commander deck (FDC, 2026-10-02),
/// exactly as MTGJSON's `WretchedRanks_FDC` prints it: 66 nonbasic cards +
/// 33 Swamps = 99. Mono-black Zombies: token makers, sacrifice outlets and
/// Gisa turning the horde into more of itself.
pub const GISA_MAIN: &[CardFactory] = &[
    army_of_the_damned, ayara_first_of_locthwain, bad_moon, cemetery_reaper,
    champion_of_the_perished, cryptbreaker, death_baron, diregraf_colossus,
    endless_ranks_of_the_dead, god_eternal_bontu, grave_titan, graveborn_muse, gravecrawler,
    headless_rider, josu_vess_lich_knight, kalitas_traitor_of_ghet, lilianas_mastery,
    lilianas_reaver, lord_of_the_undead, midnight_reaper, mutilate, necrotic_hex,
    open_the_graves, oversold_cemetery, phyrexian_arena, razorlash_transmogrant,
    zul_ashur_lich_lord, castle_locthwain, geier_reach_sanitarium, ambitions_cost,
    carrion_feeder, cemetery_recruitment, consumed_by_greed, consuming_corruption,
    eternal_taskmaster, fleshbag_marauder, go_for_the_throat, gray_merchant_of_asphodel,
    lord_of_the_accursed, marchesas_decree, mire_triton, moan_of_the_unhallowed, nights_whisper,
    noxious_ghoul, sign_in_blood, soulless_one, syphon_flesh, tendrils_of_corruption,
    undead_augur, undead_butler, undead_warchief, vengeful_dead, wight_of_precinct_six,
    withering_torment, arcane_signet, bontus_monument, charcoal_diamond, commanders_sphere,
    infernal_idol, mind_stone, patchwork_banner, sol_ring, barren_moor, bojuka_bog,
    memorial_to_folly, witchs_cottage,
    // Basics: 33 swamp
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, swamp, swamp, swamp,
];

pub const GHALTA_COMMANDERS: &[CardFactory] = &[ghalta_primal_hunger];

/// **Tramplesaurus Rex**, the Foundations Commander deck (FDC, 2026-10-02),
/// exactly as MTGJSON's `TramplesaurusRex_FDC` prints it: 67 nonbasic cards +
/// 32 Forests = 99. Mono-green stompy: mana creatures, big Dinosaurs and a
/// twelve-drop commander that costs {X} less for their power.
pub const GHALTA_MAIN: &[CardFactory] = &[
    arachnogenesis, arasta_of_the_endless_web, beast_whisperer, birds_of_paradise,
    carnage_tyrant, curious_altisaur, dungrove_elder, elder_gargaroth, ezuris_predation,
    gigantosaurus, hulking_raptor, loot_exuberant_explorer, managorger_hydra,
    overwhelming_stampede, pugnacious_hammerskull, regal_imperiosaur, rhonas_the_indomitable,
    ripjaw_raptor, rishkars_expertise, scavenging_ooze, scrapshooter, shamanic_revelation,
    steel_leaf_champion, surrak_and_goreclaw, surrak_the_hunt_caller, tangleweave_armor,
    thickest_in_the_thicket, unnatural_growth, verdant_suns_avatar, whiptongue_hydra,
    yeva_natures_herald, bonders_enclave, mosswort_bridge, scavenger_grounds, war_room,
    witchs_clinic, beast_within, bite_down, challenger_troll, clifftop_lookout,
    collective_resistance, colossal_majesty, elemental_bond, elvish_mystic, fyndhorn_elves,
    garruks_packleader, garruks_uprising, goreclaw_terror_of_qal_sisma, harmonize,
    ilysian_caryatid, kenriths_transformation, llanowar_elves, llanowar_tribe,
    monstrous_onslaught, paradise_druid, ram_through, rishkar_peema_renegade,
    tamiyos_safekeeping, terrian_world_tyrant, thrashing_brontodon, whisperer_of_the_wilds,
    commanders_sphere, rhonass_monument, sol_ring, swiftfoot_boots, rogues_passage,
    tranquil_thicket,
    // Basics: 32 forest
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
];

pub const SAI_COMMANDERS: &[CardFactory] = &[sai_master_thopterist];

/// **Keen Engineering**, the Foundations Commander deck (FDC, 2026-10-02),
/// exactly as MTGJSON's `KeenEngineering_FDC` prints it: 65 nonbasic cards +
/// 34 Islands = 99. Mono-blue artifacts: Thopters off every artifact cast,
/// mana rocks, and Vehicles for Sai's fliers to crew.
pub const SAI_MAIN: &[CardFactory] = &[
    all_is_dust, broodstar, kappa_cannoneer, master_of_etherium, master_transmuter,
    misleading_signpost, pull_from_tomorrow, research_thief, shimmer_dragon, thopter_fabricator,
    thopter_spy_network, thought_monitor, vedalken_archmage, adaptive_omnitool,
    cultivators_caravan, darksteel_juggernaut, duplicant, forsaken_monument,
    graaz_unstoppable_juggernaut, mazemind_tome, minds_eye, myr_battlesphere, nettlecyst,
    nevinyrrals_disk, psychosis_crawler, scrawling_crawler, skysovereign_consul_flagship,
    steel_hellkite, steel_overseer, war_room, aetherize, counterspell, etherium_sculptor,
    fall_from_favor, launch_mishap, memory_guardian, negate, padeem_consul_of_innovation,
    propaganda, tamiyos_logbook, thirst_for_knowledge, thoughtcast, whirler_rogue,
    aether_spellbomb, arcane_signet, chief_of_the_foundry, foundry_inspector, hedron_archive,
    ichor_wellspring, meteor_golem, mind_stone, myr_retriever, ornithopter_of_paradise,
    palladium_myr, shimmer_myr, silver_myr, sol_ring, soul_guide_lantern, spire_golem,
    thought_vessel, buried_ruin, darksteel_citadel, foundry_of_the_consuls, lonely_sandbar,
    remote_isle,
    // Basics: 34 island
    island, island, island, island, island, island, island, island, island, island, island, island,
    island, island, island, island, island, island, island, island, island, island, island, island,
    island, island, island, island, island, island, island, island, island, island,
];

pub const AESI_COMMANDERS: &[CardFactory] = &[aesi_tyrant_of_gyre_strait];

/// **Reap the Tides**, the Commander Legends deck (CMR, 2020-11-20), exactly
/// as MTGJSON's `ReapTheTides_CMR` prints it: 69 nonbasic cards + 15 Forests +
/// 15 Islands = 99. Simic lands-matter: ramp spells and landfall draws into
/// big blue-green creatures.
pub const AESI_MAIN: &[CardFactory] = &[
    coiling_oracle, eternal_witness, ramunap_excavator, reclamation_sage, stumpsquall_hydra,
    yavimaya_elder, fathom_mage, sharktocrab, wickerbough_elder, acidic_slime,
    meloku_the_clouded_mirror, mulldrifter, murkfiend_liege, sporemound, rampaging_baloths,
    shipbreaker_kraken, avenger_of_zendikar, meteor_golem, molimo_maro_sorcerer,
    nezahal_primal_tide, scourge_of_fleets, simic_sky_swallower, sphinx_of_uthuun, trench_behemoth,
    tromokratis, verdant_suns_avatar, elder_deep_fiend, slinn_voda_the_rising_deep,
    stormtide_leviathan, terastodon, arcane_denial, counterspell, growth_spiral, into_the_roil,
    peel_from_reality, simic_charm, beast_within, fact_or_fiction, explore, rampant_growth,
    compulsive_research, cultivate, kodamas_reach, search_for_tomorrow, harmonize, whelming_wave,
    urban_evolution, spitting_image, ior_ruin_expedition, khalni_heart_expedition,
    retreat_to_kazandu, sol_ring, simic_signet, swiftfoot_boots, seers_sundial, blighted_woodland,
    command_tower, coral_atoll, evolving_wilds, jungle_basin, memorial_to_genius, reliquary_tower,
    simic_growth_chamber, simic_guildgate, terramorphic_expanse, thornwood_falls, vivid_creek,
    vivid_grove, woodland_stream,
    // Basics: 15 forest, 15 island
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, island, island, island, island, island, island, island, island, island,
    island, island, island, island, island, island,
];

pub const IXHEL_COMMANDERS: &[CardFactory] = &[ixhel_scion_of_atraxa];

/// **Corrupting Influence**, the Phyrexia: All Will Be One Commander deck
/// (ONC, 2023-02-10), exactly as MTGJSON's `CorruptingInfluence_ONC` prints
/// it: 79 nonbasic cards + 6 Plains + 6 Swamps + 8 Forests = 99.
/// Abzan poison: toxic and infect creatures, proliferate, and corrupted
/// payoffs keyed to opponents at three poison counters.
pub const IXHEL_MAIN: &[CardFactory] = &[
    bilious_skulldweller, blight_mamba, blightbelly_rat, cankerbloom, contaminant_grafter,
    evolution_sage, glissas_retriever, grateful_apparition, ichor_rats, ichorclaw_myr,
    mycosynth_fiend, myr_convert, norns_choirmaster, pestilent_syphoner, phyrexian_swarmlord,
    plague_myr, plague_stinger, scavenging_ooze, venomous_brutalizer, viridian_corrupter,
    vishgraz_the_doomhive, windborn_muse, caress_of_phyrexia, culling_ritual, cultivate,
    expand_the_sphere, feed_the_infection, fumigate, geths_summons, infectious_inquiry,
    merciless_eviction, nights_whisper, noxious_assault, painful_truths, phyresis_outbreak,
    phyrexian_rebirth, unnatural_restoration, vat_emergence, wurmquake, beast_within, carrion_call,
    mortify, noxious_revival, putrefy, swords_to_plowshares, vraskas_fall, ghostly_prison,
    moldervine_reclamation, norns_decree, arcane_signet, chromatic_lantern, commanders_sphere,
    contagion_clasp, fellwar_stone, glistening_sphere, golgari_signet, grafted_exoskeleton,
    norns_annex, phyrexian_atlas, sol_ring, trailblazers_boots, bojuka_bog, canopy_vista,
    command_tower, exotic_orchard, fortified_village, karns_bastion, krosan_verge,
    myriad_landscape, necroblossom_snarl, path_of_ancestry, sandsteppe_citadel, shineshadow_snarl,
    sungrass_prairie, tainted_field, tainted_wood, temple_of_malady, temple_of_plenty,
    temple_of_silence,
    // Basics: 6 plains, 6 swamp, 8 forest
    plains, plains, plains, plains, plains, plains, swamp, swamp, swamp, swamp, swamp, swamp,
    forest, forest, forest, forest, forest, forest, forest, forest,
];

pub const ANOWON_COMMANDERS: &[CardFactory] = &[anowon_the_ruin_thief];

/// **Sneak Attack**, the Zendikar Rising Commander deck (ZNC, 2020-09-25),
/// exactly as MTGJSON's `SneakAttack_ZNC` prints it: 69 nonbasic cards +
/// 15 Islands + 15 Swamps = 99. Dimir Rogues: evasive attackers that
/// mill the players they hit, and payoffs keyed to full graveyards.
pub const ANOWON_MAIN: &[CardFactory] = &[
    lazav_dimir_mastermind, enigma_thief, scourge_of_fleets, gonti_lord_of_luxury, nighthowler,
    ogre_slumlord, sepulchral_primordial, consuming_aberration, notion_thief,
    oona_queen_of_the_fae, sygg_river_cutthroat, faerie_vandal, invisible_stalker, latchkey_faerie,
    marang_river_prowler, master_thief, nightveil_sprite, slither_blade, triton_shorestalker,
    whirler_rogue, changeling_outcast, frogtosser_banneret, marsh_flitter, oonas_blackguard,
    stinkdrinker_bandit, syr_konrad_the_grim, zulaport_cutthroat, merfolk_windrobber,
    sure_footed_infiltrator, soaring_thought_thief, notorious_throng, stolen_identity,
    in_garruks_wake, necromantic_selection, distant_melody, open_into_wonder, endless_obedience,
    rise_from_the_grave, extract_from_darkness, fated_return, silumgars_command, spinal_embrace,
    aetherize, fact_or_fiction, murder, price_of_fame, soul_manipulation, whispersteel_dagger,
    blackblade_reforged, bonehoard, obelisk_of_urd, scytheclaw, arcane_signet, commanders_sphere,
    dimir_keyrune, dimir_locket, dimir_signet, heirloom_blade, mind_stone, sol_ring,
    military_intelligence, command_tower, dimir_aqueduct, dimir_guildgate, dismal_backwater,
    jwar_isle_refuge, myriad_landscape, rogues_passage, submerged_boneyard,
    // Basics: 15 island, 15 swamp
    island, island, island, island, island, island, island, island, island, island, island, island,
    island, island, island, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, swamp,
];

pub const SHIKO_COMMANDERS: &[CardFactory] = &[shiko_and_narset_unified];

/// **Jeskai Striker**, the Tarkir: Dragonstorm Commander deck (TDC,
/// 2025-04-11), exactly as MTGJSON's `JeskaiStriker_TDC` prints it: 85
/// nonbasic cards + 4 Plains + 5 Islands + 5 Mountains = 99. Jeskai
/// spellslinging: flurry, prowess Monks and free casts off instants and
/// sorceries.
pub const SHIKO_MAIN: &[CardFactory] = &[
    elsha_threefold_master, aligned_heart, tempest_technique, adaptive_training_post,
    transcendent_dragon, voracious_bibliophile, caldera_pyremaw, transforming_flourish,
    will_of_the_jeskai, vanquish_the_horde, narsets_reversal, dismantling_wave,
    mangara_the_diplomat, monastery_mentor, ancestral_vision, archmage_emeritus, barals_expertise,
    curse_of_the_swine, haughty_djinn, lier_disciple_of_the_drowned, rite_of_replication,
    sublime_epiphany, electrodominance, manaform_hellkite, baral_and_kari_zev, expansion_explosion,
    magma_opus, prismari_command, time_wipe, velomachus_lorehold, veyran_voice_of_duality,
    whirlwind_of_thought, adarkar_wastes, battlefield_forge, cascade_bluffs, clifftop_retreat,
    exotic_orchard, ferrous_lake, glacial_fortress, irrigated_farmland, prairie_stream,
    rugged_prairie, shivan_reef, skycloud_expanse, sulfur_falls, temple_of_enlightenment,
    temple_of_epiphany, temple_of_triumph, young_pyromancer, goblin_electromancer, arcane_signet,
    sol_ring, command_tower, ghostly_prison, swords_to_plowshares, compulsive_research, consider,
    deep_analysis, frantic_search, opt, ponder, pongify, preordain, think_twice, abrade, big_score,
    curse_of_opulence, faithless_looting, guttersnipe, mana_geyser, shiny_impetus,
    storm_kiln_artist, evolving_wilds, mystic_monastery, expressive_iteration,
    third_path_iconoclast, azorius_signet, boros_signet, fellwar_stone, izzet_signet,
    talisman_of_progress, ash_barrens, path_of_ancestry, perilous_landscape, reliquary_tower,
    // Basics: 4 plains, 5 island, 5 mountain
    plains, plains, plains, plains, island, island, island, island, island, mountain, mountain,
    mountain, mountain, mountain,
];

pub const GRAVEMOTHER_COMMANDERS: &[CardFactory] = &[sliver_gravemother];

/// **Sliver Swarm**, the Commander Masters deck (CMM, 2023-08-04), exactly as
/// MTGJSON's `SliverSwarm_CMM` prints it: 88 nonbasic cards + 11 basics
/// = 99. Five-color Slivers: lords that grant to the whole hive, and a
/// commander that re-buys the dead ones with encore.
pub const GRAVEMOTHER_MAIN: &[CardFactory] = &[
    rukarumel_biologist, regal_sliver, taunting_sliver, titan_of_littjara, lazotep_sliver,
    capricious_sliver, descendants_fury, for_the_ancestors, hatchery_sliver, bonescythe_sliver,
    cleansing_nova, harsh_mercy, galerider_sliver, synapse_sliver, crippling_fear,
    syphon_sliver, spiteful_sliver, brood_sliver, megantic_sliver, realmwalker,
    cloudshredder_sliver, decimate, sliver_hivelord, icon_of_ancestry, vanquishers_banner,
    canopy_vista, cinder_glade, exotic_orchard, irrigated_farmland, prairie_stream,
    scattered_groves, sheltered_thicket, smoldering_marsh, sunken_hollow, arcane_signet,
    fellwar_stone, sol_ring, ash_barrens, command_tower, path_of_ancestry, constricting_sliver,
    sentinel_sliver, sinew_sliver, diffusion_sliver, distant_melody, shifting_sliver, windfall,
    winged_sliver, clot_sliver, crypt_sliver, blade_sliver, blur_sliver, bonesplitter_sliver,
    cleaving_sliver, hollowhead_sliver, striking_sliver, two_headed_sliver, cultivate, farseek,
    gemhide_sliver, manaweft_sliver, might_sliver, natures_lore, quick_sliver, three_visits,
    venom_sliver, crystalline_sliver, firewake_sliver, harmonic_sliver, hibernation_sliver,
    lavabelly_sliver, necrotic_sliver, heralds_horn, pillar_of_origins, flood_plain,
    frontier_bivouac, grasslands, jungle_shrine, mountain_valley, mystic_monastery,
    nomad_outpost, opulent_palace, rocky_tar_pit, sandsteppe_citadel, savage_lands,
    seaside_citadel, secluded_courtyard, unclaimed_territory,
    // Basics: 2 plains, 2 island, 2 swamp, 2 mountain, 3 forest
    plains, plains, island, island, swamp, swamp, mountain, mountain, forest, forest, forest,
];

pub const STELLA_COMMANDERS: &[CardFactory] = &[stella_lee_wild_card];

/// **Quick Draw**, the Outlaws of Thunder Junction Commander deck (OTC,
/// 2024-04-19), exactly as MTGJSON's `QuickDraw_OTC` prints it: 72 nonbasic
/// cards + 14 Islands + 13 Mountains = 99. Izzet spellslinging: second-spell
/// payoffs, storm and cascade, and copies off the commander.
pub const STELLA_MAIN: &[CardFactory] = &[
    eris_roar_of_the_storm, archmage_emeritus, barals_expertise, tezzerets_gambit,
    midnight_clock, haughty_djinn, dig_through_time, winged_boots, talrand_sky_summoner,
    octavia_living_thesis, thunderclap_drake, lock_and_load, forgers_foundry,
    finale_of_revelation, curse_of_the_swine, mizzixs_mastery, chaos_warp,
    crackling_spellslinger, pyretic_charge, smoldering_stagecoach, elemental_eruption,
    rousing_refrain, cursed_mirror, bloodthirsty_adversary, finale_of_promise,
    arcane_bombardment, niv_mizzet_parun, kaza_roil_chaser, shark_typhoon, galvanic_iteration,
    epic_experiment, veyran_voice_of_duality, leyline_dowser, exotic_orchard, shivan_reef,
    sulfur_falls, frostboil_snarl, temple_of_epiphany, cascade_bluffs, ferrous_lake,
    treasure_cruise, preordain, ponder, murmuring_mystic, deep_analysis, radical_idea, opt,
    think_twice, arcane_denial, pteramander, vandalblast, pongify, serum_visions,
    faithless_looting, big_score, storm_kiln_artist, young_pyromancer, electrostatic_field,
    guttersnipe, volcanic_torrent, windfall, goblin_electromancer, third_path_iconoclast,
    expressive_iteration, arcane_signet, izzet_signet, sol_ring, command_tower,
    temple_of_the_false_god, reliquary_tower, izzet_boilerworks, propaganda,
    // Basics: 14 island, 13 mountain
    island, island, island, island, island, island, island, island, island, island, island,
    island, island, island, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain,
];
