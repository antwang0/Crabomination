//! Functionality tests for `catalog::sets::decks::recent140` (WOE wave 13).

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

/// Food Coma exiles an opponent's creature and makes a Food; it returns when the
/// enchantment leaves.
#[test]
fn food_coma_exiles_until_leaves() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    let coma = g.add_card_to_hand(0, catalog::food_coma());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, coma, Some(Target::Permanent(victim)), None);
    assert!(g.battlefield_find(victim).is_none(), "opponent creature exiled");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"), "made a Food");
    // Destroy Food Coma → the creature returns.
    let coma_id = g.battlefield.iter().find(|c| c.definition.name == "Food Coma").unwrap().id;
    let ctx = crabomination::game::effects::EffectContext::for_ability(coma_id, 0, Some(Target::Permanent(coma_id)));
    g.resolve_effect(&crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) }, &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&[]);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Serra Angel"),
        "exiled creature returned when Food Coma left",
    );
}

/// Rankle's Prank mode "each player loses 4 life" hits both players.
#[test]
fn rankles_prank_loses_life_both() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::rankles_prank());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![1])]));
    cast(&mut g, spell, None, None);
    assert_eq!(g.players[0].life, l0 - 4, "you lose 4");
    assert_eq!(g.players[1].life, l1 - 4, "opponent loses 4");
}

/// Song of Totentanz makes X Rats.
#[test]
fn song_of_totentanz_makes_x_rats() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::song_of_totentanz());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3); // X = 3
    cast(&mut g, spell, None, Some(3));
    let rats = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Rat").count();
    assert_eq!(rats, 3, "X=3 → three Rats");
}
