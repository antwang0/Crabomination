//! Cards whose printed text is *about* the Commander format (CR 903) and
//! which need a real commander on the board to do anything — the half of
//! `sets::cmdr` that `format.rs`'s deck validation cannot reach.

use crabomination::card::{
    CardDefinition, CardId, CardType, CreatureType, Keyword, Subtypes, Supertype,
};
use crabomination::catalog;
use crabomination::format::Format;
use crabomination::game::*;
use crabomination::mana::{cost, g};

/// A free-to-name legendary Bear so a seat has a commander to control.
fn bear_commander() -> CardDefinition {
    CardDefinition {
        name: "Test Bear Commander",
        cost: cost(&[g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// A two-seat Commander game with seat 0 holding priority in its own
/// pre-combat main phase and its commander still in the command zone.
fn commander_game() -> GameState {
    let mut g = game_with_format(Format::Commander, 2);
    g.seat_commanders(0, vec![bear_commander()]);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

/// Seat 0's commander, on the battlefield. CR 903.3 makes "is a commander" a
/// property of the `CardId`, not of the zone, so putting the instance on the
/// battlefield and registering that id is the same board the command-zone
/// cast path would produce — and this way the tests are about the two cards
/// under test rather than about casting.
fn commander_on_the_battlefield(g: &mut GameState) -> CardId {
    let id = g.add_card_to_battlefield(0, bear_commander());
    g.players[0].commanders.push(id);
    id
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let cp = g.computed_permanent(id).expect("on battlefield");
    (cp.power, cp.toughness)
}

fn has_kw(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).unwrap().keywords().contains(&kw)
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

// ── Bastion Protector ───────────────────────────────────────────────────────

/// CR 903.3 — "Commander creatures you control get +2/+2 and have
/// indestructible." The anthem is keyed on `IsCommander`, so it reaches the
/// commander and nothing else on the board.
#[test]
fn cr_903_3_bastion_protector_pumps_only_the_commander() {
    let mut g = commander_game();
    let cmd = commander_on_the_battlefield(&mut g);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let protector = g.add_card_to_battlefield(0, catalog::bastion_protector());

    assert_eq!(pt(&g, cmd), (4, 4), "2/2 commander under +2/+2");
    assert!(has_kw(&g, cmd, Keyword::Indestructible), "the commander is indestructible");

    assert_eq!(pt(&g, bears), (2, 2), "a non-commander gets nothing");
    assert!(!has_kw(&g, bears, Keyword::Indestructible));
    assert_eq!(pt(&g, protector), (3, 3), "and neither does the Protector");
    assert!(!has_kw(&g, protector, Keyword::Indestructible));
}

/// With the commander still in the command zone there is nothing on the
/// battlefield for the anthem to find, so the Protector is a vanilla 3/3.
#[test]
fn bastion_protector_finds_nothing_while_the_commander_is_in_the_zone() {
    let mut g = commander_game();
    let protector = g.add_card_to_battlefield(0, catalog::bastion_protector());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, protector), (3, 3));
    assert_eq!(pt(&g, bears), (2, 2));
    assert!(!has_kw(&g, bears, Keyword::Indestructible));
}

// ── Loyal Apprentice ────────────────────────────────────────────────────────

fn thopters(g: &GameState, seat: usize) -> Vec<CardId> {
    g.battlefield
        .iter()
        .filter(|c| c.definition.name == "Thopter" && c.controller == seat)
        .map(|c| c.id)
        .collect()
}

/// Lieutenant (CR 207.2c is the ability word; the gate is the printed "if you
/// control your commander"). With the commander on the board the beginning of
/// combat mints a Thopter, and it has haste the turn it arrives.
#[test]
fn lieutenant_loyal_apprentice_mints_a_hasty_thopter_with_its_commander_out() {
    let mut g = commander_game();
    commander_on_the_battlefield(&mut g);
    g.add_card_to_battlefield(0, catalog::loyal_apprentice());
    assert!(thopters(&g, 0).is_empty(), "nothing before combat");

    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);

    let made = thopters(&g, 0);
    assert_eq!(made.len(), 1, "one 1/1 Thopter");
    let t = g.battlefield_find(made[0]).unwrap();
    assert_eq!((t.definition.power, t.definition.toughness), (1, 1));
    assert!(t.definition.card_types.contains(&CardType::Artifact));
    assert!(has_kw(&g, made[0], Keyword::Flying), "printed on the token");
    assert!(
        has_kw(&g, made[0], Keyword::Haste),
        "granted to the token, not printed on it",
    );
}

/// Without the commander on the battlefield the Lieutenant clause does
/// nothing — the trigger still fires, its intervening "if" is what fails.
#[test]
fn loyal_apprentice_mints_nothing_without_its_commander() {
    let mut g = commander_game();
    g.add_card_to_battlefield(0, catalog::loyal_apprentice());
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(thopters(&g, 0).is_empty(), "no commander, no Thopter");
}
