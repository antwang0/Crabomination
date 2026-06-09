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
fn color_identity_unions_mdfc_back_face_per_cr_903_4d() {
    // CR 903.4d — the back face of a DFC is included when determining
    // a card's color identity. Construct a synthetic blue front /
    // red back MDFC and assert color_identity returns {U, R}.
    use crate::card::{CardDefinition, CardType, Subtypes};
    use crate::effect::Effect;
    use crate::mana::{cost, r, u, Color};
    let back = CardDefinition {
        name: "Synthetic Back",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    };
    let front = CardDefinition {
        name: "Synthetic Front",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: Some(Box::new(back)),
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    };
    let id = color_identity(&front);
    assert!(id.contains(Color::Blue), "U should be in identity");
    assert!(id.contains(Color::Red), "R should be in identity (from back)");
    assert!(!id.contains(Color::White));
    assert_eq!(id.len(), 2);
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
