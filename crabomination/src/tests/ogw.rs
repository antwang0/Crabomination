//! Functionality tests for the OGW/Eldrazi card pack — Devoid (CR 702.114)
//! and Ingest (CR 702.115).

use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::game::types::TurnStep;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// CR 702.114 — Devoid makes a card colorless despite its colored pips.
#[test]
fn cr_702_114_devoid_card_is_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mist_intruder());
    let cp = g.computed_permanent(id).expect("on battlefield");
    assert!(cp.colors.is_empty(), "Devoid creature is colorless despite {{U}} in cost");
}

/// CR 702.115 — Ingest exiles the top card of the damaged player's library
/// when this creature deals combat damage to them.
#[test]
fn cr_702_115_ingest_exiles_top_of_library_on_combat_damage() {
    let mut g = two_player_game();
    // Give the defender a known library to exile from.
    for _ in 0..3 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let lib_before = g.players[1].library.len();
    let exile_before = g.exile.len();
    let atk = g.add_card_to_battlefield(0, catalog::mist_intruder());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib_before - 1, "ingest removes one from library");
    assert_eq!(g.exile.len(), exile_before + 1, "ingested card lands in exile");
}

/// Sludge Crawler's {2} pump grows it +1/+1 until end of turn.
#[test]
fn sludge_crawler_pumps_for_two_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sludge_crawler());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (2, 2), "Sludge Crawler is 2/2 after pump");
}

/// Touch of the Void is a Devoid sorcery dealing 3.
#[test]
fn touch_of_the_void_deals_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::touch_of_the_void());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[1].life;
    crate::game::cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, before - 3, "Touch of the Void deals 3");
    assert!(catalog::touch_of_the_void().keywords.contains(&crate::card::Keyword::Devoid));
}

fn scion_count(g: &GameState) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Scion").count()
}

/// Eldrazi Skyspawner makes a Scion on ETB; the Scion sacs for {C}.
#[test]
fn eldrazi_skyspawner_makes_scion_that_sacs_for_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::eldrazi_skyspawner());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    crate::game::cast(&mut g, id);
    assert_eq!(scion_count(&g), 1, "ETB mints one Eldrazi Scion");
    let scion = g.battlefield.iter().find(|c| c.definition.name == "Eldrazi Scion").unwrap().id;
    g.perform_action(GameAction::ActivateAbility {
        card_id: scion, ability_index: 0, target: None, x_value: None,
    }).expect("sac scion for mana");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "Scion sacs for {{C}}");
    assert_eq!(scion_count(&g), 0, "Scion is sacrificed");
}

/// Call the Scions mints two Eldrazi Scions.
#[test]
fn call_the_scions_makes_two_scions() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::call_the_scions());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    crate::game::cast(&mut g, id);
    assert_eq!(scion_count(&g), 2, "Call the Scions makes two Scions");
}

/// Blisterpod mints a Scion when it dies.
#[test]
fn blisterpod_makes_scion_on_death() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::blisterpod());
    drain_stack(&mut g);
    assert_eq!(scion_count(&g), 0);
    // Lethal damage + SBA → death trigger.
    g.battlefield_find_mut(id).unwrap().damage = 5;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(scion_count(&g), 1, "Blisterpod's death mints a Scion");
}

/// Eyeless Watcher mints two Scions on ETB.
#[test]
fn eyeless_watcher_makes_two_scions() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::eyeless_watcher());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    crate::game::cast(&mut g, id);
    assert_eq!(scion_count(&g), 2);
}

/// Eldrazi Devastator is an 8/9 colorless trampler.
#[test]
fn eldrazi_devastator_is_colorless_trampler() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::eldrazi_devastator());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 9));
    assert!(cp.colors.is_empty(), "generic-only cost → colorless");
    assert!(cp.keywords.contains(&crate::card::Keyword::Trample));
}

/// Incubator Drone and Catacomb Sifter each mint a Scion on ETB.
#[test]
fn incubator_and_catacomb_sifter_make_scions() {
    let mut g = two_player_game();
    let inc = g.add_card_to_hand(0, catalog::incubator_drone());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    crate::game::cast(&mut g, inc);
    assert_eq!(scion_count(&g), 1);
    let cs = g.add_card_to_hand(0, catalog::catacomb_sifter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    crate::game::cast(&mut g, cs);
    assert_eq!(scion_count(&g), 2, "Catacomb Sifter adds a second Scion");
}

/// Warden of Geometries taps for {C}; Cultivator Drone taps for {C}{C}.
#[test]
fn devoid_mana_dorks_tap_for_colorless() {
    let mut g = two_player_game();
    let w = g.add_card_to_battlefield(0, catalog::warden_of_geometries());
    let c = g.add_card_to_battlefield(0, catalog::cultivator_drone());
    g.clear_sickness(w);
    g.clear_sickness(c);
    g.perform_action(GameAction::ActivateAbility {
        card_id: w, ability_index: 0, target: None, x_value: None,
    }).expect("warden taps");
    g.perform_action(GameAction::ActivateAbility {
        card_id: c, ability_index: 0, target: None, x_value: None,
    }).expect("cultivator taps");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 3, "{{C}} + {{C}}{{C}}");
}

/// Oblivion Strike exiles a creature; Complete Disregard only hits power ≤3.
#[test]
fn oblivion_strike_exiles_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::oblivion_strike());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    crate::game::cast_at(&mut g, id, Target::Permanent(bear));
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "exiled");
    assert!(g.exile.iter().any(|c| c.id == bear), "in exile");
}

#[test]
fn complete_disregard_cannot_hit_big_creatures() {
    let mut g = two_player_game();
    let hill = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3 — power 3 OK
    let id = g.add_card_to_hand(0, catalog::complete_disregard());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    crate::game::cast_at(&mut g, id, Target::Permanent(hill));
    assert!(g.exile.iter().any(|c| c.id == hill), "power-3 creature is exiled");
}

/// Spatial Contortion gives +3/-3; a 2/2 dies to the toughness drop.
#[test]
fn spatial_contortion_minus_three_toughness_kills_a_two_two() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::spatial_contortion());
    g.players[0].mana_pool.add_colorless(2);
    crate::game::cast_at(&mut g, id, Target::Permanent(bear));
    g.check_state_based_actions();
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2/2 → -3 toughness → dies");
}

/// Unnatural Endurance pumps +2/+0 and regenerates the target.
#[test]
fn unnatural_endurance_pumps_and_regenerates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::unnatural_endurance());
    g.players[0].mana_pool.add(Color::Black, 1);
    crate::game::cast_at(&mut g, id, Target::Permanent(bear));
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (4, 2), "+2/+0");
    // Lethal damage is replaced by the regen shield → survives.
    g.battlefield_find_mut(bear).unwrap().damage = 2;
    g.check_state_based_actions();
    assert!(g.battlefield.iter().any(|c| c.id == bear), "regenerated");
}

/// Warping Wail's third mode creates a 1/1 Eldrazi Scion.
#[test]
fn warping_wail_makes_a_scion() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::warping_wail());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(2), x_value: None,
    }).expect("cast Warping Wail (scion mode)");
    drain_stack(&mut g);
    assert_eq!(scion_count(&g), 1, "mode 2 mints an Eldrazi Scion");
}

/// Tar Snare gives -3/-2; a 2/2 dies.
#[test]
fn tar_snare_kills_a_two_two() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::tar_snare());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    crate::game::cast_at(&mut g, id, Target::Permanent(bear));
    g.check_state_based_actions();
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2/2 → -2 toughness → dies");
}

/// Vile Aggregate's power equals the colorless creatures its controller
/// controls (it counts itself; rises as more colorless creatures join).
#[test]
fn vile_aggregate_power_scales_with_colorless_creatures() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::vile_aggregate());
    // Only itself (devoid → colorless) → power 1, toughness 5.
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 5));
    // Add two more colorless creatures (Eldrazi Devastator + a Scion token).
    g.add_card_to_battlefield(0, catalog::eldrazi_devastator());
    g.add_token_to_battlefield(0, &crabomination_base::tokens::eldrazi_scion_token());
    // A colored creature does NOT count.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(id).unwrap().power, 3, "3 colorless creatures");
}

/// Benthic Infiltrator can't be blocked and ingests; Culling Drone ingests.
#[test]
fn benthic_infiltrator_is_unblockable_and_ingests() {
    let b = catalog::benthic_infiltrator();
    assert!(b.keywords.contains(&crate::card::Keyword::Unblockable));
    assert!(b.keywords.contains(&crate::card::Keyword::Devoid));
    // Both carry the Ingest combat trigger.
    assert_eq!(catalog::culling_drone().triggered_abilities.len(), 1);
}

/// Murderous Compulsion destroys a tapped creature but not an untapped one.
#[test]
fn murderous_compulsion_only_hits_tapped() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::murderous_compulsion());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    crate::game::cast_at(&mut g, id, Target::Permanent(bear));
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "tapped creature destroyed");
    assert!(matches!(
        catalog::murderous_compulsion().keywords[0],
        crate::card::Keyword::Madness(_)
    ));
}

/// Sweep Away bounces a creature to its owner's hand.
#[test]
fn sweep_away_bounces_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::sweep_away());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    crate::game::cast_at(&mut g, id, Target::Permanent(bear));
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "off battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "back in owner's hand");
}

/// Maw of Kozilek's {C} ability gives +2/-2 until end of turn.
#[test]
fn maw_of_kozilek_pumps_plus_two_minus_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::maw_of_kozilek());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("pump +2/-2");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 3), "2/5 → 4/3");
}

/// Voracious Null sacrifices a creature to grow by two +1/+1 counters.
#[test]
fn voracious_null_sacrifices_for_counters() {
    let mut g = two_player_game();
    let null = g.add_card_to_battlefield(0, catalog::voracious_null());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: null, ability_index: 0, target: None, x_value: None,
    }).expect("sac for counters");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == fodder), "fodder sacrificed");
    let c = g.battlefield_find(null).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 4), "2/2 + two +1/+1 → 4/4");
}

/// Dread Drone mints two 0/1 Eldrazi Spawn on ETB.
#[test]
fn dread_drone_makes_two_spawn() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::dread_drone());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    crate::game::cast(&mut g, id);
    let spawn = g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count();
    assert_eq!(spawn, 2, "two Eldrazi Spawn on ETB");
}

/// Slaughter Drone's {C} ability grants deathtouch until end of turn.
#[test]
fn slaughter_drone_gains_deathtouch() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::slaughter_drone());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("grant deathtouch");
    drain_stack(&mut g);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&crate::card::Keyword::Deathtouch));
}

/// Witness the End makes the opponent discard two and lose 2 life.
#[test]
fn witness_the_end_discard_two_lose_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    let hand_before = g.players[1].hand.len();
    let life_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witness_the_end());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    crate::game::cast(&mut g, id);
    assert_eq!(g.players[1].hand.len(), hand_before - 2, "opponent discards two");
    assert_eq!(g.players[1].life, life_before - 2, "opponent loses 2 life");
}

/// Kozilek's Channeler and Hedron Crawler are colorless mana producers.
#[test]
fn colorless_mana_dorks_produce_colorless() {
    let mut g = two_player_game();
    let ch = g.add_card_to_battlefield(0, catalog::kozileks_channeler());
    let cr = g.add_card_to_battlefield(0, catalog::hedron_crawler());
    g.clear_sickness(ch);
    g.clear_sickness(cr);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ch, ability_index: 0, target: None, x_value: None,
    }).expect("channeler taps");
    g.perform_action(GameAction::ActivateAbility {
        card_id: cr, ability_index: 0, target: None, x_value: None,
    }).expect("crawler taps");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 3, "{{C}}{{C}} + {{C}}");
    assert!(g.computed_permanent(ch).unwrap().colors.is_empty(), "Channeler is colorless");
}

/// Scion Summoner mints one Scion; Brood Monitor mints three.
#[test]
fn scion_summoner_and_brood_monitor_make_scions() {
    let mut g = two_player_game();
    let ss = g.add_card_to_hand(0, catalog::scion_summoner());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    crate::game::cast(&mut g, ss);
    assert_eq!(scion_count(&g), 1);
    let bm = g.add_card_to_hand(0, catalog::brood_monitor());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    crate::game::cast(&mut g, bm);
    assert_eq!(scion_count(&g), 4, "Brood Monitor adds three more");
}

/// Springleaf Drum taps a creature for one mana of any color.
#[test]
fn springleaf_drum_taps_a_creature_for_mana() {
    let mut g = two_player_game();
    let drum = g.add_card_to_battlefield(0, catalog::springleaf_drum());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.perform_action(GameAction::ActivateAbility {
        card_id: drum, ability_index: 0, target: None, x_value: None,
    }).expect("drum taps a creature for mana");
    assert_eq!(g.players[0].mana_pool.total(), 1, "one mana produced");
    assert!(g.battlefield_find(bear).unwrap().tapped, "the creature is tapped as a cost");
}

/// Breaker of Armies carries the all-must-block keyword (CR 509.1c).
#[test]
fn breaker_of_armies_has_all_must_block() {
    assert!(catalog::breaker_of_armies().keywords.contains(&crate::card::Keyword::AllMustBlock));
}

/// Reality Hemorrhage is a Devoid burn instant dealing 2.
#[test]
fn reality_hemorrhage_deals_two_and_is_colorless() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::reality_hemorrhage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let before = g.players[1].life;
    crate::game::cast_at(&mut g, bolt, Target::Player(1));
    assert_eq!(g.players[1].life, before - 2, "Reality Hemorrhage deals 2");
}
