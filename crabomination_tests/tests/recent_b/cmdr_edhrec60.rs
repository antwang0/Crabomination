//! Commander: EDHREC ranks 2001-2500 — Kid Loki and the easy two-card gaps
//! (`decks::cmdr_edhrec60`).

use crabomination::card::{CardId, CounterType, Keyword};
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

fn fill_pool(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
    g.priority.player_with_priority = 0;
}

fn cast_at(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    fill_pool(g);
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    fill_pool(g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn stock(g: &mut GameState, seats: usize) {
    for seat in 0..seats {
        for _ in 0..4 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// Kid Loki: a creature you put a +1/+1 counter on this turn has hexproof
/// (Basri's Solidarity puts one on each); a second draw grows Loki.
#[test]
fn kid_loki_hides_freshly_countered_creatures() {
    let mut g = pod(2);
    stock(&mut g, 1);
    let loki = ready(&mut g, 0, catalog::kid_loki());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Hexproof));
    let spell = g.add_card_to_hand(0, catalog::basris_solidarity());
    cast_at(&mut g, spell, None).expect("solidarity");
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Hexproof));
    let before = g.battlefield_find(loki).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(loki).unwrap().counter_count(CounterType::PlusOnePlusOne), before + 1);
}

/// Thopter Mechanic leaves a Thopter; Pym Particles makes a creature
/// unblockable and draws.
#[test]
fn thopter_mechanic_and_pym_particles() {
    let mut g = pod(2);
    stock(&mut g, 1);
    let mech = ready(&mut g, 0, catalog::thopter_mechanic());
    let mut evs = Vec::new();
    g.destroy_permanent(mech, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Thopter"), 1);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let pym = g.add_card_to_hand(0, catalog::pym_particles());
    let hand = g.players[0].hand.len();
    cast_at(&mut g, pym, Some(Target::Permanent(bears))).expect("pym");
    let kws = g.computed_permanent(bears).unwrap().keywords().to_vec();
    assert!(kws.contains(&Keyword::Unblockable) && kws.contains(&Keyword::Vigilance));
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
}

/// Chaplain of Alms has ward {1}; cast with disturb it is Chapel Shieldgeist,
/// giving your creatures ward {1} (CR 702.146).
#[test]
fn chaplain_of_alms_disturbs_into_a_shieldgeist() {
    let mut g = pod(2);
    let chap = g.add_card_to_graveyard(0, catalog::chaplain_of_alms());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    fill_pool(&mut g);
    g.perform_action(GameAction::CastDisturb { card_id: chap, target: None, additional_targets: vec![] }).expect("disturb");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(chap).unwrap().def.name, "Chapel Shieldgeist");
    assert!(g.computed_permanent(bears).unwrap().keywords().iter().any(|k| matches!(k, Keyword::Ward(_))));
}

/// Clockwork Fox draws on leaving; Silverchase Fox exiles an enchantment.
#[test]
fn the_kitsune_foxes() {
    let mut g = pod(2);
    stock(&mut g, 2);
    let fox = ready(&mut g, 0, catalog::clockwork_fox());
    let (mine, theirs) = (g.players[0].hand.len(), g.players[1].hand.len());
    let mut evs = Vec::new();
    g.destroy_permanent(fox, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), mine + 2);
    assert_eq!(g.players[1].hand.len(), theirs + 1);

    let silver = ready(&mut g, 0, catalog::silverchase_fox());
    let ench = ready(&mut g, 1, catalog::rage_reflection());
    activate(&mut g, silver, 0, Some(Target::Permanent(ench))).expect("exile");
    assert!(g.exile.iter().any(|c| c.id == ench));
}

/// Rank Rat and Stony-Voiced Goblins strip a card from each opponent.
#[test]
fn rank_rat_and_stony_voiced_goblins_discard() {
    let mut g = pod(3);
    for seat in 1..3 {
        g.add_card_to_hand(seat, catalog::island());
    }
    let rat = g.add_card_to_hand(0, catalog::rank_rat());
    cast_at(&mut g, rat, None).expect("rat");
    assert!(g.players[1].hand.is_empty() && g.players[2].hand.is_empty());
    let gob = g.add_card_to_hand(0, catalog::stony_voiced_goblins());
    cast_at(&mut g, gob, None).expect("goblins");
}

/// Ruthless Knave turns a creature into two Treasures and three Treasures
/// into a card.
#[test]
fn ruthless_knave_trades_in_treasures() {
    let mut g = pod(2);
    stock(&mut g, 1);
    let knave = ready(&mut g, 0, catalog::ruthless_knave());
    ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, knave, 0, None).expect("two treasures");
    assert_eq!(named(&g, 0, "Treasure"), 2);
    ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, knave, 0, None).expect("two more");
    let hand = g.players[0].hand.len();
    activate(&mut g, knave, 1, None).expect("draw");
    assert_eq!(named(&g, 0, "Treasure"), 1);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Cloak and Dagger jumps onto an entering Rogue and gives it shroud.
#[test]
fn cloak_and_dagger_follows_rogues() {
    let mut g = pod(2);
    let cloak = ready(&mut g, 0, catalog::cloak_and_dagger());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let rogue = g.add_card_to_hand(0, catalog::zhentarim_bandit());
    cast_at(&mut g, rogue, None).expect("rogue");
    assert_eq!(g.battlefield_find(cloak).unwrap().attached_to, Some(rogue));
    let cp = g.computed_permanent(rogue).unwrap();
    assert!(cp.keywords().contains(&Keyword::Shroud));
    assert_eq!(cp.power, 4);
}

/// Firemind's Research banks charge counters off instants and sorceries;
/// Ionize counters a spell and burns its caster.
#[test]
fn fireminds_research_and_ionize() {
    let mut g = pod(2);
    let research = ready(&mut g, 0, catalog::fireminds_research());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    assert_eq!(g.battlefield_find(research).unwrap().counter_count(CounterType::Charge), 1);

    let theirs = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: theirs, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("their bears");
    let life = g.players[1].life;
    let ionize = g.add_card_to_hand(0, catalog::ionize());
    fill_pool(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: ionize,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("ionize");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none());
    assert_eq!(g.players[1].life, life - 2);
}
