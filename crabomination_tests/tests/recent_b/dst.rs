//! Darksteel gap batch (`decks::recent310`).

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, GameEvent, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// The Horn/Feather cycle watches every player's casts of its colour.
#[test]
fn color_watch_artifacts_gain_life_on_a_matching_cast() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::angels_feather());
    g.add_card_to_battlefield(0, catalog::demons_horn());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast a red spell");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "neither watches red");
    let raise = g.add_card_to_hand(0, catalog::raise_dead());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: raise, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast a black spell");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21, "Demon's Horn only");
}

/// Darksteel Forge blankets your artifacts in indestructible.
#[test]
fn darksteel_forge_makes_your_artifacts_indestructible() {
    let mut g = main_phase();
    let plain = g.add_card_to_battlefield(0, catalog::coretapper());
    assert!(!g
        .computed_permanent(plain)
        .unwrap()
        .keywords
        .contains(&Keyword::Indestructible));
    g.add_card_to_battlefield(0, catalog::darksteel_forge());
    assert!(g
        .computed_permanent(plain)
        .unwrap()
        .keywords
        .contains(&Keyword::Indestructible));
}

/// Darksteel Brute animates into a 2/2 Beast that keeps its artifact type.
#[test]
fn darksteel_brute_animates_into_a_beast() {
    let mut g = main_phase();
    let brute = g.add_card_to_battlefield(0, catalog::darksteel_brute());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: brute, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(brute).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.card_types.contains(&CardType::Creature));
    assert!(cp.card_types.contains(&CardType::Artifact));
}

/// Arcane Spyglass banks a charge per draw, then cashes three in for another.
#[test]
fn arcane_spyglass_charges_then_cashes_in() {
    let mut g = main_phase();
    let glass = g.add_card_to_battlefield(0, catalog::arcane_spyglass());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
        g.battlefield_find_mut(glass).unwrap().tapped = false;
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: glass, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None,
        })
        .expect("sac a land to draw");
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(glass).unwrap().counter_count(CounterType::Charge), 3);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: glass, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("remove three charges");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.battlefield_find(glass).unwrap().counter_count(CounterType::Charge), 0);
}

/// Coretapper's sacrifice charges an artifact twice.
#[test]
fn coretapper_sacrifices_for_two_charges() {
    let mut g = main_phase();
    let myr = g.add_card_to_battlefield(0, catalog::coretapper());
    let target = g.add_card_to_battlefield(0, catalog::arcane_spyglass());
    g.perform_action(GameAction::ActivateAbility {
        card_id: myr, ability_index: 1, target: Some(Target::Permanent(target)),
        additional_targets: vec![], x_value: None,
    })
    .expect("sacrifice for two");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::Charge), 2);
    assert!(g.battlefield_find(myr).is_none());
}

/// Drill-Skimmer only has shroud while another artifact creature is around.
#[test]
fn drill_skimmer_gains_shroud_with_a_friend() {
    let mut g = main_phase();
    let skimmer = g.add_card_to_battlefield(0, catalog::drill_skimmer());
    assert!(!g.computed_permanent(skimmer).unwrap().keywords.contains(&Keyword::Shroud));
    g.add_card_to_battlefield(0, catalog::coretapper());
    assert!(g.computed_permanent(skimmer).unwrap().keywords.contains(&Keyword::Shroud));
}

/// Dross Golem's affinity for Swamps discounts it.
#[test]
fn dross_golem_costs_less_per_swamp() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    let golem = g.add_card_to_hand(0, catalog::dross_golem());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: golem, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("{5} minus three Swamps");
    drain_stack(&mut g);
    assert!(g.battlefield_find(golem).is_some());
}

/// Auriok Glaivemaster only grows while it's carrying something.
#[test]
fn auriok_glaivemaster_grows_when_equipped() {
    let mut g = main_phase();
    let kor = g.add_card_to_battlefield(0, catalog::auriok_glaivemaster());
    assert_eq!(g.computed_permanent(kor).map(|c| (c.power, c.toughness)), Some((1, 1)));
    let sword = g.add_card_to_battlefield(0, catalog::short_bow());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(kor);
    let cp = g.computed_permanent(kor).unwrap();
    // 1/1 base + Short Bow's +1/+1 + the Glaivemaster's own equipped bonus.
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

/// Chittering Rats tucks a card off the opponent's hand.
#[test]
fn chittering_rats_tucks_a_card() {
    let mut g = main_phase();
    let theirs = g.add_card_to_hand(1, catalog::grizzly_bears());
    let rats = g.add_card_to_battlefield(0, catalog::chittering_rats());
    g.fire_self_etb_triggers(rats, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0);
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(theirs));
}

/// Burden of Greed scales with the target's tapped artifacts.
#[test]
fn burden_of_greed_counts_tapped_artifacts() {
    let mut g = main_phase();
    for _ in 0..3 {
        let a = g.add_card_to_battlefield(1, catalog::coretapper());
        g.battlefield_find_mut(a).unwrap().tapped = true;
    }
    g.add_card_to_battlefield(1, catalog::coretapper()); // untapped, doesn't count
    let burden = g.add_card_to_hand(0, catalog::burden_of_greed());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: burden, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Crazed Goblin has to swing.
#[test]
fn crazed_goblin_must_attack() {
    let g = main_phase();
    assert!(catalog::crazed_goblin().keywords.contains(&Keyword::MustAttack));
    let _ = g;
}

/// Carry Away steals the Equipment it enchants and knocks it loose.
#[test]
fn carry_away_steals_the_equipment() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(1, catalog::short_bow());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bear);
    let aura = g.add_card_to_hand(0, catalog::carry_away());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(sword)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast the Aura");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sword).unwrap().controller, 0, "you control it now");
}

// ── Darksteel completion batch (`decks::recent311`) ──

/// Modular N is a real keyword, and Arcbound Overseer's upkeep pump keys on it.
#[test]
fn arcbound_overseer_pumps_every_modular_creature() {
    let mut g = main_phase();
    g.step = TurnStep::Upkeep;
    let overseer = g.add_card_to_battlefield(0, catalog::arcbound_overseer());
    let worker = g.add_card_to_battlefield(0, catalog::arcbound_worker());
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(overseer).unwrap().add_counters(CounterType::PlusOnePlusOne, 6);
    g.battlefield_find_mut(worker).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(worker).unwrap().keywords.contains(&Keyword::Modular(1)));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(overseer).unwrap().counter_count(CounterType::PlusOnePlusOne), 7);
    assert_eq!(g.battlefield_find(worker).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield_find(plain).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Arcbound Crusher grows off any other artifact entering.
#[test]
fn arcbound_crusher_grows_on_another_artifact() {
    let mut g = main_phase();
    let crusher = g.add_card_to_battlefield(0, catalog::arcbound_crusher());
    g.battlefield_find_mut(crusher).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let other = g.add_card_to_battlefield(1, catalog::coretapper());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: other }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(crusher).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Arcbound Reclaimer cashes a counter for an artifact off the graveyard.
#[test]
fn arcbound_reclaimer_tucks_an_artifact_from_the_graveyard() {
    let mut g = main_phase();
    let reclaimer = g.add_card_to_battlefield(0, catalog::arcbound_reclaimer());
    g.battlefield_find_mut(reclaimer).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let buried = g.add_card_to_graveyard(0, catalog::coretapper());
    g.perform_action(GameAction::ActivateAbility {
        card_id: reclaimer, ability_index: 0, target: Some(Target::Permanent(buried)),
        additional_targets: vec![], x_value: None,
    })
    .expect("remove a counter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(buried));
    assert_eq!(g.battlefield_find(reclaimer).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Emissary of Despair drains for the defender's artifact count.
#[test]
fn emissary_of_despair_drains_per_artifact() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::coretapper());
    }
    let spirit = g.add_card_to_battlefield(0, catalog::emissary_of_despair());
    g.players[1].life -= 2;
    g.fire_combat_damage_to_player_triggers(spirit, 1, 2);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20 - 2 - 3);
}

/// Karstoderm shrinks as artifacts hit the table.
#[test]
fn karstoderm_sheds_a_counter_per_artifact() {
    let mut g = main_phase();
    let beast = g.add_card_to_battlefield(0, catalog::karstoderm());
    g.battlefield_find_mut(beast).unwrap().add_counters(CounterType::PlusOnePlusOne, 5);
    let art = g.add_card_to_battlefield(1, catalog::coretapper());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: art }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(beast).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

/// Echoing Decay reaches every creature sharing the target's name.
#[test]
fn echoing_decay_hits_all_same_named_creatures() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bystander = g.add_card_to_battlefield(1, catalog::coretapper());
    let decay = g.add_card_to_hand(0, catalog::echoing_decay());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: decay, target: Some(Target::Permanent(a)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both Bears die");
    assert!(g.battlefield_find(bystander).is_some());
}

/// Aether Snap strips every counter and exiles every token.
#[test]
fn aether_snap_clears_counters_and_tokens() {
    let mut g = main_phase();
    let charged = g.add_card_to_battlefield(0, catalog::arcane_spyglass());
    g.battlefield_find_mut(charged).unwrap().add_counters(CounterType::Charge, 3);
    let token = g.add_token_to_battlefield(
        1,
        &crabomination::card::TokenDefinition {
            name: "Myr".into(),
            card_types: vec![CardType::Artifact, CardType::Creature],
            power: 1,
            toughness: 1,
            ..Default::default()
        },
    );
    let snap = g.add_card_to_hand(0, catalog::aether_snap());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: snap, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(charged).unwrap().counter_count(CounterType::Charge), 0);
    assert!(g.battlefield_find(token).is_none(), "the token is exiled");
}

/// Darksteel Reactor wins the game once it hits twenty charge counters.
#[test]
fn darksteel_reactor_wins_at_twenty_charges() {
    let mut g = main_phase();
    let reactor = g.add_card_to_battlefield(0, catalog::darksteel_reactor());
    g.battlefield_find_mut(reactor).unwrap().add_counters(CounterType::Charge, 19);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(reactor).unwrap().counter_count(CounterType::Charge), 20);
    assert_eq!(g.game_over, Some(Some(0)), "twenty charges wins");
}

/// Eater of Days costs you the next two turns.
#[test]
fn eater_of_days_skips_two_turns() {
    let mut g = main_phase();
    let leviathan = g.add_card_to_battlefield(0, catalog::eater_of_days());
    g.fire_self_etb_triggers(leviathan, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].skip_turns, 2);
}

/// Myr Landshaper's artifact grant is an end-of-turn effect, not a permanent one.
#[test]
fn myr_landshaper_grant_expires_at_end_of_turn() {
    let mut g = main_phase();
    let myr = g.add_card_to_battlefield(0, catalog::myr_landshaper());
    g.battlefield_find_mut(myr).unwrap().summoning_sick = false;
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: myr, ability_index: 0, target: Some(Target::Permanent(land)),
        additional_targets: vec![], x_value: None,
    })
    .expect("animate the land's type line");
    drain_stack(&mut g);
    assert!(g.computed_permanent(land).unwrap().card_types.contains(&CardType::Artifact));
    let mut events = vec![];
    g.do_cleanup(&mut events);
    assert!(!g.computed_permanent(land).unwrap().card_types.contains(&CardType::Artifact));
}

/// Vulshok War Boar eats itself when you have no artifact to feed it.
#[test]
fn vulshok_war_boar_needs_an_artifact() {
    let mut g = main_phase();
    let boar = g.add_card_to_battlefield(0, catalog::vulshok_war_boar());
    g.fire_self_etb_triggers(boar, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(boar).is_none(), "no artifact to sacrifice");
}

/// Spawning Pit turns two sacrificed creatures into a 2/2 Spawn.
#[test]
fn spawning_pit_banks_charges_for_a_spawn() {
    let mut g = main_phase();
    let pit = g.add_card_to_battlefield(0, catalog::spawning_pit());
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.perform_action(GameAction::ActivateAbility {
            card_id: pit, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None,
        })
        .expect("sacrifice a creature");
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(pit).unwrap().counter_count(CounterType::Charge), 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pit, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("cash in two charges");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spawn"));
}

/// Leonin Bola taps a creature and falls off in the process.
#[test]
fn leonin_bola_taps_and_unattaches() {
    let mut g = main_phase();
    let kor = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bola = g.add_card_to_battlefield(0, catalog::leonin_bola());
    g.battlefield_find_mut(bola).unwrap().attached_to = Some(kor);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: bola, ability_index: 0, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], x_value: None,
    })
    .expect("tap the host, unattach, tap a creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped);
    assert!(g.battlefield_find(kor).unwrap().tapped, "the host paid the tap cost");
    assert_eq!(g.battlefield_find(bola).unwrap().attached_to, None);
}

/// Tanglewalker only slips your team past while a defender has an artifact land.
#[test]
fn tanglewalker_keys_on_an_opposing_artifact_land() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::tanglewalker());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
    g.add_card_to_battlefield(1, catalog::seat_of_the_synod());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
}

/// Soulscour leaves artifacts standing.
#[test]
fn soulscour_spares_artifacts() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let art = g.add_card_to_battlefield(1, catalog::coretapper());
    let scour = g.add_card_to_hand(0, catalog::soulscour());
    g.players[0].mana_pool.add(Color::White, 3);
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::CastSpell {
        card_id: scour, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(art).is_some(), "artifact creature survives");
}

/// Machinate digs as deep as your artifact count.
#[test]
fn machinate_digs_per_artifact() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::coretapper());
    }
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::machinate());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "one of the three looked at goes to hand");
}
