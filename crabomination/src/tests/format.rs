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
    use crate::card::{CardDefinition, CardType};
    
    use crate::mana::{cost, r, u, Color};
    let back = CardDefinition {
        name: "Synthetic Back",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        ..Default::default()
    };
    let front = CardDefinition {
        name: "Synthetic Front",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        back_face: Some(Box::new(back)),
        ..Default::default()
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

/// Banned cards are rejected per format; restricted cards cap at one copy.
#[test]
fn banlist_and_restricted_list_enforced() {
    use crate::format::{validate_deck, DeckError, Format};
    // A Modern deck running Treasure Cruise is illegal.
    let mut deck: Vec<_> = (0..59).map(|_| crate::catalog::forest()).collect();
    deck.push(crate::catalog::treasure_cruise());
    let errs = validate_deck(&deck, Format::Modern).unwrap_err();
    assert!(errs.iter().any(|e| matches!(e, DeckError::BannedCard { card_name } if *card_name == "Treasure Cruise")));
    // The same deck is fine in Vintage with one copy…
    assert!(validate_deck(&deck, Format::Vintage).is_ok());
    // …but two copies break the restricted list.
    deck.push(crate::catalog::treasure_cruise());
    let errs = validate_deck(&deck, Format::Vintage).unwrap_err();
    assert!(errs.iter().any(|e| matches!(e, DeckError::RestrictedCard { card_name, found: 2 } if *card_name == "Treasure Cruise")));
}

// ── Companion deck restrictions (CR 702.139c) ─────────────────────────────

use crate::format::companion_restriction_met;

/// Lurrus — permanents must have mana value ≤ 2; nonpermanents and lands are
/// exempt; an MV-5 permanent breaks it.
#[test]
fn lurrus_permanents_mv_two_or_less() {
    let lurrus = catalog::lurrus_of_the_dream_den();
    let mut ok: Vec<_> = make_deck(catalog::grizzly_bears, 8); // MV-2 creatures
    ok.extend(make_deck(catalog::lightning_bolt, 4)); // MV-1 instants (nonpermanent)
    ok.extend(make_deck(catalog::forest, 20));
    assert!(companion_restriction_met(&lurrus, &ok, 60).is_ok());
    ok.push(catalog::serra_angel()); // MV-5 permanent
    assert!(companion_restriction_met(&lurrus, &ok, 60).is_err());
}

/// Keruga — every nonland card must have MV ≥ 3; lands exempt.
#[test]
fn keruga_nonland_mv_three_or_more() {
    let keruga = catalog::keruga_the_macrosage();
    let mut ok = make_deck(catalog::serra_angel, 8); // MV-5
    ok.extend(make_deck(catalog::forest, 20));
    assert!(companion_restriction_met(&keruga, &ok, 60).is_ok());
    ok.push(catalog::grizzly_bears()); // MV-2 nonland
    assert!(companion_restriction_met(&keruga, &ok, 60).is_err());
}

/// Gyruda — even mana values only (lands exempt); Obosh — odd only.
#[test]
fn gyruda_even_obosh_odd_parity() {
    let gyruda = catalog::gyruda_doom_of_depths();
    let obosh = catalog::obosh_the_preypiercer();
    let even = make_deck(catalog::grizzly_bears, 8); // MV-2
    let odd = make_deck(catalog::lightning_bolt, 8); // MV-1
    assert!(companion_restriction_met(&gyruda, &even, 60).is_ok());
    assert!(companion_restriction_met(&gyruda, &odd, 60).is_err());
    assert!(companion_restriction_met(&obosh, &odd, 60).is_ok());
    assert!(companion_restriction_met(&obosh, &even, 60).is_err());
}

/// Jegantha — no card may contain two of the same mana symbol.
#[test]
fn jegantha_no_duplicate_mana_symbols() {
    let jegantha = catalog::jegantha_the_wellspring();
    let ok = make_deck(catalog::grizzly_bears, 8); // {1}{G} — one G
    assert!(companion_restriction_met(&jegantha, &ok, 60).is_ok());
    let mut bad = ok.clone();
    bad.push(catalog::serra_angel()); // {3}{W}{W} — two W
    assert!(companion_restriction_met(&jegantha, &bad, 60).is_err());
}

/// Lutri — singleton; basics are exempt from the no-duplicates clause.
#[test]
fn lutri_singleton_with_basic_exemption() {
    let lutri = catalog::lutri_the_spellchaser();
    let mut ok = make_deck(catalog::lightning_bolt, 1);
    ok.extend(make_deck(catalog::forest, 30)); // basics: duplicates allowed
    assert!(companion_restriction_met(&lutri, &ok, 60).is_ok());
    ok.extend(make_deck(catalog::lightning_bolt, 1)); // second Bolt
    assert!(companion_restriction_met(&lutri, &ok, 60).is_err());
}

/// Kaheera — every creature card must be one of the named types.
#[test]
fn kaheera_creature_type_restriction() {
    let kaheera = catalog::kaheera_the_orphanguard();
    let beasts = make_deck(catalog::garruks_companion, 8); // Beast
    assert!(companion_restriction_met(&kaheera, &beasts, 60).is_ok());
    let mut bad = beasts.clone();
    bad.push(catalog::grizzly_bears()); // Bear — not a permitted type
    assert!(companion_restriction_met(&kaheera, &bad, 60).is_err());
}

/// Yorion — the deck must hold at least 20 cards beyond the format minimum.
#[test]
fn yorion_deck_size_over_minimum() {
    let yorion = catalog::yorion_sky_nomad();
    let big = make_deck(catalog::forest, 80);
    assert!(companion_restriction_met(&yorion, &big, 60).is_ok());
    let small = make_deck(catalog::forest, 79);
    assert!(companion_restriction_met(&yorion, &small, 60).is_err());
}

/// Umori — every nonland card must share one card type.
#[test]
fn umori_shared_card_type() {
    let umori = catalog::umori_the_collector();
    let mut creatures = make_deck(catalog::grizzly_bears, 8);
    creatures.extend(make_deck(catalog::forest, 20));
    assert!(companion_restriction_met(&umori, &creatures, 60).is_ok());
    let mut mixed = creatures.clone();
    mixed.push(catalog::lightning_bolt()); // instant breaks the shared type
    assert!(companion_restriction_met(&umori, &mixed, 60).is_err());
}
