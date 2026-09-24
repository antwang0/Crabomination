//! Commander: the Heavenly Inferno precon (CMD, Kaalia, `decks::cmdr_kaalia`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn try_cast(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    try_cast(g, 0, id, targets).expect("cast");
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn attack(g: &mut GameState, attacker: CardId, defender: usize) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(defender) }]))
        .expect("attack");
    drain_stack(g);
}

fn finish_combat(g: &mut GameState, defender: usize) {
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// Archangel of Strife: each player's choice rides that player's creatures —
/// a bot picks war for itself (+3/+0) and peace for the others (+0/+3),
/// including creatures that arrive later.
#[test]
fn archangel_of_strife_splits_the_table() {
    let mut g = pod(3);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let arch = g.add_card_to_hand(0, catalog::archangel_of_strife());
    cast(&mut g, arch, &[]);
    assert_eq!(pt(&g, mine), (5, 2));
    assert_eq!(pt(&g, arch), (9, 6));
    assert_eq!(pt(&g, theirs), (2, 5));
    let later = g.add_card_to_battlefield(2, catalog::hill_giant());
    assert_eq!(pt(&g, later), (3, 6));
}

/// CR 601 — Basandra: nobody casts a spell during combat.
#[test]
fn basandra_shuts_combat_spells_off() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::basandra_battle_seraph());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.step = TurnStep::DeclareAttackers;
    assert!(try_cast(&mut g, 0, bolt, &[Target::Player(1)]).is_err());
    g.step = TurnStep::PostCombatMain;
    try_cast(&mut g, 0, bolt, &[Target::Player(1)]).expect("fine after combat");
}

/// Death by Dragons: everyone but the target gets a 5/5 flier — you included.
#[test]
fn death_by_dragons_skips_the_target() {
    let mut g = pod(3);
    let dbd = g.add_card_to_hand(0, catalog::death_by_dragons());
    cast(&mut g, dbd, &[Target::Player(1)]);
    let dragons = |s: usize| g.battlefield.iter().filter(|c| c.controller == s && c.definition.name == "Dragon").count();
    assert_eq!((dragons(0), dragons(1), dragons(2)), (1, 0, 1));
}

/// Dragon Whelp: the fourth firebreath this turn dooms it at the next end
/// step; three don't.
#[test]
fn dragon_whelp_overheats_on_the_fourth_pump() {
    for (pumps, survives) in [(3, true), (4, false)] {
        let mut g = pod(2);
        let whelp = g.add_card_to_battlefield(0, catalog::dragon_whelp());
        for _ in 0..pumps {
            activate(&mut g, whelp, None).expect("pump");
        }
        assert_eq!(pt(&g, whelp).0, 2 + pumps);
        while g.step != TurnStep::End {
            let _ = g.advance_step(Vec::new());
            drain_stack(&mut g);
        }
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(whelp).is_some(), survives, "{pumps} pumps");
    }
}

/// Dread Cacodemon cast from hand wipes the opponents' creatures and taps
/// your others; put onto the battlefield otherwise, it does nothing.
#[test]
fn dread_cacodemon_needs_to_be_cast() {
    let mut g = pod(3);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::hill_giant());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cc = g.add_card_to_hand(0, catalog::dread_cacodemon());
    cast(&mut g, cc, &[]);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert!(g.battlefield_find(mine).unwrap().tapped);
    assert!(!g.battlefield_find(cc).unwrap().tapped);

    let mut g = pod(2);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::dread_cacodemon());
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_some());
}

/// Hedge-mages: each trigger needs two of its land type (an intervening if).
#[test]
fn hedge_mages_check_their_land_types() {
    let mut g = pod(2);
    let rock = g.add_card_to_battlefield(1, catalog::sol_ring());
    let mage = g.add_card_to_hand(0, catalog::duergar_hedge_mage());
    g.add_card_to_battlefield(0, catalog::mountain());
    cast(&mut g, mage, &[]);
    assert!(g.battlefield_find(rock).is_some(), "one Mountain is not two");

    let mut g = pod(2);
    let rock = g.add_card_to_battlefield(1, catalog::sol_ring());
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mage = g.add_card_to_hand(0, catalog::duergar_hedge_mage());
    cast(&mut g, mage, &[]);
    assert!(g.battlefield_find(rock).is_none());

    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::plains());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mage = g.add_card_to_hand(0, catalog::gwyllion_hedge_mage());
    cast(&mut g, mage, &[]);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Kithkin Soldier"));
}

/// Kaalia attacking an opponent puts an Angel, Demon or Dragon from hand
/// onto the battlefield tapped and attacking that opponent — never the bigger
/// Wurm beside it.
#[test]
fn kaalia_drops_an_angel_into_the_attack() {
    let mut g = pod(3);
    let kaalia = g.add_card_to_battlefield(0, catalog::kaalia_of_the_vast());
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    g.add_card_to_hand(0, catalog::craw_wurm());
    attack(&mut g, kaalia, 2);
    let a = g.battlefield_find(angel).expect("the Angel is in play");
    assert!(a.tapped);
    assert!(g.attacking.iter().any(|x| x.attacker == angel && x.target == AttackTarget::Player(2)));
}

/// Malfegor discards your hand; each opponent sacrifices that many creatures.
#[test]
fn malfegor_trades_your_hand_for_their_board() {
    let mut g = pod(3);
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.add_card_to_battlefield(2, catalog::grizzly_bears());
    }
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let mal = g.add_card_to_hand(0, catalog::malfegor());
    cast(&mut g, mal, &[]);
    assert!(g.players[0].hand.is_empty());
    for s in [1, 2] {
        assert_eq!(g.battlefield.iter().filter(|c| c.controller == s).count(), 1, "seat {s} lost two");
    }
}

/// Sulfurous Blast is 3 in your main phase and 2 otherwise.
#[test]
fn sulfurous_blast_hits_harder_in_your_main_phase() {
    for (step, n) in [(TurnStep::PreCombatMain, 3), (TurnStep::End, 2)] {
        let mut g = pod(2);
        g.step = step;
        let sb = g.add_card_to_hand(0, catalog::sulfurous_blast());
        cast(&mut g, sb, &[]);
        assert_eq!(g.players[1].life, g.players[1].starting_life - n);
    }
}

/// Tariel takes a random creature card from target *opponent*'s graveyard;
/// your own graveyard is not a legal target.
#[test]
fn tariel_raids_an_opponents_graveyard() {
    let mut g = pod(2);
    let tar = g.add_card_to_battlefield(0, catalog::tariel_reckoner_of_souls());
    g.clear_sickness(tar);
    let wurm = g.add_card_to_graveyard(1, catalog::craw_wurm());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    assert!(activate(&mut g, tar, Some(Target::Player(0))).is_err(), "not your own");
    activate(&mut g, tar, Some(Target::Player(1))).expect("opponent");
    assert_eq!(g.battlefield_find(wurm).map(|c| c.controller), Some(0));
}

/// Vow of Lightning: +2/+2, first strike, and the enchanted creature can't
/// attack the Aura's controller.
#[test]
fn vow_of_lightning_pacifies_toward_you() {
    let mut g = pod(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let vow = g.add_card_to_hand(0, catalog::vow_of_lightning());
    cast(&mut g, vow, &[Target::Permanent(bear)]);
    assert_eq!(pt(&g, bear), (4, 4));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::FirstStrike));
    g.active_player_idx = 1;
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let at = |p| GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(p) }]);
    assert!(g.perform_action(at(0)).is_err());
    g.perform_action(at(2)).expect("another opponent is fine");
}

/// Oros: combat damage to a player, pay {2}{W}, and every nonwhite creature
/// takes 3.
#[test]
fn oros_punishes_nonwhite_creatures() {
    let mut g = pod(2);
    let oros = g.add_card_to_battlefield(0, catalog::oros_the_avenger());
    let red = g.add_card_to_battlefield(1, catalog::hill_giant());
    let white = g.add_card_to_battlefield(1, catalog::serra_angel());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, oros, 1);
    finish_combat(&mut g, 1);
    assert!(g.battlefield_find(red).is_none());
    assert_eq!(g.battlefield_find(white).unwrap().damage, 0);
}
