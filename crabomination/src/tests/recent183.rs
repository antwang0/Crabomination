//! Functionality tests for `catalog::sets::decks::recent183` (OTJ batch).

use crate::card::Keyword;
use crate::catalog;
use crate::game::*;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Ferocification's begin-combat modal buffs one of your creatures.
#[test]
fn ferocification_begin_combat_modal_buffs() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ferocification());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    let buffed = cp.power > 2 || cp.keywords.contains(&Keyword::Menace) || cp.keywords.contains(&Keyword::Haste);
    assert!(buffed, "a mode resolved (+2/+0 or menace+haste)");
}

/// Freestrider Lookout digs a land onto the battlefield when you commit a crime.
#[test]
fn freestrider_lookout_digs_land_on_crime() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::freestrider_lookout());
    // Stack a land on top of the library, then some filler beneath.
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let land = g.add_card_to_library(0, catalog::forest());
    let bf_before = g.battlefield.len();
    g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some(), "the land was put onto the battlefield");
    assert!(g.battlefield.len() > bf_before, "battlefield grew");
}

/// Fleeting Reflection makes your creature a copy of another and untaps it.
#[test]
fn fleeting_reflection_copies_and_untaps() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    let model = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let spell = g.add_card_to_hand(0, catalog::fleeting_reflection());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(model)],
        mode: None,
        x_value: None,
    })
    .expect("cast Fleeting Reflection");
    drain_stack(&mut g);
    let cp = g.computed_permanent(mine).unwrap();
    assert!(!g.battlefield_find(mine).unwrap().tapped, "untapped its target");
    assert!(cp.keywords.contains(&Keyword::Hexproof), "gained hexproof");
    assert_eq!((cp.power, cp.toughness), (3, 3), "became a copy of the 3/3");
}
