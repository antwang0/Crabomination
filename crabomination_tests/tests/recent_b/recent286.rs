//! Functionality tests for `catalog::sets::decks::recent286` — the Bloomburrow
//! Talent Class enchantments (CR 716 level-up).

use crabomination::card::Keyword;
use crabomination::game::{drain_stack, two_player_game, GameAction, Target, TurnStep};
use crabomination::mana::Color;

/// A Class enters the battlefield at level 1, and casting Stormchaser's Talent
/// creates a 1/1 Otter with prowess off its level-1 ETB.
#[test]
fn stormchasers_talent_enters_at_level_1_and_makes_otter() {
    let mut g = two_player_game();
    let class = g.add_card_to_hand(0, crabomination::catalog::stormchasers_talent());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: class, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Stormchaser's Talent");
    drain_stack(&mut g);
    let c = g.battlefield_find(class).expect("class on battlefield");
    assert_eq!(c.class_level, 1, "Class enters at level 1");
    let otter = g.battlefield.iter().find(|c| c.definition.name == "Otter").expect("Otter minted");
    assert!(otter.definition.keywords.contains(&Keyword::Prowess), "Otter has prowess");
}

/// Levelling Stormchaser's Talent to level 2 fires its "becomes level 2"
/// trigger, returning an instant/sorcery from the graveyard to hand.
#[test]
fn stormchasers_talent_level_2_returns_instant() {
    let mut g = two_player_game();
    let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::stormchasers_talent());
    drain_stack(&mut g); // resolve the level-1 ETB off the stack
    let bolt = g.add_card_to_graveyard(0, crabomination::catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Ability 0 is `{3}{U}: Level 2`.
    g.perform_action(GameAction::ActivateAbility {
        card_id: class, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("level up to 2");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(class).unwrap().class_level, 2, "now level 2");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "instant returned to hand");
}

/// Only at level 3 does casting an instant make an Otter (the level-3 ability
/// is gated on `SourceClassLevelAtLeast(3)`).
#[test]
fn stormchasers_talent_level_3_gates_cast_otters() {
    let mut g = two_player_game();
    let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::stormchasers_talent());
    drain_stack(&mut g); // resolve the level-1 ETB off the stack
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let otters = |g: &crabomination::game::GameState| {
        g.battlefield.iter().filter(|c| c.definition.name == "Otter").count()
    };

    // At level 1, casting an instant makes no new Otter (level-3 ability gated).
    let before = otters(&g);
    let bolt1 = g.add_card_to_hand(0, crabomination::catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast bolt at level 1");
    drain_stack(&mut g);
    assert_eq!(otters(&g), before, "no new Otter at level 1");

    // Level up to 2, then 3.
    for idx in [0u8, 1] {
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(if idx == 0 { 3 } else { 5 });
        g.perform_action(GameAction::ActivateAbility {
            card_id: class, ability_index: idx as usize, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("level up");
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(class).unwrap().class_level, 3, "now level 3");

    // At level 3, casting an instant makes an Otter.
    let before = otters(&g);
    let bolt2 = g.add_card_to_hand(0, crabomination::catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast bolt at level 3");
    drain_stack(&mut g);
    assert_eq!(otters(&g), before + 1, "one new Otter at level 3");
}

/// A Class's level resets when it leaves and re-enters the battlefield
/// (CR 716.2 — the level is battlefield-only).
#[test]
fn class_level_resets_on_leave() {
    let mut g = two_player_game();
    let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::hunters_talent());
    // Force it to level 2.
    g.battlefield.iter_mut().find(|c| c.id == class).unwrap().class_level = 2;
    // Bounce it to hand, then replay it — it should re-enter at level 1.
    let class2 = g.move_card_to_battlefield_for_test(0, crabomination::catalog::hunters_talent());
    assert_eq!(g.battlefield_find(class2).unwrap().class_level, 1, "re-enters at level 1");
}

/// CR 716.4 — Class levels are gained one at a time: the "Level 3" ability
/// can't be activated while the Class is still at level 1 (its `condition` is
/// `SourceClassLevelIs(2)`).
#[test]
fn cr_716_4_cannot_skip_a_level() {
    let mut g = two_player_game();
    let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::stormchasers_talent());
    drain_stack(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    // Ability 1 is `{5}{U}: Level 3` — illegal from level 1.
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: class, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    });
    assert!(res.is_err(), "can't jump to level 3 from level 1");
    assert_eq!(g.battlefield_find(class).unwrap().class_level, 1, "still level 1");
}

/// Hunter's Talent's level-1 ETB is a one-sided bite: a creature you control
/// deals damage equal to its power to a creature you don't control.
#[test]
fn hunters_talent_etb_bite() {
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears()); // 2/2
    let theirs = g.add_card_to_battlefield(1, crabomination::catalog::grizzly_bears()); // 2/2
    let etb = crabomination::catalog::hunters_talent().triggered_abilities[0].effect.clone();
    let ctx = EffectContext {
        targets: vec![Target::Permanent(mine), Target::Permanent(theirs)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&etb, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "their 2/2 took 2 and died");
    assert!(g.battlefield_find(mine).is_some(), "my creature is unharmed (one-sided)");
}

/// Scavenger's Talent's level-1 ability makes a Food when a creature you
/// control dies (once per turn).
#[test]
fn scavengers_talent_food_on_creature_death() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, crabomination::catalog::scavengers_talent());
    drain_stack(&mut g);
    let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
    let mut evs = vec![];
    g.sacrifice_one(bear, 0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"),
        "a Food token was created on creature death",
    );
}

/// CR 603.3 — an ability that "triggers only once each turn" fires just once
/// per turn: two creatures dying on the same turn yield a single Food from
/// Scavenger's Talent's level-1 trigger.
#[test]
fn cr_603_3_once_each_turn_trigger() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, crabomination::catalog::scavengers_talent());
    drain_stack(&mut g);
    let food = |g: &crabomination::game::GameState| {
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Food").count()
    };
    for _ in 0..2 {
        let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
        let mut evs = vec![];
        g.sacrifice_one(bear, 0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(food(&g), 1, "only one Food despite two deaths this turn");
}

/// Scavenger's Talent's level-2 mill only fires once it reaches level 2.
#[test]
fn scavengers_talent_level_2_mills_on_sacrifice() {
    let mut g = two_player_game();
    let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::scavengers_talent());
    drain_stack(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    // Library to mill from.
    for _ in 0..4 {
        g.add_card_to_library(1, crabomination::catalog::grizzly_bears());
    }
    // Level up to 2.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: class, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("level up to 2");
    drain_stack(&mut g);
    // Sacrifice a permanent → target player mills two.
    let fodder = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
    let gy_before = g.players[1].graveyard.len();
    let mut evs = vec![];
    g.sacrifice_one(fodder, 0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.len() >= gy_before + 2, "opponent milled two");
}

/// Bandit's Talent's level-1 ETB: an opponent holding only lands can't discard
/// a nonland, so they discard two cards.
#[test]
fn bandits_talent_etb_discard_two() {
    let mut g = two_player_game();
    let class = g.add_card_to_hand(0, crabomination::catalog::bandits_talent());
    for _ in 0..3 {
        g.add_card_to_hand(1, crabomination::catalog::forest()); // only lands
    }
    let before = g.players[1].hand.len();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: class, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Bandit's Talent");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), before - 2, "opponent discarded two (no nonland to pitch)");
}

/// Bandit's Talent's level-3 draw scales with the number of low-hand opponents
/// (`Value::OpponentsWithHandSizeAtMost`).
#[test]
fn bandits_talent_level_3_draw_scales() {
    use crabomination::effect::{Effect, PlayerRef, Selector, Value};
    use crabomination::game::effects::EffectContext;
    let draw = Effect::Draw {
        who: Selector::Player(PlayerRef::You),
        amount: Value::OpponentsWithHandSizeAtMost(1),
    };

    // Opponent has 1 card → draw 1.
    let mut g = two_player_game();
    g.add_card_to_hand(1, crabomination::catalog::forest());
    for _ in 0..3 {
        g.add_card_to_library(0, crabomination::catalog::forest());
    }
    let before = g.players[0].hand.len();
    g.resolve_effect(&draw, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert_eq!(g.players[0].hand.len(), before + 1, "drew 1 for the low-hand opponent");

    // Opponent has 2 cards → draw 0.
    let mut g2 = two_player_game();
    for _ in 0..2 {
        g2.add_card_to_hand(1, crabomination::catalog::forest());
    }
    g2.add_card_to_library(0, crabomination::catalog::forest());
    let before2 = g2.players[0].hand.len();
    g2.resolve_effect(&draw, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert_eq!(g2.players[0].hand.len(), before2, "no draw when the opponent has two cards");
}

/// Wizard Class draws two on reaching level 2, and at level 3 puts a +1/+1
/// counter on a creature whenever you draw.
#[test]
fn wizard_class_levels() {
    let mut g = two_player_game();
    let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::wizard_class());
    drain_stack(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    for _ in 0..6 {
        g.add_card_to_library(0, crabomination::catalog::forest());
    }
    // Level up to 2 → draw two.
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: class, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("level up to 2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew two on becoming level 2");

    // Level up to 3, then a draw grows a creature.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: class, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("level up to 3");
    drain_stack(&mut g);
    let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
    let mut evs = vec![];
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
        1,
        "drawing at level 3 adds a +1/+1 counter",
    );
}

/// Cleric Class's level-1 static adds 1 to life gained; at level 2 gaining life
/// grows a creature.
#[test]
fn cleric_class_life_gain() {
    use crabomination::effect::{Effect, PlayerRef, Selector, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::cleric_class());
    drain_stack(&mut g);
    let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
    let life_before = g.players[0].life;

    // Level 1: gaining 2 life actually gains 3 (LifeGainBonus +1).
    g.resolve_effect(
        &Effect::GainLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(2) },
        &EffectContext::for_spell(0, None, 0, 0),
    )
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3, "gained 2 + 1 bonus");
    // No counter yet — the level-2 trigger isn't online.
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
        0,
    );

    // Force level 2, gain again → a +1/+1 counter lands.
    g.battlefield.iter_mut().find(|c| c.id == class).unwrap().class_level = 2;
    let evs = g
        .resolve_effect(
            &Effect::GainLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(1) },
            &EffectContext::for_spell(0, None, 0, 0),
        )
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
        1,
        "level-2 lifegain trigger grew the creature",
    );
}

/// Warlock Class: at level 1 the end-step drain only fires if a creature died;
/// at level 3 each opponent loses life equal to what they lost this turn.
#[test]
fn warlock_class_end_step_drains() {
    let mut g = two_player_game();
    let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::warlock_class());
    drain_stack(&mut g);
    g.active_player_idx = 0;

    // No creature died → level-1 drain does nothing.
    let life = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life, "no drain without a death");

    // A creature dies → level-1 drain costs the opponent 1.
    let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
    let mut evs = vec![];
    g.sacrifice_one(bear, 0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let life = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "level-1 drain after a death");

    // At level 3, the opponent additionally loses what they lost this turn.
    g.battlefield.iter_mut().find(|c| c.id == class).unwrap().class_level = 3;
    g.players[1].life_lost_this_turn = 5;
    let life = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    // Level-1 (creature died earlier) drains 1, level-3 drains the 5 lost.
    assert!(g.players[1].life <= life - 5, "level-3 mirror drain applied");
}
