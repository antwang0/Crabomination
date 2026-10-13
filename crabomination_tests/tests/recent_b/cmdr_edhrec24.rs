//! Commander: the 2026-10-13 rescan's two-card gaps (`decks::cmdr_edhrec24`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
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

fn cast(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
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

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let cp = g.computed_permanent(id).expect("on the battlefield");
    (cp.power, cp.toughness)
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

/// Swing `attacker` at seat 1 and run combat damage with its triggers.
fn hit(g: &mut GameState, attacker: CardId) {
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

/// The Widows: a deathtouch creature's hit draws (Black Widow) and grows
/// (White Widow); a creature without deathtouch does neither.
#[test]
fn widows_reward_deathtouch_hits() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    ready(&mut g, 0, catalog::black_widow_deadly_hunter());
    let white = ready(&mut g, 0, catalog::white_widow_yelena_belova());
    hit(&mut g, white);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].life, 19);
    assert_eq!(g.battlefield_find(white).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let mut g = pod(2);
    ready(&mut g, 0, catalog::black_widow_deadly_hunter());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    hit(&mut g, bear);
    assert_eq!(g.players[0].life, 20, "no deathtouch, no trigger");
}

/// Gnoll War Band costs {1} less per opponent dealt damage this turn.
#[test]
fn gnoll_war_band_discount() {
    let mut g = pod(3);
    for s in [1, 2] {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, 0, bolt, &[Target::Player(s)]).expect("bolt");
    }
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let gnolls = g.add_card_to_hand(0, catalog::gnoll_war_band());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: gnolls, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{3}{R} after two opponents took damage");
}

/// Genasi Enforcers pumps every creature you control by that name.
#[test]
fn genasi_enforcers_pump_by_name() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::genasi_enforcers());
    let b = ready(&mut g, 0, catalog::genasi_enforcers());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, a, 0, None).expect("pump");
    assert_eq!((pt(&g, a), pt(&g, b), pt(&g, bear)), ((2, 3), (2, 3), (2, 2)));
}

/// Godsire makes an 8/8; Knotvine Mystic turns {1} into {R}{G}{W}.
#[test]
fn godsire_and_knotvine_mystic() {
    let mut g = pod(2);
    let sire = ready(&mut g, 0, catalog::godsire());
    activate(&mut g, sire, 0, None).expect("beast");
    let beast = g.battlefield.iter().find(|c| c.is_token).expect("token").id;
    assert_eq!(pt(&g, beast), (8, 8));
    let mystic = ready(&mut g, 0, catalog::knotvine_mystic());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mystic,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("mana");
    let pool = &g.players[0].mana_pool;
    assert_eq!((pool.amount(Color::Red), pool.amount(Color::Green), pool.amount(Color::White)), (1, 1, 1));
}

/// Ajani, Adversary of Tyrants: +1 on two creatures, −2 reanimates a cheap
/// creature.
#[test]
fn ajani_adversary_counters_and_reanimates() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    let ajani = ready(&mut g, 0, catalog::ajani_adversary_of_tyrants());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ajani,
        ability_index: 0,
        target: Some(Target::Permanent(a)),
        x_value: None,
    })
    .expect("+1");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let _ = b;
    let mut g = pod(2);
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ajani = ready(&mut g, 0, catalog::ajani_adversary_of_tyrants());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ajani,
        ability_index: 1,
        target: Some(Target::Permanent(dead)),
        x_value: None,
    })
    .expect("-2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some());
}

/// Pilfering Imp takes a nonland card from an opponent's hand.
#[test]
fn pilfering_imp_strips_a_spell() {
    let mut g = pod(2);
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let imp = ready(&mut g, 0, catalog::pilfering_imp());
    activate(&mut g, imp, 0, Some(Target::Player(1))).expect("imp");
    assert_eq!(g.players[1].hand.len(), 1);
    assert_eq!(g.players[1].hand[0].definition.name, "Island");
    assert!(g.battlefield_find(imp).is_none(), "sacrificed");
}

/// Unified Will counters only with more creatures than the caster.
#[test]
fn unified_will_needs_the_bigger_board() {
    for (mine, countered) in [(2, true), (0, false)] {
        let mut g = pod(2);
        for _ in 0..mine {
            ready(&mut g, 0, catalog::grizzly_bears());
        }
        ready(&mut g, 1, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        flood(&mut g, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        let will = g.add_card_to_hand(0, catalog::unified_will());
        cast(&mut g, 0, will, &[Target::Permanent(bolt)]).expect("will");
        assert_eq!(g.players[0].life == 20, countered, "{mine} creatures");
    }
}

/// Circle of the Land Druid returns a land card when it dies.
#[test]
fn circle_of_the_land_druid_regrows_a_land() {
    let mut g = pod(2);
    let land = g.add_card_to_graveyard(0, catalog::forest());
    let druid = ready(&mut g, 0, catalog::circle_of_the_land_druid());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, &[Target::Permanent(druid)]).expect("bolt");
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == land));
}

/// Boar Umbra: +3/+3, and umbra armor saves the creature.
#[test]
fn boar_umbra_pumps_and_protects() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let umbra = g.add_card_to_hand(0, catalog::boar_umbra());
    cast(&mut g, 0, umbra, &[Target::Permanent(bear)]).expect("cast");
    assert_eq!(pt(&g, bear), (5, 5));
    let kill = g.add_card_to_hand(1, catalog::murder());
    cast(&mut g, 1, kill, &[Target::Permanent(bear)]).expect("murder");
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_some(), "umbra armor");
    assert!(g.battlefield_find(umbra).is_none());
}

/// Feywild Visitor lends your commander a Faerie Dragon trigger on a
/// nontoken hit.
#[test]
fn feywild_visitor_grants_the_commander() {
    let mut g = pod(2);
    let cmdr = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmdr);
    ready(&mut g, 0, catalog::feywild_visitor());
    hit(&mut g, cmdr);
    assert_eq!(named(&g, "Faerie Dragon"), 1);
}

/// Boggart Cursecrafter pings each opponent when another Goblin dies.
#[test]
fn boggart_cursecrafter_pings_on_goblin_deaths() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::boggart_cursecrafter());
    let gob = ready(&mut g, 0, catalog::prickly_boggart());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, &[Target::Permanent(gob)]).expect("bolt");
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!((g.players[1].life, g.players[2].life), (19, 19));
}

/// Thieves' Guild Enforcer mills each opponent two and turns on at eight.
#[test]
fn thieves_guild_enforcer_mills_and_grows() {
    let mut g = pod(3);
    for s in [1, 2] {
        for _ in 0..8 {
            g.add_card_to_library(s, catalog::island());
        }
    }
    for _ in 0..6 {
        g.add_card_to_graveyard(1, catalog::island());
    }
    let tge = g.add_card_to_hand(0, catalog::thieves_guild_enforcer());
    cast(&mut g, 0, tge, &[]).expect("cast");
    assert_eq!((g.players[1].graveyard.len(), g.players[2].graveyard.len()), (8, 2));
    assert_eq!(pt(&g, tge), (3, 2));
    assert!(g.computed_permanent(tge).unwrap().keywords().contains(&Keyword::Deathtouch));
}

/// Grim Return takes a creature that died this turn, not an older one.
#[test]
fn grim_return_only_this_turns_dead() {
    let mut g = pod(2);
    let old = g.add_card_to_graveyard(1, catalog::hill_giant());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, &[Target::Permanent(bear)]).expect("bolt");
    g.check_state_based_actions();
    let gr = g.add_card_to_hand(0, catalog::grim_return());
    assert!(cast(&mut g, 0, gr, &[Target::Permanent(old)]).is_err(), "died earlier");
    cast(&mut g, 0, gr, &[Target::Permanent(bear)]).expect("this turn's");
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
}

/// Sturmgeist's size is your hand size; Mass Appeal draws per Human.
#[test]
fn sturmgeist_and_mass_appeal() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    let geist = ready(&mut g, 0, catalog::sturmgeist());
    assert_eq!(pt(&g, geist), (3, 3));
    for _ in 0..2 {
        ready(&mut g, 0, catalog::thieves_guild_enforcer());
        g.add_card_to_library(0, catalog::island());
    }
    let appeal = g.add_card_to_hand(0, catalog::mass_appeal());
    cast(&mut g, 0, appeal, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), 5);
}

/// Rush of Blood doubles a creature's power.
#[test]
fn rush_of_blood_doubles_power() {
    let mut g = pod(2);
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let id = g.add_card_to_hand(0, catalog::rush_of_blood());
    cast(&mut g, 0, id, &[Target::Permanent(giant)]).expect("cast");
    assert_eq!(pt(&g, giant), (6, 3));
}

/// Enlightened Ascetic may destroy an enchantment.
#[test]
fn enlightened_ascetic_breaks_an_enchantment() {
    let mut g = pod(2);
    let ench = ready(&mut g, 1, catalog::boar_umbra());
    let id = g.add_card_to_hand(0, catalog::enlightened_ascetic());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, 0, id, &[Target::Permanent(ench)]).expect("cast");
    assert!(g.battlefield_find(ench).is_none());
}


/// Bug fix (CR 603.2, 603.3): a statics-granted "whenever one or more
/// creatures you control deal combat damage to a player" listener never fired
/// — combat damage dispatch walked printed listeners only. Two attackers
/// connecting fire the once-per-batch grant once.
#[test]
fn a_granted_your_control_combat_listener_fires_once_per_batch() {
    let mut g = pod(2);
    let cmdr = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmdr);
    let other = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::feywild_visitor());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: cmdr, target: AttackTarget::Player(1) },
        Attack { attacker: other, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(named(&g, "Faerie Dragon"), 1);
}
