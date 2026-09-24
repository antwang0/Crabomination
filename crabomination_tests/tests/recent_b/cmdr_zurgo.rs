//! Commander: the Mardu Surge precon (TDC, Zurgo Stormrender, `decks::cmdr_zurgo`).

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

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, id, targets, None)
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: x,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::plains());
    }
}

fn declare(g: &mut GameState, seat: usize, attacks: Vec<Attack>) -> Result<(), String> {
    for a in &attacks {
        g.clear_sickness(a.attacker);
    }
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(attacks)).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn at(attacker: CardId, p: usize) -> Attack {
    Attack { attacker, target: AttackTarget::Player(p) }
}

fn run_combat_out(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

/// Cast seat 0's commander from the command zone.
fn commander_out(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let cmd = g.seat_commanders(0, vec![def])[0];
    flood(g, 0);
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
    .expect("commander");
    drain_stack(g);
    cmd
}

/// Zurgo: a mobilized Warrior sacrificed at the end step is no longer
/// attacking, so each opponent loses 1; a token dying in combat draws.
#[test]
fn zurgo_draws_for_attackers_and_drains_for_the_rest() {
    let mut g = main_phase(3);
    library(&mut g, 0, 3);
    let z = g.add_card_to_battlefield(0, catalog::zurgo_stormrender());
    declare(&mut g, 0, vec![at(z, 1)]).expect("attack");
    assert_eq!(named(&g, 0, "Warrior").len(), 1);
    let hand = g.players[0].hand.len();
    let warrior = named(&g, 0, "Warrior")[0];
    let mut dies = g.clone();
    let bolt = dies.add_card_to_hand(1, catalog::lightning_bolt());
    dies.players[1].mana_pool.add(Color::Red, 1);
    dies.priority.player_with_priority = 1;
    dies.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(warrior)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut dies);
    assert_eq!(dies.players[0].hand.len(), hand + 1, "it was attacking");
    run_combat_out(&mut g);
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert!(g.attacking.is_empty(), "combat is over");
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    fire(&mut g, TurnStep::End);
    assert!(named(&g, 0, "Warrior").is_empty(), "sacrificed at the end step");
    assert_eq!((g.players[1].life, g.players[2].life), (l1 - 1, l2 - 1));
    assert_eq!(g.players[0].hand.len(), hand);
}

/// Ainok Strike Leader: attacking with it sends a Goblin at each opponent;
/// attacking with something else doesn't. Sacrificed, tokens go indestructible.
#[test]
fn ainok_strike_leader_rallies_goblins_per_opponent() {
    let mut g = main_phase(3);
    let a = g.add_card_to_battlefield(0, catalog::ainok_strike_leader());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    declare(&mut g.clone(), 0, vec![at(bear, 1)]).expect("attack");
    let mut other = g.clone();
    declare(&mut other, 0, vec![at(bear, 1)]).expect("attack");
    assert!(named(&other, 0, "Goblin").is_empty(), "neither it nor a commander attacked");
    declare(&mut g, 0, vec![at(a, 1)]).expect("attack");
    let goblins = named(&g, 0, "Goblin");
    assert_eq!(goblins.len(), 2);
    let defenders: Vec<_> =
        g.attacking.iter().filter(|x| goblins.contains(&x.attacker)).map(|x| x.target).collect();
    assert!(defenders.contains(&AttackTarget::Player(1)) && defenders.contains(&AttackTarget::Player(2)));
    activate(&mut g, a, 0, &[], None).expect("sacrifice");
    assert!(g.computed_permanent(goblins[0]).unwrap().keywords().contains(&Keyword::Indestructible));
}

/// Aron: sacrifice another creature for a counter on each of yours.
#[test]
fn aron_sacrifices_for_counters() {
    let mut g = main_phase(2);
    let aron = g.add_card_to_battlefield(0, catalog::aron_benalias_ruin());
    g.clear_sickness(aron);
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let keeper = g.add_card_to_battlefield(0, catalog::hill_giant());
    activate(&mut g, aron, 0, &[], None).expect("activate");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count(), 2);
    assert!(g.battlefield_find(fodder).is_none() || g.battlefield_find(keeper).is_none());
    assert_eq!(g.battlefield_find(aron).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Bone Devourer enters with a counter per creature that died this turn;
/// dying, it draws and loses that many.
#[test]
fn bone_devourer_feeds_on_the_dead() {
    let mut g = main_phase(2);
    library(&mut g, 0, 3);
    for _ in 0..2 {
        let v = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let m = g.add_card_to_hand(0, catalog::murder());
        cast_at(&mut g, m, &[Target::Permanent(v)]).expect("kill");
    }
    let bd = g.add_card_to_hand(0, catalog::bone_devourer());
    cast_at(&mut g, bd, &[]).expect("cast");
    assert_eq!(g.battlefield_find(bd).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    let m = g.add_card_to_hand(0, catalog::murder());
    cast_at(&mut g, m, &[Target::Permanent(bd)]).expect("kill");
    assert_eq!((g.players[0].hand.len(), g.players[0].life), (hand + 2, life - 2));
}

/// Divine Visitation: creature tokens are Angels instead, still tapped
/// (a ruling); a Treasure stays a Treasure.
#[test]
fn divine_visitation_turns_tokens_into_angels() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::divine_visitation());
    let ss = g.add_card_to_hand(0, catalog::shadow_summoning());
    cast_at(&mut g, ss, &[]).expect("cast");
    let angels = named(&g, 0, "Angel");
    assert_eq!((angels.len(), named(&g, 0, "Spirit").len()), (2, 0));
    let cp = g.computed_permanent(angels[0]).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords().contains(&Keyword::Vigilance) && g.battlefield_find(angels[0]).unwrap().tapped);
}

/// Eliminate the Competition: sacrifice X creatures, destroy X.
#[test]
fn eliminate_the_competition_trades_x_for_x() {
    let mut g = main_phase(2);
    let mine = [g.add_card_to_battlefield(0, catalog::grizzly_bears()), g.add_card_to_battlefield(0, catalog::grizzly_bears())];
    let theirs = [g.add_card_to_battlefield(1, catalog::hill_giant()), g.add_card_to_battlefield(1, catalog::serra_angel())];
    let e = g.add_card_to_hand(0, catalog::eliminate_the_competition());
    cast_x(&mut g, e, &[Target::Permanent(theirs[0]), Target::Permanent(theirs[1])], Some(2)).expect("cast");
    assert!(mine.iter().chain(&theirs).all(|&c| g.battlefield_find(c).is_none()));
}

/// Gix: an opponent's creature hitting another opponent lets *its*
/// controller pay 1 life to draw. Discard X: an opponent's top X, free to cast.
#[test]
fn gix_rewards_any_attacker_and_steals_the_top() {
    let mut g = main_phase(3);
    library(&mut g, 1, 2);
    g.add_card_to_battlefield(0, catalog::gix_yawgmoth_praetor());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let (hand, life) = (g.players[1].hand.len(), g.players[1].life);
    declare(&mut g, 1, vec![at(theirs, 2)]).expect("attack");
    run_combat_out(&mut g);
    assert_eq!((g.players[1].hand.len(), g.players[1].life), (hand + 1, life - 1));

    let mut g = main_phase(2);
    let gix = g.add_card_to_battlefield(0, catalog::gix_yawgmoth_praetor());
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::hill_giant());
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::plains());
    }
    activate(&mut g, gix, 0, &[Target::Player(1)], Some(2)).expect("discard two");
    assert!(g.players[0].hand.is_empty());
    let c = g.exile.iter().find(|c| c.id == bear).expect("exiled");
    assert_eq!(c.may_play_until.map(|p| p.player), Some(0));
    g.players[0].mana_pool = Default::default();
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
}

/// Infantry Shield: menace, and mobilize equal to the equipped power.
#[test]
fn infantry_shield_mobilizes_by_power() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let s = g.add_card_to_battlefield(0, catalog::infantry_shield());
    g.battlefield_find_mut(s).unwrap().attached_to = Some(giant);
    assert!(g.computed_permanent(giant).unwrap().keywords().contains(&Keyword::Menace));
    declare(&mut g, 0, vec![at(giant, 1)]).expect("attack");
    assert_eq!(named(&g, 0, "Warrior").len(), 3);
}

/// Ironwill Forger: with your commander out, a nonlegendary creature gains
/// myriad — a copy attacks the other opponent.
#[test]
fn ironwill_forger_grants_myriad_with_a_commander() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::ironwill_forger());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut without = g.clone();
    commander_out(&mut g, catalog::zurgo_stormrender());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    fire(&mut g, TurnStep::BeginCombat);
    declare(&mut g, 0, vec![at(bear, 1)]).expect("attack");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 2, "a myriad copy");
    fire(&mut without, TurnStep::BeginCombat);
    declare(&mut without, 0, vec![at(bear, 1)]).expect("attack");
    assert_eq!(named(&without, 0, "Grizzly Bears").len(), 1, "no commander, no myriad");
}

/// Kaya, Geist Hunter: −2 doubles this turn's tokens; +1 deathtouch and a
/// counter on a token; −6 a Spirit per card exiled from graveyards.
#[test]
fn kaya_geist_hunter_doubles_and_harvests() {
    let mut g = main_phase(2);
    let k = g.add_card_to_battlefield(0, catalog::kaya_geist_hunter());
    let mut plus = g.clone();
    loyalty(&mut g, k, 1, None).expect("−2");
    let ss = g.add_card_to_hand(0, catalog::shadow_summoning());
    cast_at(&mut g, ss, &[]).expect("cast");
    assert_eq!(named(&g, 0, "Spirit").len(), 4);
    let ss = plus.add_card_to_hand(0, catalog::shadow_summoning());
    cast_at(&mut plus, ss, &[]).expect("cast");
    let spirit = named(&plus, 0, "Spirit")[0];
    loyalty(&mut plus, k, 0, Some(Target::Permanent(spirit))).expect("+1");
    assert_eq!(plus.battlefield_find(spirit).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(plus.computed_permanent(spirit).unwrap().keywords().contains(&Keyword::Deathtouch));
    let mut ult = main_phase(2);
    let k = ult.add_card_to_battlefield(0, catalog::kaya_geist_hunter());
    ult.battlefield_find_mut(k).unwrap().add_counters(CounterType::Loyalty, 3);
    for seat in [0, 1] {
        ult.add_card_to_graveyard(seat, catalog::grizzly_bears());
    }
    loyalty(&mut ult, k, 2, None).expect("−6");
    // Kaya, at 0 loyalty, is in the graveyard before the ability resolves
    // (CR 704.5i) and is exiled and counted with the rest.
    assert_eq!(named(&ult, 0, "Spirit").len(), 3);
    assert!(ult.players.iter().all(|p| p.graveyard.is_empty()));
}

/// Legion Loyalty: your attacker gets a myriad copy at the other opponent.
#[test]
fn legion_loyalty_gives_the_team_myriad() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::legion_loyalty());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    declare(&mut g, 0, vec![at(bear, 1)]).expect("attack");
    let copies: Vec<_> = named(&g, 0, "Grizzly Bears").into_iter().filter(|&c| c != bear).collect();
    assert_eq!(copies.len(), 1);
    assert!(g.attacking.iter().any(|a| a.attacker == copies[0] && a.target == AttackTarget::Player(2)));
}

/// Mindblade Render: a Warrior connecting draws a card and costs 1 life; a
/// non-Warrior doesn't.
#[test]
fn mindblade_render_rewards_warrior_hits() {
    let mut g = main_phase(2);
    library(&mut g, 0, 2);
    g.add_card_to_battlefield(0, catalog::mindblade_render());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut plain = g.clone();
    declare(&mut plain, 0, vec![at(bear, 1)]).expect("attack");
    run_combat_out(&mut plain);
    assert_eq!(plain.players[0].hand.len(), 0);
    let z = g.add_card_to_battlefield(0, catalog::zurgo_stormrender());
    let life = g.players[0].life;
    declare(&mut g, 0, vec![at(z, 1), at(bear, 1)]).expect("attack");
    run_combat_out(&mut g);
    assert_eq!((g.players[0].hand.len(), g.players[0].life), (1, life - 1), "once for the whole step");
}

/// Neriv: two Goblins on entry; attacking exiles one card per differently
/// named token, playable only on a turn you attacked with a commander.
#[test]
fn neriv_exiles_by_token_names_for_commander_turns() {
    let mut g = main_phase(2);
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    let n = g.add_card_to_hand(0, catalog::neriv_crackling_vanguard());
    cast_at(&mut g, n, &[]).expect("cast");
    assert_eq!(named(&g, 0, "Goblin").len(), 2);
    let mut with_commander = g.clone();
    declare(&mut g, 0, vec![at(n, 1)]).expect("attack");
    let c = g.exile.iter().find(|c| c.id == top).expect("one name, one card");
    assert_eq!(c.may_play_until.map(|p| p.player), Some(crabomination::card::MAY_PLAY_DORMANT));
    let z = commander_out(&mut with_commander, catalog::zurgo_stormrender());
    declare(&mut with_commander, 0, vec![at(n, 1), at(z, 1)]).expect("attack with a commander");
    let c = with_commander.exile.iter().find(|c| c.id == top).expect("exiled");
    assert_eq!(c.may_play_until.map(|p| p.player), Some(0));
}

/// Ogre Battledriver: your next creature enters +2/+0 with haste.
#[test]
fn ogre_battledriver_hastes_arrivals() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::ogre_battledriver());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, bear, &[]).expect("cast");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4);
    assert!(cp.keywords().contains(&Keyword::Haste));
}

/// Redoubled Stormsinger copies this turn's creature tokens into the attack;
/// the copies go at the end step, the originals stay.
#[test]
fn redoubled_stormsinger_doubles_fresh_tokens() {
    let mut g = main_phase(2);
    let rs = g.add_card_to_battlefield(0, catalog::redoubled_stormsinger());
    let ss = g.add_card_to_hand(0, catalog::shadow_summoning());
    cast_at(&mut g, ss, &[]).expect("two Spirits");
    declare(&mut g, 0, vec![at(rs, 1)]).expect("attack");
    let spirits = named(&g, 0, "Spirit");
    assert_eq!(spirits.len(), 4);
    assert_eq!(g.attacking.iter().filter(|a| spirits.contains(&a.attacker)).count(), 2, "the copies attack");
    run_combat_out(&mut g);
    fire(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Spirit").len(), 2);
}

/// Thalisse: each end step, a Spirit per token you created this turn.
#[test]
fn thalisse_echoes_the_turns_tokens() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::thalisse_reverent_medium());
    let ss = g.add_card_to_hand(0, catalog::shadow_summoning());
    cast_at(&mut g, ss, &[]).expect("two Spirits");
    fire(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Spirit").len(), 4);
}

/// Will of the Mardu: one mode alone; both with a commander out.
#[test]
fn will_of_the_mardu_does_both_with_a_commander() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let w = g.add_card_to_hand(0, catalog::will_of_the_mardu());
    cast_at(&mut g, w, &[Target::Player(1)]).expect("one mode");
    assert_eq!(named(&g, 0, "Warrior").len(), 4);
    let mut g = main_phase(2);
    let giant2 = g.add_card_to_battlefield(1, catalog::hill_giant());
    commander_out(&mut g, catalog::zurgo_stormrender());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let w = g.add_card_to_hand(0, catalog::will_of_the_mardu());
    cast_at(&mut g, w, &[Target::Player(1), Target::Permanent(giant2)]).expect("both");
    assert_eq!(named(&g, 0, "Warrior").len(), 1);
    assert!(g.battlefield_find(giant2).is_none(), "3 creatures deal 3");
    let _ = giant;
}

/// Within Range: two Warriors; attacking drains each opponent per creature
/// attacking them.
#[test]
fn within_range_drains_per_attacker() {
    let mut g = main_phase(3);
    let wr = g.add_card_to_hand(0, catalog::within_range());
    cast_at(&mut g, wr, &[]).expect("cast");
    let w = named(&g, 0, "Warrior");
    assert_eq!(w.len(), 2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    declare(&mut g, 0, vec![at(w[0], 1), at(w[1], 1), at(bear, 2)]).expect("attack");
    assert_eq!((g.players[1].life, g.players[2].life), (l1 - 2, l2 - 1));
}
