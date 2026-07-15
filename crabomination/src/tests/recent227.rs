//! Functionality tests for `catalog::sets::decks::recent227`.

use crate::catalog;
use crate::game::effects::EffectContext;
use crate::game::{drain_stack, two_player_game, GameAction};
use crate::mana::Color;

/// Persuasive Interrogators poisons an opponent when you sacrifice a Clue.
#[test]
fn persuasive_interrogators_poisons_on_clue_sac() {
    let mut g = two_player_game();
    let pi = g.add_card_to_battlefield(0, catalog::persuasive_interrogators());
    // The Clue-sac trigger (index 1) adds two poison to the opponent.
    let effect = catalog::persuasive_interrogators().triggered_abilities[1].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(pi, 0, None, 1)).unwrap();
    assert_eq!(g.players[1].poison_counters, 2, "opponent got two poison");
}

/// Perimeter Enforcer grows when another Detective enters.
#[test]
fn perimeter_enforcer_grows_on_detective_enter() {
    let mut g = two_player_game();
    let pe = g.add_card_to_battlefield(0, catalog::perimeter_enforcer());
    let effect = catalog::perimeter_enforcer().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(pe, 0, None, 0)).unwrap();
    assert_eq!(g.computed_permanent(pe).unwrap().power, 2, "1 + 1 = 2");
}

/// Visage Bandit enters as a copy of a creature you control.
#[test]
fn visage_bandit_enters_as_copy() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::savage_ventmaw()); // a 4/4 flier to copy
    let id = g.add_card_to_hand(0, catalog::visage_bandit());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Visage Bandit for {3}{U}");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).expect("copy survived");
    assert_eq!(cp.power, 4, "copied the 4/4");
    assert!(cp.keywords.contains(&crate::card::Keyword::Flying), "copied Flying");
}
