//! Commander: the Adaptive Enchantment precon (C18, Estrid,
//! `decks::cmdr_estrid`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn act_as(g: &mut GameState, seat: usize, action: GameAction) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_as(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    act_as(g, seat, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_as(g, 0, id, targets)
}

fn cast_commander(g: &mut GameState, id: CardId) -> Result<(), String> {
    act_as(g, 0, GameAction::CastFromCommandZone {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::plains());
    }
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// CR 903.9a/b — Myth Unbound draws whenever your commander goes to the
/// command zone, by the state-based action (from the graveyard) and by the
/// replacement (instead of your hand); and CR 903.8's tax is offset {1} per
/// earlier cast.
#[test]
fn cr_903_9_myth_unbound_draws_on_both_routes_home() {
    let mut g = main_phase(2);
    library(&mut g, 0, 5);
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    g.add_card_to_battlefield(0, catalog::myth_unbound());
    cast_commander(&mut g, cmd).expect("first cast");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[Target::Permanent(cmd)]).expect("bolt the commander");
    assert!(g.players[0].command.iter().any(|c| c.id == cmd), "home by the SBA");
    assert_eq!(g.players[0].hand.len(), 1, "drew for the 903.9a return");
    // Second cast: {1}{G} + {2} tax − {1} Myth Unbound = four mana.
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add(Color::Green, 4);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("taxed {2}, discounted {1}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cmd).is_some());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let unsummon = g.add_card_to_hand(1, catalog::unsummon());
    cast_as(&mut g, 1, unsummon, &[Target::Permanent(cmd)]).expect("bounce the commander");
    assert!(g.players[0].command.iter().any(|c| c.id == cmd), "home by the replacement");
    assert_eq!(g.players[0].hand.len(), 2, "drew for the 903.9b return");
}

/// Empyrial Storm is copied once per cast of your commander from the command
/// zone.
#[test]
fn empyrial_storm_copies_per_commander_cast() {
    let mut g = main_phase(2);
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    cast_commander(&mut g, cmd).expect("cast");
    let es = g.add_card_to_hand(0, catalog::empyrial_storm());
    cast_at(&mut g, es, &[]).expect("storm");
    assert_eq!(named(&g, 0, "Angel"), 2, "the original and one copy");
}

/// Arixmethes is a tapped land with slumber counters; each spell may wake it a
/// counter at a time.
#[test]
fn arixmethes_sleeps_until_the_counters_are_gone() {
    let mut g = main_phase(2);
    let ax = g.add_card_to_hand(0, catalog::arixmethes_slumbering_isle());
    cast_at(&mut g, ax, &[]).expect("cast");
    let c = g.computed_permanent(ax).unwrap();
    assert!(c.card_types().contains(&crabomination::card::CardType::Land));
    assert!(!c.card_types().contains(&crabomination::card::CardType::Creature));
    assert!(g.battlefield_find(ax).unwrap().tapped);
    g.battlefield_find_mut(ax).unwrap().remove_counters(CounterType::Slumber, 4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, bear, &[]).expect("a spell");
    let c = g.computed_permanent(ax).unwrap();
    assert!(c.card_types().contains(&crabomination::card::CardType::Creature));
    assert_eq!((c.power, c.toughness), (12, 12));
}

/// Elderwood Scion: your spells aimed at it are {2} cheaper, an opponent's
/// {2} dearer.
#[test]
fn elderwood_scion_bends_costs_both_ways() {
    let mut g = main_phase(2);
    let es = g.add_card_to_battlefield(0, catalog::elderwood_scion());
    let aura = g.add_card_to_hand(0, catalog::epic_proportions());
    g.players[0].mana_pool.add(Color::Green, 4);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(es)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("six less two");
    drain_stack(&mut g);
    assert_eq!(pt(&g, es), (9, 9));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let taxed = g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(es)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(taxed.is_err(), "Bolt needs {{2}} more");
}

/// Ravenous Slime exiles an opponent's dying creature — token or not — and
/// grows by its power.
#[test]
fn ravenous_slime_eats_the_dead() {
    let mut g = main_phase(2);
    let slime = g.add_card_to_battlefield(0, catalog::ravenous_slime());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, &[Target::Permanent(giant)]).expect("bolt");
    assert!(g.exile.iter().any(|c| c.id == giant) && g.players[1].graveyard.is_empty());
    assert_eq!(g.battlefield_find(slime).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Tuvasa draws for the first enchantment spell each turn only.
#[test]
fn tuvasa_draws_on_the_first_enchantment_spell() {
    let mut g = main_phase(2);
    library(&mut g, 0, 4);
    let tv = g.add_card_to_battlefield(0, catalog::tuvasa_the_sunlit());
    for _ in 0..2 {
        let mu = g.add_card_to_hand(0, catalog::myth_unbound());
        cast_at(&mut g, mu, &[]).expect("enchantment");
    }
    assert_eq!(g.players[0].hand.len(), 1, "one draw");
    assert_eq!(pt(&g, tv), (3, 3));
}

/// Bruna gathers your Auras onto itself as it attacks — from the battlefield,
/// the graveyard and hand.
#[test]
fn bruna_gathers_auras_when_attacking() {
    let mut g = main_phase(2);
    let bruna = g.add_card_to_battlefield(0, catalog::bruna_light_of_alabaster());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eel = g.add_card_to_hand(0, catalog::eel_umbra());
    cast_at(&mut g, eel, &[Target::Permanent(bear)]).expect("umbra the bear");
    let epic = g.add_card_to_hand(0, catalog::epic_proportions());
    g.clear_sickness(bruna);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bruna, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(eel).unwrap().attached_to, Some(bruna));
    assert_eq!(g.battlefield_find(epic).unwrap().attached_to, Some(bruna));
    assert_eq!(pt(&g, bruna), (11, 11));
}

/// Heavenly Blademaster collects your attachments and pumps the rest of the
/// team per attachment.
#[test]
fn heavenly_blademaster_collects_and_shares() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eel = g.add_card_to_hand(0, catalog::eel_umbra());
    cast_at(&mut g, eel, &[Target::Permanent(bear)]).expect("umbra the bear");
    let hb = g.add_card_to_hand(0, catalog::heavenly_blademaster());
    cast_at(&mut g, hb, &[]).expect("cast");
    assert_eq!(g.battlefield_find(eel).unwrap().attached_to, Some(hb));
    assert_eq!(pt(&g, bear), (3, 3), "+1/+1 for the one attachment");
}

/// Estrid: −1 masks a permanent with an umbra-armored Aura token; +2 untaps
/// what's enchanted.
#[test]
fn estrid_masks_and_untaps() {
    let mut g = main_phase(2);
    let es = g.add_card_to_battlefield(0, catalog::estrid_the_masked());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    act_as(&mut g, 0, GameAction::ActivateLoyaltyAbility {
        card_id: es,
        ability_index: 1,
        target: Some(Target::Permanent(bear)),
        x_value: None,
    })
    .expect("−1");
    let mask = g.battlefield.iter().find(|c| c.definition.name == "Mask").expect("the Mask");
    assert_eq!(mask.attached_to, Some(bear));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[Target::Permanent(bear)]).expect("bolt");
    assert!(g.battlefield_find(bear).is_some(), "umbra armor");
    assert_eq!(named(&g, 0, "Mask"), 0);
}

/// Loyal Unicorn (lieutenant): with your commander out, your creatures take no
/// combat damage this turn.
#[test]
fn loyal_unicorn_shields_with_the_commander_out() {
    let mut g = main_phase(2);
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    cast_commander(&mut g, cmd).expect("commander");
    g.add_card_to_battlefield(0, catalog::loyal_unicorn());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[Target::Permanent(cmd)]).expect("noncombat damage still lands");
    assert!(g.battlefield_find(cmd).is_none());
}

/// Nylea's Colossus doubles a creature when an enchantment enters.
#[test]
fn nyleas_colossus_doubles_on_constellation() {
    let mut g = main_phase(2);
    let nc = g.add_card_to_hand(0, catalog::nyleas_colossus());
    cast_at(&mut g, nc, &[]).expect("cast");
    assert_eq!(pt(&g, nc), (12, 12), "it is itself the entering enchantment");
}

/// Octopus Umbra sets base 8/8.
#[test]
fn octopus_umbra_makes_an_eight_eight() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ou = g.add_card_to_hand(0, catalog::octopus_umbra());
    cast_at(&mut g, ou, &[Target::Permanent(bear)]).expect("cast");
    assert_eq!(pt(&g, bear), (8, 8));
}

/// Creeping Renaissance returns every card of the chosen type.
#[test]
fn creeping_renaissance_regrows_a_type() {
    let mut g = main_phase(2);
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.add_card_to_graveyard(0, catalog::plains());
    let cr = g.add_card_to_hand(0, catalog::creeping_renaissance());
    act_as(&mut g, 0, GameAction::CastSpell { card_id: cr, target: None, additional_targets: vec![], mode: Some(0), x_value: None })
        .expect("creature mode");
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(g.players[0].graveyard.len(), 2, "the Plains and the Renaissance");
}
