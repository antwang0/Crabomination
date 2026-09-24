//! Commander: the Blame Game precon (MKC, Nelly Borca, `decks::cmdr_nelly`),
//! and the goad it is built on — CR 701.15 from a resolved goad, a held goad
//! (CR 611.2b) and an attached Aura or Equipment.

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

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target], mode: Option<usize>) -> Result<(), String> {
    act_as(g, 0, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode,
    })
}

fn goaded_by(g: &GameState, id: CardId, p: usize) -> bool {
    g.battlefield_find(id).is_some_and(|c| g.goaded_by_player(c, p))
}

fn goaded(g: &GameState, id: CardId) -> bool {
    g.battlefield_find(id).is_some_and(|c| g.is_goaded(c))
}

fn power(g: &GameState, id: CardId) -> i32 {
    g.computed_permanent(id).expect("on the battlefield").power
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::plains());
    }
}

/// Declare `attacks` for `seat` (made active); the engine's verdict.
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

/// Declare, no blocks, run combat out.
fn swing(g: &mut GameState, seat: usize, attacks: Vec<Attack>) {
    declare(g, seat, attacks).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

/// CR 701.15a/b — Bloodthirsty Blade: the equipped creature is goaded by the
/// Blade's controller for as long as it is attached, whoever's turn it is; it
/// must attack, and not the goader while another opponent is open. Moving the
/// Blade moves the goad.
#[test]
fn cr_701_15_bloodthirsty_blade_goads_while_attached() {
    let mut g = main_phase(3);
    let blade = g.add_card_to_battlefield(0, catalog::bloodthirsty_blade());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ogre = g.add_card_to_battlefield(2, catalog::hill_giant());
    activate(&mut g, blade, 0, &[Target::Permanent(bear)], None).expect("attach");
    assert_eq!(power(&g, bear), 4);
    assert!(goaded_by(&g, bear, 0));
    assert!(g.battlefield_find(bear).unwrap().goaded_by.is_empty(), "no resolved goad — the static holds it");
    g.clear_sickness(bear);
    assert!(declare(&mut g, 1, vec![]).is_err(), "CR 508.1d — a goaded creature attacks if able");
    assert!(declare(&mut g, 1, vec![at(bear, 0)]).is_err(), "CR 701.15b — not the goader while seat 2 is open");
    let mut g2 = g.clone();
    declare(&mut g2, 1, vec![at(bear, 2)]).expect("the non-goader");
    // Back on seat 0's turn, the Blade moves; the goad goes with it.
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, blade, 0, &[Target::Permanent(ogre)], None).expect("re-attach");
    assert!(!goaded(&g, bear) && goaded_by(&g, ogre, 0));
}

/// CR 701.15 — Redemption Arc: indestructible and goaded while attached;
/// {1}{W} exiles the creature.
#[test]
fn cr_701_15_redemption_arc_goads_then_exiles() {
    let mut g = main_phase(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let arc = g.add_card_to_hand(0, catalog::redemption_arc());
    cast_at(&mut g, arc, &[Target::Permanent(bear)]).expect("cast");
    assert!(goaded_by(&g, bear, 0));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Indestructible));
    activate(&mut g, arc, 0, &[], None).expect("exile");
    assert!(g.battlefield_find(bear).is_none() && g.exile.iter().any(|c| c.id == bear));
}

/// The three Impetus Auras goad by static now, so the goad ends with the
/// Aura instead of outliving it until the goader's next turn.
#[test]
fn cr_701_15_impetus_goad_ends_with_the_aura() {
    for def in [catalog::martial_impetus, catalog::shiny_impetus, catalog::ghoulish_impetus] {
        let mut g = main_phase(3);
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, def());
        cast_at(&mut g, aura, &[Target::Permanent(bear)]).expect("cast");
        assert!(goaded_by(&g, bear, 0), "{}", g.battlefield_find(aura).unwrap().definition.name);
        let shatter = g.add_card_to_hand(0, catalog::disenchant());
        cast_at(&mut g, shatter, &[Target::Permanent(aura)]).expect("disenchant");
        assert!(!goaded(&g, bear));
    }
}

/// CR 611.2b / 701.60 — Hot Pursuit: the creature is suspected for good but
/// goaded only while Hot Pursuit remains.
#[test]
fn cr_611_2b_hot_pursuit_goads_while_it_remains() {
    let mut g = main_phase(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hp = g.add_card_to_hand(0, catalog::hot_pursuit());
    cast_at(&mut g, hp, &[]).expect("cast");
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.suspected && goaded_by(&g, bear, 0));
    let shatter = g.add_card_to_hand(0, catalog::disenchant());
    cast_at(&mut g, shatter, &[Target::Permanent(hp)]).expect("disenchant");
    assert!(!goaded(&g, bear) && g.battlefield_find(bear).unwrap().suspected);
}

/// CR 800.4 — once two players have lost, Hot Pursuit's combat trigger takes
/// every goaded or suspected creature for the turn, untapped and hasty.
#[test]
fn cr_800_4_hot_pursuit_steals_once_two_have_lost() {
    let mut g = main_phase(4);
    let bear = g.add_card_to_battlefield(3, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(3, catalog::hill_giant());
    let hp = g.add_card_to_hand(0, catalog::hot_pursuit());
    cast_at(&mut g, hp, &[Target::Permanent(bear)]).expect("cast");
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let mut one_out = g.clone();
    one_out.players[1].eliminated = true;
    one_out.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut one_out);
    assert_eq!(one_out.battlefield_find(bear).unwrap().controller, 3, "one loss is not enough");
    g.players[1].eliminated = true;
    g.players[2].eliminated = true;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.controller, 0);
    assert!(!b.tapped && g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Haste));
    assert_eq!(g.battlefield_find(giant).unwrap().controller, 3, "neither goaded nor suspected");
}

/// Immortal Obligation: the returned creature is its owner's, goaded by the
/// caster while its duty counter stays, and can't attack or block against
/// the caster; without the counter it is free.
#[test]
fn cr_611_2b_immortal_obligation_binds_while_the_duty_counter_stays() {
    let mut g = main_phase(3);
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let io = g.add_card_to_hand(0, catalog::immortal_obligation());
    cast_at(&mut g, io, &[Target::Permanent(bear)]).expect("cast");
    let c = g.battlefield_find(bear).expect("returned");
    assert_eq!((c.controller, c.counter_count(CounterType::Duty)), (1, 1));
    assert!(goaded_by(&g, bear, 0));
    g.clear_sickness(bear);
    assert!(declare(&mut g.clone(), 1, vec![at(bear, 0)]).is_err(), "can't attack the caster");
    assert!(declare(&mut g.clone(), 1, vec![]).is_err(), "goaded: attacks if able");
    // Can't block the caster's creatures.
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut blk = g.clone();
    declare(&mut blk, 0, vec![at(mine, 1)]).expect("attack seat 1");
    blk.step = TurnStep::DeclareBlockers;
    blk.priority.player_with_priority = 1;
    assert!(blk.perform_action(GameAction::DeclareBlockers(vec![(bear, mine)])).is_err());
    g.battlefield_find_mut(bear).unwrap().remove_counters(CounterType::Duty, 1);
    assert!(!goaded(&g, bear));
    declare(&mut g, 1, vec![at(bear, 0)]).expect("free of its duty");
}

/// Agitator Ant: each taker's creature gets two +1/+1 counters and is goaded
/// by the Ant's controller — its own included.
#[test]
fn cr_701_15_agitator_ant_goads_each_takers_creature() {
    let mut g = main_phase(3);
    let ant = g.add_card_to_battlefield(0, catalog::agitator_ant());
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant());
    let declined = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
    ]));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    for id in [ant, theirs] {
        let c = g.battlefield_find(id).unwrap();
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2);
        assert!(goaded_by(&g, id, 0));
    }
    assert!(!goaded(&g, declined));
}

/// Darien: damage to you makes that many Soldiers.
#[test]
fn darien_turns_damage_into_soldiers() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::darien_king_of_kjeldor());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[Target::Player(0)]).expect("bolt");
    assert_eq!(named(&g, 0, "Soldier"), 3);
}

/// Feather: a spell aimed only at Feather is copied onto each chosen other
/// creature for {2} apiece.
#[test]
fn feather_copies_a_pump_at_two_apiece() {
    let mut g = main_phase(2);
    let feather = g.add_card_to_battlefield(0, catalog::feather_radiant_arbiter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(0, catalog::hill_giant());
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    flood(&mut g, 0);
    let before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::CastSpell {
        card_id: growth,
        target: Some(Target::Permanent(feather)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!((power(&g, feather), power(&g, bear), power(&g, other)), (7, 5, 3));
    assert_eq!(before - g.players[0].mana_pool.total(), 1 + 2, "Giant Growth and one copy");
}

/// CR 614.5 — Fiendish Duo doubles damage to opponents, not to their
/// creatures.
#[test]
fn cr_614_5_fiendish_duo_doubles_damage_to_opponents_only() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::fiendish_duo());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, &[Target::Player(1)]).expect("bolt");
    assert_eq!(g.players[1].life, life - 6);
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast_at(&mut g, shock, &[Target::Permanent(giant)]).expect("shock");
    assert!(g.battlefield_find(giant).is_some(), "two damage to a 3/3");
}

/// Havoc Eater: one creature per opponent is goaded, and the Eater grows by
/// their total power.
#[test]
fn havoc_eater_goads_one_per_opponent() {
    let mut g = main_phase(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(2, catalog::hill_giant());
    let he = g.add_card_to_hand(0, catalog::havoc_eater());
    cast_at(&mut g, he, &[]).expect("cast");
    assert!(goaded_by(&g, bear, 0) && goaded_by(&g, giant, 0));
    assert_eq!(g.battlefield_find(he).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
}

/// CR 701.38 — Mob Verdict's secret council: each vote an opponent gets is 2
/// to them and each of their creatures, each vote you get a card.
#[test]
fn cr_701_38_mob_verdict_votes_for_players() {
    let mut g = main_phase(3);
    library(&mut g, 0, 3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(2, catalog::hill_giant());
    let mv = g.add_card_to_hand(0, catalog::mob_verdict());
    // Ballots list the caster's opponents first (most life, then seat), so
    // for every voter option 0 is seat 1 and seat 0 comes last. Seat 0 votes
    // seat 1; seat 1 votes seat 0 (its second option, after seat 2); seat 2
    // votes seat 1.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(0),
        DecisionAnswer::Amount(1),
        DecisionAnswer::Amount(0),
    ]));
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    cast_at(&mut g, mv, &[]).expect("cast");
    assert_eq!((g.players[1].life, g.players[2].life), (l1 - 4, l2));
    assert!(g.battlefield_find(bear).is_none(), "four damage to the 2/2");
    assert!(g.battlefield_find(giant).is_some());
    assert_eq!(g.players[0].hand.len(), 1, "one vote for the caster");
}

/// Nelly Borca: attacking suspects a creature, then goads every suspected
/// creature.
#[test]
fn cr_701_60_nelly_suspects_then_goads_the_suspected() {
    let mut g = main_phase(3);
    let nelly = g.add_card_to_battlefield(0, catalog::nelly_borca_impulsive_accuser());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let prior = g.add_card_to_battlefield(2, catalog::hill_giant());
    g.battlefield_find_mut(prior).unwrap().suspected = true;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    declare(&mut g, 0, vec![at(nelly, 1)]).expect("attack");
    assert!(g.battlefield_find(bear).unwrap().suspected);
    assert!(goaded_by(&g, bear, 0) && goaded_by(&g, prior, 0));
}

/// CR 603.2c — Nelly Borca: an opponent's creatures connecting with another
/// opponent draw you and their controller a card, once per damage step.
#[test]
fn cr_603_2c_nelly_draws_when_opponents_fight() {
    let mut g = main_phase(3);
    library(&mut g, 0, 3);
    library(&mut g, 1, 3);
    g.add_card_to_battlefield(0, catalog::nelly_borca_impulsive_accuser());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    swing(&mut g, 1, vec![at(a, 2), at(b, 2)]);
    assert_eq!((g.players[0].hand.len(), g.players[1].hand.len()), (1, 1));
    let mut g = main_phase(3);
    library(&mut g, 0, 3);
    g.add_card_to_battlefield(0, catalog::nelly_borca_impulsive_accuser());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    swing(&mut g, 1, vec![at(a, 0)]);
    assert_eq!(g.players[0].hand.len(), 0, "hitting you is not hitting an opponent");
}

/// Otherworldly Escort comes back once, as a Spirit Detective with four
/// charge counters, and spends them on creatures that hurt you.
#[test]
fn otherworldly_escort_returns_once_as_a_spirit() {
    let mut g = main_phase(2);
    let oe = g.add_card_to_battlefield(0, catalog::otherworldly_escort());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[Target::Permanent(oe)]).expect("bolt");
    let c = g.battlefield_find(oe).expect("returned");
    assert_eq!(c.counter_count(CounterType::Charge), 4);
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Spirit));
    assert!(!c.definition.subtypes.creature_types.contains(&CreatureType::Human));
    // It dealt damage to us: destroy it.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    swing(&mut g, 1, vec![at(bear, 0)]);
    g.active_player_idx = 0;
    g.step = TurnStep::PostCombatMain;
    g.clear_sickness(oe);
    activate(&mut g, oe, 0, &[Target::Permanent(bear)], None).expect("destroy the attacker");
    assert!(g.battlefield_find(bear).is_none());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[Target::Permanent(oe)]).expect("bolt");
    assert!(g.battlefield_find(oe).is_none(), "a Spirit stays dead");
}

/// Prisoner's Dilemma: a mixed table burns only the silent.
#[test]
fn prisoners_dilemma_burns_the_silent() {
    let mut g = main_phase(3);
    let pd = g.add_card_to_hand(0, catalog::prisoners_dilemma());
    // Option 0 snitches, option 1 stays silent.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(1), DecisionAnswer::Amount(0)]));
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    cast_at(&mut g, pd, &[]).expect("cast");
    assert_eq!((g.players[1].life, g.players[2].life), (l1 - 12, l2));
    let mut g = main_phase(3);
    let pd = g.add_card_to_hand(0, catalog::prisoners_dilemma());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(0), DecisionAnswer::Amount(0)]));
    cast_at(&mut g, pd, &[]).expect("cast");
    assert_eq!((g.players[1].life, g.players[2].life), (l1 - 8, l2 - 8), "everyone snitched");
}

/// Ransom Note: surveil on entry; the sacrifice goads.
#[test]
fn ransom_note_goads_on_the_sacrifice() {
    let mut g = main_phase(3);
    library(&mut g, 0, 2);
    let rn = g.add_card_to_hand(0, catalog::ransom_note());
    cast_at(&mut g, rn, &[]).expect("cast");
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, rn, 0, &[Target::Permanent(bear)], Some(1)).expect("goad mode");
    assert!(goaded_by(&g, bear, 0) && g.battlefield_find(rn).is_none());
}

/// Smuggler's Share: a card per opponent who drew two, a Treasure per
/// opponent who made two land drops.
#[test]
fn smugglers_share_taxes_greedy_opponents() {
    let mut g = main_phase(3);
    library(&mut g, 0, 2);
    library(&mut g, 1, 2);
    g.add_card_to_battlefield(0, catalog::smugglers_share());
    let div = g.add_card_to_hand(1, catalog::divination());
    g.active_player_idx = 1;
    cast_as(&mut g, 1, div, &[]).expect("seat 1 draws two");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(named(&g, 0, "Treasure"), 0);
}

/// Spectacular Showdown: a double strike counter and a goad; overloaded, on
/// every creature.
#[test]
fn spectacular_showdown_goads_the_double_strikers() {
    let mut g = main_phase(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(2, catalog::hill_giant());
    let ss = g.add_card_to_hand(0, catalog::spectacular_showdown());
    cast_at(&mut g, ss, &[Target::Permanent(bear)]).expect("cast");
    assert!(goaded_by(&g, bear, 0) && !goaded(&g, giant));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::DoubleStrike));
    let ss = g.add_card_to_hand(0, catalog::spectacular_showdown());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: ss,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("overload");
    drain_stack(&mut g);
    assert!(goaded_by(&g, giant, 0));
    assert!(g.computed_permanent(giant).unwrap().keywords().contains(&Keyword::DoubleStrike));
}

/// Take the Bait: only in combat on an opponent's turn; prevents the combat
/// damage to you, goads the untapped attackers, and adds a combat.
#[test]
fn take_the_bait_turns_the_attack_around() {
    let mut g = main_phase(3);
    let tb = g.add_card_to_hand(0, catalog::take_the_bait());
    assert!(cast_at(&mut g, tb, &[]).is_err(), "not in your own main phase");
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    declare(&mut g, 1, vec![at(bear, 0)]).expect("attack");
    let life = g.players[0].life;
    cast_as(&mut g, 0, tb, &[]).expect("cast in combat");
    let b = g.battlefield_find(bear).unwrap();
    assert!(!b.tapped && goaded_by(&g, bear, 0));
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, life, "prevented");
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.step, TurnStep::BeginCombat, "the additional combat");
}

/// Vengeful Ancestor: goads on entry, and a goaded creature pings its own
/// controller when it attacks.
#[test]
fn vengeful_ancestor_punishes_goaded_attackers() {
    let mut g = main_phase(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let va = g.add_card_to_hand(0, catalog::vengeful_ancestor());
    cast_at(&mut g, va, &[Target::Permanent(bear)]).expect("cast");
    assert!(goaded_by(&g, bear, 0));
    let life = g.players[1].life;
    declare(&mut g, 1, vec![at(bear, 2)]).expect("attack");
    assert_eq!(g.players[1].life, life - 1);
}

/// Boros Reckoner (in the 99) sends the damage it is dealt at a target it
/// picks; it used to be bound to itself, which looped forever once Redemption
/// Arc made it indestructible (a 1,000-game pod's one undecided game).
#[test]
fn boros_reckoner_redirects_rather_than_hitting_itself() {
    let mut g = main_phase(2);
    let br = g.add_card_to_battlefield(0, catalog::boros_reckoner());
    let arc = g.add_card_to_hand(0, catalog::redemption_arc());
    cast_at(&mut g, arc, &[Target::Permanent(br)]).expect("indestructible");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    let life = g.players[1].life;
    let shock = g.add_card_to_hand(1, catalog::shock());
    cast_as(&mut g, 1, shock, &[Target::Permanent(br)]).expect("shock");
    assert_eq!(g.players[1].life, life - 2);
    assert!(g.stack.is_empty(), "no self-retrigger");
}
