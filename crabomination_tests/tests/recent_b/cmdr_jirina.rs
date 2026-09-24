//! Commander: the Ruthless Regiment precon (C20, Jirina Kudro,
//! `decks::cmdr_jirina`).

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
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

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    act_as(g, 0, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    act_as(g, 0, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn kw(g: &GameState, id: CardId, k: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&k))
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn bolt(g: &mut GameState, at: CardId) {
    let b = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(g, b, &[Target::Permanent(at)]).expect("bolt");
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let decl = attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect();
    g.perform_action(GameAction::DeclareAttackers(decl)).expect("attack");
    drain_stack(g);
}

/// Jirina, cast from the command zone, counts that very cast (2020-04-17
/// ruling): one Human Soldier, which gets +2/+0 from her.
#[test]
fn jirina_counts_her_own_cast_and_pumps_the_token() {
    let mut g = main_phase(4);
    g.seat_commanders(0, vec![catalog::jirina_kudro()]);
    let id = g.players[0].commanders[0];
    act_as(&mut g, 0, GameAction::CastFromCommandZone {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast the commander");
    let tokens = named(&g, 0, "Human Soldier");
    assert_eq!(tokens.len(), 1);
    assert_eq!(pt(&g, tokens[0]), (3, 1));
    assert_eq!(pt(&g, id), (3, 3), "not her own anthem");
}

/// CR 614.1c — Dearly Departed in your graveyard puts a +1/+1 counter on
/// each Human creature entering under you (one per copy), not on others.
#[test]
fn cr_614_1c_dearly_departed_counters_humans_from_the_graveyard() {
    let mut g = main_phase(2);
    g.add_card_to_graveyard(0, catalog::dearly_departed());
    g.add_card_to_graveyard(0, catalog::dearly_departed());
    let human = g.add_card_to_hand(0, catalog::bounty_agent());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, human, &[]).expect("cast");
    cast_at(&mut g, bear, &[]).expect("cast");
    assert_eq!(g.battlefield_find(human).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    // A Human token made on the battlefield takes it too.
    let doom = g.add_card_to_battlefield(0, catalog::thraben_doomsayer());
    g.clear_sickness(doom);
    activate(&mut g, doom, 0, &[]).expect("make a Human");
    let token = named(&g, 0, "Human")[0];
    assert_eq!(pt(&g, token), (3, 3));
}

/// CR 702.16 — Riders of Gavony's Humans have protection from the chosen
/// type: a creature of that type can't block them.
#[test]
fn cr_702_16_riders_of_gavony_protects_humans_from_the_chosen_type() {
    let mut g = main_phase(2);
    let riders = g.add_card_to_battlefield(0, catalog::riders_of_gavony());
    g.battlefield_find_mut(riders).unwrap().chosen_creature_type = Some(CreatureType::Bear);
    let agent = g.add_card_to_battlefield(0, catalog::bounty_agent());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(kw(&g, agent, Keyword::ProtectionFromCreatureType(CreatureType::Bear)));
    assert!(kw(&g, riders, Keyword::ProtectionFromCreatureType(CreatureType::Bear)));
    assert!(!kw(&g, bear, Keyword::ProtectionFromCreatureType(CreatureType::Bear)), "not a Human");
}

/// Riders of Gavony, cast by a pod seat, names the type the opponents field
/// (the as-enters ask is driven off the stack, where the headless decider
/// used to name Demon).
#[test]
fn riders_of_gavony_names_an_opposing_type() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::bounty_agent());
    let riders = g.add_card_to_hand(0, catalog::riders_of_gavony());
    g.players[0].wants_ui = true;
    cast_at(&mut g, riders, &[]).expect("cast");
    assert_eq!(g.battlefield_find(riders).unwrap().chosen_creature_type, Some(CreatureType::Bear));
}

/// Kelsien: the ping kills, and the delayed trigger hands out experience,
/// which grows Kelsien.
#[test]
fn kelsien_earns_experience_from_a_kill() {
    let mut g = main_phase(2);
    let k = g.add_card_to_battlefield(0, catalog::kelsien_the_plague());
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    activate(&mut g, k, 0, &[Target::Permanent(elf)]).expect("ping");
    assert!(g.battlefield_find(elf).is_none());
    assert_eq!(g.players[0].experience, 1);
    assert_eq!(pt(&g, k), (3, 3));
}

/// Xathrid Necromancer: a tapped Zombie when a Human of yours dies (itself
/// included), none for a non-Human.
#[test]
fn xathrid_necromancer_raises_dead_humans() {
    let mut g = main_phase(2);
    let x = g.add_card_to_battlefield(0, catalog::xathrid_necromancer());
    let agent = g.add_card_to_battlefield(0, catalog::bounty_agent());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    bolt(&mut g, bear);
    assert!(named(&g, 0, "Zombie").is_empty());
    bolt(&mut g, agent);
    bolt(&mut g, x);
    let zombies = named(&g, 0, "Zombie");
    assert_eq!(zombies.len(), 2, "the Agent and the Necromancer itself");
    assert!(zombies.iter().all(|&z| g.battlefield_find(z).unwrap().tapped));
}

/// Species Specialist draws when a creature of its type dies, a token
/// included.
#[test]
fn species_specialist_draws_on_the_chosen_type() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    let ss = g.add_card_to_battlefield(0, catalog::species_specialist());
    g.battlefield_find_mut(ss).unwrap().chosen_creature_type = Some(CreatureType::Human);
    let doom = g.add_card_to_battlefield(0, catalog::thraben_doomsayer());
    g.clear_sickness(doom);
    activate(&mut g, doom, 0, &[]).expect("make a Human");
    let token = named(&g, 0, "Human")[0];
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    bolt(&mut g, bear);
    assert_eq!(g.players[0].hand.len(), hand);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    bolt(&mut g, token);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Thraben Doomsayer's fateful hour: +2/+2 to your other creatures at 5 or
/// less life only.
#[test]
fn thraben_doomsayer_fateful_hour() {
    let mut g = main_phase(2);
    let doom = g.add_card_to_battlefield(0, catalog::thraben_doomsayer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, bear), (2, 2));
    g.players[0].life = 5;
    assert_eq!(pt(&g, bear), (4, 4));
    assert_eq!(pt(&g, doom), (2, 2));
}

/// General's Enforcer: legendary Humans are indestructible; exiling a
/// creature card makes a Human Soldier, a land card doesn't.
#[test]
fn generals_enforcer_shields_legends_and_eats_graveyards() {
    let mut g = main_phase(2);
    let ge = g.add_card_to_battlefield(0, catalog::generals_enforcer());
    let odric = g.add_card_to_battlefield(0, catalog::odric_master_tactician());
    assert!(kw(&g, odric, Keyword::Indestructible) && !kw(&g, ge, Keyword::Indestructible));
    let land = g.add_card_to_graveyard(1, catalog::plains());
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    activate(&mut g, ge, 0, &[Target::Permanent(land)]).expect("exile a land");
    assert!(named(&g, 0, "Human Soldier").is_empty());
    activate(&mut g, ge, 0, &[Target::Permanent(bear)]).expect("exile a creature");
    assert_eq!(named(&g, 0, "Human Soldier").len(), 1);
    assert!(g.players[1].graveyard.is_empty());
}

/// Magus of the Disk enters tapped and sweeps artifacts, creatures and
/// enchantments (lands stay).
#[test]
fn magus_of_the_disk_sweeps() {
    let mut g = main_phase(2);
    let magus = g.add_card_to_hand(0, catalog::magus_of_the_disk());
    cast_at(&mut g, magus, &[]).expect("cast");
    assert!(g.battlefield_find(magus).unwrap().tapped);
    g.battlefield_find_mut(magus).unwrap().tapped = false;
    g.clear_sickness(magus);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(1, catalog::plains());
    activate(&mut g, magus, 0, &[]).expect("sweep");
    assert!(g.battlefield_find(ring).is_none() && g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(magus).is_none() && g.battlefield_find(land).is_some());
}

/// Captivating Crew steals, untaps and hastes an opposing creature.
#[test]
fn captivating_crew_threatens() {
    let mut g = main_phase(2);
    let crew = g.add_card_to_battlefield(0, catalog::captivating_crew());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    activate(&mut g, crew, 0, &[Target::Permanent(bear)]).expect("steal");
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.controller == 0 && !b.tapped);
    assert!(kw(&g, bear, Keyword::Haste));
}

/// Bounty Agent destroys a legendary creature, and Devout Chaplain taps two
/// Humans to exile an artifact.
#[test]
fn bounty_agent_and_devout_chaplain_remove() {
    let mut g = main_phase(2);
    let agent = g.add_card_to_battlefield(0, catalog::bounty_agent());
    g.clear_sickness(agent);
    let legend = g.add_card_to_battlefield(1, catalog::odric_master_tactician());
    activate(&mut g, agent, 0, &[Target::Permanent(legend)]).expect("bounty");
    assert!(g.battlefield_find(legend).is_none() && g.battlefield_find(agent).is_none());
    let chap = g.add_card_to_battlefield(0, catalog::devout_chaplain());
    g.clear_sickness(chap);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    assert!(activate(&mut g, chap, 0, &[Target::Permanent(ring)]).is_err(), "no Humans to tap");
    g.add_card_to_battlefield(0, catalog::bounty_agent());
    g.add_card_to_battlefield(0, catalog::xathrid_necromancer());
    activate(&mut g, chap, 0, &[Target::Permanent(ring)]).expect("tap two Humans");
    assert!(g.exile.iter().any(|c| c.id == ring));
}

/// Silvar eats a Human for a counter and indestructible.
#[test]
fn silvar_devours_a_human() {
    let mut g = main_phase(2);
    let s = g.add_card_to_battlefield(0, catalog::silvar_devourer_of_the_free());
    let agent = g.add_card_to_battlefield(0, catalog::bounty_agent());
    activate(&mut g, s, 0, &[]).expect("sacrifice");
    assert!(g.battlefield_find(agent).is_none());
    assert_eq!(pt(&g, s), (5, 3));
    assert!(kw(&g, s, Keyword::Indestructible));
}

/// Trynn makes a Human Soldier at your end step only on a turn you attacked.
#[test]
fn trynn_rewards_attacking() {
    let mut g = main_phase(2);
    let t = g.add_card_to_battlefield(0, catalog::trynn_champion_of_freedom());
    attack(&mut g, &[t]);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    for _ in 0..12 {
        if g.step == TurnStep::End {
            break;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.step, TurnStep::End);
    assert_eq!(named(&g, 0, "Human Soldier").len(), 1);
}

/// Vigilante Justice pings any target when a Human of yours enters.
#[test]
fn vigilante_justice_pings_on_humans() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::vigilante_justice());
    let life = g.players[1].life;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, bear, &[]).expect("cast");
    assert_eq!(g.players[1].life, life);
    let agent = g.add_card_to_hand(0, catalog::bounty_agent());
    cast_at(&mut g, agent, &[]).expect("cast");
    assert_eq!(g.players[1].life, life - 1);
}

/// Odric with three others attacking hands you the blocks; Fireflux Squad
/// swaps an attacker for a creature off the top, tapped and attacking.
#[test]
fn odric_and_fireflux_squad_on_the_attack() {
    let mut g = main_phase(2);
    let odric = g.add_card_to_battlefield(0, catalog::odric_master_tactician());
    let bears: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
    let mut all = vec![odric];
    all.extend(&bears);
    attack(&mut g, &all);
    assert_eq!(g.block_chooser(), Some(0));

    let mut g = main_phase(2);
    let giant = g.add_card_to_library(0, catalog::hill_giant());
    g.add_card_to_library(0, catalog::plains());
    let squad = g.add_card_to_battlefield(0, catalog::fireflux_squad());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, &[squad, bear]);
    assert!(g.exile.iter().any(|c| c.id == bear));
    let gc = g.battlefield_find(giant).expect("found and deployed");
    assert!(gc.tapped);
    assert!(g.attacking.iter().any(|a| a.attacker == giant));
}
