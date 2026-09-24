//! The Commander pod's target decks.
//!
//! Ten hand-picked commanders plus eighty-five official preconstructed lists, each a
//! 99 of cards this engine implements: legal under CR 903 (100 cards
//! including the commander, singleton outside basics, every card inside the
//! commander's CR 903.4 color identity) and Commander-legal per Scryfall's
//! ban list, both of which `pod::tests` asserts rather than trusts.
//!
//! The ten were built by hand because the offline Scryfall cache cannot say
//! how a precon splits into decks; MTGJSON's deck files can, and the eleventh
//! through ninety-fifth (Sultai Arisen, Mind Flayarrrs, Blood Rites, Heads I
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
//! Superiority and Entropic Uprising, Secret Lair's Chaos Incarnate,
//! Aetherdrift Commander's Eternal Might, Commander 2017's Feline Ferocity,
//! Commander (2011)'s Heavenly Inferno and Political Puppets, Zendikar Rising
//! Commander's Land's Wrath, Commander 2019's Primal Genesis and Commander 2016's
//! Stalwart Unity, Commander (2011)'s Devour for Power, Duskmourn Commander's
//! Endless Punishment, the Starter Commander Decks' Token Triumph, Commander
//! 2016's Breed Lethality, Commander (2011)'s Mirror Mastery, Secret Lair's
//! Raining Cats and Dogs, Commander 2016's Open Hostility, the Starter Commander
//! Decks' First Flight, Commander 2017's Arcane Wizardry, March of the
//! Machine Commander's Growing Threat, Commander Masters' Enduring
//! Enchantments, The Brothers' War Commander's Urza's Iron Alliance and
//! Kaldheim Commander's Phantom Premonition, Secret Lair's Hatsune Miku,
//! Commander Legends: Battle for Baldur's Gate's Exit from Exile, Innistrad: Midnight
//! Hunt Commander's Undead Unleashed, March of the Machine Commander's
//! Call for Backup, Bloomburrow Commander's Squirreled Away, Commander
//! 2018's Exquisite Invention, Duskmourn Commander's Death Toll, Commander
//! 2021's Witherbloom Witchcraft, Edge of Eternities Commander's World
//! Shaper, Commander 2018's Nature's Vengeance, Commander 2013's Evasive
//! Maneuvers, Bloomburrow Commander's Peace Offering, The Brothers' War
//! Commander's Mishra's Burnished Banner, March of the Machine Commander's
//! Tinker Time, Dominaria United Commander's Legends' Legacy, Secrets of
//! Strixhaven Commander's Witherbloom Pestilence, the Starter Commander Decks'
//! Draconic Destruction, Commander 2013's Nature of the Beast and Mind
//! Seize, Commander 2021's Silverquill Statement, Commander Masters'
//! Planeswalker Party, Secret Lair's 20 Ways to Win, Murders at Karlov Manor
//! Commander's Blame Game, Bloomburrow Commander's Family Matters and
//! Foundations Commander's Reign of Dragons) are taken from one card for
//! card. What all of them keep
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

pub const YIDRIS_COMMANDERS: &[CardFactory] = &[yidris_maelstrom_wielder];

/// **Entropic Uprising**, the Commander 2016 deck (C16, 2016-11-11), exactly
/// as MTGJSON's `EntropicUprising_C16` prints it: 79 nonbasic cards +
/// 20 basics = 99. Four-colour (UBRG) cascade and chaos under
/// Yidris, Maelstrom Wielder.
pub const YIDRIS_MAIN: &[CardFactory] = &[
    satyr_wayfinder, wall_of_blossoms, coiling_oracle, thrasios_triton_hero, goblin_spymaster,
    spellheart_chimera, vial_smasher_the_fierce, academy_elite, gamekeeper, sangromancer,
    bloodbraid_elf, horizon_chimera, kydele_chosen_of_kruphix, glint_eye_nephilim,
    aeon_chronicler, guiltfeeder, consuming_aberration, nath_of_the_gilt_leaf,
    runehorn_hellkite, etherium_horn_sorcerer, dragon_mage, blood_tyrant, wheel_of_fate,
    windfall, parting_thoughts, far_wanderings, past_in_flames, whispering_madness, decimate,
    devastation_tide, reforge_the_soul, worm_harvest, spelltwine, whims_of_the_fates,
    grave_upheaval, cruel_entertainment, ghastly_conscription, volcanic_vision, treasure_cruise,
    treacherous_terrain, army_of_the_damned, in_garruks_wake, chain_of_vapor, rakdos_charm,
    chaos_warp, ancient_excavation, evacuation, bituminous_blast, curtains_call, sol_ring,
    fellwar_stone, rakdos_signet, simic_signet, chromatic_lantern, commanders_sphere,
    whispersilk_cloak, boompile, burgeoning, waste_not, frenzied_fugue, ash_barrens,
    command_tower, crumbling_necropolis, dismal_backwater, evolving_wilds, exotic_orchard,
    frontier_bivouac, jungle_hollow, opulent_palace, rakdos_carnarium, reliquary_tower,
    rugged_highlands, rupture_spire, savage_lands, shadowblood_ridge, simic_growth_chamber,
    swiftwater_cliffs, terramorphic_expanse, thornwood_falls,
    // Basics: 5 island, 5 swamp, 5 mountain, 5 forest
    island, island, island, island, island, swamp, swamp, swamp, swamp, swamp, mountain,
    mountain, mountain, mountain, mountain, forest, forest, forest, forest, forest,
];

pub const ARAHBO_COMMANDERS: &[CardFactory] = &[arahbo_roar_of_the_world];

/// **Feline Ferocity**, the Commander 2017 deck (C17, 2017-08-25), exactly as
/// MTGJSON's `FelineFerocity_C17` prints it: 86 nonbasic cards + 7 Plains + 6
/// Forests = 99. Selesnya Cats and Equipment under Arahbo, Roar of the World.
pub const ARAHBO_MAIN: &[CardFactory] = &[
    nazahn_revered_bladesmith, mirri_weatherlight_duelist, alms_collector,
    balan_wandering_knight, stalking_leonin, hungry_lynx, qasali_slingers, jazal_goldmane,
    jareth_leonine_titan, kemba_kha_regent, leonin_arbiter, leonin_shikari, raksha_golden_cub,
    sehts_tiger, spirit_of_the_hearth, jedit_ojanen_of_efrava, fleecemane_lion, phantom_nishoba,
    leonin_relic_warder, oreskos_explorer, sunspear_shikari, taj_nar_swordsmith,
    temur_sabertooth, qasali_pridemage, traverse_the_outlands, divine_reckoning, rout,
    hunters_prowess, souls_majesty, cultivate, harmonize, nissas_pilgrimage, kindred_summons,
    white_suns_zenith, condemn, wing_shards, crushing_vines, relic_crush,
    bloodforged_battle_axe, hammer_of_nazahn, argentum_armor, grappling_hook, quietus_spike,
    staff_of_nin, sword_of_the_animist, sword_of_vengeance, heirloom_blade, heralds_horn,
    behemoth_sledge, dreamstone_hedron, hedron_archive, heros_blade, lightning_greaves,
    loxodon_warhammer, skullclamp, sol_ring, swiftfoot_boots, miraris_wake, abundance,
    zendikar_resurgent, curse_of_vitality, curse_of_bounty, mosswort_bridge, stirring_wildwood,
    path_of_ancestry, blighted_woodland, blossoming_sands, command_tower, elfhame_palace,
    evolving_wilds, grasslands, graypelt_refuge, krosan_verge, myriad_landscape, opal_palace,
    rogues_passage, saltcrusted_steppe, secluded_steppe, selesnya_guildgate, selesnya_sanctuary,
    temple_of_the_false_god, terramorphic_expanse, tranquil_expanse, tranquil_thicket,
    vivid_meadow, vivid_grove,
    // Basics: 7 plains, 6 forest
    plains, plains, plains, plains, plains, plains, plains,
    forest, forest, forest, forest, forest, forest,
];

pub const KAALIA_COMMANDERS: &[CardFactory] = &[kaalia_of_the_vast];

/// **Heavenly Inferno**, the Commander (2011) Mardu deck (CMD, 2011-06-17),
/// exactly as MTGJSON's `HeavenlyInferno_CMD` prints it: 75 nonbasic cards +
/// 8 Plains + 8 Swamps + 8 Mountains = 99. Angels, Demons and Dragons cheated in by Kaalia.
pub const KAALIA_MAIN: &[CardFactory] = &[
    mother_of_runes, orzhov_guildmage, boros_guildmage, gwyllion_hedge_mage, duergar_hedge_mage,
    lightkeeper_of_emeria, razorjaw_oni, anger, voice_of_all, dragon_whelp, furnace_whelp,
    serra_angel, shattered_angel, fallen_angel, basandra_battle_seraph, oni_of_wild_places,
    mana_charged_dragon, oros_the_avenger, malfegor, angelic_arbiter, archangel_of_strife,
    tariel_reckoner_of_souls, angel_of_despair, bladewing_the_risen, avatar_of_slaughter,
    akroma_angel_of_fury, reiver_demon, dread_cacodemon, syphon_mind, diabolic_tutor,
    evincars_justice, syphon_flesh, akromas_vengeance, death_by_dragons, earthquake, path_to_exile,
    bathe_in_light, terminate, orims_thunder, mortify, congregate, return_to_dust, sulfurous_blast,
    wrecking_ball, master_warcraft, cleansing_beam, comet_storm, sol_ring, armillary_sphere,
    boros_signet, lightning_greaves, orzhov_signet, rakdos_signet, darksteel_ingot, soul_snare,
    vow_of_duty, vow_of_malice, vow_of_lightning, stranglehold, pyrohemia, righteous_cause,
    akoum_refuge, barren_moor, bojuka_bog, boros_garrison, command_tower, evolving_wilds,
    forgotten_cave, molten_slagheap, orzhov_basilica, rakdos_carnarium, rupture_spire,
    secluded_steppe, vivid_meadow, zoetic_cavern,
    // Basics: 8 plains, 8 swamp, 8 mountain
    plains, plains, plains, plains, plains, plains, plains, plains, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain,
];

pub const OBUUN_COMMANDERS: &[CardFactory] = &[obuun_mul_daya_ancestor];

/// **Land's Wrath**, the Zendikar Rising Commander deck (ZNC, 2020-09-25),
/// exactly as MTGJSON's `LandSWrath_ZNC` prints it: 78 nonbasic cards + 7
/// Plains + 4 Mountains + 10 Forests = 99. Naya landfall under Obuun, Mul Daya
/// Ancestor.
pub const OBUUN_MAIN: &[CardFactory] = &[
    admonition_angel, trove_warden, geode_rager, emeria_angel, emeria_shepherd, sun_titan,
    multani_yavimayas_avatar, rampaging_baloths, sylvan_advocate, waker_of_the_wilds,
    living_twister, mina_and_denn_wildborn, omnath_locus_of_rage, abzan_falconer,
    elite_scaleguard, kor_cartographer, acidic_slime, armorcraft_judge, elvish_rejuvenator,
    embodiment_of_insight, evolution_sage, fertilid, keeper_of_fables, satyr_wayfinder,
    sporemound, springbloom_druid, tuskguard_captain, yavimaya_elder, sandstone_oracle,
    scaretiller, murasa_rootgrazer, hour_of_revelation, planar_outburst, nissas_renewal,
    beanstalk_giant, circuitous_route, far_wanderings, harmonize, kodamas_reach, ground_assault,
    treacherous_terrain, return_of_the_wildspeaker, condemn, crush_contraband, harrow,
    inspiring_call, naya_charm, sylvan_reclamation, roiling_regrowth, seers_sundial,
    arcane_signet, sol_ring, together_forever, abundance, the_mending_of_dominaria,
    rites_of_flourishing, banishing_light, retreat_to_emeria, khalni_heart_expedition,
    retreat_to_kazandu, zendikars_roil, needle_spires, blighted_woodland, boros_garrison,
    boros_guildgate, command_tower, cryptic_caves, evolving_wilds, gruul_guildgate, gruul_turf,
    jungle_shrine, krosan_verge, myriad_landscape, naya_panorama, selesnya_guildgate,
    selesnya_sanctuary, terramorphic_expanse, struggle_survive,
    // Basics: 7 plains, 4 mountain, 10 forest
    plains, plains, plains, plains, plains, plains, plains, mountain, mountain, mountain,
    mountain, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
];

pub const ZEDRUU_COMMANDERS: &[CardFactory] = &[zedruu_the_greathearted];

/// **Political Puppets**, the Commander (2011) Jeskai deck (CMD, 2011-06-17),
/// as MTGJSON's `PoliticalPuppets_CMD` prints it: 71 nonbasic cards + 8
/// Plains + 12 Islands + 8 Mountains = 99. Donations and table politics under
/// Zedruu.
/// One swap: Trade Secrets is banned in Commander, so Divination (the same
/// mana value, blue sorcery draw) takes its slot.
pub const ZEDRUU_MAIN: &[CardFactory] = &[
    goblin_cadets, spurnmage_advocate, jotun_grunt, wall_of_omens, fog_bank, nin_the_pain_artist,
    azorius_guildmage, court_hussar, gomazoa, guard_gomazoa, vedalken_plotter, wall_of_denial,
    plumeveil, flametongue_kavu, windborn_muse, false_prophet, brion_stoutarm, ruhan_of_the_fomori,
    chromeshell_crab, izzet_chronarch, dominus_of_fealty, rapacious_one, numot_the_devastator,
    arbiter_of_knollridge, breath_of_darigaaz, divination, death_by_dragons, austere_command,
    insurrection, skyscribing, brainstorm, flusterstorm, lash_out, punishing_fire, pollen_lullaby,
    perilous_research, vision_skeins, whirlpool_whelm, chaos_warp, oblation, murmurs_from_beyond,
    repulse, spell_crumple, wild_ricochet, reins_of_power, scattering_stroke, sol_ring,
    armillary_sphere, fellwar_stone, howling_mine, lightning_greaves, prophetic_prism,
    champions_helm, darksteel_ingot, dreamstone_hedron, soul_snare, journey_to_nowhere,
    vow_of_lightning, ghostly_prison, vow_of_duty, propaganda, vow_of_flight, prison_term,
    crescendo_of_war, martyrs_bond, azorius_chancery, boros_garrison, command_tower, evolving_wilds,
    izzet_boilerworks, terramorphic_expanse,
    // Basics: 8 plains, 12 island, 8 mountain
    plains, plains, plains, plains, plains, plains, plains, plains, island, island, island, island,
    island, island, island, island, island, island, island, island, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain,
];

pub const GHIRED_COMMANDERS: &[CardFactory] = &[ghired_conclave_exile];

/// **Primal Genesis**, the Commander 2019 deck (C19, 2019-08-23), exactly as
/// MTGJSON's `PrimalGenesis_C19` prints it: 80 nonbasic cards + 7 Plains + 4
/// Mountains + 8 Forests = 99. Naya tokens and populate under Ghired.
pub const GHIRED_MAIN: &[CardFactory] = &[
    garruk_primal_hunter, atla_palani_nest_tender, marisi_breaker_of_the_coil, doomed_artisan,
    tectonic_hellion, ohran_frostfang, selesnya_eulogist, tahngarth_first_mate,
    angel_of_sanctions, wingmate_roc, dragonmaster_outcast, feldon_of_the_third_path,
    giant_adephage, soul_of_zendikar, trostani_selesnyas_voice, desolation_twin,
    flamerush_rider, heart_piercer_manticore, rampaging_baloths, thragtusk, emmara_tandris,
    wayfaring_temple, cliffside_rescuer, voice_of_many, scaretiller, roc_egg,
    garruks_packleader, sakura_tribe_elder, vitu_ghazi_guildmage, ghireds_belligerence,
    full_flowering, hour_of_reckoning, phyrexian_rebirth, shamanic_revelation, hate_mirage,
    cultivate, explore, farseek, harmonize, fresh_meat, momentous_fall, second_harvest,
    rootborn_defenses, trostanis_judgment, beast_within, druids_deliverance, slice_in_twain,
    naya_charm, sundering_growth, idol_of_oblivion, mimic_vat, soul_foundry, lightning_greaves,
    sol_ring, commanders_insignia, song_of_the_worldsoul, growing_ranks, intangible_virtue,
    colossal_majesty, elemental_bond, cinder_glade, exotic_orchard, gargoyle_castle,
    sungrass_prairie, ash_barrens, blossoming_sands, boros_garrison, command_tower,
    evolving_wilds, graypelt_refuge, gruul_turf, jungle_shrine, kazandu_refuge, krosan_verge,
    myriad_landscape, naya_panorama, rogues_passage, rugged_highlands, selesnya_sanctuary,
    terramorphic_expanse,
    // Basics: 7 plains, 4 mountain, 8 forest
    plains, plains, plains, plains, plains, plains, plains,
    mountain, mountain, mountain, mountain,
    forest, forest, forest, forest, forest, forest, forest, forest,
];

pub const KYNAIOS_COMMANDERS: &[CardFactory] = &[kynaios_and_tiro_of_meletis];

/// **Stalwart Unity**, the Commander 2016 deck (C16, 2016-11-11), exactly as
/// MTGJSON's `StalwartUnity_C16` prints it: 79 nonbasic cards + 5 each of
/// Plains, Islands, Mountains and Forests = 99. Four-colour group hug under
/// Kynaios and Tiro of Meletis, with Ludevic, Kraum and Sidar Kondo — three
/// partner commanders — in the 99.
pub const KYNAIOS_MAIN: &[CardFactory] = &[
    veteran_explorer, humble_defector, hushwing_gryff, orzhov_advokist, chasm_skulker,
    gwafa_hazid_profiteer, ludevic_necro_alchemist, selvala_explorer_returned,
    edric_spymaster_of_trest, akroan_horse, selfless_squire, windborn_muse,
    sidar_kondo_of_jamuraa, horizon_chimera, zedruu_the_greathearted, psychosis_crawler,
    kazuul_tyrant_of_the_cliffs, kraum_ludevics_opus, realm_seekers, rubblehulk,
    progenitor_mimic, blazing_archon, minds_aglow, collective_voyage, cultivate, kodamas_reach,
    tempt_with_discovery, wave_of_reckoning, migratory_route, seeds_of_renewal,
    reverse_the_sands, treacherous_terrain, blasphemous_act, swords_to_plowshares, swan_song,
    arcane_denial, benefactors_draught, oblation, beast_within, entrapment_maneuver,
    reins_of_power, sylvan_reclamation, sol_ring, empyrial_plate, howling_mine,
    commanders_sphere, temple_bell, assault_suit, prismatic_geoscope, vensers_journal,
    keening_stone, evolutionary_escalation, oath_of_druids, ghostly_prison, propaganda,
    rites_of_flourishing, sphere_of_safety, lurking_predators, hoofprints_of_the_stag,
    ash_barrens, azorius_chancery, command_tower, evolving_wilds, exotic_orchard,
    forbidden_orchard, frontier_bivouac, gruul_turf, homeward_path, izzet_boilerworks,
    jungle_shrine, krosan_verge, myriad_landscape, mystic_monastery, opal_palace, rupture_spire,
    seaside_citadel, selesnya_sanctuary, terramorphic_expanse, transguild_promenade,
    // Basics: 5 plains, 5 island, 5 mountain, 5 forest
    plains, plains, plains, plains, plains, island, island, island, island, island,
    mountain, mountain, mountain, mountain, mountain, forest, forest, forest, forest, forest,
];

pub const MIMEOPLASM_COMMANDERS: &[CardFactory] = &[the_mimeoplasm];

/// **Devour for Power**, the Commander (2011) Sultai deck (CMD, 2011-06-17),
/// exactly as MTGJSON's `DevourForPower_CMD` prints it: 72 nonbasic cards +
/// 8 Forests + 8 Islands + 11 Swamps = 99. Graveyard value under The
/// Mimeoplasm, with Damia and Vorosh in the 99.
pub const MIMEOPLASM_MAIN: &[CardFactory] = &[
    nezumi_graverobber, skullbriar_the_walking_grave, riddlekeeper, fleshbag_marauder,
    eternal_witness, troll_ascetic, yavimaya_elder, solemn_simulacrum, brawn, wonder,
    sewer_nemesis, gravedigger, lhurgoyf, dreamborn_muse, mortivore, desecrator_hag,
    mulldrifter, acidic_slime, vulturous_zombie, dark_hatchling, extractor_demon,
    scythe_specter, wrexial_the_risen_deep, vorosh_the_hunter, triskelavus, slipstream_eel,
    butcher_of_malakir, patron_of_the_nezumi, damia_sage_of_stone, szadek_lord_of_secrets,
    avatar_of_woe, artisan_of_kozilek, minds_aglow, shared_trauma, sign_in_blood,
    stitch_together, cultivate, windfall, buried_alive, syphon_mind, unnerve,
    rise_from_the_grave, syphon_flesh, living_death, tribute_to_the_wild, spell_crumple,
    fact_or_fiction, relic_crush, sol_ring, dimir_signet, golgari_signet, lightning_greaves,
    simic_signet, oblivion_stone, vow_of_wildness, vow_of_flight, vow_of_malice, memory_erosion,
    grave_pact, barren_moor, command_tower, dimir_aqueduct, dreadship_reef, golgari_rot_farm,
    jwar_isle_refuge, lonely_sandbar, rupture_spire, simic_growth_chamber,
    svogthos_the_restless_tomb, temple_of_the_false_god, terramorphic_expanse, tranquil_thicket,
    // Basics: 8 forest, 8 island, 11 swamp
    forest, forest, forest, forest, forest, forest, forest, forest, island, island, island,
    island, island, island, island, island, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp,
];

pub const VALGAVOTH_COMMANDERS: &[CardFactory] = &[valgavoth_harrower_of_souls];

/// **Endless Punishment**, the Duskmourn Commander deck (DSC, 2024-09-27),
/// exactly as MTGJSON's `EndlessPunishment_DSC` prints it: 83 nonbasic cards
/// + 8 Swamps + 8 Mountains = 99. Rakdos punisher under Valgavoth, Harrower
/// of Souls: opponents lose life on their own turns and can't gain it.
pub const VALGAVOTH_MAIN: &[CardFactory] = &[
    the_lord_of_pain, persistent_constrictor, sadistic_shell_game, suspended_sentence,
    barbflare_gremlin, gleeful_arsonist, spiked_corridor_torture_pit, star_athlete, seance_board, bedevil,
    mogis_god_of_slaughter, braids_arisen_nightmare, decree_of_pain, fate_unraveler,
    kederekt_parasite, mask_of_griselbrand, massacre_girl, massacre_wurm, nightshade_harvester,
    blasphemous_act, brash_taunter, chaos_warp, combustible_gearhulk, enchanters_bane,
    harsh_mentor, rampaging_ferocidon, tectonic_giant, florian_voldaren_scion,
    kaervek_the_merciless, rakdos_lord_of_riots, spiteful_visions, stormfist_crusader,
    theater_of_horrors, vial_smasher_the_fierce, basilisk_collar, solemn_simulacrum,
    blackcleave_cliffs, canyon_slough, dragonskull_summit, exotic_orchard, foreboding_ruins,
    graven_cairns, shadowblood_ridge, shivan_gorge, smoldering_marsh, spinerock_knoll,
    sulfurous_springs, temple_of_malice, witchs_clinic, fear_of_burning_alive, grab_the_prize,
    terramorphic_expanse, blood_pact, blood_seeker, feed_the_swarm, arcane_signet,
    lightning_greaves, sol_ring, command_tower, bastion_of_remembrance, blood_artist,
    falkenrath_noble, gray_merchant_of_asphodel, infernal_grasp, morbid_opportunist,
    sign_in_blood, syr_konrad_the_grim, light_up_the_stage, kardur_doomscourge, mayhem_devil,
    rakdos_charm, fellwar_stone, mind_stone, rakdos_signet, talisman_of_indulgence,
    thought_vessel, ash_barrens, bloodfell_caves, evolving_wilds, geothermal_bog,
    leechridden_swamp, tainted_peak, temple_of_the_false_god,
    // Basics: 8 swamp, 8 mountain
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain,
];

pub const EMMARA_COMMANDERS: &[CardFactory] = &[emmara_soul_of_the_accord];

/// **Token Triumph**, the Starter Commander Decks' Selesnya deck (SCD,
/// 2022-12-02), exactly as MTGJSON's `TokenTriumph_SCD` prints it: 70
/// nonbasic cards + 15 Forests + 14 Plains = 99. Tokens and anthems under
/// Emmara, Soul of the Accord.
pub const EMMARA_MAIN: &[CardFactory] = &[
    ajani_caller_of_the_pride, citywide_bust, commanders_insignia, dawn_of_hope,
    dictate_of_heliod, felidar_retreat, hour_of_reckoning, mentor_of_the_meek,
    white_suns_zenith, champion_of_lambholt, citanul_hierophants, collective_unconscious,
    harvest_season, hornet_nest, hornet_queen, rishkar_peema_renegade, scavenging_ooze,
    thunderfoot_baloth, verdant_force, aura_mutation, camaraderie, collective_blessing,
    dauntless_escort, march_of_the_multitudes, trostani_discordant, idol_of_oblivion,
    slate_of_ancestry, canopy_vista, fortified_village, temple_of_plenty, conclave_tribunal,
    devouring_light, path_to_exile, rootborn_defenses, valor_in_akros, avacyns_pilgrim,
    curse_of_bounty, eternal_witness, farhaven_elf, great_oak_guardian, harmonize, jade_mage,
    jaspera_sentinel, karametras_favor, leafkin_druid, loyal_guardian, nissas_expedition,
    nullmage_shepherd, overrun, overwhelming_instinct, presence_of_gond, reclamation_sage,
    scatter_the_seeds, sporemound, voice_of_many, maja_bretagard_protector, selesnya_evangel,
    selesnya_guildmage, sylvan_reclamation, arcane_signet, commanders_sphere, sol_ring,
    talisman_of_unity, blossoming_sands, command_tower, elfhame_palace, graypelt_refuge,
    holdout_settlement, tranquil_expanse, vitu_ghazi_the_city_tree,
    // Basics: 15 forest, 14 plains
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest,
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains,
];

pub const ATRAXA_COMMANDERS: &[CardFactory] = &[atraxa_praetors_voice];

/// **Breed Lethality**, the Commander 2016 deck (C16, 2016-11-11), exactly as
/// MTGJSON's `BreedLethality_C16` prints it: 78 nonbasic cards + 5 Plains + 4
/// Islands + 5 Swamps + 7 Forests = 99. Four-color +1/+1 counters and
/// proliferate under Atraxa, Praetors' Voice.
pub const ATRAXA_MAIN: &[CardFactory] = &[
    thrummingbird, festercreep, scavenging_ooze, abzan_falconer, orzhov_advokist,
    tuskguard_captain, necroplasm, champion_of_lambholt, reyhan_last_of_the_abzan,
    vorel_of_the_hull_clade, crystalline_crawler, custodi_soulbinders, forgotten_ancient,
    bane_of_the_living, ishai_ojutai_dragonspeaker, corpsejack_menace, fathom_mage,
    master_biomancer, elite_scaleguard, reveillark, deepglow_skate, kalonian_hydra,
    ikra_shidiqi_the_usurper, vulturous_zombie, juniper_order_ranger, ghave_guru_of_spores,
    enduring_scalelord, manifold_insights, languish, tezzerets_gambit, migratory_route,
    merciless_eviction, spitting_image, sublime_exhalation, duneblast, treasure_cruise,
    disdainful_stroke, solidarity_of_heroes, grip_of_phyresis, inspiring_call, mortify, putrefy,
    ancient_excavation, mirrorweave, sylvan_reclamation, sol_ring, fellwar_stone,
    golgari_signet, orzhov_signet, simic_signet, commanders_sphere, darksteel_ingot,
    cauldron_of_souls, astral_cornucopia, hardened_scales, brave_the_sands, duelists_heritage,
    bred_for_the_hunt, citadel_siege, cathars_crusade, arcane_sanctum, ash_barrens,
    azorius_chancery, command_tower, darkwater_catacombs, dreadship_reef, evolving_wilds,
    exotic_orchard, golgari_rot_farm, murmuring_bosk, opal_palace, opulent_palace,
    sandsteppe_citadel, seaside_citadel, sungrass_prairie, temple_of_the_false_god,
    terramorphic_expanse, underground_river,
    // Basics: 5 plains, 4 island, 5 swamp, 7 forest
    plains, plains, plains, plains, plains,
    island, island, island, island,
    swamp, swamp, swamp, swamp, swamp,
    forest, forest, forest, forest, forest, forest, forest,
];

pub const RIKU_COMMANDERS: &[CardFactory] = &[riku_of_two_reflections];

/// **Mirror Mastery**, the Commander (2011) Temur deck (CMD, 2011-06-17),
/// exactly as MTGJSON's `MirrorMastery_CMD` prints it: 71 nonbasic cards +
/// 7 Islands + 8 Mountains + 13 Forests = 99. Copies and
/// big creatures under Riku of Two Reflections.
pub const RIKU_MAIN: &[CardFactory] = &[
    garruk_wildspeaker, magus_of_the_vineyard, veteran_explorer, fierce_empath,
    edric_spymaster_of_trest, animar_soul_of_elements, conundrum_sphinx, aethersnipe,
    chartooth_cougar, rapacious_one, spitebellows, deadwood_treefolk, elvish_aberration,
    baloth_woodcrasher, hydra_omnivore, nucklavee, valley_rannet, intet_the_dreamer,
    faultgrinder, krosan_tusker, simic_sky_swallower, trench_gorger, avatar_of_fury,
    magmatic_force, artisan_of_kozilek, collective_voyage, hull_breach, cultivate,
    kodamas_reach, firespout, ruination, explosive_vegetation, chain_reaction, death_by_dragons,
    vengeful_rebirth, disaster_radius, call_the_skybreaker, savage_twister, brainstorm,
    tribute_to_the_wild, colossal_might, invigorate, spell_crumple, electrolyze, ray_of_command,
    prophetic_bolt, hunting_pack, sol_ring, armillary_sphere, gruul_signet, izzet_signet,
    lightning_greaves, prophetic_prism, simic_signet, vow_of_flight, vow_of_lightning,
    vow_of_wildness, command_tower, evolving_wilds, fungal_reaches, gruul_turf, homeward_path,
    izzet_boilerworks, kazandu_refuge, rupture_spire, simic_growth_chamber,
    temple_of_the_false_god, vivid_crag, vivid_creek, vivid_grove, fire_ice,
    // Basics: 7 island, 8 mountain, 13 forest
    island, island, island, island, island, island, island, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, forest, forest, forest, forest, forest,
    forest, forest, forest, forest, forest, forest, forest, forest,
];

pub const RIN_SERI_COMMANDERS: &[CardFactory] = &[rin_and_seri_inseparable];

/// **Raining Cats and Dogs**, the Secret Lair Commander deck (SLD,
/// 2024-01-22), exactly as MTGJSON's `RainingCatsAndDogs_SLD` prints it: 77
/// nonbasic cards + 10 Plains + 5 Mountains + 7 Forests = 99. Naya Cats and
/// Dogs tokens under Rin and Seri, Inseparable.
pub const RIN_SERI_MAIN: &[CardFactory] = &[
    jetmir_nexus_of_revels, jinnie_fay_jetmirs_second, anointed_procession, sol_ring,
    alms_collector, highcliff_felidar, kitt_kanto_mayhem_diva, brimaz_king_of_oreskos,
    stalking_leonin, cast_out, phabine_bosss_confidant, crib_swap, path_to_exile,
    king_of_the_pride, felidar_retreat, white_suns_zenith, komainu_battle_armor,
    skyhunter_strike_force, whitemane_lion, loyal_warhound, regal_caracal, lion_sash,
    pack_leader, tocasias_welcome, hungry_lynx, sehts_tiger, mirror_entity, qasali_slingers,
    spirited_companion, marisi_breaker_of_the_coil, jungle_shrine, cursed_mirror, heralds_horn,
    path_of_ancestry, oreskos_explorer, jazal_goldmane, fleetfoot_panther, taurean_mauler,
    krosan_verge, impact_tremors, temur_sabertooth, exotic_orchard, nacatl_war_pride,
    warp_world, arcane_signet, beastmaster_ascension, keeper_of_fables, natures_lore,
    command_tower, return_of_the_wildspeaker, feline_sovereign, masked_vandal, realmwalker,
    greater_tanuki, lurking_predators, rootbound_crag, sunpetal_grove, showdown_of_the_skalds,
    basilisk_collar, oketras_monument, canopy_vista, cinder_glade, bloodline_pretender,
    clifftop_retreat, maskwood_nexus, animal_sanctuary, scattered_groves, sheltered_thicket,
    jetmirs_garden, skullclamp, vanquishers_banner, three_visits, cultivate, farseek,
    fortified_village, game_trail, dusk_dawn,
    // Basics: 10 plains, 5 mountain, 7 forest
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    mountain, mountain, mountain, mountain, mountain,
    forest, forest, forest, forest, forest, forest, forest,
];

pub const SASKIA_COMMANDERS: &[CardFactory] = &[saskia_the_unyielding];

/// **Open Hostility**, the Commander 2016 deck (C16, 2016-11-11), exactly as
/// MTGJSON's `OpenHostility_C16` prints it: 83 nonbasic cards + 3 Plains + 3
/// Swamps + 5 Mountains + 5 Forests = 99. Four-color (no blue) aggression
/// under Saskia, with Tana, Tymna and Ravos in the 99.
pub const SASKIA_MAIN: &[CardFactory] = &[
    wight_of_precinct_six, den_protector, quirion_explorer, sakura_tribe_elder, sylvok_explorer,
    korozda_guildmage, zhur_taa_druid, selesnya_guildmage, mentor_of_the_meek, mirror_entity,
    alesha_who_smiles_at_death, taurean_mauler, managorger_hydra, wild_beastmaster,
    tymna_the_weaver, wilderness_elemental, dauntless_escort, brutal_hordechief,
    charging_cinderhorn, thelonite_hermit, tana_the_bloodsower, iroas_god_of_victory, mycoloth,
    ravos_soultender, ankle_shanker, thunderfoot_baloth, stalking_vengeance, stonehoof_chieftain,
    primeval_protector, farseek, rampant_growth, shamanic_revelation, grave_upheaval,
    treacherous_terrain, clan_defiance, lavalanche, terminate, artifact_mutation, boros_charm,
    aura_mutation, abzan_charm, naya_charm, crackling_doom, grab_the_reins, utter_end,
    sylvan_reclamation, divergent_transformations, order_chaos, skullclamp, sol_ring, conquerors_flail,
    fellwar_stone, gruul_signet, lightning_greaves, commanders_sphere, sunforger, blind_obedience,
    evolutionary_escalation, necrogenesis, beastmaster_ascension, everlasting_torment,
    frenzied_fugue, breath_of_fury, ash_barrens, caves_of_koilos, command_tower, dragonskull_summit,
    evolving_wilds, exotic_orchard, grand_coliseum, gruul_turf, jungle_shrine, karplusan_forest,
    mosswort_bridge, nomad_outpost, orzhov_basilica, rootbound_crag, sandsteppe_citadel,
    savage_lands, spinerock_knoll, sunpetal_grove, terramorphic_expanse, windbrisk_heights,
    // Basics: 3 plains, 3 swamp, 5 mountain, 5 forest
    plains, plains, plains, swamp, swamp, swamp,
    mountain, mountain, mountain, mountain, mountain,
    forest, forest, forest, forest, forest,
];

pub const ISPERIA_COMMANDERS: &[CardFactory] = &[isperia_supreme_judge];

/// **First Flight**, the Starter Commander Azorius deck (SCD, 2022-12-02),
/// exactly as MTGJSON's `FirstFlight_SCD` prints it: 69 nonbasic cards +
/// 15 Plains + 15 Islands = 99. Fliers under Isperia, Supreme Judge.
pub const ISPERIA_MAIN: &[CardFactory] = &[
    archon_of_redemption, cartographers_hawk, cleansing_nova, emeria_angel, gideon_jura,
    hanged_executioner, remorseful_cleric, sephara_skys_blade, steel_plume_marshal, storm_herd,
    true_conviction, angler_turtle, bident_of_thassa, diluvian_primordial,
    ever_watching_threshold, faerie_formation, gravitational_shift, inspired_sphinx,
    sharding_sphinx, sphinx_of_enlightenment, windreader_sphinx, absorb, skycat_sovereign,
    sphinxs_revelation, time_wipe, moorland_haunt, port_town, prairie_stream,
    temple_of_enlightenment, aven_gagglemaster, banishing_light, condemn, crush_contraband,
    disenchant, generous_gift, kangees_lieutenant, rally_of_wings, soul_snare,
    swords_to_plowshares, vow_of_duty, aetherize, counterspell, favorable_winds, negate,
    tide_skimmer, warden_of_evos_isle, winged_words, cloudblazer, empyrean_eagle,
    jubilant_skybonder, kangee_sky_warden, migratory_route, staggering_insight,
    thunderclap_wyvern, arcane_signet, azorius_signet, commanders_sphere, hedron_archive,
    pilgrims_eye, sky_diamond, skyscanner, sol_ring, talisman_of_progress, thought_vessel,
    coastal_tower, command_tower, meandering_river, sejiri_refuge, tranquil_cove,
    // Basics: 15 plains, 15 island
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, island, island, island, island, island, island, island,
    island, island, island, island, island, island, island, island,
];

pub const INALLA_COMMANDERS: &[CardFactory] = &[inalla_archmage_ritualist];

/// **Arcane Wizardry**, the Commander 2017 deck (C17, 2017-08-25), exactly as
/// MTGJSON's `ArcaneWizardry_C17` prints it: 79 nonbasic cards + 10 Islands +
/// 6 Swamps + 4 Mountains = 99. Grixis Wizards under Inalla, Archmage
/// Ritualist, whose eminence copies each Wizard that enters.
pub const INALLA_MAIN: &[CardFactory] = &[
    kess_dissident_mage, mairsil_the_pretender, galecaster_colossus, magus_of_the_mind,
    portal_mage, vindictive_lich, izzet_chemister, taigam_sidisis_hand, havengul_lich,
    marchesa_the_black_rose, vela_the_night_clad, arcanis_the_omnipotent, azami_lady_of_scrolls,
    body_double, harbinger_of_the_tides, serendib_sorcerer, apprentice_necromancer,
    magus_of_the_abyss, puppeteer_clique, etherium_horn_sorcerer, mercurial_chemister,
    nin_the_pain_artist, niv_mizzet_the_firemind, shadowmage_infiltrator, bloodline_necromancer,
    archaeomancer, merchant_of_secrets, sea_gate_oracle, corpse_augur, izzet_chronarch,
    nivix_guildmage, kindred_dominance, clone_legion, spelltwine, decree_of_pain,
    necromantic_selection, comet_storm, polymorphists_jest, chaos_warp, memory_plunder,
    silumgars_command, into_the_roil, opportunity, reality_shift, go_for_the_throat,
    cauldron_dance, crosiss_charm, rakdos_charm, terminate, nevinyrrals_disk,
    mirror_of_the_forebears, commanders_sphere, darksteel_ingot, fellwar_stone, sol_ring,
    unstable_obelisk, worn_powerstone, shifting_shadow, curse_of_verbosity,
    curse_of_disturbance, curse_of_opulence, exotic_orchard, mystifying_maze, path_of_ancestry,
    command_tower, crumbling_necropolis, dimir_aqueduct, dismal_backwater, evolving_wilds,
    grixis_panorama, izzet_boilerworks, jwar_isle_refuge, rakdos_carnarium, swiftwater_cliffs,
    temple_of_the_false_god, terramorphic_expanse, vivid_crag, vivid_creek, vivid_marsh,
    // Basics: 10 island, 6 swamp, 4 mountain
    island, island, island, island, island, island, island, island, island, island, swamp,
    swamp, swamp, swamp, swamp, swamp, mountain, mountain, mountain, mountain,
];

pub const BRIMAZ_COMMANDERS: &[CardFactory] = &[brimaz_blight_of_oreskos];

/// **Growing Threat**, the March of the Machine Commander deck (MOC,
/// 2023-04-21), exactly as MTGJSON's `GrowingThreat_MOC` prints it: 76
/// nonbasic cards + 10 Plains + 13 Swamps = 99. Orzhov Phyrexians, incubate
/// and proliferate under Brimaz, Blight of Oreskos.
pub const BRIMAZ_MAIN: &[CardFactory] = &[
    moira_and_teshar, ichor_elixir, blight_titan, darksteel_splicer, excise_the_imperfect,
    filigree_vector, path_of_the_schemer, bitterthorn_nissas_animus, vulpine_harvester,
    cataclysmic_gearhulk, massacre_wurm, noxious_gearhulk, phyrexian_scriptures,
    phyrexian_triniform, soul_of_new_phyrexia, ancient_stone_idol, angel_of_the_ruins,
    blade_splicer, coveted_jewel, duplicant, exotic_orchard, fetid_heath, karns_bastion,
    myr_battlesphere, nettlecyst, phyrexian_delver, phyrexian_rebirth, psychosis_crawler,
    scrap_trawler, sculpting_steel, scytheclaw, shineshadow_snarl, spire_of_industry,
    temple_of_silence, utter_end, vault_of_the_archangel, yawgmoths_vile_offering, bojuka_bog,
    command_tower, commanders_sphere, evolving_wilds, first_sphere_gargantua,
    fractured_powerstone, goldmire_bridge, nights_whisper, orzhov_locket, orzhov_signet,
    path_of_ancestry, phyrexian_ghoul, phyrexian_rager, silverquill_campus,
    terramorphic_expanse, wayfarers_bauble, hedron_archive, ambitions_cost, arcane_signet,
    bloodline_pretender, bone_shredder, burnished_hart, despark, go_for_the_throat,
    graveshifter, keskit_the_flesh_sculptor, master_splicer, meteor_golem, mind_stone, mortify,
    shattered_angel, shimmer_myr, sol_ring, swords_to_plowshares, tainted_field,
    talisman_of_hierarchy, victimize, compleated_huntmaster, phyrexian_gargantua,
    // Basics: 10 plains, 13 swamp
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
];

pub const ANIKTHEA_COMMANDERS: &[CardFactory] = &[anikthea_hand_of_erebos];

/// **Enduring Enchantments**, the Commander Masters Abzan deck (CMM,
/// 2023-08-04), exactly as MTGJSON's `EnduringEnchantments_CMM` prints it:
/// 80 nonbasic cards + 6 Plains + 5 Swamps + 8 Forests = 99. Sagas,
/// enchantresses and constellation under Anikthea, Hand of Erebos.
pub const ANIKTHEA_MAIN: &[CardFactory] = &[
    narci_fable_singer, battle_at_the_helvault, boon_of_the_spirit_realm, ondu_spiritdancer,
    cacophony_unleashed, demon_of_fates_design, ghoulish_impetus, composer_of_spring,
    nyxborn_behemoth, archon_of_suns_grace, felidar_retreat, grasp_of_fate,
    heliod_god_of_the_sun, mesa_enchantress, sigil_of_the_empty_throne, starfield_mystic,
    starfield_of_nyx, cunning_rhetoric, doomwake_giant, dreadhorde_invasion,
    erebos_bleak_hearted, abundance, arasta_of_the_endless_web, courser_of_kruphix,
    dryad_of_the_ilysian_grove, eidolon_of_blossoms, enchantresss_presence,
    herald_of_the_pantheon, the_mending_of_dominaria, sanctum_weaver, sandwurm_convergence,
    setessan_champion, verduran_enchantress, battle_for_bretagard, calix_destinys_hand,
    culling_ritual, miraris_wake, sythis_harvests_hand, canopy_vista, exotic_orchard,
    fortified_village, necroblossom_snarl, shineshadow_snarl, sungrass_prairie,
    temple_of_malady, temple_of_plenty, temple_of_silence, path_to_exile,
    extinguish_all_hope, kodamas_reach, arcane_signet, sol_ring, ash_barrens, command_tower,
    cast_out, love_song_of_night_and_day, omen_of_the_sun, spirited_companion,
    the_eldest_reborn, mindwrack_harpy, the_binding_of_the_titans, destiny_spinner, farseek,
    font_of_fertility, greater_tanuki, khalni_heart_expedition, nessian_wanderer,
    omen_of_the_hunt, rampant_growth, binding_the_old_gods, jukai_naturalist, nyx_weaver,
    satyr_enchanter, golgari_rot_farm, krosan_verge, orzhov_basilica, sandsteppe_citadel,
    selesnya_sanctuary, tainted_field, tainted_wood,
    // Basics: 6 plains, 5 swamp, 8 forest
    plains, plains, plains, plains, plains, plains, swamp, swamp, swamp, swamp, swamp, forest,
    forest, forest, forest, forest, forest, forest, forest,
];

pub const URZA_COMMANDERS: &[CardFactory] = &[urza_chief_artificer];

/// **Urza's Iron Alliance**, The Brothers' War Commander deck (BRC,
/// 2022-11-18), exactly as MTGJSON's `UrzaSIronAlliance_BRC` prints it: 88
/// nonbasic cards + 4 Plains + 4 Islands + 3 Swamps = 99. Esper artifacts under
/// Urza, Chief Artificer.
pub const URZA_MAIN: &[CardFactory] = &[
    indomitable_archangel, ethersworn_adjudicator, noxious_gearhulk, alela_artful_provocateur,
    angel_of_the_ruins, bronze_guardian, digsite_engineer, losheel_clockwork_scholar,
    teshar_ancestors_apostle, master_of_etherium, sai_master_thopterist, sharding_sphinx,
    shimmer_dragon, thought_monitor, vedalken_humiliator, marionette_master, baleful_strix,
    sharuum_the_hegemon, darksteel_juggernaut, etched_champion, myr_battlesphere,
    solemn_simulacrum, steel_hellkite, steel_overseer, filigree_attendant, whirler_rogue,
    armix_filigree_thrasher, chief_of_the_foundry, etherium_sculptor, chrome_courier,
    austere_command, phyrexian_rebirth, urzas_ruinous_blast, one_with_the_machine, vindicate,
    preordain, sphinxs_revelation, unbreakable_formation, swords_to_plowshares, despark,
    bident_of_thassa, cranial_plating, liquimetal_torque, relic_of_progenitus, skullclamp,
    sol_ring, swiftfoot_boots, thought_vessel, arcane_signet, azorius_signet, dimir_signet,
    orzhov_signet, tempered_steel, thopter_spy_network, exotic_orchard, prairie_stream,
    river_of_tears, skycloud_expanse, spire_of_industry, sunken_hollow, temple_of_deceit,
    temple_of_enlightenment, temple_of_silence, arcane_sanctum, azorius_chancery,
    darksteel_citadel, dimir_aqueduct, orzhov_basilica, ancient_den, ash_barrens, bojuka_bog,
    command_tower, evolving_wilds, goldmire_bridge, mistvault_bridge, path_of_ancestry,
    razortide_bridge, seat_of_the_synod, vault_of_whispers, tawnos_solemn_survivor,
    sanwell_avenger_ace, scholar_of_new_horizons, march_of_progress, wire_surgeons,
    wreck_hunter, hexavus, kaylas_music_box, thopter_shop,
    // Basics: 4 plains, 4 island, 3 swamp
    plains, plains, plains, plains, island, island, island, island, swamp, swamp, swamp,
];

pub const RANAR_COMMANDERS: &[CardFactory] = &[ranar_the_ever_watchful];

/// **Phantom Premonition**, the Kaldheim Commander deck (KHC, 2021-02-05),
/// exactly as MTGJSON's `PhantomPremonition_KHC` prints it: 74 nonbasic
/// cards + 13 Plains + 12 Islands = 99. Azorius foretell
/// and flicker under Ranar the Ever-Watchful.
pub const RANAR_MAIN: &[CardFactory] = &[
    angel_of_finality, angel_of_serenity, arcane_artisan, brago_king_eternal, burnished_hart,
    cloudblazer, cloudgoat_ranger, empyrean_eagle, ethereal_valkyrie, evangel_of_heliod,
    flickerwisp, geist_honored_monk, goldnight_commander, hero_of_bretagard, inspired_sphinx,
    kor_cartographer, meteor_golem, mist_raven, mistmeadow_witch, mulldrifter, restoration_angel,
    sage_of_the_beyond, sea_gate_oracle, soulherder, stoic_farmer, sun_titan, surtland_elementalist,
    thunderclap_wyvern, vega_the_watcher, wall_of_omens, whirler_rogue, behold_the_multiverse,
    cosmic_intervention, eerie_interlude, ghostly_flicker, iron_verdict, momentary_blink,
    return_to_dust, saw_it_coming, synthetic_destiny, warhorn_blast, cleansing_nova,
    curse_of_the_swine, migratory_route, ravenform, spectral_deluge, storm_herd,
    tales_of_the_ancestors, windfall, banishing_light, day_of_the_dragons, ghostly_prison,
    marshals_anthem, niko_defies_destiny, arcane_signet, azorius_signet, commanders_sphere,
    marble_diamond, mind_stone, replicating_ring, sky_diamond, sol_ring, swiftfoot_boots,
    azorius_chancery, azorius_guildgate, command_tower, cryptic_caves, gates_of_istfell,
    glacial_floodplain, meandering_river, myriad_landscape, opal_palace, sejiri_refuge,
    tranquil_cove,
    // Basics: 13 plains, 12 island
    island, island, island, island, island, island, island, island, island, island, island, island,
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains,
];

pub const TROSTANI_COMMANDERS: &[CardFactory] = &[trostani_selesnyas_voice];

/// **Hatsune Miku**, the Secret Lair Commander deck (SLD, 2026-08-10), exactly
/// as MTGJSON's `HatsuneMiku_SLD` prints it: 85 nonbasic cards + 7 Plains +
/// 7 Forests = 99. Selesnya lifegain and tokens under Trostani,
/// Selesnya's Voice, whose populate copies the best token on the board.
pub const TROSTANI_MAIN: &[CardFactory] = &[
    archangel_of_thune, halo_fountain, grand_crescendo, shalai_voice_of_plenty,
    song_of_the_worldsoul, soul_warden, break_down, cultivate, finale_of_devastation,
    vorinclex_voice_of_hunger, bountiful_promenade, angel_of_indemnity, angelic_chorus,
    boon_reflection, crested_sunmare, dazzling_theater_prop_room, elendas_hierophant,
    excavation_technique, hour_of_reckoning, nykthos_paragon, resplendent_angel,
    silverquill_lecturer, soul_of_eternity, speaker_of_the_heavens, storm_herd,
    voice_of_the_blessed, ancient_cornucopia, arasta_of_the_endless_web, blossoming_bogbeast,
    bramble_sovereign, fanatic_of_rhonas, gruff_triplets, healing_technique, pest_infestation,
    shamanic_revelation, camaraderie, conclave_evangelist, ghalta_and_mavren, growing_ranks,
    lathiel_the_bounteous_dawn, miraris_wake, rhys_the_redeemed, voice_of_resurgence,
    aetherflux_reservoir, phyrexian_processor, canopy_vista, gavony_township,
    grove_of_the_guardian, lazotep_quarry, overgrown_farmland, restless_prairie,
    sungrass_prairie, sunpetal_grove, temple_of_plenty, invincible_hymn, ajanis_pridemate,
    cleric_class, congregate, path_to_exile, rootborn_defenses, suture_priest,
    swords_to_plowshares, avacyns_pilgrim, explore, farseek, llanowar_elves, natures_lore,
    prosperous_innkeeper, song_of_freyalise, sundering_growth, idol_of_oblivion,
    selesnya_signet, skullclamp, sol_ring, springleaf_drum, blossoming_sands, brokers_hideout,
    command_tower, graypelt_refuge, krosan_verge, radiant_fountain, rogues_passage,
    sapseep_forest, selesnya_sanctuary, seraph_sanctuary,
    // Basics: 7 plains, 7 forest
    plains, plains, plains, plains, plains, plains, plains, forest, forest, forest, forest,
    forest, forest, forest,
];

pub const FALDORN_COMMANDERS: &[CardFactory] = &[faldorn_dread_wolf_herald];

/// **Exit from Exile**, the Battle for Baldur's Gate Gruul deck (CLB,
/// 2022-06-10), exactly as MTGJSON's `ExitFromExile_CLB` prints it:
/// 76 nonbasic cards + 11 Mountains + 12 Forests = 99. Impulse draw,
/// cascade and Wolves under Faldorn, Dread Wolf Herald.
pub const FALDORN_MAIN: &[CardFactory] = &[
    vivien_champion_of_the_wilds, xenagos_the_reveler, aurora_phoenix, bonecrusher_giant,
    dire_fleet_daredevil, dream_pillager, etali_primal_storm, greater_gargadon, izzet_chemister,
    laelia_the_blade_reforged, tectonic_giant, urabrask_the_hidden, wild_magic_sorcerer,
    arasta_of_the_endless_web, battle_mammoth, end_raze_forerunners, hornet_queen,
    lovestruck_beast, managorger_hydra, sweet_gum_recluse, embereth_shieldbreaker,
    sakura_tribe_elder, bloodbraid_elf, grumgully_the_generous, ignite_the_future, jeskas_will,
    mizzium_mortars, ezuris_predation, escape_to_the_wilds, natures_lore, light_up_the_stage,
    beanstalk_giant, cultivate, explore, kodamas_reach, search_for_tomorrow, terramorph,
    three_visits, return_of_the_wildspeaker, demon_bolt, beast_within, natural_reclamation,
    chaos_wand, arcane_signet, sol_ring, outpost_siege, stolen_strategy, warstorm_surge,
    primeval_bounty, sandwurm_convergence, castle_embereth, cinder_glade, game_trail,
    kessig_wolf_run, mossfire_valley, mosswort_bridge, raging_ravine, spinerock_knoll,
    temple_of_abandon, command_tower, ash_barrens, blighted_woodland, gruul_turf,
    highland_forest, myriad_landscape, temple_of_the_false_god, durnan_of_the_yawning_portal,
    passionate_archaeologist, delayed_blast_fireball, nalfeshnee, green_slime,
    journey_to_the_lost_city, tlincalli_hunter, venture_forth, sarevoks_tome, volcanic_torrent,
    // Basics: 11 mountain, 12 forest
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest,
];

pub const WILHELT_COMMANDERS: &[CardFactory] = &[wilhelt_the_rotcleaver];

/// **Undead Unleashed**, the Innistrad: Midnight Hunt Commander deck (MIC,
/// 2021-09-24), exactly as MTGJSON's `UndeadUnleashed_MIC` prints it:
/// 72 nonbasic cards + 12 Islands + 15 Swamps = 99. Dimir Zombies
/// under Wilhelt, the Rotcleaver.
pub const WILHELT_MAIN: &[CardFactory] = &[
    liliana_deaths_majesty, forgotten_creation, havengul_runebinder, stitcher_geralf,
    undead_alchemist, butcher_of_malakir, cemetery_reaper, death_baron, diregraf_colossus,
    eater_of_hope, gravespawn_sovereign, midnight_reaper, overseer_of_the_damned,
    gisa_and_geralf, eternal_skylord, corpse_augur, fleshbag_marauder, lilianas_devotee,
    lord_of_the_accursed, spark_reaper, undead_augur, diregraf_captain, gleaming_overseer,
    ruthless_deathfang, hour_of_eternity, army_of_the_damned, dark_salvation, dread_summons,
    zombie_apocalypse, distant_melody, feed_the_swarm, syphon_flesh, aetherspouts,
    go_for_the_throat, arcane_signet, charcoal_diamond, commanders_sphere, sky_diamond,
    sol_ring, talisman_of_dominance, rooftop_storm, dreadhorde_invasion,
    endless_ranks_of_the_dead, lilianas_mastery, open_the_graves, choked_estuary,
    darkwater_catacombs, exotic_orchard, sunken_hollow, temple_of_deceit, bojuka_bog,
    command_tower, dimir_aqueduct, mortuary_mire, myriad_landscape, path_of_ancestry,
    tainted_isle, unclaimed_territory, eloise_nephalia_sleuth, cleaver_skaab,
    curse_of_unbinding, drown_in_dreams, empty_the_laboratory, hordewing_skaab, shadow_kin,
    crowded_crypt, curse_of_the_restless_dead, ghouls_night_out, gorex_the_tombshell,
    prowling_geistcatcher, ravenous_rotbelly, tomb_tyrant,
    // Basics: 12 island, 15 swamp
    island, island, island, island, island, island, island, island, island, island, island,
    island, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp,
];

pub const BRIGHT_PALM_COMMANDERS: &[CardFactory] = &[bright_palm_soul_awakener];

/// **Call for Backup**, the March of the Machine Commander deck (MOC,
/// 2023-04-21), exactly as MTGJSON's `CallForBackup_MOC` prints it: 86
/// nonbasic cards + 5 Plains + 2 Mountains + 6 Forests = 99. Naya +1/+1
/// counters and backup under Bright-Palm, Soul Awakener.
pub const BRIGHT_PALM_MAIN: &[CardFactory] = &[
    shalai_and_hallar, death_greeters_champion, uncivil_unrest, ichor_elixir,
    mirror_style_master, guardian_scalelord, emergent_woodwurm, path_of_the_pyromancer,
    conclave_sledge_captain, kalonian_hydra, mikaeus_the_lunarch, canopy_vista,
    champion_of_lambholt, cinder_glade, dromokas_command, exotic_orchard, flamerush_rider,
    flameshadow_conjuring, forgotten_ancient, fortified_village, furycalm_snarl, game_trail,
    gavony_township, genesis_hydra, gyre_sage, heaven_earth, high_sentinels_of_arashin,
    incubation_druid, inscription_of_abundance, ion_storm, kessig_wolf_run,
    krenko_tin_street_kingpin, managorger_hydra, mossfire_valley, mosswort_bridge,
    restoration_angel, rishkar_peema_renegade, semesters_end, strionic_resonator,
    sungrass_prairie, sunscorch_regent, temple_of_abandon, temple_of_plenty, temple_of_triumph,
    together_forever, triskelion, command_tower, commanders_sphere, cultivate, evolving_wilds,
    fertilid, fractured_powerstone, kodamas_reach, path_of_ancestry, pridemalkin,
    return_to_nature, terramorphic_expanse, wood_elves, abzan_battle_priest, abzan_falconer,
    alharu_solemn_ritualist, arcane_signet, armorcraft_judge, brawn, bretagard_stronghold,
    conclave_mentor, constable_of_the_realm, elite_scaleguard, enduring_scalelord,
    falkenrath_exterminator, field_of_ruin, generous_gift, good_fortune_unicorn,
    hamza_guardian_of_arashin, hindervines, inspiring_call, jungle_shrine, juniper_order_ranger,
    krosan_verge, llanowar_reborn, mindless_automaton, rogues_passage, slurrk_all_ingesting,
    sol_ring, swords_to_plowshares, temple_of_the_false_god,
    // Basics: 5 plains, 2 mountain, 6 forest
    plains, plains, plains, plains, plains, mountain, mountain,
    forest, forest, forest, forest, forest, forest,
];

pub const HAZEL_COMMANDERS: &[CardFactory] = &[hazel_of_the_rootbloom];

/// **Squirreled Away**, the Bloomburrow Commander deck (BLC, 2024-08-02),
/// exactly as MTGJSON's `SquirreledAway_BLC` prints it: 82 nonbasic
/// cards + 8 Swampes + 9 Forests = 99. Golgari Squirrels, Food and tokens under Hazel of the
/// Rootbloom.
pub const HAZEL_MAIN: &[CardFactory] = &[
    the_odd_acorn_gang, garruk_cursed_huntsman, chittering_witch, insatiable_frugivore,
    moonstone_eulogist, swarmyard_massacre, hazels_brewmaster, woe_strider, saw_in_half,
    ogre_slumlord, decree_of_pain, gourmands_talent, rootcast_apprenticeship, scurry_of_squirrels,
    end_raze_forerunners, arasta_of_the_endless_web, deep_forest_hermit, toski_bearer_of_secrets,
    beastmaster_ascension, second_harvest, shamanic_revelation, chatterfang_squirrel_general,
    temple_of_malady, casualties_of_war, windgraces_judgment, maskwood_nexus, academy_manufactor,
    woodland_cemetery, necroblossom_snarl, oran_rief_the_vastwood, swarmyard, exotic_orchard,
    llanowar_wastes, grim_backwoods, viridescent_bog, twilight_mire, gilded_goose, chitterspitter,
    maelstrom_pulse, beledros_witherbloom, idol_of_oblivion, sword_of_the_squeak,
    morbid_opportunist, nadiers_nightblade, plumb_the_forbidden, bastion_of_remembrance,
    plaguecrafter, cache_grab, chatterstorm, poison_tip_archer, moldervine_reclamation,
    ravenous_squirrel, skyfisher_spider, binding_the_old_gods, golgari_rot_farm, jungle_hollow,
    haunted_mire, nested_shambler, deadly_dispute, zulaport_cutthroat, squirrel_sovereign,
    prosperous_innkeeper, haywire_mite, tireless_provisioner, squirrel_nest, honored_dreyleader,
    tear_asunder, wolfwillow_haven, putrefy, arcane_signet, golgari_signet, talisman_of_resilience,
    sol_ring, skullclamp, terramorphic_expanse, path_of_ancestry, evolving_wilds, command_tower,
    tranquil_thicket, bojuka_bog, tainted_wood, barren_moor,
    // Basics: 8 swamp, 9 forest
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, forest, forest, forest, forest, forest,
    forest, forest, forest, forest,
];

pub const SAHEELI_COMMANDERS: &[CardFactory] = &[saheeli_the_gifted];

/// **Exquisite Invention**, the Commander 2018 deck (C18, 2018-08-10), exactly
/// as MTGJSON's `ExquisiteInvention_C18` prints it: 72 nonbasic cards + 15
/// Islands + 12 Mountains = 99. Izzet artifacts under Saheeli, the Gifted — a
/// planeswalker commander.
pub const SAHEELI_MAIN: &[CardFactory] = &[
    maverick_thopterist, soul_of_new_phyrexia, inkwell_leviathan, sharding_sphinx,
    hellkite_igniter, bosh_iron_golem, darksteel_juggernaut, duplicant, myr_battlesphere,
    psychosis_crawler, scuttling_doom_engine, steel_hellkite, thopter_assembly,
    etherium_sculptor, whirler_rogue, chief_of_the_foundry, pilgrims_eye, saheelis_artistry,
    blasphemous_act, reverse_engineer, tidings, chaos_warp, magmaquake, into_the_roil,
    thirst_for_knowledge, blinkmoth_urn, mimic_vat, mirrorworks, prototype_portal,
    unwinding_clock, commanders_sphere, dreamstone_hedron, hedron_archive, izzet_signet,
    magnifying_glass, mind_stone, prismatic_lens, scrabbling_claws, sol_ring, swiftfoot_boots,
    unstable_obelisk, vessel_of_endless_rest, worn_powerstone, darksteel_citadel, great_furnace,
    seat_of_the_synod, thopter_spy_network, buried_ruin, command_tower, foundry_of_the_consuls,
    highland_lake, izzet_boilerworks, izzet_guildgate, swiftwater_cliffs,
    brudiclad_telchor_engineer, tawnos_urzas_apprentice, echo_storm, vedalken_humiliator,
    enchanters_bane, saheelis_directive, treasure_nabber, varchild_betrayer_of_kjeldor,
    ancient_stone_idol, coveted_jewel, endless_atlas, retrofitter_foundry, aether_gale,
    loyal_drake, loyal_apprentice, geode_golem, thopter_engineer, forge_of_heroes,
    // Basics: 15 island, 12 mountain
    island, island, island, island, island, island, island, island, island, island, island,
    island, island, island, island, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain,
];

pub const WINTER_COMMANDERS: &[CardFactory] = &[winter_cynical_opportunist];

/// **Death Toll**, the Duskmourn Commander Golgari deck (DSC, 2024-09-27),
/// exactly as MTGJSON's `DeathToll_DSC` prints it: 85 nonbasic cards +
/// 7 Swamps + 7 Forests = 99. Self-mill and delirium under Winter, Cynical
/// Opportunist.
pub const WINTER_MAIN: &[CardFactory] = &[
    rendmaw_creaking_nest, deluge_of_doom, demonic_covenant, into_the_pit, polluted_cistern_dim_oubliette,
    demolisher_spawn, formless_genesis, ursine_monstrosity, convert_to_slime,
    moldgraf_monstrosity, culling_ritual, cemetery_tampering, noxious_gearhulk,
    ob_nixilis_reignited, professor_onyx, reanimate, whip_of_erebos, arachnogenesis,
    deathcap_cultivator, giant_adephage, hornet_queen, inscription_of_abundance,
    ishkanah_grafwidow, scavenging_ooze, titania_natures_force, wrenn_and_seven,
    deadbridge_chant, grim_flayer, grist_the_hunger_tide, old_stickfingers, solemn_simulacrum,
    dryad_arbor, exotic_orchard, grim_backwoods, llanowar_wastes, necroblossom_snarl,
    temple_of_malady, twilight_mire, viridescent_bog, woodland_cemetery, vile_mutilator,
    terramorphic_expanse, nights_whisper, grapple_with_the_past, deathreap_ritual, putrefy,
    arcane_signet, sol_ring, suspicious_bookcase, command_tower, carrion_grub,
    stitchers_supplier, crawling_sensation, gnarlwood_dryad, harmonize, harrow,
    moldgraf_millipede, mulch, obsessive_skinner, rampant_growth, sakura_tribe_elder,
    skola_grovedancer, binding_the_old_gods, grisly_salvage, nyx_weaver, burnished_hart,
    commanders_sphere, golgari_signet, haywire_mite, mind_stone, talisman_of_resilience,
    whispersilk_cloak, ash_barrens, barren_moor, bojuka_bog, darkmoss_bridge, evolving_wilds,
    golgari_rot_farm, jungle_hollow, reliquary_tower, tainted_wood, temple_of_the_false_god,
    tranquil_thicket, tree_of_tales, vault_of_whispers,
    // Basics: 7 swamp, 7 forest
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, forest, forest, forest, forest, forest,
    forest, forest,
];

pub const WILLOWDUSK_COMMANDERS: &[CardFactory] = &[willowdusk_essence_seer];

/// **Witherbloom Witchcraft**, the Commander 2021 Strixhaven deck (C21,
/// 2021-04-23), exactly as MTGJSON's `WitherbloomWitchcraft_C21` prints it:
/// 77 nonbasic cards + 11 Swamps + 11 Forests = 99. Golgari life gain and
/// life loss under Willowdusk, Essence Seer.
pub const WILLOWDUSK_MAIN: &[CardFactory] = &[
    ob_nixilis_reignited, gyome_master_chef, marshland_bloodcaster, tivash_gloom_summoner,
    veinwitch_coven, blossoming_bogbeast, ezzaroot_channeler, sproutback_trudge,
    yedora_grave_gardener, bloodtracker, defiant_bloodlord, noxious_gearhulk, sangromancer,
    ageless_entity, gluttonous_troll, sapling_of_colfenor, honor_troll, dina_soul_steeper,
    bloodthirsty_aerialist, epicure_of_blood, silversmote_ghoul, vampire_nighthawk,
    leyline_prowler, essence_pulse, healing_technique, pest_infestation, revival_experiment,
    damnable_pact, deadly_tempest, taste_of_death, gaze_of_granite, ancient_craving,
    feed_the_swarm, cultivate, rampant_growth, mortality_spear, reckless_spite, suffer_the_past,
    pulse_of_murasa, druidic_satchel, loxodon_warhammer, well_of_lost_dreams, arcane_signet,
    elixir_of_immortality, paradise_plume, pristine_talisman, sol_ring, sun_droplet,
    talisman_of_resilience, blight_mound, trudge_garden, sanguine_bond, greed, gift_of_paradise,
    moldervine_reclamation, exotic_orchard, high_market, llanowar_wastes, temple_of_malady,
    witherbloom_campus, study_hall, blighted_woodland, command_tower, gingerbread_cabin,
    golgari_rot_farm, jungle_hollow, myriad_landscape, radiant_fountain, sapseep_forest,
    tainted_wood, temple_of_the_false_god, witchs_clinic, nissas_renewal, verdant_suns_avatar,
    alhammarrets_archive, vensers_journal, rogues_passage,
    // Basics: 11 swamp, 11 forest
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, forest, forest,
    forest, forest, forest, forest, forest, forest, forest, forest, forest,
];

pub const HEARTHHULL_COMMANDERS: &[CardFactory] = &[hearthhull_the_worldseed];

/// **World Shaper**, the Edge of Eternities Commander deck (EOC,
/// 2025-08-01), exactly as MTGJSON's `WorldShaper_EOC` prints it: 82
/// nonbasic cards + 1 Wastes + 5 Swamps + 3 Mountains + 8 Forests = 99. Jund
/// land sacrifice under Hearthhull, the Worldseed.
pub const HEARTHHULL_MAIN: &[CardFactory] = &[
    szarel_genesis_shepherd, eumidian_wastewaker, evendo_brushrazer, planetary_annihilation,
    baloth_prime, exploration_broodship, horizon_explorer, scouring_swarm, eumidian_hatchery,
    festering_thicket, vernal_fen, fabled_passage, braids_arisen_nightmare, god_eternal_bontu,
    blasphemous_act, hammer_of_purphoros, moraug_fury_of_akoum, augur_of_autumn,
    centaur_vinecrasher, formless_genesis, loamcrafter_faun, multani_yavimayas_avatar,
    oracle_of_mul_daya, pest_infestation, rampaging_baloths, splendid_reclamation,
    tireless_tracker, titania_protector_of_argoth, world_breaker, escape_to_the_wilds,
    gaze_of_granite, the_gitrog_monster, korvold_fae_cursed_king, mazirek_kraul_death_priest,
    omnath_locus_of_rage, soul_of_windgrace, windgraces_judgment, worldsouls_rage,
    canyon_slough, cinder_glade, karplusan_forest, llanowar_wastes, sheltered_thicket,
    smoldering_marsh, sulfurous_springs, twilight_mire, viridescent_bog, farseek,
    springbloom_druid, binding_the_old_gods, arcane_signet, sol_ring, command_tower,
    mountain_valley, terramorphic_expanse, infernal_grasp, nights_whisper, sprouting_goblin,
    aftermath_analyst, beast_within, cultivate, groundskeeper, harrow, natures_lore,
    roiling_regrowth, satyr_wayfinder, skyshroud_claim, tear_asunder, juri_master_of_the_revue,
    mayhem_devil, putrefy, rakdos_charm, uurg_spawn_of_turg, bojuka_bog, cabaretti_courtyard,
    dakmor_salvage, escape_tunnel, evolving_wilds, maestros_theater, myriad_landscape,
    riveteers_overlook, rocky_tar_pit,
    // Basics: 1 wastes, 5 swamp, 3 mountain, 8 forest
    swamp, swamp, swamp, swamp, swamp, mountain, mountain, mountain, forest, forest, forest,
    forest, forest, forest, forest, forest, wastes,
];

pub const WINDGRACE_COMMANDERS: &[CardFactory] = &[lord_windgrace];

/// **Nature's Vengeance**, the Commander 2018 deck (C18, 2018-08-10), exactly
/// as MTGJSON's `NatureSVengeance_C18` prints it: 81 nonbasic cards +
/// 6 Swamps + 5 Mountains + 7 Forests = 99. Jund lands-matter under
/// the planeswalker commander Lord Windgrace.
pub const WINDGRACE_MAIN: &[CardFactory] = &[
    charnelhoard_wurm, avenger_of_zendikar, soul_of_innistrad, flameblast_dragon,
    budoka_gardener, centaur_vinecrasher, moldgraf_monstrosity, rampaging_baloths, scute_mob,
    rubblehulk, acidic_slime, baloth_woodcrasher, borderland_explorer, farhaven_elf,
    sakura_tribe_elder, yavimaya_elder, zendikar_incarnate, ruinous_path, chain_reaction,
    decimate, gaze_of_granite, lavalanche, worm_harvest, stitch_together, cultivate, explore,
    explosive_vegetation, far_wanderings, hunting_wilds, savage_twister, moonlight_bargain,
    consign_to_dust, grapple_with_the_past, harrow, grisly_salvage, putrefy, seers_sundial,
    sol_ring, retreat_to_hagra, khalni_heart_expedition, deathreap_ritual, akoum_refuge,
    barren_moor, blighted_woodland, bojuka_bog, command_tower, evolving_wilds, forgotten_cave,
    golgari_rot_farm, grim_backwoods, gruul_turf, haunted_fengraf, jund_panorama, jungle_hollow,
    kazandu_refuge, khalni_garden, mountain_valley, myriad_landscape, rakdos_carnarium,
    rocky_tar_pit, savage_lands, temple_of_the_false_god, terramorphic_expanse,
    tranquil_thicket, warped_landscape, gyrus_waker_of_corpses, thantis_the_warweaver,
    bloodtracker, emissary_of_grudges, fury_storm, nesting_dragon, reality_scramble,
    crash_of_rhino_beetles, turntimber_sower, whiptongue_hydra, windgraces_judgment,
    xantcha_sleeper_agent, loyal_subordinate, loyal_apprentice, loyal_guardian, forge_of_heroes,
    // Basics: 6 swamp, 5 mountain, 7 forest
    swamp, swamp, swamp, swamp, swamp, swamp, mountain, mountain, mountain, mountain, mountain,
    forest, forest, forest, forest, forest, forest, forest,
];

pub const DEREVI_COMMANDERS: &[CardFactory] = &[derevi_empyrial_tactician];

/// **Evasive Maneuvers**, the Commander 2013 deck (C13, 2013-11-01), exactly
/// as MTGJSON's `EvasiveManeuvers_C13` prints it: 79 nonbasic cards +
/// 7 Plains + 7 Islands + 6 Forests = 99. Bant tempo, curses and
/// tap/untap tricks under Derevi, Empyrial Tactician.
pub const DEREVI_MAIN: &[CardFactory] = &[
    acidic_slime, aerie_mystics, angel_of_finality, azami_lady_of_scrolls, bane_of_progress,
    deceiver_exarch, diviner_spirit, djinn_of_infinite_deceits, dungeon_geists, farhaven_elf,
    fiend_hunter, flickerwisp, hada_spy_patrol, karmic_guide, kazandu_tuskcaller,
    lu_xun_scholar_general, mirror_entity, mistmeadow_witch, murkfiend_liege, phantom_nantuko,
    pilgrims_eye, roon_of_the_hidden_realm, rubinia_soulsinger, selesnya_guildmage,
    skyward_eye_prophets, stonecloaker, thornwind_faeries, winged_coatl, wonder,
    borrowing_100_000_arrows, kirtars_wrath, restore, tempt_with_glory, wash_out, aethermages_touch,
    arcane_denial, blue_suns_zenith, krosan_grip, selesnya_charm, unexpectedly_absent,
    azorius_keyrune, basalt_monolith, conjurers_closet, darksteel_ingot, leonin_bladetrap,
    selesnya_signet, simic_signet, sol_ring, surveyors_scope, swiftfoot_boots, sword_of_the_paruns,
    thousand_year_elixir, thunderstaff, control_magic, curse_of_inertia, curse_of_predation,
    curse_of_the_forsaken, darksteel_mutation, flickerform, leafdrake_roost, presence_of_gond,
    azorius_chancery, azorius_guildgate, bant_panorama, command_tower, evolving_wilds,
    faerie_conclave, opal_palace, rupture_spire, saltcrusted_steppe, seaside_citadel,
    secluded_steppe, sejiri_refuge, selesnya_guildgate, selesnya_sanctuary, simic_guildgate,
    temple_of_the_false_god, terramorphic_expanse, transguild_promenade,
    // Basics: 7 plains, 7 island, 6 forest
    forest, forest, forest, forest, forest, forest, island, island, island, island, island, island,
    island, plains, plains, plains, plains, plains, plains, plains,
];

pub const BUMBLEFLOWER_COMMANDERS: &[CardFactory] = &[ms_bumbleflower];

/// **Peace Offering**, the Bloomburrow Commander deck (BLC, 2024-08-02),
/// exactly as MTGJSON's `PeaceOffering_BLC` prints it: 87 nonbasic
/// cards + 4 Plains + 4 Islands + 4 Forests = 99. Bant group hug and politics
/// under Ms. Bumbleflower.
pub const BUMBLEFLOWER_MAIN: &[CardFactory] = &[
    mr_foxglove, tamiyo_field_researcher, tenuous_truce, loran_of_the_third_path,
    steelburr_champion, tempt_with_bunnies, perch_protection, sunscorch_regent,
    promise_of_loyalty, hoofprints_of_the_stag, mangara_the_diplomat, realm_cloaked_giant,
    sphinx_of_enlightenment, forgotten_ancient, bloodroot_apothecary, communal_brewing,
    managorger_hydra, rishkar_peema_renegade, rites_of_flourishing, tempt_with_discovery,
    kalonian_hydra, faeburrow_elder, exotic_orchard, simic_ascendancy, ghirapur_orrery,
    psychosis_crawler, adarkar_wastes, temple_of_enlightenment, seachrome_coast,
    glacial_fortress, hinterland_harbor, razorverge_thicket, flooded_grove, skycloud_expanse,
    canopy_vista, prairie_stream, brushland, temple_of_mystery, yavimaya_coast,
    overflowing_basin, sungrass_prairie, sunpetal_grove, triskaidekaphile, octomancer,
    twenty_toed_toad, intellectual_offering, illusionists_gambit, body_of_knowledge,
    chasm_skulker, perplexing_test, jolrael_mwonvuli_recluse, fishers_talent,
    selvala_explorer_returned, kwain_itinerant_meddler, coveted_jewel, temple_of_plenty,
    generous_gift, swords_to_plowshares, secret_rendezvous, baird_steward_of_argive,
    an_offer_you_cant_refuse, wizard_class, cultivate, farseek, broken_wings, spore_frog,
    wear_down, peerless_recycling, coiling_oracle, riot_control, martial_impetus, jolly_gerbils,
    long_rivers_pull, thought_vessel, arcane_signet, swiftfoot_boots, fellwar_stone, sol_ring,
    mind_stone, thriving_heath, thriving_isle, thriving_grove, terramorphic_expanse,
    command_tower, evolving_wilds, reliquary_tower, seaside_citadel,
    // Basics: 4 plains, 4 island, 4 forest
    plains, plains, plains, plains, island, island, island, island, forest, forest, forest,
    forest,
];

pub const MISHRA_COMMANDERS: &[CardFactory] = &[mishra_eminent_one];

/// **Mishra's Burnished Banner**, the Brothers' War Commander Grixis deck
/// (BRC, 2022-11-18), exactly as MTGJSON's `MishraSBurnishedBanner_BRC`
/// prints it: 86 nonbasic cards + 4 Swamps + 5 Islands + 4 Mountains = 99.
/// Grixis artifacts under Mishra, Eminent One.
pub const MISHRA_MAIN: &[CardFactory] = &[
    muzzio_visionary_architect, geth_lord_of_the_vault, herald_of_anguish,
    jhoira_weatherlight_captain, silas_renn_seeker_adept, emry_lurker_of_the_loch,
    master_transmuter, padeem_consul_of_innovation, workshop_elders, fain_the_broker,
    audacious_reshapers, hellkite_igniter, slobad_goblin_tinkerer, brudiclad_telchor_engineer,
    metalwork_colossus, traxos_scourge_of_kroog, blasphemous_act, expressive_iteration,
    thoughtcast, feed_the_swarm, faithless_looting, chaos_warp, bedevil, fact_or_fiction,
    thirst_for_knowledge, abrade, lithoform_engine, cursed_mirror, idol_of_oblivion,
    mirrorworks, oblivion_stone, spine_of_ish_sah, strionic_resonator, trading_post,
    oni_cult_anvil, dreamstone_hedron, fellwar_stone, hedron_archive, ichor_wellspring,
    mind_stone, servo_schematic, sol_ring, thran_dynamo, mnemonic_sphere, executioners_capsule,
    arcane_signet, commanders_sphere, dimir_signet, mycosynth_wellspring, nihil_spellbomb,
    prophetic_prism, rakdos_signet, wayfarers_bauble, darkwater_catacombs, exotic_orchard,
    shadowblood_ridge, smoldering_marsh, temple_of_deceit, temple_of_epiphany, temple_of_malice,
    buried_ruin, crumbling_necropolis, dimir_aqueduct, izzet_boilerworks, myriad_landscape,
    rakdos_carnarium, reliquary_tower, ash_barrens, command_tower, drossforge_bridge,
    great_furnace, mistvault_bridge, path_of_ancestry, seat_of_the_synod, silverbluff_bridge,
    terramorphic_expanse, vault_of_whispers, ashnod_the_uncaring, glint_raker,
    terisiares_devastation, blast_furnace_hellkite, farid_enterprising_salvager,
    machine_gods_effigy, scavenged_brawler, smelting_vat, wondrous_crucible,
    // Basics: 4 swamp, 5 island, 4 mountain
    swamp, swamp, swamp, swamp, island, island, island, island, island, mountain, mountain,
    mountain, mountain,
];

pub const GIMBAL_COMMANDERS: &[CardFactory] = &[gimbal_gremlin_prodigy];

/// **Tinker Time**, the March of the Machine Commander Temur deck (MOC,
/// 2023-04-21), exactly as MTGJSON's `TinkerTime_MOC` prints it: 76
/// nonbasic cards + 8 Islands + 8 Mountains + 7 Forests = 99. Artifact tokens under Gimbal, Gremlin
/// Prodigy.
pub const GIMBAL_MAIN: &[CardFactory] = &[
    rashmi_and_ragavan, schema_thief, sandsteppe_war_riders, cutthroat_negotiator,
    hedron_detonator, ichor_elixir, path_of_the_animist, pain_distributor, dance_with_calamity,
    pia_and_kiran_nalaar, feldon_of_the_third_path, chaos_warp, stroke_of_genius,
    academy_manufactor, aid_from_the_cowl, bloodforged_battle_axe, brasss_bounty, cinder_glade,
    echo_storm, everquill_phoenix, exotic_orchard, fiery_confluence, frostboil_snarl,
    game_trail, gilded_goose, hellkite_igniter, imprisoned_in_the_moon, inspiring_statuary,
    master_of_etherium, masterful_replication, perplexing_test, rise_and_shine,
    saheelis_artistry, sharding_sphinx, shimmer_dragon, skyclave_relic, spell_swindle,
    spine_of_ish_sah, temple_of_abandon, temple_of_epiphany, temple_of_mystery,
    thopter_assembly, thopter_spy_network, tireless_tracker, vedalken_humiliator,
    vineglimmer_snarl, workshop_elders, command_tower, crack_open, evolving_wilds,
    fractured_powerstone, gruul_signet, izzet_signet, path_of_ancestry, reverse_engineer,
    root_out, simic_growth_chamber, simic_signet, terramorphic_expanse, thoughtcast,
    replicating_ring, arcane_signet, combine_chrysalis, curse_of_opulence, frontier_bivouac,
    ghirapur_aether_grid, junk_winder, myriad_landscape, reality_shift,
    saheeli_sublime_artificer, sol_ring, struggle_survive, tireless_provisioner, vampires_vengeance,
    weirding_wood, whirler_rogue,
    // Basics: 8 island, 8 mountain, 7 forest
    island, island, island, island, island, island, island, island, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain, forest, forest, forest, forest,
    forest, forest, forest,
];

pub const DIHADA_COMMANDERS: &[CardFactory] = &[dihada_binder_of_wills];

/// **Legends' Legacy**, the Dominaria United Commander Mardu deck (DMC,
/// 2022-09-09), exactly as MTGJSON's `LegendsLegacy_DMC` prints it: 83
/// nonbasic cards + 6 Plainss + 5 Swamps + 5 Mountains = 99. Legends under Dihada, Binder of Wills, a
/// planeswalker commander.
pub const DIHADA_MAIN: &[CardFactory] = &[
    adriana_captain_of_the_guard, alesha_who_smiles_at_death, anafenza_kin_tree_spirit,
    arvad_the_cursed, ashling_the_pilgrim, bell_borca_spectral_sergeant, captain_lannery_storm,
    kothophed_soul_hoarder, drana_liberator_of_malakir, etali_primal_storm,
    garna_the_bloodflame, jazal_goldmane, josu_vess_lich_knight, kari_zev_skyship_raider,
    krenko_tin_street_kingpin, neheb_dreadhorde_champion, odric_lunarch_marshal,
    tajic_blade_of_the_legion, teshar_ancestors_apostle, traxos_scourge_of_kroog,
    zetalpa_primal_dawn, primevals_glorious_rebirth, urzas_ruinous_blast, faithless_looting,
    kayas_wrath, nights_whisper, read_the_bones, ambitions_cost, bedevil, generous_gift,
    heros_downfall, mortify, thrill_of_possibility, unbreakable_formation, wear_tear,
    blackblade_reforged, bontus_monument, commanders_sphere, arcane_signet, fellwar_stone,
    hazorets_monument, hedron_archive, heros_blade, heroes_podium, honor_worn_shaku,
    oketras_monument, sol_ring, sword_of_the_chosen, tenza_godos_maul, the_circle_of_loyalty,
    day_of_destiny, battlefield_forge, bojuka_bog, boros_garrison, command_tower,
    dragonskull_summit, evolving_wilds, foreboding_ruins, geier_reach_sanitarium,
    mikokoro_center_of_the_sea, mobilized_district, nomad_outpost, orzhov_basilica,
    rakdos_carnarium, reliquary_tower, shivan_gorge, shizo_deaths_storehouse, smoldering_marsh,
    temple_of_malice, temple_of_silence, temple_of_triumph, terramorphic_expanse,
    tyrite_sanctum, caves_of_koilos, shanid_sleepers_scourge, zeriam_golden_wind,
    bladewing_deathless_tyrant, cadric_soul_kindler, gerrards_hourglass_pendant,
    moira_urborg_haunt, the_peregrine_dynamo, verrak_warped_sengir, the_reaver_cleaver,
    // Basics: 6 plains, 5 swamp, 5 mountain
    plains, plains, plains, plains, plains, plains, swamp, swamp, swamp, swamp, swamp, mountain,
    mountain, mountain, mountain, mountain,
];

pub const DINA_COMMANDERS: &[CardFactory] = &[dina_essence_brewer];

/// **Witherbloom Pestilence**, the Secrets of Strixhaven Commander deck (SOC,
/// 2026-04-24), exactly as MTGJSON's `WitherbloomPestilence_SOC` prints it:
/// 83 nonbasic cards + 8 Swamps + 8 Forests = 99. Golgari Pests and
/// sacrifice under Dina, Essence Brewer.
pub const DINA_MAIN: &[CardFactory] = &[
    gorma_the_gullet, merchant_of_venom, defiling_daemogoth, ominous_harvest,
    stensian_sanguinist, feral_appetite, pest_rescuer, ribtruss_roaster, eccentric_pestfinder,
    immoral_bargain, turbulent_fen, ophiomancer, toxic_deluge, tendershoot_dryad,
    fabled_passage, blight_mound, bloodghast, final_act, jadar_ghoulcaller_of_nephalia,
    nether_traitor, priest_of_forgotten_gods, smothering_abomination, veinwitch_coven,
    witch_of_the_moors, woe_strider, yahenni_undying_partisan, awakening_zone,
    blossoming_bogbeast, gilded_goose, mycoloth, ohran_frostfang, pest_infestation,
    trudge_garden, assassins_trophy, beledros_witherbloom, casualties_of_war, creakwood_liege,
    culling_ritual, gyome_master_chef, mazirek_kraul_death_priest, wight_of_the_reliquary,
    witherbloom_command, exotic_orchard, festering_thicket, grim_backwoods, high_market,
    llanowar_wastes, necroblossom_snarl, temple_of_malady, twilight_mire, vernal_fen,
    viridescent_bog, woodland_cemetery, arcane_signet, sol_ring, command_tower, teachers_pest,
    witherbloom_charm, terramorphic_expanse, titans_grave, blood_artist, infernal_grasp,
    morbid_opportunist, nights_whisper, pawn_of_ulamog, plumb_the_forbidden,
    umbral_collar_zealot, viscera_seer, zulaport_cutthroat, cultivate, elvish_mystic,
    sakura_tribe_elder, springbloom_druid, deadly_brew, dina_soul_steeper,
    moldervine_reclamation, mortality_spear, haywire_mite, bojuka_bog, haunted_mire,
    path_of_ancestry, study_hall, witherbloom_campus,
    // Basics: 8 swamp, 8 forest
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, forest, forest, forest, forest,
    forest, forest, forest, forest,
];

pub const ATARKA_COMMANDERS: &[CardFactory] = &[atarka_world_render];

/// **Draconic Destruction**, a Starter Commander Deck (SCD, 2022-12-02), exactly
/// as MTGJSON's `DraconicDestruction_SCD` prints it: 69 nonbasic cards +
/// 18 Mountains + 12 Forests = 99. Gruul Dragons under Atarka, World
/// Render.
pub const ATARKA_MAIN: &[CardFactory] = &[
    akoum_hellkite, chain_reaction, crucible_of_fire, demanding_dragon, dragonkin_berserker,
    dragonmaster_outcast, drakuseth_maw_of_flames, dream_pillager, flameblast_dragon,
    hoard_smelter_dragon, magmaquake, mordant_dragon, runehorn_hellkite, sarkhan_the_dragonspeaker,
    scourge_of_valkas, spit_flame, sweltering_suns, thunderbreak_regent, thundermaw_hellkite,
    tyrants_familiar, verix_bladewing, foe_razer_regent, frontier_siege, hunters_prowess,
    primal_might, shamanic_revelation, clan_defiance, harbinger_of_the_hunt, dragons_hoard,
    steel_hellkite, cinder_glade, game_trail, haven_of_the_spirit_dragon, temple_of_abandon,
    dragon_mage, dragon_tempest, dragonlords_servant, dragonspeaker_shaman, furnace_whelp,
    provoke_the_trolls, rapacious_dragon, unleash_fury, vandalblast, beast_within,
    blossoming_defense, cultivate, drumhunter, elemental_bond, garruks_uprising, harmonize,
    hunters_insight, loaming_shaman, return_to_nature, sakura_tribe_elder, draconic_disciple,
    fires_of_yavimaya, savage_ventmaw, arcane_signet, atarka_monument, commanders_sphere, sol_ring,
    swiftfoot_boots, talisman_of_impulse, command_tower, kazandu_refuge, path_of_ancestry,
    rugged_highlands, shivan_oasis, timber_gorge,
    // Basics: 12 forest, 18 mountain
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
];

pub const MARATH_COMMANDERS: &[CardFactory] = &[marath_will_of_the_wild];

/// **Nature of the Beast**, the Commander 2013 deck (C13, 2013-11-01),
/// exactly as MTGJSON's `NatureOfTheBeast_C13` prints it: 82 nonbasic
/// cards + 4 Plains + 5 Mountains + 8 Forests = 99. Naya big creatures under
/// Marath, Will of the Wild.
pub const MARATH_MAIN: &[CardFactory] = &[
    archangel, avenger_of_zendikar, baloth_woodcrasher, crater_hellion, deadwood_treefolk,
    drumhunter, eternal_dragon, gahiji_honored_one, grazing_gladehart, krosan_tusker,
    krosan_warchief, magus_of_the_arena, mayael_the_anima, mold_shambler, naya_soulbeast,
    rakeclaw_gargantuan, rampaging_baloths, ravenous_baloth, spellbreaker_behemoth,
    spitebellows, terra_ravager, valley_rannet, cultivate, fiery_justice, fireball,
    from_the_ashes, harmonize, hull_breach, one_dozen_eyes, rain_of_thorns, restore,
    savage_twister, slice_and_dice, tempt_with_discovery, wrath_of_god, boros_charm, naya_charm,
    slice_in_twain, sprouting_vines, street_spasm, behemoth_sledge, druidic_satchel,
    seers_sundial, sol_ring, swiftfoot_boots, tower_of_fortunes, curse_of_chaos,
    curse_of_predation, curse_of_the_forsaken, darksteel_mutation, fires_of_yavimaya,
    mystic_barrier, spawning_grounds, war_cadence, warstorm_surge, where_ancients_tread,
    witch_hunt, boros_garrison, boros_guildgate, command_tower, contested_cliffs,
    drifting_meadow, evolving_wilds, forgotten_cave, gruul_guildgate, homeward_path,
    jungle_shrine, khalni_garden, mosswort_bridge, naya_panorama, new_benalia, opal_palace,
    rupture_spire, secluded_steppe, selesnya_guildgate, selesnya_sanctuary, slippery_karst,
    smoldering_crater, temple_of_the_false_god, tranquil_thicket, vitu_ghazi_the_city_tree,
    vivid_crag,
    // Basics: 4 plains, 5 mountain, 8 forest
    plains, plains, plains, plains, mountain, mountain, mountain, mountain, mountain, forest,
    forest, forest, forest, forest, forest, forest, forest,
];

pub const JELEVA_COMMANDERS: &[CardFactory] = &[jeleva_nephalias_scourge];

/// **Mind Seize**, the Commander (2013) Grixis deck (C13, 2013-11-01),
/// exactly as MTGJSON's `MindSeize_C13` prints it: 77 nonbasic cards +
/// 9 Islands + 8 Swamps + 5 Mountains = 99. Spells, theft and wheels
/// under Jeleva, Nephalia's Scourge.
pub const JELEVA_MAIN: &[CardFactory] = &[
    augur_of_bolas, baleful_force, baleful_strix, charmbreaker_devils, diviner_spirit,
    echo_mage, fog_bank, guard_gomazoa, guttersnipe, hooded_horror, jaces_archivist,
    mnemonic_wall, nekusar_the_mindrazer, nightscape_familiar, nivix_guildmage, terra_ravager,
    thraximundar, true_name_nemesis, uyo_silent_prophet, vampire_nighthawk, viseling,
    army_of_the_damned, cruel_ultimatum, decree_of_pain, fissure_vent, incendiary_command,
    infest, molten_disaster, phthisis, prosperity, skyscribing, strategic_planning,
    tempt_with_reflections, annihilate, crosiss_charm, dismiss, grixis_charm,
    illusionists_gambit, opportunity, soul_manipulation, starstorm, sudden_spoiling,
    vision_skeins, wild_ricochet, armillary_sphere, eye_of_doom, mirari, obelisk_of_grixis,
    sol_ring, swiftfoot_boots, temple_bell, wayfarers_bauble, arcane_melee, curse_of_chaos,
    curse_of_inertia, curse_of_shallow_graves, price_of_knowledge, propaganda, spiteful_visions,
    akoum_refuge, bojuka_bog, command_tower, crumbling_necropolis, dimir_guildgate,
    evolving_wilds, grixis_panorama, izzet_boilerworks, izzet_guildgate, molten_slagheap,
    opal_palace, rakdos_carnarium, rakdos_guildgate, rupture_spire, temple_of_the_false_god,
    urzas_factory, vivid_creek, vivid_marsh,
    // Basics: 9 island, 8 swamp, 5 mountain
    island, island, island, island, island, island, island, island, island, mountain, mountain,
    mountain, mountain, mountain, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
];

pub const BREENA_COMMANDERS: &[CardFactory] = &[breena_the_demagogue];

/// **Silverquill Statement**, the Commander 2021 Strixhaven deck (C21,
/// 2021-04-23), exactly as MTGJSON's `SilverquillStatement_C21` prints it:
/// 75 nonbasic cards + 14 Plainss + 10 Swamps = 99. Orzhov politics and
/// Inklings under Breena, the Demagogue.
pub const BREENA_MAIN: &[CardFactory] = &[
    gideon_champion_of_justice, felisa_fang_of_silverquill, combat_calligrapher,
    guardian_archon, nils_discipline_enforcer, scholarship_sponsor, author_of_shadows,
    bold_plagiarist, fain_the_broker, keen_duelist, angel_of_serenity, boreas_charger,
    hunted_lammasu, knight_of_the_white_orchid, selfless_squire, stalking_leonin,
    sunscorch_regent, windborn_muse, zetalpa_primal_dawn, deathbringer_regent,
    necropolis_regent, deathbringer_liege, magister_of_worth, teysa_envoy_of_ghosts,
    elite_scaleguard, oreskos_explorer, orzhov_advokist, promise_of_loyalty,
    incarnation_technique, tragic_arrogance, infernal_offering, secret_rendezvous,
    stinging_study, inkshield, oblation, utter_end, fracture, tempting_contract, coveted_jewel,
    pendant_of_prosperity, victory_chimes, arcane_signet, bloodthirsty_blade, mind_stone,
    orzhov_signet, sol_ring, spectral_searchlight, cunning_rhetoric, citadel_siege,
    together_forever, ghostly_prison, martial_impetus, soul_snare, vow_of_duty,
    curse_of_disturbance, parasitic_impetus, caves_of_koilos, exotic_orchard,
    mikokoro_center_of_the_sea, temple_of_silence, silverquill_campus, study_hall, barren_moor,
    bojuka_bog, command_tower, myriad_landscape, opal_palace, orzhov_basilica, secluded_steppe,
    tainted_field, temple_of_the_false_god, duelists_heritage, debtors_knell, ambitions_cost,
    rogues_passage,
    // Basics: 14 plains, 10 swamp
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp,
];

pub const GUFF_COMMANDERS: &[CardFactory] = &[commodore_guff];

/// **Planeswalker Party**, the Commander Masters Jeskai deck (CMM,
/// 2023-08-04), exactly as MTGJSON's `PlaneswalkerParty_CMM` prints it: 81
/// nonbasic cards + 7 Plainss + 7 Islands + 4 Mountains = 99. Superfriends under Commodore Guff, a
/// planeswalker commander.
pub const GUFF_MAIN: &[CardFactory] = &[
    leori_sparktouched_hunter, mangara_the_diplomat, gatewatch_beacon, onakke_oathkeeper,
    teyo_geometric_tactician, sparkshaper_visionary, vronos_masked_inquisitor,
    chandra_legacy_of_fire, guff_rewrites_history, jayas_phoenix, ajani_steadfast,
    deploy_the_gatewatch, elspeth_suns_champion, gideon_jura, norns_annex, oath_of_gideon,
    promise_of_loyalty, semesters_end, urzas_ruinous_blast, deepglow_skate, jace_beleren,
    jace_architect_of_thought, jace_mirror_mage, oath_of_jace, spark_double, blasphemous_act,
    chandra_awakened_inferno, chandra_torch_of_defiance, repeated_reverberation,
    sarkhan_the_masterless, nahiri_the_harbinger, narset_of_the_ancient_way,
    narset_enlightened_master, oath_of_teferi, the_chain_veil, nevinyrrals_disk, silent_arbiter,
    cascade_bluffs, exotic_orchard, frostboil_snarl, furycalm_snarl, karns_bastion,
    mobilized_district, mystic_gate, port_town, prairie_stream, rugged_prairie,
    skycloud_expanse, temple_of_enlightenment, temple_of_epiphany, temple_of_triumph,
    cartographers_hawk, path_to_exile, kazuul_tyrant_of_the_cliffs, arcane_signet,
    fellwar_stone, sol_ring, command_tower, myriad_landscape, reliquary_tower,
    grateful_apparition, oreskos_explorer, swords_to_plowshares, the_wanderer, flux_channeler,
    fog_bank, narset_parter_of_veils, thrummingbird, saheeli_sublime_artificer, wall_of_denial,
    azorius_signet, boros_signet, honor_worn_shaku, izzet_signet, talisman_of_conviction,
    talisman_of_creativity, talisman_of_progress, wayfarers_bauble, forge_of_heroes,
    interplanar_beacon, mystic_monastery,
    // Basics: 7 plains, 7 island, 4 mountain
    plains, plains, plains, plains, plains, plains, plains, island, island, island, island,
    island, island, island, mountain, mountain, mountain, mountain,
];

pub const GOSHINTAI_COMMANDERS: &[CardFactory] = &[go_shintai_of_lifes_origin];

/// **20 Ways to Win**, the Secret Lair Commander deck (SLD, 2024-12-04),
/// exactly as MTGJSON's `20WaysToWin_SLD` prints it: 99 cards, no basics.
/// Alternate win conditions, Gates and Shrines under Go-Shintai of Life's
/// Origin (its WUBRG ability makes it five-colour).
pub const GOSHINTAI_MAIN: &[CardFactory] = &[
    alseid_of_lifes_bounty, approach_of_the_second_sun, arcane_signet, augur_of_autumn,
    auriok_champion, austere_command, azorius_guildgate, baldurs_gate, biovisionary,
    black_dragon_gate, boros_guildgate, bountiful_promenade, brushland, chromatic_lantern,
    circuitous_route, citadel_gate, clever_concealment, cliffgate, command_tower, crib_swap,
    darksteel_citadel, darksteel_mutation, dimir_guildgate, drown_in_dreams,
    dryad_of_the_ilysian_grove, eerie_ultimatum, exotic_orchard, faeburrow_elder,
    felidar_sovereign, fertile_ground, forbidden_orchard, forest, forgotten_ancient,
    gateway_plaza, ghostly_prison, ghoulish_impetus, golgari_guildgate, gond_gate,
    gruul_guildgate, halo_fountain, hangarback_walker, happily_ever_after, heap_gate,
    heliods_intervention, heliod_sun_crowned, helix_pinnacle, hellkite_tyrant, homeward_path,
    izzet_guildgate, kynaios_and_tiro_of_meletis, lightning_greaves, lignify, lilianas_contract,
    mangara_the_diplomat, manor_gate, maskwood_nexus, mayaels_aria, mazes_end,
    mechanized_production, monologue_tax, morbid_opportunist, mystic_remora, orzhov_guildgate,
    overgrown_farmland, pariah, path_of_ancestry, plains, plaza_of_harmony, rakdos_guildgate,
    razorverge_thicket, reflecting_pool, revel_in_riches, rite_of_replication, sanctum_weaver,
    sea_gate, seedborn_muse, selesnya_guildgate, shalai_voice_of_plenty, shiny_impetus,
    simic_ascendancy, simic_guildgate, sol_ring, souls_attendant, sphinxs_revelation, sun_titan,
    sunscorch_regent, supreme_verdict, taurean_mauler, test_of_endurance, the_eternal_wanderer,
    the_world_tree, thornglint_bridge, tireless_tracker, trace_of_abundance, tragic_arrogance,
    triskaidekaphile, twenty_toed_toad, wooded_bastion, yahenni_undying_partisan,
];

pub const NELLY_COMMANDERS: &[CardFactory] = &[nelly_borca_impulsive_accuser];

/// **Blame Game**, the Murders at Karlov Manor Commander deck (MKC,
/// 2024-02-09), exactly as MTGJSON's `BlameGame_MKC` prints it: 83 nonbasic
/// cards + 9 Plains + 7 Mountains = 99. Boros goad under Nelly Borca.
pub const NELLY_MAIN: &[CardFactory] = &[
    feather_radiant_arbiter, immortal_obligation, otherworldly_escort, redemption_arc,
    trouble_in_pairs, havoc_eater, hot_pursuit, mob_verdict, prisoners_dilemma, take_the_bait,
    ransom_note, angel_of_the_ruins, comeuppance, darien_king_of_kjeldor, duelists_heritage,
    elspeth_suns_champion, keeper_of_the_accord, loran_of_the_third_path, promise_of_loyalty,
    selfless_squire, sevinnes_reclamation, smugglers_share, stalking_leonin, sun_titan,
    windborn_muse, winds_of_rath, agitator_ant, brash_taunter, disrupt_decorum,
    etali_primal_storm, fiendish_duo, frontier_warmonger, kazuul_tyrant_of_the_cliffs,
    spectacular_showdown, vengeful_ancestor, anya_merciless_angel, boros_reckoner,
    deflecting_palm, gisela_blade_of_goldnight, ancient_stone_idol, solemn_simulacrum,
    steel_hellkite, tome_of_legends, castle_ardenvale, exotic_orchard, furycalm_snarl,
    kher_keep, labyrinth_of_skophos, needle_spires, scavenger_grounds, slayers_stronghold,
    temple_of_triumph, throne_of_the_high_city, war_room, ghostly_prison, gideons_sacrifice,
    martial_impetus, orzhov_advokist, seal_of_cleansing, soul_snare, vow_of_duty, wall_of_omens,
    curse_of_opulence, rite_of_the_raging_storm, shiny_impetus, vow_of_lightning, arcane_signet,
    bloodthirsty_blade, fellwar_stone, mind_stone, sol_ring, talisman_of_conviction,
    thought_vessel, access_tunnel, ash_barrens, boros_garrison, command_tower, escape_tunnel,
    myriad_landscape, reliquary_tower, rogues_passage, sunhome_fortress_of_the_legion,
    temple_of_the_false_god,
    // Basics: 9 plains, 7 mountain
    plains, plains, plains, plains, plains, plains, plains, plains, plains, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain,
];

pub const ZINNIA_COMMANDERS: &[CardFactory] = &[zinnia_valleys_voice];

/// **Family Matters**, the Bloomburrow Commander deck (BLC, 2024-08-02),
/// exactly as MTGJSON's `FamilyMatters_BLC` prints it: 86 nonbasic cards +
/// 5 Plains + 4 Islands + 4 Mountains = 99. Jeskai tokens and
/// offspring under Zinnia, Valley's Voice.
pub const ZINNIA_MAIN: &[CardFactory] = &[
    arthur_marigold_knight, elspeth_suns_champion, jazal_goldmane, martial_coup, storm_of_souls,
    selfless_spirit, murmuration, blade_splicer, hanged_executioner, loyal_warhound,
    restoration_angel, jacked_rabbit, skyclave_apparition, dusk_dawn, bosss_chauffeur,
    angel_of_the_ruins, luminous_broodmoth, sun_titan, pollywog_prodigy, fortune_tellers_talent,
    aether_channeler, pull_from_tomorrow, shield_broker, stolen_by_the_fae, rapid_augmenter,
    bident_of_thassa, curiosity_crafter, devilish_valet, siege_gang_commander, echoing_assault,
    agate_instigator, calamity_of_cinders, rose_room_treasurer, combat_celebrant, inferno_titan,
    time_wipe, solemn_simulacrum, helm_of_the_host, glacial_fortress, adarkar_wastes,
    temple_of_enlightenment, castle_ardenvale, seachrome_coast, sulfur_falls, cascade_bluffs,
    exotic_orchard, clifftop_retreat, shivan_reef, temple_of_triumph, battlefield_forge,
    skycloud_expanse, temple_of_epiphany, ferrous_lake, rugged_prairie, sunscorched_divide,
    spirited_companion, inspiring_overseer, cut_a_deal, path_to_exile, illusory_ambusher,
    rowdy_research, plumecreed_escort, rapid_hybridization, junk_winder, aetherize,
    chart_a_course, tetsuko_umezawa_fugitive, thopter_engineer, cloudblazer, arcane_signet,
    boros_signet, ornithopter_of_paradise, azorius_signet, izzet_signet, circuit_mender,
    fellwar_stone, sol_ring, mind_stone, terramorphic_expanse, path_of_ancestry, thriving_heath,
    evolving_wilds, thriving_isle, thriving_bluff, command_tower, mystic_monastery,
    // Basics: 5 plains, 4 island, 4 mountain
    plains, plains, plains, plains, plains, island, island, island, island, mountain, mountain,
    mountain, mountain,
];

pub const LATHLISS_COMMANDERS: &[CardFactory] = &[lathliss_dragon_queen];

/// **Reign of Dragons**, the Foundations Commander deck (FDC, 2024-11-15),
/// exactly as MTGJSON's `ReignOfDragons_FDC` prints it: 67 nonbasic
/// cards + 32 Mountains = 99. Mono-red Dragons under Lathliss, Dragon Queen.
pub const LATHLISS_MAIN: &[CardFactory] = &[
    atsushi_the_blazing_sky, blasphemous_act, chain_reaction, chandras_ignition, chaos_warp,
    count_on_luck, crucible_of_fire, cursed_mirror, dragon_tempest, dragonhawk_fates_tempest,
    dragonmaster_outcast, drakuseth_maw_of_flames, the_elder_dragon_war,
    goddric_cloaked_reveler, goldlust_triad, hellkite_charger, hit_the_mother_lode,
    leyline_tyrant, magmaquake, minion_of_the_mighty, nogi_draco_zealot, orb_of_dragonkind,
    outpost_siege, parapet_thrasher, sarkhan_dragon_ascendant, scourge_of_the_throne,
    scourge_of_valkas, shivan_devastator, spit_flame, taurean_mauler, terror_of_mount_velus,
    thunderbreak_regent, thundermane_dragon, tyrants_familiar, utvara_hellkite, warstorm_surge,
    basilisk_collar, dragons_hoard, bonders_enclave, haven_of_the_spirit_dragon,
    spinerock_knoll, war_room, abrade, anger, bitter_reunion, breaching_dragonstorm,
    breath_weapon, carnelian_orb_of_dragonkind, dragonlords_servant, dragonspeaker_shaman,
    firespitter_whelp, lightning_bolt, mana_geyser, rapacious_dragon, skyline_despot,
    thrill_of_possibility, unexpected_windfall, arcane_signet, commanders_sphere,
    dragonstorm_globe, fire_diamond, hazorets_monument, heralds_horn, sol_ring, swiftfoot_boots,
    forgotten_cave, temple_of_the_false_god,
    // Basics: 32 mountain
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain,
];

pub const ELLIVERE_COMMANDERS: &[CardFactory] = &[ellivere_of_the_wild_court];

/// **Virtue and Valor**, the Wilds of Eldraine Commander deck (WOC,
/// 2023-09-08), exactly as MTGJSON's `VirtueAndValor_WOC` prints it:
/// 70 nonbasic cards + 15 Forests + 14 Plains = 99. Selesnya Auras and Roles under
/// Ellivere of the Wild Court.
pub const ELLIVERE_MAIN: &[CardFactory] = &[
    gylwain_casting_director, liberated_livestock, ox_drover, songbirds_blessing,
    unfinished_business, giant_inheritance, knickknack_ouphe, loamcrafter_faun, timber_paladin,
    ajanis_chosen, angelic_destiny, archon_of_suns_grace, austere_command, celestial_archon,
    daybreak_coronet, eidolon_of_countless_battles, kor_spiritdancer, mantle_of_the_ancients,
    realm_cloaked_giant, retether, shalai_voice_of_plenty, starfield_mystic, sun_titan,
    timely_ward, tithe_taker, umbra_mystic, winds_of_rath, bear_umbra, eidolon_of_blossoms,
    enchantresss_presence, indomitable_might, rishkars_expertise, sanctum_weaver,
    setessan_champion, verdant_embrace, canopy_vista, castle_ardenvale, fortified_village,
    hall_of_heliods_generosity, sungrass_prairie, temple_of_plenty, danitha_capashen,
    ethereal_armor, generous_gift, sages_reverie, spectral_steel, swords_to_plowshares,
    transcendent_envoy, ancestral_mask, aura_gnarlid, careful_cultivation, destiny_spinner,
    fertile_ground, kenriths_transformation, paradise_druid, snake_umbra, sylvan_ranger,
    utopia_sprawl, warbriar_blessing, jukai_naturalist, pollenbright_wings,
    siona_captain_of_the_pyleas, arcane_signet, sol_ring, command_tower, krosan_verge,
    myriad_landscape, vitu_ghazi_the_city_tree, tanglespan_lookout, syr_armont_the_redeemer,
    // Basics: 15 forest, 14 plains
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, plains, plains, plains,
];

pub const ESHKI_COMMANDERS: &[CardFactory] = &[eshki_temurs_roar];

/// **Temur Roar**, the Tarkir: Dragonstorm Commander deck (TDC, 2025-04-11),
/// exactly as MTGJSON's `TemurRoar_TDC` prints it: 85 nonbasic cards +
/// 3 Islands + 6 Mountains + 5 Forests = 99. Temur Dragons under Eshki.
pub const ESHKI_MAIN: &[CardFactory] = &[
    ureni_of_the_unwritten, deceptive_frostkite, hammerhead_tyrant, will_of_the_temur,
    parapet_thrasher, thundermane_dragon, zenith_festival, become_the_avalanche,
    broodcaller_scourge, keiga_the_tide_star, reflections_of_littjara, atsushi_the_blazing_sky,
    blasphemous_act, chaos_warp, dragonmaster_outcast, gadrak_the_crown_scourge, glorybringer,
    hellkite_courser, lathliss_dragon_queen, leyline_tyrant, nesting_dragon, nogi_draco_zealot,
    opportunistic_dragon, scourge_of_the_throne, skarrgan_hellkite, spit_flame, storms_wrath,
    stormbreath_dragon, taurean_mauler, territorial_hellkite, thunderbreak_regent,
    vengeful_ancestor, verix_bladewing, frontier_siege, selvalas_stampede, atarka_world_render,
    dragonlord_atarka, harbinger_of_the_hunt, sarkhan_soul_aflame, temur_ascendancy,
    dragons_hoard, steel_hellkite, cinder_glade, exotic_orchard, flooded_grove,
    haven_of_the_spirit_dragon, hinterland_harbor, karplusan_forest, kessig_wolf_run,
    mossfire_valley, mosswort_bridge, rockfall_vale, rootbound_crag, sheltered_thicket,
    shivan_reef, sulfur_falls, temple_of_abandon, temple_of_mystery, yavimaya_coast,
    dragon_tempest, breaching_dragonstorm, temple_of_the_dragon_queen, arcane_signet, sol_ring,
    command_tower, stormshriek_feral, encroaching_dragonstorm, draconic_lore,
    rapid_hybridization, reality_shift, dragonlords_servant, rapacious_dragon,
    whirlwing_stormbrood, beast_within, elemental_bond, farseek, evolving_wilds,
    frontier_bivouac, kodamas_reach, migration_path, fellwar_stone, talisman_of_creativity,
    talisman_of_impulse, bountiful_landscape, path_of_ancestry,
    // Basics: 3 island, 6 mountain, 5 forest
    island, island, island, mountain, mountain, mountain, mountain, mountain, mountain, forest,
    forest, forest, forest, forest,
];

pub const KITT_COMMANDERS: &[CardFactory] = &[kitt_kanto_mayhem_diva];

/// **Cabaretti Cacophony**, the Streets of New Capenna Commander Naya deck
/// (NCC, 2022-04-29), exactly as MTGJSON's `CabarettiCacophony_NCC` prints it:
/// 83 nonbasic cards + 8 Forests + 4 Plains + 4 Mountains = 99. Citizens,
/// alliance and goad under Kitt Kanto, Mayhem Diva.
pub const KITT_MAIN: &[CardFactory] = &[
    phabine_bosss_confidant, leafkin_druid, sakura_tribe_elder, agitator_ant,
    bess_soul_nourisher, champion_of_lambholt, magus_of_the_wheel, orzhov_advokist,
    rumor_gatherer, scute_swarm, selvala_explorer_returned, wood_elves, zurzoth_chaos_rider,
    arasta_of_the_endless_web, life_of_the_party, master_of_ceremonies, rose_room_treasurer,
    sizzling_soloist, bosss_chauffeur, gahiji_honored_one, kazuul_tyrant_of_the_cliffs,
    thunderfoot_baloth, cabaretti_confluence, camaraderie, cultivate, fell_the_mighty,
    harmonize, indulge_excess, martial_coup, seize_the_spotlight, shamanic_revelation, sylvan_offering,
    viviens_stampede, artifact_mutation, aura_mutation, beast_within, boros_charm,
    cabaretti_charm, crash_the_party, call_the_coppercoats, grand_crescendo,
    march_of_the_multitudes, path_to_exile, sol_ring, arcane_signet, bloodthirsty_blade,
    fellwar_stone, idol_of_oblivion, commanders_sphere, scepter_of_celebration, false_floor,
    intangible_virtue, awakening_zone, beastmaster_ascension, duelists_heritage, killer_service,
    prosperous_partnership, felidar_retreat, outpost_siege, assemble_the_legion,
    sandwurm_convergence, ash_barrens, cabaretti_courtyard, canopy_vista, castle_ardenvale,
    castle_embereth, cinder_glade, command_tower, exotic_orchard, fortified_village, game_trail,
    jungle_shrine, mossfire_valley, myriad_landscape, naya_panorama, path_of_ancestry,
    rugged_prairie, sungrass_prairie, temple_of_triumph, thriving_bluff, thriving_grove,
    thriving_heath, windbrisk_heights, plains, plains, plains, plains, mountain, mountain,
    mountain, mountain, forest, forest, forest, forest, forest, forest, forest, forest,
];

pub const UR_DRAGON_COMMANDERS: &[CardFactory] = &[the_ur_dragon];

/// **Draconic Domination**, the Commander 2017 deck (C17, 2017-08-25),
/// exactly as MTGJSON's `DraconicDomination_C17` prints it: 81 nonbasic
/// cards + 6 Mountains + 3 Swamps + 3 Forests + 3 Plains + 3 Islands = 99.
/// Five-color Dragons under The Ur-Dragon.
pub const UR_DRAGON_MAIN: &[CardFactory] = &[
    o_kagachi_vengeful_kami, ramos_dragon_engine, scalelord_reckoner, boneyard_scourge,
    territorial_hellkite, taigam_ojutai_master, wasitora_nekoru_queen, ryusei_the_falling_star,
    scourge_of_valkas, utvara_hellkite, sunscorch_regent, deathbringer_regent, hellkite_charger,
    tyrants_familiar, atarka_world_render, bladewing_the_risen, broodmate_dragon,
    crosis_the_purger, dromoka_the_eternal, intet_the_dreamer, kolaghan_the_storms_fury,
    nivmizzet_dracogenius, ojutai_soul_of_winter, scion_of_the_ur_dragon,
    silumgar_the_drifting_death, spellbound_dragon, teneb_the_harvester, steel_hellkite,
    orator_of_ojutai, dragonlords_servant, dragonspeaker_shaman, savage_ventmaw, fortunate_few,
    fractured_identity, crux_of_fate, painful_truths, earthquake, cultivate, farseek,
    kodamas_reach, rain_of_thorns, fist_of_suns, heralds_horn, mirror_of_the_forebears,
    armillary_sphere, commanders_sphere, darksteel_ingot, dreamstone_hedron, lightning_greaves,
    nihil_spellbomb, sol_ring, wayfarers_bauble, kindred_discovery, monastery_siege,
    palace_siege, crucible_of_fire, dragon_tempest, frontier_siege, curse_of_verbosity,
    curse_of_opulence, curse_of_bounty, elemental_bond, crucible_of_the_spirit_dragon,
    haven_of_the_spirit_dragon, path_of_ancestry, arcane_sanctum, command_tower,
    crumbling_necropolis, frontier_bivouac, jungle_shrine, mystic_monastery, nomad_outpost,
    opulent_palace, sandsteppe_citadel, savage_lands, seaside_citadel, vivid_crag, vivid_creek,
    vivid_grove, vivid_marsh, vivid_meadow,
    // Basics: 6 mountain, 3 swamp, 3 forest, 3 plains, 3 island
    mountain, mountain, mountain, mountain, mountain, mountain, swamp, swamp, swamp, forest,
    forest, forest, plains, plains, plains, island, island, island,
];
