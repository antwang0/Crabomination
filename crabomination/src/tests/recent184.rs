//! Functionality tests for `catalog::sets::decks::recent184` (OTJ outlaws).

use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::*;
use crate::mana::Color;

/// Full Steam Ahead pumps your team and grants trample + block-limit.
#[test]
fn full_steam_ahead_buffs_team() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::full_steam_ahead());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Full Steam Ahead");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::Trample), "gained trample");
    assert!(cp.keywords.contains(&Keyword::CantBeBlockedByMoreThanOne), "block-limited");
}

/// Hellspur Posse Boss makes two Mercenaries and gives other outlaws haste.
#[test]
fn hellspur_posse_boss_tokens_and_haste() {
    let mut g = two_player_game();
    // Another outlaw (Rogue) already out — it should gain haste from the lord.
    let mut rogue = catalog::grizzly_bears();
    rogue.subtypes.creature_types = vec![CreatureType::Rogue];
    let outlaw = g.add_card_to_battlefield(0, rogue);
    g.move_card_to_battlefield_for_test(0, catalog::hellspur_posse_boss());
    drain_stack(&mut g);
    let mercs = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Mercenary))
        .count();
    assert_eq!(mercs, 2, "made two Mercenary tokens");
    assert!(
        g.computed_permanent(outlaw).unwrap().keywords.contains(&Keyword::Haste),
        "other outlaw gained haste",
    );
}

/// Kraum draws and grows when you cast your second spell in a turn.
#[test]
fn kraum_flurry_draws_and_grows() {
    let mut g = two_player_game();
    let kraum = g.add_card_to_battlefield(0, catalog::kraum_violent_cacophony());
    // First spell of the turn (no trigger).
    let s1 = g.add_card_to_hand(0, catalog::divination());
    let s2 = g.add_card_to_hand(0, catalog::divination());
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: s1, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("first spell");
    drain_stack(&mut g);
    let hand_after_first = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: s2, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("second spell");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(kraum).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Kraum grew on the second spell",
    );
    // Divination draws 2; the Flurry draws one more → net > +2 vs before second.
    assert!(g.players[0].hand.len() > hand_after_first, "the Flurry drew a card too");
}

/// At Knifepoint gives your outlaws first strike and makes a Mercenary on a crime.
#[test]
fn at_knifepoint_first_strike_and_crime_token() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::at_knifepoint());
    let mut rogue = catalog::grizzly_bears();
    rogue.subtypes.creature_types = vec![CreatureType::Rogue];
    let outlaw = g.add_card_to_battlefield(0, rogue);
    assert!(
        g.computed_permanent(outlaw).unwrap().keywords.contains(&Keyword::FirstStrike),
        "outlaw has first strike",
    );
    g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
    drain_stack(&mut g);
    let mercs = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Mercenary))
        .count();
    assert_eq!(mercs, 1, "crime made a Mercenary token");
}
