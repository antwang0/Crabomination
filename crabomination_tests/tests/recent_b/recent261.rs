//! Functionality tests for `catalog::sets::decks::recent261`
//! (Buried in the Garden + the any-color land-tap rider).

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// The ETB exiles a nonland permanent an opponent controls until the Aura leaves.
#[test]
fn buried_in_the_garden_exiles_on_etb() {
    let mut g = two_player_game();
    let aura = g.add_card_to_battlefield(0, catalog::buried_in_the_garden());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let effect = catalog::buried_in_the_garden().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_trigger(aura, 0, Some(Target::Permanent(victim)), 0);
    let evs = g.resolve_effect(&effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    assert!(g.battlefield_find(victim).is_none(), "opponent's creature exiled");
}

/// The enchanted land taps for its own mana plus one of any color.
#[test]
fn buried_in_the_garden_adds_any_color_mana() {
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let aura = g.add_card_to_battlefield(0, catalog::buried_in_the_garden());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(forest);
    // The extra-mana rider asks for a color; choose blue.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("tap the enchanted Forest");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "Forest's own {{G}}");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "plus one mana of the chosen color");
}
