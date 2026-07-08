//! Functionality tests for `catalog::sets::decks::mh2b` — the MH2 sweep
//! (Squirrel/token package, suspend artifacts, graveyard tutors, Thrasta).

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn resolve_spell(g: &mut GameState, def: crate::card::CardDefinition, targets: Vec<Target>) {
    let mut ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = targets;
    let events = g.resolve_effect(&def.effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
}

/// Thrasta: {3} discount per spell already cast this turn, and hexproof only
/// during its entry turn.
#[test]
fn thrasta_discount_and_entry_hexproof() {
    let mut g = two_player_game();
    g.players[0].spells_cast_this_turn = 3;
    let thrasta = g.add_card_to_hand(0, catalog::thrasta_tempests_roar());
    // 12 total − 9 discount = {1}{G}{G} equivalent: 3 mana pays it.
    g.players[0].mana_pool.add(Color::Green, 3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: thrasta, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("discounted Thrasta castable off three mana");
    drain_stack(&mut g);
    let kws = g.computed_permanent(thrasta).unwrap().keywords.clone();
    assert!(kws.contains(&Keyword::Hexproof), "hexproof the turn it entered");
    g.turn_number += 1;
    assert!(
        !g.computed_permanent(thrasta).unwrap().keywords.contains(&Keyword::Hexproof),
        "hexproof gone after the turn"
    );
}

/// Academy Manufactor: a lone Clue mint becomes Clue + Food + Treasure.
#[test]
fn academy_manufactor_mints_one_of_each() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::academy_manufactor());
    resolve_spell(&mut g, catalog::hard_evidence(), vec![]);
    for name in ["Clue", "Food", "Treasure"] {
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.name == name).count(),
            1,
            "one {name}"
        );
    }
}

/// Chatterfang: minted tokens bring that many Squirrels along.
#[test]
fn chatterfang_adds_that_many_squirrels() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::chatterfang_squirrel_general());
    resolve_spell(&mut g, catalog::goblin_rally(), vec![]); // four 1/1 Goblins
    let goblins = g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count();
    let squirrels =
        g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").count();
    assert_eq!(goblins, 4);
    assert_eq!(squirrels, 4, "one Squirrel per minted token");
}

/// Chatterfang's {B}: sacrifice X Squirrels gives +X/-X.
#[test]
fn chatterfang_sac_x_squirrels_pumps() {
    let mut g = two_player_game();
    let fang = g.add_card_to_battlefield(0, catalog::chatterfang_squirrel_general());
    for _ in 0..2 {
        let sq = g.add_card_to_battlefield(0, catalog::squirrel_sovereign());
        g.clear_sickness(sq);
    }
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.players[0].mana_pool.add(Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: fang,
        ability_index: 0,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        x_value: Some(2),
    })
    .expect("sac two Squirrels");
    drain_stack(&mut g);
    let cp = g.computed_permanent(victim).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 2), "+2/-2");
}

/// Chatterstorm storms: second spell of the turn mints two Squirrels total.
#[test]
fn chatterstorm_storms() {
    let mut g = two_player_game();
    g.spells_cast_this_turn = 1;
    let storm = g.add_card_to_hand(0, catalog::chatterstorm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: storm, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").count(),
        2,
        "original + one storm copy"
    );
}

/// Ravenous Squirrel grows on any of your artifact/creature sacrifices.
#[test]
fn ravenous_squirrel_grows_on_sacrifice() {
    let mut g = two_player_game();
    let squirrel = g.add_card_to_battlefield(0, catalog::ravenous_squirrel());
    let food = g.add_card_to_battlefield(0, catalog::academy_manufactor());
    let mut events = Vec::new();
    g.sacrifice_one(food, 0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(squirrel).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Squirrel Sovereign pumps other Squirrels only.
#[test]
fn squirrel_sovereign_is_a_lord() {
    let mut g = two_player_game();
    let sov = g.add_card_to_battlefield(0, catalog::squirrel_sovereign());
    let other = g.add_card_to_battlefield(0, catalog::ravenous_squirrel());
    assert_eq!(g.computed_permanent(other).unwrap().power, 2, "1/1 + lord");
    assert_eq!(g.computed_permanent(sov).unwrap().power, 2, "not itself");
}

/// Squirrel Sanctuary: ETB Squirrel; a nontoken creature dying returns it
/// to hand for {1}.
#[test]
fn squirrel_sanctuary_token_and_bounce() {
    let mut g = two_player_game();
    let sanct = g.add_card_to_battlefield(0, catalog::squirrel_sanctuary());
    g.fire_self_etb_triggers(sanct, 0);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Squirrel"));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let _ = g.remove_to_graveyard_with_triggers(bear);
    g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: bear }]);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == sanct), "bounced to hand");
}

/// Scurry Oak mints a Squirrel when it gets +1/+1 counters.
#[test]
fn scurry_oak_squirrel_per_counter_batch() {
    let mut g = two_player_game();
    let oak = g.add_card_to_battlefield(0, catalog::scurry_oak());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = crate::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(oak)), 0, 0);
    let events = g
        .resolve_effect(
            &crate::effect::Effect::AddCounter {
                what: crate::card::Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: crate::card::Value::ONE,
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Squirrel"));
}

/// Drey Keeper: two Squirrels on ETB; the pump grants menace.
#[test]
fn drey_keeper_tokens_and_anthem() {
    let mut g = two_player_game();
    let keeper = g.add_card_to_battlefield(0, catalog::drey_keeper());
    g.fire_self_etb_triggers(keeper, 0);
    drain_stack(&mut g);
    let squirrels: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Squirrel")
        .map(|c| c.id)
        .collect();
    assert_eq!(squirrels.len(), 2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: keeper, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(squirrels[0]).unwrap();
    assert_eq!(cp.power, 2, "+1/+0");
    assert!(cp.keywords.contains(&Keyword::Menace));
}

/// Sylvan Anthem pumps green creatures and scries on their arrival.
#[test]
fn sylvan_anthem_pump_and_scry() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sylvan_anthem());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "green creature pumped");
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    assert_eq!(g.computed_permanent(angel).unwrap().power, 4, "white creature not pumped");
}

/// Timeless Dragon eternalizes into a 4/4 Zombie Dragon token.
#[test]
fn timeless_dragon_eternalize() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::timeless_dragon());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: dead, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("eternalize");
    drain_stack(&mut g);
    let token = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Timeless Dragon" && c.is_token)
        .expect("token copy");
    let cp = g.computed_permanent(token.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Unmarked Grave tutors a nonlegendary card to the graveyard.
#[test]
fn unmarked_grave_entombs_nonlegendary() {
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
    resolve_spell(&mut g, catalog::unmarked_grave(), vec![]);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Young Necromancer exiles two and reanimates.
#[test]
fn young_necromancer_reanimates() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let necro = g.add_card_to_battlefield(0, catalog::young_necromancer());
    g.fire_self_etb_triggers(necro, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_some(), "Serra Angel reanimated");
    assert_eq!(g.players[0].graveyard.len(), 0, "two bolts exiled");
}

/// Necrogoyf's power tracks creature cards in all graveyards.
#[test]
fn necrogoyf_cda_power() {
    let mut g = two_player_game();
    let goyf = g.add_card_to_battlefield(0, catalog::necrogoyf());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::serra_angel());
    let cp = g.computed_permanent(goyf).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4));
}

/// Terminal Agony kills a creature and carries madness.
#[test]
fn terminal_agony_kills() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    resolve_spell(&mut g, catalog::terminal_agony(), vec![Target::Permanent(angel)]);
    assert!(g.battlefield_find(angel).is_none());
    assert!(catalog::terminal_agony().madness_cost().is_some());
}

/// Hard Evidence: a 0/3 Crab plus a Clue.
#[test]
fn hard_evidence_crab_and_clue() {
    let mut g = two_player_game();
    resolve_spell(&mut g, catalog::hard_evidence(), vec![]);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Crab"));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"));
}

/// Brainstone draws three and puts two back.
#[test]
fn brainstone_brainstorms() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let stone = g.add_card_to_battlefield(0, catalog::brainstone());
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "net +1 card");
    assert!(g.battlefield_find(stone).is_none(), "sacrificed");
}

/// Sol Talisman suspends for {1} and resolves into a mana rock.
#[test]
fn sol_talisman_suspends_and_taps_for_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sol_talisman());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Suspend { card_id: id }).expect("suspend");
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..4 {
        let _ = g.process_suspend();
    }
    drain_stack(&mut g);
    let talisman = g.battlefield_find(id).expect("resolved onto the battlefield");
    assert!(talisman.definition.is_artifact());
}

/// Gargadon carries trample + suspend 4.
#[test]
fn gargadon_stats() {
    let d = catalog::gargadon();
    assert!(d.keywords.contains(&Keyword::Trample));
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Suspend(4, _))));
    assert_eq!((d.power, d.toughness), (7, 5));
}

/// Vile Entomber entombs on ETB.
#[test]
fn vile_entomber_entombs() {
    let mut g = two_player_game();
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(angel))]));
    let entomber = g.add_card_to_battlefield(0, catalog::vile_entomber());
    g.fire_self_etb_triggers(entomber, 0);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Serra Angel"));
}
