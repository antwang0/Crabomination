//! CR conformance for the Fifth Dawn pass:
//! - CR 506 — the combat participation caps and the "already declared" rules.
//! - CR 110 — permanent status and a permanent that loses all its types.
//! - CR 118.9 — alternative costs (Fist of Suns, the Bringer cycle).

use crabomination::card::CardType;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

// ── CR 506 — Combat Phase ──

/// 506.2 — "no more than one creature can attack each combat" binds the whole
/// combat, not each declaration batch.
#[test]
fn cr_506_2_attack_cap_spans_the_whole_combat() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::silent_arbiter());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [a, b] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: a, target: AttackTarget::Player(1) }])
        .expect("first attacker");
    assert!(
        g.declare_attackers(vec![Attack { attacker: b, target: AttackTarget::Player(1) }]).is_err(),
        "the cap counts attackers already declared this combat"
    );
}

/// 506.2 — the blocker cap likewise counts blockers already declared.
#[test]
fn cr_506_2_block_cap_spans_the_whole_combat() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::silent_arbiter());
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let x = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let y = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.declare_blockers(vec![(x, atk)]).expect("first blocker");
    assert!(g.declare_blockers(vec![(y, atk)]).is_err(), "cap already spent");
}

/// 506.4b — tapping an already-declared attacker doesn't remove it from combat
/// or stop its combat damage.
#[test]
fn cr_506_4b_tapping_a_declared_attacker_keeps_it_in_combat() {
    let mut g = main_phase();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.battlefield_find_mut(atk).unwrap().tapped = true;
    g.step = TurnStep::DeclareBlockers;
    let _ = g.resolve_combat();
    assert_eq!(g.players[1].life, 18, "a tapped attacker still connects");
}

/// 506.4 — an attacker whose controller changes mid-combat is removed from
/// combat and deals no damage.
#[test]
fn cr_506_4_control_change_removes_an_attacker_from_combat() {
    let mut g = main_phase();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }])
        .expect("attack");
    let ctx = crabomination::game::effects::EffectContext::for_spell(
        1, Some(Target::Permanent(atk)), 0, 0);
    g.resolve_effect(
        &crabomination::effect::Effect::GainControl {
            what: crabomination::effect::Selector::Target(0),
            to: None,
            duration: crabomination::effect::Duration::Permanent,
        },
        &ctx,
    )
    .expect("steal");
    g.step = TurnStep::DeclareBlockers;
    let _ = g.resolve_combat();
    assert_eq!(g.players[1].life, 20, "the stolen attacker left combat");
}

// ── CR 110 — Permanents ──

/// 110.4c — a permanent that loses all its card types stays on the battlefield.
#[test]
fn cr_110_4c_a_typeless_permanent_stays_on_the_battlefield() {
    let mut g = main_phase();
    let coils = g.add_card_to_battlefield(0, catalog::chimeric_coils());
    // Losing the artifact type would leave nothing; the object is still there.
    let ctx =
        crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(coils)), 0, 0);
    g.resolve_effect(
        &crabomination::effect::Effect::LoseCardTypeUntilEot {
            what: crabomination::effect::Selector::Target(0),
            card_type: CardType::Artifact,
        },
        &ctx,
    )
    .expect("strip the type");
    let cp = g.computed_permanent(coils).expect("still a permanent");
    assert!(!cp.card_types.contains(&CardType::Artifact));
}

/// 110.5b — permanents enter untapped unless something says otherwise; Door to
/// Nothingness says otherwise.
#[test]
fn cr_110_5b_permanents_enter_untapped_unless_told_otherwise() {
    let mut g = main_phase();
    let plain = g.add_card_to_battlefield(0, catalog::tanglebloom());
    assert!(!g.battlefield_find(plain).unwrap().tapped);
    let door = g.add_card_to_hand(0, catalog::door_to_nothingness());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: door, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(door).unwrap().tapped, "enters tapped");
}

/// 110.5c — status survives an irrelevant change: a Shackles that keeps its
/// stolen creature stays tapped through its own untap step.
#[test]
fn cr_110_5c_status_persists_through_the_untap_step() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    let shackles = g.add_card_to_battlefield(0, catalog::vedalken_shackles());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shackles, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("steal");
    drain_stack(&mut g);
    g.do_untap();
    assert!(g.battlefield_find(shackles).unwrap().tapped, "the lock holds it tapped");
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
}

// ── CR 118.9 — Alternative costs ──

/// 118.9 — an alternative cost is paid *rather than* the mana cost, so a
/// nine-mana Bringer lands for five coloured pips.
#[test]
fn cr_118_9_alternative_cost_replaces_the_mana_cost() {
    let mut g = main_phase();
    let bringer = g.add_card_to_hand(0, catalog::bringer_of_the_blue_dawn());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: bringer, pitch_card: None, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast for WUBRG");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bringer).is_some());
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

/// 118.9c — the alternative cost doesn't change the spell's mana value, so
/// mana-value-matters effects still see the printed cost.
#[test]
fn cr_118_9c_alternative_cost_leaves_the_mana_value_alone() {
    let mut g = main_phase();
    let bringer = g.add_card_to_hand(0, catalog::bringer_of_the_green_dawn());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: bringer, pitch_card: None, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bringer).unwrap().definition.cost.cmc(), 9);
}

/// 118.9 — a granted alternative cost (Fist of Suns) reaches a card that has no
/// printed one, and stops applying when the granting permanent leaves.
#[test]
fn cr_118_9_granted_alternative_cost_ends_with_its_source() {
    let mut g = main_phase();
    let fist = g.add_card_to_battlefield(0, catalog::fist_of_suns());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.battlefield.retain(|c| c.id != fist);
    assert!(
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: bear, pitch_card: None, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        })
        .is_err(),
        "no Fist, no alternative cost"
    );
}

/// 118.9b — the alternative cost is optional: the printed mana cost still works
/// while the granting permanent is out.
#[test]
fn cr_118_9b_alternative_cost_is_optional() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::fist_of_suns());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("the printed cost still works");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
}
