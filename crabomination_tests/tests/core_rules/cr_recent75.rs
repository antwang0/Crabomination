//! CR conformance: 502.3 untap restrictions, 508.1g attack costs, 614.9
//! redirection ordering, 615.1 partial prevention, and 120.3 damage memory.

use crabomination::catalog;
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::mountain());
        }
    }
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// CR 502.3 — an "attacked during your last turn" untap gate reads the
/// previous *own* turn, so the creature untaps normally the cycle after.
#[test]
fn cr_502_3_attacked_last_turn_untap_gate_is_one_turn_only() {
    let mut g = main_phase();
    let sled = g.add_card_to_battlefield(0, catalog::goblin_rock_sled());
    g.clear_sickness(sled);
    g.add_card_to_battlefield(1, catalog::mountain());
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: sled, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);

    g.do_untap(); // the turn after the attack: still down
    assert!(g.battlefield_find(sled).expect("sled").tapped);
    g.do_untap(); // and the one after that: free again
    assert!(!g.battlefield_find(sled).expect("sled").tapped);
}

/// CR 502.3 — the gate is read off the *computed* keywords, so an Aura's
/// grant locks a creature that doesn't have the keyword printed.
#[test]
fn cr_502_3_granted_untap_gate_locks_the_host() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let kelp = g.add_card_to_battlefield(1, catalog::tangle_kelp());
    g.battlefield_find_mut(kelp).expect("kelp").attached_to = Some(bear);
    g.battlefield_find_mut(bear).expect("bear").tapped = false;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    g.do_untap();
    assert!(g.battlefield_find(bear).expect("bear").tapped);
}

/// CR 508.1g — a sacrifice attack cost is paid from a shared pool, so two
/// attackers that each want two Islands need four.
#[test]
fn cr_508_1g_sacrifice_attack_costs_share_one_pool() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::leviathan());
    let b = g.add_card_to_battlefield(0, catalog::leviathan());
    for id in [a, b] {
        g.clear_sickness(id);
        g.battlefield_find_mut(id).expect("leviathan").tapped = false;
    }
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.declare_attackers(vec![
            Attack { attacker: a, target: AttackTarget::Player(1) },
            Attack { attacker: b, target: AttackTarget::Player(1) },
        ])
        .is_err(),
        "three Islands can't pay for two attackers",
    );
    // Nothing was spent on the rejected declaration.
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), 3);
}

/// CR 614.9 / 614.5 — a redirect applies once per damage event; the
/// redirected damage isn't redirected again.
#[test]
fn cr_614_9_creature_damage_redirect_applies_once() {
    let mut g = main_phase();
    let blood = g.add_card_to_hand(0, catalog::blood_of_the_martyr());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: blood,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.deal_damage_to_from(EntityRef::Permanent(bear), 2, None, &mut vec![]);
    assert_eq!(g.battlefield_find(bear).expect("bear").damage, 0);
    assert_eq!(g.players[0].life, 18);
}

/// CR 615.1 — a "prevent half, rounded down" shield only soaks its share;
/// the remainder is still dealt. CR 615.13: it fires once.
#[test]
fn cr_615_1_half_prevention_soaks_only_its_share() {
    let mut g = main_phase();
    let sphere = g.add_card_to_battlefield(0, catalog::dark_sphere());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sphere,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    g.deal_damage_to_from(EntityRef::Player(0), 7, None, &mut vec![]);
    assert_eq!(g.players[0].life, 16, "7 damage, 3 prevented");
    g.deal_damage_to_from(EntityRef::Player(0), 7, None, &mut vec![]);
    assert_eq!(g.players[0].life, 9, "the shield is spent");
}

/// CR 120.3 — a source remembers what it has damaged. The record is
/// per-source and survives across turns.
#[test]
fn cr_120_3_a_source_remembers_what_it_damaged() {
    let mut g = main_phase();
    let fallen = g.add_card_to_battlefield(0, catalog::the_fallen());
    let other = g.add_card_to_battlefield(0, catalog::the_fallen());
    g.deal_damage_to_from(EntityRef::Player(1), 1, Some(fallen), &mut vec![]);
    assert_eq!(g.battlefield_find(fallen).expect("fallen").damaged_players_this_game, vec![1]);
    assert!(g.battlefield_find(other).expect("other").damaged_players_this_game.is_empty());
    // A second hit on the same seat doesn't duplicate the entry.
    g.deal_damage_to_from(EntityRef::Player(1), 1, Some(fallen), &mut vec![]);
    assert_eq!(g.battlefield_find(fallen).expect("fallen").damaged_players_this_game, vec![1]);
}

/// CR 115.6 — a "can't be the target of spells unless…" restriction doesn't
/// stop abilities from targeting.
#[test]
fn cr_115_6_spell_only_target_restriction_lets_abilities_through() {
    let mut g = main_phase();
    let lurker = g.add_card_to_battlefield(1, catalog::lurker());
    let cage = g.add_card_to_battlefield(0, catalog::barls_cage());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cage,
        ability_index: 0,
        target: Some(Target::Permanent(lurker)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("an ability may target it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(lurker).expect("lurker").skip_next_untap);
}

/// CR 104.4b / 732.4 — a loop of mandatory triggered abilities with no way to
/// stop draws the game. A state-neutral trigger that keeps re-resolving never
/// moves the fingerprint the watchdog samples, so the watchdog fires.
#[test]
fn cr_104_4b_mandatory_trigger_loop_draws_the_game() {
    use crabomination::effect::Effect;
    let mut g = main_phase();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..(GameState::MANDATORY_LOOP_DRAW_REPEATS + 2) {
        if g.game_over.is_some() {
            break;
        }
        g.stack.push(TriggerPush::new(src, 0, Effect::Noop).build());
        g.resolve_top_of_stack().expect("resolve");
    }
    assert_eq!(g.game_over, Some(None), "mandatory loop is a draw");
}

/// The watchdog must not fire on a *progressing* trigger chain — one that
/// changes the game state each time still resolves normally.
#[test]
fn cr_104_4b_progressing_trigger_chain_is_not_a_draw() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = main_phase();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..(GameState::MANDATORY_LOOP_DRAW_REPEATS + 2) {
        g.stack.push(
            TriggerPush::new(src, 0, Effect::GainLife {
                who: Selector::You,
                amount: Value::ONE,
            })
            .build(),
        );
        g.resolve_top_of_stack().expect("resolve");
    }
    assert!(g.game_over.is_none(), "a chain that gains life each time is progress");
}
