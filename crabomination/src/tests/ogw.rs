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

/// Salvage Drone optionally draws when it dies.
#[test]
fn salvage_drone_may_draw_on_death() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::salvage_drone());
    // Accept the optional "may draw" on death.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let hand_before = g.players[0].hand.len();
    g.battlefield_find_mut(id).unwrap().damage = 5;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew on death");
}

/// Skitterskin regenerates for {1}{B} and can't block.
#[test]
fn skitterskin_regenerates() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::skitterskin());
    assert!(catalog::skitterskin().keywords.contains(&crate::card::Keyword::CantBlock));
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("set up regen shield");
    drain_stack(&mut g);
    g.battlefield_find_mut(id).unwrap().damage = 99;
    g.check_state_based_actions();
    assert!(g.battlefield.iter().any(|c| c.id == id), "regenerated instead of dying");
}

/// Mindmelter makes the opponent discard; Deepfathom Skulker grants
/// unblockable to a target creature.
#[test]
fn mindmelter_discards_deepfathom_grants_unblockable() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let mm = g.add_card_to_battlefield(0, catalog::mindmelter());
    g.clear_sickness(mm);
    let hand_before = g.players[1].hand.len();
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mm, ability_index: 0, target: None, x_value: None,
    }).expect("opp discards");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before - 1, "opponent discarded one");

    let skulker = g.add_card_to_battlefield(0, catalog::deepfathom_skulker());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(skulker);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: skulker, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("grant unblockable");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&crate::card::Keyword::Unblockable));
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

// ── Eldrazi titans & colossi ────────────────────────────────────────────────

/// Ulamog's cast trigger destroys a target permanent; the titan resolves
/// onto the battlefield with Indestructible + Annihilator 4.
#[test]
fn ulamog_cast_destroys_target_permanent() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::ulamog_the_infinite_gyre());
    g.players[0].mana_pool.add_colorless(11);
    crate::game::cast_at(&mut g, id, Target::Permanent(bear));
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Ulamog's cast trigger destroys the bear");
    let u = g.battlefield_find(id).expect("Ulamog resolves");
    assert!(u.definition.keywords.contains(&Keyword::Indestructible));
    assert!(u.definition.keywords.contains(&Keyword::Annihilator(4)));
}

/// Kozilek's cast trigger draws four cards.
#[test]
fn kozilek_cast_draws_four() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::kozilek_butcher_of_truth());
    let before = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(10);
    crate::game::cast(&mut g, id);
    // -1 for Kozilek leaving hand, +4 drawn.
    assert_eq!(g.players[0].hand.len(), before - 1 + 4, "Kozilek draws four on cast");
}

/// Ulamog/Kozilek shuffle their owner's graveyard back into the library when
/// they die.
#[test]
fn kozilek_dies_shuffles_graveyard_into_library() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    let id = g.add_card_to_battlefield(0, catalog::kozilek_butcher_of_truth());
    let lib_before = g.players[0].library.len();
    g.battlefield_find_mut(id).unwrap().damage = 99;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.is_empty(), "graveyard shuffled away");
    // 2 old cards + Kozilek itself returned to library.
    assert_eq!(g.players[0].library.len(), lib_before + 3);
}

/// Pathrazer can't be blocked by fewer than three creatures.
#[test]
fn pathrazer_requires_three_blockers() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::pathrazer_of_ulamog());
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b3 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Annihilator 3 fires on attack; give the defender 3 lands as cheaper
    // fodder so the auto-picker sacrifices those and the bears survive.
    for _ in 0..3 { g.add_card_to_battlefield(1, catalog::plains()); }
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(b1, atk), (b2, atk)])).is_err(),
        "two blockers is illegal");
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, atk), (b2, atk), (b3, atk)]))
        .expect("three blockers is legal");
}

/// Ulamog's Crusher attacks each combat if able, with Annihilator 2.
#[test]
fn ulamogs_crusher_annihilator_sacrifices_two() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::ulamogs_crusher());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::hill_giant());
    g.clear_sickness(atk);
    let opp_perms = g.battlefield.iter().filter(|c| c.controller == 1).count();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let after = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(after, opp_perms - 2, "Annihilator 2 sacrifices two of the defender's permanents");
    assert!(catalog::ulamogs_crusher().keywords.contains(&crate::card::Keyword::MustAttack));
}

/// Artisan of Kozilek's cast trigger reanimates a creature from the graveyard.
#[test]
fn artisan_cast_reanimates_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::artisan_of_kozilek());
    g.players[0].mana_pool.add_colorless(9);
    crate::game::cast_at(&mut g, id, Target::Permanent(bear));
    assert!(g.battlefield.iter().any(|c| c.id == bear), "the bear is reanimated");
}

/// Desolation Twin's cast trigger mints a 10/10 Eldrazi token.
#[test]
fn desolation_twin_cast_makes_ten_ten() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::desolation_twin());
    g.players[0].mana_pool.add_colorless(10);
    crate::game::cast(&mut g, id);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Eldrazi"
        && c.id != id).expect("10/10 token minted");
    assert_eq!((token.power(), token.toughness()), (10, 10));
}

/// Bane of Bala Ged's attack trigger makes the defender exile two permanents.
#[test]
fn bane_of_bala_ged_attack_exiles_two_permanents() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::bane_of_bala_ged());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::hill_giant());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    let exile_before = g.exile.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), exile_before + 2, "defender exiles two permanents on attack");
}

/// Birthing Hulk mints two Eldrazi Scions on ETB.
#[test]
fn birthing_hulk_makes_two_scions() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::birthing_hulk());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(6);
    crate::game::cast(&mut g, id);
    assert_eq!(scion_count(&g), 2);
}

/// Drowner of Hope sacrifices a Scion to tap a creature.
#[test]
fn drowner_of_hope_sacs_scion_to_tap() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::drowner_of_hope());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    crate::game::cast(&mut g, id); // ETB mints two Scions
    assert_eq!(scion_count(&g), 2);
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Permanent(target)), x_value: None,
    }).expect("sac scion, tap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).unwrap().tapped, "target creature tapped");
    assert_eq!(scion_count(&g), 1, "one Scion sacrificed");
}

/// Kozilek's Return deals 2 to each creature, killing a 2/2.
#[test]
fn kozileks_return_sweeps_two_toughness_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::kozileks_return());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    crate::game::cast(&mut g, id);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "the 2/2 dies to 2 damage");
}

/// Kozilek's Shrieker pumps +1/+0 and gains menace for {C}.
#[test]
fn kozileks_shrieker_pumps_and_gains_menace() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::kozileks_shrieker());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2));
    assert!(cp.keywords.contains(&Keyword::Menace));
}

/// Sifter of Skulls mints a Scion when another nontoken creature dies, but not
/// when a token dies.
#[test]
fn sifter_of_skulls_mints_on_nontoken_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sifter_of_skulls());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Kill the bear with a damage spell so CreatureDied fires through the
    // normal path (raw damage + SBA doesn't dispatch the death trigger here).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    crate::game::cast_at(&mut g, bolt, Target::Permanent(bear));
    assert_eq!(scion_count(&g), 1, "another nontoken creature dying mints a Scion");
}

/// Pawn of Ulamog mints an Eldrazi Spawn when it itself dies.
#[test]
fn pawn_of_ulamog_mints_on_own_death() {
    let mut g = two_player_game();
    let pawn = g.add_card_to_battlefield(0, catalog::pawn_of_ulamog());
    let spawn_before = g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count();
    g.battlefield_find_mut(pawn).unwrap().damage = 99;
    g.check_state_based_actions();
    drain_stack(&mut g);
    let spawn_after = g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count();
    assert_eq!(spawn_after, spawn_before + 1, "Pawn's own death mints an Eldrazi Spawn");
}

/// Vestige of Emrakul is a Devoid 3/4 trampler.
#[test]
fn vestige_of_emrakul_is_devoid_trampler() {
    use crate::card::Keyword;
    let def = catalog::vestige_of_emrakul();
    assert_eq!((def.power, def.toughness), (3, 4));
    assert!(def.keywords.contains(&Keyword::Devoid));
    assert!(def.keywords.contains(&Keyword::Trample));
}

/// Stalking Drone pumps +1/+2 once each turn.
#[test]
fn stalking_drone_pumps_once_per_turn() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::stalking_drone());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 4));
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).is_err(), "only once each turn");
}

/// Nettle Drone pings each opponent for {T}, and a colorless cast untaps it.
#[test]
fn nettle_drone_pings_and_untaps_on_colorless_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::nettle_drone());
    g.clear_sickness(id);
    let before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 1, "pings the opponent for 1");
    assert!(g.battlefield_find(id).unwrap().tapped, "tapped after activating");
    // Cast a colorless spell → untap.
    let spell = g.add_card_to_hand(0, catalog::eldrazi_devastator());
    g.players[0].mana_pool.add_colorless(8);
    crate::game::cast(&mut g, spell);
    assert!(!g.battlefield_find(id).unwrap().tapped, "colorless cast untaps Nettle Drone");
}

/// Scour from Existence exiles any permanent.
#[test]
fn scour_from_existence_exiles_permanent() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::scour_from_existence());
    g.players[0].mana_pool.add_colorless(7);
    crate::game::cast_at(&mut g, id, Target::Permanent(land));
    assert!(g.exile.iter().any(|c| c.id == land), "the land is exiled");
}

/// Ruination Guide's anthem reaches a Devoid creature (colored pips, colorless
/// object) — the Devoid-aware Colorless filter at work.
#[test]
fn ruination_guide_anthem_buffs_devoid_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ruination_guide());
    let mist = g.add_card_to_battlefield(0, catalog::mist_intruder()); // Devoid 1/2
    let cp = g.computed_permanent(mist).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "Devoid creature gets +1/+0 from the anthem");
}

/// The anthem excludes Ruination Guide itself ("other").
#[test]
fn ruination_guide_does_not_buff_itself() {
    let mut g = two_player_game();
    let rg = g.add_card_to_battlefield(0, catalog::ruination_guide());
    let cp = g.computed_permanent(rg).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2), "Ruination Guide is unaffected by its own anthem");
}

/// Dominator Drone drains 2 only when another colorless creature is present.
#[test]
fn dominator_drone_drains_with_another_colorless() {
    // Alone → no drain.
    let mut g = two_player_game();
    let d1 = g.add_card_to_hand(0, catalog::dominator_drone());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[1].life;
    crate::game::cast(&mut g, d1);
    assert_eq!(g.players[1].life, before, "no other colorless creature → no drain");
    // With a colorless creature already out → drain 2.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::eldrazi_devastator());
    let d2 = g.add_card_to_hand(0, catalog::dominator_drone());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[1].life;
    crate::game::cast(&mut g, d2);
    assert_eq!(g.players[1].life, before - 2, "another colorless creature → drain 2");
}

/// Blinding Drone taps a target creature for {C}{T}.
#[test]
fn blinding_drone_taps_target_creature() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::blinding_drone());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(id);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Permanent(target)), x_value: None,
    }).expect("tap target");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).unwrap().tapped, "target creature is tapped");
}

/// Kozilek's Translator taps for {C} by paying 1 life, once per turn.
#[test]
fn kozileks_translator_pays_life_for_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::kozileks_translator());
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("translate");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
    assert_eq!(g.players[0].life, life - 1, "paid 1 life");
    // Once-per-turn: a second activation is rejected.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).is_err(), "only once each turn");
}

/// Flayer Drone drains when another *Devoid* (colorless-despite-pips) creature
/// enters — exercising the Devoid-aware Colorless filter.
#[test]
fn flayer_drone_drains_on_devoid_creature_enter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::flayer_drone());
    let before = g.players[1].life;
    // Mist Intruder is Devoid (has a {U} pip) — must still count as colorless.
    let mist = g.add_card_to_hand(0, catalog::mist_intruder());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    crate::game::cast(&mut g, mist);
    assert_eq!(g.players[1].life, before - 1, "Devoid creature entering drains the opponent");
}

/// Kozilek's Sentinel grows +1/+0 when its controller casts a colorless spell.
#[test]
fn kozileks_sentinel_pumps_on_colorless_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::kozileks_sentinel());
    // Cast a colorless spell (generic-only cost → colorless).
    let spell = g.add_card_to_hand(0, catalog::eldrazi_devastator());
    g.players[0].mana_pool.add_colorless(8);
    crate::game::cast(&mut g, spell);
    let s = g.battlefield_find(id).unwrap();
    assert_eq!((s.power(), s.toughness()), (2, 4), "+1/+0 after a colorless cast");
}

/// Spawnsire's {4} ability mints two Eldrazi Spawn; it has Annihilator 1.
#[test]
fn spawnsire_makes_two_spawn() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::spawnsire_of_ulamog());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("make spawn");
    drain_stack(&mut g);
    let spawn = g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count();
    assert_eq!(spawn, 2);
    assert!(catalog::spawnsire_of_ulamog().keywords.contains(&crate::card::Keyword::Annihilator(1)));
}

/// Matter Reshaper's death puts a cheap permanent from the top onto the
/// battlefield, but a 4-MV permanent goes to hand instead.
#[test]
fn matter_reshaper_dies_puts_cheap_permanent() {
    let mut g = two_player_game();
    // Top card is a 2-MV creature → onto battlefield.
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::matter_reshaper());
    g.battlefield_find_mut(id).unwrap().damage = 99;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "MV-2 permanent enters from top");
}

/// A too-expensive top card goes to hand instead of the battlefield.
#[test]
fn matter_reshaper_expensive_top_goes_to_hand() {
    let mut g = two_player_game();
    let big = g.add_card_to_library(0, catalog::eldrazi_devastator()); // MV 8
    let id = g.add_card_to_battlefield(0, catalog::matter_reshaper());
    g.battlefield_find_mut(id).unwrap().damage = 99;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == big), "MV-8 permanent goes to hand");
    assert!(!g.battlefield.iter().any(|c| c.id == big));
}

/// Hand of Emrakul is a 7/7 with Annihilator 1.
#[test]
fn hand_of_emrakul_is_annihilator_one() {
    let def = catalog::hand_of_emrakul();
    assert_eq!((def.power, def.toughness), (7, 7));
    assert!(def.keywords.contains(&crate::card::Keyword::Annihilator(1)));
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

// ── Process (CR — Battle for Zendikar / OGW) ─────────────────────────────────

/// Wasteland Strangler processes an opponent-owned exile card on ETB; if it
/// does, target creature gets -3/-3 (here lethal to a 3/3).
#[test]
fn wasteland_strangler_processes_then_shrinks_target() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let exiled = g.add_card_to_exile(1, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    // Accept the "you may process" decision.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let str_id = g.add_card_to_battlefield(0, catalog::wasteland_strangler());
    g.fire_self_etb_triggers(str_id, 0);
    drain_stack(&mut g);
    assert!(!g.exile.iter().any(|c| c.id == exiled), "exile card was processed away");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == exiled), "into owner's graveyard");
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "3/3 with -3/-3 dies as SBA");
}

/// With no opponent-owned exile card, Wasteland Strangler can't process, so
/// the "if you do" -3/-3 rider is skipped.
#[test]
fn wasteland_strangler_no_exile_skips_rider() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let str_id = g.add_card_to_battlefield(0, catalog::wasteland_strangler());
    g.fire_self_etb_triggers(str_id, 0);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == victim), "no process → no -3/-3, victim lives");
}

/// Mind Raker processes on ETB; if it does, each opponent discards a card.
#[test]
fn mind_raker_processes_then_each_opponent_discards() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let exiled = g.add_card_to_exile(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let hand_before = g.players[1].hand.len();
    let mr = g.add_card_to_battlefield(0, catalog::mind_raker());
    g.fire_self_etb_triggers(mr, 0);
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == exiled), "processed card");
    assert_eq!(g.players[1].hand.len(), hand_before - 1, "opponent discarded one");
}

/// Blight Herder's cast trigger processes two exile cards; if it does, makes
/// three Eldrazi Scions.
#[test]
fn blight_herder_processes_two_then_makes_three_scions() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let e1 = g.add_card_to_exile(1, catalog::grizzly_bears());
    let e2 = g.add_card_to_exile(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let bh = g.add_card_to_hand(0, catalog::blight_herder());
    g.players[0].mana_pool.add_colorless(5);
    let scions_before = g.battlefield.iter()
        .filter(|c| c.definition.name == "Eldrazi Scion").count();
    crate::game::cast(&mut g, bh); // cast trigger processes on the stack
    assert!(!g.exile.iter().any(|c| c.id == e1 || c.id == e2), "both processed");
    let scions_after = g.battlefield.iter()
        .filter(|c| c.definition.name == "Eldrazi Scion").count();
    assert_eq!(scions_after - scions_before, 3, "three Scions created");
}

/// Eldrazi Aggressor gains haste only while another colorless creature is on
/// its controller's battlefield (static keyword grant gated by a live predicate).
#[test]
fn eldrazi_aggressor_has_haste_only_with_another_colorless() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let agg = g.add_card_to_battlefield(0, catalog::eldrazi_aggressor());
    // Alone: no haste.
    assert!(!g.computed_permanent(agg).unwrap().keywords.contains(&Keyword::Haste),
        "no haste while it is the only colorless creature");
    // Add another colorless creature → haste turns on.
    g.add_card_to_battlefield(0, catalog::eldrazi_devastator());
    assert!(g.computed_permanent(agg).unwrap().keywords.contains(&Keyword::Haste),
        "haste while controlling another colorless creature");
}

/// Reaver Drone costs you 1 life at upkeep unless you control another
/// colorless creature.
#[test]
fn reaver_drone_upkeep_life_loss_unless_another_colorless() {
    let mut g = two_player_game();
    let drone = g.add_card_to_battlefield(0, catalog::reaver_drone());
    // Lone Reaver Drone: lose 1 at upkeep.
    let before = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before - 1, "lone drone bleeds 1 at upkeep");
    // Add another colorless creature → no life loss.
    g.add_card_to_battlefield(0, catalog::eldrazi_devastator());
    let _ = drone;
    let before2 = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before2, "no bleed with another colorless creature");
}

/// Void Grafter has Flash and grants hexproof to another of your creatures on ETB.
#[test]
fn void_grafter_grants_hexproof_on_etb() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(catalog::void_grafter().keywords.contains(&Keyword::Flash));
    let vg = g.add_card_to_battlefield(0, catalog::void_grafter());
    g.fire_self_etb_triggers(vg, 0);
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof),
        "the other creature gains hexproof");
}

/// Brood Butcher mints a Scion on ETB and can sacrifice a creature to give a
/// target -1/-1.
#[test]
fn brood_butcher_sacrifices_to_shrink_target() {
    let mut g = two_player_game();
    let bb = g.add_card_to_hand(0, catalog::brood_butcher());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    crate::game::cast(&mut g, bb); // ETB makes a Scion
    let scion = g.battlefield.iter().find(|c| c.definition.name == "Eldrazi Scion").unwrap().id;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bb, ability_index: 0,
        target: Some(Target::Permanent(victim)), x_value: None,
    }).expect("sac scion to shrink target");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == scion), "Scion sacrificed as cost");
    let cp = g.computed_permanent(victim).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "2/2 with -1/-1");
}

/// Lifespring Druid taps for one mana of any color.
#[test]
fn lifespring_druid_taps_for_any_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lifespring_druid());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.total(), 1, "produced one mana");
}

/// Havoc Sower pumps itself +2/+1 for {1}{C}.
#[test]
fn havoc_sower_pumps_itself() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::havoc_sower());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (5, 4), "3/3 → 5/4 after +2/+1");
}

/// Defiant Bloodlord drains an opponent whenever its controller gains life.
#[test]
fn defiant_bloodlord_drains_on_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::defiant_bloodlord());
    let opp_before = g.players[1].life;
    g.adjust_life(0, 3); // gain 3 life
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 3 }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3, "opponent loses life equal to life gained");
}

/// Cinder Hellion deals 2 to an opponent on ETB.
#[test]
fn cinder_hellion_etb_burns_opponent() {
    let mut g = two_player_game();
    let before = g.players[1].life;
    let id = g.add_card_to_battlefield(0, catalog::cinder_hellion());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 2, "ETB deals 2 to the opponent");
}

/// Natural State destroys a cheap artifact.
#[test]
fn natural_state_destroys_cheap_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::natural_state());
    g.players[0].mana_pool.add(Color::Green, 1);
    crate::game::cast_at(&mut g, id, Target::Permanent(art));
    assert!(!g.battlefield.iter().any(|c| c.id == art), "MV-2 artifact destroyed");
}

/// Eldrazi Mimic copies the base P/T of a colorless creature that enters under
/// its controller until end of turn.
#[test]
fn eldrazi_mimic_copies_base_pt_of_entering_colorless() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let mimic = g.add_card_to_battlefield(0, catalog::eldrazi_mimic());
    assert_eq!(g.computed_permanent(mimic).unwrap().power, 2, "starts 2/1");
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    // An 8/9 colorless Eldrazi enters under our control.
    let big = g.add_card_to_battlefield(0, catalog::eldrazi_devastator());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: big }]);
    drain_stack(&mut g);
    let cp = g.computed_permanent(mimic).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 9), "Mimic becomes 8/9 to match");
}

/// Stormrider Spirit is a 3/3 with Flash and Flying.
#[test]
fn stormrider_spirit_has_flash_and_flying() {
    use crate::card::Keyword;
    let def = catalog::stormrider_spirit();
    assert_eq!((def.power, def.toughness), (3, 3));
    assert!(def.keywords.contains(&Keyword::Flash) && def.keywords.contains(&Keyword::Flying));
}

/// Make a Stand pumps your team +1/+0 and grants indestructible.
#[test]
fn make_a_stand_pumps_and_protects() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // opponent's untouched
    let id = g.add_card_to_hand(0, catalog::make_a_stand());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    crate::game::cast(&mut g, id);
    let cp = g.computed_permanent(mine).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2), "your creature is +1/+0");
    assert!(cp.keywords.contains(&Keyword::Indestructible), "and indestructible");
}

/// Flaying Tendrils sweeps for -2/-2 and exiles the dead instead of bin.
#[test]
fn flaying_tendrils_sweeps_and_exiles() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies
    let id = g.add_card_to_hand(0, catalog::flaying_tendrils());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    crate::game::cast(&mut g, id);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "2/2 died to -2/-2");
    assert!(g.exile.iter().any(|c| c.id == victim), "exiled instead of graveyard");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == victim), "not in graveyard");
}
