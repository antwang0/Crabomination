//! Commander: the Most Wanted precon (OTC, Olivia, Opulent Outlaw,
//! `decks::cmdr_olivia`).

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

fn act_as(g: &mut GameState, seat: usize, action: GameAction) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    act_as(g, 0, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, id, targets, None)
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

/// Attack player `at` with `attackers`, take no blocks, and finish combat.
fn swing(g: &mut GameState, attackers: &[CardId], at: usize) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let decl = attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(at) }).collect();
    g.perform_action(GameAction::DeclareAttackers(decl)).expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = at;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    for _ in 0..8 {
        if g.step == TurnStep::EndCombat {
            break;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// Olivia: one Treasure per combat-damage batch of outlaws, however many
/// connect; {3} and two Treasures put two +1/+1 counters on each creature.
#[test]
fn olivia_mints_treasure_and_spends_it() {
    let mut g = main_phase(2);
    let olivia = g.add_card_to_battlefield(0, catalog::olivia_opulent_outlaw());
    let skulk = g.add_card_to_battlefield(0, catalog::mistmeadow_skulk());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    swing(&mut g, &[olivia, skulk, bear], 1);
    assert_eq!(named(&g, 0, "Treasure").len(), 1, "one batch, one Treasure");
    g.step = TurnStep::PostCombatMain;
    // Glittering Stockpile is a Treasure too.
    let extra = g.add_card_to_battlefield(0, catalog::glittering_stockpile());
    activate(&mut g, olivia, 0, &[]).expect("sacrifice two Treasures");
    assert!(named(&g, 0, "Treasure").is_empty() && g.battlefield_find(extra).is_none());
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// CR 613.1d — Vihaan's animated Treasures are Construct Assassins, so
/// they're outlaws on the layered type line: one connecting mints Olivia's
/// Treasure. (Vihaan's own haste grant reads printed types — a residual.)
#[test]
fn cr_613_1d_vihaan_treasures_become_outlaws() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::vihaan_goldwaker());
    g.add_card_to_battlefield(0, catalog::olivia_opulent_outlaw());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let skulk = g.add_card_to_battlefield(0, catalog::mistmeadow_skulk());
    let stock = g.add_card_to_battlefield(0, catalog::glittering_stockpile());
    assert!(kw(&g, skulk, Keyword::Haste) && kw(&g, skulk, Keyword::Vigilance));
    assert!(!kw(&g, bear, Keyword::Haste), "not an outlaw");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::PreCombatMain;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(pt(&g, stock), (3, 3));
    swing(&mut g, &[stock], 1);
    assert_eq!(named(&g, 0, "Treasure").len(), 1, "the animated Assassin is an outlaw");
}

/// Mari exiles an opponent's dead creature with a hit counter; an Assassin
/// hitting that player cashes the counter in for a card and two Treasures.
#[test]
fn mari_marks_and_collects() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    let mari = g.add_card_to_battlefield(0, catalog::mari_the_killing_quill());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(kw(&g, mari, Keyword::Deathtouch));
    bolt(&mut g, bear);
    let exiled = g.exile.iter().find(|c| c.id == bear).expect("exiled instead of staying in the yard");
    assert_eq!(exiled.counter_count(CounterType::Hit), 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    swing(&mut g, &[mari], 1);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(named(&g, 0, "Treasure").len(), 2);
    assert_eq!(g.exile.iter().find(|c| c.id == bear).unwrap().counter_count(CounterType::Hit), 0);
}

/// Discreet Retreat's mana pays for an outlaw spell only; the first outlaw
/// spell each turn draws a card for 1 life.
#[test]
fn discreet_retreat_funds_outlaws_only() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::plains());
    let land = g.add_card_to_battlefield(0, catalog::swamp());
    let dr = g.add_card_to_battlefield(0, catalog::discreet_retreat());
    g.battlefield_find_mut(dr).unwrap().attached_to = Some(land);
    let teller = g.add_card_to_hand(0, catalog::mistmeadow_skulk());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    let n = g.battlefield_find(land).map(|c| c.definition.activated_abilities.len()).unwrap();
    let granted = (0..4)
        .find(|&i| {
            let mut probe = g.clone();
            probe
                .perform_action(GameAction::ActivateAbility {
                    card_id: land,
                    ability_index: n + i,
                    target: None,
                    additional_targets: vec![],
                    x_value: None,
                    mode: None,
                })
                .is_ok()
                && probe.players[0].mana_pool.total() == 2
        })
        .map(|i| n + i)
        .unwrap_or(n);
    let mut probe = g.clone();
    probe
        .perform_action(GameAction::ActivateAbility {
            card_id: land,
            ability_index: granted,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("tap for two");
    let mut bad = probe.clone();
    assert!(
        bad.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "a Bear isn't an outlaw"
    );
    let life = probe.players[0].life;
    probe
        .perform_action(GameAction::CastSpell { card_id: teller, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("a Rogue is");
    drain_stack(&mut probe);
    assert_eq!(probe.players[0].life, life - 1, "drew for 1 life");
    assert_eq!(probe.players[0].hand.len(), 2, "the Bear and the drawn card");
}

/// Bounty Board: a bounty creature dying feeds each of its controller's
/// opponents (a four-seat pod: the other three) a card and 2 life.
#[test]
fn bounty_board_pays_the_table() {
    let mut g = main_phase(4);
    for s in 0..4 {
        g.add_card_to_library(s, catalog::plains());
    }
    let board = g.add_card_to_battlefield(0, catalog::bounty_board());
    let bear = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    activate(&mut g, board, 1, &[Target::Permanent(bear)]).expect("bounty");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Bounty), 1);
    let before: Vec<_> = g.players.iter().map(|p| (p.life, p.hand.len())).collect();
    bolt(&mut g, bear);
    for s in [0, 1, 3] {
        assert_eq!((g.players[s].life, g.players[s].hand.len()), (before[s].0 + 2, before[s].1 + 1), "seat {s}");
    }
    assert_eq!(g.players[2].life, before[2].0);
}

/// Dire Fleet Ravager: each player loses a third of their life, rounded up.
#[test]
fn dire_fleet_ravager_takes_a_third() {
    let mut g = main_phase(4);
    g.players[0].life = 40;
    g.players[1].life = 13;
    g.players[2].life = 1;
    g.players[3].life = 30;
    let dfr = g.add_card_to_hand(0, catalog::dire_fleet_ravager());
    cast_at(&mut g, dfr, &[]).expect("cast");
    assert_eq!(g.players.iter().map(|p| p.life).collect::<Vec<_>>(), vec![26, 8, 0, 20]);
}

/// CR 616 — Angrath's Marauders doubles your sources' damage, and two
/// copies quadruple it.
#[test]
fn cr_616_angraths_marauders_doubles_and_stacks() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::angraths_marauders());
    let life = g.players[1].life;
    let b = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, b, &[Target::Player(1)]).expect("bolt");
    assert_eq!(g.players[1].life, life - 6);
    g.add_card_to_battlefield(0, catalog::angraths_marauders());
    let b = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, b, &[Target::Player(1)]).expect("bolt");
    assert_eq!(g.players[1].life, life - 6 - 12);
}

/// Glittering Stockpile banks a stash counter per {R}, then sacrifices for
/// that many of one colour.
#[test]
fn glittering_stockpile_cashes_in() {
    let mut g = main_phase(2);
    let gs = g.add_card_to_battlefield(0, catalog::glittering_stockpile());
    g.battlefield_find_mut(gs).unwrap().add_counters(CounterType::Stash, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: gs, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("tap for R");
    assert_eq!(g.battlefield_find(gs).unwrap().counter_count(CounterType::Stash), 3);
    g.battlefield_find_mut(gs).unwrap().tapped = false;
    let before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility { card_id: gs, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("sacrifice");
    assert!(g.battlefield_find(gs).is_none());
    assert_eq!(g.players[0].mana_pool.total(), before + 3);
}

/// Mistmeadow Skulk: protection from mana value 3 or greater — a Hill Giant
/// can't block it, a Grizzly Bears can; a 3-mana spell can't target it.
#[test]
fn mistmeadow_skulk_is_protected_from_big_things() {
    let mut g = main_phase(2);
    let skulk = g.add_card_to_battlefield(0, catalog::mistmeadow_skulk());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.clear_sickness(skulk);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: skulk, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(giant, skulk)])).is_err(), "MV 4 can't block");
    let mut g = main_phase(2);
    let skulk = g.add_card_to_battlefield(1, catalog::mistmeadow_skulk());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, &[Target::Permanent(skulk)]).expect("a 1-mana spell may target it");
    assert!(g.battlefield_find(skulk).is_none());
}

/// Mass Mutiny takes one creature from each opponent, untapped and hasty.
#[test]
fn mass_mutiny_takes_one_from_each() {
    let mut g = main_phase(4);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::hill_giant());
    g.battlefield_find_mut(b).unwrap().tapped = true;
    let mm = g.add_card_to_hand(0, catalog::mass_mutiny());
    cast_at(&mut g, mm, &[Target::Permanent(a), Target::Permanent(b)]).expect("cast");
    for id in [a, b] {
        let c = g.battlefield_find(id).unwrap();
        assert!(c.controller == 0 && !c.tapped);
        assert!(kw(&g, id, Keyword::Haste));
    }
}

/// Misfortune Teller: exiling a creature card makes a 2/2 Rogue (not a
/// Treasure).
#[test]
fn misfortune_teller_reads_what_it_exiled() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let mt = g.add_card_to_hand(0, catalog::misfortune_teller());
    cast_at(&mut g, mt, &[]).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bear), "the only graveyard card");
    assert_eq!(named(&g, 0, "Rogue").len(), 1);
    assert!(named(&g, 0, "Treasure").is_empty());
}

/// Life Insurance: a nontoken creature dying costs 1 life and makes a
/// Treasure; a token dying does nothing.
#[test]
fn life_insurance_pays_out_on_nontoken_deaths() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::life_insurance());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    bolt(&mut g, bear);
    assert_eq!(g.players[0].life, life - 1);
    assert_eq!(named(&g, 0, "Treasure").len(), 1);
}

/// We Ride at Dawn makes a Mercenary when your commander attacks, not when
/// another creature does.
#[test]
fn we_ride_at_dawn_follows_the_commander() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::we_ride_at_dawn());
    g.seat_commanders(0, vec![catalog::olivia_opulent_outlaw()]);
    let olivia = g.players[0].commanders[0];
    g.players[0].command.retain(|c| c.id != olivia);
    let def = catalog::olivia_opulent_outlaw();
    let card = crabomination::card::CardInstance::new(olivia, def, 0);
    g.battlefield.push(card);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(olivia);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: olivia, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Mercenary").len(), 1);
}

/// Back in Town returns X outlaw creature cards (not a Bear).
#[test]
fn back_in_town_returns_outlaws() {
    let mut g = main_phase(2);
    let a = g.add_card_to_graveyard(0, catalog::mistmeadow_skulk());
    let b = g.add_card_to_graveyard(0, catalog::misfortune_teller());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bit = g.add_card_to_hand(0, catalog::back_in_town());
    cast_x(&mut g, bit, &[], Some(3)).expect("cast");
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some());
    assert!(g.battlefield_find(bear).is_none());
}

/// Charred Graverobber returns an outlaw card on entry.
#[test]
fn charred_graverobber_digs_up_an_outlaw() {
    let mut g = main_phase(2);
    let skulk = g.add_card_to_graveyard(0, catalog::mistmeadow_skulk());
    let cg = g.add_card_to_hand(0, catalog::charred_graverobber());
    cast_at(&mut g, cg, &[]).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == skulk));
    assert_eq!(g.battlefield_find(cg).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "not escaped");
}

/// Dead Before Sunrise: your outlaws get +1/+0 and can tap to deal damage
/// equal to their power to a creature.
#[test]
fn dead_before_sunrise_arms_the_outlaws() {
    let mut g = main_phase(2);
    let skulk = g.add_card_to_battlefield(0, catalog::mistmeadow_skulk());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(skulk);
    let dbs = g.add_card_to_hand(0, catalog::dead_before_sunrise());
    cast_at(&mut g, dbs, &[]).expect("cast");
    assert_eq!(pt(&g, skulk), (2, 1));
    assert_eq!(pt(&g, bear), (2, 2));
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let idx = g.battlefield_find(skulk).unwrap().definition.activated_abilities.len();
    let ok = (0..=idx).any(|i| {
        let mut p = g.clone();
        activate(&mut p, skulk, i, &[Target::Permanent(target)]).is_ok() && p.battlefield_find(target).is_none()
    });
    assert!(ok, "the granted tap kills a 2/2 with 2 power");
}

/// Angelic Sell-Sword makes a Mercenary for itself and each nontoken
/// creature entering (not for a token).
#[test]
fn angelic_sell_sword_hires_mercenaries() {
    let mut g = main_phase(2);
    let ass = g.add_card_to_hand(0, catalog::angelic_sell_sword());
    cast_at(&mut g, ass, &[]).expect("cast");
    assert_eq!(named(&g, 0, "Mercenary").len(), 1);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, bear, &[]).expect("cast");
    assert_eq!(named(&g, 0, "Mercenary").len(), 2, "the token Mercenary didn't make a third");
}

/// Graywater's Fixer gives outlaw creature cards in your graveyard encore.
#[test]
fn graywaters_fixer_grants_encore() {
    let mut g = main_phase(4);
    g.add_card_to_battlefield(0, catalog::graywaters_fixer());
    let skulk = g.add_card_to_graveyard(0, catalog::mistmeadow_skulk());
    let before = named(&g, 0, "Mistmeadow Skulk").len();
    let ok = (0..3).any(|i| {
        let mut p = g.clone();
        activate(&mut p, skulk, i, &[]).is_ok() && named(&p, 0, "Mistmeadow Skulk").len() == before + 3
    });
    assert!(ok, "encore makes one token per opponent");
}
