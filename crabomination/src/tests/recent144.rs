//! Functionality tests for `catalog::sets::decks::recent144` (WOE wave 17).

use crate::catalog;
use crate::card::CounterType;
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

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

fn cast_adventure(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastAdventure {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast adventure");
    drain_stack(g);
}

/// Icewrought Sentry grows when you tap an opponent's creature.
#[test]
fn icewrought_sentry_pumps_on_you_tap() {
    let mut g = two_player_game();
    let sentry = g.add_card_to_battlefield(0, catalog::icewrought_sentry());
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: enemy, actor: Some(0) }]);
    drain_stack(&mut g);
    let s = g.computed_permanent(sentry).unwrap();
    assert_eq!((s.power, s.toughness), (4, 4), "+2/+1 when you tap an enemy creature");
}

/// Galvanic Giant taps and stuns an opponent's creature when you cast a MV-5+ spell.
#[test]
fn galvanic_giant_high_mv_tap_stun() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::galvanic_giant());
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_hand(0, catalog::serra_angel()); // MV 5
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, big, None);
    let e = g.battlefield_find(enemy).unwrap();
    assert!(e.tapped, "enemy tapped");
    assert_eq!(e.counter_count(CounterType::Stun), 1, "stun counter added");
}

/// Aquatic Alchemist grows only on the first instant/sorcery each turn.
#[test]
fn aquatic_alchemist_first_spell_only() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let alch = g.add_card_to_battlefield(0, catalog::aquatic_alchemist());
    let b1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let b2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, b1, Some(Target::Player(1)));
    cast(&mut g, b2, Some(Target::Player(1)));
    assert_eq!(g.computed_permanent(alch).unwrap().power, 3, "+2/+0 once (first spell only)");
}

/// Rip the Seams destroys a tapped creature.
#[test]
fn rip_the_seams_destroys_tapped() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let enemy = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.battlefield_find_mut(enemy).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::threadbind_clique());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_adventure(&mut g, id, Some(Target::Permanent(enemy)));
    assert!(g.battlefield_find(enemy).is_none(), "tapped creature destroyed");
}

/// Swift Spiral flickers a nontoken creature (exiled now, returns later).
#[test]
fn swift_spiral_flickers() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::twining_twins());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_adventure(&mut g, id, Some(Target::Permanent(mine)));
    assert!(g.battlefield_find(mine).is_none(), "creature exiled by the flicker");
}

/// Spellscorn Coven makes each opponent discard on entry.
#[test]
fn spellscorn_coven_etb_discard() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::spellscorn_coven());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let opp_hand = g.players[1].hand.len();
    cast(&mut g, id, None);
    assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent discarded a card");
}
