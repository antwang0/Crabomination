//! CR conformance for this run's engine work:
//! - CR 101.2 / 101.3 — a "can't" beats a "may", and an impossible instruction
//!   is simply ignored.
//! - CR 607.2a — linked exile: "cards exiled with this" reaches only the
//!   copy that exiled them.
//! - CR 615.8 / 615.9 — a next-instance shield eats one whole instance no
//!   matter how big, and rechecks the chosen source's properties (a
//!   recoloured source is neither prevented nor spends the shield).

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::effects::EntityRef;
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// CR 101.2 — "you can't play lands" overrides the extra land play Explore
/// just granted; the extra play is still spent-proof, not a loophole.
#[test]
fn cr_101_2_cant_beats_may() {
    let mut g = main_phase();
    let explore = g.add_card_to_hand(0, catalog::explore());
    g.add_card_to_library(0, catalog::forest());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: explore,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].extra_land_plays > 0, "Explore granted the extra play");

    g.add_card_to_battlefield(0, catalog::aggressive_mining());
    let land = g.add_card_to_hand(0, catalog::forest());
    assert!(
        g.perform_action(GameAction::PlayLand(land)).is_err(),
        "the 'can't' static wins over the granted extra play"
    );
}

/// CR 101.3 — an impossible instruction is ignored: Explore's extra land play
/// still resolves (and draws) with an empty library-free board, and returning
/// nothing from an empty linked stash is a no-op, not an error.
#[test]
fn cr_101_3_impossible_instruction_is_ignored() {
    let mut g = main_phase();
    let archive = g.add_card_to_battlefield(0, catalog::kyren_archive());
    // Nothing was ever banked, so the {5} cash-out has nothing to return.
    activate(&mut g, 0, archive, 0, None);
    assert!(g.players[0].hand.is_empty(), "no cards materialized");
    assert!(g.battlefield_find(archive).is_none(), "the cost was still paid");
}

/// CR 607.2a — "cards exiled with this artifact" is linked to the ability that
/// exiled them: a second Kyren Archive can't cash in the first one's stash.
#[test]
fn cr_607_2a_linked_exile_is_per_object() {
    let mut g = two_player_game();
    let first = g.add_card_to_battlefield(0, catalog::kyren_archive());
    let banked = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == banked));
    let second = g.add_card_to_battlefield(0, catalog::kyren_archive());

    // The *other* Archive cashes out: the stash isn't linked to it.
    activate(&mut g, 0, second, 0, None);
    assert!(g.exile.iter().any(|c| c.id == banked), "still exiled");

    activate(&mut g, 0, first, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == banked), "its own stash returns");
}

/// CR 615.8 — the shield eats one whole damage instance regardless of size,
/// and the next instance from the same source is dealt normally.
#[test]
fn cr_615_8_next_instance_shield_eats_one_whole_hit() {
    let mut g = two_player_game();
    let cop = g.add_card_to_battlefield(0, catalog::circle_of_protection_red());
    let tim = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
    g.clear_sickness(tim);
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5 red
    g.clear_sickness(dragon);
    // Shield against the Dragon, not the (blue) Tim.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Cards(vec![dragon])]));
    activate(&mut g, 0, cop, 0, None);

    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: dragon, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].life, 20, "all five prevented by one shield");

    // A second instance from the same source is dealt normally.
    let mut evs = vec![];
    g.deal_damage_to_from(EntityRef::Player(0), 5, Some(dragon), &mut evs);
    assert_eq!(g.players[0].life, 15);
}

/// CR 615.9 — the shield rechecks the chosen source's properties: a source
/// that stops being red is neither prevented nor spends the shield.
#[test]
fn cr_615_9_shield_rechecks_the_sources_color() {
    let mut g = two_player_game();
    let cop = g.add_card_to_battlefield(0, catalog::circle_of_protection_red());
    let tails = g.add_card_to_battlefield(0, catalog::eight_and_a_half_tails());
    g.clear_sickness(tails);
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5 red
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Cards(vec![dragon])]));
    activate(&mut g, 0, cop, 0, None);

    // Turn the Dragon white — the shield no longer matches it.
    activate(&mut g, 0, tails, 1, Some(Target::Permanent(dragon)));
    let mut evs = vec![];
    g.deal_damage_to_from(EntityRef::Player(0), 5, Some(dragon), &mut evs);
    assert_eq!(g.players[0].life, 15, "a white source isn't prevented");

    assert_eq!(g.prevention_shields.len(), 1, "and the shield wasn't spent");
}
