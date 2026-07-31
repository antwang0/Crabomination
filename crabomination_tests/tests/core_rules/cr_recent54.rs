//! CR conformance for this run's engine work:
//! - CR 205.1b — an animation that names a card type REPLACES the type line.
//! - CR 300.2a — a land that's also another card type can only be played.
//! - CR 309 — dungeon cards: the venture marker, the one-dungeon rule and
//!   completion.

use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn all_mana(g: &mut GameState, seat: usize) {
    for c in [
        crabomination::mana::Color::White,
        crabomination::mana::Color::Blue,
        crabomination::mana::Color::Black,
        crabomination::mana::Color::Red,
        crabomination::mana::Color::Green,
    ] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

// ── CR 205 — Type Line ──────────────────────────────────────────────────────

/// CR 205.1b — "it becomes a 2/2 Soldier creature" sets the card type rather
/// than adding to it: Opal Caryatid stops being an enchantment.
#[test]
fn cr_205_1b_animation_replaces_the_type_line() {
    let mut g = two_player_game();
    let opal = g.add_card_to_battlefield(0, catalog::opal_caryatid());
    assert!(g.computed_permanent(opal).unwrap().card_types.contains(&CardType::Enchantment));
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    all_mana(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    cast(&mut g, bear, None);
    let cp = g.computed_permanent(opal).unwrap();
    assert_eq!(cp.card_types, vec![CardType::Creature]);
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// CR 205.1b — a later type-setting effect wins on timestamp: Opal Acrolith's
/// `{0}` puts the enchantment type back and drops the creature type.
#[test]
fn cr_205_1b_a_later_type_change_wins_on_timestamp() {
    let mut g = two_player_game();
    let opal = g.add_card_to_battlefield(0, catalog::opal_acrolith());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    all_mana(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    cast(&mut g, bear, None);
    assert!(g.computed_permanent(opal).unwrap().card_types.contains(&CardType::Creature));
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: opal,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(opal).unwrap().card_types, vec![CardType::Enchantment]);
}

// ── CR 300 — Card types ─────────────────────────────────────────────────────

/// CR 300.2a — an artifact land can only be played as a land; casting it is
/// rejected, and playing it uses the land drop.
#[test]
fn cr_300_2a_an_artifact_land_can_only_be_played() {
    let mut g = two_player_game();
    let seat = g.add_card_to_hand(0, catalog::seat_of_the_synod());
    all_mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: seat,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "an artifact land isn't castable"
    );
    g.perform_action(GameAction::PlayLand(seat)).expect("play");
    let cp = g.computed_permanent(seat).unwrap();
    assert!(cp.card_types.contains(&CardType::Land));
    assert!(cp.card_types.contains(&CardType::Artifact));
}

// ── CR 309 — Dungeons ───────────────────────────────────────────────────────

fn venture(g: &mut GameState, source: CardId, controller: usize) {
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(controller, None, 0, 0);
    ctx.source = Some(source);
    let events = g.resolve_effect(&crabomination::effect::Effect::Venture, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

/// CR 309.4a — a player who ventures with no dungeon in the command zone puts
/// their venture marker on the topmost room of the dungeon they chose.
#[test]
fn cr_309_4a_the_first_venture_starts_on_the_top_room() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.players[0].dungeon.is_none());
    venture(&mut g, src, 0);
    let (name, room) = g.players[0].dungeon.clone().expect("in a dungeon");
    assert_eq!(room, 0, "the marker starts on the topmost room");
    assert!(crabomination_base::dungeons::dungeon_by_name(&name).is_some());
}

/// CR 309.3 — a player owns only one dungeon at a time: a second venture
/// advances the existing marker instead of starting a new dungeon.
#[test]
fn cr_309_3_a_second_venture_advances_the_same_dungeon() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    venture(&mut g, src, 0);
    let (first, _) = g.players[0].dungeon.clone().unwrap();
    venture(&mut g, src, 0);
    let (second, room) = g.players[0].dungeon.clone().expect("still in a dungeon");
    assert_eq!(first, second, "same dungeon");
    assert_ne!(room, 0, "the marker moved on");
}

/// CR 309.5 — resolving the final room's ability completes the dungeon: the
/// marker leaves and the completion tally bumps.
#[test]
fn cr_309_5_the_final_room_completes_the_dungeon() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..8 {
        venture(&mut g, src, 0);
        drain_stack(&mut g);
        if g.players[0].dungeons_completed > 0 {
            break;
        }
    }
    assert_eq!(g.players[0].dungeons_completed, 1);
    assert!(g.players[0].dungeon.is_none(), "the marker leaves on completion");
}

// ── CR 118.4 — costs ────────────────────────────────────────────────────────

/// CR 118.4 — "Pay half your life, rounded up" is a real activation cost: it's
/// paid up front and rounds up.
#[test]
fn cr_118_4_half_life_activation_cost_rounds_up() {
    let mut g = two_player_game();
    let evil = g.add_card_to_battlefield(0, catalog::lurking_evil());
    g.players[0].life = 7;
    g.perform_action(GameAction::ActivateAbility {
        card_id: evil,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 3, "7 → pay 4");
    let cp = g.computed_permanent(evil).unwrap();
    assert_eq!(cp.card_types, vec![CardType::Creature]);
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

// ── CR 122 — counters ───────────────────────────────────────────────────────

/// CR 122.1 — petal counters are their own pool: Lotus Blossom's tally doesn't
/// share with charge counters.
#[test]
fn cr_122_1_petal_counters_are_their_own_kind() {
    let mut g = two_player_game();
    let blossom = g.add_card_to_battlefield(0, catalog::lotus_blossom());
    let c = g.battlefield_find_mut(blossom).unwrap();
    c.add_counters(CounterType::Petal, 3);
    c.add_counters(CounterType::Charge, 1);
    assert_eq!(g.battlefield_find(blossom).unwrap().counter_count(CounterType::Petal), 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: blossom,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3, "three petals, three mana");
}
