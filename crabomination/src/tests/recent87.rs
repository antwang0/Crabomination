//! Functionality tests for `catalog::sets::decks::recent87`.

use crate::card::{CreatureType, Keyword, Subtypes};
use crate::catalog;
use crate::game::two_player_game;
use crate::mana::Color;

/// A vanilla creature of the given types for tribal-count tests.
fn vanilla(name: &'static str, types: Vec<CreatureType>) -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name,
        cost: crate::mana::cost(&[crate::mana::generic(1)]),
        card_types: vec![crate::card::CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

#[test]
fn coat_of_arms_scales_with_shared_types() {
    let mut g = two_player_game();
    // Three Goblins (share "Goblin") + one lone Elf.
    let g1 = g.add_card_to_battlefield(0, vanilla("Gob A", vec![CreatureType::Goblin]));
    let g2 = g.add_card_to_battlefield(0, vanilla("Gob B", vec![CreatureType::Goblin]));
    let g3 = g.add_card_to_battlefield(1, vanilla("Gob C", vec![CreatureType::Goblin]));
    let elf = g.add_card_to_battlefield(0, vanilla("Lone Elf", vec![CreatureType::Elf]));
    g.add_card_to_battlefield(0, catalog::coat_of_arms());
    let cp = g.compute_battlefield();
    let pt = |id| { let c = cp.iter().find(|c| c.id == id).unwrap(); (c.power, c.toughness) };
    // Each Goblin shares with the other two → +2/+2 → 3/3.
    assert_eq!(pt(g1), (3, 3));
    assert_eq!(pt(g2), (3, 3));
    assert_eq!(pt(g3), (3, 3), "shared type counts across controllers");
    // The Elf shares with nobody → unchanged 1/1.
    assert_eq!(pt(elf), (1, 1));
}

#[test]
fn coat_of_arms_changeling_shares_with_everyone() {
    let mut g = two_player_game();
    let gob = g.add_card_to_battlefield(0, vanilla("Gob", vec![CreatureType::Goblin]));
    let mut cl = vanilla("Shifty", vec![CreatureType::Shapeshifter]);
    cl.keywords = vec![Keyword::Changeling];
    let clid = g.add_card_to_battlefield(0, cl);
    g.add_card_to_battlefield(0, catalog::coat_of_arms());
    let cp = g.compute_battlefield();
    let pt = |id| { let c = cp.iter().find(|c| c.id == id).unwrap(); (c.power, c.toughness) };
    // Changeling shares a type with the Goblin, so both get +1/+1.
    assert_eq!(pt(gob), (2, 2), "Goblin shares with the Changeling");
    assert_eq!(pt(clid), (2, 2), "Changeling shares with the Goblin");
}

#[test]
fn akromas_memorial_grants_six_keywords() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::akromas_memorial());
    let cp = g.compute_battlefield();
    let kws = &cp.iter().find(|c| c.id == bear).unwrap().keywords;
    for kw in [Keyword::Flying, Keyword::FirstStrike, Keyword::Vigilance, Keyword::Trample,
               Keyword::Haste, Keyword::Protection(Color::Black), Keyword::Protection(Color::Red)] {
        assert!(kws.contains(&kw), "granted {kw:?}");
    }
}
