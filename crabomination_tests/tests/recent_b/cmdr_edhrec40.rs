//! Commander: Braids', Taigam's, Carth's, Gogo's, Edea's, Uugguu's and Loot's
//! last gaps (`decks::cmdr_edhrec40`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// CR 500.8 — Shadow of the Second Sun on an opponent: their postcombat main
/// banks an extra beginning phase; its caster's own doesn't.
#[test]
fn cr_500_8_shadow_of_the_second_sun_follows_the_enchanted_player() {
    let mut g = pod(3);
    let shadow = g.add_card_to_hand(0, catalog::shadow_of_the_second_sun());
    cast(&mut g, shadow, Some(Target::Player(2))).expect("shadow");
    g.step = TurnStep::PostCombatMain;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.additional_beginning_phases, 0, "not the enchanted player's turn");
    g.active_player_idx = 2;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.additional_beginning_phases, 1);
}

/// Jhoira's Timebug: its controller picks — a vanishing permanent of theirs
/// gains a time counter or loses one.
#[test]
fn jhoiras_timebug_moves_a_time_counter_either_way() {
    let mut g = pod(2);
    let bug = g.add_card_to_battlefield(0, catalog::jhoiras_timebug());
    let clock = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(clock).unwrap().counters.insert(CounterType::Time, 2);
    for (answer, want) in [(1, 3), (0, 2)] {
        g.battlefield_find_mut(bug).unwrap().tapped = false;
        g.battlefield_find_mut(bug).unwrap().summoning_sick = false;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(answer)]));
        g.perform_action(GameAction::ActivateAbility {
            card_id: bug,
            ability_index: 0,
            target: Some(Target::Permanent(clock)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("timebug");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(clock).unwrap().counter_count(CounterType::Time), want);
    }
}

/// CR 606.1 — a loyalty ability is an activated ability: Way of the Paradox
/// sees Vraska's +2 (a life and a land play); Way of the Necromancer's Jace
/// and Vraska each gain loyalty when a creature of yours dies.
#[test]
fn cr_606_1_the_ways_see_loyalty_activations_and_deaths() {
    let mut g = pod(2);
    let necro = g.add_card_to_hand(0, catalog::way_of_the_necromancer());
    cast(&mut g, necro, None).expect("necromancer");
    let jace = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Jace").map(|c| c.id).expect("Jace");
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 2);
    let paradox = g.add_card_to_hand(0, catalog::way_of_the_paradox());
    cast(&mut g, paradox, None).expect("paradox");
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 7, "empower 5 on the same Jace");
    let vraska = g.add_card_to_battlefield(0, catalog::vraska_relic_seeker());
    let life = g.players[0].life;
    loyalty(&mut g, vraska, 0, None).expect("+2");
    assert_eq!(g.players[0].life, life + 1, "Paradox: 1 life");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Pirate"));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Permanent(bear))).expect("shock");
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 8);
    assert_eq!(g.battlefield_find(vraska).unwrap().counter_count(CounterType::Loyalty), 6 + 2 + 1);
}

/// Vraska, Relic Seeker's −10 sets a player's life total to 1 (CR 119.5).
#[test]
fn cr_119_5_vraska_ultimate_sets_life_to_one() {
    let mut g = pod(3);
    let vraska = g.add_card_to_battlefield(0, catalog::vraska_relic_seeker());
    g.battlefield_find_mut(vraska).unwrap().counters.insert(CounterType::Loyalty, 10);
    loyalty(&mut g, vraska, 2, Some(Target::Player(2))).expect("-10");
    assert_eq!(g.players[2].life, 1);
    assert!(g.battlefield_find(vraska).is_none(), "zero loyalty: to the graveyard");
}

/// A bot casts Shadow of the Second Sun on itself — the extra untap, upkeep
/// and draw are its own.
#[test]
fn bot_enchants_itself_with_shadow_of_the_second_sun() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = pod(3);
    let shadow = g.add_card_to_hand(0, catalog::shadow_of_the_second_sun());
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    let action = HeuristicBot::new().next_action(&g, 0).expect("an action");
    match action {
        GameAction::CastSpell { card_id, target, .. } => {
            assert_eq!(card_id, shadow);
            assert_eq!(target, Some(Target::Player(0)));
        }
        other => panic!("expected the Shadow cast, got {other:?}"),
    }
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// Loot's upkeep exiles one card per card type among its controller's other
/// nonland permanents — an artifact creature counts twice — playable this turn.
#[test]
fn loot_exiles_by_card_types() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    g.add_card_to_battlefield(0, catalog::loot_the_key_to_everything());
    g.add_card_to_battlefield(0, catalog::jhoiras_timebug());
    g.add_card_to_battlefield(0, catalog::gutter_grime());
    g.add_card_to_battlefield(0, catalog::forest());
    let exiled = g.exile.len();
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), exiled + 3, "artifact, creature, enchantment");
}

/// Invasion of Kaldheim swaps the hand for as many fresh cards; its back face
/// burns for a discarded land and impulse-draws off it.
#[test]
fn invasion_of_kaldheim_wheels_the_hand_into_exile() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let inv = g.add_card_to_hand(0, catalog::invasion_of_kaldheim());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let exiled = g.exile.len();
    cast(&mut g, inv, None).expect("invasion");
    assert_eq!(g.players[0].hand.len(), 3, "three exiled, three drawn");
    assert_eq!(g.exile.len(), exiled + 3);

    let pyre = *catalog::invasion_of_kaldheim().back_face.unwrap();
    let pyre = g.add_card_to_battlefield(0, pyre);
    g.add_card_to_hand(0, catalog::forest());
    let graveyard = g.players[0].graveyard.len();
    let life = g.players[1].life;
    let exiled = g.exile.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: pyre,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("pyre");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), graveyard + 1);
    assert!(g.players[0].graveyard.last().unwrap().definition.is_land(), "a land was discarded");
    assert_eq!(g.players[1].life, life - 2);
    assert_eq!(g.exile.len(), exiled + 1, "the discard trigger exiled the top card");
}

/// Omenpath Journey exiles lands with different names, then drops one at
/// random onto the battlefield tapped at its controller's end step.
#[test]
fn omenpath_journey_banks_differently_named_lands() {
    let mut g = pod(2);
    for land in [catalog::forest(), catalog::forest(), catalog::island(), catalog::swamp()] {
        g.add_card_to_library(0, land);
    }
    let journey = g.add_card_to_hand(0, catalog::omenpath_journey());
    cast(&mut g, journey, None).expect("journey");
    let banked: Vec<_> = g.exile.iter().filter(|c| c.exiled_with == Some(journey)).map(|c| c.definition.name).collect();
    assert_eq!(banked.len(), 3, "Forest, Island, Swamp — not the second Forest: {banked:?}");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let landed: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).collect();
    assert_eq!(landed.len(), 1);
    assert!(landed[0].tapped);
}

/// Uugguu: a nontoken Ooze of yours dying leaves two 2/2 nonlegendary token
/// copies; Gutter Grime adds a slime counter and an Ooze as big as its count.
#[test]
fn uugguu_and_gutter_grime_replace_dead_oozes() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::uugguu_the_omniplasm());
    let grime = g.add_card_to_battlefield(0, catalog::gutter_grime());
    let slime = g.add_card_to_battlefield(0, catalog::consumptive_goo());
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Permanent(slime))).expect("shock");
    let copies = named(&g, 0, "Consumptive Goo");
    assert_eq!(copies.len(), 2);
    for c in &copies {
        let cp = g.computed_permanent(*c).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2));
    }
    assert_eq!(g.battlefield_find(grime).unwrap().counter_count(CounterType::Slime), 1);
    let oozes = named(&g, 0, "Ooze");
    assert_eq!(oozes.len(), 1);
    assert_eq!(g.computed_permanent(oozes[0]).unwrap().power, 1);
    let bolt = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, bolt, Some(Target::Permanent(copies[0]))).expect("shock a token");
    assert_eq!(g.battlefield_find(grime).unwrap().counter_count(CounterType::Slime), 1, "a token isn't nontoken");
}

/// Omni-Changeling enters as a copy that keeps changeling (CR 702.73a).
#[test]
fn cr_702_73a_omni_changeling_copies_with_changeling() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let omni = g.add_card_to_hand(0, catalog::omni_changeling());
    cast(&mut g, omni, None).expect("omni");
    let cp = g.computed_permanent(omni).expect("on the battlefield");
    assert_eq!(cp.def.name, "Grizzly Bears");
    assert!(cp.keywords().contains(&Keyword::Changeling), "changeling: every creature type");
}

/// Edea steals a creature at her beginning of combat, untapped and hasty;
/// when a creature she controls but doesn't own dies it goes home and she
/// draws.
#[test]
fn edea_borrows_and_returns_creatures() {
    use crabomination::card::Keyword;
    let mut g = pod(3);
    stock_libraries(&mut g, 5);
    g.add_card_to_battlefield(0, catalog::edea_possessed_sorceress());
    let bear = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.controller, 0);
    assert!(!b.tapped);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Haste));
    let hand = g.players[0].hand.len();
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Permanent(bear))).expect("shock");
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(2), "back under its owner");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// CR 122.1h — Chaos Shrine's Black Crystal banks a dead creature and brings
/// it back at upkeep with a finality counter.
#[test]
fn cr_122_1h_black_crystal_returns_with_finality() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::chaos_shrines_black_crystal());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Permanent(bear))).expect("shock");
    assert!(g.exile.iter().any(|c| c.id == bear), "exiled, not in the graveyard");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let back = named(&g, 0, "Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(g.battlefield_find(back[0]).unwrap().counter_count(CounterType::Finality), 1);
}

/// CR 724 / 611.2c — Garland makes an opponent the monarch, takes one of
/// their creatures (+2/+2 while it's Garland's), and loses it when the crown
/// moves on.
#[test]
fn cr_611_2c_garland_holds_a_hostage_while_they_reign() {
    let mut g = pod(3);
    let bears = [g.add_card_to_battlefield(1, catalog::grizzly_bears()), g.add_card_to_battlefield(2, catalog::grizzly_bears())];
    let garland = g.add_card_to_hand(0, catalog::garland_royal_kidnapper());
    cast(&mut g, garland, None).expect("garland");
    let monarch = g.monarch.expect("an opponent was crowned");
    assert_ne!(monarch, 0);
    let bear = bears[monarch - 1];
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "the kidnap trigger");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+2 while not its owner's");
    assert_eq!(g.battlefield_find(bears[2 - monarch]).unwrap().controller, 3 - monarch, "only the monarch's");
    g.monarch = Some(3 - monarch);
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(bear).unwrap().controller, monarch, "its owner lost the crown");
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// CR 702.167 / 602.5b — The Enigma Jewel crafts with four nonlands that have
/// activated abilities; the Locus borrows their abilities, each once a turn,
/// and copies every non-mana ability its controller activates.
#[test]
fn cr_702_167_locus_borrows_and_copies_crafted_abilities() {
    let mut g = pod(2);
    let jewel = g.add_card_to_battlefield(0, catalog::the_enigma_jewel());
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::jhoiras_timebug());
    }
    let clock = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(clock).unwrap().counters.insert(CounterType::Time, 5);
    flood(&mut g, 0);
    activate(&mut g, jewel, 1, None).expect("craft");
    assert_eq!(g.battlefield_find(jewel).unwrap().definition.name, "Locus of Enlightenment");
    assert_eq!(g.exile.iter().filter(|c| c.exiled_with == Some(jewel)).count(), 4);
    g.battlefield_find_mut(jewel).unwrap().tapped = false;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(0), DecisionAnswer::Amount(0)]));
    let printed = catalog::the_enigma_jewel().back_face.unwrap().activated_abilities.len();
    activate(&mut g, jewel, printed, Some(Target::Permanent(clock))).expect("borrowed Timebug ability");
    assert_eq!(g.battlefield_find(clock).unwrap().counter_count(CounterType::Time), 3, "the ability and its copy");
    g.battlefield_find_mut(jewel).unwrap().tapped = false;
    assert!(activate(&mut g, jewel, printed, Some(Target::Permanent(clock))).is_err(), "once each turn");
}
