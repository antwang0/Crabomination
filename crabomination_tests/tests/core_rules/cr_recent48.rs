//! CR conformance for this run's sweep:
//! - CR 503 — the upkeep step.
//! - CR 512 — the ending phase.
//! - CR 714 — Saga cards.
//! - CR 721 — station cards.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    for _ in 0..64 {
        if g.step == step {
            return;
        }
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    panic!("never reached {step:?}");
}

// ── CR 503 — Upkeep Step ──

/// CR 503.1a — abilities that triggered at the beginning of upkeep are on the
/// stack before the active player gets priority.
#[test]
fn cr_503_1a_upkeep_triggers_precede_priority() {
    let mut g = two_player_game();
    // Dark Confidant's upkeep trigger fires for its controller.
    g.add_card_to_battlefield(0, catalog::dark_confidant());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.step = TurnStep::Untap;
    g.perform_action(GameAction::PassPriority).ok();
    advance_to(&mut g, TurnStep::Upkeep);
    assert!(!g.stack.is_empty(), "the upkeep trigger is already on the stack");
    assert_eq!(g.priority.player_with_priority, 0, "and the active player holds priority");
}

/// CR 503.1 — the upkeep step has no turn-based actions: nothing is drawn,
/// untapped or sacrificed just by entering it.
#[test]
fn cr_503_1_upkeep_has_no_turn_based_actions() {
    let mut g = two_player_game();
    let hand = g.players[0].hand.len();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.step = TurnStep::Untap;
    advance_to(&mut g, TurnStep::Upkeep);
    assert_eq!(g.players[0].hand.len(), hand, "no draw in upkeep");
    assert!(g.battlefield_find(bear).is_some(), "nothing is sacrificed");
}

// ── CR 512 — Ending Phase ──

/// CR 512.1 — the ending phase is exactly two steps, end then cleanup, and
/// cleanup hands off to the next turn.
#[test]
fn cr_512_1_ending_phase_is_end_then_cleanup() {
    let mut g = two_player_game();
    g.step = TurnStep::PostCombatMain;
    let turn = g.turn_number;
    advance_to(&mut g, TurnStep::End);
    g.perform_action(GameAction::PassPriority).expect("pass");
    g.perform_action(GameAction::PassPriority).expect("pass");
    assert!(
        g.turn_number > turn || g.step == TurnStep::Cleanup,
        "end hands off to cleanup, which hands off to the next turn"
    );
}

/// CR 514.2 (inside the ending phase) — marked damage and "until end of turn"
/// pumps both clear in cleanup.
#[test]
fn cr_512_1_cleanup_clears_damage_and_pumps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().damage = 1;
    g.battlefield_find_mut(bear).unwrap().power_bonus = 3;
    g.step = TurnStep::End;
    advance_to(&mut g, TurnStep::Upkeep);
    let c = g.battlefield_find(bear).expect("survived");
    assert_eq!((c.damage, c.power_bonus), (0, 0));
}

// ── CR 714 — Saga Cards ──

/// CR 714.3a / 714.2b — a Saga enters with one lore counter and its chapter I
/// ability triggers off that placement.
#[test]
fn cr_714_3a_saga_enters_with_a_lore_counter() {
    let mut g = two_player_game();
    let saga = g.add_card_to_hand(0, catalog::history_of_benalia());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: saga,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(saga).unwrap().counter_count(CounterType::Lore), 1);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Knight"), "chapter I fired");
}

/// CR 714.3c — the controller puts a lore counter on each of their Sagas as
/// their precombat main phase begins, and only on their own turn.
#[test]
fn cr_714_3c_lore_counter_at_precombat_main() {
    let mut g = two_player_game();
    for p in 0..2 {
        for _ in 0..5 {
            g.add_card_to_library(p, catalog::island());
        }
    }
    let saga = g.add_card_to_battlefield(0, catalog::history_of_benalia());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 1);
    // The opponent's precombat main doesn't advance it.
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 1;
    advance_to(&mut g, TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(saga).unwrap().counter_count(CounterType::Lore),
        1,
        "an opponent's main phase doesn't tick your Saga"
    );
    // The controller's does.
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    advance_to(&mut g, TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(saga).unwrap().counter_count(CounterType::Lore), 2);
}

/// CR 714.2b — "if the number of lore counters was less than N and became at
/// least N": a proliferated lore counter fires the chapter it crosses, and a
/// multi-counter jump fires every chapter it passes.
#[test]
fn cr_714_2b_chapters_fire_on_every_threshold_crossed() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::history_of_benalia());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 1);
    // Chapters II and III both cross when two counters land at once.
    let ctx = crabomination::game::effects::EffectContext::for_ability(saga, 0, None);
    g.resolve_effect(
        &Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Lore,
            amount: Value::Const(2),
        },
        &ctx,
    )
    .expect("place two lore counters");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Knight"),
        "chapter II minted a Knight on the way past"
    );
    assert!(g.battlefield_find(saga).is_none(), "and chapter III finished the Saga (CR 714.4)");
}

/// CR 714.4 — a Saga at its final chapter number is sacrificed, but not while
/// that chapter's ability is still on the stack.
#[test]
fn cr_714_4_final_chapter_resolves_before_the_sacrifice() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::history_of_benalia());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    g.saga_advance(saga);
    assert!(!g.stack.is_empty(), "chapter III is on the stack");
    assert!(g.battlefield_find(saga).is_some(), "the Saga is still around for it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(saga).is_none(), "then the SBA sacrifices it");
}

// ── CR 721 — Station Cards ──

/// CR 721.2b — a station band's `{N+}` makes the permanent a creature with that
/// base P/T only once it has N or more charge counters.
#[test]
fn cr_721_2b_station_band_animates_at_its_threshold() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::sledge_class_seedship());
    let cp = g.computed_permanent(ship).expect("on battlefield");
    assert!(!cp.card_types.contains(&CardType::Creature), "no charges, no creature");
    g.battlefield_find_mut(ship).unwrap().add_counters(CounterType::Charge, 7);
    let cp = g.computed_permanent(ship).expect("still there");
    assert!(cp.card_types.contains(&CardType::Creature), "{{7+}} animates it");
    assert_eq!((cp.power, cp.toughness), (4, 5));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// CR 721.2c — a station card has no power or toughness outside the
/// battlefield: its printed characteristics carry none.
#[test]
fn cr_721_2c_no_pt_outside_the_battlefield() {
    let def = catalog::sledge_class_seedship();
    assert!(!def.card_types.contains(&CardType::Creature));
    assert_eq!((def.power, def.toughness), (0, 0), "the P/T lives in the station band");
}

/// CR 721.4 — the station ability itself is always available, whatever the
/// charge count, and it charges by the tapped creature's power.
#[test]
fn cr_721_4_station_ability_is_always_active() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::sledge_class_seedship());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ship,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("station with no charges yet");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ship).unwrap().counter_count(CounterType::Charge),
        2,
        "the 2/2 charged it by its power"
    );
}

// ── CR 601.2b — modal activated abilities ──

/// CR 601.2b — the mode of a modal activated ability is chosen as part of the
/// activation. A submitted mode is authoritative, which is the only way to pick
/// a mode whose body takes a target (Teardrop Kami's tap / untap).
#[test]
fn cr_601_2b_activated_ability_mode_is_submitted() {
    let run = |mode: Option<usize>| {
        let mut g = two_player_game();
        let kami = g.add_card_to_battlefield(0, catalog::teardrop_kami());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        g.perform_action(GameAction::ActivateAbility {
            card_id: kami,
            ability_index: 0,
            target: Some(crabomination::game::types::Target::Permanent(bear)),
            additional_targets: vec![],
            x_value: None,
            mode,
        })
        .expect("activate");
        drain_stack(&mut g);
        g.battlefield_find(bear).unwrap().tapped
    };
    assert!(run(Some(0)), "mode 0 taps — the already-tapped bear stays tapped");
    assert!(!run(Some(1)), "mode 1 untaps it");
}

/// The server view surfaces a modal activated ability's mode texts so the
/// client can offer one row per mode.
#[test]
fn cr_601_2b_ability_view_lists_modes() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::teardrop_kami());
    let cv = crabomination::server::view::project(&g, 0);
    let pv = cv.battlefield.iter().find(|p| p.id == kami).expect("on battlefield");
    assert_eq!(pv.abilities[0].modes.len(), 2, "tap / untap");
}

// ── Snapshot schema stability ──

/// Every `#[serde(default)]` field this run added survives a full-state
/// snapshot round-trip, so an in-flight match can be reloaded across a deploy.
#[test]
fn new_serde_fields_survive_snapshot_roundtrip() {
    use crabomination::game::types::{CastProfile, PreventionShield, PreventionTarget, TriggerPush};
    let mut g = two_player_game();
    g.players[0].spell_casts_this_turn.push(CastProfile {
        colors: vec![crabomination::mana::Color::Blue],
        card_types: vec![CardType::Instant],
    });
    g.players[1].lands_entered_this_turn = 2;
    g.prevention_shields.push(PreventionShield {
        target: PreventionTarget::PlayerAndPermanents(0),
        remaining: Some(3),
        ..Default::default()
    });
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.stack.push(
        TriggerPush::new(src, 0, crabomination::effect::Effect::Noop)
            .trigger_player(Some(1))
            .build(),
    );

    let json = serde_json::to_string(&g).expect("serialize");
    let g2: GameState = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g2.players[0].spell_casts_this_turn.len(), 1, "cast profiles round-trip");
    assert_eq!(g2.players[1].lands_entered_this_turn, 2);
    assert_eq!(
        g2.prevention_shields[0].target,
        PreventionTarget::PlayerAndPermanents(0),
        "the team prevention target round-trips",
    );
    assert!(
        matches!(
            g2.stack.first(),
            Some(crabomination::game::types::StackItem::Trigger { trigger_player: Some(1), .. })
        ),
        "the trigger's named player round-trips",
    );
}
