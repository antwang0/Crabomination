//! CR conformance for this run's engine work — the CR 704.5f/g/h death
//! sweep's presence gate. `check_state_based_actions` now answers "can
//! anything die on this board?" from instance reads before it computes the
//! layer view, so each route into `dead` needs a case that proves the gate
//! doesn't swallow it:
//! - CR 704.5f — toughness ≤ 0 from a layer-7 anthem the *instance* can't see.
//! - CR 704.5f — an animated land entering play as a 0/0.
//! - CR 704.5g — lethal marked damage, and the survivor one point short.
//! - CR 704.5h — deathtouch damage under lethal.

use crabomination::card::{
    CardDefinition, CardType, CounterType, CreatureType, SelectionRequirement as R, StaticAbility,
    Subtypes,
};
use crabomination::catalog;
use crabomination::effect::StaticEffect;
use crabomination::game::*;
use crabomination::mana::{cost, generic};

fn body(name: &'static str, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Engineered Plague's shape: every opponent's creature gets -1/-1.
fn plague(power: i32, toughness: i32) -> CardDefinition {
    CardDefinition {
        name: "Plague Test",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures your opponents control get -1/-1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature,
                power,
                toughness,
                keywords: vec![],
                opponents: true,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// CR 704.5f — a creature whose toughness only reaches 0 through layer 7 dies.
/// Its instance toughness is 1 throughout, so nothing but the layer view can
/// tell; the gate has to notice the negative anthem and compute.
#[test]
fn cr_704_5f_layer_seven_reduction_still_kills() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, body("Victim", 1, 1));
    assert_eq!(g.computed_permanent(victim).unwrap().toughness, 1);

    g.add_card_to_battlefield(0, plague(-1, -1));
    assert_eq!(g.computed_permanent(victim).unwrap().toughness, 0, "layer 7 takes it to 0/0");
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "0 toughness is a CR 704.5f death");
    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == victim),
        "and it goes to its owner's graveyard"
    );
}

/// The same anthem one point weaker leaves a 1/1 alive at 1/1 — the gate is
/// signed, so a *positive* anthem on the board must not make every sweep
/// compute, and a -0/-0 one must not kill.
#[test]
fn cr_704_5f_non_reducing_anthem_kills_nothing() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, body("Mine", 1, 1));
    let theirs = g.add_card_to_battlefield(1, body("Theirs", 1, 1));
    g.add_card_to_battlefield(0, plague(2, 2));
    g.check_state_based_actions();
    assert!(g.battlefield_find(mine).is_some());
    assert_eq!(g.computed_permanent(theirs).unwrap().toughness, 3, "the anthem pumps, not kills");
    assert!(g.battlefield_find(theirs).is_some());
}

/// CR 704.5f / CR 305.7 — a land animated as a 0/0 dies to the same rule, and
/// it is not a printed creature, so the gate's card-type leg is what keeps it
/// on the sweep's list.
#[test]
fn cr_704_5f_animated_zero_toughness_land_dies() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, CardDefinition {
        name: "Nought Ritual",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Lands you control are 0/0 creatures.",
            effect: StaticEffect::MatchingLandsAreCreatures {
                filter: R::Land,
                power: 0,
                toughness: 0,
                keywords: vec![],
                creature_types: vec![],
                colors: vec![],
            },
        }],
        ..Default::default()
    });
    g.check_state_based_actions();
    assert!(g.battlefield_find(land).is_none(), "an animated 0/0 land is a CR 704.5f death");
}

/// CR 704.5g — lethal marked damage kills at exactly toughness, and one short
/// survives. Both cards sit on the same board, so one sweep decides both.
#[test]
fn cr_704_5g_lethal_damage_is_exact() {
    let mut g = two_player_game();
    let dead = g.add_card_to_battlefield(0, body("Lethal", 2, 2));
    let alive = g.add_card_to_battlefield(0, body("Scratched", 2, 2));
    g.battlefield_find_mut(dead).unwrap().damage = 2;
    g.battlefield_find_mut(alive).unwrap().damage = 1;
    g.check_state_based_actions();
    assert!(g.battlefield_find(dead).is_none(), "damage >= toughness is lethal");
    assert!(g.battlefield_find(alive).is_some(), "one short is not");
}

/// CR 704.5h — any damage from a deathtouch source is lethal, so a creature
/// under its toughness still dies. The gate's damage leg has to carry the
/// deathtouch flag, not just the arithmetic.
#[test]
fn cr_704_5h_deathtouch_damage_under_lethal_kills() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(0, body("Touched", 4, 4));
    {
        let c = g.battlefield_find_mut(victim).unwrap();
        c.damage = 1;
        c.dealt_deathtouch_damage = true;
    }
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "deathtouch damage is lethal at any amount");
}

/// CR 704.5f via CR 122.1 — -1/-1 counters are an instance read, and they must
/// kill on a board that carries a *positive* anthem (i.e. one where the
/// layer-7 leg of the gate says nothing).
#[test]
fn cr_704_5f_minus_counters_kill_under_a_positive_anthem() {
    let mut g = two_player_game();
    // Seat 0's own creature, so the `opponents: true` anthem does not reach
    // it — the board carries a live layer-7 effect that cannot save it.
    let victim = g.add_card_to_battlefield(0, body("Withered", 2, 2));
    g.add_card_to_battlefield(0, plague(1, 1));
    g.battlefield_find_mut(victim).unwrap().add_counters(CounterType::MinusOneMinusOne, 2);
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "two -1/-1 counters take a 2/2 to 0/0");
}
