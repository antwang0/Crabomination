//! Duskmourn's remaining Rooms (`decks::dsk_rooms`).

use crabomination::card::CardDefinition;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// Unlock a still-locked door through the real special action.
fn unlock(g: &mut GameState, id: CardId, right: bool) {
    flood(g, 0);
    g.perform_action(GameAction::UnlockRoomDoor { card_id: id, right }).expect("unlock door");
    drain_stack(g);
}

/// Cast a door and let its unlock triggers resolve.
fn cast_door(g: &mut GameState, def: CardDefinition, right: bool) -> CardId {
    let id = g.add_card_to_hand(0, def);
    flood(g, 0);
    g.perform_action(GameAction::CastRoomDoor { card_id: id, right }).expect("cast door");
    drain_stack(g);
    id
}

/// Central Elevator tutors a Room whose name isn't already on your board.
#[test]
fn central_elevator_finds_a_room_you_dont_control() {
    let mut g = main_phase();
    let room = g.add_card_to_library(0, catalog::mirror_room_fractured_realm());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(room))]));
    cast_door(&mut g, catalog::central_elevator_promising_stairs(), false);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Mirror Room // Fractured Realm"),
        "the Room card was tutored to hand"
    );
}

/// Promising Stairs wins once eight differently-named doors stand open.
#[test]
fn promising_stairs_wins_on_eight_open_doors() {
    let mut g = main_phase();
    let rooms = [
        catalog::central_elevator_promising_stairs(),
        catalog::charred_foyer_warped_space(),
        catalog::dazzling_theater_prop_room(),
        catalog::dollmakers_shop_porcelain_gallery(),
    ];
    for def in rooms {
        let id = g.add_card_to_battlefield(0, def);
        unlock(&mut g, id, false);
        unlock(&mut g, id, true);
    }
    g.add_card_to_library(0, catalog::island());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.game_over, Some(Some(0)), "eight distinct open doors win the game");
}

/// Warped Space waives the impulse cost once, then makes you pay again.
#[test]
fn warped_space_frees_one_exile_cast_per_turn() {
    let mut g = main_phase();
    cast_door(&mut g, catalog::charred_foyer_warped_space(), true);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::shivan_dragon());
    }
    let foyer = g.add_card_to_battlefield(0, catalog::charred_foyer_warped_space());
    unlock(&mut g, foyer, false);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.may_play_until.is_some()).map(|c| c.id).unwrap();
    // Empty pool: the free cast still goes through.
    g.players[0].mana_pool = Default::default();
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: exiled,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("first exile cast is free");
    assert!(g.players[0].free_exile_cast_used_this_turn, "the once-per-turn waiver is spent");
}

/// Prop Room untaps your creatures on the opponent's untap step.
#[test]
fn prop_room_untaps_your_creatures_off_turn() {
    let mut g = main_phase();
    cast_door(&mut g, catalog::dazzling_theater_prop_room(), true);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.active_player_idx = 1;
    g.do_untap();
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped during the opponent's untap");
}

/// Porcelain Gallery sets your team's base P/T to your creature count.
#[test]
fn porcelain_gallery_sets_base_pt_to_creature_count() {
    let mut g = main_phase();
    cast_door(&mut g, catalog::dollmakers_shop_porcelain_gallery(), true);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "two creatures → base 2/2");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "a third creature grows the whole team");
}

/// Fractured Realm doubles a triggered ability of a permanent you control.
#[test]
fn fractured_realm_doubles_your_permanent_triggers() {
    let mut g = main_phase();
    cast_door(&mut g, catalog::mirror_room_fractured_realm(), true);
    let shop = g.add_card_to_battlefield(0, catalog::dollmakers_shop_porcelain_gallery());
    unlock(&mut g, shop, false);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Toy").count(),
        2,
        "the Shop's attack trigger fired twice"
    );
}

/// Misty Salon's Spirit is sized by the open doors you control.
#[test]
fn misty_salon_spirit_counts_open_doors() {
    let mut g = main_phase();
    let other = g.add_card_to_battlefield(0, catalog::dazzling_theater_prop_room());
    unlock(&mut g, other, false);
    cast_door(&mut g, catalog::smoky_lounge_misty_salon(), true);
    let spirit = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Spirit")
        .expect("Spirit minted");
    let cp = g.computed_permanent(spirit.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "two open doors → a 2/2");
}

/// Smoky Lounge's ritual mana only pays for Rooms and doors.
#[test]
fn smoky_lounge_mana_is_room_restricted() {
    let mut g = main_phase();
    cast_door(&mut g, catalog::smoky_lounge_misty_salon(), false);
    g.players[0].mana_pool = Default::default();
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the restricted {{R}}{{R}} can't pay for a creature"
    );
    let room = g.add_card_to_hand(0, catalog::smoky_lounge_misty_salon());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
        .expect("but it does pay for a Room door");
}
