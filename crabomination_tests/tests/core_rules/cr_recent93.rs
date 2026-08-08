//! CR conformance for this run:
//! - CR 800.4b — nothing is created under, or handed to, a player who has
//!   left the game.
//! - CR 509.1h — a creature stays blocked after every one of its blockers
//!   leaves combat.
//! - CR 111.7 — a token outside the battlefield ceases to exist at the next
//!   state-based check.

use crabomination::card::{CardId, TokenDefinition, Value};
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn bear_token(who: PlayerRef) -> Effect {
    Effect::CreateToken {
        who,
        count: Value::ONE,
        definition: Box::new(TokenDefinition {
            name: "Bear".into(),
            power: 2,
            toughness: 2,
            card_types: vec![crabomination::card::CardType::Creature],
            ..Default::default()
        }),
    }
}

/// CR 800.4b — "If a token would be created under the control of a player who
/// has left the game, no token is created."
#[test]
fn cr_800_4b_no_token_is_created_for_a_departed_player() {
    let mut g = multi_player_game(3);
    g.players[1].life = 0;
    g.check_state_based_actions();
    assert!(!g.players[1].is_alive(), "seat 1 has left");

    let ctx = EffectContext::for_ability(CardId(0), 0, None);
    let before = g.battlefield.len();
    g.resolve_effect(&bear_token(PlayerRef::Seat(1)), &ctx).expect("resolve");
    assert_eq!(g.battlefield.len(), before, "no token entered for the departed seat");

    g.resolve_effect(&bear_token(PlayerRef::Seat(2)), &ctx).expect("resolve");
    assert_eq!(g.battlefield.len(), before + 1, "a live seat still gets its token");
}

/// CR 800.4b — "If an object would change to the control of a player who has
/// left the game, it doesn't."
#[test]
fn cr_800_4b_control_does_not_pass_to_a_departed_player() {
    let mut g = multi_player_game(3);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[1].life = 0;
    g.check_state_based_actions();

    let ctx = EffectContext::for_ability(
        CardId(0),
        1,
        Some(crabomination::game::types::Target::Permanent(bear)),
    );
    g.resolve_effect(
        &Effect::GainControl {
            what: Selector::Target(0),
            to: None,
            duration: crabomination::effect::Duration::Permanent,
        },
        &ctx,
    )
    .expect("resolve");
    assert_eq!(
        g.battlefield_find(bear).unwrap().controller,
        0,
        "the steal is dropped; the creature stays where it was"
    );
}

/// CR 509.1h — "A creature remains blocked even if all the creatures blocking
/// it are removed from combat."
#[test]
fn cr_509_1h_a_creature_stays_blocked_when_its_blockers_leave() {
    let mut g = two_player_game();
    let attacker = ready(&mut g, 0, catalog::grizzly_bears());
    let blocker = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.declare_blockers(vec![(blocker, attacker)]).expect("block");
    assert!(g.attacker_is_blocked(attacker));

    // Kill the blocker: it leaves combat, but the attacker stays blocked.
    let mut events = vec![];
    g.destroy_permanent(blocker, false, &mut events);
    assert!(g.blockers_of(attacker).is_empty(), "no blockers left");
    assert!(g.attacker_is_blocked(attacker), "the attacker is still a blocked creature");

    let life_before = g.players[1].life;
    g.resolve_combat().expect("combat damage");
    assert_eq!(g.players[1].life, life_before, "a blocked attacker with no blockers hits nothing");
}

/// CR 111.7 — a token that leaves the battlefield ceases to exist at the next
/// state-based check rather than lingering in its new zone.
#[test]
fn cr_111_7_a_token_off_the_battlefield_ceases_to_exist() {
    let mut g = two_player_game();
    let ctx = EffectContext::for_ability(CardId(0), 0, None);
    g.resolve_effect(&bear_token(PlayerRef::You), &ctx).expect("mint");
    let token = g.battlefield.iter().find(|c| c.is_token).expect("a token entered").id;

    let mut events = vec![];
    g.destroy_permanent(token, false, &mut events);
    g.check_state_based_actions();
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.id == token),
        "the token doesn't stay in the graveyard"
    );
    assert!(g.find_card_anywhere(token).is_none(), "it's gone from every zone");
}

/// CR 101.4 — an "each player sacrifices unless they pay" effect asks in
/// APNAP order, not battlefield order.
#[test]
fn cr_101_4_each_unless_pays_asks_in_apnap_order() {
    use crabomination::card::SelectionRequirement as R;
    let mut g = multi_player_game(3);
    // Seed the battlefield out of turn order: seat 2, then 0, then 1.
    let c2 = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let c0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;

    // Nobody can pay, so everything is sacrificed — the order the graveyards
    // fill in is the order the seats were asked.
    let ctx = EffectContext::for_ability(CardId(0), 1, None);
    let events = g
        .resolve_effect(
            &Effect::SacrificeEachUnlessPays {
                filter: R::Creature,
                cost: crabomination::mana::cost(&[crabomination::mana::generic(1)]),
            },
            &ctx,
        )
        .expect("resolve");
    for id in [c0, c1, c2] {
        assert!(g.battlefield_find(id).is_none(), "everything unpaid was sacrificed");
    }
    let sacrificed: Vec<CardId> = events
        .iter()
        .filter_map(|e| match e {
            GameEvent::PermanentSacrificed { card_id, .. } => Some(*card_id),
            _ => None,
        })
        .collect();
    assert_eq!(sacrificed, vec![c1, c2, c0], "APNAP from the active player is 1, 2, 0");
}
