//! Commander: the Riveteers Rampage precon (NCC, Henzie, `decks::cmdr_henzie`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = if n == 2 { two_player_game() } else { multi_player_game(n) };
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn attack(g: &mut GameState, attackers: &[(CardId, usize)]) -> Result<(), GameError> {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&(a, p)| Attack { attacker: a, target: AttackTarget::Player(p) }).collect(),
    ))?;
    drain_stack(g);
    Ok(())
}

fn finish_combat(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn end_step(g: &mut GameState) {
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// Henzie (CR 903.3) grants blitz at the spell's mana cost to a creature
/// spell of mana value 4+ (CR 702.152), {1} less per commander cast.
#[test]
fn henzie_grants_discounted_blitz() {
    assert!(catalog::henzie_toolbox_torre().can_be_commander);
    let mut g = pod(2);
    let h = g.add_card_to_battlefield(0, catalog::henzie_toolbox_torre());
    g.players[0].commanders.push(h);
    g.commander_cast_count.insert(h, 2);
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let alt = |g: &mut GameState, id: CardId| {
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: id,
            pitch_card: None,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    g.players[0].mana_pool.add(Color::Green, 2);
    assert!(alt(&mut g, bear).is_err(), "mana value 2: no blitz");
    // {4}{G}{G} less {2}: exactly four mana.
    g.players[0].mana_pool.add_colorless(2);
    alt(&mut g, wurm).expect("blitz");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_some_and(|c| c.blitzed));
}

/// Jolene: an attack on your opponent makes the attacker a Treasure, and your
/// own Treasure creations make one more.
#[test]
fn jolene_plunders() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::jolene_the_plunder_queen());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    attack(&mut g, &[(bear, 1)]).expect("attack");
    assert_eq!(named(&g, 0, "Treasure"), 2, "the attack's Treasure plus Jolene's");
}

/// Grime Gorger exiles one card per card type from the defender's graveyard
/// (CR 205.2a: a card of two types stands for one) and grows per card.
#[test]
fn grime_gorger_eats_one_per_type() {
    let mut g = pod(2);
    let gg = g.add_card_to_battlefield(0, catalog::grime_gorger());
    for f in [catalog::grizzly_bears, catalog::craw_wurm, catalog::lightning_bolt, catalog::forest] {
        g.add_card_to_graveyard(1, f());
    }
    g.add_card_to_graveyard(1, catalog::ornithopter());
    g.clear_sickness(gg);
    attack(&mut g, &[(gg, 1)]).expect("attack");
    // creature, instant, land, and the artifact creature as artifact.
    assert_eq!(g.battlefield_find(gg).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
    assert_eq!(g.players[1].graveyard.len(), 1);
}

/// Weathered Sentinels (CR 508.1a) attacks only a player who attacked you
/// during their last turn.
#[test]
fn weathered_sentinels_answers_attackers() {
    let mut g = pod(3);
    let ws = g.add_card_to_battlefield(0, catalog::weathered_sentinels());
    g.clear_sickness(ws);
    assert!(attack(&mut g, &[(ws, 1)]).is_err(), "no one attacked us");
    g.players[2].attacked_players_this_turn.push(0);
    assert!(attack(&mut g, &[(ws, 1)]).is_err(), "seat 1 didn't attack us");
    attack(&mut g, &[(ws, 2)]).expect("seat 2 did");
    assert_eq!(g.computed_permanent(ws).map(|c| c.power), Some(5));
}

/// Turf War: combat damage takes one of the damaged player's contested lands.
#[test]
fn turf_war_takes_contested_land() {
    let mut g = pod(2);
    let theirs = g.add_card_to_battlefield(1, catalog::forest());
    g.add_card_to_battlefield(0, catalog::mountain());
    let tw = g.add_card_to_hand(0, catalog::turf_war());
    cast(&mut g, tw, &[]);
    assert_eq!(g.battlefield_find(theirs).unwrap().counter_count(CounterType::Contested), 1);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    attack(&mut g, &[(bear, 1)]).expect("attack");
    finish_combat(&mut g);
    assert_eq!(g.battlefield_find(theirs).map(|c| c.controller), Some(0));
}

/// Bellowing Mauler: at your end step a player without a nontoken creature
/// to sacrifice loses 4.
#[test]
fn bellowing_mauler_taxes() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::bellowing_mauler());
    end_step(&mut g);
    assert_eq!(g.players[1].starting_life - g.players[1].life, 4);
}

/// Wave of Rats returns only after it dealt combat damage to a player.
#[test]
fn wave_of_rats_returns_after_hitting() {
    let mut g = pod(2);
    let w = g.add_card_to_battlefield(0, catalog::wave_of_rats());
    g.clear_sickness(w);
    attack(&mut g, &[(w, 1)]).expect("attack");
    finish_combat(&mut g);
    let m = g.add_card_to_hand(0, catalog::murder());
    g.step = TurnStep::PostCombatMain;
    cast(&mut g, m, &[Target::Permanent(w)]);
    assert!(g.battlefield_find(w).is_some());
    let m = g.add_card_to_hand(0, catalog::murder());
    let w2 = g.add_card_to_battlefield(0, catalog::wave_of_rats());
    cast(&mut g, m, &[Target::Permanent(w2)]);
    assert!(g.battlefield_find(w2).is_none());
}

/// Kresh grows by a dying creature's last-known power (CR 603.10).
#[test]
fn kresh_feeds() {
    let mut g = pod(2);
    let k = g.add_card_to_battlefield(0, catalog::kresh_the_bloodbraided());
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(wurm)]);
    assert_eq!(g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne), 6);
}

/// Caldaia Guardian: a mana value 4+ creature of yours dying makes two
/// Citizens; a two-drop doesn't.
#[test]
fn caldaia_guardian_makes_citizens() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::caldaia_guardian());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(bear)]);
    assert_eq!(named(&g, 0, "Citizen"), 0);
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(wurm)]);
    assert_eq!(named(&g, 0, "Citizen"), 2);
}

/// The Beamtown Bullies hand a graveyard creature to the opponent whose turn
/// it is, goaded (CR 701.15), exiled at the next end step.
#[test]
fn beamtown_bullies_loan_a_creature() {
    let mut g = pod(3);
    let bb = g.add_card_to_battlefield(0, catalog::the_beamtown_bullies());
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bb,
        ability_index: 0,
        target: Some(Target::Permanent(wurm)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let c = g.battlefield_find(wurm).expect("loaned");
    assert_eq!(c.controller, 1);
    assert!(g.is_goaded(c));
    end_step(&mut g);
    assert!(g.exile.iter().any(|c| c.id == wurm));
}

/// Industrial Advancement trades a creature for a look at its mana value in
/// cards, deploying a creature from among them.
#[test]
fn industrial_advancement_digs() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::industrial_advancement());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_library(0, catalog::craw_wurm());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    end_step(&mut g);
    assert!(g.battlefield_find(wurm).is_some());
}
