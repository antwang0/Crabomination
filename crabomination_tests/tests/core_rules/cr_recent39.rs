//! CR conformance for the modern_decks M15 run:
//! - CR 609.7b — a damage shield rechecks its source's properties, and a
//!   shield that prevents nothing isn't used up.
//! - CR 610.3b/3c — a linked "until this leaves" exile doesn't happen when
//!   the source already left, and returns under the card's *owner's* control.
//! - CR 402.2 — the cleanup discard-down to maximum hand size.
//! - CR 609.2/609.3 — effects apply only to permanents; impossible effects do
//!   as much as possible.
//! - CR 703.4c/4d/4f — untap, draw, and Saga lore-counter turn-based actions.
//! - CR 708.2b/708.3/708.8 — face-down permanents can't re-flip down, and
//!   neither entering face down nor flipping up fires ETB abilities.

use crabomination::card::{CardInstance, CounterType};
use crabomination::catalog;
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// CR 609.7b — a color-restricted shield only soaks matching sources, and a
/// non-matching hit leaves the shield intact for a later matching one.
#[test]
fn cr_609_7b_shield_rechecks_source_color_and_survives_a_miss() {
    let mut g = main_phase();
    let avacyn = g.add_card_to_battlefield(0, catalog::avacyn_guardian_angel());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // AutoDecider names the first legal color (white).
    let white = g.add_card_to_battlefield(1, catalog::sungrace_pegasus());
    let green = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: avacyn,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let mut events = Vec::new();
    // A green source misses the shield entirely...
    g.deal_damage_to_from(EntityRef::Permanent(bear), 1, Some(green), &mut events);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1);
    // ...and the shield is still there for the white one.
    g.deal_damage_to_from(EntityRef::Permanent(bear), 1, Some(white), &mut events);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1, "white damage prevented");
}

/// CR 610.3b — the source left before its own trigger resolved, so the
/// object doesn't move (the return half could never happen).
#[test]
fn cr_610_3b_exile_doesnt_happen_when_the_source_already_left() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hunter = g.add_card_to_hand(0, catalog::fiend_hunter());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: hunter,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    // Resolve only the creature spell, leaving its ETB trigger on the stack.
    while g.stack.len() > 1 || g.battlefield_find(hunter).is_none() {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    assert_eq!(g.stack.len(), 1, "the ETB trigger is still waiting");
    // Kill the Hunter in response to its own trigger.
    g.destroy_permanent(hunter, false, &mut Vec::new());
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_some(), "victim never left the battlefield");
    assert!(g.exile.iter().all(|c| c.id != victim));
}

/// CR 610.3c — the exiled card returns under its OWNER's control, not the
/// exiler's.
#[test]
fn cr_610_3c_linked_exile_returns_to_its_owner() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hunter = g.add_card_to_battlefield(0, catalog::fiend_hunter());
    use crabomination::game::effects::EffectContext;
    let mut ctx =
        EffectContext::for_spell_with_source(hunter, "Fiend Hunter", 0, None, vec![], 0, 0, 0, 0);
    ctx.targets = vec![Target::Permanent(victim)];
    g.resolve_effect(&catalog::fiend_hunter().triggered_abilities[0].effect, &ctx)
        .expect("exile");
    assert!(g.exile.iter().any(|c| c.id == victim));
    g.destroy_permanent(hunter, false, &mut Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 1);
}

/// CR 402.2 — the active player discards down to their maximum hand size in
/// the cleanup step, and a no-maximum static skips it entirely.
#[test]
fn cr_402_2_cleanup_discards_down_to_maximum_hand_size() {
    let mut g = main_phase();
    for _ in 0..10 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    g.do_cleanup(&mut Vec::new());
    assert_eq!(g.players[0].hand.len(), 7);
    // Reliquary Tower-style "no maximum hand size" skips the discard.
    g.add_card_to_battlefield(0, catalog::reliquary_tower());
    for _ in 0..5 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    g.do_cleanup(&mut Vec::new());
    assert_eq!(g.players[0].hand.len(), 12);
}

/// CR 606.3 — one loyalty activation per planeswalker per turn, and the
/// per-turn budget resets with the turn.
#[test]
fn cr_606_3_loyalty_budget_resets_each_turn() {
    let mut g = main_phase();
    let jace = g.add_card_to_battlefield(0, catalog::jace_the_living_guildpact());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let plus = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: jace,
            ability_index: 0,
            target: None,
            x_value: None,
        })
    };
    plus(&mut g).expect("first");
    drain_stack(&mut g);
    assert!(plus(&mut g).is_err());
    assert!(g.players[0].activated_loyalty_this_turn);
    g.battlefield_find_mut(jace).unwrap().loyalty_uses_this_turn = 0;
    g.players[0].activated_loyalty_this_turn = false;
    plus(&mut g).expect("next turn");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 7);
}

// ── CR 609 — Effects ────────────────────────────────────────────────────────

/// CR 609.3 — an effect that asks for more than is available does only as much
/// as possible: "each player discards two cards" takes the one card a player
/// has and doesn't fail.
#[test]
fn cr_609_3_impossible_effect_does_as_much_as_possible() {
    let mut g = main_phase();
    g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        g.add_card_to_battlefield(0, catalog::grizzly_bears()),
        0,
        None,
    );
    g.resolve_effect(
        &crabomination::effect::Effect::Discard {
            who: crabomination::effect::Selector::Player(
                crabomination::effect::PlayerRef::EachPlayer,
            ),
            amount: crabomination::effect::Value::Const(2),
            random: false,
        },
        &ctx,
    )
    .expect("discard resolves");
    assert_eq!(g.players[0].hand.len(), 0, "discarded the one card it had");
    assert_eq!(g.players[1].hand.len(), 1, "discarded the full two");
}

/// CR 609.2 — effects apply only to permanents unless stated otherwise: a
/// "destroy all creatures" sweeper leaves creature *cards* in graveyards and
/// hands alone.
#[test]
fn cr_609_2_effects_apply_only_to_permanents() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let in_hand = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let wrath = g.add_card_to_hand(0, catalog::wrath_of_god());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: wrath,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the permanent died");
    assert!(g.players[0].hand.iter().any(|c| c.id == in_hand), "the hand card is untouched");
    assert_eq!(g.players[1].graveyard.len(), 1, "the graveyard card is untouched");
}

// ── CR 703 — Turn-Based Actions ─────────────────────────────────────────────

/// CR 703.4c/703.3 — the untap turn-based action happens as the untap step
/// begins, before any player gets priority, so an upkeep trigger already sees
/// an untapped board.
#[test]
fn cr_703_4c_untap_precedes_upkeep_priority() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    let _ = g.advance_step(Vec::new()); // → cleanup → seat 0's untap
    while g.step != TurnStep::Upkeep {
        let _ = g.advance_step(Vec::new());
    }
    assert_eq!(g.active_player_idx, 0);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped by the turn-based action");
}

/// CR 703.4d — the active player draws immediately as the draw step begins
/// (CR 103.7a's opening-hand skip is a one-shot, not a turn-based action).
#[test]
fn cr_703_4d_draw_step_draws_a_card() {
    let mut g = main_phase();
    g.skip_first_draw = false;
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    g.step = TurnStep::Upkeep;
    let _ = g.advance_step(Vec::new());
    assert_eq!(g.step, TurnStep::Draw);
    assert_eq!(g.players[0].hand.len(), before + 1);
}

/// CR 703.4f — a lore counter goes on each Saga the active player controls as
/// their precombat main phase begins (and the chapter ability then triggers).
#[test]
fn cr_703_4f_precombat_main_puts_a_lore_counter_on_each_saga() {
    let mut g = main_phase();
    let saga = g.add_card_to_battlefield(0, catalog::history_of_benalia());
    assert_eq!(g.battlefield_find(saga).unwrap().counter_count(CounterType::Lore), 0);
    g.step = TurnStep::Draw;
    let _ = g.advance_step(Vec::new());
    assert_eq!(g.step, TurnStep::PreCombatMain);
    assert_eq!(g.battlefield_find(saga).unwrap().counter_count(CounterType::Lore), 1);
}

// ── CR 708 — Face-Down Permanents ───────────────────────────────────────────

/// CR 708.2b — a face-down permanent can't be turned face down again; the
/// stashed real card survives a second attempt.
#[test]
fn cr_708_2b_face_down_permanent_cant_be_turned_face_down_again() {
    let mut g = main_phase();
    let id = g.add_card_to_battlefield(0, catalog::elder_gargaroth());
    let c = g.battlefield_find_mut(id).unwrap();
    c.turn_face_down();
    c.turn_face_down();
    assert_eq!(c.definition.name, "");
    c.turn_face_up();
    assert_eq!(c.definition.name, "Elder Gargaroth", "the real card is still there");
}

/// CR 708.3 / 708.8 — a card put onto the battlefield face down doesn't fire
/// its enters-the-battlefield ability, and turning it face up later doesn't
/// fire it either.
#[test]
fn cr_708_3_face_down_entry_and_face_up_flip_skip_etb_triggers() {
    let mut g = main_phase();
    let top = g.next_id();
    g.players[0]
        .library
        .insert(0, CardInstance::new(top, catalog::elvish_visionary(), 0));
    let hand_before = g.players[0].hand.len();
    let ctx = crabomination::game::effects::EffectContext::for_ability(top, 0, None);
    g.manifest_card(top, 0, &ctx, &mut Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "708.3 — no ETB draw on the way in");
    // A counter placed while face down survives the flip (708.8).
    g.battlefield_find_mut(top).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::TurnFaceUp { card_id: top }).expect("turn face up");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "708.8 — no ETB draw on the flip");
    let c = g.battlefield_find(top).unwrap();
    assert_eq!(c.definition.name, "Elvish Visionary");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "708.8 — the counter persists");
    assert_eq!((c.power(), c.toughness()), (2, 2), "1/1 plus the counter");
}
