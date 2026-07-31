//! CR conformance for the card types and zones the coverage report flagged:
//! - CR 408 — the command zone (emblems, commanders, and the CR 903.9b
//!   redirect) is not the battlefield and not a permanent zone.
//! - CR 718 — Prototype cards' alternative cost / P/T / color set.
//! - CR 720 — Omen cards' alternative spell half and the shuffle-on-resolve.
//! - CR 602.1b — "Activate only if …" restrictions.

use crabomination::card::Zone;
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef};
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

// ── CR 408 — Command ────────────────────────────────────────────────────────

/// CR 408.1 — the command zone holds objects that aren't permanents: an
/// emblem lands there and never touches the battlefield or a graveyard.
#[test]
fn cr_408_1_command_zone_objects_are_not_permanents() {
    let mut g = two_player_game();
    let (bf, gy) = (g.battlefield.len(), g.players[0].graveyard.len());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::CreateEmblem {
            who: PlayerRef::You,
            name: "Command Zone Emblem".into(),
            triggered: vec![],
            statics: vec![],
        },
        &ctx,
    )
    .expect("emblem");
    assert_eq!(g.players[0].emblems.len(), 1);
    assert_eq!(g.battlefield.len(), bf, "emblems are not permanents");
    assert_eq!(g.players[0].graveyard.len(), gy, "and can't be destroyed");
}

/// CR 408.3 — a Commander variant card starts in the command zone and every
/// zone change off the battlefield may redirect back to it (CR 903.9b).
#[test]
fn cr_408_3_commanders_start_and_return_to_the_command_zone() {
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    assert_eq!(g.players[0].command.len(), 1);
    for would_be in [Zone::Graveyard, Zone::Exile, Zone::Hand, Zone::Library] {
        assert_eq!(g.resolve_zone_change(cmd, Zone::Battlefield, would_be), Zone::Command);
    }
}

// ── CR 718 — Prototype cards ────────────────────────────────────────────────

/// CR 718.4 — outside the stack and battlefield a prototype card has only its
/// normal characteristics; the alternative set is carried alongside.
#[test]
fn cr_718_4_prototype_card_keeps_its_normal_characteristics_in_hand() {
    let def = catalog::goring_warplow();
    assert_eq!((def.power, def.toughness), (5, 4));
    assert_eq!(def.cost.cmc(), 6);
    let proto = def.has_prototype().expect("prototype face");
    assert_eq!((proto.power, proto.toughness), (1, 1));
    assert_eq!(proto.cost.cmc(), 2);
}

/// CR 718.3b — a prototyped permanent has ONLY the alternative mana cost,
/// power, and toughness, and takes its color from that cost (CR 105.2).
#[test]
fn cr_718_3b_prototyped_permanent_uses_only_the_alternative_characteristics() {
    let mut g = two_player_game();
    let warplow = g.add_card_to_hand(0, catalog::goring_warplow());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastPrototype {
        card_id: warplow,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast for the prototype cost");
    drain_stack(&mut g);
    let cp = g.computed_permanent(warplow).expect("on the battlefield");
    assert_eq!((cp.power, cp.toughness), (1, 1), "the 5/4 body is gone");
    let card = g.battlefield_find(warplow).expect("permanent");
    assert_eq!(card.definition.cost.cmc(), 2, "and so is the printed six-drop cost");
    assert!(card.cast_as_prototype);
    // CR 718.5 — everything else is unchanged.
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Deathtouch));
}

/// CR 718.3 — casting normally is still available and yields the big body.
#[test]
fn cr_718_3_normal_cast_keeps_the_printed_body() {
    let mut g = two_player_game();
    let warplow = g.add_card_to_hand(0, catalog::goring_warplow());
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: warplow,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast normally");
    drain_stack(&mut g);
    let cp = g.computed_permanent(warplow).expect("on the battlefield");
    assert_eq!((cp.power, cp.toughness), (5, 4));
    assert!(!g.battlefield_find(warplow).unwrap().cast_as_prototype);
}

/// CR 718.3a — the prototype cost is the only one evaluated for a prototyped
/// cast: mana that would pay the printed {6} but not {1}{B} is rejected.
#[test]
fn cr_718_3a_prototyped_cast_evaluates_only_the_alternative_cost() {
    let mut g = two_player_game();
    let warplow = g.add_card_to_hand(0, catalog::goring_warplow());
    g.players[0].mana_pool.add_colorless(6);
    assert!(
        g.perform_action(GameAction::CastPrototype {
            card_id: warplow,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "six colorless can't pay the prototype cost's black pip"
    );
}

// ── CR 720 — Omen cards ─────────────────────────────────────────────────────

/// CR 720.2c — an omen card is one card: it's in hand once, and its Omen half
/// isn't separately castable as a creature.
#[test]
fn cr_720_2c_an_omen_card_is_a_single_card() {
    let def = catalog::marang_river_regent();
    let omen = def.has_omen().expect("omen half");
    assert_ne!(omen.name, def.name, "the two halves have different names");
    assert!(def.is_creature(), "the normal characteristics are the creature");
    assert!(omen.card_types.contains(&crabomination::card::CardType::Instant));
}

/// CR 720.3d — an Omen spell is shuffled into its owner's library as it
/// resolves rather than going to the graveyard.
#[test]
fn cr_720_3d_omen_shuffles_into_the_library_on_resolution() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let regent = g.add_card_to_hand(0, catalog::marang_river_regent());
    g.players[0].mana_pool.add(Color::Blue, 4);
    let lib = g.players[0].library.len();
    g.perform_action(GameAction::CastOmen {
        card_id: regent,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Omen half");
    drain_stack(&mut g);
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.id == regent),
        "the Omen never reaches the graveyard"
    );
    assert!(g.players[0].library.iter().any(|c| c.id == regent), "it went to the library");
    // Coil and Catch draws 3 and discards 1; the shuffle-in nets the library
    // down 3 and back up 1.
    assert_eq!(g.players[0].library.len(), lib - 3 + 1);
}

/// CR 720.3b — while on the stack as an Omen the spell has only the
/// alternative characteristics, so the creature body never enters.
#[test]
fn cr_720_3b_omen_spell_does_not_become_a_creature() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let regent = g.add_card_to_hand(0, catalog::marang_river_regent());
    g.players[0].mana_pool.add(Color::Blue, 4);
    g.perform_action(GameAction::CastOmen {
        card_id: regent,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(regent).is_none(), "no 6/7 Dragon entered");
}

// ── CR 602.1b — activation restrictions ─────────────────────────────────────

/// CR 602.1b — "Activate only as a sorcery" is rejected while a spell is on
/// the stack and accepted once it clears.
#[test]
fn cr_602_1b_sorcery_speed_activation_needs_an_empty_stack() {
    let mut g = two_player_game();
    let stalker = g.add_card_to_battlefield(0, catalog::ghost_lit_stalker());
    g.battlefield_find_mut(stalker).unwrap().summoning_sick = false;
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::forest());
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Black, 5);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let activate = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: stalker,
            ability_index: 0,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(activate(&mut g).is_err(), "the stack is not empty");
    drain_stack(&mut g);
    activate(&mut g).expect("legal once the stack clears");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "the discard-2 resolved");
}
