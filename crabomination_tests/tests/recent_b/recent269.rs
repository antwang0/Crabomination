//! Functionality tests for `catalog::sets::decks::recent269`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::{drain_stack, two_player_game, CardId, GameState};
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// Gilded Scuttler taps and stuns an opponent's creature on ETB.
#[test]
fn gilded_scuttler_taps_and_stuns() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let scut = g.add_card_to_hand(0, catalog::gilded_scuttler());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, scut, Some(Target::Permanent(victim)));
    let v = g.battlefield_find(victim).unwrap();
    assert!(v.tapped, "tapped");
    assert_eq!(v.counter_count(CounterType::Stun), 1, "stunned");
    assert!(
        g.battlefield_find(scut).unwrap().has_keyword(&Keyword::Unblockable),
        "unblockable"
    );
}

/// Go Forth can tutor a basic land to hand.
#[test]
fn go_forth_tutors_basic() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let effect = catalog::go_forth().effect;
    // Resolving the effect directly runs mode 0 (the tutor); ctx.mode defaults to 0.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Forest"),
        "basic land tutored to hand"
    );
}

/// Hearts on Fire can pump two creatures.
#[test]
fn hearts_on_fire_pumps_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::hearts_on_fire().effect;
    let ctx = EffectContext {
        targets: vec![Target::Permanent(a), Target::Permanent(b)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.computed_permanent(a).unwrap().power, 4, "+2/+1");
    assert_eq!(g.computed_permanent(b).unwrap().power, 4, "+2/+1");
}

/// Hungry Megasloth grows itself with its mana ability.
#[test]
fn hungry_megasloth_grows() {
    let mut g = two_player_game();
    let sloth = g.add_card_to_battlefield(0, catalog::hungry_megasloth());
    g.clear_sickness(sloth);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sloth,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(sloth).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Phantasmal Shieldback sacrifices itself when targeted, then draws.
#[test]
fn phantasmal_shieldback_sacs_when_targeted() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let shield = g.add_card_to_battlefield(0, catalog::phantasmal_shieldback());
    // Target it with a pump spell → it sacrifices itself, then draws.
    let bolt = g.add_card_to_hand(0, catalog::antagonize());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    cast(&mut g, bolt, Some(Target::Permanent(shield)));
    assert!(g.battlefield_find(shield).is_none(), "sacrificed itself on being targeted");
    // Bolt left hand (-1) and the death draw added one (+1) → net unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1, "drew on death");
}

/// Battlefield Butcher's activation cost drops {1} per creature card in the
/// graveyard (new `cost_reduction_per_graveyard` primitive).
#[test]
fn battlefield_butcher_graveyard_discount() {
    let mut g = two_player_game();
    let butcher = g.add_card_to_battlefield(0, catalog::battlefield_butcher());
    g.clear_sickness(butcher);
    // Two creature cards + a noncreature in the graveyard → {5} - {2} = {3}.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: butcher,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activated for the reduced {3}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "opponent lost 2 life");
}

/// Razorgrass Invoker pumps itself and one other creature.
#[test]
fn razorgrass_invoker_pumps_pair() {
    let mut g = two_player_game();
    let inv = g.add_card_to_battlefield(0, catalog::razorgrass_invoker());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(inv);
    g.players[0].mana_pool.add_colorless(8);
    g.perform_action(GameAction::ActivateAbility {
        card_id: inv,
        ability_index: 0,
        target: Some(Target::Permanent(ally)),
        additional_targets: vec![],
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(inv).unwrap().power, 7, "self +3/+3");
    assert_eq!(g.computed_permanent(ally).unwrap().power, 5, "ally +3/+3");
}
