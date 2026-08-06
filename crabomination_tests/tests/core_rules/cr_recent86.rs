//! CR conformance for this run:
//! - CR 708.4 — a face-down permanent is a nameless, costless, ability-less
//!   2/2 colorless creature, and turning it up restores everything.
//! - CR 613.7b/7d — a set-base-P/T continuous effect is applied before +1/+1
//!   counters, so the counter stacks on the new base.
//! - CR 606.5 — a minus loyalty ability can't be activated when the cost
//!   would drop loyalty below zero.
//! - CR 106.6 — spend-restricted mana funds only what its clause allows.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::{Color, SpellKind, SpendRestriction};

/// A face-down permanent loses its name, mana cost, types, and abilities and
/// is a 2/2 colorless creature; turning it face up restores all of them.
#[test]
fn cr_708_4_face_down_permanent_is_a_vanilla_two_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::exalted_angel());
    g.battlefield_find_mut(id).unwrap().turn_face_down();

    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "2/2 while face down");
    assert!(cp.keywords.is_empty(), "no abilities while face down");
    assert!(cp.colors.is_empty(), "colorless while face down");
    assert_eq!(cp.card_types, vec![CardType::Creature], "creature and nothing else");
    assert_eq!(g.battlefield_find(id).unwrap().definition.cost.cmc(), 0, "no mana cost");
    assert!(cp.subtypes.creature_types.is_empty(), "no creature types");

    g.battlefield_find_mut(id).unwrap().turn_face_up();
    let back = g.battlefield_find(id).unwrap();
    assert_eq!(back.definition.name, "Exalted Angel", "the real card is back");
    assert!(
        g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Flying),
        "and so are its abilities"
    );
}

/// Layer 7b sets base power/toughness; layer 7d applies counters on top, so a
/// shrunk creature carrying a +1/+1 counter is 2/2, not 5/5.
#[test]
fn cr_613_7b_counters_apply_after_a_base_pt_set() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.battlefield_find_mut(angel).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert_eq!(g.computed_permanent(angel).unwrap().power, 5, "4/4 plus a counter");

    let aura = g.add_card_to_battlefield(0, catalog::burden_of_proof());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(angel);
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "base 1/1 (7b) then the counter (7d)");
}

/// A planeswalker can't activate a minus ability that would take its loyalty
/// below zero.
#[test]
fn cr_606_5_minus_ability_needs_enough_loyalty() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::liliana_of_the_veil()); // starts at 3
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let minus_six = catalog::liliana_of_the_veil()
        .loyalty_abilities
        .iter()
        .position(|a| a.loyalty_cost <= -6)
        .expect("Liliana's ultimate");
    assert!(
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: pw,
            ability_index: minus_six,
            target: None,
            x_value: None,
        })
        .is_err(),
        "3 loyalty can't pay -6"
    );
    assert_eq!(
        g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty),
        3,
        "and the failed activation costs nothing"
    );
}

/// Spend-restricted mana funds only the payments its clause allows.
#[test]
fn cr_106_6_restricted_mana_only_funds_its_clause() {
    let creature_spell = SpellKind { creature: true, ..Default::default() };
    let face_down = SpellKind { face_down: true, ..Default::default() };
    let flip_up = SpellKind { turning_face_up: true, ..Default::default() };

    assert!(SpendRestriction::CreatureOnly.allows(&creature_spell));
    assert!(!SpendRestriction::CreatureOnly.allows(&face_down));
    assert!(SpendRestriction::FaceDownSpellsOrTurnFaceUp.allows(&face_down));
    assert!(SpendRestriction::FaceDownSpellsOrTurnFaceUp.allows(&flip_up));
    assert!(!SpendRestriction::FaceDownSpellsOrTurnFaceUp.allows(&creature_spell));

    // And the pool keeps it out of the freely-spendable total.
    let mut g = two_player_game();
    g.players[0].mana_pool.add_restricted(
        Color::Red,
        2,
        SpendRestriction::FaceDownSpellsOrTurnFaceUp,
    );
    assert_eq!(g.players[0].mana_pool.total(), 0);
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2);
}
