//! Tests for the recent300 Ravnica batch 10 (tap-down + MV-9 transmute).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, GameAction};

#[test]
fn thundersong_trumpeter_locks_a_creature_down() {
    let mut g = two_player_game();
    let trumpeter = g.add_card_to_battlefield(0, catalog::thundersong_trumpeter());
    g.clear_sickness(trumpeter);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: trumpeter, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], x_value: None,
    }).expect("tap it down");
    drain_stack(&mut g);
    let kw = g.computed_permanent(foe).unwrap().keywords;
    assert!(kw.contains(&Keyword::CantAttack) && kw.contains(&Keyword::CantBlock),
        "target can't attack or block this turn");
}

#[test]
fn grozoth_fetches_every_mv_nine_card() {
    let mut g = two_player_game();
    // Two MV-9 cards (Grozoth itself) and a cheap card in the library.
    let a = g.add_card_to_library(0, catalog::grozoth());
    let b = g.add_card_to_library(0, catalog::grozoth());
    g.add_card_to_library(0, catalog::grizzly_bears());
    // AutoDecider declines searches; script three picks (the two MV-9, then stop).
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(a)),
        DecisionAnswer::Search(Some(b)),
        DecisionAnswer::Search(None),
    ]));
    let grz = g.add_card_to_battlefield(0, catalog::grozoth());
    g.fire_self_etb_triggers(grz, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == a) && g.players[0].hand.iter().any(|c| c.id == b),
        "both MV-9 cards fetched into hand");
}

#[test]
fn grozoth_can_shed_defender_to_attack() {
    let mut g = two_player_game();
    let grz = g.add_card_to_battlefield(0, catalog::grozoth());
    g.clear_sickness(grz);
    assert!(g.computed_permanent(grz).unwrap().keywords.contains(&Keyword::Defender));
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: grz, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("shed defender");
    drain_stack(&mut g);
    assert!(!g.computed_permanent(grz).unwrap().keywords.contains(&Keyword::Defender),
        "defender dropped until end of turn");
}
