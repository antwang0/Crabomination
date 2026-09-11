//! "Choose a color" named White for every headless seat.
//!
//! `AutoDecider` answers `Decision::ChooseColor` with the first legal colour,
//! and every bare site listed it WUBRG. Bot seats set `wants_ui`, so a bare ask
//! never reaches `decide_pending_policy` either: Ward Sliver was protected from
//! nothing, Iona locked a colour nobody was playing, Heraldic Banner anthemed a
//! colour we don't run. The pick keys on what the card *does* with the answer
//! (`CardDefinition::chosen_color_aimed_at_opponents`), and both censuses are
//! `color_weights` — the one walker, after three hand-written copies of it.

use crabomination::card::{CardDefinition, CardType, Keyword};
use crabomination::catalog;
use crabomination::effect::Effect;
use crabomination::game::effects::EffectContext;
use crabomination::game::*;
use crabomination::mana::{self, Color};

fn pip(name: &'static str, c: Color) -> CardDefinition {
    let one = match c {
        Color::White => mana::w(),
        Color::Blue => mana::u(),
        Color::Black => mana::b(),
        Color::Red => mana::r(),
        Color::Green => mana::g(),
    };
    CardDefinition {
        name,
        cost: mana::cost(&[one]),
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

/// The classifier: a card whose chosen colour feeds protection / a lock / a
/// prevention is read off the opponents' side, one whose chosen colour feeds an
/// anthem or a mana ability off ours. Both halves are ~21 cards, which is why
/// one blanket answer was wrong for about half of them.
#[test]
fn chosen_color_consumers_split_hostile_from_friendly() {
    for f in [
        catalog::ward_sliver as fn() -> CardDefinition,
        catalog::voice_of_all,
        catalog::story_circle,
        catalog::iona_shield_of_emeria,
        catalog::teferis_moat,
    ] {
        let def = f();
        assert!(def.chosen_color_aimed_at_opponents(), "{} reads it off them", def.name);
    }
    for f in [
        catalog::heraldic_banner as fn() -> CardDefinition,
        catalog::hall_of_triumph,
        catalog::caged_sun,
        catalog::utopia_sprawl,
        catalog::diamond_mare,
    ] {
        let def = f();
        assert!(!def.chosen_color_aimed_at_opponents(), "{} reads it off us", def.name);
    }
}

/// Ward Sliver grants protection from the chosen colour, so the headless pick is
/// the colour the opponents are most invested in — not White.
#[test]
fn ward_sliver_names_the_colour_the_opponents_play() {
    let mut g = two_player_game();
    let sliver = g.add_card_to_battlefield(0, catalog::ward_sliver());
    for i in 0..3 {
        g.add_card_to_battlefield(1, pip(["R1", "R2", "R3"][i], Color::Red));
    }
    g.add_card_to_battlefield(1, pip("W1", Color::White));
    let ctx = EffectContext::for_ability(sliver, 0, None);
    g.resolve_effect(&Effect::ChooseColorForSelf, &ctx).unwrap();
    assert_eq!(g.battlefield_find(sliver).unwrap().chosen_color, Some(Color::Red));
}

/// The same census counts hands, which the battlefield-only copy this replaces
/// did not: an opponent holding black spells and controlling nothing used to
/// read as "no colour", i.e. White.
#[test]
fn the_hostile_census_counts_hands_not_just_boards() {
    let mut g = two_player_game();
    let voice = g.add_card_to_battlefield(0, catalog::voice_of_all());
    g.add_card_to_hand(1, pip("B1", Color::Black));
    g.add_card_to_hand(1, pip("B2", Color::Black));
    let ctx = EffectContext::for_ability(voice, 0, None);
    g.resolve_effect(&Effect::ChooseColorForSelf, &ctx).unwrap();
    assert_eq!(g.battlefield_find(voice).unwrap().chosen_color, Some(Color::Black));
}

/// Heraldic Banner anthems creatures of the chosen colour — ours. The pick is
/// our own investment even when the opponents are heavier in another colour.
#[test]
fn heraldic_banner_names_our_own_colour() {
    let mut g = two_player_game();
    let banner = g.add_card_to_battlefield(0, catalog::heraldic_banner());
    g.add_card_to_battlefield(0, pip("G1", Color::Green));
    g.add_card_to_battlefield(0, pip("G2", Color::Green));
    for i in 0..4 {
        g.add_card_to_battlefield(1, pip(["R1", "R2", "R3", "R4"][i], Color::Red));
    }
    let ctx = EffectContext::for_ability(banner, 0, None);
    g.resolve_effect(&Effect::ChooseColorForSelf, &ctx).unwrap();
    assert_eq!(g.battlefield_find(banner).unwrap().chosen_color, Some(Color::Green));
}

/// `GrantProtectionFromChosenColor` (Gods Willing, Mother of Runes — 41 cards)
/// kept a fourth hand-written copy of the opposing census, battlefield-only.
/// It shares the one census now, so the grant reads hands too.
#[test]
fn granted_protection_names_the_colour_held_against_us() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, pip("U1", Color::Blue));
    g.add_card_to_hand(1, pip("U2", Color::Blue));
    let ctx = EffectContext::for_ability(bear, 0, Some(Target::Permanent(bear)));
    g.resolve_effect(
        &Effect::GrantProtectionFromChosenColor {
            what: crabomination::effect::Selector::Target(0),
            duration: crabomination::effect::Duration::EndOfTurn,
        },
        &ctx,
    )
    .unwrap();
    let kws = g.computed_permanent(bear).unwrap().keywords().to_vec();
    assert!(
        kws.iter().any(|k| matches!(k, Keyword::Protection(Color::Blue))),
        "{kws:?}"
    );
}

/// Realmwright's "choose a basic land type" rides the colour decision and used
/// to fall through to Forest. It names what the hand needs.
#[test]
fn realmwright_names_the_basic_type_the_hand_wants() {
    let mut g = two_player_game();
    let wright = g.add_card_to_battlefield(0, catalog::realmwright());
    g.add_card_to_hand(0, pip("U1", Color::Blue));
    g.add_card_to_hand(0, pip("U2", Color::Blue));
    let ctx = EffectContext::for_ability(wright, 0, None);
    g.resolve_effect(&Effect::ChooseBasicLandTypeForSource, &ctx).unwrap();
    assert_eq!(
        g.battlefield_find(wright).unwrap().chosen_land_type,
        Some(crabomination::card::LandType::Island)
    );
}
