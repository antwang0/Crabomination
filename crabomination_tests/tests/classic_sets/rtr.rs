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

/// Explosive Impact burns a player for 5.
#[test]
fn explosive_impact_deals_5() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::explosive_impact());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 5, "5 damage");
}

/// Auger Spree gives +4/-4.
#[test]
fn auger_spree_pumps_and_shrinks() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let spree = g.add_card_to_hand(0, catalog::auger_spree());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spree, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // 2/2 → 6/-2 → dies to SBA.
    let _ = g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "toughness dropped to -2, creature died");
}

/// Avenging Arrow only hits a creature that dealt damage this turn.
#[test]
fn avenging_arrow_needs_damage_dealt() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let dealt = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(dealt).unwrap().dealt_damage_this_turn = true;
    let arrow = g.add_card_to_hand(0, catalog::avenging_arrow());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: arrow, target: Some(Target::Permanent(dealt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dealt).is_none(), "the damaging creature is destroyed");
}

/// Skull Rend burns each opponent and discards two at random.
#[test]
fn skull_rend_damages_and_discards() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    let rend = g.add_card_to_hand(0, catalog::skull_rend());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    let life = g.players[1].life;
    let h = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: rend, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "2 damage to the opponent");
    assert_eq!(g.players[1].hand.len(), h - 2, "discarded two");
}

/// Dynacharge's overload pumps every creature you control.
#[test]
fn dynacharge_overload_pumps_team() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let over = catalog::dynacharge().alternative_cost.unwrap().effect_override.unwrap();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&over, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.computed_permanent(a).unwrap().power, 4, "first creature +2/+0");
    assert_eq!(g.computed_permanent(b).unwrap().power, 4, "second creature +2/+0");
}

/// Batch-4 vanilla/hybrid stat lines.
#[test]
fn rtr_batch4_stat_lines() {
    let rs = catalog::risen_sanctuary();
    assert_eq!((rs.power, rs.toughness), (8, 8));
    assert!(rs.keywords.contains(&Keyword::Vigilance));
    assert!(catalog::rakdos_shred_freak().keywords.contains(&Keyword::Haste));
    let gl = catalog::golgari_longlegs();
    assert_eq!((gl.power, gl.toughness), (5, 4));
}

/// Frostburn Weird pumps +1/-1 with its hybrid ability.
#[test]
fn frostburn_weird_pumps() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let weird = g.add_card_to_battlefield(0, catalog::frostburn_weird()); // 1/4
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: weird, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(weird).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3), "1/4 -> 2/3");
}

/// Phantom General buffs your creature tokens only.
#[test]
fn phantom_general_buffs_tokens() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::phantom_general());
    // A real (nontoken) creature is unaffected.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // A token creature (via Seller of Songbirds' Bird) gets +1/+1.
    let seller = g.add_card_to_hand(0, catalog::seller_of_songbirds());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: seller, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let bird = g.battlefield.iter().find(|c| c.definition.name == "Bird").unwrap().id;
    assert_eq!(g.computed_permanent(bird).unwrap().power, 2, "1/1 token -> 2/2");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "nontoken bear unaffected");
}

/// Slum Reaper makes each player sacrifice a creature.
#[test]
fn slum_reaper_edicts_everyone() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let reaper = g.add_card_to_hand(0, catalog::slum_reaper());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: reaper, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "opponent's only creature sacrificed");
}

/// Soulsworn Spirit detains an opponent's creature on entry.
#[test]
fn soulsworn_spirit_detains() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(
        [crabomination::decision::DecisionAnswer::Target(Target::Permanent(foe))],
    ));
    let spirit = g.add_card_to_hand(0, catalog::soulsworn_spirit());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spirit, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().detained_by.is_some(), "opponent's creature detained");
}

/// Chaos Imps has trample only while it carries a +1/+1 counter.
#[test]
fn chaos_imps_trample_gated_on_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let imps = g.add_card_to_battlefield(0, catalog::chaos_imps());
    assert!(!g.computed_permanent(imps).unwrap().keywords.contains(&Keyword::Trample),
        "no trample without a counter");
    g.battlefield_find_mut(imps).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(imps).unwrap().keywords.contains(&Keyword::Trample),
        "trample once it has a +1/+1 counter");
}
