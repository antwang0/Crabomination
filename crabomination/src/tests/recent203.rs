//! Functionality tests for `catalog::sets::decks::recent203`.

use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Valkyrie's Call returns a slain nontoken non-Angel with a counter, flying, and
/// the Angel type.
#[test]
fn valkyries_call_returns_as_angel() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::valkyries_call());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().damage = 2; // lethal on the 2/2
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let returned = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Grizzly Bears")
        .expect("bears returned");
    assert!(returned.counter_count(CounterType::PlusOnePlusOne) >= 1, "+1/+1 counter");
    let c = g.computed_permanent(returned.id).unwrap();
    assert!(c.keywords.contains(&Keyword::Flying), "gained flying");
    assert!(c.subtypes.creature_types.contains(&CreatureType::Angel), "became an Angel");
}

/// Infernal Vessel returns once with two counters as a Demon, then stays dead.
#[test]
fn infernal_vessel_returns_as_demon_once() {
    let mut g = two_player_game();
    let iv = g.add_card_to_battlefield(0, catalog::infernal_vessel());
    g.battlefield_find_mut(iv).unwrap().damage = 1; // lethal on the 2/1
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let back = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Infernal Vessel")
        .expect("returned");
    assert_eq!(back.counter_count(CounterType::PlusOnePlusOne), 2, "two +1/+1 counters");
    let back_id = back.id;
    assert!(
        g.computed_permanent(back_id).unwrap().subtypes.creature_types.contains(&CreatureType::Demon),
        "now a Demon"
    );
    // Kill the Demon copy — it must not loop back.
    let count_before = g.battlefield.iter().filter(|c| c.definition.name == "Infernal Vessel").count();
    g.battlefield_find_mut(back_id).unwrap().damage = 99;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let count_after = g.battlefield.iter().filter(|c| c.definition.name == "Infernal Vessel").count();
    assert_eq!(count_after, count_before - 1, "the Demon copy did not return");
}

/// Fiery Annihilation deals 5 and exiles the creature instead of letting it die.
#[test]
fn fiery_annihilation_exiles_the_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::fiery_annihilation());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Fiery Annihilation");
    drain_stack(&mut g);
    let _ = g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "creature left the battlefield");
    assert!(g.exile.iter().any(|c| c.id == victim), "exiled, not in graveyard");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == victim), "not in graveyard");
}

/// Violent Urge grants +1/+0 and first strike; with delirium it adds double strike.
#[test]
fn violent_urge_first_strike_and_delirium_double_strike() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Four card types in the graveyard → delirium active.
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
    g.add_card_to_graveyard(0, catalog::island()); // land
    g.add_card_to_graveyard(0, catalog::rite_of_the_dragoncaller()); // enchantment
    let spell = g.add_card_to_hand(0, catalog::violent_urge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Violent Urge");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.power, 3, "+1/+0");
    assert!(c.keywords.contains(&Keyword::FirstStrike), "first strike");
    assert!(c.keywords.contains(&Keyword::DoubleStrike), "delirium double strike");
}

/// Elenda scales with life above her controller's starting total.
#[test]
fn elenda_scales_with_life() {
    let mut g = two_player_game();
    let elenda = g.add_card_to_battlefield(0, catalog::elenda_saint_of_dusk());
    let start = g.players[0].life;
    // At starting life: base 4/4, no menace.
    let c = g.computed_permanent(elenda).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4));
    assert!(!c.keywords.contains(&Keyword::Menace));
    // One above: 5/5 with menace.
    g.players[0].life = start + 1;
    let c = g.computed_permanent(elenda).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5));
    assert!(c.keywords.contains(&Keyword::Menace));
    // Ten above: additional +5/+5 → 10/10.
    g.players[0].life = start + 10;
    let c = g.computed_permanent(elenda).unwrap();
    assert_eq!((c.power, c.toughness), (10, 10));
}

/// Quilled Greatwurm counters up on combat damage during your turn.
#[test]
fn quilled_greatwurm_counters_on_combat_damage() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let wurm = g.add_card_to_battlefield(0, catalog::quilled_greatwurm());
    g.fire_combat_damage_to_player_triggers(wurm, 1, 7);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(wurm).unwrap().counter_count(CounterType::PlusOnePlusOne),
        7,
        "seven +1/+1 counters"
    );
}
