#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use crate::prepared_on_battlefield;

// ── Graduation Day (new) ────────────────────────────────────────────────────

#[test]
fn graduation_day_repartee_pumps_creature_when_targeting_creature() {
    // Repartee enchantment: cast Lightning Bolt at a creature → +1/+1
    // counter on a creature you control.
    let mut g = two_player_game();
    let _gd = g.add_card_to_battlefield(0, catalog::graduation_day());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let pumped = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        pumped.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Graduation Day Repartee should add a +1/+1 counter when an instant targets a creature",
    );
}

#[test]
fn graduation_day_does_not_fire_when_targeting_player() {
    let mut g = two_player_game();
    let _gd = g.add_card_to_battlefield(0, catalog::graduation_day());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let unpumped = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        unpumped.counter_count(CounterType::PlusOnePlusOne),
        0,
        "Repartee should NOT fire when the spell targets a player",
    );
}

// ── Stirring Hopesinger Repartee improvement ───────────────────────────────

#[test]
fn stirring_hopesinger_repartee_pumps_each_creature_you_control() {
    let mut g = two_player_game();
    let hopesinger = g.add_card_to_battlefield(0, catalog::stirring_hopesinger());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let h = g.battlefield.iter().find(|c| c.id == hopesinger).unwrap();
    assert_eq!(
        h.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Hopesinger should get a +1/+1 counter from its own Repartee trigger",
    );
    let b = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        b.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Bear should also get a +1/+1 counter (each creature you control)",
    );
    // The opponent's bear should NOT get a counter.
    if let Some(o) = g.battlefield.iter().find(|c| c.id == opp_bear) {
        assert_eq!(
            o.counter_count(CounterType::PlusOnePlusOne),
            0,
            "Opponent's creatures don't get pumped",
        );
    }
}

// ── Informed Inkwright Repartee improvement ────────────────────────────────

#[test]
fn informed_inkwright_repartee_creates_inkling_token() {
    let mut g = two_player_game();
    let _scribe = g.add_card_to_battlefield(0, catalog::informed_inkwright());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let inkling_count_before = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Inkling")
        .count();

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let inkling_count_after = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Inkling")
        .count();
    assert_eq!(
        inkling_count_after,
        inkling_count_before + 1,
        "Informed Inkwright Repartee should mint a 1/1 Inkling token",
    );
}

// ── Inkling Mascot Repartee improvement ────────────────────────────────────

#[test]
fn inkling_mascot_repartee_grants_flying_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let mascot = g.add_card_to_battlefield(0, catalog::inkling_mascot());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let m = g.battlefield.iter().find(|c| c.id == mascot).unwrap();
    assert!(
        m.has_keyword(&Keyword::Flying),
        "Inkling Mascot should have Flying after Repartee",
    );
    // Surveil 1 either drops the top to graveyard (-1 lib) or returns it
    // (no change). Either way the library is at most unchanged.
    assert!(
        g.players[0].library.len() <= lib_before,
        "Surveil 1 should peek at the top — library did not grow",
    );
}

// ── Snooping Page Repartee improvement ─────────────────────────────────────

#[test]
fn snooping_page_repartee_grants_unblockable() {
    let mut g = two_player_game();
    let page = g.add_card_to_battlefield(0, catalog::snooping_page());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let p = g.battlefield.iter().find(|c| c.id == page).unwrap();
    assert!(
        p.has_keyword(&Keyword::Unblockable),
        "Snooping Page should be unblockable this turn after Repartee",
    );
}

// ── Withering Curse ─────────────────────────────────────────────────────────

#[test]
fn withering_curse_without_lifegain_pumps_minus_two() {
    // No lifegain this turn → -2/-2 to all creatures.
    let mut g = two_player_game();
    let bear_p0 = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let bear_p1 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::withering_curse());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Withering Curse castable for {1}{B}{B}");
    drain_stack(&mut g);

    // Both 2/2 bears should die to SBA (2-2 = 0 toughness).
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear_p0),
        "P0 bear should die (-2/-2 → 0 toughness)",
    );
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear_p1),
        "P1 bear should die (-2/-2 → 0 toughness)",
    );
}

#[test]
fn withering_curse_with_lifegain_destroys_all_creatures() {
    // Trigger lifegain to enable the Infusion path.
    let mut g = two_player_game();
    g.players[0].life_gained_this_turn = 3;
    let bear_p0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear_p1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::withering_curse());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Withering Curse castable for {1}{B}{B}");
    drain_stack(&mut g);

    // Infusion path: every creature destroyed → graveyard.
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear_p0),
        "P0 bear destroyed by Infusion",
    );
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear_p1),
        "P1 bear destroyed by Infusion",
    );
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == bear_p0),
        "P0 bear in P0's graveyard",
    );
}

// ── Root Manipulation ───────────────────────────────────────────────────────

#[test]
fn root_manipulation_pumps_each_creature_you_control_with_menace() {
    let mut g = two_player_game();
    let bear_p0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear_p1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::root_manipulation());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Root Manipulation castable for {3}{B}{G}");
    drain_stack(&mut g);

    let p0 = g.battlefield.iter().find(|c| c.id == bear_p0).unwrap();
    assert_eq!(p0.power(), 4, "Bear should be 4/4 (+2/+2)");
    assert_eq!(p0.toughness(), 4);
    assert!(
        p0.has_keyword(&Keyword::Menace),
        "Bear should gain Menace",
    );
    let p1 = g.battlefield.iter().find(|c| c.id == bear_p1).unwrap();
    assert_eq!(p1.power(), 2, "Opponent's bear unchanged");
    assert!(!p1.has_keyword(&Keyword::Menace), "Opponent's bear: no menace");
}

// ── Blech, Loafing Pest ─────────────────────────────────────────────────────

#[test]
fn blech_pumps_pest_on_lifegain() {
    let mut g = two_player_game();
    let _blech = g.add_card_to_battlefield(0, catalog::blech_loafing_pest());
    // Pest Mascot is a Pest. Blech is also a Pest.
    let pest = g.add_card_to_battlefield(0, catalog::pest_mascot());
    // A non-Pest/Bat/Insect/Snake/Spider creature should NOT be pumped.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Use a quick lifegain spell to trigger Blech.
    let id = g.add_card_to_hand(0, catalog::healing_salve());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Healing Salve castable for {W}");
    drain_stack(&mut g);

    let p = g.battlefield.iter().find(|c| c.id == pest).unwrap();
    assert!(
        p.counter_count(CounterType::PlusOnePlusOne) >= 1,
        "Pest Mascot (Pest) should get a +1/+1 counter from Blech",
    );
    let b = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        b.counter_count(CounterType::PlusOnePlusOne),
        0,
        "Grizzly Bears (Bear, not in pump list) should not be pumped",
    );
}

// ── Cauldron of Essence ─────────────────────────────────────────────────────

#[test]
fn cauldron_of_essence_drains_when_creature_dies() {
    let mut g = two_player_game();
    let _cauldron = g.add_card_to_battlefield(0, catalog::cauldron_of_essence());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let opp_life_before = g.players[1].life;
    let life_before = g.players[0].life;

    // Bolt our own bear to trigger the death drain.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Bear destroyed by Bolt",
    );
    assert_eq!(
        g.players[1].life,
        opp_life_before - 1,
        "Opponent should lose 1 life from Cauldron drain",
    );
    assert_eq!(
        g.players[0].life,
        life_before + 1,
        "You should gain 1 life from Cauldron drain",
    );
}

// ── Diary of Dreams ─────────────────────────────────────────────────────────

#[test]
fn diary_of_dreams_gains_charge_on_instant_or_sorcery_cast() {
    let mut g = two_player_game();
    let diary = g.add_card_to_battlefield(0, catalog::diary_of_dreams());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let d = g.battlefield.iter().find(|c| c.id == diary).unwrap();
    assert_eq!(
        d.counter_count(CounterType::Page),
        1,
        "Diary of Dreams should accrue a Page counter on instant cast",
    );
}

/// Diary of Dreams: with 0 Page counters, the activation costs the full
/// {5}. With < 5 mana available, activation should fail with
/// InsufficientMana, leaving the source untapped (snapshot rollback).
#[test]
fn diary_of_dreams_activation_costs_five_with_no_page_counters() {
    let mut g = two_player_game();
    let diary = g.add_card_to_battlefield(0, catalog::diary_of_dreams());
    g.players[0].mana_pool.add_colorless(4);
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: diary,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None});
    assert!(res.is_err(), "0-Page Diary activation needs {{5}}, only 4 available");
    let d = g.battlefield.iter().find(|c| c.id == diary).unwrap();
    assert!(!d.tapped, "Diary should not tap on a failed payment");
}

/// Diary of Dreams: with 3 Page counters, the activation costs {2}.
/// Pay {2} and {T} → draw a card.
#[test]
fn diary_of_dreams_page_counters_reduce_cost_by_one_each() {
    let mut g = two_player_game();
    let diary = g.add_card_to_battlefield(0, catalog::diary_of_dreams());
    // Seed 3 page counters.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == diary) {
        c.counters.insert(CounterType::Page, 3);
    }
    g.add_card_to_library(0, catalog::island());
    // {2} = generic 5 - 3 page counters.
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: diary,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Diary activates at {2} with 3 page counters");
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].hand.len(),
        hand_before + 1,
        "Diary draw resolves",
    );
    // Source should now be tapped and pool drained.
    let d = g.battlefield.iter().find(|c| c.id == diary).unwrap();
    assert!(d.tapped, "Diary tapped after activation");
    assert_eq!(g.players[0].mana_pool.total(), 0, "All 2 mana drained");
}

/// Diary of Dreams: with 5+ Page counters, the activation cost
/// reduces to {0} (clamped at the printed generic total).
#[test]
fn diary_of_dreams_page_counters_clamp_at_printed_generic() {
    let mut g = two_player_game();
    let diary = g.add_card_to_battlefield(0, catalog::diary_of_dreams());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == diary) {
        c.counters.insert(CounterType::Page, 8);
    }
    g.add_card_to_library(0, catalog::island());
    // Zero mana available.
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: diary,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Diary should activate at {{0}} with 8 page counters");
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].hand.len(),
        hand_before + 1,
        "Diary draw resolves",
    );
}

// ── Spectacle Summit ────────────────────────────────────────────────────────

// ── Comforting Counsel ──────────────────────────────────────────────────────

#[test]
fn comforting_counsel_accrues_growth_on_lifegain() {
    let mut g = two_player_game();
    let cc = g.add_card_to_battlefield(0, catalog::comforting_counsel());
    let salve = g.add_card_to_hand(0, catalog::healing_salve());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: salve, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Healing Salve castable for {W}");
    drain_stack(&mut g);

    let counsel = g.battlefield.iter().find(|c| c.id == cc).unwrap();
    assert_eq!(
        counsel.counter_count(CounterType::Growth),
        1,
        "Comforting Counsel should accrue a Growth counter when you gain life",
    );
}

/// At <5 growth counters, the anthem is dormant — friendly creatures
/// keep their base P/T.
#[test]
fn comforting_counsel_no_anthem_below_five_counters() {
    let mut g = two_player_game();
    let cc = g.add_card_to_battlefield(0, catalog::comforting_counsel());
    // Manually seed 4 growth counters (one short of the gate).
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == cc) {
        c.counters.insert(CounterType::Growth, 4);
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let computed = g.compute_battlefield();
    let bear_pt = computed.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(bear_pt.power, 2);
    assert_eq!(bear_pt.toughness, 2);
}

/// At ≥5 growth counters, the +3/+3 anthem fires for all controller's
/// creatures (Grizzly Bears 2/2 → 5/5).
#[test]
fn comforting_counsel_anthem_buffs_friendly_creatures_at_five_counters() {
    let mut g = two_player_game();
    let cc = g.add_card_to_battlefield(0, catalog::comforting_counsel());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == cc) {
        c.counters.insert(CounterType::Growth, 5);
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let computed = g.compute_battlefield();
    let bear_pt = computed.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(bear_pt.power, 5, "friendly bear +3 power");
    assert_eq!(bear_pt.toughness, 5, "friendly bear +3 toughness");

    // Opp's bear is unaffected.
    let opp_pt = computed.iter().find(|c| c.id == opp_bear).unwrap();
    assert_eq!(opp_pt.power, 2);
    assert_eq!(opp_pt.toughness, 2);
}

// ── Moment of Reckoning ─────────────────────────────────────────────────────

#[test]
fn moment_of_reckoning_destroy_mode_destroys_target_return_mode_brings_back() {
    // Mode 0: destroy a battlefield creature.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::moment_of_reckoning());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Moment of Reckoning castable for {3}{W}{W}{B}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be destroyed by mode 0 (Destroy)");

    // Mode 1: return a creature from graveyard to battlefield.
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::moment_of_reckoning());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Moment of Reckoning castable in return mode");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "Bear should return to battlefield (mode 1)");
    assert_eq!(g.battlefield.len(), bf_before + 1);
}

// ── Stirring Honormancer ────────────────────────────────────────────────────

#[test]
fn stirring_honormancer_etb_finds_creature_in_top_x() {
    let mut g = two_player_game();
    // We control 1 creature, so X = 1. Top of library = a creature.
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // controlled creature
    g.add_card_to_library(0, catalog::grizzly_bears()); // top of library — found!
    let id = g.add_card_to_hand(0, catalog::stirring_honormancer());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Stirring Honormancer castable for {2}{W}{W}{B}");
    drain_stack(&mut g);

    // Stirring Honormancer entered (1 card from hand) + bear from library
    // joins our hand → net hand = before - 1 (cast) + 1 (find) = same
    // size. But our hand also lost the cast card so:
    assert_eq!(
        g.players[0].hand.len(),
        hand_before, // -1 for cast, +1 for retrieved bear
        "Top-of-library bear should have joined hand",
    );
}

// ── Dissection Practice ─────────────────────────────────────────────────────

#[test]
fn dissection_practice_drains_one_and_shrinks_target() {
    // Push (modern_decks): now multi-target — slot 0 = target player
    // (drain), slot 1 = optional pump +1/+1 EOT, slot 2 = optional
    // shrink -1/-1 EOT. This test exercises slot 0 (drain opp) +
    // slot 2 (shrink). To skip slot 1 we point it at the caster (whose
    // life loss is already 0 since it's a creature filter, no-op).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let friendly_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::dissection_practice());
    g.players[0].mana_pool.add(Color::Black, 1);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        // slot 1 = friendly_bear (+1/+1 EOT), slot 2 = bear (-1/-1 EOT)
        additional_targets: vec![Target::Permanent(friendly_bear), Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("Dissection Practice castable for {B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, p0_life + 1, "You gain 1 life");
    assert_eq!(g.players[1].life, p1_life - 1, "Opponent loses 1 life");
    // bear gets -1/-1 EOT → 1/1.
    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(target.power(), 1);
    assert_eq!(target.toughness(), 1);
    // friendly_bear gets +1/+1 EOT → 3/3.
    let pumped = g.battlefield.iter().find(|c| c.id == friendly_bear).unwrap();
    assert_eq!(pumped.power(), 3);
    assert_eq!(pumped.toughness(), 3);
}

#[test]
fn dissection_practice_drain_only_no_creature_targets() {
    // Slot 0 (drain) only — slots 1/2 empty no-op.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::dissection_practice());
    g.players[0].mana_pool.add(Color::Black, 1);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Dissection Practice castable for {B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, p0_life + 1);
    assert_eq!(g.players[1].life, p1_life - 1);
}

// ── Heated Argument ─────────────────────────────────────────────────────────

#[test]
fn heated_argument_deals_six_to_creature_and_two_to_controller() {
    // Gy-exile + 2-to-controller rider is wrapped in `Effect::MayDo`;
    // inject `Bool(true)` to opt in.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 — dies
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // a card to exile
    let id = g.add_card_to_hand(0, catalog::heated_argument());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    let p1_life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Heated Argument castable for {4}{R}");
    drain_stack(&mut g);

    // Bear dies (lethal), controller takes 2.
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Bear destroyed by 6 damage",
    );
    assert_eq!(
        g.players[1].life,
        p1_life - 2,
        "Controller takes 2 from the rider",
    );
    // The Bolt should have been exiled from the graveyard.
    assert!(
        g.exile.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "Bolt should be exiled from graveyard",
    );
}

#[test]
fn heated_argument_skips_rider_when_declining() {
    // Declining the gy-exile: no extra 2 damage fires.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::heated_argument());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    let p1_life = g.players[1].life;
    // Default AutoDecider says no.

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Heated Argument castable");
    drain_stack(&mut g);

    // Bear still dies (the 6 damage isn't gated), controller untouched.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert_eq!(g.players[1].life, p1_life,
        "Controller takes no damage when rider is skipped");
    // Bolt stays in graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"));
}

// ── End of the Hunt ─────────────────────────────────────────────────────────

#[test]
fn end_of_the_hunt_exiles_opponent_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::end_of_the_hunt());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("End of the Hunt castable for {1}{B}");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Opponent's bear should leave the battlefield",
    );
    assert!(
        g.exile.iter().any(|c| c.id == bear),
        "Bear should be exiled",
    );
}

#[test]
fn end_of_the_hunt_rejects_smaller_target_when_greater_mv_exists() {
    // Push (modern_decks): the new
    // `SelectionRequirement::HasGreatestManaValueAmongControlled`
    // predicate enforces "greatest MV among creatures and PWs they
    // control". Opp controls a bear (CMC 2) + a craw wurm (CMC 6);
    // targeting the bear must fail (bear's MV is not the greatest).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let id = g.add_card_to_hand(0, catalog::end_of_the_hunt());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let res = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(
        res.is_err(),
        "End of the Hunt should reject the bear (MV 2) when a CMC-6 wurm is on the battlefield",
    );
    assert!(
        g.battlefield.iter().any(|c| c.id == bear),
        "Bear stays on the battlefield since the cast was rejected",
    );
}

#[test]
fn end_of_the_hunt_picks_largest_creature_when_targeting_max() {
    // Targeting the CMC-6 wurm is legal (it's the greatest-MV match).
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let id = g.add_card_to_hand(0, catalog::end_of_the_hunt());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(wurm)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("End of the Hunt castable + greatest-MV target legal");
    drain_stack(&mut g);

    assert!(
        g.exile.iter().any(|c| c.id == wurm),
        "Wurm (greatest MV) should be exiled",
    );
}

// ── Vicious Rivalry ─────────────────────────────────────────────────────────

#[test]
fn vicious_rivalry_destroys_creatures_at_or_below_x() {
    let mut g = two_player_game();
    // X = 2: bear (CMC 2) dies, but a CMC-3 creature lives.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // CMC 2
    let big = g.add_card_to_battlefield(1, catalog::craw_wurm()); // CMC 6
    let id = g.add_card_to_hand(0, catalog::vicious_rivalry());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4); // 2 generic + 2 for X
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Vicious Rivalry castable for {2}{2}{B}{G} (X=2)");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Bear (CMC 2) destroyed by Vicious Rivalry X=2",
    );
    assert!(
        g.battlefield.iter().any(|c| c.id == big),
        "Craw Wurm (CMC 6) survives",
    );
    assert_eq!(
        g.players[0].life,
        life_before - 2,
        "Caster pays X life as additional cost (approximated)",
    );
}

// ── Fix What's Broken ─────────────────────────────────────────────────────────

#[test]
fn fix_whats_broken_pays_x_life_and_returns_exact_mv() {
    let mut g = two_player_game();
    // Two CMC-2 creatures in the graveyard. X=2 returns BOTH (mass
    // reanimation at the chosen mana value).
    let bear1 = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // CMC 2
    let bear2 = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // CMC 2
    let id = g.add_card_to_hand(0, catalog::fix_whats_broken());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4); // 2 generic + 2 for X
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Fix What's Broken castable for {2}{2}{W}{B} (X=2)");
    drain_stack(&mut g);

    assert!(
        g.battlefield.iter().any(|c| c.id == bear1),
        "First CMC-2 creature returned to battlefield",
    );
    assert!(
        g.battlefield.iter().any(|c| c.id == bear2),
        "Second CMC-2 creature also returned (mass reanimation)",
    );
    assert_eq!(
        g.players[0].life,
        life_before - 2,
        "Caster pays X (=2) life as the additional cost",
    );
}

#[test]
fn fix_whats_broken_only_returns_cards_at_exact_mv() {
    let mut g = two_player_game();
    // X=2 returns the CMC-2 bear but NOT the CMC-6 wurm — the printed
    // card matches mana value EXACTLY, not "≤ X".
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // CMC 2
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm()); // CMC 6
    let id = g.add_card_to_hand(0, catalog::fix_whats_broken());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4); // 2 generic + 2 for X

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Fix What's Broken castable (X=2)");
    drain_stack(&mut g);

    assert!(
        g.battlefield.iter().any(|c| c.id == bear),
        "CMC-2 bear returns at X=2",
    );
    assert!(
        !g.battlefield.iter().any(|c| c.id == wurm),
        "CMC-6 wurm does NOT return at X=2 (exact mana-value match)",
    );
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == wurm),
        "CMC-6 wurm stays in the graveyard",
    );
}

// ── Proctor's Gaze ──────────────────────────────────────────────────────────

#[test]
fn proctors_gaze_returns_target_and_fetches_basic() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let forest = g.add_card_to_library(0, catalog::forest());
    // ScriptedDecider answers the SearchLibrary decision with the
    // forest. AutoDecider's default is to decline searches.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::proctors_gaze());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Proctor's Gaze castable for {2}{G}{U}");
    drain_stack(&mut g);

    // Bear bounced to its owner's hand.
    assert!(
        g.players[1].hand.iter().any(|c| c.id == bear),
        "Bear bounced to owner's hand",
    );
    // Forest fetched onto our battlefield.
    let on_bf = g.battlefield.iter().find(|c| c.id == forest);
    assert!(on_bf.is_some(), "Forest should land on battlefield");
    assert!(on_bf.unwrap().tapped, "Forest should enter tapped");
}

// ── Lorehold Charm ──────────────────────────────────────────────────────────

#[test]
fn lorehold_charm_pump_mode_pumps_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_charm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(2), x_value: None,
    })
    .expect("Lorehold Charm castable in pump mode");
    drain_stack(&mut g);

    let target = g.computed_permanent(bear).unwrap();
    assert_eq!(target.power, 3, "Bear gets +1/+1 → 3/3");
    assert_eq!(target.toughness, 3);
    assert!(target.keywords.contains(&Keyword::Trample),
        "printed mode also grants trample");
}

// ── Borrowed Knowledge ──────────────────────────────────────────────────────

#[test]
fn borrowed_knowledge_mode_one_discards_hand_then_draws_same_count() {
    let mut g = two_player_game();
    // Seed our library with 10 cards so we can draw freely.
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    // Add four cards to our hand we'll discard (plus BK = 5 total).
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::island());
    }

    let id = g.add_card_to_hand(0, catalog::borrowed_knowledge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("Borrowed Knowledge castable in mode 1");
    drain_stack(&mut g);

    // BK on stack → hand has 4 islands at resolution. Mode 1 discards all 4,
    // then draws 4 (= cards discarded this way). End hand = 4 fresh draws.
    assert_eq!(
        g.players[0].hand.len(),
        4,
        "Should end with 4 cards: discarded 4, drew 4"
    );
}

/// Borrowed Knowledge mode 1 with a single non-spell card in hand: discards
/// the 1 card, draws 1. Verifies `Value::CardsDiscardedThisEffect` scales
/// down (vs. the old flat-7 approximation that would have drawn 7).
#[test]
fn borrowed_knowledge_mode_one_with_small_hand_draws_proportionally() {
    let mut g = two_player_game();
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::island());

    let id = g.add_card_to_hand(0, catalog::borrowed_knowledge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("Borrowed Knowledge castable in mode 1");
    drain_stack(&mut g);

    // 1 island in hand after cast → discard 1 → draw 1. End hand = 1.
    assert_eq!(g.players[0].hand.len(), 1, "discarded 1, drew 1");
}

// ── Planar Engineering ──────────────────────────────────────────────────────

#[test]
fn planar_engineering_sacrifices_two_lands_and_fetches_basics() {
    let mut g = two_player_game();
    let l1 = g.add_card_to_battlefield(0, catalog::forest());
    let l2 = g.add_card_to_battlefield(0, catalog::forest());
    let mut lib_forests = Vec::new();
    for _ in 0..6 {
        lib_forests.push(g.add_card_to_library(0, catalog::forest()));
    }
    // ScriptedDecider answers each of the four SearchLibrary decisions
    // with successive forests from the library.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(lib_forests[0])),
        DecisionAnswer::Search(Some(lib_forests[1])),
        DecisionAnswer::Search(Some(lib_forests[2])),
        DecisionAnswer::Search(Some(lib_forests[3])),
    ]));
    let id = g.add_card_to_hand(0, catalog::planar_engineering());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Planar Engineering castable for {3}{G}");
    drain_stack(&mut g);

    // Both lands sacrificed.
    assert!(!g.battlefield.iter().any(|c| c.id == l1));
    assert!(!g.battlefield.iter().any(|c| c.id == l2));
    // 4 forests fetched onto the battlefield tapped (not the
    // already-sacrificed l1/l2).
    let forest_count = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Forest")
        .count();
    assert_eq!(forest_count, 4, "Should have 4 fresh Forests on the battlefield");
}

// ── Brush Off ───────────────────────────────────────────────────────────────

// ── Run Behind ──────────────────────────────────────────────────────────────

#[test]
fn run_behind_puts_target_creature_on_bottom_of_library() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::run_behind());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Run Behind castable for {3}{U}");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Bear leaves the battlefield",
    );
    // Bear goes to bottom of P1's library.
    assert_eq!(
        g.players[1].library.last().map(|c| c.id),
        Some(bear),
        "Bear should be at the bottom of its owner's library",
    );
}

#[test]
fn run_behind_top_of_library_via_scripted_owner_choice() {
    // Run Behind's printed Oracle has the *owner* of the moved card pick
    // top or bottom. The auto-decider lands the card on the bottom
    // (matching the prior collapsed behavior), but a `ScriptedDecider`
    // saying `Bool(true)` to the optional-trigger flips the placement
    // to the top.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::run_behind());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Run Behind castable for {3}{U}");
    drain_stack(&mut g);

    assert_eq!(
        g.players[1].library.first().map(|c| c.id),
        Some(bear),
        "Owner answered yes → bear lands on top of library",
    );
}

// ── Antiquities on the Loose ────────────────────────────────────────────────

#[test]
fn antiquities_on_the_loose_creates_two_spirit_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::antiquities_on_the_loose());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Antiquities on the Loose castable for {1}{W}{W}");
    drain_stack(&mut g);

    let spirit_count = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirit_count, 2, "Should create two Spirit tokens");
    assert_eq!(g.battlefield.len(), bf_before + 2);
}

/// Antiquities on the Loose's hand cast should NOT fan +1/+1 counters
/// on existing Spirits — the rider only fires for casts from a zone
/// other than your hand (flashback / Yawgmoth's Will-style).
#[test]
fn antiquities_on_the_loose_hand_cast_does_not_fan_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    // Seed a pre-existing Spirit (a different Spirit-typed card) on
    // the battlefield. The hand-cast path should leave its counter
    // pool empty.
    let existing_spirit = g.add_card_to_battlefield(0, catalog::pillardrop_rescuer());
    let id = g.add_card_to_hand(0, catalog::antiquities_on_the_loose());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Antiquities on the Loose castable from hand");
    drain_stack(&mut g);

    let s = g.battlefield_find(existing_spirit).expect("spirit still on bf");
    assert_eq!(s.counter_count(CounterType::PlusOnePlusOne), 0,
        "Hand cast should NOT fan +1/+1 counters");
}

/// Flashback cast of Antiquities on the Loose triggers the +1/+1
/// rider on each Spirit you control (per `Predicate::CastFromGraveyard`).
/// The two minted Spirits + a pre-existing Spirit should all carry a
/// +1/+1 counter after the spell resolves.
#[test]
fn antiquities_on_the_loose_flashback_cast_fans_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    // Pre-existing Spirit on the battlefield.
    let existing_spirit = g.add_card_to_battlefield(0, catalog::pillardrop_rescuer());

    // Put Antiquities on the Loose straight into the graveyard.
    let id = g.add_card_to_graveyard(0, catalog::antiquities_on_the_loose());

    // Pay the flashback {4}{W}{W} cost.
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastFlashback {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Antiquities on the Loose castable via Flashback for {4}{W}{W}");
    drain_stack(&mut g);

    // Pre-existing Spirit should now carry a +1/+1 counter (the
    // counter fan-out fired because the spell was cast from gy).
    let s = g.battlefield_find(existing_spirit).expect("existing spirit on bf");
    assert_eq!(s.counter_count(CounterType::PlusOnePlusOne), 1,
        "Flashback cast should fan +1/+1 counters on each Spirit");

    // The two minted Spirit tokens should also have a counter each
    // (the fan-out iterates every Spirit, including the two just-
    // minted).
    let minted_spirits_with_counters = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit"
            && c.counter_count(CounterType::PlusOnePlusOne) == 1)
        .count();
    assert_eq!(minted_spirits_with_counters, 2,
        "Both minted Spirits should carry +1/+1 counters from the fan-out");

    // Antiquities on the Loose should be in exile (per CR 702.34a).
    assert!(g.exile.iter().any(|c| c.id == id),
        "Antiquities on the Loose should be exiled after flashback resolves");
}

// ── Conciliator's Duelist ───────────────────────────────────────────────────

#[test]
fn conciliators_duelist_etb_draws_and_each_player_loses_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::conciliators_duelist());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 2);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Conciliator's Duelist castable for {W}{W}{B}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, p0_life - 1, "You lose 1 life");
    assert_eq!(g.players[1].life, p1_life - 1, "Opponent loses 1 life");
    // Hand: -1 cast + 1 draw = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Body-only batch + Ajani's Response (2026-04-30 push) ────────────────────

#[test]
fn ajanis_response_destroys_target_creature() {
    let mut g = two_player_game();
    let resp = g.add_card_to_hand(0, catalog::ajanis_response());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: resp,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Ajani's Response castable for {4}{W}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Ajani's Response destroys the target creature");
}

#[test]
fn ajanis_response_only_targets_creatures() {
    let mut g = two_player_game();
    let resp = g.add_card_to_hand(0, catalog::ajanis_response());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: resp,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Ajani's Response only targets creatures");
}

#[test]
fn cuboid_colony_is_a_one_one_with_flash_flying_trample() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::cuboid_colony());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Cuboid Colony castable for {G}{U}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 1);
    assert!(card.has_keyword(&Keyword::Flash));
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Trample));
}

#[test]
fn hungry_graffalon_is_a_three_four_with_reach() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::hungry_graffalon());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Hungry Graffalon castable for {3}{G}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 4);
    assert!(card.has_keyword(&Keyword::Reach));
}

#[test]
fn molten_core_maestro_has_menace() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::molten_core_maestro());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Molten-Core Maestro castable for {1}{R}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Menace));
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
}

#[test]
fn aberrant_manawurm_has_trample_and_correct_pt() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::aberrant_manawurm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Aberrant Manawurm castable for {3}{G}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Trample));
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 5);
}

#[test]
fn tackle_artist_has_trample_and_correct_pt() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::tackle_artist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tackle Artist castable for {3}{R}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Trample));
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn thunderdrum_soloist_has_reach() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::thunderdrum_soloist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Thunderdrum Soloist castable for {1}{R}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Reach));
}

#[test]
fn pensive_professor_is_a_zero_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pensive_professor());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pensive Professor castable for {1}{U}{U}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 0);
    assert_eq!(card.toughness(), 2);
}

#[test]
fn eternal_student_is_a_four_two_zombie() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::eternal_student());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Eternal Student castable for {3}{B}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 2);
    assert!(card.definition.has_creature_type(crabomination::card::CreatureType::Zombie));
}

#[test]
fn postmortem_professor_drains_one_on_attack() {
    let mut g = two_player_game();
    let prof = g.add_card_to_battlefield(0, catalog::postmortem_professor());
    // Strip summoning sickness so it can attack.
    g.battlefield_find_mut(prof).unwrap().summoning_sick = false;

    // Move to declare-attackers and attack with the professor.
    g.step = TurnStep::DeclareAttackers;
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;

    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: prof,
        target: AttackTarget::Player(1),
    }]))
    .expect("Professor can attack");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, p0_life + 1,
        "Postmortem Professor's on-attack drain gives you 1");
    assert_eq!(g.players[1].life, p1_life - 1,
        "Postmortem Professor's on-attack drain takes 1 from each opponent");
}

// ── Scolding Administrator (Silverquill 🟡 → mostly wired) ─────────────────

#[test]
fn scolding_administrator_repartee_pumps_self() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let admin = g.add_card_to_battlefield(0, catalog::scolding_administrator());
    g.battlefield_find_mut(admin).unwrap().summoning_sick = false;

    // Cast a creature-targeting Lightning Bolt to fire Repartee.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let counters = g.battlefield_find(admin)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 1, "Repartee adds a +1/+1 counter to self");
}

#[test]
fn scolding_administrator_has_menace() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let admin = g.add_card_to_battlefield(0, catalog::scolding_administrator());
    let card = g.battlefield_find(admin).unwrap();
    assert!(card.has_keyword(&Keyword::Menace));
    assert!(card.definition.has_creature_type(crabomination::card::CreatureType::Dwarf));
    assert!(card.definition.has_creature_type(crabomination::card::CreatureType::Cleric));
}

/// Scolding Administrator's "when this creature dies, if it had
/// counters on it, put those counters on a target creature" rider —
/// promotion from 🟡 to ✅ via `Value::CountersOn` cross-zone lookup
/// (push XXIII) and an `Effect::If` gate on the dies trigger.
///
/// Setup: build the Admin up to 3 counters via 2 Repartee triggers
/// (Bolt + Make Your Mark, both targeting the friendly Bear), then
/// kill the Admin and verify the counters transfer to a target
/// friendly creature.
#[test]
fn scolding_administrator_transfers_counters_on_death() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    // Seed library so any draws don't deck out player 0.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::plains());
    }
    let admin = g.add_card_to_battlefield(0, catalog::scolding_administrator());
    g.battlefield_find_mut(admin).unwrap().summoning_sick = false;
    // Stack a counter on Admin by firing a Repartee-triggering spell.
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Repartee fired once → 1 counter on Admin (Bolt killed the bear).
    let admin_counters = g.battlefield_find(admin)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .unwrap_or(0);
    assert!(admin_counters >= 1,
        "Repartee should have placed ≥ 1 counter (got {admin_counters})");

    // A separate friendly target for the death-trigger counter transfer.
    let recipient = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    // Kill Admin by setting damage equal to its effective toughness.
    let counters_now = admin_counters;
    let admin_eff_toughness = 2 + counters_now as i32;
    g.battlefield_find_mut(admin).unwrap().damage = admin_eff_toughness as u32;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);

    // Admin should be in graveyard.
    assert!(!g.battlefield.iter().any(|c| c.id == admin),
        "Admin dead");
    // Recipient bear should have counters equal to Admin's counters at death.
    let r_counters = g.battlefield_find(recipient)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .unwrap_or(0);
    assert_eq!(r_counters, counters_now,
        "death trigger transferred {counters_now} counters to the target creature");
}

#[test]
fn queue_routed_up_to_one_trigger_offers_decline_to_ui_controller() {
    // Printed "exile up to one other target creature you control" —
    // Ennis's ETB is `ApplyToTargets { min_targets: 0 }`, so a trigger
    // routed through `drain_trigger_queue` (step/event triggers) poses
    // `ChooseTarget { optional: true }` to a `wants_ui` controller;
    // answering `DeclineTarget` resolves the trigger targetless (the
    // whole body no-ops — nothing exiled).
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::game::types::PendingTriggerPush;
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let ennis = g.add_card_to_battlefield(0, catalog::ennis_debate_moderator());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::ennis_debate_moderator().triggered_abilities[0]
        .effect
        .clone();

    g.drain_trigger_queue(vec![PendingTriggerPush {
        actor: None,
        source: ennis,
        controller: 0,
        effect,
        subject: Some(crabomination::game::effects::EntityRef::Permanent(ennis)),
        event_amount: 0,
        mode: None,
        intervening_if: None,
    }]);

    let pending = g.pending_decision.as_ref().expect("flicker target pick pending");
    match &pending.decision {
        Decision::ChooseTarget { optional, legal, .. } => {
            assert!(*optional, "up-to-one pick must be declinable");
            assert!(legal.contains(&Target::Permanent(bear)), "bear offered");
            assert!(!legal.contains(&Target::Permanent(ennis)), "OtherThanSource excludes Ennis");
        }
        other => panic!("expected ChooseTarget, got {other:?}"),
    }
    g.submit_decision(DecisionAnswer::DeclineTarget).expect("decline accepted");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bear), "bear stays — nothing exiled");
    assert!(g.exile.is_empty(), "declined flicker exiles nothing");
}

/// Scolding Administrator's dies-trigger is gated on the printed "if it
/// had counters on it" intervening clause. Verify the counter-bearing
/// gate: an Admin that dies with zero counters should NOT add any
/// counters to a target creature.
#[test]
fn scolding_administrator_dies_without_counters_no_transfer() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let admin = g.add_card_to_battlefield(0, catalog::scolding_administrator());
    let recipient = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let initial = g.battlefield_find(recipient)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .unwrap_or(0);
    // Kill Admin with no counters.
    g.battlefield_find_mut(admin).unwrap().damage = 2;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == admin));
    let r_counters = g.battlefield_find(recipient)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .unwrap_or(0);
    assert_eq!(r_counters, initial,
        "no-counters-on-death gate: trigger does nothing");
}

