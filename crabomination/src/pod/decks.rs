//! The Commander pod's target decks.
//!
//! Ten hand-picked commanders plus forty official preconstructed lists, each a
//! 99 of cards this engine implements: legal under CR 903 (100 cards
//! including the commander, singleton outside basics, every card inside the
//! commander's CR 903.4 color identity) and Commander-legal per Scryfall's
//! ban list, both of which `pod::tests` asserts rather than trusts.
//!
//! The ten were built by hand because the offline Scryfall cache cannot say
//! how a precon splits into decks; MTGJSON's deck files can, and the eleventh
//! through fiftieth (Sultai Arisen, Mind Flayarrrs, Blood Rites, Heads I
//! Win, Tails You Lose, Goblin Storm, Foundations Commander's Wretched Ranks,
//! Tramplesaurus Rex and Keen Engineering, Commander Legends' Reap the Tides,
//! Phyrexia: All Will Be One's Corrupting Influence, Zendikar Rising's Sneak
//! Attack, Tarkir: Dragonstorm's Jeskai Striker, Commander Masters' Sliver
//! Swarm, Outlaws of Thunder Junction's Quick Draw, Commander 2017's
//! Vampiric Bloodlust, Foundations Commander's Calling All Angels,
//! Commander 2014's Guided by Nature, Bloomburrow Commander's Animated Army,
//! Secret Lair's Grave Danger, Commander 2014's Forged in Stone, Modern
//! Horizons 3 Commander's Graveyard Overdrive, Commander 2015's Swell the
//! Host, Secret Lair's Angels: They're Just Like Us, Commander 2014's Built From
//! Scratch and Sworn to Darkness, Innistrad: Crimson Vow Commander's Vampiric
//! Bloodline, Commander 2015's Plunder the Graves and Seize Control,
//! Commander 2021's Quantum Quandrix, Commander 2015's Call the Spirits,
//! Phyrexia: All Will Be One's Rebellion Rising, Commander 2014's Peer
//! Through Time, Commander 2021's Lorehold Legacies, Commander (2011)'s
//! Counterpunch, Wilds of Eldraine Commander's Fae Dominion, Commander 2015's
//! Wade into Battle, Commander Legends' Arm for Battle, Commander 2016's Invent
//! Superiority, Secret Lair's Chaos Incarnate and Aetherdrift Commander's
//! Eternal Might) are taken from one card for card. What all of them keep
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

pub const EDGAR_C17_COMMANDERS: &[CardFactory] = &[edgar_markov];

/// **Vampiric Bloodlust**, the Commander 2017 deck (C17, 2017-08-25), exactly
/// as MTGJSON's `VampiricBloodlust_C17` prints it: 84 nonbasic cards + 8
/// Swamps + 4 Mountains + 3 Plains = 99. Mardu Vampires under Edgar Markov —
/// the precon the hand-built Edgar seat was modelled on, curses and all.
pub const EDGAR_C17_MAIN: &[CardFactory] = &[
    licia_sanguine_tribune, mathas_fiend_seeker, kheru_mind_eater, patron_of_the_vein,
    bloodsworn_steward, crimson_honor_guard, anowon_the_ruin_sage, bloodlord_of_vaasgoth,
    blood_baron_of_vizkopa, butcher_of_malakir, captivating_vampire, dark_impostor,
    drana_kalastria_bloodchief, malakir_bloodwitch, sangromancer, skeletal_vampire,
    vein_drinker, bloodline_necromancer, blood_artist, bloodhusk_ritualist, falkenrath_noble,
    pawn_of_ulamog, vampire_nighthawk, rakish_heir, stromkirk_captain, tithe_drinker,
    new_blood, disrupt_decorum, kindred_charge, fell_the_mighty, blood_tribute,
    consuming_vapors, damnable_pact, merciless_eviction, ambitions_cost, read_the_bones,
    syphon_mind, teferis_protection, crackling_doom, return_to_dust, swords_to_plowshares,
    go_for_the_throat, skeletal_scrying, mortify, blade_of_the_bloodchief, door_of_destinies,
    well_of_lost_dreams, heirloom_blade, boros_signet, orzhov_signet, rakdos_signet,
    skullclamp, sol_ring, worn_powerstone, kindred_boon, blind_obedience, black_market,
    sanguine_bond, underworld_connections, outpost_siege, curse_of_vitality,
    curse_of_disturbance, path_of_ancestry, akoum_refuge, bloodfell_caves, bojuka_bog,
    boros_garrison, boros_guildgate, cinder_barrens, command_tower, evolving_wilds,
    forsaken_sanctuary, kabira_crossroads, nomad_outpost, opal_palace, orzhov_basilica,
    orzhov_guildgate, rakdos_carnarium, rakdos_guildgate, scoured_barrens, stone_quarry,
    terramorphic_expanse, urborg_volcano, wind_scarred_crag,
    // Basics: 8 swamp, 4 mountain, 3 plains
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, mountain, mountain, mountain,
    mountain, plains, plains, plains,
];

pub const GIADA_COMMANDERS: &[CardFactory] = &[giada_font_of_hope];

/// **Calling All Angels**, the Foundations Commander deck (FDC), exactly as
/// MTGJSON's `CallingAllAngels_FDC` prints it: 67 nonbasic cards + 32
/// Plains = 99. Mono-white Angels: Giada's growing entries, lieutenants, the
/// monarch, and the field's first mono-white identity.
pub const GIADA_MAIN: &[CardFactory] = &[
    always_watching, angel_of_the_ruins, angelic_destiny, angelic_field_marshal, angelic_sleuth,
    archangel_of_tithes, austere_command, bishop_of_wings, cleansing_nova, court_of_grace,
    day_of_judgment, emeria_shepherd, exemplar_of_light, fateful_absence, firemane_commando,
    grasp_of_fate, herald_of_eternal_dawn, herald_of_war, linvala_the_preserver,
    lyra_dawnbringer, merchant_of_truth, metropolis_reformer, norns_choirmaster,
    reya_dawnbringer, righteous_valkyrie, search_the_premises, sephara_skys_blade,
    seraph_of_the_sword, serra_avenger, speaker_of_the_heavens, sunblast_angel,
    wojek_investigator, endless_atlas, metallic_mimic, tome_of_legends, vanquishers_banner,
    bonders_enclave, war_room, angel_of_finality, angel_of_vitality, cut_a_deal, dazzling_angel,
    defy_death, destroy_evil, exorcise, inspiring_overseer, invoke_the_divine,
    secret_rendezvous, segovian_angel, starnheim_aspirant, swords_to_plowshares,
    thraben_watcher, valorous_stance, vanguard_seraph, youthful_valkyrie, arcane_signet,
    commanders_sphere, heraldic_banner, marble_diamond, mind_stone, patchwork_banner, sol_ring,
    swiftfoot_boots, radiant_fountain, secluded_steppe, seraph_sanctuary,
    temple_of_the_false_god,
    // Basics: 32 plains
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
];

pub const FREYALISE_C14_COMMANDERS: &[CardFactory] = &[freyalise_llanowars_fury];

/// **Guided by Nature**, the Commander 2014 deck (C14, 2014-11-07), exactly as
/// MTGJSON's `GuidedByNature_C14` prints it: 74 nonbasic cards + 25
/// Forests = 99. Mono-green Elves under the field's second planeswalker
/// commander (CR 903.3a).
pub const FREYALISE_C14_MAIN: &[CardFactory] = &[
    elvish_mystic, elvish_skysweeper, essence_warden, joraga_warcaller, llanowar_elves,
    sylvan_safekeeper, elvish_visionary, priest_of_titania, sylvan_ranger, thornweald_archer,
    wellwisher, farhaven_elf, imperious_perfect, reclamation_sage, timberwatch_elf,
    titanias_chosen, wood_elves, elvish_archdruid, ezuri_renegade_leader, drove_of_elves,
    immaculate_magistrate, wrens_run_packmaster, lys_alana_huntmaster, masked_admirers,
    wolfbriar_elemental, creeperhulk, silklash_spider, titania_protector_of_argoth,
    grave_sifter, primordial_sage, rampaging_baloths, soul_of_the_harvest, thunderfoot_baloth,
    siege_behemoth, tornado_elemental, terastodon, lifeblood_hydra, hunting_triad, whirlwind,
    overwhelming_stampede, overrun, grim_flowering, collective_unconscious, desert_twister,
    wave_of_vitriol, praetors_counsel, sylvan_offering, harrow, fresh_meat, skullclamp,
    sol_ring, emerald_medallion, moss_diamond, swiftfoot_boots, commanders_sphere,
    assault_suit, seers_sundial, predator_flagship, loreseekers_stone, beastmaster_ascension,
    song_of_the_dryads, wolfcallers_howl, crystal_vein, evolving_wilds, gargoyle_castle,
    ghost_quarter, haunted_fengraf, havenwood_battleground, jungle_basin, myriad_landscape,
    oran_rief_the_vastwood, slippery_karst, terramorphic_expanse, tranquil_thicket,
    // Basics: 25 forest
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest,
];

pub const BELLO_COMMANDERS: &[CardFactory] = &[bello_bard_of_the_brambles];

/// **Animated Army**, the Bloomburrow Commander deck (BLC, 2024-08-02),
/// exactly as MTGJSON's `AnimatedArmy_BLC` prints it: 81 nonbasic cards +
/// 10 Forests + 8 Mountains = 99. Gruul artifacts and enchantments that
/// Bello animates on its turn — the field's first RG identity.
pub const BELLO_MAIN: &[CardFactory] = &[
    wildsear_scouring_maw, domri_anarch_of_bolas, etali_primal_storm, prosperous_bandit,
    pyreswipe_hawk, alchemists_talent, berserkers_onslaught, outpost_siege, rain_of_riches,
    chaos_warp, sunbirds_invocation, gratuitous_violence, warstorm_surge, starstorm,
    lotus_cobra, evercoat_ursine, brightcap_badger, trailtracker_scout, thickest_in_the_thicket,
    ghalta_primal_hunger, esikas_chariot, unnatural_growth, greater_good,
    kodama_of_the_east_tree, grothama_all_devouring, rampaging_baloths, bootleggers_stash,
    primeval_bounty, gilded_lotus, spine_of_ish_sah, temple_of_abandon, karplusan_forest,
    exotic_orchard, sheltered_thicket, game_trail, raging_ravine, copperline_gorge,
    mossfire_valley, cinder_glade, rootbound_crag, mosswort_bridge, blasphemous_act,
    llanowar_loamspeaker, tendershoot_dryad, goreclaw_terror_of_qal_sisma, path_of_discovery,
    decimate, rolling_hamsphere, teapot_slinger, explore, farseek, cultivate,
    grumgully_the_generous, wandertale_mentor, thought_vessel, arcane_signet, wooded_ridgeline,
    big_score, abrade, rampant_growth, sakura_tribe_elder, beast_within, garruks_packleader,
    harmonize, garruks_uprising, gruul_signet, burnished_hart, hedron_archive, fellwar_stone,
    thran_dynamo, sol_ring, mind_stone, talisman_of_impulse, terramorphic_expanse,
    path_of_ancestry, gruul_turf, evolving_wilds, forgotten_cave, tranquil_thicket,
    command_tower, reliquary_tower,
    // Basics: 10 forest, 8 mountain
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain,
];

pub const GISA_GERALF_COMMANDERS: &[CardFactory] = &[gisa_and_geralf];

/// **Grave Danger**, the Secret Lair Commander deck (SCD, 2022-12-02), exactly
/// as MTGJSON's `GraveDanger_SCD` prints it: 68 nonbasic cards + 31
/// basics = 99. Dimir Zombies that live in the graveyard: Gisa and Geralf
/// recast one a turn, and the rest mill to feed them.
pub const GISA_GERALF_MAIN: &[CardFactory] = &[
    geralfs_mindcrusher, laboratory_drudge, army_of_the_damned, cemetery_reaper,
    champion_of_the_perished, crippling_fear, gravespawn_sovereign, josu_vess_lich_knight,
    liliana_untouched_by_death, lilianas_mastery, lilianas_standard_bearer, midnight_reaper,
    necromantic_selection, necrotic_hex, open_the_graves, overseer_of_the_damned,
    scourge_of_nel_toth, unbreathing_horde, zombie_apocalypse, enter_the_god_eternals,
    havengul_lich, undermine, vela_the_night_clad, grimoire_of_the_dead, choked_estuary,
    sunken_hollow, temple_of_deceit, deep_analysis, distant_melody, eternal_skylord,
    lazotep_plating, sinister_sabotage, cruel_revival, curse_of_disturbance, feed_the_swarm,
    fleshbag_marauder, gray_merchant_of_asphodel, lazotep_reaver, lilianas_devotee,
    lord_of_the_accursed, lotleth_giant, loyal_subordinate, mire_triton, murder, spark_reaper,
    syphon_flesh, undead_augur, vampiric_rites, vengeful_dead, victimize,
    vizier_of_the_scorpion, withered_wretch, diregraf_captain, gleaming_overseer,
    pilfered_plans, arcane_signet, commanders_sphere, dimir_signet, heraldic_banner, sol_ring,
    talisman_of_dominance, unstable_obelisk, wayfarers_bauble, command_tower, dismal_backwater,
    jwar_isle_refuge, salt_marsh, submerged_boneyard,
    // Basics: 13 island, 18 swamp
    island, island, island, island, island, island, island, island, island, island, island,
    island, island, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
];

pub const NAHIRI_COMMANDERS: &[CardFactory] = &[nahiri_the_lithomancer];

/// **Forged in Stone**, the Commander 2014 deck (C14, 2014-11-07), exactly as
/// MTGJSON's `ForgedInStone_C14` prints it: 67 nonbasic cards + 32
/// basics = 99. Mono-white Equipment and tokens under Nahiri, the field's
/// third planeswalker commander (CR 903.3a).
pub const NAHIRI_MAIN: &[CardFactory] = &[
    containment_priest, whitemane_lion, grand_abolisher, kor_sanctifiers, mentor_of_the_meek,
    flickerwisp, hallowed_spiritkeeper, kemba_kha_regent, silverblade_paladin,
    skyhunter_skirmisher, angelic_field_marshal, celestial_crusader, jazal_goldmane,
    geist_honored_monk, requiem_angel, adarkar_valkyrie, sun_titan, sunblast_angel,
    twilight_shepherd, angel_of_the_dire_hour, serra_avatar, gift_of_estates,
    spectral_procession, fell_the_mighty, nomads_assembly, deploy_to_the_front, martial_coup,
    decree_of_justice, brave_the_elements, condemn, afterlife, midnight_haunting, oblation,
    wing_shards, benevolent_offering, comeuppance, return_to_dust, white_suns_zenith,
    masterwork_of_ingenuity, skullclamp, sol_ring, marble_diamond, mask_of_memory,
    pearl_medallion, swiftfoot_boots, commanders_sphere, loxodon_warhammer, strata_scythe,
    sword_of_vengeance, assault_suit, bonehoard, moonsilver_spear, argentum_armor,
    loreseekers_stone, armistice, mobilization, sacred_mesa, marshals_anthem, cathars_crusade,
    true_conviction, arcane_lighthouse, drifting_meadow, emeria_the_sky_ruin, ghost_quarter,
    karoo_land, secluded_steppe, temple_of_the_false_god,
    // Basics: 32 plains
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
];

pub const DISA_COMMANDERS: &[CardFactory] = &[disa_the_restless];

/// **Graveyard Overdrive**, the Modern Horizons 3 Commander deck (M3C,
/// 2024-06-14), exactly as MTGJSON's `GraveyardOverdrive_M3C` prints it:
/// 87 nonbasic cards + 12 basics = 99. Jund Lhurgoyfs off a full graveyard —
/// the field's first BRG identity.
pub const DISA_MAIN: &[CardFactory] = &[
    coram_the_undertaker, bloodbraid_challenger, broodmate_tyrant, tempt_with_mayhem,
    gluttonous_hellkite, pyrogoyf, polygoyf, barrowgoyf, sawhorn_nemesis, infested_thrinax,
    final_act, siege_gang_lieutenant, tarmogoyf_nest, exterminator_magmarch,
    liliana_deaths_majesty, maelstrom_pulse, junji_the_midnight_sky, garruk_apex_predator,
    the_reaver_cleaver, temple_of_malady, deadbridge_chant, kolaghans_command,
    izoni_thousand_eyed, lhurgoyf, selvala_heart_of_the_wilds, kessig_wolf_run,
    archon_of_cruelty, maskwood_nexus, grist_the_hunger_tide, ignoble_hierarch, necrogoyf,
    mortivore, chandras_ignition, viridescent_bog, find_finality, mossfire_valley,
    ziatora_the_incinerator, canyon_slough, cinder_glade, exotic_orchard, shadowblood_ridge,
    sheltered_thicket, smoldering_marsh, temple_of_abandon, temple_of_malice, raging_ravine,
    terminate, demolition_field, command_tower, twisted_landscape, grisly_salvage,
    yavimaya_elder, bituminous_blast, bloodbraid_elf, eternal_witness, savage_lands,
    tainted_wood, burnished_hart, deathreap_ritual, arcane_signet, syr_konrad_the_grim,
    grapple_with_the_past, dakmor_salvage, accursed_marauder, brawn, faithless_looting,
    rampant_growth, anger, tranquil_thicket, stitchers_supplier, graveshifter,
    talisman_of_resilience, altar_of_the_goyf, syphon_mind, talisman_of_indulgence,
    sakura_tribe_elder, terramorphic_expanse, tainted_peak, riveteers_overlook, riveteers_charm,
    forgotten_cave, path_of_ancestry, lightning_greaves, myriad_landscape, sol_ring,
    talisman_of_impulse, evolving_wilds,
    // Basics: 4 swamp, 3 mountain, 5 forest
    swamp, swamp, swamp, swamp, mountain, mountain, mountain, forest, forest, forest, forest,
    forest,
];

pub const EZURI_COMMANDERS: &[CardFactory] = &[ezuri_claw_of_progress];

/// **Swell the Host**, the Commander 2015 deck (C15, 2015-11-13), exactly as
/// MTGJSON's `SwellTheHost_C15` prints it: 74 nonbasic cards + 14 Forests +
/// 11 Islands = 99. Simic +1/+1 counters and experience: Ezuri grows a small
/// board, myriad and Snakes spread it around the table.
pub const EZURI_MAIN: &[CardFactory] = &[
    experiment_one, elvish_visionary, sakura_tribe_elder, plaxmanta, coiling_oracle,
    caller_of_the_claw, loaming_shaman, noble_quarry, skullwinder, viridian_shaman,
    eternal_witness, ohran_viper, kaseto_orochi_archmage, lorescale_coatl, trygon_predator,
    cold_eyed_selkie, wistful_selkie, solemn_simulacrum, forgotten_ancient, patagia_viper,
    thelonite_hermit, ninja_of_the_deep_hours, chameleon_colossus, mystic_snake,
    stingerfling_spider, broodbirth_viper, illusory_ambusher, mulldrifter, arbor_colossus,
    great_oak_guardian, bane_of_progress, prime_speaker_zegana, caller_of_the_pack, rampant_growth,
    kodamas_reach, overrun, desert_twister, verdant_confluence, biomantic_mastery,
    ezuris_predation, rapid_hybridization, arachnogenesis, krosan_grip, snakeform, cobra_trap,
    mirror_match, synthetic_destiny, sol_ring, simic_signet, swiftfoot_boots, thought_vessel,
    simic_keyrune, sword_of_vengeance, bident_of_thassa, scytheclaw, orochi_hatchery,
    beastmaster_ascension, day_of_the_dragons, command_beacon, command_tower, evolving_wilds,
    high_market, llanowar_reborn, mosswort_bridge, novijen_heart_of_progress,
    oran_rief_the_vastwood, reliquary_tower, simic_growth_chamber, simic_guildgate,
    terramorphic_expanse, thornwood_falls, vivid_creek, vivid_grove, zoetic_cavern,
    // Basics: 14 forest, 11 island
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, island, island, island, island, island, island, island, island, island, island,
    island,
];

pub const GISELA_COMMANDERS: &[CardFactory] = &[gisela_the_broken_blade];

/// **Angels: They're Just Like Us but Cooler and with Wings**, the Secret Lair
/// Commander deck (SLD, 2023-08-14), exactly as MTGJSON's
/// `AngelsTheyReJustLikeUsButCoolerAndWithWings_SLD` prints it: 69 nonbasic
/// cards + 30 Plains = 99. Mono-white Angels and lifegain under Gisela, who
/// melds with Bruna (in the 99) into Brisela — a melded commander (CR 903.3).
pub const GISELA_MAIN: &[CardFactory] = &[
    bruna_the_fading_light, ajani_strength_of_the_pride, angel_of_destiny, angel_of_finality,
    angel_of_serenity, angel_of_the_ruins, angel_of_vitality, angelic_accord,
    angelic_field_marshal, arcane_signet, arch_of_orazca, archangel_of_thune, archangel_of_tithes,
    arden_angel, austere_command, bishop_of_wings, bonders_enclave, breathkeeper_seraph,
    cleansing_nova, commanders_plate, cosmos_elixir, court_of_grace, dawn_of_hope,
    dawnbreak_reclaimer, dismantling_wave, emeria_shepherd, emeria_the_sky_ruin, endless_atlas,
    entreat_the_angels, everflowing_chalice, giada_font_of_hope, griffin_aerie, heirloom_blade,
    invoke_the_divine, karoo_land, keeper_of_the_accord, kindred_boon, lightning_greaves,
    marble_diamond, mazemind_tome, mind_stone, myriad_landscape, nykthos_paragon,
    nykthos_shrine_to_nyx, oketras_monument, path_of_ancestry, path_to_exile, pearl_medallion,
    righteous_valkyrie, search_for_glory, sephara_skys_blade, seraph_sanctuary, serra_ascendant,
    shattered_angel, sol_ring, speaker_of_the_heavens, starnheim_aspirant, sunblast_angel,
    swiftfoot_boots, sword_of_the_animist, swords_to_plowshares, thalias_lancers,
    the_book_of_exalted_deeds, tome_of_legends, urzas_incubator, valkyrie_harbinger,
    vanquishers_banner, war_room, well_of_lost_dreams,
    // Basics: 30 plains
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains,
];

pub const DARETTI_COMMANDERS: &[CardFactory] = &[daretti_scrap_savant];

/// **Built From Scratch**, the Commander 2014 deck (C14, 2014-11-07), exactly
/// as MTGJSON's `BuiltFromScratch_C14` prints it: 70 nonbasic cards + 29
/// Mountains = 99. Mono-red artifacts under Daretti, another
/// planeswalker commander (CR 903.3a) — scrap in, recursion out.
pub const DARETTI_MAIN: &[CardFactory] = &[
    goblin_welder, epochrasite, myr_retriever, myr_sire, bottle_gnomes, cathodion, junk_diver,
    palladium_myr, pilgrims_eye, tuktuk_the_explorer, dualcaster_mage, feldon_of_the_third_path,
    solemn_simulacrum, flametongue_kavu, beetleback_chief, ingot_chewer, steel_hellkite,
    wurmcoil_engine, spitebellows, hoard_smelter_dragon, warmonger_hellkite, myr_battlesphere,
    pentavus, tyrants_familiar, bosh_iron_golem, bogardan_hellkite, faithless_looting, whipflare,
    scrap_mastery, incite_rebellion, blasphemous_act, impact_resonance, chaos_warp,
    volcanic_offering, word_of_seizing, magmaquake, starstorm, everflowing_chalice,
    panic_spellbomb, sol_ring, wayfarers_bauble, fire_diamond, ichor_wellspring,
    liquimetal_coating, mind_stone, mycosynth_wellspring, ruby_medallion, swiftfoot_boots,
    commanders_sphere, jalum_tome, pristine_talisman, unstable_obelisk, trading_post, caged_sun,
    dreamstone_hedron, loreseekers_stone, spine_of_ish_sah, darksteel_citadel, great_furnace,
    bitter_feud, arcane_lighthouse, buried_ruin, dormant_volcano, flamekin_village, forgotten_cave,
    ghost_quarter, phyrexias_core, reliquary_tower, smoldering_crater, temple_of_the_false_god,
    // Basics: 29 mountain
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain,
];

pub const OB_NIXILIS_COMMANDERS: &[CardFactory] = &[ob_nixilis_of_the_black_oath];

/// **Sworn to Darkness**, the Commander 2014 deck (C14, 2014-11-07), exactly
/// as MTGJSON's `SwornToDarkness_C14` prints it: 67 nonbasic cards + 32
/// basics = 99. Mono-black Demons and morbid under Ob Nixilis of the Black
/// Oath, a planeswalker commander (CR 903.3a).
pub const OB_NIXILIS_MAIN: &[CardFactory] = &[
    skirsdag_high_priest, nantuko_shade, vampire_hexmage, burnished_hart, flesh_carver,
    crypt_ghast, disciple_of_bolas, evernight_shade, abyssal_persecutor, nekrataal,
    magus_of_the_coffers, raving_dead, shriekmaw, bloodgift_demon, demon_of_wailing_agonies,
    drana_kalastria_bloodchief, ghoulcaller_gisa, gray_merchant_of_asphodel, morkrut_banshee,
    grave_titan, phyrexian_gargantua, pontiff_of_blight, reaper_from_the_abyss,
    butcher_of_malakir, overseer_of_the_damned, pestilence_demon, lilianas_reaver,
    xathrid_demon, sign_in_blood, read_the_bones, victimize, syphon_mind, dread_return,
    mutilate, infernal_offering, aether_snap, promise_of_power, necromantic_selection,
    profane_command, dregs_of_sorrow, black_suns_zenith, spoils_of_blood, tragic_slip,
    malicious_affliction, sudden_spoiling, tendrils_of_corruption, annihilate, skeletal_scrying,
    wake_the_dead, sol_ring, charcoal_diamond, jet_medallion, mind_stone, swiftfoot_boots,
    unstable_obelisk, worn_powerstone, lashwrithe, commanders_sphere, bad_moon,
    arcane_lighthouse, barren_moor, bojuka_bog, crypt_of_agadeem, everglades, ghost_quarter,
    myriad_landscape, polluted_mire,
    // Basics: 32 swamp
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, swamp, swamp,
];

pub const STREFAN_COMMANDERS: &[CardFactory] = &[strefan_maurer_progenitor];

/// **Vampiric Bloodline**, the Innistrad: Crimson Vow Commander deck (VOC,
/// 2021-11-19), exactly as MTGJSON's `VampiricBloodline_VOC` prints it:
/// 74 nonbasic cards + 14 Swamps + 11 Mountains = 99. Rakdos Vampires and
/// Blood tokens under Strefan; Kamber and Laurine ride along as a "Partner
/// with" pair in the 99.
pub const STREFAN_MAIN: &[CardFactory] = &[
    timothar_baron_of_bats, crossway_troublemakers, kamber_the_plunderer, shadowgrange_archfiend,
    laurine_the_diversion, markov_enforcer, midnight_arsonist, scion_of_opulence,
    anowon_the_ruin_sage, bloodlord_of_vaasgoth, bloodtracker, butcher_of_malakir,
    champion_of_dusk, cordial_vampire, dark_impostor, malakir_bloodwitch, necropolis_regent,
    nirkana_revenant, patron_of_the_vein, sanctum_seeker, stromkirk_condemned, anjes_ravager,
    bloodsworn_steward, crimson_honor_guard, falkenrath_gorger, stromkirk_occultist,
    vampiric_dragon, bloodtithe_harvester, blood_artist, bloodline_necromancer, falkenrath_noble,
    indulgent_aristocrat, vampire_nighthawk, rakish_heir, stromkirk_captain, olivias_wrath,
    predators_hour, imposing_grandeur, sinister_waltz, damnable_pact, avacyns_judgment,
    blasphemous_act, mob_rule, ancient_craving, feed_the_swarm, nights_whisper, vandalblast,
    urge_to_feed, rakdos_charm, glass_cast_heart, arcane_signet, charcoal_diamond,
    commanders_sphere, fire_diamond, rakdos_signet, sol_ring, swiftfoot_boots, unstable_obelisk,
    arterial_alchemy, underworld_connections, molten_echoes, stensia_masquerade, exotic_orchard,
    foreboding_ruins, shadowblood_ridge, smoldering_marsh, temple_of_malice, command_tower,
    myriad_landscape, path_of_ancestry, rakdos_carnarium, tainted_peak, temple_of_the_false_god,
    unclaimed_territory,
    // Basics: 14 swamp, 11 mountain
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain,
];

pub const MEREN_COMMANDERS: &[CardFactory] = &[meren_of_clan_nel_toth];

/// **Plunder the Graves**, the Commander 2015 deck (C15, 2015-11-13), exactly
/// as MTGJSON's `PlunderTheGraves_C15` prints it: 74 nonbasic cards + 13
/// Swamps + 12 Forests = 99. Golgari sacrifice and recursion: Meren's
/// experience counters return what died.
pub const MEREN_MAIN: &[CardFactory] = &[
    sakura_tribe_elder, satyr_wayfinder, viridian_emissary, wall_of_blossoms, viridian_zealot,
    korozda_guildmage, lotleth_troll, blood_bairn, phyrexian_rager, skullwinder, wood_elves,
    eternal_witness, corpse_augur, centaur_vinecrasher, bloodspore_thrinax,
    jarad_golgari_lich_lord, shriekmaw, indrik_stomphowler, kessig_cagebreakers,
    banshee_of_the_dread_choir, phyrexian_plaguelord, acidic_slime, mycoloth,
    mazirek_kraul_death_priest, vulturous_zombie, great_oak_guardian, champion_of_stray_souls,
    extractor_demon, thief_of_blood, pathbreaker_ibex, cloudthresher, butcher_of_malakir,
    eater_of_hope, scourge_of_nel_toth, caller_of_the_pack, terastodon, verdant_force, mulch,
    victimize, primal_growth, ambitions_cost, sever_the_bloodline, barter_in_blood,
    rise_from_the_grave, spider_spawning, overwhelming_stampede, dread_summons, altars_reap,
    tribute_to_the_wild, golgari_charm, grisly_salvage, putrefy, wretched_confluence, skullclamp,
    sol_ring, golgari_signet, lightning_greaves, thought_vessel, bonehoard, eldrazi_monument,
    diabolic_servitude, command_tower, evolving_wilds, golgari_guildgate, golgari_rot_farm,
    grim_backwoods, high_market, jungle_hollow, polluted_mire, slippery_karst, tainted_wood,
    terramorphic_expanse, vivid_grove, vivid_marsh,
    // Basics: 13 swamp, 12 forest
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
];

pub const MIZZIX_COMMANDERS: &[CardFactory] = &[mizzix_of_the_izmagnus];

/// **Seize Control**, the Commander 2015 deck (C15, 2015-11-13), exactly as
/// MTGJSON's `SeizeControl_C15` prints it: 72 nonbasic cards + 14 Islands + 13
/// Mountains = 99. Izzet spells under Mizzix, whose experience counters
/// discount every instant and sorcery.
pub const MIZZIX_MAIN: &[CardFactory] = &[
    goblin_electromancer, jaces_archivist, gigantoplasm, talrand_sky_summoner,
    psychosis_crawler, broodbirth_viper, illusory_ambusher, lone_revenant, warchief_giant,
    charmbreaker_devils, arjun_the_shifting_flame, etherium_horn_sorcerer, melek_izzet_paragon,
    dragon_mage, preordain, faithless_looting, vandalblast, mizzium_mortars, windfall,
    mystic_retrieval, stolen_goods, mizzixs_mastery, rite_of_replication, sleep, chain_reaction,
    call_the_skybreaker, blatant_thievery, epic_experiment, meteor_blast, blustersquall,
    brainstorm, echoing_truth, desperate_ravings, urzas_rage, counterflux, aetherize,
    fact_or_fiction, reins_of_power, steam_augury, mystic_confluence, word_of_seizing,
    act_of_aggression, prophetic_bolt, aethersnatch, mirror_match, fireminds_foresight, repeal,
    comet_storm, magmaquake, stroke_of_genius, dominate, blue_suns_zenith, sol_ring,
    izzet_signet, thought_vessel, worn_powerstone, seal_of_the_guildpact, awaken_the_sky_tyrant,
    rite_of_the_raging_storm, thought_reflection, command_tower, evolving_wilds,
    izzet_boilerworks, izzet_guildgate, reliquary_tower, rogues_passage, spinerock_knoll,
    swiftwater_cliffs, temple_of_the_false_god, terramorphic_expanse, vivid_crag, vivid_creek,
    // Basics: 14 island, 13 mountain
    island, island, island, island, island, island, island, island, island, island, island,
    island, island, island,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain,
];

pub const ADRIX_NEV_COMMANDERS: &[CardFactory] = &[adrix_and_nev_twincasters];

/// **Quantum Quandrix**, the Commander 2021 deck (C21, 2021-04-23), exactly as
/// MTGJSON's `QuantumQuandrix_C21` prints it: 78 nonbasic cards + 21
/// basics = 99. Simic Fractals and token doubling under Adrix and Nev,
/// Twincasters.
pub const ADRIX_NEV_MAIN: &[CardFactory] = &[
    garruk_primal_hunter, esix_fractal_bloom, curiosity_crafter, deekah_fractal_theorist,
    spawning_kraken, guardian_augmenter, ruxa_patient_professor, desolation_twin,
    champion_of_wits, crafty_cutpurse, reef_worm, arashi_the_sky_asunder, forgotten_ancient,
    hornet_nest, hornet_queen, hydra_broodmaster, incubation_druid, kazandu_tuskcaller,
    managorger_hydra, rampaging_baloths, terastodon, kaseto_orochi_archmage, master_biomancer,
    biomathematician, quandrix_cultivator, zimone_quandrix_prodigy, coiling_oracle,
    plaxcaster_frogling, trygon_predator, replication_technique, oversimplify,
    curse_of_the_swine, rite_of_replication, shamanic_revelation, spitting_image, golden_ratio,
    rampant_growth, incubation_incongruity, perplexing_test, theoretical_duplication,
    return_of_the_wildspeaker, biomass_mutation, eureka_moment, rapid_hybridization,
    beast_within, krosan_grip, fractal_harness, sequence_engine, geometric_nexus,
    idol_of_oblivion, arcane_signet, simic_signet, sol_ring, paradox_zone, primal_empathy,
    exotic_orchard, lumbering_falls, mosswort_bridge, oran_rief_the_vastwood, temple_of_mystery,
    yavimaya_coast, quandrix_campus, study_hall, blighted_woodland, command_tower,
    llanowar_reborn, lonely_sandbar, myriad_landscape, novijen_heart_of_progress, opal_palace,
    simic_growth_chamber, temple_of_the_false_god, tranquil_thicket, commanders_insight,
    ezuris_predation, kodamas_reach, nissas_expedition, rogues_passage,
    // Basics: 11 forest, 10 island
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    island, island, island, island, island, island, island, island, island, island,
];

pub const DAXOS_COMMANDERS: &[CardFactory] = &[daxos_the_returned];

/// **Call the Spirits**, the Commander 2015 deck (C15, 2015-11-13), exactly as
/// MTGJSON's `CallTheSpirits_C15` prints it: 75 nonbasic cards + 11 Plains +
/// 13 Swamps = 99. Orzhov enchantments under Daxos the Returned, whose
/// experience counters size his Spirit tokens.
pub const DAXOS_MAIN: &[CardFactory] = &[
    oreskos_explorer, karlov_of_the_ghost_council, underworld_coinsmith, burnished_hart,
    bastion_protector, dawnglare_invoker, ghostblade_eidolon, kor_sanctifiers, monk_idealist,
    mesa_enchantress, nighthowler, corpse_augur, fate_unraveler, ajanis_chosen, doomwake_giant,
    dreadbringer_lampads, celestial_ancient, celestial_archon, herald_of_the_host,
    banshee_of_the_dread_choir, thief_of_blood, treasury_thrull, sandstone_oracle,
    silent_sentinel, teysa_envoy_of_ghosts, ancient_craving, gild, dawn_to_dusk,
    righteous_confluence, open_the_vaults, deadly_tempest, death_grasp, sol_ring,
    wayfarers_bauble, lightning_greaves, orzhov_signet, thought_vessel, crystal_chimes,
    orzhov_cluestone, phyrexian_reclamation, seal_of_cleansing, grave_peril, banishing_light,
    cage_of_hands, karmic_justice, vow_of_duty, fallen_ideal, seal_of_doom, vow_of_malice,
    aura_of_silence, grasp_of_fate, shielded_by_faith, phyrexian_arena, underworld_connections,
    daxoss_torment, marshals_anthem, dictate_of_heliod, sigil_of_the_empty_throne, black_market,
    necromancers_covenant, barren_moor, command_tower, evolving_wilds, ghost_quarter,
    new_benalia, orzhov_basilica, orzhov_guildgate, rogues_passage, scoured_barrens,
    secluded_steppe, tainted_field, temple_of_the_false_god, terramorphic_expanse, vivid_marsh,
    vivid_meadow,
    // Basics: 11 plains, 13 swamp
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
];

pub const NEYALI_COMMANDERS: &[CardFactory] = &[neyali_suns_vanguard];

/// **Rebellion Rising**, the Phyrexia: All Will Be One Commander deck (ONC,
/// 2023-02-10), exactly as MTGJSON's `RebellionRising_ONC` prints it: 77
/// nonbasic cards + 11 Plains + 11 Mountains = 99. Boros tokens and Equipment
/// under Neyali, whose attacking tokens strike twice.
pub const NEYALI_MAIN: &[CardFactory] = &[
    elspeth_tirel, adriana_captain_of_the_guard, dragonmaster_outcast, emeria_angel,
    goldnight_commander, harmonious_archon, jor_kadeen_the_prevailer, legion_warboss,
    loyal_apprentice, mentor_of_the_meek, myr_battlesphere, otharri_suns_glory, phantom_general,
    prava_of_the_steel_legion, siege_gang_commander, silverwing_squadron, solemn_simulacrum,
    boros_charm, call_the_coppercoats, clever_concealment, flawless_maneuver, generous_gift,
    midnight_haunting, path_to_exile, white_suns_zenith, battle_screech, chain_reaction,
    collective_effort, cut_a_deal, finale_of_glory, goldwardens_gambit, hate_mirage,
    heroic_reinforcements, hordeling_outburst, hour_of_reckoning, increasing_devotion,
    martial_coup, rip_apart, assemble_the_legion, court_of_grace, felidar_retreat,
    intangible_virtue, roar_of_resistance, arcane_signet, boros_signet, commanders_sphere,
    fellwar_stone, glimmer_lens, hexplate_wallbreaker, idol_of_oblivion, kembas_banner,
    loxodon_warhammer, mace_of_the_valiant, mask_of_memory, maul_of_the_skyclaves, mind_stone,
    sol_ring, soul_guide_lantern, staff_of_the_storyteller, talisman_of_conviction,
    vulshok_factory, boros_garrison, buried_ruin, castle_ardenvale, castle_embereth,
    command_tower, exotic_orchard, forgotten_cave, furycalm_snarl, kher_keep, myriad_landscape,
    path_of_ancestry, secluded_steppe, slayers_stronghold, temple_of_triumph,
    temple_of_the_false_god, windbrisk_heights,
    // Basics: 11 plains, 11 mountain
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain,
];

pub const TEFERI_COMMANDERS: &[CardFactory] = &[teferi_temporal_archmage];

/// **Peer Through Time**, the Commander 2014 mono-blue deck (C14,
/// 2014-11-07), exactly as MTGJSON's `PeerThroughTime_C14` prints it:
/// 68 nonbasic cards + 31 Islands = 99. Teferi is the pod's fifth
/// planeswalker commander (CR 903.3a); Stormsurge Kraken is its lieutenant.
pub const TEFERI_MAIN: &[CardFactory] = &[
    azure_mage, fathom_seer, fog_bank, willbender, dulcet_sirens, riptide_survivor, sea_gate_oracle,
    shaper_parasite, reef_worm, mulldrifter, ixidron, stitcher_geralf, stormsurge_kraken,
    steel_hellkite, brine_elemental, frost_titan, sphinx_of_jwar_isle, sphinx_of_magosi,
    phyrexian_ingester, sphinx_of_uthuun, hoverguard_sweepers, lorthos_the_tidemaker,
    artisan_of_kozilek, breaching_leviathan, deep_sea_kraken, call_to_mind, compulsive_research,
    concentrate, rite_of_replication, rush_of_knowledge, aether_gale, distorting_wake, pongify,
    cyclonic_rift, into_the_roil, turn_to_frog, exclude, cackling_counterpart, domineering_will,
    dismiss, intellectual_offering, stroke_of_genius, everflowing_chalice, sol_ring, mind_stone,
    sapphire_medallion, sky_diamond, swiftfoot_boots, crown_of_doom, unstable_obelisk,
    worn_powerstone, assault_suit, thran_dynamo, dreamstone_hedron, tormods_crypt,
    commanders_sphere, nevinyrrals_disk, ur_golems_eye, infinite_reflection, well_of_ideas,
    fools_demise, coral_atoll, ghost_quarter, lonely_sandbar, myriad_landscape, remote_isle,
    tectonic_edge, zoetic_cavern,
    // Basics: 31 island
    island, island, island, island, island, island, island, island, island, island, island, island,
    island, island, island, island, island, island, island, island, island, island, island, island,
    island, island, island, island, island, island, island,
];

pub const OSGIR_COMMANDERS: &[CardFactory] = &[osgir_the_reconstructor];

/// **Lorehold Legacies**, the Commander 2021 deck (C21, 2021-04-23), exactly
/// as MTGJSON's `LoreholdLegacies_C21` prints it: 79 nonbasic cards +
/// 20 basics = 99. Boros artifact recursion under Osgir, the Reconstructor.
pub const OSGIR_MAIN: &[CardFactory] = &[
    daretti_scrap_savant, combustible_gearhulk, alibou_ancient_witness, angel_of_the_ruins,
    bronze_guardian, digsite_engineer, losheel_clockwork_scholar, audacious_reshapers,
    laelia_the_blade_reforged, ruin_grinder, triplicate_titan, sun_titan,
    feldon_of_the_third_path, hellkite_igniter, hellkite_tyrant, hoard_smelter_dragon,
    pia_nalaar, jor_kadeen_the_prevailer, bosh_iron_golem, duplicant, myr_battlesphere,
    scrap_trawler, solemn_simulacrum, steel_hellkite, steel_overseer, sanctum_gargoyle,
    quicksmith_genius, thopter_engineer, burnished_hart, meteor_golem, pilgrims_eye,
    excavation_technique, wake_the_past, cleansing_nova, rout, chain_reaction,
    secret_rendezvous, reconstruct_history, rip_apart, faithless_looting, dispatch,
    return_to_dust, boros_charm, archaeomancers_map, battlemages_bracers, cursed_mirror,
    key_to_the_city, sculpting_steel, thousand_year_elixir, dispellers_capsule, arcane_signet,
    boros_locket, commanders_sphere, hedron_archive, ichor_wellspring, mind_stone,
    mycosynth_wellspring, sol_ring, unstable_obelisk, monologue_tax, darksteel_mutation,
    battlefield_forge, exotic_orchard, slayers_stronghold, temple_of_triumph, lorehold_campus,
    study_hall, ancient_den, boros_garrison, command_tower, darksteel_citadel, forgotten_cave,
    great_furnace, myriad_landscape, phyrexias_core, rogues_passage, secluded_steppe,
    sunhome_fortress_of_the_legion, temple_of_the_false_god,
    // Basics: 12 mountain, 8 plains
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, plains, plains, plains, plains, plains, plains, plains,
    plains,
];

pub const GHAVE_COMMANDERS: &[CardFactory] = &[ghave_guru_of_spores];

/// **Counterpunch**, the Commander (2011) deck (CMD, 2011-06-17), exactly as
/// MTGJSON's `Counterpunch_CMD` prints it: 73 nonbasic cards + 8 Swamps +
/// 10 Forests + 8 Plains = 99. Abzan Saprolings and +1/+1 counters under
/// Ghave, Guru of Spores.
pub const GHAVE_MAIN: &[CardFactory] = &[
    aquastrand_spider, deadly_recluse, sakura_tribe_elder, scavenging_ooze, monk_realist,
    selesnya_evangel, golgari_guildmage, selesnya_guildmage, nantuko_husk, fertilid,
    spawnwrithe, vampire_nighthawk, spike_feeder, yavimaya_elder, squallmonger, penumbra_spider,
    sigil_captain, shriekmaw, dark_hatchling, teneb_the_harvester, hornet_queen,
    vish_kal_blood_arbiter, symbiotic_wurm, celestial_force, karador_ghost_chieftain,
    chorus_of_the_conclave, alliance_of_arms, cultivate, harmonize, syphon_flesh,
    bestial_menace, hex, hour_of_reckoning, death_mutation, storm_herd, doom_blade,
    tribute_to_the_wild, footbottom_feast, afterlife, mortify, nemesis_trap, cobra_trap,
    skullclamp, sol_ring, golgari_signet, lightning_greaves, orzhov_signet, selesnya_signet,
    darksteel_ingot, acorn_catapult, soul_snare, fists_of_ironwood, necrogenesis, vow_of_malice,
    awakening_zone, vow_of_wildness, oblivion_ring, vow_of_duty, attrition, aura_shards,
    barren_moor, command_tower, evolving_wilds, golgari_rot_farm, orzhov_basilica,
    rupture_spire, secluded_steppe, selesnya_sanctuary, temple_of_the_false_god,
    tranquil_thicket, vivid_grove, vivid_marsh, vivid_meadow,
    // Basics: 8 swamp, 10 forest, 8 plains
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    plains, plains, plains, plains, plains, plains, plains, plains,
];

pub const TEGWYLL_COMMANDERS: &[CardFactory] = &[tegwyll_duke_of_splendor];

/// **Fae Dominion**, the Wilds of Eldraine Commander deck (WOC, 2023-09-08),
/// exactly as MTGJSON's `FaeDominion_WOC` prints it: 74 nonbasic cards +
/// 25 basics = 99. Dimir Faeries under Tegwyll, Duke of Splendor.
pub const TEGWYLL_MAIN: &[CardFactory] = &[
    alela_cunning_conqueror, archmage_of_echoes, malleable_impostor, misleading_signpost,
    shadow_puppeteers, blightwing_bandit, faerie_bladecrafter, nettling_nuisance,
    tegwylls_scouring, brazen_borrower, dig_through_time, faerie_formation,
    glen_elendra_archmage, hullbreaker_horror, illusionists_gambit, midnight_clock,
    perplexing_test, reflections_of_littjara, scion_of_oona, sower_of_temptation,
    theoretical_duplication, kindred_dominance, nightmare_unmaking, puppeteer_clique,
    rankle_master_of_pranks, thrilling_encore, glen_elendra_liege, nymris_oonas_trickster,
    oona_queen_of_the_fae, choked_estuary, darkwater_catacombs, exotic_orchard, secluded_glen,
    sunken_hollow, temple_of_deceit, mocking_sprite, picklock_prankster, spell_stutter,
    obyra_dreaming_duelist, spellscorn_coven, arcane_denial, cloud_of_faeries, consider,
    distant_melody, fact_or_fiction, faerie_seer, frantic_search, hypnotic_sprite, keep_watch,
    nightveil_sprite, opt, quickling, reality_shift, reconnaissance_mission, repulse,
    run_away_together, snap, reckless_spite, halo_forager, arcane_signet, dimir_signet,
    fellwar_stone, mind_stone, sol_ring, talisman_of_dominance, wayfarers_bauble, bojuka_bog,
    command_tower, dimir_aqueduct, faerie_conclave, myriad_landscape, path_of_ancestry,
    tainted_isle, temple_of_the_false_god,
    // Basics: 13 island, 12 swamp
    island, island, island, island, island, island, island, island, island, island, island,
    island, island, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp,
];

pub const KALEMNE_COMMANDERS: &[CardFactory] = &[kalemne_disciple_of_iroas];

/// **Wade into Battle**, the Commander 2015 Boros deck (C15, 2015-11-13),
/// exactly as MTGJSON's `WadeIntoBattle_C15` prints it: 74 nonbasic
/// cards + 14 Mountains + 11 Plains = 99. Giants and experience counters
/// under Kalemne.
pub const KALEMNE_MAIN: &[CardFactory] = &[
    oreskos_explorer, magus_of_the_wheel, stinkdrinker_daredevil, taurean_mauler, dawnglare_invoker,
    desolation_giant, fumiko_the_lowblood, hunted_dragon, stoneshock_giant, thundercloud_shaman,
    warchief_giant, herald_of_the_host, kalemnes_captain, anya_merciless_angel, sunrise_sovereign,
    hammerfist_giant, inferno_titan, dawnbreak_reclaimer, sun_titan, hostility,
    jareth_leonine_titan, victorys_herald, sandstone_oracle, hamletback_goliath,
    arbiter_of_knollridge, borderland_behemoth, dream_pillager, magma_giant, angel_of_serenity,
    gisela_blade_of_goldnight, breath_of_darigaaz, fiery_confluence, disaster_radius, earthquake,
    meteor_blast, fall_of_the_hammer, orims_thunder, sol_ring, blade_of_selves, boros_signet,
    coldsteel_heart, fellwar_stone, lightning_greaves, mind_stone, thought_vessel, basalt_monolith,
    boros_cluestone, darksteel_ingot, loxodon_warhammer, urzas_incubator, worn_powerstone,
    seers_sundial, dreamstone_hedron, staff_of_nin, curse_of_the_nightly_hunt, banishing_light,
    faiths_fetters, rite_of_the_raging_storm, warstorm_surge, ancient_amphitheater,
    blasted_landscape, boros_garrison, boros_guildgate, command_tower, drifting_meadow,
    evolving_wilds, forgotten_cave, secluded_steppe, smoldering_crater, terramorphic_expanse,
    vivid_crag, vivid_meadow, wind_scarred_crag, crib_swap,
    // Basics: 14 mountain, 11 plains
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains,
];

pub const WYLETH_COMMANDERS: &[CardFactory] = &[wyleth_soul_of_steel];

/// **Arm for Battle**, the Commander Legends deck (CMR, 2020-11-20), exactly
/// as MTGJSON's `ArmForBattle_CMR` prints it: 76 nonbasic cards + 9
/// Mountains + 14 Plains = 99. Boros Auras and Equipment under Wyleth, Soul of
/// Steel.
pub const WYLETH_MAIN: &[CardFactory] = &[
    oreskos_explorer, relic_seeker, sram_senior_edificer, brass_squire,
    danitha_capashen, dualcaster_mage, flickerwisp, ironclad_slayer, kor_cartographer,
    odric_lunarch_marshal, tiana_ships_caretaker, condemn, expedite, swords_to_plowshares,
    abrade, boros_charm, comet_storm, dawn_charm, deflecting_palm, disenchant, fists_of_flame,
    temur_battle_rage, valorous_stance, generous_gift, unbreakable_formation, volcanic_fallout,
    wear_tear, white_suns_zenith, master_warcraft, return_to_dust, wild_ricochet, word_of_seizing,
    response_resurgence, jayas_immolating_inferno, martial_coup, relentless_assault,
    winds_of_rath, bonesplitter, explorers_scope, sol_ring, blackblade_reforged,
    blazing_sunsteel, boros_signet, heros_blade, mask_of_avacyn, ring_of_thune, ring_of_valkas,
    swiftfoot_boots, fireshrieker, haunted_cloak, loxodon_warhammer, sunforger,
    sword_of_vengeance, sigardas_aid, spirit_mantle, timely_ward, unquestioned_authority,
    faith_unbroken, on_serras_wings, boros_garrison, boros_guildgate, command_tower,
    encroaching_wastes, evolving_wilds, forgotten_cave, memorial_to_war, myriad_landscape,
    rogues_passage, rupture_spire, secluded_steppe, slayers_stronghold, stone_quarry,
    sunhome_fortress_of_the_legion, terramorphic_expanse, transguild_promenade,
    wind_scarred_crag,
    // Basics: 9 mountain, 14 plains
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains,
];

pub const BREYA_COMMANDERS: &[CardFactory] = &[breya_etherium_shaper];

/// **Invent Superiority**, the Commander 2016 deck (C16, 2016-11-11), exactly
/// as MTGJSON's `InventSuperiority_C16` prints it: 81 nonbasic cards +
/// 18 basics = 99. Four-colour artifacts under Breya,
/// Etherium Shaper — the pod's first four-colour identity.
pub const BREYA_MAIN: &[CardFactory] = &[
    daretti_scrap_savant, myr_retriever, chief_engineer, etherium_sculptor, vedalken_engineer,
    slobad_goblin_tinkerer, baleful_strix, akiri_line_slinger, armory_automaton, shimmer_myr,
    master_of_etherium, trinket_mage, magus_of_the_will, hanna_ships_navigator,
    silas_renn_seeker_adept, sydri_galvanic_genius, etched_oracle, solemn_simulacrum,
    sanctum_gargoyle, faerie_artisans, bruse_tarl_boorish_herder, ethersworn_adjudicator,
    sphinx_summoner, jor_kadeen_the_prevailer, soul_of_new_phyrexia, godo_bandit_warlord,
    hellkite_tyrant, sharuum_the_hegemon, myr_battlesphere, hellkite_igniter, filigree_angel,
    whipflare, parting_thoughts, trash_for_treasure, beacon_of_unrest, migratory_route,
    open_the_vaults, phyrexian_rebirth, grave_upheaval, coastal_breach, grip_of_phyresis,
    ancient_excavation, read_the_runes, trial_error, everflowing_chalice, skullclamp, sol_ring,
    dispellers_capsule, executioners_capsule, cranial_plating, fellwar_stone, ichor_wellspring,
    mycosynth_wellspring, swiftfoot_boots, thopter_foundry, commanders_sphere,
    loxodon_warhammer, bonehoard, nevinyrrals_disk, trading_post, blinkmoth_urn,
    darksteel_citadel, seat_of_the_synod, curse_of_vengeance, arcane_sanctum, ash_barrens,
    azorius_chancery, boros_garrison, buried_ruin, command_tower, crumbling_necropolis,
    dimir_aqueduct, evolving_wilds, exotic_orchard, mystic_monastery, nomad_outpost,
    rakdos_carnarium, rupture_spire, temple_of_the_false_god, terramorphic_expanse,
    transguild_promenade,
    // Basics: 5 plains, 5 island, 4 swamp, 4 mountain
    plains, plains, plains, plains, plains, island, island, island, island, island, swamp,
    swamp, swamp, swamp, mountain, mountain, mountain, mountain,
];

pub const KARDUR_COMMANDERS: &[CardFactory] = &[kardur_doomscourge];

/// **Chaos Incarnate**, the Secret Lair Commander deck (SCD, 2022-12-02),
/// exactly as MTGJSON's `ChaosIncarnate_SCD` prints it: 70 nonbasic cards +
/// 15 Swamps + 14 Mountains = 99. Rakdos goad and punishment under
/// Kardur, Doomscourge.
pub const KARDUR_MAIN: &[CardFactory] = &[
    archfiend_of_depravity, bloodgift_demon, deadly_tempest, dredge_the_mire, ob_nixilis_reignited,
    profane_command, rakshasa_debaser, reign_of_the_pit, sangromancer, scythe_specter,
    sepulchral_primordial, soul_shatter, titan_hunter, blasphemous_act, brash_taunter, chaos_warp,
    combustible_gearhulk, dictate_of_the_twin_gods, fiery_confluence, geode_rager,
    kazuul_tyrant_of_the_cliffs, magmatic_force, sunbirds_invocation, tectonic_giant, wild_ricochet,
    wildfire_devils, kaervek_the_merciless, spiteful_visions, stormfist_crusader,
    theater_of_horrors, coveted_jewel, solemn_simulacrum, foreboding_ruins, smoldering_marsh,
    stensia_bloodhall, temple_of_malice, ambitions_cost, feed_the_swarm, indulgent_tormentor,
    read_the_bones, sign_in_blood, syphon_mind, vampire_nighthawk, abrade, explosion_of_riches,
    guttersnipe, hate_mirage, mana_geyser, thermo_alchemist, breath_of_malfegor, rakdos_charm,
    terminate, unlicensed_disintegration, arcane_signet, burnished_hart, commanders_sphere,
    lightning_greaves, nihil_spellbomb, rakdos_signet, sol_ring, talisman_of_indulgence,
    wayfarers_bauble, worn_powerstone, akoum_refuge, bloodfell_caves, cinder_barrens, command_tower,
    molten_slagheap, myriad_landscape, urborg_volcano,
    // Basics: 15 swamp, 14 mountain
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain,
];

pub const TEMMET_COMMANDERS: &[CardFactory] = &[temmet_naktamuns_will];

/// **Eternal Might**, the Aetherdrift Commander deck (DRC, 2025-02-14),
/// exactly as MTGJSON's `EternalMight_DRC` prints it: 85 nonbasic cards + 5
/// Plains + 4 Islands + 5 Swamps = 99. Esper Zombies and embalm under Temmet,
/// Naktamun's Will, with Hashaton, Scarab's Fist in the 99.
pub const TEMMET_MAIN: &[CardFactory] = &[
    hashaton_scarabs_fist, on_wings_of_gold, priest_of_the_crossing, renewed_solidarity,
    wizened_mentor, prophet_of_the_scarab, rhet_tomb_mystic, lost_monarch_of_ifnir,
    accursed_duneyard, commence_the_endgame, cryptbreaker, grave_titan, gravecrawler,
    midnight_reaper, murderous_rider, zombie_master, angel_of_sanctions, dusk_dawn,
    god_eternal_oketra, timeless_dragon, champion_of_wits, forgotten_creation,
    pull_from_tomorrow, vizier_of_many_faces, archfiend_of_ifnir, cemetery_reaper,
    crowded_crypt, damn, dread_summons, dreadhorde_invasion, liliana_deaths_majesty, never_return,
    plague_belcher, rot_hulk, the_scarab_god, god_pharaohs_gift, maskwood_nexus, adarkar_wastes,
    caves_of_koilos, drowned_catacomb, exotic_orchard, fetid_pools, glacial_fortress,
    irrigated_farmland, isolated_chapel, prairie_stream, sunken_hollow, temple_of_deceit,
    temple_of_silence, underground_river, unholy_grotto, swords_to_plowshares, corpse_augur,
    corpse_knight, arcane_signet, sol_ring, command_tower, path_of_ancestry, binding_mummy,
    cast_out, eternal_skylord, fleshbag_marauder, gempalm_polluter, lord_of_the_accursed,
    twisted_abomination, undead_augur, despark, gleaming_overseer, lazotep_chancellor,
    wayward_servant, bontus_monument, commanders_sphere, dimir_signet, gate_to_the_afterlife,
    orzhov_signet, talisman_of_dominance, talisman_of_hierarchy, arcane_sanctum, ash_barrens,
    desert_of_the_glorified, desert_of_the_mindful, desert_of_the_true, evolving_wilds,
    orzhov_basilica, terramorphic_expanse,
    // Basics: 5 plains, 4 island, 5 swamp
    plains, plains, plains, plains, plains, island, island, island, island,
    swamp, swamp, swamp, swamp, swamp,
];
