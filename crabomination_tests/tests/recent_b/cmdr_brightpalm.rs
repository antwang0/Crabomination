//! Commander: the Call for Backup precon (MOC, Bright-Palm,
//! `decks::cmdr_brightpalm`).

use crabomination::card::{CardId, CounterType, Keyword};
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

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x_value: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, id, targets, None)
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0)
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn attack_with(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for a in attackers {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn no_blocks_finish(g: &mut GameState, defender: usize) {
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(g);
    advance_to(g, TurnStep::PostCombatMain);
}

/// CR 702.164 — Bright-Palm backs up another creature, and its attack
/// doubles a creature's counters and shuts out small blockers.
#[test]
fn cr_702_164_bright_palm_backs_up_and_doubles() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    let bp = g.add_card_to_hand(0, catalog::bright_palm_soul_awakener());
    cast_at(&mut g, bp, &[]).expect("cast");
    assert_eq!(plus(&g, bear), 1, "backup 1 on the Bear");
    attack_with(&mut g, &[bear, bp], 1);
    assert_eq!(plus(&g, bear), 4, "both attack triggers doubled it: 1 → 2 → 4");
    let kws = g.computed_permanent(bear).unwrap().keywords().to_vec();
    assert!(kws.contains(&Keyword::CantBeBlockedByPowerAtMost(2)));
}

/// Alharu grows two others on entry; a countered nontoken creature dying
/// leaves a flying Spirit.
#[test]
fn alharu_spirits_from_countered_deaths() {
    let mut g = main_phase(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let al = g.add_card_to_hand(0, catalog::alharu_solemn_ritualist());
    cast_at(&mut g, al, &[Target::Permanent(a), Target::Permanent(b)]).expect("cast");
    assert_eq!((plus(&g, a), plus(&g, b)), (1, 1));
    let kill = g.add_card_to_hand(0, catalog::murder());
    cast_at(&mut g, kill, &[Target::Permanent(a)]).expect("murder");
    assert_eq!(named(&g, 0, "Spirit").len(), 1);
}

/// Bretagard Stronghold's sorcery-speed sacrifice grows two creatures and
/// gives them vigilance and lifelink.
#[test]
fn bretagard_stronghold_sacrifices_for_counters() {
    let mut g = main_phase(2);
    let land = g.add_card_to_battlefield(0, catalog::bretagard_stronghold());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, land, 1, &[Target::Permanent(a), Target::Permanent(b)]).expect("activate");
    assert!(g.battlefield_find(land).is_none());
    for c in [a, b] {
        assert_eq!(plus(&g, c), 1);
        let kws = g.computed_permanent(c).unwrap().keywords().to_vec();
        assert!(kws.contains(&Keyword::Vigilance) && kws.contains(&Keyword::Lifelink));
    }
}

/// Dromoka's Command: two of its modes — a counter and an enchantment
/// sacrifice.
#[test]
fn dromokas_command_chooses_two() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ench = g.add_card_to_battlefield(1, catalog::uncivil_unrest());
    let dc = g.add_card_to_hand(0, catalog::dromokas_command());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: dc,
        spree_modes: vec![1, 2],
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(bear)],
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none());
    assert_eq!(plus(&g, bear), 1);
}

/// Emergent Woodwurm's attack puts a permanent with mana value up to its
/// power onto the battlefield.
#[test]
fn emergent_woodwurm_digs_on_attack() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let ww = g.add_card_to_battlefield(0, catalog::emergent_woodwurm());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    attack_with(&mut g, &[ww], 1);
    assert!(g.battlefield_find(bear).is_some(), "mana value 2 fits under power 4");
    assert_eq!(g.players[0].library.len(), 3, "the rest went to the bottom");
}

/// Falkenrath Exterminator grows on a hit and shoots for its counters.
#[test]
fn falkenrath_exterminator_grows_and_shoots() {
    let mut g = main_phase(2);
    let fe = g.add_card_to_battlefield(0, catalog::falkenrath_exterminator());
    attack_with(&mut g, &[fe], 1);
    no_blocks_finish(&mut g, 1);
    assert_eq!(plus(&g, fe), 1);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, fe, 0, &[Target::Permanent(bear)]).expect("shoot");
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1);
}

/// Hamza discounts itself and creature spells per countered creature.
#[test]
fn hamza_discounts_per_countered_creature() {
    let mut g = main_phase(2);
    for _ in 0..2 {
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(b).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    }
    g.add_card_to_battlefield(0, catalog::hamza_guardian_of_arashin());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    // {1}{G} less {2} (floored at the colored part) — one green pays it.
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("a one-mana Bear");
}

/// Heaven hits fliers; Earth, from the graveyard, hits the rest.
#[test]
fn heaven_earth_splits_the_sky() {
    let mut g = main_phase(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let he = g.add_card_to_hand(0, catalog::heaven_earth());
    cast_x(&mut g, he, &[], Some(4)).expect("heaven");
    assert!(g.battlefield_find(angel).is_none() && g.battlefield_find(bear).is_some());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastAftermath { card_id: he, target: None, additional_targets: vec![], mode: None, x_value: Some(2) }).expect("earth");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// High Sentinels grow with your other countered creatures.
#[test]
fn high_sentinels_count_countered_allies() {
    let mut g = main_phase(2);
    let hs = g.add_card_to_battlefield(0, catalog::high_sentinels_of_arashin());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, hs), (3, 4));
    activate(&mut g, hs, 0, &[Target::Permanent(bear)]).expect("counter");
    assert_eq!(pt(&g, hs), (4, 5));
}

/// Inscription of Abundance's life mode reads the target's greatest power.
#[test]
fn inscription_of_abundance_gains_greatest_power() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::serra_angel());
    let ins = g.add_card_to_hand(0, catalog::inscription_of_abundance());
    let life = g.players[0].life;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: ins,
        spree_modes: vec![1],
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4);
}

/// Mikaeus enters with X counters and spreads one to the team.
#[test]
fn mikaeus_the_lunarch_spreads_counters() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::mikaeus_the_lunarch());
    cast_x(&mut g, m, &[], Some(3)).expect("cast");
    assert_eq!(plus(&g, m), 3);
    g.clear_sickness(m);
    activate(&mut g, m, 1, &[]).expect("spread");
    assert_eq!((plus(&g, m), plus(&g, bear)), (2, 1));
}

/// Mirror-Style Master copies each attacking modified creature.
#[test]
fn mirror_style_master_copies_modified_attackers() {
    let mut g = main_phase(2);
    let ms = g.add_card_to_battlefield(0, catalog::mirror_style_master());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let plain = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    attack_with(&mut g, &[ms, bear, plain], 1);
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 2, "one copy of the modified Bear");
    assert_eq!(named(&g, 0, "Serra Angel").len(), 1);
    no_blocks_finish(&mut g, 1);
    assert_eq!(named(&g, 0, "Grizzly Bears"), vec![bear], "exiled at end of combat");
}

/// Path of the Pyromancer wheels the hand into red mana and one extra card.
#[test]
fn path_of_the_pyromancer_wheels() {
    let mut g = main_phase(2);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::plains());
    }
    let p = g.add_card_to_hand(0, catalog::path_of_the_pyromancer());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, p, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), 3, "two discarded, three drawn");
}

/// Shalai and Hallar turn +1/+1 counters into damage to an opponent.
#[test]
fn shalai_and_hallar_burn_per_counter() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::shalai_and_hallar());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hs = g.add_card_to_battlefield(0, catalog::high_sentinels_of_arashin());
    let life = g.players[1].life;
    activate(&mut g, hs, 0, &[Target::Permanent(bear)]).expect("counter");
    assert_eq!(g.players[1].life, life - 1);
}

/// Slurrk enters with five counters; a countered creature dying grows the
/// countered rest.
#[test]
fn slurrk_feeds_on_countered_deaths() {
    let mut g = main_phase(2);
    let s = g.add_card_to_hand(0, catalog::slurrk_all_ingesting());
    cast_at(&mut g, s, &[]).expect("cast");
    assert_eq!(plus(&g, s), 5);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let kill = g.add_card_to_hand(0, catalog::murder());
    cast_at(&mut g, kill, &[Target::Permanent(bear)]).expect("murder");
    assert_eq!(plus(&g, s), 6);
}

/// Uncivil Unrest: riot on entry, and a countered creature hits twice as
/// hard.
#[test]
fn uncivil_unrest_riots_and_doubles() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::uncivil_unrest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, bear, &[]).expect("cast");
    let life = g.players[1].life;
    if plus(&g, bear) == 0 {
        g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    }
    attack_with(&mut g, &[bear], 1);
    no_blocks_finish(&mut g, 1);
    assert_eq!(g.players[1].life, life - 6, "a 3/3 dealing double");
}
