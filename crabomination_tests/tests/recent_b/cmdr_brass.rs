//! Commander: the Ahoy Mateys precon (LCC, Admiral Brass,
//! `decks::cmdr_brass`).

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

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    act(g, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    act(g, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
}

fn stock(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::grizzly_bears());
    }
}

fn tokens(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.is_token && c.definition.name == name).count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn has(g: &GameState, id: CardId, k: &Keyword) -> bool {
    g.computed_permanent(id).expect("on the battlefield").keywords().contains(k)
}

fn attack(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for a in attackers {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn connect(g: &mut GameState, attackers: &[CardId], defender: usize) {
    attack(g, attackers, defender);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(g);
    for _ in 0..6 {
        if g.step == TurnStep::PostCombatMain {
            break;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// Admiral Beckett Brass: other Pirates +1/+1; three Pirates connecting
/// earn a steal at the end step.
#[test]
fn admiral_beckett_brass_takes_a_prize() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::admiral_beckett_brass());
    let crew: Vec<CardId> = (0..3).map(|_| g.add_card_to_battlefield(0, catalog::warkite_marauder())).collect();
    assert_eq!(pt(&g, crew[0]), (3, 2));
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    connect(&mut g, &crew, 1);
    for _ in 0..4 {
        if g.step == TurnStep::End {
            break;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ring).unwrap().controller, 0);
}

/// Admiral Brass returns a Pirate as a hasty 4/4 with a finality counter.
#[test]
fn admiral_brass_raises_the_crew() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::admiral_brass_unsinkable());
    let w = g.add_card_to_graveyard(0, catalog::warkite_marauder());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    let c = g.battlefield_find(w).expect("returned");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
    assert_eq!(pt(&g, w), (4, 4));
    assert!(has(&g, w, &Keyword::Haste));
}

/// Arm-Mounted Anchor: +2/+2 and menace, draw two on connecting.
#[test]
fn arm_mounted_anchor_hauls_in() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 4);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let a = g.add_card_to_battlefield(0, catalog::arm_mounted_anchor());
    act(&mut g, GameAction::Equip { equipment: a, target: bear }).expect("equip");
    assert_eq!(pt(&g, bear), (4, 4));
    assert!(has(&g, bear, &Keyword::Menace));
    let lib = g.players[0].library.len();
    connect(&mut g, &[bear], 1);
    assert_eq!(g.players[0].library.len(), lib - 2);
}

/// Azure Fleet Admiral: you become the monarch; the monarch's creatures
/// can't block it.
#[test]
fn azure_fleet_admiral_crowns_you() {
    let mut g = main_phase(2);
    let a = g.add_card_to_hand(0, catalog::azure_fleet_admiral());
    cast_at(&mut g, a, &[]).expect("cast");
    assert_eq!(g.monarch, Some(0));
    g.monarch = Some(1);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    attack(&mut g, &[a], 1);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, a)])).is_err(),
        "the monarch's Bears can't block it"
    );
}

/// Blood Money: a tapped Treasure per nontoken creature destroyed.
#[test]
fn blood_money_pays_out() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_hand(0, catalog::blood_money());
    cast_at(&mut g, b, &[]).expect("cast");
    assert_eq!(tokens(&g, 0, "Treasure"), 2);
    assert!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").all(|c| c.tapped));
}

/// Breeches: a Pirate connecting exiles the opponent's top card to play.
#[test]
fn breeches_plunders() {
    let mut g = main_phase(2);
    stock(&mut g, 1, 3);
    let b = g.add_card_to_battlefield(0, catalog::breeches_brazen_plunderer());
    connect(&mut g, &[b], 1);
    assert!(g.exile.iter().any(|c| c.owner == 1 && c.may_play_until.is_some()));
}

/// Coercive Recruiter steals a creature as a hasty Pirate.
#[test]
fn coercive_recruiter_recruits() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::coercive_recruiter());
    cast_at(&mut g, c, &[Target::Permanent(bear)]).expect("cast");
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.controller, 0);
    assert!(g.computed_permanent(bear).unwrap().subtypes().creature_types.contains(&CreatureType::Pirate));
}

/// Departed Deckhand is sacrificed when a spell targets it.
#[test]
fn departed_deckhand_departs() {
    let mut g = main_phase(2);
    let d = g.add_card_to_battlefield(0, catalog::departed_deckhand());
    let b = g.add_card_to_hand(0, catalog::coercive_recruiter());
    let _ = b;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, &[Target::Permanent(d)]).expect("bolt it");
    assert!(g.battlefield_find(d).is_none());
    assert!(g.players[0].graveyard.iter().any(|c| c.id == d));
}

/// Don Andres pumps the creatures you've stolen.
#[test]
fn don_andres_empowers_the_stolen() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::don_andres_the_renegade());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().controller = 0;
    assert_eq!(pt(&g, bear), (4, 4));
    assert!(has(&g, bear, &Keyword::Deathtouch));
}

/// Fathom Fleet Captain recruits beside another nontoken Pirate.
#[test]
fn fathom_fleet_captain_recruits() {
    let mut g = main_phase(2);
    let f = g.add_card_to_battlefield(0, catalog::fathom_fleet_captain());
    g.add_card_to_battlefield(0, catalog::warkite_marauder());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    attack(&mut g, &[f], 1);
    assert_eq!(tokens(&g, 0, "Pirate"), 1);
}

/// Francisco explores when Pirates connect.
#[test]
fn francisco_explores() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    let f = g.add_card_to_battlefield(0, catalog::francisco_fowl_marauder());
    let w = g.add_card_to_battlefield(0, catalog::warkite_marauder());
    connect(&mut g, &[w], 1);
    let explored = g.players[0].hand.len() == 1 || g.battlefield_find(f).unwrap().counter_count(CounterType::PlusOnePlusOne) == 1;
    assert!(explored);
}

/// Gemcutter Buccaneer: Pirates entering make tapped Treasures that equip.
#[test]
fn gemcutter_buccaneer_cuts_gems() {
    let mut g = main_phase(2);
    let gb = g.add_card_to_hand(0, catalog::gemcutter_buccaneer());
    cast_at(&mut g, gb, &[]).expect("cast");
    assert_eq!(tokens(&g, 0, "Treasure"), 1);
    let t = g.battlefield.iter().find(|c| c.definition.name == "Treasure").unwrap().id;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    act(&mut g, GameAction::Equip { equipment: t, target: bear }).expect("equip the Treasure");
    assert_eq!(pt(&g, bear), (4, 2));
}

/// Ghost of Ramirez returns a card milled this turn.
#[test]
fn ghost_of_ramirez_recovers() {
    let mut g = main_phase(2);
    let ghost = g.add_card_to_battlefield(0, catalog::ghost_of_ramirez_depietro());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let mill = catalog::admiral_brass_unsinkable();
    let _ = mill;
    let m = g.players[0].library[0].id;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&crabomination::effect::Effect::Mill { who: crabomination::effect::Selector::You, amount: crabomination::card::Value::ONE }, &ctx)
        .unwrap();
    assert!(g.players[0].graveyard.iter().any(|c| c.id == m));
    connect(&mut g, &[ghost], 1);
    assert!(g.players[0].hand.iter().any(|c| c.id == m));
}

/// King Narfi's Betrayal: chapter I mills four each and exiles a creature
/// card from each graveyard.
#[test]
fn king_narfis_betrayal_betrays() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 4);
    stock(&mut g, 1, 4);
    let k = g.add_card_to_hand(0, catalog::king_narfis_betrayal());
    cast_at(&mut g, k, &[]).expect("cast");
    assert_eq!(g.exile.iter().filter(|c| c.exiled_with == Some(k)).count(), 2);
}

/// Merchant Raiders taps down a creature.
#[test]
fn merchant_raiders_lock_a_creature() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::merchant_raiders());
    cast_at(&mut g, m, &[Target::Permanent(bear)]).expect("cast");
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Ramirez DePietro: lose 2, two Treasures.
#[test]
fn ramirez_depietro_pillages() {
    let mut g = main_phase(2);
    let life = g.players[0].life;
    let r = g.add_card_to_hand(0, catalog::ramirez_depietro_pillager());
    cast_at(&mut g, r, &[]).expect("cast");
    assert_eq!(g.players[0].life, life - 2);
    assert_eq!(tokens(&g, 0, "Treasure"), 2);
}

/// Skeleton Crew returns itself from the graveyard, tapped.
#[test]
fn skeleton_crew_rises() {
    let mut g = main_phase(2);
    let s = g.add_card_to_graveyard(0, catalog::skeleton_crew());
    activate(&mut g, s, 0, &[]).expect("from the graveyard");
    assert!(g.battlefield_find(s).is_some_and(|c| c.tapped));
}

/// Storm Fleet Negotiator parleys: Maps per nonland, then everyone draws.
#[test]
fn storm_fleet_negotiator_parleys() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 2);
    stock(&mut g, 1, 2);
    let s = g.add_card_to_battlefield(0, catalog::storm_fleet_negotiator());
    attack(&mut g, &[s], 1);
    assert_eq!(tokens(&g, 0, "Map"), 2);
    assert_eq!(g.players[1].hand.len(), 1);
}

/// The Indomitable crews into a 6/6 trampler.
#[test]
fn the_indomitable_sails() {
    let mut g = main_phase(2);
    let i = g.add_card_to_battlefield(0, catalog::the_indomitable());
    assert!(g.computed_permanent(i).is_some());
    assert!(has(&g, i, &Keyword::Trample));
}

/// Timestream Navigator needs the city's blessing.
#[test]
fn timestream_navigator_needs_the_blessing() {
    let mut g = main_phase(2);
    let t = g.add_card_to_battlefield(0, catalog::timestream_navigator());
    assert!(activate(&mut g, t, 0, &[]).is_err(), "no blessing yet");
}

/// Warkite Marauder shrinks a defender to a 0/1 with no abilities.
#[test]
fn warkite_marauder_strips_a_defender() {
    let mut g = main_phase(2);
    let w = g.add_card_to_battlefield(0, catalog::warkite_marauder());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    attack(&mut g, &[w], 1);
    assert_eq!(pt(&g, angel), (0, 1));
    assert!(!has(&g, angel, &Keyword::Flying));
}

/// Zara recruits a creature from the defending player's hand, attacking.
#[test]
fn zara_recruits_from_their_hand() {
    let mut g = main_phase(2);
    let z = g.add_card_to_battlefield(0, catalog::zara_renegade_recruiter());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, &[z], 1);
    let b = g.battlefield_find(bear).expect("recruited");
    assert_eq!(b.controller, 0);
    assert!(g.attacking.iter().any(|a| a.attacker == bear));
}

/// The Grim Captain's Locker's second {T}: creature cards in your graveyard
/// gain escape {3}{B} (exile four others) until end of turn.
#[test]
fn grim_captains_locker_grants_escape() {
    let mut g = main_phase(2);
    let locker = g.add_card_to_battlefield(0, catalog::the_grim_captains_locker());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let fodder: Vec<CardId> = (0..4).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
    activate(&mut g, locker, 1, &[]).expect("grant escape");
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    act(&mut g, GameAction::CastEscape {
        card_id: giant,
        exile_cards: fodder,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("escape for {3}{B}");
    crabomination::game::drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_some());
}
