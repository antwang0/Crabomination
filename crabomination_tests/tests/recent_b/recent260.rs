//! Functionality tests for `catalog::sets::decks::recent260` (Anzrag).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::{drain_stack, two_player_game};

/// When Anzrag becomes blocked it untaps your creatures and grants an extra
/// combat phase.
#[test]
fn anzrag_untaps_and_adds_combat_on_block() {
    let mut g = two_player_game();
    let anzrag = g.add_card_to_battlefield(0, catalog::anzrag_the_quake_mole());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(ally).unwrap().tapped = true;
    let before = g.additional_combat_phases;
    let effect = catalog::anzrag_the_quake_mole().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_trigger(anzrag, 0, None, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield_find(ally).unwrap().tapped, "your creatures untapped");
    assert_eq!(g.additional_combat_phases, before + 1, "an extra combat phase was queued");
}

/// Anzrag's activated ability makes it must-be-blocked.
#[test]
fn anzrag_forces_a_block() {
    let mut g = two_player_game();
    let anzrag = g.add_card_to_battlefield(0, catalog::anzrag_the_quake_mole());
    g.clear_sickness(anzrag);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: anzrag,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate the must-be-blocked ability");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(anzrag).unwrap().keywords.contains(&Keyword::MustBeBlocked),
        "Anzrag must be blocked this turn",
    );
}
