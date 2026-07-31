//! Tests for the recent298 Ravnica batch 8 (Golgari death payoffs + guild spell).

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, CardId, GameAction, GameState};
use crabomination::mana::Color;

fn kill(g: &mut GameState, id: CardId) {
    let ctrl = g.battlefield_find(id).unwrap().controller;
    let ctx = EffectContext::for_ability(id, ctrl, Some(Target::Permanent(id)));
    let evs = g.resolve_effect(&Effect::SacrificePermanent { what: Selector::Target(0) }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 6);
    }
    g.players[0].mana_pool.add_colorless(8);
}

#[test]
fn golgari_germination_makes_a_saproling_on_nontoken_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::golgari_germination());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let before = g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count();
    kill(&mut g, bear);
    let after = g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count();
    assert_eq!(after, before + 1, "nontoken creature death minted a Saproling");
}

#[test]
fn corpse_blockade_gains_deathtouch_by_sacrifice() {
    let mut g = two_player_game();
    let blockade = g.add_card_to_battlefield(0, catalog::corpse_blockade());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // fodder
    g.clear_sickness(blockade);
    g.perform_action(GameAction::ActivateAbility {
        card_id: blockade, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("sac for deathtouch");
    drain_stack(&mut g);
    assert!(g.computed_permanent(blockade).unwrap().keywords.contains(&Keyword::Deathtouch));
}

#[test]
fn vulturous_zombie_grows_on_each_other_death() {
    let mut g = two_player_game();
    let vz = g.add_card_to_battlefield(0, catalog::vulturous_zombie());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    kill(&mut g, bear);
    assert_eq!(g.battlefield_find(vz).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "a +1/+1 counter for the other creature's death");
}

#[test]
fn grave_shell_scarab_sacrifices_for_a_card() {
    let mut g = two_player_game();
    let scarab = g.add_card_to_battlefield(0, catalog::grave_shell_scarab());
    g.clear_sickness(scarab);
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: scarab, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("sac, draw");
    drain_stack(&mut g);
    assert!(g.battlefield_find(scarab).is_none(), "sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

#[test]
fn vindictive_mob_sacrifices_a_creature_on_enter() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mob = g.add_card_to_hand(0, catalog::vindictive_mob());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: mob, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "ETB sacrifice hit the Bear");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Vindictive Mob"), "Mob stuck around");
}

#[test]
fn seed_spark_makes_saprolings_only_when_green_spent() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::azorius_signet());
    let spell = g.add_card_to_hand(0, catalog::seed_spark());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(art)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast with green");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count(), 2,
        "green spent → two Saprolings");
}
