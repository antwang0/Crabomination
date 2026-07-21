//! Functionality tests for Return to Ravnica (RTR) gap cards.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::*;

/// Static stat/keyword lines for the simple RTR creatures.
#[test]
fn rtr_stat_and_keyword_lines() {
    let peg = catalog::concordia_pegasus();
    assert_eq!((peg.power, peg.toughness), (1, 3));
    assert!(peg.keywords.contains(&Keyword::Flying));

    let imp = catalog::daggerdrome_imp();
    assert!(imp.keywords.contains(&Keyword::Flying) && imp.keywords.contains(&Keyword::Lifelink));

    let slug = catalog::catacomb_slug();
    assert_eq!((slug.power, slug.toughness), (2, 6));

    let brush = catalog::brushstrider();
    assert!(brush.keywords.contains(&Keyword::Vigilance));
}

/// Bellows Lizard firebreathes for +1/+0.
#[test]
fn bellows_lizard_firebreathes() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let lizard = g.add_card_to_battlefield(0, catalog::bellows_lizard());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lizard, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("firebreathe");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(lizard).unwrap().power, 2, "pumped to 2/1");
}

/// Centaur Healer gains 3 life on entry.
#[test]
fn centaur_healer_gains_life() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    let healer = g.add_card_to_hand(0, catalog::centaur_healer());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: healer, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "gained 3 life");
}

/// Crosstown Courier mills the player it damages by that much.
#[test]
fn crosstown_courier_mills_on_hit() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let courier = g.add_card_to_battlefield(0, catalog::crosstown_courier()); // 2/1
    g.clear_sickness(courier);
    for _ in 0..5 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let lib1 = g.players[1].library.len();
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: courier, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.resolve_combat().expect("combat");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib1 - 2, "milled 2 (combat damage)");
}

/// Centaur's Herald sacrifices itself to make a 3/3.
#[test]
fn centaurs_herald_makes_centaur() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let herald = g.add_card_to_battlefield(0, catalog::centaurs_herald());
    g.clear_sickness(herald);
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: herald, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for token");
    drain_stack(&mut g);
    assert!(g.battlefield_find(herald).is_none(), "Herald sacrificed");
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Centaur" && c.definition.power == 3),
        "made a 3/3 Centaur token",
    );
}

/// Doorkeeper mills by the number of defenders you control.
#[test]
fn doorkeeper_mills_by_defenders() {
    use crabomination::game::types::Target;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let door = g.add_card_to_battlefield(0, catalog::doorkeeper());
    g.clear_sickness(door);
    g.add_card_to_battlefield(0, catalog::doorkeeper()); // a second defender
    for _ in 0..5 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let lib1 = g.players[1].library.len();
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: door, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("mill");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib1 - 2, "milled 2 (two defenders)");
}

/// Dead Reveler may enter unleashed with a +1/+1 counter.
#[test]
fn dead_reveler_unleash_counter() {
    use crabomination::card::CounterType;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::mana::Color;
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let rev = g.add_card_to_hand(0, catalog::dead_reveler());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: rev, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cid = g.battlefield.iter().find(|c| c.definition.name == "Dead Reveler").unwrap().id;
    assert_eq!(g.battlefield_find(cid).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "entered unleashed with a +1/+1 counter");
}

/// Stat/keyword lines for the vanilla / french-vanilla RTR batch-2 creatures.
#[test]
fn rtr_batch2_stat_lines() {
    assert!(catalog::rubbleback_rhino().keywords.contains(&Keyword::Hexproof));
    assert!(catalog::skyline_predator().keywords.contains(&Keyword::Flash));
    assert!(catalog::towering_indrik().keywords.contains(&Keyword::Reach));
    assert!(catalog::tenement_crasher().keywords.contains(&Keyword::Haste));
    let thug = catalog::splatter_thug();
    assert!(thug.keywords.contains(&Keyword::FirstStrike) && thug.keywords.contains(&Keyword::Unleash));
}

/// Runewing draws a card when it dies.
#[test]
fn runewing_draws_on_death() {
    let mut g = two_player_game();
    let wing = g.add_card_to_battlefield(0, catalog::runewing()); // 2/2
    g.add_card_to_library(0, catalog::grizzly_bears());
    let h = g.players[0].hand.len();
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(wing), 2, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert!(g.battlefield_find(wing).is_none(), "Runewing died");
    assert_eq!(g.players[0].hand.len(), h + 1, "drew on death");
}

/// Seller of Songbirds makes a 1/1 flying Bird on entry.
#[test]
fn seller_of_songbirds_makes_bird() {
    let mut g = two_player_game();
    let seller = g.add_card_to_hand(0, catalog::seller_of_songbirds());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: seller, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Bird"
        && c.definition.keywords.contains(&Keyword::Flying)), "1/1 flying Bird token");
}

/// Korozda Monitor scavenges from the graveyard to grow a creature.
#[test]
fn korozda_monitor_scavenges() {
    use crabomination::card::CounterType;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let mon = g.add_card_to_graveyard(0, catalog::korozda_monitor()); // power 3
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mon, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: vec![], x_value: None,
    }).expect("scavenge");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 3,
        "three +1/+1 counters (Monitor's power)");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == mon), "Monitor exiled by scavenge");
}

/// Tavern Swindler's coin flip gains 6 life on a win.
#[test]
fn tavern_swindler_wins_flip() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let sw = g.add_card_to_battlefield(0, catalog::tavern_swindler());
    g.clear_sickness(sw);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sw, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("flip");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 3 + 6, "paid 3 life, won 6");
}
