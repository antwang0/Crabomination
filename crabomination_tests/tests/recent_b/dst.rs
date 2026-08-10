//! Darksteel gap batch (`decks::recent310`).

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, GameEvent, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// Swing `attacker` (seat 1's) into seat 0 and run combat to end of combat.
fn swing_at_seat_zero(g: &mut GameState, attacker: crabomination::card::CardId) {
    use crabomination::game::types::{Attack, AttackTarget};
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

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
        card_id: brute, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
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
            x_value: None, mode: None,
        })
        .expect("sac a land to draw");
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(glass).unwrap().counter_count(CounterType::Charge), 3);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: glass, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
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
        additional_targets: vec![], x_value: None, mode: None,
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
    let overseer = g.add_card_to_battlefield_with_counters(0, catalog::arcbound_overseer());
    let worker = g.add_card_to_battlefield_with_counters(0, catalog::arcbound_worker());
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
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
    let crusher = g.add_card_to_battlefield_with_counters(0, catalog::arcbound_crusher());
    assert_eq!(g.battlefield_find(crusher).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let other = g.add_card_to_battlefield(1, catalog::coretapper());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: other }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(crusher).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Arcbound Reclaimer cashes a counter for an artifact off the graveyard.
#[test]
fn arcbound_reclaimer_tucks_an_artifact_from_the_graveyard() {
    let mut g = main_phase();
    let reclaimer = g.add_card_to_battlefield_with_counters(0, catalog::arcbound_reclaimer());
    let buried = g.add_card_to_graveyard(0, catalog::coretapper());
    g.perform_action(GameAction::ActivateAbility {
        card_id: reclaimer, ability_index: 0, target: Some(Target::Permanent(buried)),
        additional_targets: vec![], x_value: None, mode: None,
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
    let beast = g.add_card_to_battlefield_with_counters(0, catalog::karstoderm());
    assert_eq!(g.battlefield_find(beast).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
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
        additional_targets: vec![], x_value: None, mode: None,
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
            x_value: None, mode: None,
        })
        .expect("sacrifice a creature");
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(pit).unwrap().counter_count(CounterType::Charge), 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pit, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
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
        additional_targets: vec![], x_value: None, mode: None,
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

// ── Darksteel completion batch 2 (`decks::recent312`) ──

/// Thunderstaff shaves a point off each attacker while it's untapped.
#[test]
fn thunderstaff_shaves_combat_damage_while_untapped() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::thunderstaff());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    swing_at_seat_zero(&mut g, bear);
    assert_eq!(g.players[0].life, 19, "2 power minus 1");
}

/// A tapped Thunderstaff shaves nothing.
#[test]
fn tapped_thunderstaff_shaves_nothing() {
    let mut g = main_phase();
    let staff = g.add_card_to_battlefield(0, catalog::thunderstaff());
    g.battlefield_find_mut(staff).unwrap().tapped = true;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    swing_at_seat_zero(&mut g, bear);
    assert_eq!(g.players[0].life, 18);
}

/// Neurok Transmuter's second mode strips the artifact type for the turn.
#[test]
fn neurok_transmuter_unmakes_an_artifact_creature() {
    let mut g = main_phase();
    let wizard = g.add_card_to_battlefield(0, catalog::neurok_transmuter());
    g.battlefield_find_mut(wizard).unwrap().summoning_sick = false;
    let myr = g.add_card_to_battlefield(1, catalog::coretapper());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wizard, ability_index: 1, target: Some(Target::Permanent(myr)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("strip the artifact type");
    drain_stack(&mut g);
    let cp = g.computed_permanent(myr).unwrap();
    assert!(!cp.card_types.contains(&CardType::Artifact));
    assert!(cp.colors.contains(Color::Blue));
}

/// Chimeric Egg charges off opponents' nonartifact spells, then animates.
#[test]
fn chimeric_egg_charges_then_animates() {
    let mut g = main_phase();
    let egg = g.add_card_to_battlefield(0, catalog::chimeric_egg());
    g.battlefield_find_mut(egg).unwrap().add_counters(CounterType::Charge, 2);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("opponent casts a nonartifact spell");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(egg).unwrap().counter_count(CounterType::Charge), 3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: egg, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("cash three charges");
    drain_stack(&mut g);
    let cp = g.computed_permanent(egg).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Talon of Pain banks a charge per damaging source and fires them back.
#[test]
fn talon_of_pain_banks_damage_then_burns() {
    let mut g = main_phase();
    let talon = g.add_card_to_battlefield(0, catalog::talon_of_pain());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("burn the opponent");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(talon).unwrap().counter_count(CounterType::Charge), 1);
    g.battlefield_find_mut(talon).unwrap().add_counters(CounterType::Charge, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: talon, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: Some(2), mode: None,
    })
    .expect("spend two charges");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20 - 3 - 2);
    // CR-faithful: "a source you control OTHER than this artifact", so its own
    // shot doesn't re-charge it.
    assert_eq!(g.battlefield_find(talon).unwrap().counter_count(CounterType::Charge), 0);
}

/// Test of Faith turns prevented damage into +1/+1 counters.
#[test]
fn test_of_faith_converts_prevented_damage_to_counters() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let test = g.add_card_to_hand(0, catalog::test_of_faith());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: test, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("bolt it");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).expect("the Bear lives");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Death Cloud bleeds both players for X across four clauses.
#[test]
fn death_cloud_hits_life_hand_creatures_and_lands() {
    let mut g = main_phase();
    for p in 0..2 {
        g.add_card_to_hand(p, catalog::grizzly_bears());
        g.add_card_to_battlefield(p, catalog::grizzly_bears());
        g.add_card_to_battlefield(p, catalog::forest());
    }
    let cloud = g.add_card_to_hand(0, catalog::death_cloud());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: cloud, target: None, additional_targets: vec![], mode: None, x_value: Some(1),
    })
    .expect("cast for X=1");
    drain_stack(&mut g);
    for p in 0..2 {
        assert_eq!(g.players[p].life, 19, "player {p} lost 1");
        assert_eq!(g.players[p].hand.len(), 0, "player {p} discarded");
    }
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_creature()).count(), 0);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), 0);
}

/// Lich's Tomb keeps you alive but charges a permanent per point.
#[test]
fn lichs_tomb_trades_permanents_for_life() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::lichs_tomb());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.adjust_life_applied(0, -2);
    g.dispatch_triggers_for_events(&[GameEvent::LifeLost { player: 0, amount: 2 }]);
    drain_stack(&mut g);
    let after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(before - after, 2, "one permanent per point of life lost");
}

/// Heartseeker's unattach shot kills a creature and taps its host.
#[test]
fn heartseeker_unattaches_to_kill() {
    let mut g = main_phase();
    let kor = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blade = g.add_card_to_battlefield(0, catalog::heartseeker());
    g.battlefield_find_mut(blade).unwrap().attached_to = Some(kor);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(kor).map(|c| (c.power, c.toughness)), Some((4, 3)));
    g.perform_action(GameAction::ActivateAbility {
        card_id: blade, ability_index: 0, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("fire the Heartseeker");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert_eq!(g.battlefield_find(blade).unwrap().attached_to, None);
}

/// Pulse of the Fields comes back while you're behind on life.
#[test]
fn pulse_of_the_fields_rebuys_while_behind() {
    let mut g = main_phase();
    g.players[0].life = 10;
    let pulse = g.add_card_to_hand(0, catalog::pulse_of_the_fields());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: pulse, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 14);
    assert!(g.players[0].hand.iter().any(|c| c.id == pulse), "still behind, so it bounces");
}

/// Drooling Ogre defects to whoever cast an artifact spell.
#[test]
fn drooling_ogre_defects_on_an_artifact_cast() {
    let mut g = main_phase();
    let ogre = g.add_card_to_battlefield(0, catalog::drooling_ogre());
    let rock = g.add_card_to_hand(1, catalog::coretapper());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: rock, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast an artifact");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ogre).unwrap().controller, 1);
}

/// Chromescale Drake keeps the artifacts it reveals and bins the rest.
#[test]
fn chromescale_drake_keeps_revealed_artifacts() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::coretapper());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::coretapper());
    let drake = g.add_card_to_battlefield(0, catalog::chromescale_drake());
    g.fire_self_etb_triggers(drake, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2, "both artifacts");
    assert_eq!(g.players[0].graveyard.len(), 1, "the Bear is binned");
}

// ── Darksteel completion batch 3 (`decks::recent313`) ──

/// Pristine Angel's protection suite is live only while it's untapped.
#[test]
fn pristine_angel_loses_protection_when_tapped() {
    let mut g = main_phase();
    let angel = g.add_card_to_battlefield(0, catalog::pristine_angel());
    let pro_red = Keyword::Protection(Color::Red);
    assert!(g.computed_permanent(angel).unwrap().keywords.contains(&pro_red));
    assert!(g
        .computed_permanent(angel)
        .unwrap()
        .keywords
        .contains(&Keyword::ProtectionFromCardType(CardType::Artifact)));
    g.battlefield_find_mut(angel).unwrap().tapped = true;
    assert!(!g.computed_permanent(angel).unwrap().keywords.contains(&pro_red));
}

/// Screams from Within crawls back out of the graveyard when its host dies.
#[test]
fn screams_from_within_returns_when_the_host_dies() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::screams_from_within());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("enchant the Bear");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).map(|c| (c.power, c.toughness)), Some((1, 1)));
    let mut events = vec![];
    g.destroy_permanent(bear, false, &mut events);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Screams from Within"),
        "the Aura returns from the graveyard",
    );
}

/// Roaring Slagwurm locks down every artifact when it swings.
#[test]
fn roaring_slagwurm_taps_all_artifacts_on_attack() {
    let mut g = main_phase();
    let wurm = g.add_card_to_battlefield(0, catalog::roaring_slagwurm());
    let mine = g.add_card_to_battlefield(0, catalog::coretapper());
    let theirs = g.add_card_to_battlefield(1, catalog::coretapper());
    g.clear_sickness(wurm);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![crabomination::game::types::Attack {
        attacker: wurm,
        target: crabomination::game::types::AttackTarget::Player(1),
    }])
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).unwrap().tapped);
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}

/// Psychic Overload taps its host, locks the untap step, and sells an escape.
#[test]
fn psychic_overload_locks_and_sells_an_escape() {
    let mut g = main_phase();
    let rock = g.add_card_to_battlefield(1, catalog::coretapper());
    let aura = g.add_card_to_hand(0, catalog::psychic_overload());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(rock)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("enchant it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).unwrap().tapped, "the ETB taps it");
    assert!(g.untap_prevented_by_static(rock), "and it stays down");
}

/// Oxidize and Purge both blank regeneration before they destroy.
#[test]
fn oxidize_and_purge_beat_regeneration() {
    for (def, victim) in [
        (catalog::oxidize(), catalog::coretapper()),
        (catalog::purge(), catalog::coretapper()),
    ] {
        let mut g = main_phase();
        let target = g.add_card_to_battlefield(1, victim);
        g.battlefield_find_mut(target).unwrap().regeneration_shields = 1;
        let spell = g.add_card_to_hand(0, def);
        for c in [Color::Green, Color::White] {
            g.players[0].mana_pool.add(c, 1);
        }
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(target)), additional_targets: vec![],
            mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(target).is_none(), "the shield doesn't save it");
    }
}

/// Shield of Kaldra blankets the whole Kaldra set in indestructible.
#[test]
fn shield_of_kaldra_protects_its_siblings() {
    let mut g = main_phase();
    let shield = g.add_card_to_battlefield(0, catalog::shield_of_kaldra());
    assert!(g
        .computed_permanent(shield)
        .unwrap()
        .keywords
        .contains(&Keyword::Indestructible));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(shield).unwrap().attached_to = Some(bear);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Indestructible));
}

/// Shunt repoints a single-target spell (CR 115.7).
#[test]
fn shunt_repoints_a_single_target_spell() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::coretapper());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(mine)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("they bolt your Bear");
    let shunt = g.add_card_to_hand(0, catalog::shunt());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Target(Target::Permanent(theirs)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: shunt, target: Some(Target::Permanent(bolt)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast Shunt at the Bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_some(), "your Bear is spared");
    assert!(g.battlefield_find(theirs).is_none(), "the Bolt lands on their rock");
}

/// Savage Beating's entwined halves need combat on your turn.
#[test]
fn savage_beating_only_casts_during_your_combat() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let beating = g.add_card_to_hand(0, catalog::savage_beating());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: beating, target: None, additional_targets: vec![], mode: Some(0),
            x_value: None,
        })
        .is_err(),
        "a main phase isn't combat",
    );
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::CastSpell {
        card_id: beating, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    })
    .expect("combat on your turn is fine");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

/// Scrounge steals an artifact out of an opponent's graveyard.
#[test]
fn scrounge_reanimates_an_opposing_artifact() {
    let mut g = main_phase();
    let buried = g.add_card_to_graveyard(1, catalog::coretapper());
    let spell = g.add_card_to_hand(0, catalog::scrounge());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(buried)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(buried).map(|c| c.controller), Some(0));
}

// ── Darksteel completion batch 4 (`decks::recent314`) ──

/// Mycosynth Lattice lets any mana pay a coloured pip and blanks colour.
#[test]
fn mycosynth_lattice_relaxes_colors_and_makes_everything_an_artifact() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bear).unwrap().colors.is_empty(), "a Bear starts green");
    g.add_card_to_battlefield(0, catalog::mycosynth_lattice());
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.colors.is_empty(), "everything is colourless");
    assert!(cp.card_types.contains(&CardType::Artifact), "everything is an artifact");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("{C} pays {R} under the Lattice");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Gemini Engine's Twin copies the Engine's live P/T.
#[test]
fn gemini_engine_twin_copies_pumped_pt() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase();
    let engine = g.add_card_to_battlefield(0, catalog::gemini_engine());
    g.clear_sickness(engine);
    // A +1/+1 counter makes the Engine a 4/5; the Twin must follow.
    g.battlefield_find_mut(engine).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: engine, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    let twin = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Twin")
        .map(|c| c.id)
        .expect("Twin minted");
    let cp = g.computed_permanent(twin).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 5));
}

/// Auriok Siege Sled's second mode only bars blocking the Sled itself.
#[test]
fn auriok_siege_sled_forbids_only_its_own_block() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase();
    let sled = g.add_card_to_battlefield(0, catalog::auriok_siege_sled());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::ornithopter());
    g.clear_sickness(sled);
    g.clear_sickness(other);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sled, ability_index: 1, target: Some(Target::Permanent(blocker)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("forbid the block");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![
        Attack { attacker: sled, target: AttackTarget::Player(1) },
        Attack { attacker: other, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, sled)])).is_err(),
        "it can't block the Sled",
    );
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, other)]))
        .expect("but it can block anything else");
}

/// Synod Artificer taps exactly X noncreature artifacts.
#[test]
fn synod_artificer_taps_x_artifacts() {
    let mut g = main_phase();
    let artificer = g.add_card_to_battlefield(0, catalog::synod_artificer());
    g.clear_sickness(artificer);
    let a = g.add_card_to_battlefield(1, catalog::angels_feather());
    let b = g.add_card_to_battlefield(1, catalog::demons_horn());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: artificer, ability_index: 0, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], x_value: Some(2), mode: None,
    })
    .expect("tap two");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped);
    assert!(g.battlefield_find(b).unwrap().tapped);
}

/// One target isn't enough for X = 2 (CR 601.2c).
#[test]
fn synod_artificer_rejects_too_few_targets() {
    let mut g = main_phase();
    let artificer = g.add_card_to_battlefield(0, catalog::synod_artificer());
    g.clear_sickness(artificer);
    let a = g.add_card_to_battlefield(1, catalog::angels_feather());
    g.players[0].mana_pool.add_colorless(2);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: artificer, ability_index: 0, target: Some(Target::Permanent(a)),
            additional_targets: vec![], x_value: Some(2), mode: None,
        })
        .is_err(),
        "X = 2 needs two targets",
    );
}

/// Thought Dissector digs X deep, steals the artifact and sacrifices itself.
#[test]
fn thought_dissector_steals_an_artifact_and_sacrifices_itself() {
    let mut g = main_phase();
    let dissector = g.add_card_to_battlefield(0, catalog::thought_dissector());
    g.players[1].library.clear();
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    let rock = g.add_card_to_library(1, catalog::coretapper());
    // Library front is the top of the deck: Bear, then the artifact.
    assert_eq!(g.players[1].library[0].id, bear);
    assert_eq!(g.players[1].library[1].id, rock);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dissector, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: Some(3), mode: None,
    })
    .expect("dig three");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rock).map(|c| c.controller), Some(0), "you keep the artifact");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "the miss is milled");
    assert!(g.battlefield_find(dissector).is_none(), "the Dissector is sacrificed");
}

/// Thought Dissector survives a whiff.
#[test]
fn thought_dissector_survives_finding_nothing() {
    let mut g = main_phase();
    let dissector = g.add_card_to_battlefield(0, catalog::thought_dissector());
    g.players[1].library.clear();
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dissector, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: Some(1), mode: None,
    })
    .expect("dig one");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dissector).is_some(), "no artifact, no sacrifice");
}

/// Hallow zeroes a burn spell and refunds the damage as life.
#[test]
fn hallow_prevents_a_spells_damage_and_gains_that_much_life() {
    let mut g = main_phase();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("they aim a Bolt at you");
    let hallow = g.add_card_to_hand(0, catalog::hallow());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: hallow, target: Some(Target::Permanent(bolt)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Hallow the Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23, "3 prevented, 3 gained");
}

/// Dismantle hands the destroyed artifact's counter total on.
#[test]
fn dismantle_moves_the_counter_tally_to_your_artifact() {
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::coretapper());
    g.battlefield_find_mut(theirs).unwrap().add_counters(CounterType::Charge, 3);
    let mine = g.add_card_to_battlefield(0, catalog::angels_feather());
    let spell = g.add_card_to_hand(0, catalog::dismantle());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(theirs)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "destroyed");
    assert_eq!(
        g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3,
        "three counters follow (the auto-pick takes the first listed kind)",
    );
}

/// Pulse of the Dross discards a revealed card and recurs while they're ahead.
#[test]
fn pulse_of_the_dross_discards_and_returns_itself() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::pulse_of_the_dross());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 1, "one card discarded");
    assert!(
        g.players[0].hand.iter().any(|c| c.id == spell),
        "they still hold more than you, so it comes back",
    );
}

/// Turn the Tables sends only combat damage at the chosen attacker.
#[test]
fn turn_the_tables_redirects_combat_damage_to_an_attacker() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let spell = g.add_card_to_hand(0, catalog::turn_the_tables());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(attacker)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast at the attacker");
    drain_stack(&mut g);
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, 20, "you take nothing");
    assert!(g.battlefield_find(attacker).is_none(), "the Bear kills itself");
}

/// Shriveling Rot mode 1 destroys any creature that takes damage.
#[test]
fn shriveling_rot_destroys_damaged_creatures() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_blossoms());
    let rot = g.add_card_to_hand(0, catalog::shriveling_rot());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: rot, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    })
    .expect("cast mode 1");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(wall)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("ping the 0/4");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wall).is_none(), "3 damage on a 0/4 is still lethal now");
}

/// Shriveling Rot mode 2 drains the dying creature's controller.
#[test]
fn shriveling_rot_drains_toughness_on_death() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_blossoms());
    let rot = g.add_card_to_hand(0, catalog::shriveling_rot());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: rot, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("cast mode 2");
    drain_stack(&mut g);
    let kill = g.add_card_to_hand(0, catalog::doom_blade());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: kill, target: Some(Target::Permanent(wall)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("kill the Wall");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wall).is_none());
    assert_eq!(g.players[1].life, 16, "the 0/4 drains its controller for 4");
}

/// Death-Mask Duplicant wears the imprinted creature's evasion.
#[test]
fn death_mask_duplicant_gains_imprinted_keywords() {
    let mut g = main_phase();
    let dup = g.add_card_to_battlefield(0, catalog::death_mask_duplicant());
    let flier = g.add_card_to_graveyard(0, catalog::serra_angel());
    assert!(!g.computed_permanent(dup).unwrap().keywords.contains(&Keyword::Flying));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dup, ability_index: 0, target: Some(Target::Permanent(flier)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("imprint the Angel");
    drain_stack(&mut g);
    let kws = g.computed_permanent(dup).unwrap().keywords.clone();
    assert!(kws.contains(&Keyword::Flying), "flying bleeds through");
    assert!(!kws.contains(&Keyword::Vigilance), "vigilance isn't on the printed list");
}

/// Panoptic Mirror imprints an X-matching spell and recasts it free each upkeep.
#[test]
fn panoptic_mirror_recasts_the_imprinted_spell_for_free() {
    let mut g = main_phase();
    let mirror = g.add_card_to_battlefield(0, catalog::panoptic_mirror());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mirror, ability_index: 0, target: None, additional_targets: vec![],
        x_value: Some(1), mode: None,
    })
    .expect("imprint a 1-drop instant");
    drain_stack(&mut g);
    assert!(
        g.exile.iter().any(|c| c.id == bolt && c.exiled_with == Some(mirror)),
        "the Bolt is imprinted",
    );
    // Two yes/no asks: the printed "you may copy", then the free cast itself.
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.step, TurnStep::Upkeep);
    assert_eq!(g.players[1].life, 17, "the free copy resolves");
    assert!(g.exile.iter().any(|c| c.id == bolt), "the original stays imprinted");
}
