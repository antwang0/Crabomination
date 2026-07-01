//! Functionality tests for `catalog::sets::decks::recent61` — red/white aggro.

use crate::card::{CardDefinition, CardType, CreatureType, Keyword, Subtypes};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;

fn human(name: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

fn vanilla(name: &'static str, p: i32, t: i32) -> CardDefinition {
    CardDefinition { name, card_types: vec![CardType::Creature], power: p, toughness: t, ..Default::default() }
}

#[test]
fn kessig_malcontents_burns_for_humans() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, human("Villager A"));
    g.add_card_to_battlefield(0, human("Villager B"));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    let life = g.players[1].life;
    // Kessig is itself a Human → 3 Humans total → 3 damage.
    let k = g.add_card_to_battlefield(0, catalog::kessig_malcontents());
    g.fire_self_etb_triggers(k, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "3 damage = number of Humans");
}

#[test]
fn somberwald_vigilante_pings_its_blocker() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::somberwald_vigilante());
    g.clear_sickness(atk);
    let blocker = g.add_card_to_battlefield(1, vanilla("Weakling", 1, 1));
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, atk)])).unwrap();
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(blocker).is_none(), "1/1 blocker dies to the 1-damage ping");
}

#[test]
fn ash_zealot_punishes_graveyard_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ash_zealot());
    // A flashback sorcery in the opponent's graveyard.
    fn flashy() -> CardDefinition {
        CardDefinition {
            name: "Flashy Bolt",
            cost: crate::mana::cost(&[crate::mana::r()]),
            card_types: vec![CardType::Sorcery],
            keywords: vec![Keyword::Flashback(crate::mana::cost(&[crate::mana::r()]))],
            effect: crate::effect::Effect::Noop,
            ..Default::default()
        }
    }
    let sp = g.add_card_to_hand(1, flashy());
    // Move it to the graveyard.
    let pos = g.players[1].hand.iter().position(|c| c.id == sp).unwrap();
    let card = g.players[1].hand.remove(pos);
    g.players[1].graveyard.push(card);
    g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let life = g.players[1].life;
    g.perform_action(GameAction::CastFlashback {
        card_id: sp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("flashback cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "Ash Zealot deals 3 to the graveyard caster");
}

#[test]
fn perimeter_captain_gains_on_defender_block() {
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::perimeter_captain());
    let atk = g.add_card_to_battlefield(1, vanilla("Raider", 2, 2));
    g.clear_sickness(atk);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[0].life;
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(0),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![(cap, atk)])).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained 2 when the defender blocked");
}

#[test]
fn firefist_striker_battalion_locks_a_blocker() {
    let mut g = two_player_game();
    let striker = g.add_card_to_battlefield(0, catalog::firefist_striker());
    let a = g.add_card_to_battlefield(0, vanilla("A", 1, 1));
    let b = g.add_card_to_battlefield(0, vanilla("B", 1, 1));
    for id in [striker, a, b] { g.clear_sickness(id); }
    let foe = g.add_card_to_battlefield(1, vanilla("Wall", 2, 2));
    // Auto-target picks the opponent's creature for the "can't block" debuff.
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: striker, target: AttackTarget::Player(1) },
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ])).unwrap();
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::CantBlock),
        "battalion granted the target creature can't-block",
    );
}

#[test]
fn scab_clan_berserker_punishes_only_once_renowned() {
    let mut g = two_player_game();
    let scab = g.add_card_to_battlefield(0, catalog::scab_clan_berserker());
    // Opponent's noncreature spell before renown: no damage.
    fn bolt() -> CardDefinition {
        CardDefinition {
            name: "Zap",
            cost: crate::mana::cost(&[crate::mana::r()]),
            card_types: vec![CardType::Instant],
            effect: crate::effect::Effect::Noop,
            ..Default::default()
        }
    }
    let cast_zap = |g: &mut GameState| {
        let z = g.add_card_to_hand(1, bolt());
        g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: z, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(g);
    };
    let life0 = g.players[1].life;
    cast_zap(&mut g);
    assert_eq!(g.players[1].life, life0, "no punish before renown");
    // Make it renowned, then cast again → 2 damage.
    g.battlefield_find_mut(scab).unwrap().renowned = true;
    let life1 = g.players[1].life;
    cast_zap(&mut g);
    assert_eq!(g.players[1].life, life1 - 2, "renowned Scab-Clan deals 2 to the caster");
}

#[test]
fn fireblade_charger_dies_deals_its_power() {
    let mut g = two_player_game();
    let fc = g.add_card_to_battlefield(0, catalog::fireblade_charger());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    let life = g.players[1].life;
    // Lethal damage + SBA kills it, firing the death trigger.
    g.battlefield_find_mut(fc).unwrap().damage = 1;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "dealt damage equal to its power (1)");
}

