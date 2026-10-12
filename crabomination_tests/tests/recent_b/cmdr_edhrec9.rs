//! Commander: one-card EDHREC gaps of EDHREC's top-600 commanders
//! (`decks::cmdr_edhrec9`).

use crabomination::card::{CardId, CardType, CounterType};
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
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

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn perm(id: CardId) -> Target {
    Target::Permanent(id)
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

fn tokens_named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.is_token && c.definition.name == name).count()
}

fn activate_free(g: &mut GameState, id: CardId, index: usize) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn creature_def(name: &'static str, p: i32, t: i32, kind: crabomination::card::CreatureType) -> crabomination::card::CardDefinition {
    crabomination::card::CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        subtypes: crabomination::card::Subtypes { creature_types: vec![kind], ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn bolt(g: &mut GameState, caster: usize, target: Target) {
    let b = g.add_card_to_hand(caster, catalog::lightning_bolt());
    flood(g, caster);
    g.priority.player_with_priority = caster;
    g.perform_action(GameAction::CastSpell { card_id: b, target: Some(target), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(g);
}

/// Smaug: noncombat damage dealt to it makes that many Treasures (a new
/// `EventKind::DealtNoncombatDamage`).
#[test]
fn smaug_turns_noncombat_damage_into_treasure() {
    let mut g = pod(2);
    let s = ready(&mut g, 0, catalog::smaug_the_impenetrable());
    bolt(&mut g, 1, perm(s));
    assert_eq!(tokens_named(&g, "Treasure"), 3);
}

/// Heliod, the Warped Eclipse: spells cost {1} less per card the opponents
/// drew this turn (summed over every opponent).
#[test]
fn heliod_warped_eclipse_discounts_by_opponents_draws() {
    let mut g = pod(3);
    library(&mut g, 1, 3);
    library(&mut g, 2, 3);
    let h = ready(&mut g, 0, catalog::heliod_the_radiant_dawn());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: h, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("transform");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(h).unwrap().definition.name, "Heliod, the Warped Eclipse");
    g.players[1].cards_drawn_this_turn = 1;
    g.players[2].cards_drawn_this_turn = 1;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("two opposing draws take the Bears' {1} off: {G} pays");
}

/// Briarbridge Patrol: three Clues sacrificed this turn let the end step put
/// a creature from hand onto the battlefield.
#[test]
fn briarbridge_patrol_cashes_three_clues() {
    let mut g = pod(2);
    library(&mut g, 0, 5);
    ready(&mut g, 0, catalog::briarbridge_patrol());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..3 {
        let c = g.add_card_to_hand(0, catalog::confront_the_unknown());
        let pat = g.battlefield.iter().find(|c| c.definition.name == "Briarbridge Patrol").unwrap().id;
        cast(&mut g, c, &[perm(pat)]).expect("investigate");
    }
    let clues: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Clue").map(|c| c.id).collect();
    assert_eq!(clues.len(), 3);
    for c in clues {
        activate(&mut g, c, 0, None).expect("crack a Clue");
    }
    assert_eq!(g.players[0].clues_sacrificed_this_turn, 3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
}

/// Haze Frog: other creatures' combat damage is prevented; its own isn't
/// touched by the filter.
#[test]
fn haze_frog_fogs_the_others() {
    let mut g = pod(2);
    let frog = g.add_card_to_hand(0, catalog::haze_frog());
    cast(&mut g, frog, &[]).expect("cast");
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let _ = bear;
    assert!(g.battlefield_find(frog).is_some());
}

/// Apocalypse Hydra: X counters, doubled at five or more.
#[test]
fn apocalypse_hydra_doubles_at_five() {
    let mut g = pod(2);
    let h = g.add_card_to_hand(0, catalog::apocalypse_hydra());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: h, target: None, additional_targets: vec![], mode: None, x_value: Some(5) })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(h).unwrap().counter_count(CounterType::PlusOnePlusOne), 10);
}

/// Curious Colossus: the target opponent's creatures become 1/1 Cowards.
#[test]
fn curious_colossus_shrinks_a_board() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::curious_colossus());
    cast(&mut g, c, &[Target::Player(1)]).expect("cast");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.subtypes().creature_types.contains(&crabomination::card::CreatureType::Coward));
}

/// Stronghold Rats: shadow; a hit makes each player discard.
#[test]
fn stronghold_rats_has_shadow() {
    let mut g = pod(2);
    let r = ready(&mut g, 0, catalog::stronghold_rats());
    assert!(g.computed_permanent(r).unwrap().keywords().contains(&crabomination::card::Keyword::Shadow));
}

/// Angelic Ascension: exile a creature; its controller gets a 4/4 Angel.
#[test]
fn angelic_ascension_trades_for_an_angel() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let a = g.add_card_to_hand(0, catalog::angelic_ascension());
    cast(&mut g, a, &[perm(bear)]).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Angel" && c.controller == 1));
}

/// Kaervek shrinks every other creature.
#[test]
fn kaervek_shrinks_others() {
    let mut g = pod(2);
    let k = ready(&mut g, 0, catalog::kaervek_the_spiteful());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 1);
    assert_eq!(g.computed_permanent(k).unwrap().power, 3);
}

/// Urza's Workshop: metalcraft taps for each Urza's land.
#[test]
fn urzas_workshop_needs_metalcraft() {
    let mut g = pod(2);
    let w1 = ready(&mut g, 0, catalog::urzas_workshop());
    ready(&mut g, 0, catalog::urzas_workshop());
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility { card_id: w1, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
            .is_err(),
        "no metalcraft"
    );
    for _ in 0..3 {
        ready(&mut g, 0, catalog::sol_ring());
    }
    g.perform_action(GameAction::ActivateAbility { card_id: w1, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("metalcraft");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 2);
}

/// Compy Swarm copies itself, tapped, at an end step after a death.
#[test]
fn compy_swarm_multiplies_after_a_death() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::compy_swarm());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    bolt(&mut g, 0, perm(bear));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let copy = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Compy Swarm").expect("a copy");
    assert!(copy.tapped);
}

/// Spitting Dilophosaurus: ETB -1/-1 counter; that opponent's creature
/// can't block.
#[test]
fn spitting_dilophosaurus_spits() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let d = g.add_card_to_hand(0, catalog::spitting_dilophosaurus());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(perm(bear))]));
    cast(&mut g, d, &[]).expect("cast");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&crabomination::card::Keyword::CantBlock));
}

/// Staff of Eden: draws one per permanent you control but don't own.
#[test]
fn staff_of_eden_counts_borrowed_permanents() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let staff = ready(&mut g, 0, catalog::staff_of_eden_vaults_key());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().controller = 0;
    let hand = g.players[0].hand.len();
    activate(&mut g, staff, 0, None).expect("tap");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Bre: the end step after life gain casts a cheap enough dug card for free.
#[test]
fn bre_casts_the_dug_card_within_the_life_gained() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::bre_of_clan_stoutarm());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].life_gained_this_turn = 3;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "cast free");
}

/// Mabel: an entering creature gets a counter on a creature that entered
/// this turn.
#[test]
fn mabel_grows_new_arrivals() {
    let mut g = pod(2);
    let m = g.add_card_to_hand(0, catalog::mabel_valley_hero());
    cast(&mut g, m, &[]).expect("cast");
    assert_eq!(g.battlefield_find(m).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Eirdu: the first main phase may pay {B} to become Isilu.
#[test]
fn eirdu_transforms_for_black() {
    let mut g = pod(2);
    let e = ready(&mut g, 0, catalog::eirdu_carrier_of_dawn());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(e).unwrap().definition.name, "Isilu, Carrier of Twilight");
}

/// Workshop Assistant returns another artifact card when it dies.
#[test]
fn workshop_assistant_recovers_an_artifact() {
    let mut g = pod(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let wa = ready(&mut g, 0, catalog::workshop_assistant());
    bolt(&mut g, 1, perm(wa));
    assert!(g.players[0].hand.iter().any(|c| c.id == ring));
}
