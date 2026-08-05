//! CR conformance for this run:
//! - CR 506.2 — a permanent with "can't be attacked" isn't a legal attack
//!   target, for planeswalkers and battles alike.
//! - CR 118 / 305 — "you may play that card" covers land plays from exile.
//! - CR 614 — an as-enters replacement resolves before the enters-with-counters
//!   replacement, so a count can read what it did.
//! - CR 716.2 — a Class's level-gated cost static applies only at that level.

use crabomination::card::{CounterType, MayPlayDuration, MayPlayPermission};
use crabomination::catalog;
use crabomination::game::actions::cost_reduction_for_spell;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
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

/// CR 506.2 — a granted "can't be attacked" removes a planeswalker from the
/// legal attack targets.
#[test]
fn cr_506_2_cant_be_attacked_planeswalker_is_not_a_legal_target() {
    let mut g = game();
    let pw = g.add_card_to_battlefield(1, catalog::the_aetherspark());
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Planeswalker(pw) }])
            .is_ok(),
        "legal while nothing grants the restriction"
    );
    g.attacking.clear();
    // Attach it to a creature — The Aetherspark's own static then grants the
    // restriction (CR 613 layer 6).
    let host = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(pw).unwrap().attached_to = Some(host);
    assert!(
        g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Planeswalker(pw) }])
            .is_err(),
        "and illegal once it does"
    );
}

/// CR 118 / 305.1 — a play permission on an exiled card lets its holder take
/// the land drop from exile; another seat's permission doesn't.
#[test]
fn cr_118_may_play_permission_covers_a_land_in_exile() {
    let mut g = game();
    let land = g.add_card_to_exile(0, catalog::forest());
    let grant = |g: &mut GameState, player: usize| {
        let turn = g.turn_number;
        g.find_card_anywhere_mut(land).unwrap().may_play_until = Some(MayPlayPermission {
            player,
            granted_turn: turn,
            duration: MayPlayDuration::WhileExiled,
            exile_after: false,
            miracle: false,
        });
    };
    grant(&mut g, 1);
    assert!(g.perform_action(GameAction::PlayLand(land)).is_err(), "not your permission");
    grant(&mut g, 0);
    g.perform_action(GameAction::PlayLand(land)).expect("your permission, your land drop");
    assert!(g.battlefield.iter().any(|c| c.id == land));
}

/// CR 614 — the as-enters replacement runs first, so an enters-with-counters
/// count reads the state it produced.
#[test]
fn cr_614_as_enters_effect_precedes_enters_with_counters() {
    let mut g = game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ooze = g.add_card_to_hand(0, catalog::mimeoplasm_revered_one());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: ooze,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(1),
    })
    .expect("cast for X=1");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ooze).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3,
        "three counters for the one creature the as-enters exile took"
    );
}

/// CR 716.2 — a Class's level-2 cost static is inert at level 1.
#[test]
fn cr_716_2_class_level_gates_a_cost_static() {
    let mut g = game();
    let talent = g.add_card_to_battlefield(0, catalog::artists_talent());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let card = g.find_card_anywhere(bolt).unwrap().clone();
    assert_eq!(cost_reduction_for_spell(&g, 0, &card, None), 0);
    g.battlefield_find_mut(talent).unwrap().class_level = 2;
    assert_eq!(cost_reduction_for_spell(&g, 0, &card, None), 1);
}
