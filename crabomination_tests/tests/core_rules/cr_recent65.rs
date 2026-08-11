//! CR conformance for this run's engine work:
//! - CR 115.7c — "change any targets" repoints every declared slot.
//! - CR 707.10 — a spell copy is put on the stack, not cast.
//! - CR 611.2 — a static under a duration wrapper still grants its ability.
//! - CR 601.2b — "discard X cards" as an additional cast cost.

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
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

/// CR 115.7c — Reroute changes *every* target of the ability, not just the
/// first: Drooling Groodion's pump and shrink both move.
#[test]
fn cr_115_7c_reroute_repoints_every_slot() {
    let mut g = main_phase();
    let groodion = g.add_card_to_battlefield(0, catalog::drooling_groodion());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Big enough that the ability's "sacrifice a creature" cost takes the
    // fodder instead.
    let pumped = g.add_card_to_battlefield(0, catalog::gurzigost());
    let shrunk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // The only other legal bodies for the two slots.
    let alt_a = g.add_card_to_battlefield(1, catalog::mystic_familiar());
    let alt_b = g.add_card_to_battlefield(1, catalog::gurzigost());
    g.add_card_to_library(1, catalog::forest());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: groodion,
        ability_index: 0,
        target: Some(Target::Permanent(pumped)),
        additional_targets: vec![Target::Permanent(shrunk)],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    let _ = fodder;
    let reroute = g.add_card_to_hand(1, catalog::reroute());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(alt_a)),
        DecisionAnswer::Target(Target::Permanent(alt_b)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: reroute,
        target: Some(Target::Permanent(groodion)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast reroute");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(pumped).unwrap().power, 6, "the original slot 0 was spared");
    assert_eq!(g.computed_permanent(alt_a).unwrap().power, 3, "slot 0 moved");
    assert_eq!(g.computed_permanent(alt_b).unwrap().power, 4, "slot 1 moved too");
}

/// CR 707.10 — a copy is put on the stack rather than cast, so it doesn't
/// bump the storm count or fire cast-watching triggers.
#[test]
fn cr_707_10_spell_copy_is_not_cast() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::mystic_familiar());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bolt");
    let cast_count = g.spells_cast_this_turn;
    let radiate = g.add_card_to_hand(0, catalog::radiate());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: radiate,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast radiate");
    let after_radiate = g.spells_cast_this_turn;
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both burned");
    assert_eq!(
        g.spells_cast_this_turn, after_radiate,
        "the copies were put on the stack, not cast"
    );
    assert_eq!(after_radiate, cast_count + 1, "only Radiate itself was cast");
}

/// CR 611.2 — a granted activated ability under a Threshold wrapper is
/// reachable at its virtual index once the gate opens, and gone before it.
#[test]
fn cr_611_2_wrapped_grant_surfaces_only_while_open() {
    let mut g = main_phase();
    let aven = g.add_card_to_battlefield(0, catalog::possessed_aven());
    assert!(g.granted_abilities_for(aven).is_empty(), "closed before Threshold");
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    assert_eq!(g.granted_abilities_for(aven).len(), 1, "open past Threshold");
}

/// CR 601.2b — "discard X cards" is a real additional cost on the main cast
/// path, not just on flashback.
#[test]
fn cr_601_2b_discard_x_cost_is_paid_when_cast_from_hand() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::sickening_dreams());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast for X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "two of the three Forests were discarded");
}

/// CR 601.2b — the cast is rejected when the hand can't cover X.
#[test]
fn cr_601_2b_discard_x_cost_rejects_an_empty_hand() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::sickening_dreams());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: Some(2),
        })
        .is_err(),
        "no cards to discard, no spell"
    );
}

/// The bot fires a Threshold-*granted* removal ability, not just printed ones.
#[test]
fn bot_uses_a_granted_removal_ability() {
    use crabomination::server::bot::{Bot, HeuristicBot};

    let mut g = main_phase();
    let aven = g.add_card_to_battlefield(0, catalog::possessed_aven());
    g.battlefield_find_mut(aven).unwrap().summoning_sick = false;
    let prey = g.add_card_to_battlefield(1, catalog::hydromorph_gull());
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    mana(&mut g, 0);
    let action = HeuristicBot::new().next_action(&g, 0).expect("the bot acts");
    assert!(
        matches!(
            action,
            GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. }
                if card_id == aven && t == prey
        ),
        "expected the granted destroy, got {action:?}"
    );
}
