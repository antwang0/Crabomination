//! Saviors of Kamigawa closure — the eight primitive-blocked gaps.

use crabomination::card::{CounterType, CreatureType, SelectionRequirement as R};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// Every SOK closure factory is registered under its printed name.
#[test]
fn sok3_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::ashes_of_the_fallen as fn() -> crabomination::card::CardDefinition,
        catalog::choice_of_damnations,
        catalog::kaho_minamo_historian,
        catalog::murmurs_from_beyond,
        catalog::pains_reward,
        catalog::pure_intentions,
        catalog::rally_the_horde,
        catalog::sekki_seasons_guide,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// Ashes of the Fallen gives graveyard creature cards the chosen type.
#[test]
fn ashes_of_the_fallen_types_your_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ashes = g.add_card_to_battlefield(0, catalog::ashes_of_the_fallen());
    g.battlefield_find_mut(ashes).unwrap().chosen_creature_type = Some(CreatureType::Spirit);
    let card = g.find_card_anywhere(bear).expect("in graveyard").clone();
    assert!(
        g.evaluate_requirement_on_card(&R::HasCreatureType(CreatureType::Spirit), &card, 0),
        "the Bear is a Spirit in the graveyard"
    );
    // An opponent's graveyard is untouched.
    let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let card = g.find_card_anywhere(theirs).expect("in graveyard").clone();
    assert!(!g.evaluate_requirement_on_card(&R::HasCreatureType(CreatureType::Spirit), &card, 1));
}

/// Choice of Damnations: taking the life branch drains for the chosen number.
#[test]
fn choice_of_damnations_drains_the_chosen_number() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::choice_of_damnations());
    g.players[0].mana_pool.add(Color::Black, 6);
    // The opponent picks 3; the caster says yes to the life drain.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Amount(3),
        DecisionAnswer::Bool(true),
    ]));
    let life = g.players[1].life;
    cast(&mut g, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 3);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1).count(), 4, "nothing sacrificed");
}

/// Declining the drain makes the opponent keep only the chosen number.
#[test]
fn choice_of_damnations_sacrifices_all_but_the_chosen_number() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::choice_of_damnations());
    g.players[0].mana_pool.add(Color::Black, 6);
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Amount(1),
        DecisionAnswer::Bool(false),
    ]));
    let life = g.players[1].life;
    cast(&mut g, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life, "no life lost");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1).count(), 1);
}

/// Kaho exiles instants on entry and free-casts the one whose MV matches X.
#[test]
fn kaho_free_casts_an_exiled_instant_of_mana_value_x() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(None)]));
    let kaho = g.add_card_to_battlefield(0, catalog::kaho_minamo_historian());
    drain_stack(&mut g);
    g.battlefield_find_mut(kaho).unwrap().summoning_sick = false;
    // Stamp the Bolt as exiled-with-Kaho directly: the ETB search's pick path
    // is covered by the search tests.
    let pos = g.players[0]
        .library
        .iter()
        .position(|c| c.definition.name == "Lightning Bolt")
        .expect("bolt in library");
    let mut card = g.players[0].library.remove(pos);
    card.exiled_with = Some(kaho);
    g.exile.push(card);
    g.decider = Box::new(crabomination::decision::AutoDecider);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: kaho,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(1),
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "the free Bolt resolved");
    assert!(g.exile.is_empty(), "the cast card left exile");
}

/// Murmurs from Beyond: the opponent bins one of the three, the rest go to hand.
#[test]
fn murmurs_from_beyond_lets_an_opponent_bin_one() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::murmurs_from_beyond());
    g.players[0].mana_pool.add(Color::Blue, 3);
    let hand = g.players[0].hand.len();
    let gy = g.players[0].graveyard.len();
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].graveyard.len(), gy + 2, "one binned card plus the spell");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "the other two to hand");
}

/// Pain's Reward: the opener wins an unopposed bid, pays it, and draws four.
#[test]
fn pains_reward_pays_the_high_bid_and_draws_four() {
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::pains_reward());
    g.players[0].mana_pool.add(Color::Black, 3);
    // Caster opens at 5; the opponent passes with 0.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Amount(5),
        DecisionAnswer::Amount(0),
    ]));
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].life, life - 5);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 4);
}

/// A higher counter-bid moves both the life payment and the draw.
#[test]
fn pains_reward_pays_out_to_the_top_bidder() {
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_library(1, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::pains_reward());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Amount(2),
        DecisionAnswer::Amount(6),
        DecisionAnswer::Amount(0),
    ]));
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].life, mine, "the caster was outbid");
    assert_eq!(g.players[1].life, theirs - 6);
    assert_eq!(g.players[1].hand.len(), 4);
}

/// Pure Intentions hands back the cards an opponent's spell made you discard.
#[test]
fn pure_intentions_returns_opponent_forced_discards() {
    let mut g = two_player_game();
    let pure = g.add_card_to_hand(0, catalog::pure_intentions());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, pure, None);
    // Now an opponent's discard spell resolves.
    let mind_rot = g.add_card_to_hand(1, catalog::mind_rot());
    g.players[1].mana_pool.add(Color::Black, 3);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: mind_rot,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the discarded Bear came straight back"
    );
}

/// Rally the Horde exiles in threes while the last card is a land, then pays
/// out one Warrior per nonland exiled.
#[test]
fn rally_the_horde_makes_a_warrior_per_nonland_exiled() {
    let mut g = two_player_game();
    // Library top-down: Bear, Bear, Forest (repeat), then three nonlands.
    for def in [
        catalog::grizzly_bears(),
        catalog::grizzly_bears(),
        catalog::forest(),
        catalog::grizzly_bears(),
        catalog::grizzly_bears(),
        catalog::grizzly_bears(),
    ] {
        g.add_card_to_library(0, def);
    }
    let spell = g.add_card_to_hand(0, catalog::rally_the_horde());
    g.players[0].mana_pool.add(Color::Red, 6);
    cast(&mut g, spell, None);
    // First batch ends on the Forest so it repeats; the second ends on a Bear.
    let warriors = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Warrior")
        .count();
    assert_eq!(warriors, 5, "five nonland cards across two batches");
}

/// Sekki turns incoming damage into Spirits, one per counter spent.
#[test]
fn sekki_trades_counters_for_spirits() {
    let mut g = two_player_game();
    let sekki = g.add_card_to_battlefield_with_counters(0, catalog::sekki_seasons_guide());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sekki).unwrap().counter_count(CounterType::PlusOnePlusOne), 8);
    let mut ev = vec![];
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(sekki), 3, None, &mut ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sekki).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count(),
        3,
        "one Spirit per counter removed"
    );
    assert_eq!(g.battlefield_find(sekki).unwrap().damage, 0, "the damage was prevented");
}

/// Eight Spirits reanimate Sekki from the graveyard.
#[test]
fn sekki_returns_itself_for_eight_spirits() {
    let mut g = two_player_game();
    let sekki = g.add_card_to_graveyard(0, catalog::sekki_seasons_guide());
    for _ in 0..8 {
        g.add_card_to_battlefield(0, catalog::kami_of_ancient_law());
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: sekki,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate from graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sekki).is_some(), "Sekki came back");
}
