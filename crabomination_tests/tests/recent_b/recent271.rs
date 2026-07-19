//! Functionality tests for `catalog::sets::decks::recent271`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Body Dropper's sac ability grants menace and its own sacrifice trigger
/// grows it with a +1/+1 counter.
#[test]
fn body_dropper_sac_ability_grows_and_menaces() {
    let mut g = two_player_game();
    let dropper = g.add_card_to_battlefield(0, catalog::body_dropper());
    g.clear_sickness(dropper);
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // fodder to sacrifice
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dropper,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate sac-menace");
    drain_stack(&mut g);
    let cp = g.computed_permanent(dropper).unwrap();
    assert!(cp.keywords.contains(&Keyword::Menace), "gained menace");
    assert_eq!(
        g.battlefield_find(dropper).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "the sacrifice grew it"
    );
}

/// Boon of Safety shields a creature.
#[test]
fn boon_of_safety_shields() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![],
    }]));
    let effect = catalog::boon_of_safety().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Shield), 1);
}

/// Brokers Initiate becomes a 5/5 with its hybrid ability.
#[test]
fn brokers_initiate_becomes_5_5() {
    let mut g = two_player_game();
    let init = g.add_card_to_battlefield(0, catalog::brokers_initiate());
    g.clear_sickness(init);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: init,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(init).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Brokers Veteran shields a creature you control when it dies.
#[test]
fn brokers_veteran_death_shield() {
    let mut g = two_player_game();
    let vet = g.add_card_to_battlefield(0, catalog::brokers_veteran());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(vet).unwrap().damage = 100;
    let evs = g.check_state_based_actions();
    // Target the ally with the death trigger.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(
        Target::Permanent(ally),
    )]));
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::Shield), 1);
}

/// Battle-Rage Blessing grants deathtouch and indestructible.
#[test]
fn battle_rage_blessing_grants_both() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::battle_rage_blessing().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Deathtouch));
    assert!(cp.keywords.contains(&Keyword::Indestructible));
}

/// Benalish Sleeper's kicked ETB forces an edict on each player.
#[test]
fn benalish_sleeper_kicked_edict() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let effect = catalog::benalish_sleeper().triggered_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_ability(mine, 0, None);
    ctx.kicked = true;
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "you sacrificed a creature");
    assert!(g.battlefield_find(theirs).is_none(), "opponent sacrificed a creature");
}

/// Argivian Avenger shrinks and grants a chosen keyword.
#[test]
fn argivian_avenger_modal_grant() {
    let mut g = two_player_game();
    let av = g.add_card_to_battlefield(0, catalog::argivian_avenger());
    g.clear_sickness(av);
    g.players[0].mana_pool.add_colorless(1);
    // Choose mode 0 (flying).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: av,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(av).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "-1/-1");
    assert!(cp.keywords.contains(&Keyword::Flying), "gained the chosen keyword");
}
