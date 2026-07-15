//! Functionality tests for `catalog::sets::decks::recent33` — recursion,
//! aristocrat drain, and sacrifice-outlet staples.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::two_player_game;
use crabomination::game::*;

fn ctx_for(source: CardId) -> EffectContext {
    EffectContext::for_ability(source, 0, None)
}

#[test]
fn endless_cockroaches_returns_to_hand_on_death() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::endless_cockroaches());
    g.battlefield_find_mut(id).unwrap().damage = 1; // lethal
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "left the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Endless Cockroaches"),
        "returned to its owner's hand");
}

#[test]
fn poison_tip_archer_drains_on_any_other_death() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    g.add_card_to_battlefield(0, catalog::poison_tip_archer());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().damage = 2; // lethal
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "each opponent loses 1 when another creature dies");
}

#[test]
fn poison_tip_archer_has_reach_and_deathtouch() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::poison_tip_archer());
    let kw = g.computed_permanent(id).unwrap().keywords;
    assert!(kw.contains(&Keyword::Reach) && kw.contains(&Keyword::Deathtouch));
}

#[test]
fn altar_of_dementia_mills_equal_to_power() {
    let mut g = two_player_game();
    for _ in 0..10 { g.add_card_to_library(1, catalog::island()); }
    let altar = g.add_card_to_battlefield(0, catalog::altar_of_dementia());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2 power
    let lib_before = g.players[1].library.len();
    let mut ctx = ctx_for(altar);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&catalog::altar_of_dementia().activated_abilities[0].effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib_before - 2, "milled = sacrificed creature's power");
}

#[test]
fn sadistic_hypnotist_discards_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::island()); }
    let hyp = g.add_card_to_battlefield(0, catalog::sadistic_hypnotist());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // fodder
    let hand_before = g.players[1].hand.len();
    let mut ctx = ctx_for(hyp);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&catalog::sadistic_hypnotist().activated_abilities[0].effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before - 2, "target player discards two");
}

#[test]
fn sprout_swarm_makes_a_saproling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sprout_swarm());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sprout Swarm castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count(), 1);
}

#[test]
fn sprout_swarm_has_convoke_and_buyback() {
    let kw = catalog::sprout_swarm().keywords;
    assert!(kw.contains(&Keyword::Convoke));
    assert!(kw.iter().any(|k| matches!(k, Keyword::Buyback(_))));
}
