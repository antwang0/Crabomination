//! Functionality tests for `catalog::sets::decks::recent188` (OTJ gaps).

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::two_player_game;
use crabomination::game::*;
use crabomination::mana::Color;

/// Map the Frontier fetches up to two basic/Desert lands onto the battlefield
/// tapped.
#[test]
fn map_the_frontier_fetches_two_lands_tapped() {
    let mut g = two_player_game();
    let f1 = g.add_card_to_library(0, catalog::forest());
    let f2 = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f1)),
        DecisionAnswer::Search(Some(f2)),
    ]));
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::map_the_frontier().effect, &ctx).unwrap();
    for id in [f1, f2] {
        let c = g.battlefield_find(id).expect("land fetched to battlefield");
        assert!(c.tapped, "enters tapped");
    }
}

/// Neutralize the Guards shrinks the opponent's creatures by -1/-1.
#[test]
fn neutralize_the_guards_shrinks_opponent() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.add_card_to_library(0, catalog::grizzly_bears()); // to surveil
    g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::neutralize_the_guards());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Neutralize the Guards");
    drain_stack(&mut g);
    let cp = g.computed_permanent(victim).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "opponent creature is -1/-1");
}

/// Rise of the Varmints makes one 2/1 Varmint per creature card in your graveyard.
#[test]
fn rise_of_the_varmints_scales_with_graveyard() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::rise_of_the_varmints().effect, &ctx).unwrap();
    let varmints = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Varmint" && c.controller == 0)
        .count();
    assert_eq!(varmints, 3, "one Varmint per graveyard creature");
}

/// Overzealous Muscle gains indestructible when you commit a crime on your turn.
#[test]
fn overzealous_muscle_indestructible_on_crime() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let muscle = g.add_card_to_battlefield(0, catalog::overzealous_muscle());
    assert!(!g.computed_permanent(muscle).unwrap().keywords.contains(&crabomination::card::Keyword::Indestructible));
    g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(muscle).unwrap().keywords.contains(&crabomination::card::Keyword::Indestructible),
        "crime on your turn grants indestructible",
    );
}

/// Outlaws' Fury pumps your team and impulses a card when you control an outlaw.
#[test]
fn outlaws_fury_pumps_and_impulses() {
    let mut g = two_player_game();
    // An outlaw (Rogue) plus a vanilla creature.
    let mut rogue = catalog::grizzly_bears();
    rogue.subtypes.creature_types = vec![crabomination::card::CreatureType::Rogue];
    let outlaw = g.add_card_to_battlefield(0, rogue);
    g.add_card_to_library(0, catalog::mountain());
    let spell = g.add_card_to_hand(0, catalog::outlaws_fury());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Outlaws' Fury");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(outlaw).unwrap().power, 4, "+2/+0 team pump");
    assert_eq!(g.exile.len(), 1, "outlaw controlled → impulsed one card");
}
