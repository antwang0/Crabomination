//! Functionality tests for the Strixhaven Commander (C21) card pack
//! (`catalog::sets::c21`).

use crate::card::Keyword;
use crate::catalog;
use crate::game::*;
use crate::mana::Color;

/// Temple of Epiphany enters tapped, scries on ETB, and taps for U or R.
#[test]
fn temple_of_epiphany_enters_tapped_and_taps_for_mana() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::temple_of_epiphany());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().tapped, "scryland enters tapped");
    // Untap and tap for blue.
    g.battlefield_find_mut(id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("tap for U");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
}

/// Radiant Fountain gains 2 life on entry.
#[test]
fn radiant_fountain_gains_two_life() {
    let mut g = two_player_game();
    let before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::radiant_fountain());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before + 2);
}

/// Rogue's Passage makes a creature unblockable for {4}.
#[test]
fn rogues_passage_grants_unblockable() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::rogues_passage());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None,
    })
    .expect("activate unblockable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Unblockable));
}

/// Mikokoro makes each player draw a card.
#[test]
fn mikokoro_each_player_draws() {
    let mut g = two_player_game();
    for p in 0..2 {
        g.add_card_to_library(p, catalog::island());
    }
    let id = g.add_card_to_battlefield(0, catalog::mikokoro_center_of_the_sea());
    let h0 = g.players[0].hand.len();
    let h1 = g.players[1].hand.len();
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 + 1);
    assert_eq!(g.players[1].hand.len(), h1 + 1);
}

/// High Market sacrifices a creature for 1 life.
#[test]
fn high_market_sacrifices_creature_for_life() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::high_market());
    let before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("sac creature");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == fodder), "creature sacrificed");
    assert_eq!(g.players[0].life, before + 1);
}

/// Temple of the False God only taps for {C}{C} with five+ lands.
#[test]
fn temple_of_false_god_gated_on_five_lands() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::temple_of_the_false_god());
    // With only the Temple, activation is illegal.
    assert!(g
        .perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .is_err());
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("five lands → {C}{C}");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 2);
}

/// Blighted Woodland sacrifices itself to fetch two basics onto the battlefield.
#[test]
fn blighted_woodland_fetches_two_basics() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let f1 = g.add_card_to_library(0, catalog::forest());
    let f2 = g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f1)),
        DecisionAnswer::Search(Some(f2)),
    ]));
    let id = g.add_card_to_battlefield(0, catalog::blighted_woodland());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("sac fetch");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id), "land sacrificed");
    assert!(g.battlefield_find(f1).is_some_and(|c| c.tapped), "basic 1 tapped in");
    assert!(g.battlefield_find(f2).is_some_and(|c| c.tapped), "basic 2 tapped in");
}

/// Barren Moor can be cycled from hand for {2}.
#[test]
fn barren_moor_cycles() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::barren_moor());
    assert!(catalog::barren_moor().keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle Barren Moor");
    drain_stack(&mut g);
    // -1 (cycled away) + 1 (drew) = net 0; the land is in the graveyard.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id));
}

/// Zetalpa carries its full keyword suite.
#[test]
fn zetalpa_has_all_keywords() {
    let z = catalog::zetalpa_primal_dawn();
    for kw in [
        Keyword::Flying, Keyword::DoubleStrike, Keyword::Vigilance,
        Keyword::Trample, Keyword::Indestructible,
    ] {
        assert!(z.keywords.contains(&kw), "missing {kw:?}");
    }
}

/// Verdant Sun's Avatar gains life equal to a creature's toughness on ETB.
#[test]
fn verdant_suns_avatar_gains_life_on_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::verdant_suns_avatar());
    let before = g.players[0].life;
    // Cast a 2/2 bear through the full path so the ETB reaches Verdant's
    // trigger — gain 2 (the bear's toughness).
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bear castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before + 2);
}

/// Sanctum Gargoyle returns an artifact card from the graveyard on ETB.
#[test]
fn sanctum_gargoyle_returns_artifact_from_graveyard() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let relic = g.add_card_to_graveyard(0, catalog::sol_ring());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true), // "may" → yes
        DecisionAnswer::Target(Target::Permanent(relic)),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::sanctum_gargoyle());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == relic), "artifact back in hand");
}

/// Boros Locket sacrifices for two cards.
#[test]
fn boros_locket_sacrifices_for_two_cards() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_battlefield(0, catalog::boros_locket());
    g.players[0].mana_pool.add(Color::Red, 4);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 2, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("sac Boros Locket");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id), "locket sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 2);
}

/// Chain Reaction deals damage to every creature equal to the creature count.
#[test]
fn chain_reaction_scales_with_creature_count() {
    let mut g = two_player_game();
    // Three creatures on the battlefield → 3 damage to each.
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → dies
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4 → survives (4>3)
    let id = g.add_card_to_hand(0, catalog::chain_reaction());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Chain Reaction");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == a), "2/2 dies to 3");
    assert!(!g.battlefield.iter().any(|c| c.id == b), "2/2 dies to 3");
    assert!(g.battlefield.iter().any(|c| c.id == wurm), "6/4 survives 3 damage");
}

/// Gaze of Granite destroys nonland permanents with MV <= X.
#[test]
fn gaze_of_granite_destroys_by_mana_value() {
    let mut g = two_player_game();
    let cheap = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let pricey = g.add_card_to_battlefield(1, catalog::craw_wurm());    // MV 6
    let id = g.add_card_to_hand(0, catalog::gaze_of_granite());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3); // X = 3
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    })
    .expect("cast Gaze of Granite X=3");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == cheap), "MV 2 destroyed");
    assert!(g.battlefield.iter().any(|c| c.id == pricey), "MV 6 survives");
}

/// Biomass Mutation sets your creatures' base P/T to X/X.
#[test]
fn biomass_mutation_sets_base_pt() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::biomass_mutation());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4); // X = 4
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(4),
    })
    .expect("cast Biomass Mutation X=4");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4));
}

/// Perplexing Test mode 1 bounces all nontoken creatures to owners' hands.
#[test]
fn perplexing_test_bounces_nontoken_creatures() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let id = g.add_card_to_hand(0, catalog::perplexing_test());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("cast Perplexing Test (nontoken)");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "own creature bounced to my hand");
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs), "opp creature bounced to their hand");
}

/// Taste of Death makes each player sacrifice three creatures and nets Food.
#[test]
fn taste_of_death_sacrifices_and_makes_food() {
    let mut g = two_player_game();
    let mut theirs = vec![];
    for _ in 0..3 { theirs.push(g.add_card_to_battlefield(1, catalog::grizzly_bears())); }
    let id = g.add_card_to_hand(0, catalog::taste_of_death());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Taste of Death");
    drain_stack(&mut g);
    for t in theirs {
        assert!(!g.battlefield.iter().any(|c| c.id == t), "opponent creature sacrificed");
    }
    let food = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Food").count();
    assert_eq!(food, 3, "three Food tokens created");
}

/// Sculpting Steel enters as a copy of an artifact on the battlefield.
#[test]
fn sculpting_steel_copies_an_artifact() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sol_ring());
    let steel = g.add_card_to_hand(0, catalog::sculpting_steel());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: steel, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Sculpting Steel");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(steel).unwrap().definition.name, "Sol Ring",
        "Sculpting Steel enters as a Sol Ring copy");
}

/// Phyrexia's Core sacrifices an artifact for 1 life.
#[test]
fn phyrexias_core_sacrifices_artifact_for_life() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let id = g.add_card_to_battlefield(0, catalog::phyrexias_core());
    g.players[0].mana_pool.add_colorless(1);
    let before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("sac artifact");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == ring), "artifact sacrificed");
    assert_eq!(g.players[0].life, before + 1);
}

/// Brass's Bounty makes a Treasure for each land you control.
#[test]
fn brasss_bounty_makes_treasure_per_land() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::brasss_bounty());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Brass's Bounty");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Treasure").count();
    assert_eq!(treasures, 3, "one Treasure per land");
}

/// Oblation shuffles a permanent into its owner's library and they draw two.
#[test]
fn oblation_shuffles_and_owner_draws_two() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(1, catalog::island()); }
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::oblation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_hand = g.players[1].hand.len();
    cast_at(&mut g, id, Target::Permanent(ring));
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == ring), "permanent left battlefield");
    assert!(g.players[1].library.iter().any(|c| c.id == ring), "shuffled into owner's library");
    assert_eq!(g.players[1].hand.len(), opp_hand + 2, "owner drew two");
}
