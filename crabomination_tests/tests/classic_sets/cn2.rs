//! Conspiracy: Take the Crown (CN2) — the monarch shell, melee, goad,
//! monstrosity and the council's dilemma (`catalog::sets::cn2`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
}

fn swing(g: &mut GameState, id: CardId) {
    g.clear_sickness(id);
    advance_to(g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(g);
}

/// Every CN2 factory builds and lands in the catalog.
#[test]
fn cn2_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for name in [
        "Ballot Broker",
        "Crown-Hunter Hireling",
        "Queen Marchesa",
        "Throne of the High City",
        "Splitting Slime",
        "Selvala, Heart of the Wilds",
    ] {
        assert!(names.contains(&name), "{name} is missing from the catalog");
    }
}

/// Protector of the Crown crowns you and soaks damage aimed at your face.
#[test]
fn protector_of_the_crown_crowns_and_soaks() {
    let mut g = two_player_game();
    let prot = g.move_card_to_battlefield_for_test(0, catalog::protector_of_the_crown());
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0), "CR 725.3 — the ETB crowns its controller");

    let mut evs = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        3,
        None,
        &mut evs,
    );
    assert_eq!(g.players[0].life, 20, "the damage was redirected");
    assert_eq!(g.battlefield_find(prot).unwrap().damage, 3);
}

/// Crown-Hunter Hireling can only swing at whoever holds the crown.
#[test]
fn crown_hunter_hireling_only_attacks_the_monarch() {
    let mut g = two_player_game();
    let ogre = g.add_card_to_battlefield(0, catalog::crown_hunter_hireling());
    g.clear_sickness(ogre);
    g.monarch = Some(0);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: ogre,
            target: AttackTarget::Player(1),
        }]))
        .is_err(),
        "seat 1 isn't the monarch"
    );
    g.monarch = Some(1);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ogre,
        target: AttackTarget::Player(1),
    }]))
    .expect("the monarch is a legal defender");
}

/// Knights of the Black Rose bleeds whoever steals your crown mid-turn.
#[test]
fn knights_of_the_black_rose_punishes_a_crown_theft() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::knights_of_the_black_rose());
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0));
    g.monarch_at_turn_start = Some(0);

    let mut evs = vec![];
    g.set_monarch(1, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 22);
}

/// Queen Marchesa mints an Assassin each upkeep the crown sits elsewhere.
#[test]
fn queen_marchesa_mints_an_assassin_without_the_crown() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::queen_marchesa());
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0));
    // Wearing the crown yourself, the upkeep trigger stays quiet.
    g.monarch = Some(1);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let assassin = g.battlefield.iter().find(|c| c.definition.name == "Assassin");
    assert!(assassin.is_some_and(|c| c.definition.keywords.contains(&Keyword::Deathtouch)));
}

/// Throne of the High City taps for {C} and buys the crown.
#[test]
fn throne_of_the_high_city_buys_the_crown() {
    let mut g = two_player_game();
    let throne = g.add_card_to_battlefield(0, catalog::throne_of_the_high_city());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: throne,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sacrifice for the crown");
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0));
    assert!(g.battlefield_find(throne).is_none(), "sacrificed as a cost");
}

/// Custodi Soulcaller's melee pump and its mana-value-gated reanimation both
/// read the number of players it attacked.
#[test]
fn custodi_soulcaller_reanimates_up_to_the_players_attacked() {
    let mut g = two_player_game();
    let caller = g.add_card_to_battlefield(0, catalog::custodi_soulcaller());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    swing(&mut g, caller);
    // One player attacked: melee makes it 2/3, and only MV ≤ 1 comes back.
    assert_eq!(g.computed_permanent(caller).unwrap().power, 2);
    assert!(g.battlefield_find(bears).is_none(), "Grizzly Bears costs {{1}}{{G}}");
}

/// Sinuous Vermin only gains menace once it goes monstrous.
#[test]
fn sinuous_vermin_gains_menace_when_monstrous() {
    let mut g = two_player_game();
    let rat = g.add_card_to_battlefield(0, catalog::sinuous_vermin());
    assert!(!g.computed_permanent(rat).unwrap().keywords.contains(&Keyword::Menace));

    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rat,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("monstrosity 3");
    drain_stack(&mut g);
    let vermin = g.computed_permanent(rat).unwrap();
    assert_eq!((vermin.power, vermin.toughness), (5, 5));
    assert!(vermin.keywords.contains(&Keyword::Menace));
}

/// Splitting Slime clones itself the moment it becomes monstrous.
#[test]
fn splitting_slime_clones_itself_on_monstrosity() {
    let mut g = two_player_game();
    let slime = g.add_card_to_battlefield(0, catalog::splitting_slime());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: slime,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("monstrosity 3");
    drain_stack(&mut g);
    let copy = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.definition.name == "Splitting Slime")
        .expect("token copy");
    assert_eq!(copy.counter_count(CounterType::PlusOnePlusOne), 0, "the copy has no counters");
}

/// Orchard Elemental's council's dilemma pays per vote (CR 701.38).
#[test]
fn orchard_elemental_pays_per_vote() {
    let mut g = two_player_game();
    // Both seats vote Harvest (option index 1).
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(1),
        DecisionAnswer::Amount(1),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::orchard_elemental());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 26, "two harvest votes = 6 life");
}

/// Illusion of Choice hands you every ballot for the turn (CR 701.38).
#[test]
fn illusion_of_choice_answers_every_ballot() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::illusion_of_choice());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.vote_controller_this_turn, Some(0));

    // Seat 0 now answers seat 1's ballot too: both votes go to Sprout.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(0),
        DecisionAnswer::Amount(0),
    ]));
    let elem = g.move_card_to_battlefield_for_test(0, catalog::orchard_elemental());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(elem).unwrap().counter_count(CounterType::PlusOnePlusOne),
        4,
        "two sprout votes = four counters"
    );
}

/// Ballot Broker casts a second vote for its controller.
#[test]
fn ballot_broker_votes_twice() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ballot_broker());
    // Seat 0 votes Sprout twice, seat 1 votes Harvest.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(0),
        DecisionAnswer::Amount(0),
        DecisionAnswer::Amount(1),
    ]));
    let elem = g.move_card_to_battlefield_for_test(0, catalog::orchard_elemental());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(elem).unwrap().counter_count(CounterType::PlusOnePlusOne),
        4
    );
    assert_eq!(g.players[0].life, 23, "the lone harvest vote still paid");
}

/// Deadly Designs is fed by any player and pops at five plot counters.
#[test]
fn deadly_designs_pops_at_five_plot_counters() {
    let mut g = two_player_game();
    let plot = g.add_card_to_battlefield(0, catalog::deadly_designs());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    for _ in 0..5 {
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: plot,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("add a plot counter");
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(plot).is_none(), "sacrificed itself");
    assert!(g.battlefield_find(victim).is_none(), "and took a creature with it");
}

/// Selvala's mana ability scales with your biggest creature.
#[test]
fn selvala_taps_for_your_greatest_power() {
    let mut g = two_player_game();
    let selvala = g.add_card_to_battlefield(0, catalog::selvala_heart_of_the_wilds());
    g.clear_sickness(selvala);
    g.add_card_to_battlefield(0, catalog::craw_wurm()); // 6/4
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: selvala,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.total(), 6, "greatest power among your creatures");
}

/// Besmirch borrows a creature and goads it so it can't swing back.
#[test]
fn besmirch_borrows_and_goads() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::besmirch());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Besmirch");
    drain_stack(&mut g);
    let stolen = g.battlefield_find(bear).unwrap();
    assert_eq!(stolen.controller, 0);
    assert!(!stolen.tapped, "untapped");
    assert!(!stolen.goaded_by.is_empty(), "and goaded");
}
