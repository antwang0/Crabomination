use crabomination::format::*;
use crabomination::card::CardDefinition;
use crabomination::catalog;

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
    use crabomination::card::{CardDefinition, CardType};
    
    use crabomination::mana::{cost, r, u, Color};
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
    use crabomination::format::{validate_deck, DeckError, Format};
    // A Modern deck running Treasure Cruise is illegal.
    let mut deck: Vec<_> = (0..59).map(|_| crabomination::catalog::forest()).collect();
    deck.push(crabomination::catalog::treasure_cruise());
    let errs = validate_deck(&deck, Format::Modern).unwrap_err();
    assert!(errs.iter().any(|e| matches!(e, DeckError::BannedCard { card_name } if *card_name == "Treasure Cruise")));
    // The same deck is fine in Vintage with one copy…
    assert!(validate_deck(&deck, Format::Vintage).is_ok());
    // …but two copies break the restricted list.
    deck.push(crabomination::catalog::treasure_cruise());
    let errs = validate_deck(&deck, Format::Vintage).unwrap_err();
    assert!(errs.iter().any(|e| matches!(e, DeckError::RestrictedCard { card_name, found: 2 } if *card_name == "Treasure Cruise")));
}

// ── Companion deck restrictions (CR 702.139c) ─────────────────────────────

use crabomination::format::companion_restriction_met;

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

// ── Commander decklist import (CR 903.3 / 903.5) ─────────────────────────────

use crabomination::cube::CardFactory;
use crabomination::decklist::parse_decklist;
use crabomination::pod::decks;

/// `1 Name` per card, as an exporter writes a singleton list.
fn list_lines(cards: &[CardFactory]) -> String {
    cards.iter().map(|f| format!("1 {}\n", f().name)).collect()
}

fn names(cards: &[CardFactory]) -> Vec<&'static str> {
    cards.iter().map(|f| f().name).collect()
}

/// CR 903.3 — the commander designation survives every common export shape:
/// Arena/Moxfield's leading `Commander` header (with and without `Deck`, with a
/// `(1)` count), a trailing header, Moxfield's `*CMDR*`, Archidekt's
/// `[Commander{top}]` category, and MTGO's commander-as-sideboard. Before, the
/// header filed the commander with the maindeck and the designation was lost.
#[test]
fn commander_sections_parse_in_every_export_shape() {
    let (cmd, main) = (decks::SIGARDA_COMMANDERS, decks::SIGARDA_MAIN);
    let cmd_name = cmd[0]().name;
    let body = list_lines(main);
    let shapes = [
        ("arena header", format!("Commander\n1 {cmd_name}\n\nDeck\n{body}")),
        ("header, no Deck", format!("Commander (1)\n1 {cmd_name}\n\n{body}")),
        ("trailing header", format!("Deck\n{body}\nCommander:\n1 {cmd_name}\n")),
        ("moxfield marker", format!("{body}1 {cmd_name} (SNC) 16 *CMDR*\n")),
        ("archidekt tag", format!("1x {cmd_name} (emn) 1 [Commander{{top}}] ^Have,#37d67a^\n{body}")),
        ("mtgo sideboard", format!("{body}\n1 {cmd_name}\n")),
    ];
    for (shape, text) in shapes {
        let parsed = parse_decklist(&text);
        assert!(parsed.unknown.is_empty(), "{shape}: unknown {:?}", parsed.unknown);
        let list = parsed.commander_list().unwrap_or_else(|e| panic!("{shape}: {e:?}"));
        assert_eq!(names(&list.commanders), vec![cmd_name], "{shape}");
        assert_eq!(list.main.len(), 99, "{shape}");
    }

    // A pair (CR 702.124): Partner, and Choose a Background whose second
    // commander is a legendary enchantment.
    for (cmds, main) in [
        (decks::KRARK_COMMANDERS, decks::KRARK_MAIN),
        (decks::ZELLIX_COMMANDERS, decks::ZELLIX_MAIN),
    ] {
        let text = format!("Commander\n{}\nDeck\n{}", list_lines(cmds), list_lines(main));
        let list = parse_decklist(&text).commander_list().expect("legal pair");
        assert_eq!(names(&list.commanders), names(cmds));
    }

    // Maybeboard cards are not part of the deck; `*F*` foil marks are stripped.
    let text =
        format!("Commander\n1 {cmd_name} *F*\n\nDeck\n{body}\nMaybeboard\n1 Lightning Bolt\n");
    let parsed = parse_decklist(&text);
    assert_eq!(parsed.main.len(), 99);
    assert!(parsed.sideboard.is_empty());
    assert!(parsed.commander_list().is_ok());
}

/// CR 903.3 / 903.5a / 903.5c / 702.124 — an imported Commander list is
/// refused with a reason: no commander section, a card outside the
/// commander's identity, two commanders that can't pair, a short deck, a
/// commander that isn't legendary.
#[test]
fn commander_import_rejects_illegal_lists_with_a_reason() {
    let (cmd, main) = (decks::SIGARDA_COMMANDERS, decks::SIGARDA_MAIN);
    let cmd_name = cmd[0]().name;
    let errs = |text: String| parse_decklist(&text).commander_list().err().unwrap_or_default();

    // A plain 100-card pile: no section, no marker, no MTGO sideboard.
    let e = errs(format!("1 {cmd_name}\n{}", list_lines(main)));
    assert!(e.first().is_some_and(|m| m.contains("No commander section")), "{e:?}");

    // Lightning Bolt (red) in a GW deck, in place of one of the 99.
    let e = errs(format!(
        "Commander\n1 {cmd_name}\n\nDeck\n{}1 Lightning Bolt\n",
        list_lines(&main[1..])
    ));
    assert!(e.iter().any(|m| m.contains("Lightning Bolt") && m.contains("identity")), "{e:?}");

    // Sigarda and Judith have no pairing ability.
    let e = errs(format!(
        "Commander\n1 {cmd_name}\n1 {}\n\nDeck\n{}",
        decks::JUDITH_COMMANDERS[0]().name,
        list_lines(&main[1..])
    ));
    assert!(e.iter().any(|m| m.contains("can't be commanders together")), "{e:?}");

    // 99 cards including the commander.
    let e = errs(format!("Commander\n1 {cmd_name}\n\nDeck\n{}", list_lines(&main[1..])));
    assert!(e.iter().any(|m| m.contains("99 cards")), "{e:?}");

    // A non-legendary "commander".
    let e = errs(format!("Commander\n1 Grizzly Bears\n\nDeck\n{}", list_lines(main)));
    assert!(e.iter().any(|m| m.contains("Grizzly Bears")), "{e:?}");
}

/// A player's own list seats in a 2-, 3- or 4-player pod: seat 0 holds the
/// imported deck with its commander in the command zone (CR 903.6), every
/// seat starts at 40 life (CR 903.7), and the opponents are the stock decks.
#[test]
fn custom_commander_deck_seats_in_a_pod_of_any_size() {
    let text = format!(
        "Commander\n1 {}\n\nDeck\n{}",
        decks::EDGAR_COMMANDERS[0]().name,
        list_lines(decks::EDGAR_MAIN)
    );
    let list = parse_decklist(&text).commander_list().expect("legal list");
    let field = crabomination::pod::target_decks();
    for opponents in 1..=3 {
        let state = crabomination::demo::build_custom_commander_state_seeded(
            &list.commanders,
            &list.main,
            &field[..opponents],
            7,
        );
        assert_eq!(state.players.len(), opponents + 1);
        assert!(state.players.iter().all(|p| p.life == 40));
        let me = &state.players[0];
        assert_eq!(me.command.len(), 1);
        assert_eq!(me.command[0].definition.name, "Edgar Markov");
        assert_eq!(me.library.len(), 99);
        let opp = &state.players[1];
        assert_eq!(opp.command[0].definition.name, field[0].commanders[0]().name);
    }
    // Seeded: the same seed deals the same library order.
    let order = |seed| {
        crabomination::demo::build_custom_commander_state_seeded(
            &list.commanders,
            &list.main,
            &field[..3],
            seed,
        )
        .players[0]
        .library
        .iter()
        .map(|c| c.definition.name)
        .collect::<Vec<_>>()
    };
    assert_eq!(order(3), order(3));
}
