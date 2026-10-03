//! Commander: the Elven Council precon (LTC, Galadriel,
//! `decks::cmdr_galadriel`).

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

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    act(g, GameAction::CastSpell {
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

fn to_step(g: &mut GameState, step: TurnStep) {
    for _ in 0..20 {
        if g.step == step {
            return;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    panic!("never reached {step:?}");
}

/// Arwen: another creature enters with counters equal to her toughness.
#[test]
fn arwen_weaves_counters() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::arwen_weaver_of_hope());
    let b = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, b, &[]).expect("cast");
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Asceticism gives your creatures hexproof.
#[test]
fn asceticism_hides_your_creatures() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::asceticism());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(has(&g, b, &Keyword::Hexproof));
}

/// Celeborn: attacking with an Elf scries, and the scry grows him.
#[test]
fn celeborn_scries_and_grows() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    let c = g.add_card_to_battlefield(0, catalog::celeborn_the_wise());
    attack(&mut g, &[c], 1);
    assert_eq!(pt(&g, c), (4, 4));
}

/// Colossal Whale exiles a defending creature while it stays.
#[test]
fn colossal_whale_swallows() {
    let mut g = main_phase(2);
    let w = g.add_card_to_battlefield(0, catalog::colossal_whale());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, &[w], 1);
    assert!(g.exile.iter().any(|c| c.id == bear));
}

/// Círdan's secret council: each vote received is a card (CR 701.38).
#[test]
fn cirdan_hands_out_cards_by_vote() {
    let mut g = main_phase(3);
    for s in 0..3 {
        stock(&mut g, s, 3);
    }
    let c = g.add_card_to_hand(0, catalog::cirdan_the_shipwright());
    cast_at(&mut g, c, &[]).expect("cast");
    let hands: usize = (0..3).map(|s| g.players[s].hand.len()).sum();
    assert_eq!(hands, 3, "three votes, three cards");
}

/// Elrond's aid votes grow your team.
#[test]
fn elrond_gathers_aid() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let e = g.add_card_to_hand(0, catalog::elrond_of_the_white_council());
    cast_at(&mut g, e, &[]).expect("cast");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2, "both aid");
}

/// Elrond's fellowship votes: each voter chooses a creature of theirs (its
/// least valuable, headless) and you gain control of it; it can't attack its
/// owner.
#[test]
fn elrond_fellowship_takes_the_voters_choice() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(1), DecisionAnswer::Amount(1)]));
    let e = g.add_card_to_hand(0, catalog::elrond_of_the_white_council());
    cast_at(&mut g, e, &[]).expect("cast");
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(wurm).unwrap().controller, 1);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords().contains(&Keyword::CantAttackOwner));
}

/// Erestor: after a vote, agreeing opponents make Treasures and you draw.
#[test]
fn erestor_rewards_agreement() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    g.add_card_to_battlefield(0, catalog::erestor_of_the_council());
    let e = g.add_card_to_hand(0, catalog::elrond_of_the_white_council());
    let hand = g.players[0].hand.len();
    cast_at(&mut g, e, &[]).expect("cast");
    assert_eq!(tokens(&g, 1, "Treasure"), 1, "the opponent voted aid too");
    assert_eq!(g.players[0].hand.len(), hand, "Elrond left, a card came");
}

/// Galadhrim Ambush: an Elf per attacker, and non-Elves deal no combat damage.
#[test]
fn galadhrim_ambush_springs() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    let a = g.add_card_to_hand(0, catalog::galadhrim_ambush());
    cast_at(&mut g, a, &[]).expect("cast");
    assert_eq!(tokens(&g, 0, "Elf Warrior"), 1);
    let life = g.players[0].life;
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    for _ in 0..4 {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, life);
}

/// Galadriel's will of the council: dominion tempts you with the Ring.
#[test]
fn galadriel_calls_a_council() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    g.add_card_to_battlefield(0, catalog::galadriel_elven_queen());
    let arwen = g.add_card_to_hand(0, catalog::arwen_weaver_of_hope());
    cast_at(&mut g, arwen, &[]).expect("an Elf enters");
    to_step(&mut g, TurnStep::BeginCombat);
    let bearer = g.players[0].ring_bearer.expect("tempted");
    assert!(g.battlefield_find(bearer).unwrap().counter_count(CounterType::PlusOnePlusOne) >= 1);
}

/// Gandalf copies a big spell when an opponent's top card shares a type.
#[test]
fn gandalf_voyages_west() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::gandalf_westward_voyager());
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let whale = g.add_card_to_hand(0, catalog::colossal_whale());
    let mut seen = Vec::new();
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.priority.player_with_priority = 0;
    seen.extend(
        g.perform_action(GameAction::CastSpell { card_id: whale, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("cast"),
    );
    seen.extend(drain_stack(&mut g));
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Colossal Whale").count(), 2);
    assert_eq!(g.players[1].hand.len(), 1, "the opponent drew");
    // CR 701.20 — the opponent's top card is revealed, not just read.
    assert!(seen.iter().any(|e| matches!(e, GameEvent::TopCardRevealed { player: 1, .. })));
}

/// Haldir pumps the other Elves by his counters.
#[test]
fn haldir_leads_the_elves() {
    let mut g = main_phase(2);
    let h = g.add_card_to_hand(0, catalog::haldir_lorien_lieutenant());
    cast_x(&mut g, h, &[], Some(3)).expect("cast");
    let c = g.add_card_to_battlefield(0, catalog::celeborn_the_wise());
    activate(&mut g, h, 0, &[]).expect("pump");
    assert_eq!(pt(&g, c), (6, 6));
    assert!(has(&g, c, &Keyword::Vigilance));
}

/// Learn from the Past shuffles a graveyard away and draws.
#[test]
fn learn_from_the_past_recycles() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 2);
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let l = g.add_card_to_hand(0, catalog::learn_from_the_past());
    cast_at(&mut g, l, &[Target::Player(1)]).expect("cast");
    assert!(g.players[1].graveyard.is_empty());
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Legolas grows when another legend enters.
#[test]
fn legolas_grows_with_legends() {
    let mut g = main_phase(2);
    let l = g.add_card_to_battlefield(0, catalog::legolas_greenleaf());
    let c = g.add_card_to_hand(0, catalog::celeborn_the_wise());
    cast_at(&mut g, c, &[]).expect("cast");
    assert_eq!(g.battlefield_find(l).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Lothlórien Blade: equip an Elf for {2}; attacking, it shoots a creature.
#[test]
fn lothlorien_blade_shoots() {
    let mut g = main_phase(2);
    let elf = g.add_card_to_battlefield(0, catalog::celeborn_the_wise());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let blade = g.add_card_to_battlefield(0, catalog::lothlorien_blade());
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: blade, target: elf }).expect("equip Elf {2}");
    drain_stack(&mut g);
    attack(&mut g, &[elf], 1);
    assert!(g.battlefield_find(foe).is_none());
}

/// Mirkwood Elk returns an Elf and gains life equal to its power.
#[test]
fn mirkwood_elk_recovers_an_elf() {
    let mut g = main_phase(2);
    let c = g.add_card_to_graveyard(0, catalog::celeborn_the_wise());
    let life = g.players[0].life;
    let e = g.add_card_to_hand(0, catalog::mirkwood_elk());
    cast_at(&mut g, e, &[Target::Permanent(c)]).expect("cast");
    assert!(g.players[0].hand.iter().any(|x| x.id == c));
    assert_eq!(g.players[0].life, life + 3);
}

/// Mirkwood Trapper shrinks an attacker coming at you.
#[test]
fn mirkwood_trapper_springs_the_trap() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::mirkwood_trapper());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (0, 2));
}

/// Mirkwood Trapper's second ability (CR 508.1 "whenever a player attacks"):
/// a player attacking only someone else pumps their biggest attacker by 2;
/// attacking you instead, the first ability shrinks it and this one is off.
#[test]
fn mirkwood_trapper_pumps_an_attack_aimed_elsewhere() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::mirkwood_trapper());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(2) }]))
        .expect("attack seat 2");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (4, 2));
}

/// Model of Unity taps for any color.
#[test]
fn model_of_unity_taps_for_mana() {
    let mut g = main_phase(2);
    let m = g.add_card_to_battlefield(0, catalog::model_of_unity());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: m, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("tap");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Radagast: a big spell makes a Beast or a Bird, and Beasts have ward.
#[test]
fn radagast_calls_the_wilds() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::radagast_wizard_of_wilds());
    let whale = g.add_card_to_hand(0, catalog::colossal_whale());
    cast_at(&mut g, whale, &[]).expect("cast");
    assert_eq!(tokens(&g, 0, "Beast") + tokens(&g, 0, "Bird"), 1);
}

/// Sail into the West: return wins the vote, each player takes back cards.
#[test]
fn sail_into_the_west_returns() {
    let mut g = main_phase(2);
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::sail_into_the_west());
    cast_at(&mut g, s, &[]).expect("cast");
    assert_eq!(g.players[1].hand.len(), 1);
    assert!(g.exile.iter().any(|c| c.id == s), "and it's exiled");
}

/// Sail into the West: on embark each player MAY wheel — a "no" keeps its hand.
#[test]
fn sail_into_the_west_embark_is_optional() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 10);
    stock(&mut g, 1, 10);
    let kept = g.add_card_to_hand(1, catalog::craw_wurm());
    let tossed = g.add_card_to_hand(0, catalog::craw_wurm());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(1),
        DecisionAnswer::Amount(1),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Discard(vec![tossed]),
        DecisionAnswer::Bool(false),
    ]));
    let s = g.add_card_to_hand(0, catalog::sail_into_the_west());
    cast_at(&mut g, s, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[1].hand.len(), 1);
    assert!(g.players[1].hand.iter().any(|c| c.id == kept));
}

/// Sail into the West with both seats at a UI: the votes and each player's
/// "may" suspend and resume without replaying one another's answers.
#[test]
fn sail_into_the_west_embark_resumes_for_ui_seats() {
    use crabomination::decision::Decision;
    let mut g = main_phase(2);
    stock(&mut g, 0, 10);
    stock(&mut g, 1, 10);
    let kept = g.add_card_to_hand(1, catalog::craw_wurm());
    g.add_card_to_hand(0, catalog::craw_wurm());
    g.players[0].wants_ui = true;
    g.players[1].wants_ui = true;
    let s = g.add_card_to_hand(0, catalog::sail_into_the_west());
    cast_at(&mut g, s, &[]).expect("cast");
    for _ in 0..20 {
        let Some(p) = g.pending_decision.as_ref() else { break };
        let seat = p.acting_player();
        let answer = match &p.decision {
            Decision::ChooseOption { .. } => DecisionAnswer::Amount(1),
            Decision::Discard { .. } => DecisionAnswer::Discard(g.players[seat].hand.iter().map(|c| c.id).collect()),
            _ => DecisionAnswer::Bool(seat == 0),
        };
        g.submit_decision(answer).expect("answer");
        drain_stack(&mut g);
    }
    assert!(g.pending_decision.is_none());
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[1].hand.len(), 1);
    assert!(g.players[1].hand.iter().any(|c| c.id == kept));
}

/// Song of Eärendil's first chapter scries and draws two.
#[test]
fn song_of_earendil_sings() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 4);
    let s = g.add_card_to_hand(0, catalog::song_of_earendil());
    cast_at(&mut g, s, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Trap the Trespassers stuns the creature everyone voted for.
#[test]
fn trap_the_trespassers_stuns() {
    let mut g = main_phase(3);
    let big = g.add_card_to_battlefield(1, catalog::colossal_whale());
    let t = g.add_card_to_hand(0, catalog::trap_the_trespassers());
    cast_at(&mut g, t, &[]).expect("cast");
    let c = g.battlefield_find(big).unwrap();
    assert!(c.tapped);
    assert_eq!(c.counter_count(CounterType::Stun), 3, "one per vote");
}

/// Travel Through Caradhras: Redhorn Pass votes fetch basics tapped.
#[test]
fn travel_through_caradhras_finds_lands() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let t = g.add_card_to_hand(0, catalog::travel_through_caradhras());
    cast_at(&mut g, t, &[]).expect("cast");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Forest" && c.tapped).count(), 2);
    assert!(g.exile.iter().any(|c| c.id == t));
}

/// Windswift Slice: excess damage becomes Elf Warriors.
#[test]
fn windswift_slice_cuts_through() {
    let mut g = main_phase(2);
    let whale = g.add_card_to_battlefield(0, catalog::colossal_whale());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let w = g.add_card_to_hand(0, catalog::windswift_slice());
    cast_at(&mut g, w, &[Target::Permanent(whale), Target::Permanent(bear)]).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(tokens(&g, 0, "Elf Warrior"), 3, "5 damage into a 2-toughness Bear");
}
