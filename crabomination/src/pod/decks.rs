//! The Commander pod's target decks.
//!
//! Ten hand-picked commanders plus a hundred and fifty-three official preconstructed lists, each a
//! 99 of cards this engine implements: legal under CR 903 (100 cards
//! including the commander, singleton outside basics, every card inside the
//! commander's CR 903.4 color identity) and Commander-legal per Scryfall's
//! ban list, both of which `pod::tests` asserts rather than trusts.
//!
//! The ten were built by hand because the offline Scryfall cache cannot say
//! how a precon splits into decks; MTGJSON's deck files can, and the eleventh
//! through hundred-and-sixty-third (Sultai Arisen, Mind Flayarrrs, Blood Rites, Heads I
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
//! Foundations Commander's Reign of Dragons, among others — seat 124 is
//! Innistrad: Crimson Vow Commander's Spirit Squadron, seat 133 Duskmourn
//! Commander's Jump Scare!, seat 143 Streets of New Capenna Commander's
//! Obscura Operation, seat 150 Tales of Middle-earth Commander's Elven
//! Council, seat 158 The Lost Caverns of Ixalan Commander's Ahoy Mateys
//! and seat 163 Final Fantasy XIV's Scions & Spellcraft) are taken from one card for card. What all of them keep
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

pub const ULALEK_COMMANDERS: &[CardFactory] = &[ulalek_fused_atrocity];

/// **Eldrazi Incursion**, the Modern Horizons 3 Commander five-color deck (M3C,
/// 2024-06-14), exactly as MTGJSON's `EldraziIncursion_M3C` prints it: 93
/// nonbasic cards + 1 Plains + 1 Island + 1 Swamp + 1 Mountain + 1 Forest + 1 Wastes = 99. Colorless Eldrazi under Ulalek, Fused
/// Atrocity.
pub const ULALEK_MAIN: &[CardFactory] = &[
    azlask_the_swelling_scourge, ulamogs_dreadsire, benthic_anomaly, eldritch_immunity,
    selective_obliteration, angelic_aberration, bismuth_mindrender, hideous_taskmaster,
    twins_of_discord, mutated_cultist, eldrazi_confluence, spawnbed_protector,
    chittering_dispatcher, inversion_behemoth, ugin_the_ineffable, eldrazi_conscription,
    cascading_cataracts, eldrazi_monument, sire_of_stagnation, shrine_of_the_forsaken_gods,
    ugins_insight, drowner_of_hope, return_of_the_wildspeaker, tendo_ice_bridge,
    battlefield_forge, llanowar_wastes, brushland, underground_river, oblivion_sower,
    adarkar_wastes, karplusan_forest, yavimaya_coast, sulfurous_springs, caves_of_koilos,
    shivan_reef, imprisoned_in_the_moon, temple_of_malady, temple_of_silence,
    morophon_the_boundless, kozileks_return, world_breaker, ruins_of_oran_rief,
    sifter_of_skulls, endbringer, corrupted_crossroads, deepfathom_skulker, vile_redeemer,
    eldrazi_displacer, mystic_forge, all_is_dust, rishkars_expertise, awakening_zone,
    eldrazi_temple, elder_deep_fiend, bonders_enclave, exotic_orchard, forsaken_monument,
    snapping_voidcraw, heralds_horn, everflowing_chalice, glaring_fleshraker,
    wastescape_battlemage, crib_swap, titans_vanguard, talisman_of_resilience, hedron_archive,
    spawning_bed, ulamogs_nullifier, twisted_landscape, tranquil_landscape, tectonic_edge,
    reliquary_tower, sol_ring, artisan_of_kozilek, opal_palace, dreamstone_hedron,
    arcane_signet, tomb_of_the_spirit_dragon, secluded_courtyard, idol_of_oblivion,
    talisman_of_curiosity, talisman_of_impulse, warping_wail, talisman_of_dominance,
    command_tower, ash_barrens, ancient_stirrings, ulamogs_crusher, garruks_uprising,
    suffer_the_past, skittering_invasion, path_of_ancestry, unclaimed_territory,
    // Basics: 1 plains, 1 island, 1 swamp, 1 mountain, 1 forest, 1 wastes
    plains, island, swamp, mountain, forest, wastes,
];

pub const KASLA_COMMANDERS: &[CardFactory] = &[kasla_the_broken_halo];

/// **Divine Convocation**, the March of the Machine Commander deck (MOC,
/// 2023-04-21), exactly as MTGJSON's `DivineConvocation_MOC` prints it:
/// 75 nonbasic cards + 8 Plains + 8 Islands + 8 Mountains = 99. Jeskai convoke under Kasla, the
/// Broken Halo.
pub const KASLA_MAIN: &[CardFactory] = &[
    saint_traft_and_rem_karolus, wand_of_the_worldsoul, flockchaser_phantom, wildfire_awakener,
    ichor_elixir, path_of_the_ghosthunter, deluxe_dragster, mistmeadow_vanisher,
    nesting_dovehawk, kykar_winds_fury, elspeth_suns_champion, the_locust_god,
    angel_of_finality, angel_of_salvation, austere_command, chasm_skulker, cultivators_caravan,
    emeria_angel, exotic_orchard, frostboil_snarl, furycalm_snarl, hour_of_reckoning,
    keeper_of_the_accord, kher_keep, mentor_of_the_meek, nadir_kraken, port_town,
    prairie_stream, secure_the_wastes, skycloud_expanse, temple_of_enlightenment,
    temple_of_epiphany, temple_of_triumph, venerated_loxodon, whirlwind_of_thought,
    cloud_of_faeries, command_tower, commanders_sphere, ephemeral_shields, evolving_wilds,
    fractured_powerstone, goblin_instigator, goblin_medics, impact_tremors, spirited_companion,
    suture_priest, terramorphic_expanse, village_bell_ringer, banisher_priest, arcane_signet,
    battle_screech, chant_of_vitu_ghazi, conclave_tribunal, devouring_light, duergar_hedge_mage,
    fallowsage, flight_of_equenauts, improbable_alliance, migratory_route, mystic_monastery,
    rogues_passage, seraph_of_the_masses, skullclamp, sol_ring, stoke_the_flames,
    swords_to_plowshares, tetsuko_umezawa_fugitive, wear_tear, temporal_cleansing, meeting_of_minds,
    shatter_the_source, wrenns_resolve, cut_short, joyful_stormsculptor, artistic_refusal,
    // Basics: 8 plains, 8 island, 8 mountain
    plains, plains, plains, plains, plains, plains, plains, plains, island, island, island,
    island, island, island, island, island, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain,
];

pub const ESTRID_COMMANDERS: &[CardFactory] = &[estrid_the_masked];

/// **Adaptive Enchantment**, the Commander 2018 deck (C18, 2018-08-10), exactly
/// as MTGJSON's `AdaptiveEnchantment_C18` prints it: 76 nonbasic cards +
/// 9 Plains + 6 Islands + 8 Forests = 99. Bant enchantments under the planeswalker
/// Estrid, the Masked.
pub const ESTRID_MAIN: &[CardFactory] = &[
    bruna_light_of_alabaster, eidolon_of_blossoms, hydra_omnivore, ajanis_chosen,
    celestial_archon, silent_sentinel, boon_satyr, herald_of_the_pantheon, cold_eyed_selkie,
    daxos_of_meletis, elderwood_scion, archetype_of_imagination, whitewater_naiads,
    aura_gnarlid, reclamation_sage, yavimaya_enchantress, martial_coup, phyrexian_rebirth,
    winds_of_rath, creeping_renaissance, kruphixs_insight, dismantling_blow, bant_charm,
    sol_ring, sigil_of_the_empty_throne, dictate_of_kruphix, bear_umbra, enchantresss_presence,
    epic_proportions, ground_seal, spawning_grounds, finest_hour, righteous_authority,
    sages_reverie, soul_snare, unquestioned_authority, eel_umbra, vow_of_flight,
    dawns_reflection, fertile_ground, overgrowth, snake_umbra, vow_of_wildness, wild_growth,
    unflinching_courage, azorius_chancery, blossoming_sands, command_tower, evolving_wilds,
    krosan_verge, meandering_river, mosswort_bridge, seaside_citadel, selesnya_sanctuary,
    simic_growth_chamber, terramorphic_expanse, thornwood_falls, tranquil_cove,
    tranquil_expanse, woodland_stream, kestia_the_cultivator, tuvasa_the_sunlit, empyrial_storm,
    heavenly_blademaster, estrids_invocation, ever_watching_threshold, octopus_umbra,
    genesis_storm, myth_unbound, nyleas_colossus, ravenous_slime, arixmethes_slumbering_isle,
    loyal_unicorn, loyal_drake, loyal_guardian, forge_of_heroes,
    // Basics: 9 plains, 6 island, 8 forest
    plains, plains, plains, plains, plains, plains, plains, plains, plains, island, island,
    island, island, island, island, forest, forest, forest, forest, forest, forest, forest,
    forest,
];

pub const KATHRIL_COMMANDERS: &[CardFactory] = &[kathril_aspect_warper];

/// **Symbiotic Swarm**, the Ikoria Commander deck (C20, 2020-04-17), exactly
/// as MTGJSON's `SymbioticSwarm_C20` prints it: 81 nonbasic cards +
/// 7 Plains + 7 Swamps + 4 Forests = 99. Abzan keyword counters under Kathril, Aspect Warper.
pub const KATHRIL_MAIN: &[CardFactory] = &[
    ajani_unyielding, tayam_luminous_enigma, nikara_lair_scavenger, yannik_scavenging_sentinel,
    avenging_huntbonder, cartographers_hawk, vitality_hunter, daring_fiendbonder,
    slippery_bogbonder, akroma_angel_of_wrath, angel_of_finality, cataclysmic_gearhulk,
    kalemnes_captain, odric_lunarch_marshal, reveillark, solemn_recruit, sunblast_angel,
    zetalpa_primal_dawn, cairn_wanderer, soul_of_innistrad, soulflayer, hornet_queen,
    majestic_myriarch, splinterfright, archon_of_valors_reach, karametra_god_of_harvests,
    void_beckoner, titanoth_rex, aerial_responder, vampire_nighthawk, acidic_slime,
    sakura_tribe_elder, satyr_wayfinder, skullwinder, nyx_weaver, selective_adaptation,
    ever_after, duneblast, unbreakable_bond, unburial_rites, cultivate, harmonize,
    obscuring_haze, blood_curdle, abzan_charm, deathsprout, despark, grisly_salvage,
    netherborn_altar, mimic_vat, bonders_ornament, arcane_signet, commanders_sphere, sol_ring,
    swiftfoot_boots, together_forever, abzan_ascendancy, deadbridge_chant, martial_impetus,
    parasitic_impetus, predatory_impetus, nesting_grounds, canopy_vista, caves_of_koilos,
    exotic_orchard, gavony_township, grim_backwoods, sungrass_prairie, blighted_woodland,
    command_tower, golgari_rot_farm, krosan_verge, memorial_to_folly, myriad_landscape,
    orzhov_basilica, sandsteppe_citadel, selesnya_sanctuary, blossoming_sands, evolving_wilds,
    jungle_hollow, scoured_barrens,
    // Basics: 7 plains, 7 swamp, 4 forest
    plains, plains, plains, plains, plains, plains, plains, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, forest, forest, forest, forest,
];

pub const DRACONIC_RAGE_COMMANDERS: &[CardFactory] = &[vrondiss_rage_of_ancients];

/// Draconic Rage (AFC, 2021), card for card: Vrondiss's Gruul dragons and dice.
pub const DRACONIC_RAGE_MAIN: &[CardFactory] = &[
    bogardan_hellkite, demanding_dragon, dragonmaster_outcast, hoard_smelter_dragon,
    opportunistic_dragon, scourge_of_valkas, shivan_hellkite, skyline_despot, skyship_stalker,
    taurean_mauler, terror_of_mount_velus, thunderbreak_regent, chameleon_colossus,
    atarka_world_render, anger, dragonlords_servant, savage_ventmaw, dragonspeaker_shaman,
    chain_reaction, rishkars_expertise, shamanic_revelation, rile, cultivate, explore,
    rampant_growth, magmaquake, spit_flame, decree_of_savagery, kindred_summons,
    return_of_the_wildspeaker, beast_within, return_to_nature, dragons_hoard, arcane_signet,
    commanders_sphere, gruul_signet, heirloom_blade, sol_ring, gratuitous_violence,
    outpost_siege, warstorm_surge, colossal_majesty, garruks_uprising, cinder_glade,
    crucible_of_the_spirit_dragon, exotic_orchard, game_trail, haven_of_the_spirit_dragon,
    mossfire_valley, mosswort_bridge, command_tower, desert, gruul_turf, path_of_ancestry,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest, forest, forest, forest, forest, klauth_unrivaled_ancient,
    berserkers_frenzy, chaos_dragon, maddening_hex, vengeful_ancestor, bag_of_tricks,
    druid_of_purification, indomitable_might, neverwinter_hydra, wild_endeavor,
    dragonborn_champion, klauths_will, wulfgar_of_icewind_dale, barbarian_class,
    earth_cult_elemental, component_pouch, sword_of_hours, underdark_rift,
];

pub const KILLIAN_COMMANDERS: &[CardFactory] = &[killian_decisive_mentor];

/// **Silverquill Influence**, the Secrets of Strixhaven Commander deck (SOC),
/// exactly as MTGJSON's `SilverquillInfluence_SOC` prints it: 83 nonbasic
/// cards + 8 Plains + 8 Swamps = 99. Orzhov Auras and goad
/// under Killian, Decisive Mentor.
pub const KILLIAN_MAIN: &[CardFactory] = &[
    scriv_the_obligator, eiganjo_dynastorian, forum_filibuster, herald_of_amity,
    changing_loyalty, coercive_impetus, intermediate_chirography, defacing_duskmage,
    eclipsed_steppe, turbulent_moor, umbral_expanse, fabled_passage, eldrazi_conscription,
    ajanis_chosen, angelic_destiny, archon_of_suns_grace, armored_skyhunter,
    combat_calligrapher, eidolon_of_countless_battles, firemane_commando, gift_of_immortality,
    kor_spiritdancer, land_tax, mangara_the_diplomat, nils_discipline_enforcer,
    pearl_ear_imperial_advisor, promise_of_loyalty, redemption_arc, shielded_by_faith,
    songbirds_blessing, sram_senior_edificer, starfield_mystic, winds_of_rath, doomwake_giant,
    ghoulish_impetus, keen_duelist, anguished_unmaking, breena_the_demagogue,
    eriette_of_the_charmed_apple, inkshield, shadrix_silverquill, tomik_wielder_of_law,
    vanishing_verse, caves_of_koilos, desolate_mire, exotic_orchard, fetid_heath,
    isolated_chapel, shineshadow_snarl, temple_of_silence, war_room, flickering_ward,
    fallen_ideal, screams_from_within, arcane_signet, sol_ring, command_tower, forum_of_amity,
    terramorphic_expanse, chains_of_custody, darksteel_mutation, ghostly_prison,
    martial_impetus, raffines_guidance, sages_reverie, secret_rendezvous, sentinels_eyes,
    sheltered_by_ghosts, spirit_mantle, transcendent_envoy, animate_dead, hateful_eidolon,
    parasitic_impetus, fracture, killian_ink_duelist, fellwar_stone, talisman_of_hierarchy,
    arcane_lighthouse, bojuka_bog, path_of_ancestry, silverquill_campus, study_hall,
    sunlit_marsh,
    // Basics: 8 plains, 8 swamp
    plains, plains, plains, plains, plains, plains, plains, plains, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp,
];

pub const GALEA_COMMANDERS: &[CardFactory] = &[galea_kindler_of_hope];

/// **Aura of Courage**, the Adventures in the Forgotten Realms Commander deck
/// (AFC, 2021-07-23), exactly as MTGJSON's `AuraOfCourage_AFC` prints it:
/// 85 nonbasic cards + 2 Plains + 4 Islands + 8 Forests = 99. Bant Auras and
/// Equipment under Galea, Kindler of Hope.
pub const GALEA_MAIN: &[CardFactory] = &[
    angel_of_finality, puresteel_paladin, realm_cloaked_giant, sram_senior_edificer,
    prognostic_sphinx, cold_eyed_selkie, fleecemane_lion, knight_of_autumn, riverwise_augur,
    acidic_slime, paradise_druid, winds_of_rath, serum_visions, natures_lore,
    heroic_intervention, valorous_stance, brainstorm, bant_charm, argentum_armor,
    basilisk_collar, masterwork_of_ingenuity, moonsilver_spear, sword_of_the_animist,
    behemoth_sledge, arcane_signet, colossus_hammer, explorers_scope, sol_ring, swiftfoot_boots,
    viridian_longbow, imprisoned_in_the_moon, greater_good, verdant_embrace, angelic_gift,
    gryffs_boon, curse_of_verbosity, eel_umbra, psychic_impetus, abundant_growth,
    fertile_ground, kenriths_transformation, rancor, utopia_sprawl, wild_growth, shielding_plax,
    canopy_vista, exotic_orchard, fortified_village, lumbering_falls, port_town, prairie_stream,
    skycloud_expanse, sungrass_prairie, evolving_wilds, azorius_chancery, bant_panorama,
    command_tower, flood_plain, grasslands, halimar_depths, mishras_factory, path_of_ancestry,
    seaside_citadel, simic_growth_chamber, terramorphic_expanse, thriving_grove, thriving_heath,
    thriving_isle, vitu_ghazi_the_city_tree, storvald_frost_giant_jarl, fey_steed, holy_avenger,
    mantle_of_the_ancients, robe_of_stars, valiant_endeavor, diviners_portent,
    netherese_puzzle_ward, winged_boots, belt_of_giant_strength, song_of_inspiration,
    catti_brie_of_mithral_hall, ride_the_avalanche, clay_golem, ebony_fly, sword_of_hours,
    // Basics: 2 plains, 4 island, 8 forest
    plains, plains, island, island, island, island, forest, forest, forest, forest, forest,
    forest, forest, forest,
];

pub const FIRKRAAG_COMMANDERS: &[CardFactory] = &[firkraag_cunning_instigator];

/// Draconic Dissent (CLB, 2022), card for card: Firkraag's Izzet goad dragons.
pub const FIRKRAAG_MAIN: &[CardFactory] = &[
    will_kenrith, rowan_kenrith, keiga_the_tide_star, pursued_whale, sly_instigator,
    agitator_ant, avatar_of_slaughter, brash_taunter, chaos_dragon, drakuseth_maw_of_flames,
    geode_rager, goblin_spymaster, kazuul_tyrant_of_the_cliffs, ryusei_the_falling_star,
    territorial_hellkite, thunder_dragon, vengeful_ancestor, warmonger_hellkite,
    niv_mizzet_parun, solemn_simulacrum, steel_hellkite, stuffy_doll, burnished_hart,
    sprite_dragon, aether_gale, curse_of_the_swine, blasphemous_act, chain_reaction,
    disrupt_decorum, compulsive_research, domineering_will, reins_of_power, chaos_warp,
    midnight_clock, dragons_hoard, arcane_signet, mind_stone, wayfarers_bauble,
    bloodthirsty_blade, fellwar_stone, hedron_archive, izzet_signet, sol_ring,
    talisman_of_creativity, dissipation_field, the_akroan_war, psychic_impetus, shiny_impetus,
    curse_of_verbosity, propaganda, curse_of_opulence, castle_vantress, desolate_lighthouse,
    kher_keep, temple_of_epiphany, wandering_fumarole, command_tower, ash_barrens,
    izzet_boilerworks, myriad_landscape, path_of_ancestry, prismari_campus, reliquary_tower,
    temple_of_the_false_god, terrain_generator, island, island, island, island, island, island,
    island, island, island, island, island, island, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    baeloth_barrityl_entertainer, clan_crafter, artificer_class, astral_dragon,
    mocking_doppelganger, bothersome_quasit, death_kiss, loot_dispute, spectacular_showdown,
    angler_turtle,
];

pub const ZHULODOK_COMMANDERS: &[CardFactory] = &[zhulodok_void_gorger];

/// **Eldrazi Unbound**, the Commander Masters colorless deck (CMM,
/// 2023-08-04), exactly as MTGJSON's `EldraziUnbound_CMM` prints it: 84
/// nonbasic cards + 15 Wastess = 99. Colorless Eldrazi ramp under Zhulodok,
/// Void Gorger.
pub const ZHULODOK_MAIN: &[CardFactory] = &[
    omarthis_ghostfire_initiate, kozilek_the_great_distortion, abstruse_archaic,
    calamity_of_the_titans, desecrate_reality, flayer_of_loyalties, rise_of_the_eldrazi,
    skittering_cicada, ugins_mastery, darksteel_monolith, all_is_dust, endbringer, endless_one,
    it_that_betrays, matter_reshaper, oblivion_sower, ugin_the_ineffable, ancient_stone_idol,
    duplicant, endless_atlas, forsaken_monument, hangarback_walker, investigators_journal,
    kaldra_compleat, mazemind_tome, metalwork_colossus, mirage_mirror, myriad_construct,
    mystic_forge, perilous_vault, phyrexian_triniform, solemn_simulacrum, soul_of_new_phyrexia,
    steel_hellkite, stonecoil_serpent, transmogrifying_wand, arch_of_orazca, blast_zone,
    bonders_enclave, geier_reach_sanitarium, mirrorpool, ruins_of_oran_rief, scavenger_grounds,
    sea_gate_wreckage, shrine_of_the_forsaken_gods, tyrite_sanctum, war_room, burnished_hart,
    geode_golem, lightning_greaves, meteor_golem, sol_ring, thought_vessel, thran_dynamo,
    unstable_obelisk, reliquary_tower, rogues_passage, artisan_of_kozilek, bane_of_bala_ged,
    not_of_this_world, spatial_contortion, titans_presence, warping_wail, crashing_drawbridge,
    dreamstone_hedron, everflowing_chalice, fireshrieker, hedron_archive, mind_stone,
    ornithopter_of_paradise, palladium_myr, scaretiller, suspicious_bookcase, worn_powerstone,
    arcane_lighthouse, eldrazi_temple, forge_of_heroes, guildless_commons, mage_ring_network,
    temple_of_the_false_god, tomb_of_the_spirit_dragon, urzas_mine, urzas_power_plant,
    urzas_tower,
    // Basics: 15 wastes
    wastes, wastes, wastes, wastes, wastes, wastes, wastes, wastes, wastes, wastes, wastes,
    wastes, wastes, wastes, wastes,
];

pub const CHISHIRO_COMMANDERS: &[CardFactory] = &[chishiro_the_shattered_blade];

/// **Upgrades Unleashed**, the Kamigawa: Neon Dynasty Commander deck (NEC,
/// 2022-02-18), as MTGJSON's `UpgradesUnleashed_NEC` prints it: 72 nonbasic
/// cards + 12 Mountains + 14 Forests + a second Mossfire Valley = 99. ⚠ The
/// printed deck really ships two Mossfire Valleys, so it isn't singleton out
/// of the box (CR 903.5b); the extra copy is a thirteenth Mountain here.
/// Gruul "modified" creatures — counters, Auras, Equipment — under Chishiro.
pub const CHISHIRO_MAIN: &[CardFactory] = &[
    kaima_the_fractured_calm, ox_of_agonas, akki_battle_squad, kami_of_celebration,
    komainu_battle_armor, ascendant_acolyte, kosei_penitent_warlord, rampant_rejuvenator,
    tanuki_transplanter, orochi_merge_keeper, towashi_guide_bot, walking_skyscraper,
    agitator_ant, goblin_razerunners, krenko_tin_street_kingpin, taurean_mauler, acidic_slime,
    champion_of_lambholt, fertilid, forgotten_ancient, genesis_hydra, loyal_guardian,
    primeval_protector, rishkar_peema_renegade, sakura_tribe_elder, spearbreaker_behemoth,
    whiptongue_hydra, grumgully_the_generous, ulasht_the_hate_seed, collision_of_realms,
    smoke_spirits_aid, chain_reaction, kodamas_reach, rampant_growth, rishkars_expertise,
    shamanic_revelation, souls_majesty, vastwood_surge, decimate, silkguard, chaos_warp,
    starstorm, beast_within, hunters_insight, mage_slayer, arcane_signet, blackblade_reforged,
    bonehoard, fireshrieker, sol_ring, swiftfoot_boots, sword_of_vengeance, unquenchable_fury,
    concord_with_the_kami, one_with_the_kami, invigorating_hot_spring, elemental_mastery,
    shifting_shadow, bear_umbra, ordeal_of_nylea, snake_umbra, rhythm_of_the_wild, cinder_glade,
    command_tower, exotic_orchard, game_trail, gruul_turf, mossfire_valley, opal_palace,
    oran_rief_the_vastwood, raging_ravine, temple_of_abandon,
    // Basics: 13 mountain (12 + the second Mossfire Valley), 14 forest
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest, forest, forest, forest, forest,
];

pub const JARED_COMMANDERS: &[CardFactory] = &[jared_carthalion];

/// **Painbow**, the Dominaria United Commander deck (DMC, 2022-09-09),
/// exactly as MTGJSON's `Painbow_DMC` prints it: 88 nonbasic cards +
/// 3 Forests + 2 Mountains + 2 Islands + 2 Plains + 2 Swamps = 99. Five-color multicolor matters under Jared Carthalion, a
/// planeswalker commander.
pub const JARED_MAIN: &[CardFactory] = &[
    archelos_lagoon_mystic, atla_palani_nest_tender, baleful_strix, chromanticore,
    coiling_oracle, o_kagachi_vengeful_kami, faeburrow_elder, fusion_elemental,
    zaxara_the_exemplary, hero_of_precinct_one, illuna_apex_of_wishes, knight_of_new_alara,
    maelstrom_archangel, nethroi_apex_of_death, rienne_angel_of_rebirth,
    selvala_explorer_returned, solemn_simulacrum, surrak_dragonclaw, transguild_courier,
    glint_eye_nephilim, xyris_the_writhing_storm, cultivate, duneblast, explore,
    explosive_vegetation, farseek, kodamas_reach, lavalanche, merciless_eviction,
    migration_path, painful_truths, radiant_flames, search_for_tomorrow, time_wipe, abzan_charm,
    beast_within, echoing_truth, growth_spiral, terminate, naya_charm, path_to_exile,
    sultai_charm, sylvan_reclamation, arcane_signet, coalition_relic, fellwar_stone,
    commanders_sphere, prophetic_prism, abundant_growth, maelstrom_nexus,
    path_to_the_world_tree, bad_river, arcane_sanctum, canopy_vista, cascading_cataracts,
    cinder_glade, command_tower, crumbling_necropolis, crystal_quarry, evolving_wilds,
    exotic_orchard, flood_plain, frontier_bivouac, grasslands, jungle_shrine, krosan_verge,
    mountain_valley, murmuring_bosk, mystic_monastery, nomad_outpost, opulent_palace,
    prairie_stream, rocky_tar_pit, sandsteppe_citadel, savage_lands, seaside_citadel,
    smoldering_marsh, sunken_hollow, terramorphic_expanse, jenson_carthalion_druid_exile,
    two_headed_hellkite, tiller_engine, unite_the_coalition, fallaji_wayfarer,
    iridian_maelstrom, mana_cannons, primeval_spawn, obsidian_obelisk,
    // Basics: 3 forest, 2 mountain, 2 island, 2 plains, 2 swamp
    forest, forest, forest, mountain, mountain, island, island, plains, plains, swamp, swamp,
];

pub const JIRINA_COMMANDERS: &[CardFactory] = &[jirina_kudro];

/// **Ruthless Regiment**, the Commander 2020 deck (C20, 2020-04-17), exactly
/// as MTGJSON's `RuthlessRegiment_C20` prints it: 83 nonbasic cards + 4
/// Mountains + 8 Plains + 4 Swamps = 99. Mardu Humans under Jirina Kudro.
pub const JIRINA_MAIN: &[CardFactory] = &[
    nahiri_the_harbinger, kelsien_the_plague, trynn_champion_of_freedom,
    silvar_devourer_of_the_free, verge_rangers, species_specialist, titan_hunter,
    fireflux_squad, frontier_warmonger, bounty_agent, dearly_departed, frontline_medic,
    knight_of_the_white_orchid, magus_of_the_disk, odric_master_tactician, riders_of_gavony,
    thalias_lieutenant, thraben_doomsayer, disciple_of_bolas, xathrid_necromancer,
    alesha_who_smiles_at_death, captivating_crew, fumiko_the_lowblood, magus_of_the_wheel,
    titan_of_eternal_fire, adriana_captain_of_the_guard, generals_enforcer, banisher_priest,
    cavalry_pegasus, devout_chaplain, zulaport_cutthroat, humble_defector, garna_the_bloodflame,
    citywide_bust, cleansing_nova, increasing_devotion, painful_truths, ambitions_cost,
    call_the_coppercoats, flawless_maneuver, unexpectedly_absent, crackling_doom, dire_tactics,
    terminate, sanctuary_blade, bonders_ornament, arcane_signet, boros_signet,
    commanders_sphere, heirloom_blade, orzhov_signet, rakdos_signet, skullclamp, sol_ring,
    molten_echoes, outpost_siege, shared_animosity, sanctuary_lockdown, bastion_of_remembrance,
    martial_impetus, parasitic_impetus, shiny_impetus, vigilante_justice, battlefield_forge,
    exotic_orchard, shadowblood_ridge, smoldering_marsh, spinerock_knoll, windbrisk_heights,
    bojuka_bog, boros_garrison, command_tower, myriad_landscape, nomad_outpost, orzhov_basilica,
    path_of_ancestry, rakdos_carnarium, temple_of_the_false_god, unclaimed_territory,
    bloodfell_caves, evolving_wilds, scoured_barrens, wind_scarred_crag,
    // Basics: 4 mountain, 8 plains, 4 swamp
    mountain, mountain, mountain, mountain, plains, plains, plains, plains, plains, plains,
    plains, plains, swamp, swamp, swamp, swamp,
];

pub const ZAFFAI_COMMANDERS: &[CardFactory] = &[zaffai_thunder_conductor];

/// Prismari Performance (C21, 2021), card for card: Zaffai's Izzet big-spell magecraft.
pub const ZAFFAI_MAIN: &[CardFactory] = &[
    jaya_ballard, veyran_voice_of_duality, dazzling_sphinx, octavia_living_thesis,
    sly_instigator, inferno_project, radiant_performer, rionya_fire_dancer, diluvian_primordial,
    naru_meha_master_wizard, talrand_sky_summoner, charmbreaker_devils, dualcaster_mage,
    erratic_cyclops, etali_primal_storm, wildfire_devils, storm_kiln_artist,
    rootha_mercurial_artist, living_lore, humble_defector, crackling_drake, inspiring_refrain,
    muse_vortex, creative_technique, fiery_encore, rousing_refrain, surge_to_victory,
    aether_gale, apex_of_power, blasphemous_act, volcanic_vision, call_the_skybreaker,
    epic_experiment, elemental_masterpiece, expressive_iteration, ponder, serum_visions,
    treasure_cruise, faithless_looting, mana_geyser, reinterpret, aetherspouts,
    dig_through_time, resculpt, brainstorm, traumatic_visions, fiery_fall, seething_song,
    letter_of_acceptance, arcane_signet, hedron_archive, izzet_signet, mind_stone, sol_ring,
    talisman_of_creativity, metallurgic_summonings, swarm_intelligence, exotic_orchard,
    scavenger_grounds, shivan_reef, temple_of_epiphany, prismari_campus, study_hall,
    blighted_cataract, command_tower, desert_of_the_fervent, desert_of_the_mindful,
    forgotten_cave, izzet_boilerworks, lonely_sandbar, mage_ring_network, memorial_to_genius,
    myriad_landscape, reliquary_tower, temple_of_the_false_god, island, island, island, island,
    island, island, island, island, island, island, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain, elementalists_palette, minds_desire,
    brasss_bounty, sunbirds_invocation, pyromancers_goggles,
];

pub const ZURGO_COMMANDERS: &[CardFactory] = &[zurgo_stormrender];

/// **Mardu Surge**, the Tarkir: Dragonstorm Commander deck (TDC, 2025-04-11),
/// exactly as MTGJSON's `MarduSurge_TDC` prints it: 84 nonbasic cards +
/// 5 Plains + 5 Swamps + 5 Mountains = 99. Mardu tokens and attack triggers
/// under Zurgo Stormrender.
pub const ZURGO_MAIN: &[CardFactory] = &[
    neriv_crackling_vanguard, ainok_strike_leader, ironwill_forger, will_of_the_mardu,
    bone_devourer, within_range, goldlust_triad, infantry_shield, redoubled_stormsinger,
    adeline_resplendent_cathar, angel_of_invention, commanders_insignia, divine_visitation,
    emeria_angel, grand_crescendo, hero_of_bladehold, hour_of_reckoning, legion_loyalty,
    selfless_spirit, sun_titan, tocasias_welcome, twilight_drover, chittering_witch,
    eliminate_the_competition, gix_yawgmoth_praetor, mindblade_render, ophiomancer,
    yahenni_undying_partisan, grenzo_havoc_raiser, legion_warboss, ogre_battledriver,
    siege_gang_commander, tempt_with_vengeance, kaya_geist_hunter, blade_of_selves,
    idol_of_oblivion, myr_battlesphere, solemn_simulacrum, battlefield_forge, canyon_slough,
    castle_ardenvale, castle_embereth, caves_of_koilos, clifftop_retreat, dragonskull_summit,
    exotic_orchard, fetid_heath, isolated_chapel, shattered_sanctum, smoldering_marsh,
    temple_of_silence, temple_of_triumph, vault_of_the_archangel, windbrisk_heights,
    shadow_summoning, lightning_greaves, skullclamp, arcane_signet, sol_ring, command_tower,
    goldnight_commander, lingering_souls, release_the_dogs, stroke_of_midnight,
    swords_to_plowshares, bastion_of_remembrance, bitter_triumph, deadly_dispute,
    morbid_opportunist, viscera_seer, abrade, beetleback_chief, loyal_apprentice, nomad_outpost,
    aron_benalias_ruin, thalisse_reverent_medium, fellwar_stone, talisman_of_conviction,
    talisman_of_hierarchy, wayfarers_bauble, bojuka_bog, path_of_ancestry, shattered_landscape,
    terramorphic_expanse,
    // Basics: 5 plains, 5 swamp, 5 mountain
    plains, plains, plains, plains, plains, swamp, swamp, swamp, swamp, swamp, mountain,
    mountain, mountain, mountain, mountain,
];

pub const AMINATOU_COMMANDERS: &[CardFactory] = &[aminatou_the_fateshifter];

/// **Subjective Reality**, the Commander 2018 Esper deck (C18,
/// 2018-08-10), exactly as MTGJSON's `SubjectiveReality_C18` prints it: 83
/// nonbasic cards + 8 Plainss + 5 Islands + 3 Swamps = 99. Esper top-of-library under Aminatou, the
/// Fateshifter, a planeswalker commander.
pub const AMINATOU_MAIN: &[CardFactory] = &[
    enigma_sphinx, serra_avatar, adarkar_valkyrie, conundrum_sphinx, djinn_of_wishes,
    jeskai_infiltrator, sphinx_of_jwar_isle, sphinx_of_uthuun, phyrexian_delver,
    duskmantle_seer, high_priest_of_penance, silent_blade_oni, mulldrifter,
    ninja_of_the_deep_hours, sigiled_starfish, pilgrims_eye, terminus, entreat_the_angels,
    army_of_the_damned, akromas_vengeance, devastation_tide, dream_cache, ponder, portent,
    treasure_hunt, utter_end, banishing_stroke, return_to_dust, brainstorm, predict,
    telling_time, esper_charm, mortify, azorius_signet, commanders_sphere, crystal_ball,
    dimir_signet, mind_stone, orzhov_signet, seers_lantern, sol_ring, lightform, cloudform,
    arcane_sanctum, azorius_chancery, azorius_guildgate, barren_moor, command_tower,
    dimir_aqueduct, dimir_guildgate, dismal_backwater, forsaken_sanctuary, halimar_depths,
    jwar_isle_refuge, lonely_sandbar, meandering_river, mortuary_mire, new_benalia,
    orzhov_basilica, orzhov_guildgate, scoured_barrens, secluded_steppe, sejiri_refuge,
    submerged_boneyard, tranquil_cove, varina_lich_queen, yennett_cryptic_sovereign,
    boreas_charger, magus_of_the_balance, aminatous_augury, primordial_mist, entreat_the_dead,
    night_incarnate, skull_storm, sower_of_discord, yuriko_the_tigers_shadow,
    isolated_watchtower, aethermages_touch, loyal_unicorn, loyal_subordinate, geode_golem,
    forge_of_heroes, crib_swap,
    // Basics: 8 plains, 5 island, 3 swamp
    plains, plains, plains, plains, plains, plains, plains, plains, island, island, island,
    island, island, swamp, swamp, swamp,
];

pub const ROOTHA_COMMANDERS: &[CardFactory] = &[rootha_mastering_the_moment];

/// **Prismari Artistry**, the Secrets of Strixhaven Commander deck (SOC,
/// 2026-04-24), exactly as MTGJSON's `PrismariArtistry_SOC` prints it: 84
/// nonbasic cards + 8 Islands + 7 Mountains = 99. Izzet spells and
/// Elementals under Rootha, Mastering the Moment.
pub const ROOTHA_MAIN: &[CardFactory] = &[
    muddle_the_ever_changing, inspired_skypainter, abstract_performance, dirgur_focusmage,
    leitmotif_composer, furygale_flocking, prismari_pianist, renegade_bull, coastal_peak,
    scorched_geyser, turbulent_springs, faerie_mastermind, chain_reaction, determined_iteration,
    harmonic_prodigy, fabled_passage, archmage_emeritus, brazen_borrower, curiosity_crafter,
    dig_through_time, replication_technique, rite_of_replication, thunderclap_drake,
    blasphemous_act, chaos_warp, creative_technique, cursed_mirror, dance_with_calamity,
    goldspan_dragon, manaform_hellkite, mirrorwing_dragon, plargg_and_nassari,
    redoubled_stormsinger, rionya_fire_dancer, rousing_refrain, surge_to_victory, twinflame,
    volcanic_salvo, brudiclad_telchor_engineer, galazeth_prismari, magma_opus, prismari_command,
    veyran_voice_of_duality, solemn_simulacrum, cascade_bluffs, exotic_orchard, ferrous_lake,
    frostboil_snarl, hall_of_oracles, restless_spire, shivan_reef, sulfur_falls,
    temple_of_epiphany, arcane_signet, sol_ring, command_tower, prismari_charm,
    spectacle_summit, terramorphic_expanse, aether_gale, arcane_denial, deep_analysis,
    reality_shift, resculpt, treasure_cruise, abrade, big_score, mana_geyser, storm_kiln_artist,
    throes_of_chaos, volcanic_torrent, expressive_iteration, rootha_mercurial_artist,
    stormcatch_mentor, fellwar_stone, lightning_greaves, talisman_of_creativity,
    molten_tributary, mystic_sanctuary, path_of_ancestry, prismari_campus, reliquary_tower,
    study_hall, temple_of_the_false_god,
    // Basics: 8 island, 7 mountain
    island, island, island, island, island, island, island, island, mountain, mountain,
    mountain, mountain, mountain, mountain, mountain,
];

pub const OLIVIA_COMMANDERS: &[CardFactory] = &[olivia_opulent_outlaw];

/// **Most Wanted**, the Outlaws of Thunder Junction Commander deck (OTC,
/// 2024-04-19), exactly as MTGJSON's `MostWanted_OTC` prints it: 91 nonbasic
/// cards + 2 Plains + 4 Swamps + 2 Mountains = 99. Mardu outlaws and
/// Treasure under Olivia, Opulent Outlaw.
pub const OLIVIA_MAIN: &[CardFactory] = &[
    vihaan_goldwaker, councils_judgment, heliods_intervention, angelic_sell_sword,
    we_ride_at_dawn, massacre_girl, fain_the_broker, witch_of_the_moors, nighthawk_scavenger,
    curtains_call, misfortune_teller, painful_truths, kamber_the_plunderer, ogre_slumlord, hex,
    mari_the_killing_quill, discreet_retreat, charred_graverobber, back_in_town,
    marshland_bloodcaster, veinwitch_coven, rankle_master_of_pranks, dire_fleet_ravager,
    mirror_entity, dire_fleet_daredevil, captain_lannery_storm, seize_the_spotlight,
    grenzo_havoc_raiser, angraths_marauders, captivating_crew, rain_of_riches,
    laurine_the_diversion, mass_mutiny, dead_before_sunrise, graywaters_fixer, life_insurance,
    breena_the_demagogue, queen_marchesa, idol_of_oblivion, academy_manufactor, bounty_board,
    fetid_heath, command_beacon, vault_of_the_archangel, dragonskull_summit, temple_of_silence,
    temple_of_malice, exotic_orchard, temple_of_triumph, clifftop_retreat, isolated_chapel,
    bonders_enclave, caves_of_koilos, battlefield_forge, sulfurous_springs, rugged_prairie,
    desolate_mire, shadowblood_ridge, canyon_slough, smoldering_marsh, blackcleave_cliffs,
    mistmeadow_skulk, requisition_raid, changeling_outcast, feed_the_swarm, deadly_dispute,
    morbid_opportunist, aetherborn_marauder, tenured_inkcaster, shoot_the_sheriff,
    lightning_greaves, impulsive_pilferer, shiny_impetus, humble_defector, glittering_stockpile,
    boros_charm, arcane_signet, trailblazers_boots, bandits_haul, orzhov_signet, sol_ring,
    rakdos_signet, command_tower, bojuka_bog, path_of_ancestry, rogues_passage,
    demolition_field, tainted_peak, sunhome_fortress_of_the_legion, nomad_outpost,
    temple_of_the_false_god,
    // Basics: 2 plains, 4 swamp, 2 mountain
    plains, plains, swamp, swamp, swamp, swamp, mountain, mountain,
];

pub const KALAMAX_COMMANDERS: &[CardFactory] = &[kalamax_the_stormsire];

/// **Arcane Maelstrom**, the Ikoria Commander deck (C20, 2020-04-17), exactly
/// as MTGJSON's `ArcaneMaelstrom_C20` prints it: 81 nonbasic cards +
/// 8 Forests + 5 Islands + 5 Mountains = 99. Temur instants and copies under Kalamax, the Stormsire.
pub const KALAMAX_MAIN: &[CardFactory] = &[
    jace_architect_of_thought, xyris_the_writhing_storm, haldan_avid_arcanist,
    pako_arcane_retriever, eon_frolicker, nascent_metamorph, glademuse, ravenous_gigantotherium,
    lunar_mystic, niblis_of_frost, talrand_sky_summoner, charmbreaker_devils, dualcaster_mage,
    etali_primal_storm, goblin_dark_dwellers, djinn_illuminatus, melek_izzet_paragon,
    rashmi_eternities_crafter, wort_the_raidmother, solemn_simulacrum, murmuring_mystic,
    crackling_drake, surreal_memoir, decoy_gambit, deflecting_swat, curious_herd, chaos_warp,
    comet_storm, commune_with_lava, starstorm, strength_of_the_tajuru, artifact_mutation,
    prophetic_bolt, clash_of_titans, channeled_force, chemisters_insight, frantic_search,
    whiplash_trap, crop_rotation, evolution_charm, harrow, hunters_insight, hunting_pack,
    natural_connection, slice_in_twain, tribute_to_the_wild, growth_spiral, temur_charm,
    lavabrink_floodgates, twinning_staff, bonders_ornament, arcane_signet, commanders_sphere,
    lightning_greaves, sol_ring, swarm_intelligence, primal_empathy, psychic_impetus,
    shiny_impetus, predatory_impetus, wilderness_reclamation, cinder_glade, desolate_lighthouse,
    exotic_orchard, kessig_wolf_run, mossfire_valley, mosswort_bridge, oran_rief_the_vastwood,
    scavenger_grounds, yavimaya_coast, command_tower, frontier_bivouac, gruul_turf,
    halimar_depths, izzet_boilerworks, myriad_landscape, rupture_spire, simic_growth_chamber,
    rugged_highlands, swiftwater_cliffs, thornwood_falls,
    // Basics: 8 forest, 5 island, 5 mountain
    forest, forest, forest, forest, forest, forest, forest, forest, island, island, island,
    island, island, mountain, mountain, mountain, mountain, mountain,
];

pub const VEIL_PIERCER_COMMANDERS: &[CardFactory] = &[aminatou_veil_piercer];

/// **Miracle Worker**, the Duskmourn Commander deck (DSC, 2024-09-27),
/// exactly as MTGJSON's `MiracleWorker_DSC` prints it: 85 nonbasic cards +
/// 5 Plains + 4 Islands + 5 Swamps = 99. Esper enchantments and miracles
/// under Aminatou, Veil Piercer.
pub const VEIL_PIERCER_MAIN: &[CardFactory] = &[
    the_master_of_keys, redress_fate, secret_arcade_dusty_parlor, soaring_lightbringer,
    fear_of_sleep_paralysis, ancient_cellarspawn, cramped_vents_access_maze,
    metamorphosis_fanatic, phenomenon_investigators, mesa_enchantress, terminus,
    aminatous_augury, utter_end, entreat_the_angels, monologue_tax, ondu_spiritdancer,
    sigil_of_the_empty_throne, starfield_mystic, timely_ward, verge_rangers, dream_eater,
    extravagant_replication, mirrormade, one_with_the_multiverse, prognostic_sphinx,
    shark_typhoon, arvinox_the_mind_flail, demon_of_fates_design, doomwake_giant,
    nightmare_shepherd, athreos_shroud_veiled, inkshield, life_insurance, spirit_sisters_call,
    time_wipe, solemn_simulacrum, adarkar_wastes, caves_of_koilos, hall_of_heliods_generosity,
    temple_of_deceit, temple_of_enlightenment, temple_of_silence, underground_river,
    bottomless_pool_locker_room, terramorphic_expanse, auramancer, swords_to_plowshares,
    moon_blessed_cleric, ponder, portent, telling_time, diabolic_vision, arcane_signet,
    sol_ring, command_tower, cast_out, return_to_dust, sphere_of_safety, arcane_denial,
    archetype_of_imagination, brainstorm, otherworldly_gaze, thirst_for_meaning,
    the_eldest_reborn, read_the_bones, azorius_signet, brainstone, burnished_hart,
    commanders_sphere, mind_stone, orzhov_signet, arcane_sanctum, ash_barrens, azorius_chancery,
    bojuka_bog, dimir_aqueduct, evolving_wilds, halimar_depths, obscura_storefront,
    orzhov_basilica, tainted_field, tainted_isle, thriving_heath, thriving_isle, thriving_moor,
    // Basics: 5 plains, 4 island, 5 swamp
    plains, plains, plains, plains, plains, island, island, island, island, swamp, swamp, swamp,
    swamp, swamp,
];

pub const OTRIMI_COMMANDERS: &[CardFactory] = &[otrimi_the_ever_playful];

/// **Enhanced Evolution**, the Ikoria Commander deck (C20, 2020-04-17),
/// exactly as MTGJSON's `EnhancedEvolution_C20` prints it: 81 nonbasic cards +
/// 5 Swamps + 11 Forests + 2 Islands = 99. Sultai mutate under Otrimi, the Ever-Playful.
pub const OTRIMI_MAIN: &[CardFactory] = &[
    nissa_steward_of_elements, zaxara_the_exemplary, cazur_ruthless_stalker,
    ukkima_stalking_shadow, souvenir_snatcher, tidal_barracuda, boneyard_mycodrax, mindleecher,
    capricopian, sawtusk_demolisher, beast_whisperer, genesis_hydra, hungering_hydra,
    masked_admirers, predator_ooze, vastwood_hydra, vorapede, cold_eyed_selkie,
    wydwen_the_biting_gale, silent_arbiter, archipelagore, dreamtail_heron, pouncing_shoreshark,
    cavern_whisperer, chittering_harvester, insatiable_hemophage, auspicious_starrix, fertilid,
    glowstone_recluse, migratory_greathorn, boneyard_lurker, trumpeting_gnarr,
    illusory_ambusher, mulldrifter, shriekmaw, heroes_bane, reclamation_sage, yavimaya_dryad,
    trygon_predator, dredge_the_mire, mind_spring, deadly_tempest, profane_command,
    animists_awakening, find_finality, gaze_of_granite, villainous_wealth, migration_path, kodamas_reach,
    deadly_rollick, beast_within, krosan_grip, putrefy, manascape_refractor,
    lifecrafters_bestiary, bonders_ornament, arcane_signet, sol_ring, psychic_impetus,
    parasitic_impetus, predatory_impetus, propaganda, darkwater_catacombs, endless_sands,
    exotic_orchard, llanowar_wastes, sunken_hollow, blighted_woodland, command_tower,
    dimir_aqueduct, golgari_rot_farm, mortuary_mire, myriad_landscape, opulent_palace,
    rogues_passage, simic_growth_chamber, soaring_seacliff, temple_of_the_false_god,
    dismal_backwater, jungle_hollow, thornwood_falls,
    // Basics: 5 swamp, 11 forest, 2 island
    swamp, swamp, swamp, swamp, swamp, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest, island, island,
];

pub const LEINORE_COMMANDERS: &[CardFactory] = &[leinore_autumn_sovereign];

/// **Coven Counters**, the Innistrad: Midnight Hunt Commander deck (MIC,
/// 2021-09-24), exactly as MTGJSON's `CovenCounters_MIC` prints it: 75 nonbasic
/// cards + 12 Plains + 12 Forests = 99. Selesnya +1/+1 counters and coven
/// (three different powers) under Leinore, Autumn Sovereign.
pub const LEINORE_MAIN: &[CardFactory] = &[
    angel_of_glorys_rise, bastion_protector, custodi_soulbinders, dearly_departed,
    herald_of_war, knight_of_the_white_orchid, mikaeus_the_lunarch, odric_master_tactician,
    riders_of_gavony, victorys_envoy, champion_of_lambholt, gyre_sage, kessig_cagebreakers,
    somberwald_sage, verdurous_gearhulk, wild_beastmaster, herons_grace_champion,
    sigarda_herons_grace, abzan_falconer, ainok_bond_kin, elite_scaleguard, orzhov_advokist,
    avacyns_pilgrim, eternal_witness, yavimaya_elder, enduring_scalelord, juniper_order_ranger,
    trostanis_summoner, cleansing_nova, hour_of_reckoning, shamanic_revelation, bestial_menace,
    biogenic_upgrade, growth_spasm, unbreakable_formation, return_to_dust, swords_to_plowshares,
    beast_within, inspiring_call, lifecrafters_bestiary, arcane_signet, sol_ring,
    swiftfoot_boots, talisman_of_unity, citadel_siege, deaths_presence, canopy_vista,
    exotic_orchard, fortified_village, sungrass_prairie, temple_of_plenty, blighted_woodland,
    command_tower, krosan_verge, myriad_landscape, path_of_ancestry, rogues_passage,
    selesnya_sanctuary, temple_of_the_false_god, kyler_sigardian_emissary, celestial_judgment,
    curse_of_conformity, moorland_rescuer, sigardas_vanguard, stalwart_pathlighter,
    wall_of_mourning, celebrate_the_harvest, curse_of_clinging_webs, heronblade_elite,
    kurbis_harvest_celebrant, ruinous_intrusion, sigardian_zealot, somberwald_beastmaster,
    dawnhart_wardens, moonsilver_key,
    // Basics: 12 plains, 12 forest
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest,
];

pub const SIDAR_COMMANDERS: &[CardFactory] = &[sidar_jabari_of_zhalfir];

/// **Cavalry Charge**, the March of the Machine Commander deck (MOC,
/// 2023-04-21), exactly as MTGJSON's `CavalryCharge_MOC` prints it: 80 nonbasic
/// cards + 8 Plains + 6 Islands + 5 Swamps = 99. Esper Knights under Sidar
/// Jabari of Zhalfir.
pub const SIDAR_MAIN: &[CardFactory] = &[
    elenda_and_azor, exsanguinator_cavalry, ichor_elixir, herald_of_hoofbeats, locthwain_lancer,
    chivalric_alliance, path_of_the_enigma, vodalian_wave_knight, conjurers_mantle,
    ethersworn_adjudicator, hero_of_bladehold, vona_butcher_of_magan, acclaimed_contender,
    adeline_resplendent_cathar, aryel_knight_of_windgrace, choked_estuary, exotic_orchard,
    fell_the_mighty, haakon_stromgald_scourge, josu_vess_lich_knight, knight_exemplar,
    knight_of_the_white_orchid, knights_charge, lilianas_standard_bearer, maul_of_the_skyclaves,
    midnight_reaper, murderous_rider, painful_truths, port_town, prairie_stream,
    promise_of_loyalty, pull_from_tomorrow, shineshadow_snarl, sigiled_sword_of_valeron,
    silverwing_squadron, sunken_hollow, temple_of_deceit, temple_of_enlightenment,
    temple_of_silence, time_wipe, unbreakable_formation, valiant_knight, vanquishers_banner,
    worthy_knight, bojuka_bog, command_tower, commanders_sphere, distant_melody, evolving_wilds,
    fractured_powerstone, orzhov_signet, path_of_ancestry, read_the_bones, smitten_swordmaster,
    terramorphic_expanse, thriving_heath, thriving_isle, thriving_moor, arcane_sanctum,
    arcane_signet, arvad_the_cursed, corpse_knight, despark, fellwar_stone, foulmire_knight,
    heralds_horn, knight_of_the_last_breath, knights_of_the_black_rose, mind_stone,
    myriad_landscape, order_of_midnight, path_to_exile, return_to_dust, sol_ring,
    swords_to_plowshares, syr_elenora_the_discerning, syr_konrad_the_grim,
    temple_of_the_false_god, wintermoor_commander, xerex_strobe_knight,
    // Basics: 8 plains, 6 island, 5 swamp
    plains, plains, plains, plains, plains, plains, plains, plains, island, island, island,
    island, island, island, swamp, swamp, swamp, swamp, swamp,
];

pub const OMO_COMMANDERS: &[CardFactory] = &[omo_queen_of_vesuva];

/// **Tricky Terrain**, the Modern Horizons 3 Commander deck (M3C,
/// 2024-06-14), exactly as MTGJSON's `TrickyTerrain_M3C` prints it: 92
/// nonbasic cards + 3 Islands + 4 Forests = 99. Simic lands-matter under Omo,
/// Queen of Vesuva.
pub const OMO_MAIN: &[CardFactory] = &[
    jyoti_moag_ancient, march_from_velis_vel, wonderscape_sage, copy_land, desert_warfare,
    trenchpost, sage_of_the_maze, lazotep_quarry, rampant_frogantua, sunken_palace,
    horizon_of_progress, planar_nexus, aggressive_biomancy, talon_gates_of_madara,
    hydroid_krasis, terastodon, lair_of_the_hydra, nissa_steward_of_elements,
    magus_of_the_candelabra, lumbering_falls, blast_zone, apex_devastator, dreamroot_cascade,
    summary_dismissal, yavimaya_coast, flooded_grove, hour_of_promise, ramunap_excavator,
    hydra_broodmaster, drown_in_dreams, vineglimmer_snarl, vivien_reid,
    yavimaya_cradle_of_growth, dryad_of_the_ilysian_grove, mirage_mirror, overflowing_basin,
    temple_of_mystery, chromatic_lantern, oblivion_stone, scute_swarm, ulvenwald_hydra,
    evacuation, replication_technique, finale_of_revelation, mana_reflection,
    uro, curse_of_the_swine, vesuva, dark_depths, thespians_stage,
    avenger_of_zendikar, seers_sundial, rampaging_baloths, fog_bank, urzas_power_plant,
    urzas_tower, urzas_mine, satyr_wayfinder, arcane_denial, simic_guildgate, treasure_cruise,
    sol_ring, basilisk_gate, beast_within, tatyova_benthic_druid, propaganda,
    simic_growth_chamber, floriferous_vinewall, arcane_signet, desert_of_the_indomitable,
    hashep_oasis, expedition_map, hidden_cataract, hidden_nursery, volatile_fault,
    growth_spiral, elvish_rejuvenator, urban_evolution, harmonize, thornwood_falls, cloudpost,
    sylvan_scrying, acidic_slime, pongify, lush_oasis, skullwinder, desert_of_the_mindful,
    command_tower, poison_dart_frog, glimmerpost, eureka_moment, quandrix_campus,
    // Basics: 3 island, 4 forest
    island, island, island, forest, forest, forest, forest,
];

pub const YUMA_COMMANDERS: &[CardFactory] = &[yuma_proud_protector];

/// **Desert Bloom**, the Outlaws of Thunder Junction Commander deck (OTC,
/// 2024-04-19), exactly as MTGJSON's `DesertBloom_OTC` prints it: 82
/// nonbasic cards + 6 Plainss + 4 Mountains + 7 Forests = 99. Naya Deserts and lands-matter
/// under Yuma, Proud Protector.
pub const YUMA_MAIN: &[CardFactory] = &[
    kirri_talented_sprout, scavenger_grounds, sun_titan, omnath_locus_of_rage,
    descend_upon_the_sinful, chromatic_lantern, marshals_anthem, sheltered_thicket, scute_swarm,
    hour_of_promise, oracle_of_mul_daya, ramunap_excavator, scattered_groves, world_shaper,
    nesting_dragon, turntimber_sower, sevinnes_reclamation, ancient_greenwarden,
    titania_protector_of_argoth, return_of_the_wildspeaker, perennial_behemoth,
    avenger_of_zendikar, hazezon_shaper_of_sand, escape_to_the_wilds, heaven_earth, genesis_hydra,
    sunscorched_divide, the_mending_of_dominaria, decimate, sand_scout, embrace_the_unknown,
    dune_chanter, cataclysmic_prospecting, vengeful_regrowth, angel_of_indemnity,
    cactus_preserve, rumbleweed, terramorphic_expanse, evolving_wilds, swiftfoot_boots, explore,
    sol_ring, satyr_wayfinder, perpetual_timepiece, crawling_sensation, painted_bluffs,
    command_tower, magmatic_insight, krosan_verge, desert_of_the_true, skullwinder,
    desert_of_the_indomitable, jungle_shrine, bitter_reunion, desert_of_the_fervent,
    valorous_stance, dunes_of_the_dead, shefet_dunes, hashep_oasis, elvish_rejuvenator,
    winding_way, springbloom_druid, arcane_signet, unholy_heat, thrilling_discovery,
    electric_revelation, eccentric_farmer, harrow, ramunap_ruins, path_to_exile,
    requisition_raid, bovine_intervention, map_the_frontier, conduit_pylons, mirage_mesa,
    wreck_and_rebuild, angel_of_the_ruins, bristling_backwoods, creosote_heath, abraded_bluffs,
    scaretiller, nantuko_cultivator,
    // Basics: 6 plains, 4 mountain, 7 forest
    plains, plains, plains, plains, plains, plains, mountain, mountain, mountain, mountain,
    forest, forest, forest, forest, forest, forest, forest,
];

pub const MILLICENT_COMMANDERS: &[CardFactory] = &[millicent_restless_revenant];

/// **Spirit Squadron**, the Innistrad: Crimson Vow Commander deck (VOC,
/// 2021-11-19), exactly as MTGJSON's `SpiritSquadron_VOC` prints it:
/// 76 nonbasic cards + 12 Plains + 11 Islands = 99. Azorius Spirits under
/// Millicent, Restless Revenant.
pub const MILLICENT_MAIN: &[CardFactory] = &[
    dovin_grand_arbiter, donal_herald_of_wings, rhoda_geist_avenger, timin_youthful_geist,
    drogskol_reinforcements, priest_of_the_blessed_graf, ethereal_investigator,
    spectral_arcanist, angel_of_flight_alabaster, boreas_charger, bygone_bishop,
    custodi_soulbinders, hallowed_spiritkeeper, hanged_executioner, karmic_guide,
    knight_of_the_white_orchid, mentor_of_the_meek, mirror_entity, oyobi_who_split_the_heavens,
    remorseful_cleric, twilight_drover, windborn_muse, ghostly_pilferer,
    kami_of_the_crescent_moon, rattlechains, shacklegeist, supreme_phantom,
    geist_of_saint_traft, custodi_squire, spectral_shepherd, nebelgast_herald,
    sire_of_the_storm, spectral_sailor, drogskol_captain, storm_of_souls, haunting_imitation,
    fell_the_mighty, kirtars_wrath, flood_of_tears, distant_melody, sudden_salvation,
    occult_epiphany, disorder_in_the_court, benevolent_offering, crush_contraband,
    swords_to_plowshares, arcane_denial, midnight_clock, arcane_signet, azorius_locket,
    azorius_signet, commanders_sphere, marble_diamond, sky_diamond, sol_ring, haunted_library,
    breath_of_the_sleepless, promise_of_bunrei, imprisoned_in_the_moon, verity_circle,
    darksteel_mutation, field_of_souls, ghostly_prison, reconnaissance_mission, exotic_orchard,
    moorland_haunt, port_town, prairie_stream, skycloud_expanse, temple_of_enlightenment,
    azorius_chancery, command_tower, myriad_landscape, path_of_ancestry,
    temple_of_the_false_god, unclaimed_territory,
    // Basics: 12 plains, 11 island
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, island, island, island, island, island, island, island, island, island, island,
    island,
];

pub const HENZIE_COMMANDERS: &[CardFactory] = &[henzie_toolbox_torre];

/// **Riveteers Rampage**, the Streets of New Capenna Commander deck (NCC,
/// 2022-04-29), exactly as MTGJSON's `RiveteersRampage_NCC` prints it:
/// 84 nonbasic cards + 4 Swamps + 5 Mountains + 6 Forests = 99. Jund big creatures and blitz under
/// Henzie "Toolbox" Torre.
pub const HENZIE_MAIN: &[CardFactory] = &[
    the_beamtown_bullies, jolene_the_plunder_queen, caldaia_guardian, mezzio_mugger,
    wave_of_rats, weathered_sentinels, bellowing_mauler, grime_gorger, first_responder,
    deathbringer_regent, disciple_of_bolas, noxious_gearhulk, etali_primal_storm, inferno_titan,
    stalking_vengeance, avenger_of_zendikar, giant_adephage, greenwarden_of_murasa,
    mitotic_slime, thragtusk, treeshaker_chimera, woodfall_primus, world_shaper,
    kresh_the_bloodbraided, solemn_simulacrum, artisan_of_kozilek, indrik_stomphowler,
    overgrown_battlement, temur_sabertooth, riveteers_confluence, aether_snap, painful_truths,
    blasphemous_act, lifes_legacy, victimize, explore, farseek, kodamas_reach, migration_path,
    rampant_growth, chaos_warp, riveteers_charm, windgraces_judgment, terminate, arcane_signet,
    commanders_sphere, fellwar_stone, sol_ring, dodgy_jalopy, glittering_stockpile,
    lifecrafters_bestiary, protection_racket, industrial_advancement, rain_of_riches, turf_war,
    next_of_kin, evolutionary_leap, warstorm_surge, garruks_uprising, deathreap_ritual,
    cinder_glade, exotic_orchard, foreboding_ruins, game_trail, kessig_wolf_run,
    mossfire_valley, mosswort_bridge, shadowblood_ridge, smoldering_marsh, spinerock_knoll,
    temple_of_malady, twilight_mire, ash_barrens, blighted_woodland, command_tower,
    jund_panorama, myriad_landscape, path_of_ancestry, riveteers_overlook, savage_lands,
    temple_of_the_false_god, thriving_bluff, thriving_grove, thriving_moor,
    // Basics: 4 swamp, 5 mountain, 6 forest
    swamp, swamp, swamp, swamp, mountain, mountain, mountain, mountain, mountain, forest,
    forest, forest, forest, forest, forest,
];

pub const OLORO_COMMANDERS: &[CardFactory] = &[oloro_ageless_ascetic];

/// **Eternal Bargain**, the Commander 2013 Esper deck (C13, 2013-11-01),
/// exactly as MTGJSON's `EternalBargain_C13` prints it: 75 nonbasic cards +
/// 9 Plains + 6 Islands + 9 Swamps = 99. Esper lifegain and artifacts under Oloro,
/// Ageless Ascetic, whose upkeep life works from the command zone.
pub const OLORO_MAIN: &[CardFactory] = &[
    ajanis_pridemate, augury_adept, azorius_herald, disciple_of_griselbrand, diviner_spirit,
    divinity_of_pride, filigree_angel, hooded_horror, kongming_sleeping_dragon, marrow_bats,
    myr_battlesphere, phyrexian_delver, phyrexian_gargantua, raven_familiar, razor_hippogriff,
    serene_master, serra_avatar, sharding_sphinx, sharuum_the_hegemon, sphinx_of_the_steel_wind,
    stormscape_battlemage, sydri_galvanic_genius, tidal_force, tidehollow_strix, tower_gargoyle,
    vizkopa_guildmage, wall_of_reverence, brilliant_plan, death_grasp, deep_analysis, famine,
    order_of_succession, survival_cache, tempt_with_immortality, toxic_deluge, dromars_charm,
    lim_duls_vault, reckless_spite, spinal_embrace, crawlspace, nevinyrrals_disk,
    nihil_spellbomb, obelisk_of_esper, pristine_talisman, sol_ring, sun_droplet,
    swiftfoot_boots, thopter_foundry, well_of_lost_dreams, act_of_authority, cradle_of_vitality,
    curse_of_inertia, curse_of_shallow_graves, curse_of_the_forsaken, darksteel_mutation, greed,
    phyrexian_reclamation, sanguine_bond, arcane_sanctum, azorius_chancery, azorius_guildgate,
    barren_moor, command_tower, dimir_guildgate, esper_panorama, evolving_wilds,
    jwar_isle_refuge, lonely_sandbar, opal_palace, orzhov_basilica, orzhov_guildgate,
    rupture_spire, springjack_pasture, temple_of_the_false_god, transguild_promenade,
    // Basics: 9 plains, 6 island, 9 swamp
    plains, plains, plains, plains, plains, plains, plains, plains, plains, island, island,
    island, island, island, island, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp,
];

pub const QUINTORIUS_COMMANDERS: &[CardFactory] = &[quintorius_history_chaser];

/// **Lorehold Spirit**, the Secrets of Strixhaven Commander deck (SOC,
/// 2026-04-24), exactly as MTGJSON's `LoreholdSpirit_SOC` prints it: 82
/// nonbasic cards + 11 Plainss + 6 Mountains = 99. Boros Spirits and graveyard
/// departures under the planeswalker Quintorius, History Chaser.
pub const QUINTORIUS_MAIN: &[CardFactory] = &[
    excava_the_risen_past, lorehold_archivist, augusta_order_returned, ceaseless_conflict,
    vanguard_of_the_restless, advanced_reconstruction, fateful_tempest, naktamun_lorespinner,
    relic_retriever, spirit_of_resilience, turbulent_steppe, moonshaker_cavalry,
    staff_of_the_storyteller, wave_of_reckoning, fabled_passage, angel_of_indemnity,
    ao_the_dawn_sky, archaeomancers_map, claim_jumper, drumbellower, guardian_of_faith,
    guardian_scalelord, karmic_guide, monologue_tax, remorseful_cleric, selfless_spirit,
    serra_paragon, sevinnes_reclamation, skyclave_apparition, sun_titan, tocasias_welcome,
    tragic_arrogance, white_orchid_phantom, atsushi_the_blazing_sky, conspiracy_theorist,
    laelia_the_blade_reforged, balefire_liege, hofri_ghostforge, quintorius_loremaster,
    venerable_warsinger, bitterthorn_nissas_animus, currency_converter, battlefield_forge,
    clifftop_retreat, emeria_the_sky_ruin, exotic_orchard, furycalm_snarl, glittering_massif,
    lotus_field, radiant_summit, rugged_prairie, sunscorched_divide, temple_of_triumph,
    arcane_signet, sol_ring, command_tower, primary_research, seize_the_spoils,
    kirol_history_buff, lorehold_charm, fields_of_strife, terramorphic_expanse,
    kami_of_ancient_law, path_to_exile, secret_rendezvous, swords_to_plowshares,
    teshar_ancestors_apostle, anger, faithless_looting, squee_goblin_nabob,
    quintorius_field_historian, rip_apart, containment_construct, fellwar_stone, millikin,
    mind_stone, patchwork_banner, perpetual_timepiece, lorehold_campus, mistveil_plains,
    sacred_peaks, study_hall,
    // Basics: 11 plains, 6 mountain
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    mountain, mountain, mountain, mountain, mountain, mountain,
];

pub const GAVI_COMMANDERS: &[CardFactory] = &[gavi_nest_warden];

/// **Timeless Wisdom**, the Ikoria Commander deck (C20, 2020-04-17), exactly
/// as MTGJSON's `TimelessWisdom_C20` prints it: 84 nonbasic cards +
/// 5 Islands + 6 Mountains + 4 Plains = 99. Jeskai cycling under Gavi, Nest Warden.
pub const GAVI_MAIN: &[CardFactory] = &[
    chandra_flamecaller, akim_the_soaring_wind, brallin_skyshark_rider, shabraz_the_skyshark,
    cryptic_trilobite, herald_of_the_forgotten, ethereal_forager, agitator_ant, spellpyre_phoenix,
    surly_badgersaur, eternal_dragon, sun_titan, curator_of_mysteries, nimble_obstructionist,
    portal_mage, isperia_supreme_judge, mercurial_chemister, niv_mizzet_the_firemind,
    the_locust_god, psychosis_crawler, valiant_rescuer, rooting_moloch, savai_thundermane,
    vizier_of_tumbling_sands, dismantling_wave, akromas_vengeance, decree_of_justice,
    descend_upon_the_sinful, boon_of_the_wish_giver, windfall, slice_and_dice, migratory_route,
    fierce_guardianship, neutralize, zenith_flare, hieroglyphic_illumination, abandoned_sarcophagus,
    fluctuator, bonders_ornament, arcane_signet, azorius_signet, boros_signet, commanders_sphere,
    izzet_signet, sol_ring, raugrin_crystal, crystalline_resonance, astral_drift,
    hoofprints_of_the_stag, drake_haven, new_perspectives, tectonic_reformation, ominous_seas,
    reconnaissance_mission, martial_impetus, psychic_impetus, shiny_impetus, cast_out, spirit_cairn,
    lightning_rift, exotic_orchard, hostile_desert, irrigated_farmland, prairie_stream, shivan_reef,
    skycloud_expanse, ash_barrens, azorius_chancery, boros_garrison, command_tower,
    desert_of_the_fervent, desert_of_the_mindful, desert_of_the_true, drifting_meadow,
    forgotten_cave, izzet_boilerworks, lonely_sandbar, myriad_landscape, mystic_monastery,
    reliquary_tower, remote_isle, secluded_steppe, smoldering_crater, temple_of_the_false_god,
    // Basics: 5 island, 6 mountain, 4 plains
    island, island, island, island, island, mountain, mountain, mountain, mountain, mountain,
    mountain, plains, plains, plains, plains,
];

pub const GONTI_COMMANDERS: &[CardFactory] = &[gonti_canny_acquisitor];

/// **Grand Larceny**, the Outlaws of Thunder Junction Commander deck (OTC,
/// 2024-04-19), exactly as MTGJSON's `GrandLarceny_OTC` prints it: 84
/// nonbasic cards + 5 Islands + 6 Swamps + 4 Forests = 99. Sultai theft under Gonti, Canny
/// Acquisitor.
pub const GONTI_MAIN: &[CardFactory] = &[
    felix_five_boots, thieving_skydiver, sage_of_the_beyond, ghostly_pilferer,
    diluvian_primordial, stolen_goods, curse_of_the_swine, dazzling_sphinx, arcane_heist,
    smirking_spelljacker, minds_dilation, gonti_lord_of_luxury, predators_hour,
    brainstealer_dragon, cunning_rhetoric, thieving_amalgam, orochi_soul_reaver,
    thieving_varmint, heartless_conscription, nashi_moon_sages_scion, baleful_mastery,
    ohran_frostfang, savvy_trader, tower_winder, cazur_ruthless_stalker, silent_blade_oni,
    hostage_taker, edric_spymaster_of_trest, fallen_shinobi, cold_eyed_selkie, siphon_insight,
    extract_brain, shadowmage_infiltrator, baleful_strix, culling_ritual, thief_of_sanity,
    villainous_wealth, the_mimeoplasm, plasm_capture, ukkima_stalking_shadow,
    bladegriff_prototype, chaos_wand, dream_thiefs_bandana, oblivion_sower, overflowing_basin,
    drowned_catacomb, temple_of_deceit, temple_of_malady, darkwater_catacombs, fetid_pools,
    yavimaya_coast, viridescent_bog, exotic_orchard, underground_river, woodland_cemetery,
    sunken_hollow, temple_of_mystery, hinterland_harbor, darkslick_shores, flooded_grove,
    twilight_mire, llanowar_wastes, slither_blade, whirler_rogue, triton_shorestalker,
    feed_the_swarm, silhana_ledgewalker, rampant_growth, kodamas_reach, void_attendant,
    three_visits, putrefy, trygon_predator, doc_aurlock_grizzled_genius, arcane_signet,
    darksteel_ingot, fellwar_stone, prismatic_lens, sol_ring, command_tower, reliquary_tower,
    access_tunnel, dimir_aqueduct, opulent_palace,
    // Basics: 5 island, 6 swamp, 4 forest
    island, island, island, island, island, swamp, swamp, swamp, swamp, swamp, swamp, forest,
    forest, forest, forest,
];

pub const PROSSH_COMMANDERS: &[CardFactory] = &[prossh_skyraider_of_kher];

/// **Power Hungry**, the Commander 2013 Jund deck (C13, 2013-11-01), exactly
/// as MTGJSON's `PowerHungry_C13` prints it: 79 nonbasic cards + 6 Swamps +
/// 7 Mountains + 7 Forests = 99. Jund tokens and sacrifice under Prossh, Skyraider
/// of Kher.
pub const PROSSH_MAIN: &[CardFactory] = &[
    brooding_saurian, capricious_efreet, charnelhoard_wurm, deathbringer_thoctar,
    deepfire_elemental, elvish_skysweeper, endless_cockroaches, endrek_sahr_master_breeder,
    fell_shepherd, goblin_sharpshooter, golgari_guildmage, hooded_horror,
    hua_tuo_honored_physician, hunted_troll, inferno_titan, jade_mage, ophiomancer,
    quagmire_druid, sakura_tribe_elder, scarland_thrinax, sekkuar_deathkeeper,
    shattergang_brothers, silklash_spider, sprouting_thrinax, stalking_vengeance,
    stronghold_assassin, terra_ravager, viscera_seer, walker_of_the_grove,
    wight_of_precinct_six, dirge_of_dread, mass_mutiny, restore, rough_tumble, spoils_of_victory,
    sudden_demise, tempt_with_vengeance, jund_charm, reincarnation, armillary_sphere,
    carnage_altar, jar_of_eyeballs, obelisk_of_jund, plague_boiler, sol_ring, spine_of_ish_sah,
    swiftfoot_boots, blood_rites, curse_of_chaos, curse_of_predation, curse_of_shallow_graves,
    fecundity, foster, furnace_celebration, goblin_bombardment, night_soil, primal_vigor,
    tooth_and_claw, vile_requiem, widespread_panic, akoum_refuge, command_tower, evolving_wilds,
    golgari_guildgate, golgari_rot_farm, grim_backwoods, gruul_guildgate, jund_panorama,
    kazandu_refuge, khalni_garden, kher_keep, llanowar_reborn, opal_palace, rakdos_guildgate,
    rupture_spire, savage_lands, temple_of_the_false_god, terramorphic_expanse, vivid_grove,
    // Basics: 6 swamp, 7 mountain, 7 forest
    swamp, swamp, swamp, swamp, swamp, swamp, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, forest, forest, forest, forest, forest, forest, forest,
];

pub const MOROPHON_COMMANDERS: &[CardFactory] = &[morophon_the_boundless];

/// **Everyone's Invited!**, the Secret Lair Commander deck (SLD, 2025-05-12),
/// exactly as MTGJSON's `EveryoneSInvited_SLD` prints it: 87 nonbasic cards
/// + 3 Forests + 3 Plains + 2 Mountains + 3 Islands + 1 Swamp = 99. Five-color
/// changelings and tribal payoffs under Morophon, the Boundless.
pub const MOROPHON_MAIN: &[CardFactory] = &[
    raise_the_palisade, bitterblossom, taurean_mauler, avenger_of_zendikar, coat_of_arms,
    kindred_summons, maskwood_nexus, sol_ring, tendershoot_dryad, adarkar_wastes,
    amoeboid_changeling, ancient_amphitheater, arcane_adaptation, arcane_denial, arcane_sanctum,
    arcane_signet, atla_palani_nest_tender, beast_within, black_market_connections,
    bloodline_pretender, brenard_ginger_sculptor, bruse_tarl_roving_rancher, brushland,
    chameleon_colossus, cloudshredder_sliver, command_tower, crib_swap, cultivate,
    darkwater_catacombs, distant_melody, double_down, exotic_orchard, fabled_passage, farseek,
    feline_sovereign, fire_belly_changeling, frontier_bivouac, gemhide_sliver, gilt_leaf_palace,
    graveshifter, guardian_gladewalker, harabaz_druid, harper_recruiter,
    impostor_of_the_sixth_pride, jungle_shrine, karplusan_forest, kindred_discovery,
    kindred_dominance, kinsbaile_cavalier, kirri_talented_sprout, kodamas_reach,
    llanowar_wastes, magda_brazen_outlaw, manaweft_sliver, masked_vandal, mirror_entity,
    moritte_of_the_frost, mossfire_valley, mothdust_changeling, murmuring_bosk, mutavault,
    nameless_inversion, opulent_palace, overflowing_basin, path_of_ancestry, pongify,
    realmbreaker_the_invasion_tree, realmwalker, rin_and_seri_inseparable, risen_reef,
    rukarumel_biologist, seaside_citadel, secluded_glen, shapesharer, shields_of_velis_vel,
    skeletal_changeling, sophia_dogged_detective, spoils_of_adventure, stick_together,
    sungrass_prairie, tazri_beacon_of_unity, the_bears_of_littjara, the_world_tree,
    universal_automaton, unsettled_mariner, wanderwine_hub, yavimaya_coast,
    // Basics: 3 forest, 3 plains, 2 mountain, 3 island, 1 swamp
    forest, forest, forest, plains, plains, plains, mountain, mountain, island, island, island, swamp,
];

pub const SEFRIS_COMMANDERS: &[CardFactory] = &[sefris_of_the_hidden_ways];

/// **Dungeons of Death**, the Adventures in the Forgotten Realms Commander
/// deck (AFC, 2021-07-23), exactly as MTGJSON's `DungeonsOfDeath_AFC` prints
/// it: 80 nonbasic cards + 7 Plains + 5 Islands + 7 Swamps = 99. Esper
/// reanimation and dungeon venturing under Sefris of the Hidden Ways.
pub const SEFRIS_MAIN: &[CardFactory] = &[
    cataclysmic_gearhulk, eternal_dragon, karmic_guide, sun_titan, sunblast_angel,
    champion_of_wits, curator_of_mysteries, phantasmal_image, doomed_necromancer, ashen_rider,
    baleful_strix, hostage_taker, solemn_simulacrum, ronom_unicorn, wall_of_omens,
    merfolk_looter, mulldrifter, murder_of_crows, plaguecrafter, reassembling_skeleton,
    shriekmaw, cloudblazer, necrotic_sliver, obsessive_stitcher, burnished_hart, meteor_golem,
    necromantic_selection, unburial_rites, victimize, utter_end, swords_to_plowshares,
    forbidden_alchemy, despark, vanish_into_memory, arcane_signet, commanders_sphere,
    fellwar_stone, sol_ring, wayfarers_bauble, lightning_greaves, propaganda, choked_estuary,
    darkwater_catacombs, exotic_orchard, geier_reach_sanitarium, high_market, nimbus_maze,
    port_town, prairie_stream, sunken_hollow, evolving_wilds, arcane_sanctum, azorius_chancery,
    command_tower, dimir_aqueduct, esper_panorama, orzhov_basilica, terramorphic_expanse,
    thriving_heath, thriving_isle, thriving_moor, nihiloor, immovable_rod, radiant_solar,
    revivify, thorough_investigation, arcane_endeavor, minn_wily_illusionist, phantom_steed,
    rod_of_absorption, grave_endeavor, wand_of_orcus, extract_brain, midnight_pathlighter,
    minimus_containment, hama_pashar_ruin_seeker, dungeon_map, bucknards_everfull_purse,
    clay_golem, component_pouch,
    // Basics: 7 plains, 5 island, 7 swamp
    plains, plains, plains, plains, plains, plains, plains, island, island, island, island,
    island, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
];

pub const ZIMONE_COMMANDERS: &[CardFactory] = &[zimone_mystery_unraveler];

/// **Jump Scare!**, the Duskmourn Commander deck (DSC, 2024-09-27), exactly
/// as MTGJSON's `JumpScare_DSC` prints it: 81 nonbasic cards + 9 Islands + 9 Forests =
/// 99. Simic manifest dread and morph under Zimone, Mystery Unraveler.
pub const ZIMONE_MAIN: &[CardFactory] = &[
    kianne_corrupted_memory, glitch_interpreter, they_came_from_the_pipes, zimones_hypothesis,
    curator_beastie, disorienting_choice, experimental_lab_staff_room, shriekwood_devourer,
    giggling_skitterspike, cackling_counterpart, citanul_hierophants, aether_gale,
    body_of_knowledge, dig_through_time, kefnet_the_mindful, kheru_spellsnatcher,
    primordial_mist, skaab_ruinator, ashaya_soul_of_the_wild, augur_of_autumn, deathmist_raptor,
    ezuris_predation, hydra_omnivore, multani_yavimayas_avatar, overwhelming_stampede,
    sandwurm_convergence, scute_swarm, shigeki_jukai_visionary, temur_war_shaman,
    thunderfoot_baloth, trail_of_mystery, whisperwood_elemental, worldspine_wurm,
    aesi_tyrant_of_gyre_strait, arixmethes_slumbering_isle, biomass_mutation, oversimplify,
    rashmi_eternities_crafter, scroll_of_fate, castle_vantress, drownyard_temple, flooded_grove,
    hinterland_harbor, mosswort_bridge, overflowing_basin, temple_of_mystery, vineglimmer_snarl,
    yavimaya_coast, overgrown_zealot, growing_dread, terramorphic_expanse, beast_within,
    growth_spiral, arcane_signet, sol_ring, command_tower, counterspell, reality_shift,
    retreat_to_coralhelm, beanstalk_giant, cultivate, explosive_vegetation, greater_tanuki,
    rampant_growth, sakura_tribe_elder, wilderness_reclamation, yavimaya_elder,
    yedora_grave_gardener, eureka_moment, tatyova_benthic_druid, trygon_predator, simic_signet,
    ash_barrens, evolving_wilds, myriad_landscape, quandrix_campus, reliquary_tower,
    simic_growth_chamber, tangled_islet, temple_of_the_false_god, thornwood_falls,
    // Basics: 9 island, 9 forest
    island, island, island, island, island, island, island, island, island, forest, forest,
    forest, forest, forest, forest, forest, forest, forest,
];

pub const MORSKA_COMMANDERS: &[CardFactory] = &[morska_undersea_sleuth];

/// **Deep Clue Sea**, the Murders at Karlov Manor Commander deck (MKC,
/// 2024-02-09), exactly as MTGJSON's `DeepClueSea_MKC` prints it: 85
/// nonbasic cards + 3 Plains + 6 Islands + 5 Forests = 99. Bant Clues and
/// second-draw payoffs under Morska, Undersea Sleuth.
pub const MORSKA_MAIN: &[CardFactory] = &[
    sophia_dogged_detective, armed_with_proof, merchant_of_truth, serene_sleuth,
    detective_of_the_month, follow_the_bodies, tangletrove_kelp, innocuous_researcher,
    on_the_trail, knowledge_is_power, ransom_note, aerial_extortionist, bennie_bracks_zoologist,
    farewell, fumigate, organic_extinction, search_the_premises, alandra_sky_dreamer,
    confirm_suspicions, ethereal_investigator, finale_of_revelation, kappa_cannoneer,
    mechanized_production, nadir_kraken, shimmer_dragon, teferis_ageless_insight,
    tezzeret_betrayer_of_flesh, thought_monitor, hornet_queen, jolrael_mwonvuli_recluse,
    killer_service, tireless_tracker, adrix_and_nev_twincasters, chulane_teller_of_tales,
    disorder_in_the_court, esix_fractal_bloom, hydroid_krasis, koma_cosmos_serpent,
    lonis_cryptozoologist, selvala_explorer_returned, academy_manufactor, idol_of_oblivion,
    inspiring_statuary, nettlecyst, psychosis_crawler, canopy_vista, exotic_orchard,
    irrigated_farmland, prairie_stream, scattered_groves, skycloud_expanse, spire_of_industry,
    sungrass_prairie, temple_of_enlightenment, temple_of_mystery, temple_of_plenty,
    swords_to_plowshares, erdwal_illuminator, junk_winder, ongoing_investigation, whirler_rogue,
    graf_mole, ulvenwald_mysteries, wilderness_reclamation, wavesifter, arcane_signet,
    azorius_signet, simic_signet, sol_ring, talisman_of_curiosity, talisman_of_progress,
    talisman_of_unity, azorius_chancery, magnifying_glass, command_tower, krosan_verge,
    lonely_sandbar, path_of_ancestry, reliquary_tower, seaside_citadel, secluded_steppe,
    selesnya_sanctuary, simic_growth_chamber, temple_of_the_false_god, tranquil_thicket,
    // Basics: 3 plains, 6 island, 5 forest
    plains, plains, plains, island, island, island, island, island, island, forest, forest, forest,
    forest, forest,
];

pub const KAUST_COMMANDERS: &[CardFactory] = &[kaust_eyes_of_the_glade];

/// **Deadly Disguise**, the Murders at Karlov Manor Commander deck
/// (2024-02-09), exactly as MTGJSON's `DeadlyDisguise_MKC` prints it: 88
/// nonbasic cards + 4 Plains + 3 Mountain + 4 Forest = 99. Naya morph,
/// disguise and cloak under Kaust, Eyes of the Glade.
pub const KAUST_MAIN: &[CardFactory] = &[
    duskana_the_rage_mother, true_identity, unexplained_absence, veiled_ascension, boltbender,
    showstopping_surprise, tesak_judiths_hellhound, experiment_twelve, printlifter_ooze,
    panoptic_projektor, ransom_note, ugins_mastery, austere_command, dusk_dawn, exalted_angel,
    fell_the_mighty, hidden_dragonslayer, master_of_pearls, mastery_of_the_unseen,
    mirror_entity, welcoming_vampire, akroma_angel_of_fury, ashcloud_phoenix, chaos_warp,
    imperial_hellkite, jeskas_will, neheb_the_eternal, scourge_of_the_throne, beast_whisperer,
    deathmist_raptor, den_protector, hooded_hydra, krosan_cloudscraper, krosan_colossus,
    obscuring_aether, ohran_frostfang, return_of_the_wildspeaker, root_elemental,
    saryth_the_vipers_fang, seedborn_muse, temur_war_shaman, thelonite_hermit,
    toski_bearer_of_secrets, trail_of_mystery, whisperwood_elemental, yedora_grave_gardener,
    decimate, sidar_kondo_of_jamuraa, lifecrafters_bestiary, scroll_of_fate, canopy_vista,
    cinder_glade, exotic_orchard, fortified_village, furycalm_snarl, game_trail,
    kessig_wolf_run, mossfire_valley, mosswort_bridge, scattered_groves, sheltered_thicket,
    shrine_of_the_forsaken_gods, sungrass_prairie, temple_of_abandon, temple_of_plenty,
    temple_of_triumph, path_to_exile, ainok_survivalist, broodhatch_nantuko, nervous_gardener,
    nantuko_vigilante, natures_lore, sakura_tribe_elder, salt_road_ambushers, three_visits,
    wild_growth, arcane_signet, sol_ring, boros_garrison, command_tower, branch_of_vitu_ghazi,
    gruul_turf, jungle_shrine, krosan_verge, sacred_peaks, selesnya_sanctuary,
    temple_of_the_false_god, zoetic_cavern,
    // Basics: 4 plains, 3 mountain, 4 forest
    plains, plains, plains, plains, mountain, mountain, mountain, forest, forest, forest,
    forest,
];

pub const NALIA_COMMANDERS: &[CardFactory] = &[nalia_dearnise];

/// **Party Time**, the Commander Legends: Battle for Baldur's Gate Commander
/// deck (CLB, 2022-06-10), exactly as MTGJSON's `PartyTime_CLB` prints it:
/// 79 nonbasic cards + 10 Plainss + 10 Swamps = 99. Orzhov parties under Nalia
/// de'Arnise.
pub const NALIA_MAIN: &[CardFactory] = &[
    archpriest_of_iona, bygone_bishop, eight_and_a_half_tails, frontline_medic, galepowder_mage,
    glorious_protector, jazal_goldmane, magus_of_the_balance, mikaeus_the_lunarch,
    mirror_entity, order_of_whiteclay, selfless_spirit, solemn_recruit, squad_commander,
    bloodsoaked_champion, butcher_of_malakir, calculating_lich, dire_fleet_ravager,
    gonti_lord_of_luxury, grim_haruspex, grim_hireling, mardu_strike_leader, mindblade_render,
    nighthawk_scavenger, pontiff_of_blight, puppeteer_clique, felisa_fang_of_silverquill,
    high_priest_of_penance, aven_mindcensor, irregular_cohort, mages_attendant, mother_of_runes,
    priest_of_ancient_lore, rumor_gatherer, valiant_changeling, changeling_outcast,
    corpse_augur, malakir_blood_priest, zulaport_cutthroat, austere_command, dusk_dawn,
    sevinnes_reclamation, thwart_the_grave, unbreakable_formation, despark, maskwood_nexus,
    arcane_signet, orzhov_signet, skullclamp, sol_ring, talisman_of_hierarchy,
    firjas_retribution, castle_locthwain, mutavault, shambling_vent, temple_of_silence,
    vault_of_the_archangel, war_room, windbrisk_heights, command_tower, ash_barrens, bojuka_bog,
    mortuary_mire, myriad_landscape, orzhov_basilica, path_of_ancestry, snowfield_sinkhole,
    starlit_sanctum, tainted_field, burakos_party_leader, folk_hero, deep_gnome_terramancer,
    harper_recruiter, seasoned_dungeoneer, stick_together, black_market_connections,
    solemn_doomguide, multiclass_baldric, crib_swap,
    // Basics: 10 plains, 10 swamp
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, swamp,
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
];

pub const ZIMONE_IA_COMMANDERS: &[CardFactory] = &[zimone_infinite_analyst];

/// **Quandrix Unlimited**, the Secrets of Strixhaven Commander deck
/// (2026-04-24), exactly as MTGJSON's `QuandrixUnlimited_SOC` prints it: 86
/// nonbasic cards + 7 Island + 6 Forest = 99. Simic X spells and +1/+1
/// counters under Zimone, Infinite Analyst.
pub const ZIMONE_IA_MAIN: &[CardFactory] = &[
    primo_the_unbounded, owlin_spiralmancer, expansion_algorithm, nexus_mentality, kinetic_ooze,
    lattice_library, nev_the_practical_dean, yavimaya_bloomsage, striding_shotcaller,
    brass_infiniscope, turbulent_wilderness, commanders_insight, ingenious_prodigy,
    pull_from_tomorrow, benevolent_hydra, unbound_flourishing, fabled_passage,
    curse_of_the_swine, deekah_fractal_theorist, entrancing_melody, perplexing_test,
    stroke_of_genius, zimones_hypothesis, animists_awakening, forgotten_ancient,
    fractal_harness, goldvein_hydra, guardian_augmenter, hardened_scales, lifeblood_hydra,
    mana_bloom, open_the_way, ozolith_the_shattered_spire, primal_might, primordial_hydra,
    silkguard, steelbane_hydra, altered_ego, biomass_mutation, elusive_otter, the_goose_mother,
    hydroid_krasis, oversimplify, quandrix_command, tanazir_quandrix, zimone_all_questioning,
    astral_cornucopia, elementalists_palette, hangarback_walker, stonecoil_serpent,
    alchemists_refuge, exotic_orchard, flooded_grove, hinterland_harbor, oran_rief_the_vastwood,
    overflowing_basin, rain_slicked_copse, sodden_verdure, temple_of_mystery, vineglimmer_snarl,
    yavimaya_coast, arcane_signet, sol_ring, command_tower, quandrix_charm, paradox_gardens,
    terramorphic_expanse, rapid_hybridization, beast_within, kami_of_whispered_hopes,
    natures_lore, three_visits, tyvars_stand, decisive_denial, eureka_moment,
    quandrix_apprentice, troyan_gutsy_explorer, zimone_quandrix_prodigy, opal_palace,
    path_of_ancestry, quandrix_campus, reliquary_tower, rogues_passage, study_hall,
    tangled_islet, temple_of_the_false_god,
    // Basics: 7 island, 6 forest
    island, island, island, island, island, island, island, forest, forest, forest, forest,
    forest, forest,
];

pub const SAHEELI_RADIANT_COMMANDERS: &[CardFactory] = &[saheeli_radiant_creator];

/// **Living Energy**, the Aetherdrift Commander deck (DRC, 2025-02-14),
/// exactly as MTGJSON's `LivingEnergy_DRC` prints it: 83 nonbasic cards +
/// 5 Islands + 5 Mountains + 6 Forests = 99. Temur energy and
/// artifacts under Saheeli, Radiant Creator.
pub const SAHEELI_RADIANT_MAIN: &[CardFactory] = &[
    pia_nalaar_chief_mechanic, territorial_aetherkite, nissa_worldsoul_speaker,
    peema_trailblazer, rampaging_aetherhood, adaptive_omnitool, aetherflux_conduit,
    aetheric_amplifier, stridehangar_automaton, chain_reaction, chaos_warp,
    druid_of_purification, chromatic_lantern, duplicant, academy_ruins, aethersquall_ancient,
    aethertide_whale, confiscation_coup, disallow, midnight_clock, one_with_the_machine,
    sai_master_thopterist, thopter_spy_network, blasphemous_act, combustible_gearhulk,
    lightning_runner, pia_and_kiran_nalaar, aetherwind_basker, architect_of_the_untamed,
    bootleggers_stash, elder_gargaroth, aetherworks_marvel, conjurers_closet,
    cultivators_caravan, panharmonicon, retrofitter_foundry, solemn_simulacrum,
    triplicate_titan, exotic_orchard, frostboil_snarl, hinterland_harbor, karplusan_forest,
    overflowing_basin, rootbound_crag, sheltered_thicket, shivan_reef, spire_of_industry,
    sulfur_falls, temple_of_epiphany, treasure_vault, vineglimmer_snarl, yavimaya_coast,
    reality_shift, arcane_signet, lightning_greaves, sol_ring, command_tower, arcane_denial,
    bespoke_battlewagon, era_of_innovation, glimmer_of_genius, whirler_rogue, loyal_apprentice,
    reckless_fireweaver, attune_with_aether, explosive_vegetation, peema_aether_seer,
    servant_of_the_conduit, rogue_refiner, saheeli_sublime_artificer, whirler_virtuoso,
    commanders_sphere, decoction_module, ornithopter_of_paradise, solar_transformer,
    soul_guide_lantern, talisman_of_curiosity, aether_hub, evolving_wilds, frontier_bivouac,
    path_of_ancestry, slagwoods_bridge, tanglepool_bridge,
    // Basics: 5 island, 5 mountain, 6 forest
    island, island, island, island, island, mountain, mountain, mountain, mountain, mountain,
    forest, forest, forest, forest, forest, forest,
];

pub const INSPIRIT_COMMANDERS: &[CardFactory] = &[inspirit_flagship_vessel];

/// **Counter Intelligence**, the Edge of Eternities Commander deck (EOC,
/// 2025-08-01), exactly as MTGJSON's `CounterIntelligence_EOC` prints it: 90
/// nonbasic cards + 3 Plains + 3 Islands + 3 Mountains = 99. Jeskai artifacts
/// and counters under the Spacecraft commander Inspirit, Flagship Vessel.
pub const INSPIRIT_MAIN: &[CardFactory] = &[
    kilo_apogee_mind, patrolling_peacemaker, insight_engine, uthros_research_craft,
    depthshaker_titan, long_range_sensor, moxite_refinery, solar_array, surge_conductor,
    glittering_massif, radiant_summit, swan_song, chaos_warp, cloud_key, gavel_of_the_righteous,
    battlefield_forge, angel_of_the_ruins, fumigate, organic_extinction, resourceful_defense,
    chrome_host_seedshark, cyberdrive_awakener, deepglow_skate, emry_lurker_of_the_loch,
    kappa_cannoneer, phyrexian_metamorph, pull_from_tomorrow, ripples_of_potential,
    tekuthal_inquiry_dominus, thought_monitor, universal_surveillance, chain_reaction,
    alibou_ancient_witness, jhoira_weatherlight_captain, wake_the_past, astral_cornucopia,
    crystalline_crawler, darksteel_reactor, empowered_autogenerator, hangarback_walker,
    lux_artillery, lux_cannon, steel_overseer, threefold_thunderhulk, titan_forge,
    adarkar_wastes, cascade_bluffs, clifftop_retreat, exotic_orchard, glacial_fortress,
    irrigated_farmland, karns_bastion, the_mycosynth_gardens, rugged_prairie, shivan_reef,
    skycloud_expanse, spire_of_industry, sulfur_falls, temple_of_enlightenment,
    temple_of_epiphany, temple_of_triumph, swords_to_plowshares, tezzerets_gambit,
    thirst_for_knowledge, arcane_signet, pentad_prism, sol_ring, command_tower, dispatch,
    etherium_sculptor, experimental_augury, thrummingbird, enthusiastic_mechanaut, coretapper,
    etched_oracle, everflowing_chalice, golem_foundry, mindless_automaton, soul_guide_lantern,
    ancient_den, buried_ruin, evolving_wilds, great_furnace, lonely_sandbar, mystic_monastery,
    razortide_bridge, rustvale_bridge, seat_of_the_synod, secluded_steppe, silverbluff_bridge,
    // Basics: 3 plains, 3 island, 3 mountain
    plains, plains, plains, island, island, island, mountain, mountain, mountain,
];

pub const ANHELO_COMMANDERS: &[CardFactory] = &[anhelo_the_painter];

/// **Maestros Massacre**, the Streets of New Capenna Commander deck
/// (2022-04-29), exactly as MTGJSON's `MaestrosMassacre_NCC` prints it: 81
/// nonbasic cards + 7 Island + 6 Swamp + 5 Mountain = 99. Grixis spell-
/// copying and casualty under Anhelo, the Painter.
pub const ANHELO_MAIN: &[CardFactory] = &[
    parnesse_the_subtle_brush, bloodsoaked_champion, cormela_glamour_thief, dogged_detective,
    goblin_electromancer, kess_dissident_mage, puppeteer_clique, rekindling_phoenix,
    spellbinding_soprano, squee_the_immortal, sinister_concierge, syrix_carrier_of_the_flame,
    skyclave_shade, woe_strider, army_of_the_damned, bedevil, call_the_skybreaker,
    chain_reaction, clone_legion, damnable_pact, deep_analysis, drawn_from_dreams,
    dread_summons, feed_the_swarm, flawless_forgery, hex, maestros_confluence, make_an_example,
    ponder, preordain, reign_of_the_pit, rivers_rebuke, sever_the_bloodline,
    talrands_invocation, xanders_pact, zndrsplts_judgment, a_little_chat, dig_through_time,
    fact_or_fiction, frantic_search, maestros_charm, mystic_confluence, waste_management,
    arcane_signet, audacious_swap, body_count, commanders_sphere, dimir_signet, fellwar_stone,
    izzet_signet, lightning_greaves, rakdos_signet, sol_ring, mimic_vat, smugglers_buggy,
    twinning_staff, wayfarers_bauble, cryptic_pursuit, determined_iteration, double_vision,
    extravagant_replication, rite_of_the_raging_storm, maestros_theater, ash_barrens,
    command_tower, crumbling_necropolis, grixis_panorama, myriad_landscape, path_of_ancestry,
    thriving_bluff, thriving_isle, thriving_moor, cascade_bluffs, choked_estuary,
    darkwater_catacombs, exotic_orchard, foreboding_ruins, shadowblood_ridge, smoldering_marsh,
    sunken_hollow, temple_of_epiphany,
    // Basics: 7 island, 6 swamp, 5 mountain
    island, island, island, island, island, island, island, swamp, swamp, swamp, swamp, swamp,
    swamp, mountain, mountain, mountain, mountain, mountain,
];

pub const PROSPER_COMMANDERS: &[CardFactory] = &[prosper_tome_bound];

/// **Planar Portal**, the Adventures in the Forgotten Realms Commander deck
/// (AFC, 2021-07-23), exactly as MTGJSON's `PlanarPortal_AFC` prints it: 72
/// nonbasic cards + 14 Swamps + 13 Mountains = 99. Rakdos exile-and-play
/// under Prosper, Tome-Bound.
pub const PROSPER_MAIN: &[CardFactory] = &[
    chittering_witch, fiend_of_the_shadows, gonti_lord_of_luxury, marionette_master,
    ogre_slumlord, piper_of_the_swarm, pontiff_of_blight, dark_dweller_oracle,
    dire_fleet_daredevil, dream_pillager, etali_primal_storm, izzet_chemister, tectonic_giant,
    loyal_apprentice, consuming_vapors, hex, apex_of_power, disrupt_decorum, ignite_the_future,
    phthisis, light_up_the_stage, throes_of_chaos, vandalblast, chaos_warp, commune_with_lava,
    bedevil, bituminous_blast, rakdos_charm, terminate, chaos_wand, arcane_signet,
    commanders_sphere, fellwar_stone, mind_stone, orazca_relic, rakdos_signet, sol_ring,
    talisman_of_indulgence, unstable_obelisk, dead_mans_chest, theater_of_horrors,
    shiny_impetus, exotic_orchard, foreboding_ruins, shadowblood_ridge, smoldering_marsh,
    spinerock_knoll, bojuka_bog, command_tower, mortuary_mire, rakdos_carnarium, tainted_peak,
    zhalfirin_void, karazikar_the_eye_tyrant, bag_of_devouring, danse_macabre, death_tyrant,
    grim_hireling, hellish_rebuke, lorcan_warlock_collector, fiendlash, reckless_endeavor,
    share_the_spoils, wild_magic_sorcerer, fevered_suspicion, hurl_through_hell, warlock_class,
    chaos_channeler, you_find_some_prisoners, bucknards_everfull_purse, ebony_fly,
    underdark_rift,
    // Basics: 14 swamp, 13 mountain
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain, mountain, mountain,
];

pub const KADENA_COMMANDERS: &[CardFactory] = &[kadena_slinking_sorcerer];

/// **Faceless Menace**, the Commander 2019 deck (C19, 2019-08-23), exactly as
/// MTGJSON's `FacelessMenace_C19` prints it: 84 nonbasic cards +
/// 5 Islands + 3 Swamps + 7 Forests = 99. Sultai morph under Kadena, Slinking Sorcerer.
pub const KADENA_MAIN: &[CardFactory] = &[
    vraska_the_unseen, rayami_first_of_the_fallen, volrath_the_shapestealer, kadenas_silencer,
    thought_sponge, thieving_amalgam, apex_altisaur, grismold_the_dreadsower, deathmist_raptor,
    hooded_hydra, chromeshell_crab, ixidron, kheru_spellsnatcher, stratus_dancer,
    thousand_winds, vesuvan_shapeshifter, bane_of_the_living, grim_haruspex, silumgar_assassin,
    den_protector, seedborn_muse, thelonite_hermit, sagu_mauler, voice_of_many, scaretiller,
    willbender, skinthinner, ainok_survivalist, great_oak_guardian, nantuko_vigilante,
    sakura_tribe_elder, icefeather_aven, road_of_return, ghastly_conscription, hex,
    overwhelming_stampede, tempt_with_discovery, mire_in_misery, tezzerets_gambit, cultivate,
    explore, farseek, urban_evolution, sudden_substitution, biomass_mutation, leadership_vacuum,
    echoing_truth, reality_shift, putrefy, sultai_charm, pendant_of_prosperity, scroll_of_fate,
    strionic_resonator, sol_ring, thran_dynamo, gift_of_doom, trail_of_mystery,
    bounty_of_the_luxa, secret_plans, darkwater_catacombs, exotic_orchard, llanowar_wastes,
    shrine_of_the_forsaken_gods, sunken_hollow, thespians_stage, yavimaya_coast, ash_barrens,
    bojuka_bog, command_tower, dimir_aqueduct, evolving_wilds, foul_orchard, golgari_guildgate,
    golgari_rot_farm, jungle_hollow, myriad_landscape, opulent_palace, reliquary_tower,
    simic_growth_chamber, simic_guildgate, temple_of_the_false_god, terramorphic_expanse,
    thornwood_falls, woodland_stream,
    // Basics: 5 island, 3 swamp, 7 forest
    island, island, island, island, island, swamp, swamp, swamp, forest, forest, forest, forest,
    forest, forest, forest,
];

pub const KAMIZ_COMMANDERS: &[CardFactory] = &[kamiz_obscura_oculus];

/// **Obscura Operation**, the Streets of New Capenna Commander deck (NCC,
/// 2022-04-29), exactly as MTGJSON's `ObscuraOperation_NCC` prints it:
/// 82 nonbasic cards + 5 Plains + 6 Islands + 6 Swamps = 99. Esper connive and evasion under
/// Kamiz, Obscura Oculus.
pub const KAMIZ_MAIN: &[CardFactory] = &[
    tivit_seller_of_secrets, aerial_extortionist, cephalid_facetaker, skyway_robber,
    misfortune_teller, oskar_rubbish_reclaimer, silent_blade_oni, wrexial_the_risen_deep,
    alela_artful_provocateur, fallen_shinobi, drana_liberator_of_malakir, archon_of_coronation,
    sun_titan, champion_of_wits, chasm_skulker, ghostly_pilferer, identity_thief, nadir_kraken,
    custodi_lich, graveblade_marauder, daxos_of_meletis, dragonlord_ojutai,
    shadowmage_infiltrator, thief_of_sanity, daring_saboteur, looter_il_kor, whirler_rogue,
    inkfathom_witch, jailbreak, writ_of_return, austere_command, dusk_dawn, stolen_identity,
    nightmare_unmaking, profane_command, treasure_cruise, an_offer_you_cant_refuse,
    change_of_plans, lethal_scheme, obscura_charm, obscura_confluence, commit_memory, utter_end,
    swords_to_plowshares, in_too_deep, life_insurance, smugglers_share, arcane_signet,
    azorius_signet, commanders_sphere, currency_converter, dimir_signet, fellwar_stone,
    mask_of_riddles, mask_of_the_schemer, orzhov_signet, quietus_spike, strionic_resonator,
    sol_ring, swiftfoot_boots, wayfarers_bauble, fetid_heath, creeping_tar_pit, sunken_hollow,
    choked_estuary, arcane_sanctum, darkwater_catacombs, exotic_orchard, port_town,
    prairie_stream, skycloud_expanse, temple_of_silence, ash_barrens, command_tower,
    esper_panorama, myriad_landscape, obscura_storefront, path_of_ancestry, rogues_passage,
    thriving_heath, thriving_isle, thriving_moor,
    // Basics: 5 plains, 6 island, 6 swamp
    plains, plains, plains, plains, plains, island, island, island, island, island, island,
    swamp, swamp, swamp, swamp, swamp, swamp,
];

pub const SATYA_COMMANDERS: &[CardFactory] = &[satya_aetherflux_genius];

/// **Creative Energy**, the Modern Horizons 3 Commander deck (2024-06-14),
/// exactly as MTGJSON's `CreativeEnergy_M3C` prints it: 79 nonbasic cards +
/// 10 Plains + 5 Island + 5 Mountain = 99. Jeskai energy and token copies
/// under Satya, Aetherflux Genius.
pub const SATYA_MAIN: &[CardFactory] = &[
    cayth_famed_mechanist, sphinx_of_the_revelation, filigree_racer, localized_destruction,
    aether_refinery, hourglass_of_the_lost, salvation_colossus, blaster_hulk, razorfield_ripper,
    silverquill_lecturer, overclocked_electromancer, stone_idol_generator, aurora_shifter,
    conversion_apparatus, gontis_aether_heart, lightning_runner, aethertide_whale,
    aethergeode_miner, aethersphere_harvester, bident_of_thassa, temple_of_enlightenment,
    battlefield_forge, myr_battlesphere, grenzo_havoc_raiser, brudiclad_telchor_engineer,
    coveted_jewel, goldspan_dragon, legion_loyalty, mystic_gate, akromas_will, adarkar_wastes,
    shivan_reef, coalition_relic, combustible_gearhulk, aetherworks_marvel, angel_of_invention,
    confiscation_coup, aetherstorm_roc, aethersquall_ancient, frostboil_snarl, austere_command,
    furycalm_snarl, midnight_clock, professional_face_breaker, farewell, port_town,
    temple_of_triumph, solemn_simulacrum, temple_of_epiphany, prairie_stream,
    skyclave_apparition, castle_vantress, jolted_awake, bespoke_battlewagon, roil_cartographer,
    unstable_amulet, amped_raptor, solar_transformer, scurry_of_gremlins, izzet_generatorium,
    talisman_of_conviction, tezzerets_gambit, izzet_boilerworks, azorius_chancery,
    talisman_of_progress, command_tower, burnished_hart, arcane_signet, sol_ring,
    era_of_innovation, aether_hub, decoction_module, glimmer_of_genius, whirler_virtuoso,
    talisman_of_creativity, swords_to_plowshares, wayfarers_bauble, demolition_field,
    mystic_monastery,
    // Basics: 10 plains, 5 island, 5 mountain
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, island,
    island, island, island, island, mountain, mountain, mountain, mountain, mountain,
];

pub const HAKBAL_COMMANDERS: &[CardFactory] = &[hakbal_of_the_surging_soul];

/// **Explorers of the Deep**, the Lost Caverns of Ixalan Commander deck (LCC,
/// 2023-11-17), exactly as MTGJSON's `ExplorersOfTheDeep_LCC` prints it: 79
/// nonbasic cards + 7 Forests + 13 Islands = 99. Simic Merfolk and explore under
/// Hakbal of the Surging Soul.
pub const HAKBAL_MAIN: &[CardFactory] = &[
    xolatoyac_the_smiling_flood, mist_dancer, ripples_of_potential, wave_goodbye,
    bygone_marvels, deeproot_historian, topography_tracker, tributary_instructor,
    singer_of_swift_rivers, benthic_biomancer, commit_memory, coralhelm_commander, curse_of_the_swine,
    emperor_mihail_ii, herald_of_secret_streams, kindred_discovery, kopala_warden_of_waves,
    master_of_the_pearl_trident, merfolk_sovereign, reflections_of_littjara, seafloor_oracle,
    surgespanner, svyelun_of_sea_and_sky, thassa_god_of_the_sea, thieving_skydiver,
    branching_evolution, deeproot_elite, hardened_scales, realmwalker, ruinous_intrusion,
    cold_eyed_selkie, kumena_tyrant_of_orazca, prime_speaker_zegana, quandrix_command,
    simic_ascendancy, tishana_voice_of_thunder, vorel_of_the_hull_clade, zegana_utopian_speaker,
    metallic_mimic, alchemists_refuge, hinterland_harbor, karns_bastion, mosswort_bridge,
    temple_of_mystery, vineglimmer_snarl, merfolk_cave_diver, aetherize, deeproot_waters,
    merrow_reejerey, rapid_hybridization, ravenform, sage_of_fables, stonybrook_banneret,
    beast_within, nicanzil_current_conductor, evolution_sage, explore, inspiring_call,
    kodamas_reach, growth_spiral, kioras_follower, merfolk_mistbinder, merfolk_skydiver,
    tatyova_benthic_druid, arcane_signet, commanders_sphere, simic_signet, sol_ring,
    swiftfoot_boots, command_tower, llanowar_reborn, myriad_landscape, path_of_ancestry,
    reliquary_tower, rogues_passage, secluded_courtyard, simic_growth_chamber,
    temple_of_the_false_god, unclaimed_territory,
    // Basics: 7 forest, 13 island
    forest, forest, forest, forest, forest, forest, forest, island, island, island, island,
    island, island, island, island, island, island, island, island, island,
];

pub const PERRIE_COMMANDERS: &[CardFactory] = &[perrie_the_pulverizer];

/// **Bedecked Brokers**, the Streets of New Capenna Commander deck (NCC,
/// 2022-04-29), exactly as MTGJSON's `BedeckedBrokers_NCC` prints it:
/// 85 nonbasic cards + 5 Plains + 4 Islands + 5 Forests = 99. Bant counters
/// and shields under Perrie, the Pulverizer.
pub const PERRIE_MAIN: &[CardFactory] = &[
    kros_defense_contractor, ajani_unyielding, angelic_sleuth, aven_courier,
    aven_mimeomancer, avenging_huntbonder, bribe_taker, crystalline_giant, devoted_druid,
    denry_klin_editor_in_chief, evolution_sage, fathom_mage, forgotten_ancient,
    grateful_apparition, incubation_druid, jenara_asura_of_war, luminarch_aspirant,
    park_heights_maverick, rishkar_peema_renegade, roalesk_apex_hybrid, scavenging_ooze,
    shield_broker, skyboon_evangelist, skyship_plunderer, slippery_bogbonder,
    steelbane_hydra, thrummingbird, vorel_of_the_hull_clade, wall_of_roots,
    wickerbough_elder, wingspan_mentor, declaration_in_stone, damning_verdict,
    planar_outburst, tezzerets_gambit, rishkars_expertise, urban_evolution, bant_charm,
    brokers_charm, brokers_confluence, contractual_safeguard, exotic_pets, generous_gift,
    storm_of_forms, familys_favor, hoofprints_of_the_stag, primal_empathy,
    resourceful_defense, together_forever, agents_toolkit, arcane_signet, commanders_sphere,
    everflowing_chalice, fellwar_stone, gavel_of_the_righteous, midnight_clock,
    oblivion_stone, oracles_vault, power_conduit, sol_ring, swiftfoot_boots, ash_barrens,
    bant_panorama, brokers_hideout, canopy_vista, command_tower, exotic_orchard,
    flooded_grove, fortified_village, gavony_township, karns_bastion, littjara_mirrorlake,
    llanowar_reborn, myriad_landscape, nesting_grounds, path_of_ancestry, port_town,
    prairie_stream, seaside_citadel, skycloud_expanse, sungrass_prairie, temple_of_mystery,
    vivid_creek, vivid_grove, vivid_meadow,
    // Basics: 5 plains, 4 island, 5 forest
    forest, forest, forest, forest, forest, island, island, island, island, plains, plains,
    plains, plains, plains,
];

pub const EOWYN_COMMANDERS: &[CardFactory] = &[eowyn_shieldmaiden];

/// **Riders of Rohan**, the The Lord of the Rings: Tales of Middle-earth
/// Commander deck (2023-06-23), exactly as MTGJSON's `RidersOfRohan_LTC`
/// prints it: 80 nonbasic cards + 9 Plains + 5 Island + 5 Mountain = 99.
/// Jeskai Humans and the monarch under Éowyn, Shieldmaiden.
pub const EOWYN_MAIN: &[CardFactory] = &[
    aragorn_king_of_gondor, beregond_of_the_guard, champions_of_minas_tirith,
    gilraen_dunedain_protector, grey_host_reinforcements, lossarnach_captain,
    archivist_of_gondor, denethor_stone_seer, fealty_to_the_realm, call_for_aid,
    gimli_of_the_glittering_caves, boromir_gondors_hope, eomer_king_of_rohan,
    faramir_steward_of_gondor, forth_eorlingas, oath_of_eorl, riders_of_rohan,
    taunt_from_the_rampart, crown_of_gondor, bastion_protector, dearly_departed,
    frontline_medic, increasing_devotion, marshals_anthem, selfless_squire,
    unbreakable_formation, verge_rangers, visions_of_glory, weathered_wayfarer,
    combat_celebrant, court_of_ire, earthquake, flamerush_rider, frontier_warmonger,
    harsh_mentor, shared_animosity, zealous_conscripts, supreme_verdict, door_of_destinies,
    vanquishers_banner, battlefield_forge, clifftop_retreat, exotic_orchard, furycalm_snarl,
    glacial_fortress, port_town, prairie_stream, sulfur_falls, throne_of_the_high_city,
    windbrisk_heights, lost_to_legend, erkenbrand_lord_of_westfold, banishing_light,
    fiend_hunter, palace_jailer, path_to_exile, sunset_revelry, swords_to_plowshares,
    village_bell_ringer, prince_imrahil_the_fair, humble_defector, theoden_king_of_rohan,
    arcane_signet, commanders_sphere, heirloom_blade, heralds_horn, sol_ring,
    talisman_of_conviction, talisman_of_progress, thought_vessel, wayfarers_bauble,
    command_tower, evolving_wilds, field_of_ruin, path_of_ancestry, rogues_passage,
    secluded_courtyard, terramorphic_expanse, tranquil_cove, wind_scarred_crag,
    // Basics: 9 plains, 5 island, 5 mountain
    plains, plains, plains, plains, plains, plains, plains, plains, plains, island, island,
    island, island, island, mountain, mountain, mountain, mountain, mountain,
];

pub const ASHLING_COMMANDERS: &[CardFactory] = &[ashling_the_limitless];

/// **Dance of the Elements**, the Lorwyn Eclipsed Commander deck, exactly
/// as MTGJSON prints it: 83 nonbasic cards + 2 Plains + 2 Island + 2 Swamp +
/// 2 Mountain + 8 Forest = 99. Five-color Elementals under Ashling, the
/// Limitless — evoke from hand, then a hasty token copy of what was
/// sacrificed.
pub const ASHLING_MAIN: &[CardFactory] = &[
    mass_of_mysteries, elemental_spectacle, springleaf_parade, jubilation, impulsivity, lamentation,
    belonging, subterfuge, rain_slicked_copse, sodden_verdure, abundant_countryside, endurance,
    fury, haunting_voyage, avenger_of_zendikar, cavalier_of_thorns, greenwarden_of_murasa,
    selvala_heart_of_the_wilds, titan_of_industry, muldrotha_the_gravetide, omnath_locus_of_rage,
    yarok_the_desecrated, omnath_locus_of_the_roil, timeless_lotus, shatter_the_sky,
    hoofprints_of_the_stag, slithermuse, descendants_fury, blasphemous_act, cream_of_the_crop,
    kindred_summons, bane_of_progress, realmwalker, return_of_the_wildspeaker, faeburrow_elder,
    vernal_sovereign, horde_of_notions, maelstrom_wanderer, jegantha_the_wellspring,
    chromatic_lantern, primal_beyond, raging_ravine, exotic_orchard, flamekin_village,
    path_to_exile, mulldrifter, reality_shift, shriekmaw, shimmercreep, flamebraider, crib_swap,
    secluded_courtyard, sol_ring, command_tower, arcane_signet, distant_melody,
    incandescent_soulstoke, eclipsed_flamekin, foundation_breaker, garruks_uprising, risen_reef,
    fellwar_stone, unclaimed_territory, ancient_ziggurat, frontier_bivouac, sandsteppe_citadel,
    savage_lands, opulent_palace, seaside_citadel, jungle_shrine, smokebraider, ingot_chewer,
    fertile_ground, abundant_growth, kodamas_reach, cultivate, path_of_ancestry, thriving_grove,
    thriving_heath, thriving_isle, opal_palace, thriving_bluff, thriving_moor,
    // Basics: 2 plains, 2 island, 2 swamp, 2 mountain, 8 forest
    plains, plains, island, island, swamp, swamp, mountain, mountain, forest, forest, forest,
    forest, forest, forest, forest, forest,
];

pub const FELOTHAR_COMMANDERS: &[CardFactory] = &[felothar_the_steadfast];

/// **Abzan Armor**, the Tarkir: Dragonstorm Commander deck (2025-04-11),
/// exactly as MTGJSON's `AbzanArmor_TDC` prints it: 81 nonbasic cards + 6
/// Plains + 5 Swamp + 7 Forest = 99. Abzan toughness-matters and defenders
/// under Felothar the Steadfast.
pub const FELOTHAR_MAIN: &[CardFactory] = &[
    betor_ancestors_voice, protector_of_the_wastes, reunion_of_the_house, jaws_of_defeat,
    tip_the_scales, will_of_the_abzan, arbor_adherent, canopy_gargantuan, rampart_architect,
    tree_of_redemption, ikra_shidiqi_the_usurper, baldin_century_herdmaster,
    expel_the_interlopers, indomitable_ancients, rhox_faithmender, shalai_voice_of_plenty,
    wakestone_gargoyle, wall_of_reverence, welcoming_vampire, zetalpa_primal_dawn,
    arasta_of_the_endless_web, assault_formation, hornet_nest, seedborn_muse, sylvan_caryatid,
    towering_titan, anguished_unmaking, dragonlord_dromoka, faeburrow_elder,
    shadrix_silverquill, sidar_kondo_of_jamuraa, colfenors_urn, staff_of_compleation,
    weathered_sentinels, canopy_vista, exotic_orchard, fortified_village, isolated_chapel,
    overgrown_farmland, sungrass_prairie, sunpetal_grove, temple_of_malady, temple_of_plenty,
    temple_of_silence, twilight_mire, woodland_cemetery, arcane_signet, sol_ring, command_tower,
    nyx_fleece_ram, slaughter_the_strong, swords_to_plowshares, wall_of_omens,
    wingmantle_chaplain, behind_the_scenes, blight_pile, feed_the_swarm, infernal_grasp,
    wall_of_limbs, arboreal_grazer, axebane_guardian, carven_caryatid, evolving_wilds,
    jaddi_offshoot, overgrown_battlement, sandsteppe_citadel, tower_defense, wall_of_blossoms,
    wall_of_roots, despark, indulging_patrician, crashing_drawbridge, orzhov_signet,
    selesnya_signet, swiftfoot_boots, walking_bulwark, access_tunnel, bojuka_bog,
    deceptive_landscape, path_of_ancestry, radiant_grove,
    // Basics: 6 plains, 5 swamp, 7 forest
    plains, plains, plains, plains, plains, plains, swamp, swamp, swamp, swamp, swamp, forest,
    forest, forest, forest, forest, forest, forest,
];

pub const GALADRIEL_COMMANDERS: &[CardFactory] = &[galadriel_elven_queen];

/// **Elven Council**, the Tales of Middle-earth Commander deck (LTC,
/// 2023-06-23), exactly as MTGJSON's `ElvenCouncil_LTC` prints it:
/// 73 nonbasic cards + 11 Islands + 15 Forests = 99. Simic Elves and council votes
/// under Galadriel, Elven-Queen.
pub const GALADRIEL_MAIN: &[CardFactory] = &[
    gandalf_westward_voyager, raise_the_palisade, trap_the_trespassers, arwen_weaver_of_hope,
    galadhrim_ambush, haldir_lorien_lieutenant, legolas_greenleaf, mirkwood_elk,
    travel_through_caradhras, windswift_slice, cirdan_the_shipwright,
    elrond_of_the_white_council, erestor_of_the_council, mirkwood_trapper,
    radagast_wizard_of_wilds, sail_into_the_west, song_of_earendil, lothlorien_blade,
    model_of_unity, colossal_whale, devastation_tide, mystic_confluence, plea_for_power,
    swan_song, asceticism, elvish_archdruid, elvish_piper, elvish_warmaster, genesis_wave,
    heroic_intervention, hornet_queen, inscription_of_abundance, overwhelming_stampede,
    realm_seekers, seeds_of_renewal, sylvan_offering, exotic_orchard, flooded_grove,
    hinterland_harbor, rejuvenating_springs, vineglimmer_snarl, lorien_revealed,
    celeborn_the_wise, elven_farsight, wose_pathfinder, learn_from_the_past, opt, preordain,
    arbor_elf, beast_within, cultivate, elvish_mystic, elvish_visionary, farhaven_elf,
    mirror_of_galadriel, lignify, paradise_druid, rampant_growth, reclamation_sage, wood_elves,
    growth_spiral, arcane_signet, commanders_sphere, lightning_greaves, sol_ring,
    whispersilk_cloak, ash_barrens, command_tower, field_of_ruin, lonely_sandbar,
    thornwood_falls, tranquil_thicket, woodland_stream,
    // Basics: 11 island, 15 forest
    island, island, island, island, island, island, island, island, island, island, island,
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest,
];

pub const KOTORI_COMMANDERS: &[CardFactory] = &[kotori_pilot_prodigy];

/// **Buckle Up**, the Kamigawa: Neon Dynasty Commander deck (NEC,
/// 2022-02-18), exactly as MTGJSON's `BuckleUp_NEC` prints it: 69
/// nonbasic cards + 15 Plains + 15 Islands = 99. Azorius Vehicles and artifacts under
/// Kotori, Pilot Prodigy.
pub const KOTORI_MAIN: &[CardFactory] = &[
    shorikai_genesis_engine, jace_architect_of_thought, drumbellower, ironsoul_enforcer,
    cyberdrive_awakener, kappa_cannoneer, katsumasa_the_animator, research_thief,
    aeronaut_admiral, cataclysmic_gearhulk, indomitable_archangel, myrsmith,
    sram_senior_edificer, teshar_ancestors_apostle, emry_lurker_of_the_loch, etherium_sculptor,
    master_of_etherium, riddlesmith, sai_master_thopterist, vedalken_engineer, whirler_rogue,
    arcanists_owl, hanna_ships_navigator, raff_capashen_ships_mage, foundry_inspector, gold_myr,
    shimmer_myr, silver_myr, solemn_simulacrum, organic_extinction, universal_surveillance,
    thoughtcast, dance_of_the_manse, release_to_memory, access_denied, armed_and_armored,
    crush_contraband, dispatch, generous_gift, swords_to_plowshares, reality_shift,
    aerial_surveyor, imposter_mech, imperial_recovery_unit, mobilizer_mech, prodigys_prototype,
    surgehacker_mech, parhelion_ii, arcane_signet, azorius_signet, colossal_plow,
    cultivators_caravan, fellwar_stone, mirage_mirror, peacewalker_colossus, raiders_karve,
    skysovereign_consul_flagship, smugglers_copter, sol_ring, weatherlight,
    swift_reconfiguration, thopter_spy_network, command_tower, exotic_orchard, port_town,
    prairie_stream, skycloud_expanse, spire_of_industry, temple_of_enlightenment,
    // Basics: 15 plains, 15 island
    plains, plains, plains, plains, plains, plains, plains, plains, plains, plains, plains,
    plains, plains, plains, plains, island, island, island, island, island, island, island,
    island, island, island, island, island, island, island, island,
];

pub const ANJE_COMMANDERS: &[CardFactory] = &[anje_falkenrath];

/// **Merciless Rage**, the Commander 2019 deck (C19, 2019-08-23), exactly as
/// MTGJSON's `MercilessRage_C19` prints it: 79 nonbasic cards + 10 Swamps +
/// 10 Mountains = 99. Rakdos madness under Anje Falkenrath.
pub const ANJE_MAIN: &[CardFactory] = &[
    ob_nixilis_reignited, chainer_nightmare_adept, greven_predator_captain,
    archfiend_of_spite, bone_miser, krrik_son_of_yawgmoth, anjes_ravager, skyfire_phoenix,
    wildfire_devils, champion_of_stray_souls, geth_lord_of_the_vault, soul_of_innistrad,
    asylum_visitor, doomed_necromancer, overseer_of_the_damned, flayer_of_the_hatebound,
    magus_of_the_wheel, squee_goblin_nabob, stromkirk_occultist, bloodhall_priest,
    solemn_simulacrum, scaretiller, big_game_hunter, gorgon_recluse, grave_scrabbler,
    nightshade_assassin, plaguecrafter, sanitarium_skeleton, meteor_golem,
    nightmare_unmaking, boneyard_parley, beacon_of_unrest, from_under_the_floorboards,
    in_garruks_wake, avacyns_judgment, mire_in_misery, hate_mirage, call_to_the_netherworld,
    murderous_compulsion, alchemists_greeting, malevolent_whispers, chaos_warp,
    dark_withering, fiery_temper, violent_eruption, aeon_engine, grimoire_of_the_dead,
    key_to_the_city, bloodthirsty_blade, armillary_sphere, hedron_archive, rakdos_locket,
    sol_ring, curse_of_fools_wisdom, hedonists_trove, warstorm_surge, faith_of_the_devoted,
    the_eldest_reborn, zombie_infestation, sanctum_of_eternity, drownyard_temple,
    exotic_orchard, geier_reach_sanitarium, akoum_refuge, ash_barrens, barren_moor,
    bloodfell_caves, cinder_barrens, command_tower, evolving_wilds, forgotten_cave,
    memorial_to_folly, mortuary_mire, myriad_landscape, rakdos_carnarium, rakdos_guildgate,
    rix_maadi_dungeon_palace, terramorphic_expanse, temple_of_the_false_god,
    // Basics: 10 swamp, 10 mountain
    swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, swamp, mountain,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain, mountain,
    mountain,
];

pub const SEVINNE_COMMANDERS: &[CardFactory] = &[sevinne_the_chronoclasm];

/// **Mystic Intellect**, the Commander 2019 deck (C19, 2019-08-23), exactly as
/// MTGJSON's `MysticIntellect_C19` prints it: 78 nonbasic cards + 9 Plains +
/// 8 Islands + 4 Mountains = 99. Jeskai flashback and graveyard casting under
/// Sevinne, the Chronoclasm.
/// One swap: Dockside Extortionist is banned in Commander (2024-09-23), so
/// Ragavan, Nimble Pilferer (a cheap red creature that makes Treasure) takes
/// its slot; the card itself is in the catalog.
pub const SEVINNE_MAIN: &[CardFactory] = &[
    ral_zarek, elsha_of_the_infinite, pramikon_sky_rampart, thalias_geistcaller,
    wall_of_stolen_identity, backdraft_hellkite, ragavan_nimble_pilferer,
    gerrard_weatherlight_hero, pristine_angel, sun_titan, clever_impersonator,
    zetalpa_primal_dawn, river_kelpie, talrand_sky_summoner, pristine_skywise,
    cliffside_rescuer, scaretiller, guttersnipe, crackling_drake, burnished_hart,
    sevinnes_reclamation, mass_diminish, ignite_the_future, divine_reckoning, dusk_dawn,
    increasing_devotion, storm_herd, devils_play, deep_analysis, mystic_retrieval,
    runic_repetition, faithless_looting, rolling_temblor, mandate_of_peace,
    increasing_vengeance, magmaquake, refuse_cooperate, leadership_vacuum,
    prismatic_strands, purify_the_grave, ray_of_distortion, chemisters_insight,
    fact_or_fiction, fervent_denial, oonas_grace, think_twice, desperate_ravings,
    farm_market, empowered_autogenerator, bloodthirsty_blade, armillary_sphere,
    azorius_locket, commanders_sphere, izzet_locket, sol_ring, jaces_sanctum,
    ghostly_prison, secrets_of_the_dead, burning_vengeance, exotic_orchard, prairie_stream,
    ash_barrens, azorius_chancery, boros_garrison, boros_guildgate, command_tower,
    evolving_wilds, highland_lake, izzet_boilerworks, izzet_guildgate, myriad_landscape,
    mystic_monastery, stone_quarry, swiftwater_cliffs, temple_of_the_false_god,
    terramorphic_expanse, tranquil_cove, wind_scarred_crag,
    // Basics: 9 plains, 8 island, 4 mountain
    plains, plains, plains, plains, plains, plains, plains, plains, plains, island, island,
    island, island, island, island, island, island, mountain, mountain, mountain, mountain,
];

pub const SAURON_COMMANDERS: &[CardFactory] = &[sauron_lord_of_the_rings];

/// **The Hosts of Mordor**, the LTC Commander deck (2023-06-23), exactly as
/// MTGJSON's `TheHostsOfMordor_LTC` prints it: 80 nonbasic cards + 6 Island
/// + 6 Swamp + 7 Mountain = 99. Grixis Orc armies, the Ring tempting you,
/// and the opponents' own creatures and spells turned against them.
pub const SAURON_MAIN: &[CardFactory] = &[
    saruman_the_white_hand, corsairs_of_umbar, monstrosity_of_the_lake, subjugate_the_hobbits,
    shelob_dread_weaver, cavern_hoard_dragon, orcish_siegemaster, rampaging_war_mammoth,
    the_balrog_of_moria, grima_sarumans_footman, in_the_darkness_bind_them, lidless_gaze,
    lord_of_the_nazgul, moria_scavenger, summons_of_saruman, too_greedily_too_deep,
    wake_the_dragon, relic_of_sauron, the_black_gate, decree_of_pain, languish, living_death,
    reanimate, blasphemous_act, goblin_dark_dwellers, inferno_titan, knollspine_dragon,
    scourge_of_the_throne, siege_gang_commander, treasure_nabber, hostage_taker, notion_thief,
    choked_estuary, desolate_lighthouse, dragonskull_summit, drowned_catacomb, foreboding_ruins,
    frostboil_snarl, smoldering_marsh, sulfur_falls, sulfurous_springs, sunken_hollow,
    underground_river, treason_of_isengard, bitter_downfall, troll_of_khazad_dum,
    voracious_fell_beast, fiery_inscription, grishnakh_brash_instigator, arcane_denial,
    boon_of_the_wish_giver, consider, deep_analysis, fact_or_fiction, forbidden_alchemy,
    feed_the_swarm, merciless_executioner, revenge_of_ravens, anger, faithless_looting,
    the_mouth_of_sauron, goblin_cratermaker, guttersnipe, shiny_impetus, thrill_of_possibility,
    extract_from_darkness, arcane_signet, basalt_monolith, commanders_sphere,
    everflowing_chalice, mind_stone, sol_ring, worn_powerstone, command_tower,
    crumbling_necropolis, evolving_wilds, field_of_ruin, path_of_ancestry, rogues_passage,
    terramorphic_expanse,
    // Basics: 6 island, 6 swamp, 7 mountain
    island, island, island, island, island, island, swamp, swamp, swamp, swamp, swamp, swamp,
    mountain, mountain, mountain, mountain, mountain, mountain, mountain,
];

pub const PANTLAZA_COMMANDERS: &[CardFactory] = &[pantlaza_sun_favored];

/// **Veloci-Ramp-Tor**, the Lost Caverns of Ixalan Commander deck
/// (2023-11-17), exactly as MTGJSON's `VelociRampTor_LCC` prints it:
/// 83 nonbasic cards + 8 Forest + 4 Plains + 4 Mountain = 99. Naya
/// Dinosaurs, enrage and ramp under Pantlaza, Sun-Favored.
pub const PANTLAZA_MAIN: &[CardFactory] = &[
    wayta_trainer_prodigy, bronzebeak_foragers, from_the_rubble, wrathful_raptors, curious_altisaur,
    dinosaur_egg, scion_of_calamity, sunfrill_imitator, progenitors_icon, akromas_will,
    kinjallis_sunwing, temple_altisaur, wakening_suns_avatar, zetalpa_primal_dawn,
    chandras_ignition, etali_primal_storm, fiery_confluence, marauding_raptor, apex_altisaur,
    deathgorge_scavenger, descendants_path, rampaging_brontodon, regal_behemoth,
    return_of_the_wildspeaker, ripjaw_raptor, rishkars_expertise, runic_armasaur, shifting_ceratops,
    topiary_stomper, verdant_suns_avatar, wayward_swordtooth, quartzwood_crasher, regisaur_alpha,
    xenagos_god_of_revels, zacama_primal_calamity, lifecrafters_bestiary, arch_of_orazca,
    canopy_vista, cinder_glade, clifftop_retreat, exotic_orchard, fortified_village, furycalm_snarl,
    game_trail, kessig_wolf_run, mosswort_bridge, bellowing_aegisaur, generous_gift,
    majestic_heliopterus, path_to_exile, earthshaker_dreadmaw, ixallis_lorekeeper,
    thrashing_brontodon, otepec_huntmaster, itzquinth_firstborn_of_gishath, cultivate,
    drover_of_the_mighty, farseek, migration_path, rampant_growth, ranging_raptors, savage_stomp,
    thunderherd_migration, thundering_spineback, atzocan_seer, raging_regisaur, raging_swordtooth,
    rhythm_of_the_wild, arcane_signet, sol_ring, command_tower, evolving_wilds, jungle_shrine,
    myriad_landscape, path_of_ancestry, rogues_passage, secluded_courtyard, temple_of_the_false_god,
    terramorphic_expanse, thriving_bluff, thriving_grove, thriving_heath, unclaimed_territory,
    forest, forest, forest, forest, forest, forest, forest, forest, plains, plains, plains, plains,
    mountain, mountain, mountain, mountain,
];

pub const LATHRIL_COMMANDERS: &[CardFactory] = &[lathril_blade_of_the_elves];

/// **Elven Empire**, the Kaldheim Commander deck (KHC, 2021-02-05), exactly
/// as MTGJSON's `ElvenEmpire_KHC` prints it: 70 nonbasic cards + 16 Forests +
/// 13 Swamps = 99. Golgari Elves under Lathril, Blade of the Elves.
pub const LATHRIL_MAIN: &[CardFactory] = &[
    abomination_of_llanowar, beast_whisperer, canopy_tactician, cultivator_of_blades,
    dwynen_gilt_leaf_daen, elderfang_ritualist, elvish_archdruid, elvish_mystic,
    elvish_rejuvenator, end_raze_forerunners, eyeblight_cullers, farhaven_elf,
    golgari_findbroker, harald_king_of_skemfar, imperious_perfect, jagged_scar_archers,
    jaspera_sentinel, llanowar_tribe, lys_alana_huntmaster, lys_alana_scarblade,
    marwyn_the_nurturer, masked_admirers, miara_thorn_of_the_glade, nullmage_shepherd,
    numa_joraga_chieftain, poison_tip_archer, reclamation_sage, rhys_the_exiled,
    ruthless_winnower, shaman_of_the_pack, skemfar_shadowsage, springbloom_druid,
    sylvan_messenger, timberwatch_elf, twinblade_assassins, voice_of_many,
    voice_of_the_woods, wirewood_channeler, wolverine_riders, wood_elves, elven_ambush,
    poison_the_cup, putrefy, tergrids_shadow, ambitions_cost, bounty_of_skemfar,
    casualties_of_war, eyeblight_massacre, elvish_promenade, harvest_season,
    pact_of_the_serpent, return_upon_the_tide, roots_of_wisdom, arcane_signet,
    serpents_soul_jar, sol_ring, binding_the_old_gods, crown_of_skemfar, elderfang_venom,
    pride_of_the_perfect, prowess_of_the_fair, moldervine_reclamation, command_tower,
    foul_orchard, golgari_guildgate, golgari_rot_farm, jungle_hollow, myriad_landscape,
    path_of_ancestry, skemfar_elderhall,
    // Basics: 16 forest, 13 swamp
    forest, forest, forest, forest, forest, forest, forest, forest, forest, forest, forest,
    forest, forest, forest, forest, forest, swamp, swamp, swamp, swamp, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, swamp, swamp,
];

pub const CLOUD_FIC_COMMANDERS: &[CardFactory] = &[cloud_ex_soldier];

/// **Limit Break**, the Final Fantasy VII Commander deck (FIC, 2025-06-13),
/// exactly as MTGJSON's `LimitBreakFinalFantasyVii_FIC` prints it:
/// 90 nonbasic cards + 3 Forests + 3 Mountains + 3 Plainss = 99. Naya Equipment and
/// power-7 payoffs under Cloud, Ex-SOLDIER.
pub const CLOUD_FIC_MAIN: &[CardFactory] = &[
    tifa_martial_artist, cid_freeflier_pilot, clouds_limit_break, elena_turk_recruit,
    heidegger_shinra_executive, helitrooper, soldier_military_program, ultimate_magic_holy,
    avalanche_of_sector_7, cait_sith_fortune_teller, summon_kujata, ultimate_magic_meteor,
    vincent_vengeful_atoner, yuffie_materia_hunter, bugenhagen_wise_elder,
    lifestreams_blessing, professor_hojo, summoning_materia, aerith_last_ancient,
    barret_avalanche_leader, red_xiii_proud_warrior, sephiroth_fallen_hero,
    conformer_shuriken, wrecking_ball_arm, austere_command, bastion_protector,
    bronze_guardian, clever_concealment, puresteel_paladin, unfinished_business,
    vanquish_the_horde, chaos_warp, hellkite_tyrant, professional_face_breaker, decimate,
    armory_automaton, champions_helm, conquerors_flail, darksteel_plate, inspiring_statuary,
    sword_of_the_animist, battlefield_forge, bonders_enclave, brushland, canopy_vista,
    cinder_glade, clifftop_retreat, exotic_orchard, fire_lit_thicket, fortified_village,
    furycalm_snarl, game_trail, mossfire_valley, rootbound_crag, rugged_prairie,
    scavenger_grounds, slayers_stronghold, spire_of_industry, sungrass_prairie,
    sunpetal_grove, sunscorched_divide, zack_fair, barret_wallace, dispatch,
    secret_rendezvous, furious_rise, vandalblast, cultivate, harmonize, natures_lore,
    rampant_growth, behemoth_sledge, arcane_signet, colossus_hammer, explorers_scope,
    heros_blade, heros_heirloom, lightning_greaves, mask_of_memory, skullclamp, sol_ring,
    trailblazers_boots, ash_barrens, evolving_wilds, jungle_shrine, path_of_ancestry,
    radiant_grove, sacred_peaks, wooded_ridgeline, command_tower,
    // Basics
    mountain, mountain, mountain, forest, forest, forest, plains, plains, plains,
];

pub const BRASS_COMMANDERS: &[CardFactory] = &[admiral_brass_unsinkable];

/// **Ahoy Mateys**, the Lost Caverns of Ixalan Commander deck (LCC,
/// 2023-11-17), exactly as MTGJSON's `AhoyMateys_LCC` prints it:
/// 84 nonbasic cards + 6 Islands + 4 Swamps + 5 Mountains = 99. Grixis Pirates under Admiral
/// Brass, Unsinkable.
pub const BRASS_MAIN: &[CardFactory] = &[
    don_andres_the_renegade, the_indomitable, storm_fleet_negotiator, francisco_fowl_marauder,
    the_grim_captains_locker, skeleton_crew, broadside_bombardiers, gemcutter_buccaneer,
    arm_mounted_anchor, amphin_mutineer, bident_of_thassa, corsair_captain, evacuation,
    timestream_navigator, warkite_marauder, black_market_connections, blood_money,
    dire_fleet_ravager, fathom_fleet_captain, lethal_scheme, angraths_marauders,
    blasphemous_act, captain_lannery_storm, captivating_crew, chaos_warp, coercive_recruiter,
    dire_fleet_daredevil, kari_zev_skyship_raider, port_razer, shared_animosity,
    admiral_beckett_brass, hostage_taker, king_narfis_betrayal, prismari_command,
    zara_renegade_recruiter, icon_of_ancestry, vanquishers_banner, choked_estuary,
    desolate_lighthouse, exotic_orchard, foreboding_ruins, frostboil_snarl,
    geier_reach_sanitarium, nephalia_drownyard, smoldering_marsh, sulfur_falls, sunken_hollow,
    azure_fleet_admiral, enterprising_scallywag, daring_saboteur, departed_deckhand,
    distant_melody, ghost_of_ramirez_depietro, malcolm_keen_eyed_navigator, merchant_raiders,
    siren_stormtamer, spectral_sailor, windfall, feed_the_swarm, pitiless_plunderer,
    breeches_brazen_plunderer, faithless_looting, rakdos_charm, ramirez_depietro_pillager,
    arcane_signet, commanders_sphere, dimir_signet, heralds_horn, izzet_signet, rakdos_signet,
    sol_ring, wayfarers_bauble, command_tower, crumbling_necropolis, evolving_wilds,
    path_of_ancestry, port_of_karfell, rogues_passage, secluded_courtyard, terramorphic_expanse,
    thriving_bluff, thriving_isle, thriving_moor, unclaimed_territory,
    // Basics: 6 island, 4 swamp, 5 mountain
    island, island, island, island, island, island, swamp, swamp, swamp, swamp, mountain,
    mountain, mountain, mountain, mountain,
];

pub const MIRKO_COMMANDERS: &[CardFactory] = &[mirko_obsessive_theorist];

/// **Revenant Recon**, the Murders at Karlov Manor Commander deck (MKC,
/// 2024-02-09), exactly as MTGJSON's `RevenantRecon_MKC` prints it: 81
/// nonbasic cards + 9 Island + 9 Swamp = 99. Dimir surveil, reanimation and
/// copies under Mirko, Obsessive Theorist.
pub const MIRKO_MAIN: &[CardFactory] = &[
    marvo_deep_operative, case_of_the_shifting_visage, copy_catchers, final_word_phantom,
    watcher_of_hours, charnel_serenade, eye_of_duskmantle, foreboding_steamboat,
    unshakable_tail, counterpoint, ransom_note, amphin_mutineer, dream_eater, mission_briefing,
    phyrexian_metamorph, sphinx_of_the_second_sun, vizier_of_many_faces, black_suns_zenith,
    dogged_detective, doom_whisperer, grave_titan, massacre_wurm, overseer_of_the_damned,
    phyrexian_arena, pile_on, reanimate, rise_of_the_dark_realms, toxic_deluge,
    twilight_prophet, baleful_strix, connive_concoct, lazav_the_multifarious, master_of_death,
    choked_estuary, darkwater_catacombs, drownyard_temple, fetid_pools, hostile_desert,
    river_of_tears, sunken_hollow, brainstorm, consider, curate, deep_analysis,
    enhanced_surveillance, epharas_dispersal, mulldrifter, nightveil_sprite, otherworldly_gaze,
    thoughtbound_phantasm, animate_dead, necromancy, price_of_fame, ravenous_chupacabra,
    shriekmaw, sinister_starfish, syr_konrad_the_grim, whispering_snitch, dimir_spybug,
    discovery_dispersal, disinformation_campaign, notion_rain, arcane_signet, dimir_signet,
    everflowing_chalice, mind_stone, sol_ring, talisman_of_dominance, thought_vessel,
    ash_barrens, bojuka_bog, command_tower, dimir_aqueduct, myriad_landscape, mystic_sanctuary,
    port_of_karfell, reliquary_tower, rogues_passage, tainted_isle, temple_of_the_false_god,
    tocasias_dig_site,
    // Basics: 9 island, 9 swamp
    island, island, island, island, island, island, island, island, island, swamp, swamp, swamp,
    swamp, swamp, swamp, swamp, swamp, swamp,
];

pub const TERRA_COMMANDERS: &[CardFactory] = &[terra_herald_of_hope];

/// **Revival Trance**, the Final Fantasy VI Commander deck (FIC, 2025-06-13),
/// exactly as MTGJSON's `RevivalTranceFinalFantasyVi_FIC` prints it:
/// 89 nonbasic cards + 3 Mountains + 4 Plainss + 3 Swamps = 99. Mardu graveyard recursion
/// under Terra, Herald of Hope.
pub const TERRA_MAIN: &[CardFactory] = &[
    celes_rune_knight, coin_of_fate, cyan_vengeful_samurai, general_leo_cristophe,
    espers_to_magicite, the_falcon_airship_restored, interceptor_shadows_hound,
    rejoin_the_fight, shadow_mysterious_assassin, siegfried_famed_swordsman,
    gau_feral_youth, gogo_mysterious_mime, sabin_master_monk, snort, strago_and_relm,
    summon_esper_valigarmanda, umaro_raging_yeti, banon_the_returners_leader,
    edgar_master_machinist, kefka_dancing_mad, locke_treasure_hunter, mog_moogle_warrior,
    setzer_wandering_gambler, the_warring_triad, sun_titan, tragic_arrogance,
    archfiend_of_depravity, reanimate, rise_of_the_dark_realms, sepulchral_primordial,
    combustible_gearhulk, flayer_of_the_hatebound, ruin_grinder, bedevil, legions_to_ashes,
    priest_of_fell_rites, ruinous_ultimatum, key_to_the_city, solemn_simulacrum,
    battlefield_forge, clifftop_retreat, desolate_mire, dragonskull_summit, exotic_orchard,
    fetid_heath, foreboding_ruins, furycalm_snarl, graven_cairns, high_market,
    isolated_chapel, rugged_prairie, shadowblood_ridge, shineshadow_snarl, smoldering_marsh,
    sulfurous_springs, sunscorched_divide, phoenix_down, laughing_mad, angel_of_the_ruins,
    palace_jailer, morbid_opportunist, nights_whisper, pitiless_plunderer, stitch_together,
    stitchers_supplier, anger, big_score, crackling_doom, mortify, arcane_signet,
    commanders_sphere, meteor_golem, millikin, mind_stone, sol_ring, swiftfoot_boots,
    talisman_of_conviction, talisman_of_indulgence, wayfarers_bauble, ash_barrens,
    demolition_field, evolving_wilds, geothermal_bog, nomad_outpost, path_of_ancestry,
    rogues_passage, sacred_peaks, sunlit_marsh, command_tower,
    // Basics
    plains, plains, plains, plains, swamp, swamp, swamp, mountain, mountain, mountain,
];

pub const JACE_COMMANDERS: &[CardFactory] = &[jace_multiverse_architect];

/// **Multiverse Reforged**, the Final Reforging Commander deck (2026-10-02),
/// exactly as MTGJSON's `MultiverseReforged_FRC` prints it: 88 nonbasic
/// cards + 4 Plains + 3 Island + 2 Swamp + 2 Mountain = 99. Four-color
/// legends and planeswalker tricks under Jace, Multiverse Architect (a
/// planeswalker commander, CR 903.3a's printed exception).
pub const JACE_MAIN: &[CardFactory] = &[
    nissa_leyline_tamer, omnath_locus_of_the_void, dack_fayden_helping_hand,
    ob_nixilis_the_ascended, teferis_reproach, avacyn_angel_of_horror, jhoira_weatherlight_corsair,
    venser_fervent_forger, niv_mizzet_ghost_counsel, tamiyo_upriser_crowned, the_ur_sphinx,
    darksteel_angel, ginger_queen_of_sweets, memnarch_the_warden, turbulent_crater, turbulent_shore,
    turbulent_wetlands, akroma_angel_of_fury, reflecting_pool, elspeth_suns_champion,
    flawless_maneuver, grand_crescendo, martial_coup, overlord_of_the_mistmoors, secure_the_wastes,
    serras_emissary, skrelvs_hive, staff_of_the_storyteller, sunfall, white_suns_twilight,
    mass_polymorph, occult_epiphany, shark_typhoon, synthetic_destiny, archfiend_of_despair,
    archon_of_cruelty, dreadhorde_invasion, cursed_mirror, whirlwind_of_thought, windcrag_siege,
    chromatic_lantern, currency_converter, proteus_staff, battlefield_forge, caves_of_koilos,
    clifftop_retreat, drowned_catacomb, exotic_orchard, fabled_passage, fetid_heath,
    glacial_fortress, isolated_chapel, kher_keep, mystic_gate, prairie_stream, radiant_summit,
    restless_anchorage, restless_spire, shivan_reef, sulfur_falls, sulfurous_springs, sunken_ruins,
    underground_river, plan_for_all_outcomes, fatehold_charm, arcane_signet, sol_ring,
    command_tower, lingering_souls, path_to_exile, stroke_of_midnight, swords_to_plowshares,
    brainstorm, brainsurge, fact_or_fiction, despark, azorius_signet, dimir_signet, fellwar_stone,
    izzet_signet, rakdos_signet, talisman_of_creativity, talisman_of_dominance,
    talisman_of_indulgence, talisman_of_progress, contaminated_landscape, path_of_ancestry,
    perilous_landscape, plains, plains, plains, plains, island, island, island, swamp, swamp,
    mountain, mountain,
];

pub const FRODO_COMMANDERS: &[CardFactory] = &[frodo_adventurous_hobbit, sam_loyal_attendant];

/// **Food and Fellowship**, the Tales of Middle-earth Commander deck (LTC,
/// 2023-06-23), exactly as MTGJSON's `FoodAndFellowship_LTC` prints it: 82
/// nonbasic cards + 4 Plains + 4 Swamps + 8 Forests = 98, beside two
/// commanders that partner with each other (CR 702.124j). Abzan Food and
/// lifegain under Frodo and Sam.
pub const FRODO_MAIN: &[CardFactory] = &[
    field_tested_frying_pan, the_gaffer, gwaihir_greatest_of_the_eagles,
    of_herbs_and_stewed_rabbit, gollum_obsessed_stalker, lobelia_defender_of_bag_end,
    rapacious_guest, assemble_the_entmoot, feasting_hobbit, motivated_pony, prize_pig,
    banquet_guests, bilbo_birthday_celebrant, farmer_cotton, merry_warden_of_isengard,
    pippin_warden_of_isengard, treebeard_gracious_host, hithlain_rope, call_for_unity,
    dawn_of_hope, dusk_dawn, fell_the_mighty, fumigate, mentor_of_the_meek, sanguine_bond,
    toxic_deluge, birds_of_paradise, gilded_goose, woodfall_primus, anguished_unmaking,
    chromatic_lantern, trading_post, well_of_lost_dreams, brushland, canopy_vista,
    exotic_orchard, fortified_village, isolated_chapel, murmuring_bosk, necroblossom_snarl,
    scattered_groves, shineshadow_snarl, sunpetal_grove, woodland_cemetery,
    eagles_of_the_north, landroval_horizon_witness, rosie_cotton_of_south_lane,
    shire_shirriff, mirkwood_bats, generous_ent, path_to_exile, swords_to_plowshares,
    revive_the_shire, butterbur_bree_innkeeper, crypt_incursion, go_for_the_throat,
    nights_whisper, cultivate, essence_warden, farseek, great_oak_guardian, harmonize,
    orchard_strider, prosperous_innkeeper, shire_terrace, tireless_provisioner, mortify,
    savvy_hunter, arcane_signet, commanders_sphere, pristine_talisman, sol_ring,
    access_tunnel, ash_barrens, command_tower, evolving_wilds, ghost_quarter,
    graypelt_refuge, path_of_ancestry, rogues_passage, sandsteppe_citadel, scoured_barrens,
    // Basics: 4 plains, 4 swamp, 8 forest
    plains, plains, plains, plains, swamp, swamp, swamp, swamp, forest, forest, forest,
    forest, forest, forest, forest, forest,
];

pub const YSHTOLA_COMMANDERS: &[CardFactory] = &[yshtola_nights_blessed];

/// **Scions & Spellcraft**, the Final Fantasy XIV Commander deck (FIC,
/// 2025-06-13), exactly as MTGJSON's `ScionsSpellcraftFinalFantasyXiv_FIC`
/// prints it: 88 nonbasic cards + 4 Plains + 3 Islands + 4 Swamps = 99. Esper noncreature
/// spells under Y'shtola, Night's Blessed.
pub const YSHTOLA_MAIN: &[CardFactory] = &[
    graha_tia_scion_reborn, alisaie_leveilleur, champions_from_beyond, dancers_chakrams,
    summon_good_king_mog_xii, tataru_taru, thancred_waters, alphinaud_leveilleur,
    blue_mages_cane, hermes_overseer_of_elpis, hraesvelgr_of_the_first_brood, observed_stasis,
    eye_of_nidhogg, fandaniel_telophoroi_ascian, astrologians_planisphere, reapers_scythe,
    transpose, ardbert_warrior_of_darkness, emet_selch_of_the_third_seat, estinien_varlineau,
    hildibrand_manderville, krile_baldesion, lyse_hext, papalymo_totolymo, urianger_augurelt,
    archaeomancers_map, authority_of_the_consuls, cleansing_nova, final_judgment,
    archmage_emeritus, dig_through_time, rite_of_replication, sublime_epiphany,
    torrential_gearhulk, crux_of_fate, lethal_scheme, murderous_rider, baleful_strix, vindicate,
    void_rend, coveted_jewel, tome_of_legends, choked_estuary, darkwater_catacombs,
    desolate_mire, drowned_catacomb, exotic_orchard, fetid_heath, glacial_fortress,
    isolated_chapel, port_town, prairie_stream, scavenger_grounds, shineshadow_snarl,
    skycloud_expanse, sunken_hollow, sunken_ruins, underground_river, white_auracite,
    sages_nouliths, circle_of_power, cut_a_deal, lingering_souls, swords_to_plowshares,
    hypnotic_sprite, into_the_story, propaganda, bastion_of_remembrance, exsanguinate,
    snuff_out, syphon_mind, arcane_signet, relic_of_legends, sol_ring, talisman_of_dominance,
    talisman_of_hierarchy, talisman_of_progress, thought_vessel, arcane_sanctum, ash_barrens,
    command_tower, contaminated_aquifer, demolition_field, evolving_wilds, idyllic_beachfront,
    path_of_ancestry, sunlit_marsh, temple_of_the_false_god,
    // Basics: 4 plains, 3 island, 4 swamp
    plains, plains, plains, plains, island, island, island, swamp, swamp, swamp, swamp,
];
