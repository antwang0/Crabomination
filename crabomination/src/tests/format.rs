use super::*;
use crate::catalog;

fn make_deck(card_fn: fn() -> CardDefinition, count: usize) -> Vec<CardDefinition> {
    (0..count).map(|_| card_fn()).collect()
}

#[test]
fn standard_minimum_deck_size() {
    let rules = Format::Standard.rules();
    assert_eq!(rules.min_deck_size, 60);
    assert_eq!(rules.max_copies, 4);
    assert_eq!(rules.starting_life, 20);
}

#[test]
fn commander_rules() {
    let rules = Format::Commander.rules();
    assert_eq!(rules.min_deck_size, 100);
    assert_eq!(rules.max_deck_size, Some(100));
    assert_eq!(rules.max_copies, 1);
    assert_eq!(rules.starting_life, 40);
    assert!(rules.singleton);
}

#[test]
fn limited_rules() {
    let rules = Format::Draft.rules();
    assert_eq!(rules.min_deck_size, 40);
    assert_eq!(rules.max_copies, u32::MAX);
}

#[test]
fn valid_60_card_deck_passes() {
    let mut deck = make_deck(catalog::lightning_bolt, 4);
    deck.extend(make_deck(catalog::grizzly_bears, 4));
    deck.extend(make_deck(catalog::forest, 52)); // basics are unlimited
    assert!(validate_deck(&deck, Format::Standard).is_ok());
}

#[test]
fn too_few_cards_rejected() {
    let deck = make_deck(catalog::lightning_bolt, 4);
    let errs = validate_deck(&deck, Format::Standard).unwrap_err();
    assert!(errs.iter().any(|e| matches!(e, DeckError::TooFewCards { .. })));
}

#[test]
fn too_many_copies_rejected() {
    let mut deck = make_deck(catalog::lightning_bolt, 5); // 5 > max 4
    deck.extend(make_deck(catalog::forest, 55));
    let errs = validate_deck(&deck, Format::Standard).unwrap_err();
    assert!(errs.iter().any(|e| matches!(e, DeckError::TooManyCopies { .. })));
}

#[test]
fn basic_lands_are_unlimited() {
    // 100 basic forests: valid in Standard (no max deck size) and Commander.
    let deck = make_deck(catalog::forest, 100);
    assert!(validate_deck(&deck, Format::Standard).is_ok());
    assert!(validate_deck(&deck, Format::Commander).is_ok());

    // 5 copies of a non-basic is illegal in Commander (singleton) but fine in Standard.
    let mut nonbasic_deck = make_deck(catalog::forest, 95);
    nonbasic_deck.extend(make_deck(catalog::lightning_bolt, 5)); // 5 bolts
    let errs = validate_deck(&nonbasic_deck, Format::Commander).unwrap_err();
    assert!(errs.iter().any(|e| matches!(e, DeckError::TooManyCopies { .. })));
    // Standard allows up to 4
    let mut standard_deck = make_deck(catalog::forest, 56);
    standard_deck.extend(make_deck(catalog::lightning_bolt, 4));
    assert!(validate_deck(&standard_deck, Format::Standard).is_ok());
}
