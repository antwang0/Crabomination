//! CR conformance for this run:
//! - CR 615.8 — a "next time … would deal damage to you" shield soaks exactly
//!   one instance and only damage aimed at its owner.
//! - CR 701.19c — a library search shuffles even when nothing is taken.
//! - CR 611.2c — a "for as long as this remains attached" effect ends the
//!   moment its source unattaches.
//! - CR 702.165 — a permanent spell's promised gift resolves as it enters.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, ZoneDest};
use crabomination::game::effects::{EffectContext, EntityRef};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn game() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    mana(g, 0);
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

/// CR 615.8 — the shield expires after one damage event, and damage the same
/// source deals to a *creature* is never soaked by a "to you" shield.
#[test]
fn cr_615_8_next_damage_shield_is_one_instance_and_seat_scoped() {
    let mut g = game();
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::new_way_forward());
    cast(&mut g, spell, None);

    // The creature hit is not "damage to you" — the shield ignores it.
    let mut ev = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(bear), 1, Some(dragon), &mut ev);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1);

    let opp_before = g.players[1].life;
    let mut ev = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 3, Some(dragon), &mut ev);
    assert_eq!(g.players[0].life, 20, "first instance prevented");
    assert_eq!(g.players[1].life, opp_before - 3, "and reflected");

    let mut ev = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 3, Some(dragon), &mut ev);
    assert_eq!(g.players[0].life, 17, "the shield is spent");
}

/// CR 701.19c — searching shuffles the library even when the player takes
/// nothing.
#[test]
fn cr_701_19c_search_any_number_shuffles_on_an_empty_pick() {
    let mut g = game();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let before = g.players[0].library.iter().map(|c| c.id).collect::<Vec<_>>();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    // A filter nothing matches: the pick is empty, the shuffle still happens.
    let events = g
        .resolve_effect(
            &Effect::SearchAnyNumber {
                who: PlayerRef::You,
                filter: crabomination::card::SelectionRequirement::Land,
                to: ZoneDest::Exile,
            },
            &ctx,
        )
        .expect("search");
    assert_eq!(g.players[0].library.len(), before.len(), "nothing was taken");
    assert!(
        events.iter().any(|e| matches!(e, GameEvent::LibraryShuffled { .. })),
        "the library was shuffled anyway"
    );
}

/// CR 611.2c — the Assimilation Aegis copy is bound to the attachment, not to
/// the Equipment merely being on the battlefield.
#[test]
fn cr_611_2c_while_attached_effect_ends_on_unattach() {
    let mut g = game();
    g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let aegis = g.add_card_to_battlefield(0, catalog::assimilation_aegis());
    g.fire_self_etb_triggers(aegis, 0);
    drain_stack(&mut g);
    let bearer = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: aegis, target: bearer }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bearer).unwrap().power, 5);

    // The Equipment stays on the battlefield; only the link is broken.
    g.battlefield_find_mut(aegis).unwrap().attached_to = None;
    let _ = g.check_state_based_actions();
    assert_eq!(g.computed_permanent(bearer).unwrap().power, 2);
}

/// CR 702.165 — a permanent spell resolves its gifted effect as it enters; the
/// plain cast does not.
#[test]
fn cr_702_165_permanent_gift_resolves_on_entry() {
    for gifted in [false, true] {
        let mut g = game();
        let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let sword = g.add_card_to_hand(0, catalog::starforged_sword());
        mana(&mut g, 0);
        let target = Some(Target::Permanent(host));
        let action = if gifted {
            GameAction::CastGift {
                card_id: sword,
                target,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        } else {
            GameAction::CastSpell {
                card_id: sword,
                target,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        };
        g.perform_action(action).expect("cast");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().any(|c| c.controller == 1 && c.is_token),
            gifted,
            "gift token only when promised"
        );
        assert_eq!(
            g.battlefield_find(sword).unwrap().attached_to.is_some(),
            gifted,
            "the attach rider is gated on the promise"
        );
    }
}
