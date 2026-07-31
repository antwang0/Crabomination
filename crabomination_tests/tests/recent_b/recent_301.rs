//! Tests for the recent301 Ravnica batch 11 (death-tuck, Hellbent grant,
//! hand-empty counter, group basic-type animator, delayed-token burn).

use crabomination::card::LandType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, CardId, GameAction, GameState};
use crabomination::mana::Color;

fn kill(g: &mut GameState, id: CardId) {
    let ctrl = g.battlefield_find(id).unwrap().controller;
    let ctx = EffectContext::for_ability(id, ctrl, Some(Target::Permanent(id)));
    let evs = g
        .resolve_effect(&Effect::SacrificePermanent { what: Selector::Target(0) }, &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

#[test]
fn sadistic_augermage_makes_each_player_tuck_a_card() {
    let mut g = two_player_game();
    let aug = g.add_card_to_battlefield(0, catalog::sadistic_augermage());
    let mine = g.add_card_to_hand(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_hand(1, catalog::grizzly_bears());
    kill(&mut g, aug);
    // Bots auto-put their first hand card on top of library.
    assert!(g.players[0].library.first().map(|c| c.id) == Some(mine), "my card tucked on top");
    assert!(g.players[1].library.first().map(|c| c.id) == Some(theirs), "their card tucked on top");
    assert!(g.players[0].hand.is_empty() && g.players[1].hand.is_empty(), "both hands emptied");
}

#[test]
fn gobhobbler_rats_pumps_and_regenerates_while_hellbent() {
    let mut g = two_player_game();
    let rats = g.add_card_to_battlefield(0, catalog::gobhobbler_rats());
    // Hand empty → Hellbent live: +1/+0 and the {B}: Regenerate grant.
    assert!(g.players[0].hand.is_empty());
    assert_eq!(g.computed_permanent(rats).unwrap().power, 3, "Hellbent +1/+0");
    assert!(!g.granted_abilities_for(rats).is_empty(), "granted the regenerate ability");
    // Fill the hand → Hellbent off: back to base 2/2 with no granted ability.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(rats).unwrap().power, 2, "no Hellbent pump with a card in hand");
    assert!(g.granted_abilities_for(rats).is_empty(), "regenerate grant is gone");
}

#[test]
fn perplex_makes_the_controller_discard_their_hand() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // P0 puts a Bolt on the stack and still holds two other cards.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    // P0 Perplexes its own Bolt — the ward-style auto-pay discards the hand.
    let perplex = g.add_card_to_hand(0, catalog::perplex());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: perplex, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("perplex castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.is_empty(), "controller discarded their whole hand to pay");
    assert!(!g.battlefield.iter().any(|c| c.id == foe), "Bolt resolved (payment averted the counter)");
}

#[test]
fn terraformer_turns_your_lands_into_a_chosen_basic() {
    let mut g = two_player_game();
    let tf = g.add_card_to_battlefield(0, catalog::terraformer());
    g.clear_sickness(tf);
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    let f2 = g.add_card_to_battlefield(0, catalog::mountain());
    // Choose Island (Blue).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: tf, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate Terraformer");
    drain_stack(&mut g);
    for land in [f1, f2] {
        let cp = g.computed_permanent(land).unwrap();
        assert!(cp.subtypes.land_types == vec![LandType::Island], "land became an Island");
    }
}

#[test]
fn skeletonize_burns_and_leaves_a_skeleton_on_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 dies to 3
    let spell = g.add_card_to_hand(0, catalog::skeletonize());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("skeletonize castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear died to 3 damage");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Skeleton"),
        "a Skeleton token was made for its controller",
    );
}

#[test]
fn skeletonize_makes_no_token_if_the_creature_survives() {
    let mut g = two_player_game();
    // 3/4 survives 3 damage → no delayed death, no token.
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_omens());
    let spell = g.add_card_to_hand(0, catalog::skeletonize());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(wall)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("skeletonize castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Skeleton"), "no token while it lives");
}
