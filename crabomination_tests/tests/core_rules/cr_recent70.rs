//! CR conformance for this run's engine work:
//! - CR 708.2b / 708.9 — turning an already-face-down permanent face down is a
//!   no-op, and a face-down permanent is revealed as it leaves the battlefield.
//! - CR 121.6a — a draw replacement applies even with an empty library.
//! - CR 614.9 — a turn-scoped damage redirect applies to every damage event,
//!   not just the first, and only one redirect happens per event.
//! - CR 613.7 — an indefinite keyword removal outlives the turn.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EntityRef;
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

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

// ── CR 708 — Face-down permanents ───────────────────────────────────────────

/// 708.2b — a second "turn this face down" changes nothing, so the real card
/// stashed underneath survives (and 708.2: the face-down body has no abilities,
/// so the printed one can't be re-activated from under it).
#[test]
fn cr_708_2b_turning_a_face_down_permanent_face_down_again_is_a_noop() {
    let mut g = main_phase();
    let quanar = g.add_card_to_battlefield(0, catalog::mischievous_quanar());
    g.clear_sickness(quanar);
    activate(&mut g, 0, quanar, 0);
    assert!(g.battlefield_find(quanar).unwrap().face_down);
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: quanar,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a face-down permanent has no printed abilities"
    );
    // The effect itself is also a no-op when aimed at a face-down permanent.
    let ctx = crabomination::game::effects::EffectContext::for_ability(quanar, 0, None);
    g.resolve_effect(
        &crabomination::effect::Effect::TurnFaceDown {
            what: crabomination::effect::Selector::This,
        },
        &ctx,
    )
    .expect("resolve");
    let c = g.battlefield_find(quanar).unwrap();
    assert!(c.face_down);
    assert_eq!(
        c.face_up_def.as_ref().map(|d| d.name),
        Some("Mischievous Quanar"),
        "the stashed card is still the real one, not the 2/2 body"
    );
}

/// 708.9 — a face-down permanent is revealed as it leaves the battlefield, so
/// the graveyard holds the real card.
#[test]
fn cr_708_9_a_face_down_permanent_is_revealed_as_it_leaves() {
    let mut g = main_phase();
    let quanar = g.add_card_to_battlefield(0, catalog::mischievous_quanar());
    g.battlefield.iter_mut().find(|c| c.id == quanar).unwrap().turn_face_down();
    let mut events = vec![];
    g.destroy_permanent(quanar, false, &mut events);
    let dead = g.players[0].graveyard.iter().find(|c| c.id == quanar).expect("in the graveyard");
    assert_eq!(dead.definition.name, "Mischievous Quanar");
    assert!(!dead.face_down, "revealed on the way out");
}

// ── CR 121 — Drawing a card ─────────────────────────────────────────────────

/// 121.6a — the replacement applies even though the library is empty.
#[test]
fn cr_121_6a_draw_replacement_applies_with_an_empty_library() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::silver_knight());
    }
    let thoughts = g.add_card_to_hand(0, catalog::parallel_thoughts());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: thoughts,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].library.is_empty(), "the pile ate the library");
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let mut events = vec![];
    assert!(g.draw_one(0, &mut events), "the replacement still applies");
    assert_eq!(g.players[0].hand.len(), 1);
}

// ── CR 614.9 — Redirection ──────────────────────────────────────────────────

/// A turn-scoped redirect isn't spent by the first event, and the destination
/// doesn't re-redirect (614.5 — one replacement per event).
#[test]
fn cr_614_9_turn_scoped_redirect_applies_to_every_damage_event() {
    let mut g = main_phase();
    let zealot = g.add_card_to_battlefield(0, catalog::karonas_zealot());
    let sink = g.add_card_to_battlefield(0, catalog::ageless_sentinels()); // 4/4
    g.battlefield.iter_mut().find(|c| c.id == zealot).unwrap().turn_face_down();
    mana(&mut g, 0);
    g.decider =
        Box::new(ScriptedDecider::new(vec![DecisionAnswer::Target(Target::Permanent(sink))]));
    g.perform_action(GameAction::TurnFaceUp { card_id: zealot }).expect("unmorph");
    drain_stack(&mut g);
    let mut events = vec![];
    g.deal_damage_to_from(EntityRef::Permanent(zealot), 1, None, &mut events);
    g.deal_damage_to_from(EntityRef::Permanent(zealot), 2, None, &mut events);
    assert_eq!(g.battlefield_find(zealot).unwrap().damage, 0);
    assert_eq!(g.battlefield_find(sink).unwrap().damage, 3, "both events landed on the sink");
}

// ── CR 613.7 — Indefinite keyword removal ───────────────────────────────────

/// A `Duration::Permanent` keyword loss survives the cleanup step that clears
/// the until-end-of-turn removals.
#[test]
fn cr_613_7_indefinite_keyword_removal_survives_cleanup() {
    let mut g = main_phase();
    let sentinels = g.add_card_to_battlefield(0, catalog::ageless_sentinels());
    let ctx = crabomination::game::effects::EffectContext::for_ability(sentinels, 0, None);
    g.resolve_effect(
        &crabomination::effect::Effect::LoseKeyword {
            what: crabomination::effect::Selector::This,
            keyword: Keyword::Defender,
            duration: crabomination::effect::Duration::Permanent,
        },
        &ctx,
    )
    .expect("resolve");
    assert!(!g.computed_permanent(sentinels).unwrap().keywords.contains(&Keyword::Defender));
    g.do_cleanup(&mut vec![]);
    assert!(
        !g.computed_permanent(sentinels).unwrap().keywords.contains(&Keyword::Defender),
        "the indefinite removal outlives cleanup"
    );
}

/// The until-end-of-turn sibling is cleared at cleanup.
#[test]
fn cr_613_7_end_of_turn_keyword_removal_is_cleared_at_cleanup() {
    let mut g = main_phase();
    let sentinels = g.add_card_to_battlefield(0, catalog::ageless_sentinels());
    let ctx = crabomination::game::effects::EffectContext::for_ability(sentinels, 0, None);
    g.resolve_effect(
        &crabomination::effect::Effect::LoseKeyword {
            what: crabomination::effect::Selector::This,
            keyword: Keyword::Flying,
            duration: crabomination::effect::Duration::EndOfTurn,
        },
        &ctx,
    )
    .expect("resolve");
    assert!(!g.computed_permanent(sentinels).unwrap().keywords.contains(&Keyword::Flying));
    g.do_cleanup(&mut vec![]);
    assert!(g.computed_permanent(sentinels).unwrap().keywords.contains(&Keyword::Flying));
}

