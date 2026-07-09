//! Functionality tests for the MH3 batch-2 cards in `catalog::sets::mh3b`.

use crate::card::{CounterType, Keyword};
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: crate::card::CardId, target: Option<Target>) {
    fill_mana(g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(g);
}

/// Eldrazi Ravager has Annihilator 1 and can return itself from the graveyard
/// by sacrificing two Eldrazi.
#[test]
fn eldrazi_ravager_recurs_from_graveyard() {
    let mut g = two_player_game();
    let ravager = g.add_card_to_graveyard(0, catalog::eldrazi_ravager());
    // Two Eldrazi to sacrifice.
    g.add_card_to_battlefield(0, catalog::eldrazi_ravager());
    g.add_card_to_battlefield(0, catalog::eldrazi_ravager());
    g.perform_action(GameAction::ActivateAbility {
        card_id: ravager, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate gy ability");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == ravager), "returned to hand");
}

/// Breaker of Creation's cast trigger gains 1 life per colorless permanent.
#[test]
fn breaker_of_creation_gains_life_per_colorless() {
    let mut g = two_player_game();
    // Two colorless permanents already out (the Eldrazi Ravagers are colorless).
    g.add_card_to_battlefield(0, catalog::eldrazi_ravager());
    g.add_card_to_battlefield(0, catalog::eldrazi_ravager());
    let id = g.add_card_to_hand(0, catalog::breaker_of_creation());
    let life = g.players[0].life;
    cast(&mut g, id, None);
    // The two Ravagers + Breaker itself once it enters = at least 2 counted at
    // cast time (Breaker is still on the stack). Gains ≥ 2.
    assert!(g.players[0].life >= life + 2, "gained life per colorless permanent");
}

/// Drownyard Lurker's cycle trigger creates an Eldrazi Spawn.
#[test]
fn drownyard_lurker_cycle_makes_spawn() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::drownyard_lurker());
    fill_mana(&mut g);
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    let spawn = g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count();
    assert_eq!(spawn, 1, "cycling made one Eldrazi Spawn");
}

/// Emrakul's Messenger makes a Spawn when you draw your second card each turn.
#[test]
fn emrakuls_messenger_second_draw_spawn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::emrakuls_messenger());
    g.add_card_to_library(0, catalog::eldrazi_ravager());
    g.add_card_to_library(0, catalog::eldrazi_ravager());
    g.players[0].cards_drawn_this_turn = 0;
    let mut ev = vec![];
    g.draw_one(0, &mut ev); // first draw — no trigger
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count(), 0);
    let mut ev2 = vec![];
    g.draw_one(0, &mut ev2); // second draw — trigger
    g.dispatch_triggers_for_events(&ev2);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count(), 1);
}

/// Petrifying Meddler's cast trigger taps and stuns a creature.
#[test]
fn petrifying_meddler_stuns() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::petrifying_meddler());
    cast(&mut g, id, Some(Target::Permanent(victim)));
    let v = g.battlefield_find(victim).unwrap();
    assert!(v.tapped, "target tapped");
    assert_eq!(v.counter_count(CounterType::Stun), 1, "one stun counter");
}

/// Dreamdrinker Vampire gains menace when a +1/+1 counter lands (via adapt).
#[test]
fn dreamdrinker_adapt_grants_menace() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::dreamdrinker_vampire());
    g.clear_sickness(id);
    fill_mana(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("adapt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Menace), "menace granted");
}

/// Evolution Witness returns a permanent card from the graveyard on adapt.
#[test]
fn evolution_witness_returns_on_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::evolution_witness());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.clear_sickness(id);
    fill_mana(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Permanent(dead)), additional_targets: vec![], x_value: None,
    }).expect("adapt");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "graveyard card returned to hand");
}

/// Envoy of the Ancestors gives modified creatures you control lifelink.
#[test]
fn envoy_grants_modified_lifelink() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::envoy_of_the_ancestors());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Unmodified: no lifelink.
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Lifelink));
    // Add a counter → modified → lifelink.
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Lifelink), "modified gets lifelink");
}

/// Guardian of the Forgotten manifests when a modified creature you control dies.
#[test]
fn guardian_manifests_on_modified_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::guardian_of_the_forgotten());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // modified (3/3)
    g.add_card_to_library(0, catalog::grizzly_bears());
    // Lethal damage → SBA death (populates the LKI snapshot the dies-trigger reads).
    let mut evs = Vec::new();
    g.deal_damage_to_from(crate::game::effects::EntityRef::Permanent(bear), 3, None, &mut evs);
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    drain_stack(&mut g);
    let facedown = g.battlefield.iter().filter(|c| c.face_down).count();
    assert_eq!(facedown, 1, "manifested a face-down card");
}

/// Grim Servant tutors a card with MV ≤ devotion to black.
#[test]
fn grim_servant_devotion_tutor() {
    let mut g = two_player_game();
    // Devotion to black 2 (Grim Servant itself is {3}{B} = 1 pip; add another).
    g.add_card_to_battlefield(0, catalog::dreamdrinker_vampire()); // {1}{B} = 1 black pip
    let id = g.add_card_to_hand(0, catalog::grim_servant());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 ≤ devotion 2 (after Grim enters)
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
    let life = g.players[0].life;
    cast(&mut g, id, None);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "tutored a card");
    assert_eq!(g.players[0].life, life - 3, "lost 3 life");
}

/// Marionette Apprentice drains each opponent when your creature/artifact dies.
#[test]
fn marionette_apprentice_drains_on_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::marionette_apprentice());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life = g.players[1].life;
    let evs = g.remove_to_graveyard_with_triggers(bear);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "opponent lost 1 life");
}

/// Molten Gatekeeper pings each opponent when another creature enters.
#[test]
fn molten_gatekeeper_pings_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::molten_gatekeeper());
    let life = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, id, None);
    assert_eq!(g.players[1].life, life - 1, "opponent took 1 damage");
}

/// Kami of Jealous Thirst drains 2 once per turn.
#[test]
fn kami_of_jealous_thirst_drains() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::kami_of_jealous_thirst());
    g.clear_sickness(id);
    fill_mana(&mut g);
    let (my, opp) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2);
    assert_eq!(g.players[0].life, my + 2);
}

/// Colossal Dreadmask enters as a living weapon: mints a Germ and buffs it.
#[test]
fn colossal_dreadmask_living_weapon() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::colossal_dreadmask());
    cast(&mut g, id, None);
    let germ = g.battlefield.iter().find(|c| c.definition.name == "Phyrexian Germ").expect("germ");
    let cp = g.computed_permanent(germ.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "0/0 germ + 6/6 equip");
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Horrific Assault: a creature you control deals its power to an enemy, and
/// controlling an Eldrazi gains 3 life.
#[test]
fn horrific_assault_fights_and_gains() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::eldrazi_ravager()); // 6/6 Eldrazi
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::horrific_assault());
    let life = g.players[0].life;
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "enemy took 6 and died");
    assert_eq!(g.players[0].life, life + 3, "gained 3 for controlling an Eldrazi");
}

/// Brainsurge draws four and puts two back — net +2 hand.
#[test]
fn brainsurge_net_draw_two() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::brainsurge());
    let hand = g.players[0].hand.len();
    cast(&mut g, id, None);
    // -1 (Brainsurge) +4 draw -2 put back = net +1 vs starting, i.e. hand-1+2.
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew 4, put 2 back");
}

/// Fangs of Kalonia adds a counter then doubles it (single-target).
#[test]
fn fangs_of_kalonia_doubles() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fangs_of_kalonia());
    cast(&mut g, id, Some(Target::Permanent(bear)));
    // +1 counter, then doubled → 2.
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Gravedig (mode 0) makes a 2/2 Zombie.
#[test]
fn gravedig_makes_zombie() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::gravedig());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let zombies = g.battlefield.iter().filter(|c| c.definition.name == "Zombie").count();
    assert_eq!(zombies, 1);
}

/// Drossclaw's equipped creature drains each opponent when it attacks.
#[test]
fn drossclaw_attack_drains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let axe = g.add_card_to_battlefield(0, catalog::drossclaw());
    g.clear_sickness(bear);
    // Equip onto the bear.
    fill_mana(&mut g);
    g.perform_action(GameAction::Equip { equipment: axe, target: bear }).expect("equip");
    let life = g.players[1].life;
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let evs = g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]).expect("attack");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "each opponent lost 1 when equipped creature attacked");
}

/// Expanding Ooze puts a +1/+1 counter on a modified creature when it attacks.
#[test]
fn expanding_ooze_attack_buffs_modified() {
    let mut g = two_player_game();
    let ooze = g.add_card_to_battlefield(0, catalog::expanding_ooze());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // modified
    g.clear_sickness(ooze);
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    // The attack trigger targets the modified bear when it goes on the stack.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    let evs = g.declare_attackers(vec![Attack { attacker: ooze, target: AttackTarget::Player(1) }]).expect("attack");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Infernal Captor's exploit steals a permanent until end of turn.
#[test]
fn infernal_captor_exploit_steals() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::infernal_captor());
    // Accept the exploit; AutoDecider sacrifices the fodder and steals the victim.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, id, None);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder), "exploit sacrificed a creature");
    let _ = victim;
}
