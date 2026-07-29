//! CR conformance for the modern_decks M15 run:
//! - CR 609.7b — a damage shield rechecks its source's properties, and a
//!   shield that prevents nothing isn't used up.
//! - CR 610.3b/3c — a linked "until this leaves" exile doesn't happen when
//!   the source already left, and returns under the card's *owner's* control.
//! - CR 402.2 — the cleanup discard-down to maximum hand size.

use crabomination::card::CounterType;
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
        x_value: None,
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
