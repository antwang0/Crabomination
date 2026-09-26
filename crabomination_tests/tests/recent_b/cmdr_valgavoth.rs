//! Commander: the Endless Punishment precon (DSC, Valgavoth,
//! `decks::cmdr_valgavoth`) and the primitives it needed.

use crabomination::card::{CardDefinition, CardId, CardType, CounterType, CreatureType, Keyword, Subtypes};
use crabomination::card::SelectionRequirement as R;
use crabomination::effect::{Effect, PlayerRef, Selector, Value};
use crabomination::game::effects::EffectContext;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::{Color, SpendRestriction, cost, generic};

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// CR 106.6 — "spend this mana only to cast instant, sorcery, Demon, and
/// Spirit spells": a sorcery and a Demon may use it, a Bear may not.
#[test]
fn cr_106_6_instant_sorcery_demon_or_spirit_mana() {
    let restricted = SpendRestriction::InstantSorceryOrTypes([CreatureType::Demon, CreatureType::Spirit]);
    let demon = || CardDefinition {
        name: "Test Demon",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    };
    let bear2 = || CardDefinition { name: "Test Bear", subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() }, ..demon() };
    let mut g = pod(2);
    let b = g.add_card_to_hand(0, bear2());
    g.players[0].mana_pool.add_restricted(Color::Black, 2, restricted);
    assert!(cast(&mut g, 0, b, None).is_err(), "a Bear can't spend it");
    let d = g.add_card_to_hand(0, demon());
    cast(&mut g, 0, d, None).expect("a Demon can");
    g.players[0].mana_pool.add_restricted(Color::Red, 1, restricted);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Player(1))).expect("an instant can");
}

/// CR 702.62 — "exile it with three time counters on it" as it resolves: the
/// card is suspended again, and the upkeeps tick it back into a free cast.
#[test]
fn cr_702_62_a_spell_that_suspends_itself() {
    let spell = || CardDefinition {
        name: "Test Sentence",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Suspend(3, cost(&[generic(1)]))],
        exile_on_resolve_time_counters: 3,
        effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        ..Default::default()
    };
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, spell());
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[1].life;
    cast(&mut g, 0, s, None).expect("cast");
    let exiled = g.exile.iter().find(|c| c.id == s).expect("exiled");
    assert_eq!(exiled.counter_count(CounterType::Time), 3);
    for _ in 0..3 {
        g.process_suspend();
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, life - 2, "cast again off its last counter");
}

/// Each player, the caster last, picks a creature the caster doesn't
/// control; the picks die together, the caster's own creatures never.
#[test]
fn each_player_chooses_a_creature_you_dont_control_to_destroy() {
    let mut g = pod(3);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::serra_angel());
    // Seat 1 picks first (its own Bears), seat 2 next (seat 1's Bears too),
    // the caster last (seat 2's Angel).
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Target(Target::Permanent(a)),
        DecisionAnswer::Target(Target::Permanent(a)),
        DecisionAnswer::Target(Target::Permanent(b)),
    ]));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::EachPlayerChoosesToDestroy { filter: R::Creature.and(R::ControlledByOpponent), starting_with_you: false },
        &ctx,
    )
    .expect("resolve");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert!(g.battlefield_find(mine).is_some());
}

// ── The cards ──

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn fcast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    cast(g, seat, id, target)
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn run(g: &mut GameState, seat: usize, e: &Effect) {
    let ctx = EffectContext::for_spell(seat, None, 0, 0);
    g.resolve_effect(e, &ctx).expect("resolve");
    drain_stack(g);
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
}

/// An opponent's first life loss on their own turn grows it and draws; the
/// second doesn't, nor a loss on your turn.
#[test]
fn valgavoth_feeds_on_first_losses() {
    let mut g = pod(2);
    let v = g.add_card_to_battlefield(0, catalog::valgavoth_harrower_of_souls());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    let bolt = |g: &mut GameState| {
        let b = g.add_card_to_hand(0, catalog::lightning_bolt());
        fcast(g, 0, b, Some(Target::Player(1))).expect("bolt");
    };
    bolt(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "not on your turn");
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    bolt(&mut g);
    bolt(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "once on their turn");
    assert_eq!(g.battlefield_find(v).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// While it's tapped, a land tapped for mana gives one more and pings.
#[test]
fn barbflare_gremlin_floods_and_burns() {
    let mut g = pod(2);
    let gr = g.add_card_to_battlefield(0, catalog::barbflare_gremlin());
    g.battlefield_find_mut(gr).unwrap().tapped = true;
    let m = g.add_card_to_battlefield(1, catalog::mountain());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility { card_id: m, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("tap");
    drain_stack(&mut g);
    assert_eq!(g.players[1].mana_pool.amount(Color::Red), 2);
    assert_eq!(g.players[1].life, life - 1);
}

/// At your end step: the enchantment's controller sacrifices it or burns.
#[test]
fn enchanters_bane_taxes_enchantments() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::enchanters_bane());
    let e = g.add_card_to_battlefield(1, catalog::goblin_oriflamme());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(false)]));
    let life = g.players[1].life;
    step(&mut g, TurnStep::End);
    assert_eq!(g.players[1].life, life - 2, "kept it, took its mana value");
    assert!(g.battlefield_find(e).is_some());
}

/// Geothermal Bog enters tapped; Leechridden Swamp drains with two black
/// permanents, not with one.
#[test]
fn the_lands() {
    let mut g = pod(2);
    let bog = g.add_card_to_hand(0, catalog::geothermal_bog());
    g.perform_action(GameAction::PlayLand(bog)).expect("play");
    assert!(g.battlefield_find(bog).unwrap().tapped);
    let s = g.add_card_to_battlefield(0, catalog::leechridden_swamp());
    let act = |g: &mut GameState| {
        flood(g, 0);
        g.battlefield_find_mut(s).unwrap().tapped = false;
        g.perform_action(GameAction::ActivateAbility { card_id: s, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
    };
    assert!(act(&mut g).is_err(), "one black permanent");
    g.add_card_to_battlefield(0, catalog::kederekt_parasite());
    g.add_card_to_battlefield(0, catalog::kederekt_parasite());
    let life = g.players[1].life;
    act(&mut g).expect("two black permanents");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
}

/// An opponent's noncreature spell burns them for its power.
#[test]
fn gleeful_arsonist_punishes_spells() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::gleeful_arsonist());
    g.active_player_idx = 1;
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    fcast(&mut g, 1, bolt, Some(Target::Player(0))).expect("cast");
    assert_eq!(g.players[1].life, life - 1);
}

/// An opponent drawing may take 1, with a red permanent around.
#[test]
fn kederekt_parasite_punishes_draws() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::kederekt_parasite());
    for _ in 0..6 {
        g.add_card_to_library(1, catalog::island());
    }
    g.active_player_idx = 1;
    let life = g.players[1].life;
    let d = g.add_card_to_hand(1, catalog::divination());
    fcast(&mut g, 1, d, None).expect("cast");
    assert_eq!(g.players[1].life, life, "no red permanent");
    g.add_card_to_battlefield(0, catalog::goblin_oriflamme());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 2]));
    let d = g.add_card_to_hand(1, catalog::divination());
    fcast(&mut g, 1, d, None).expect("cast");
    assert_eq!(g.players[1].life, life - 2, "one per card drawn");
}

/// An opponent's land costs them a life and grows it.
#[test]
fn nightshade_harvester_taxes_lands() {
    let mut g = pod(2);
    let n = g.add_card_to_battlefield(0, catalog::nightshade_harvester());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let life = g.players[1].life;
    let l = g.add_card_to_hand(1, catalog::island());
    g.perform_action(GameAction::PlayLand(l)).expect("play");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
    assert_eq!(g.battlefield_find(n).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Each opponent's upkeep: a life and a -1/-1 counter.
#[test]
fn persistent_constrictor_squeezes() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::persistent_constrictor());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    let life = g.players[1].life;
    step(&mut g, TurnStep::Upkeep);
    assert_eq!(g.players[1].life, life - 1);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
}

/// The Corridor's unlock makes three Devils; the Pit adds 2 to noncombat
/// damage at opponents.
#[test]
fn spiked_corridor_torture_pit_both_doors() {
    let mut g = pod(2);
    let room = g.add_card_to_hand(0, catalog::spiked_corridor_torture_pit());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false }).expect("cast the Corridor");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Devil").len(), 3);
    let r = named(&g, 0, "Spiked Corridor // Torture Pit")[0];
    flood(&mut g, 0);
    g.perform_action(GameAction::UnlockRoomDoor { card_id: r, right: true }).expect("unlock the Pit");
    drain_stack(&mut g);
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    fcast(&mut g, 0, bolt, Some(Target::Player(1))).expect("bolt");
    assert_eq!(g.players[1].life, life - 5);
}

/// Attacking: the permanent's controller sacrifices it or takes 5.
#[test]
fn star_athlete_forces_a_choice() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::star_athlete());
    let e = g.add_card_to_battlefield(1, catalog::goblin_oriflamme());
    g.clear_sickness(a);
    let life = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: a, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(e).is_none() || g.players[1].life == life - 5,
        "sacrificed, or 5 damage instead"
    );
}

/// Destroy an opponent's creature, they lose 3, and it suspends itself.
#[test]
fn suspended_sentence_returns() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::suspended_sentence());
    let life = g.players[1].life;
    fcast(&mut g, 0, s, Some(Target::Permanent(bear))).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[1].life, life - 3);
    assert_eq!(g.exile.iter().find(|c| c.id == s).unwrap().counter_count(CounterType::Time), 3);
}

/// Morbid end steps add soul counters; it taps for that much restricted mana.
#[test]
fn seance_board_counts_the_dead() {
    let mut g = pod(2);
    let b = g.add_card_to_battlefield(0, catalog::seance_board());
    step(&mut g, TurnStep::End);
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::Soul), 0, "nothing died");
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    fcast(&mut g, 0, bolt, Some(Target::Permanent(bear))).expect("bolt");
    g.players[0].mana_pool = Default::default();
    step(&mut g, TurnStep::End);
    step(&mut g, TurnStep::End);
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::Soul), 2);
    g.perform_action(GameAction::ActivateAbility { card_id: b, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("tap");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2);
}

/// Opponents can't gain life; a player's first spell burns another player.
#[test]
fn the_lord_of_pain_punishes() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::the_lord_of_pain());
    let life = g.players[1].life;
    run(&mut g, 1, &Effect::GainLife { who: Selector::You, amount: Value::Const(5) });
    assert_eq!(g.players[1].life, life, "an opponent can't gain life");
    g.active_player_idx = 1;
    let (l0, l1, l2) = (g.players[0].life, g.players[1].life, g.players[2].life);
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    fcast(&mut g, 1, bear, None).expect("cast");
    assert_eq!(g.players[1].life, l1, "not the caster");
    assert_eq!(l0 + l2 - g.players[0].life - g.players[2].life, 2, "another player took 2");
}

/// Enters with ward—pay 2 life; the shell game is the effect already tested.
#[test]
fn sadistic_shell_game_casts() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::sadistic_shell_game());
    fcast(&mut g, 0, s, None).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    assert!(matches!(
        catalog::valgavoth_harrower_of_souls().keywords.iter().find(|k| matches!(k, Keyword::Ward(_))),
        Some(Keyword::Ward(crabomination::card::WardCost::Life(2)))
    ));
}
