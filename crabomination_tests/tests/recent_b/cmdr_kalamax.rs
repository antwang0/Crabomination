//! Commander: the Arcane Maelstrom precon (C20, Kalamax, `decks::cmdr_kalamax`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, targets: &[Target], x: Option<u32>) {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_by(g, 0, id, targets, None);
}

fn lost(g: &GameState, s: usize) -> i32 {
    g.players[s].starting_life - g.players[s].life
}

/// Kalamax, tapped, copies the first instant each turn (CR 707.10), and the
/// copy grows it; Twinning Staff adds one more copy.
#[test]
fn kalamax_copies_and_grows() {
    let mut g = pod(2);
    let k = g.add_card_to_battlefield(0, catalog::kalamax_the_stormsire());
    g.battlefield_find_mut(k).unwrap().tapped = true;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]);
    assert_eq!(lost(&g, 1), 6, "the bolt and its copy");
    assert_eq!(g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let mut g = pod(2);
    let k = g.add_card_to_battlefield(0, catalog::kalamax_the_stormsire());
    g.battlefield_find_mut(k).unwrap().tapped = true;
    g.add_card_to_battlefield(0, catalog::twinning_staff());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]);
    assert_eq!(lost(&g, 1), 9, "the bolt and two copies");
}

/// Whiplash Trap's {U} alternative cost needs an opponent's two creature
/// entries this turn.
#[test]
fn whiplash_trap_alt_cost_condition() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wt = g.add_card_to_hand(0, catalog::whiplash_trap());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let alt = |g: &mut GameState| {
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: wt,
            pitch_card: None,
            target: Some(Target::Permanent(a)),
            additional_targets: vec![Target::Permanent(b)],
            mode: None,
            x_value: None,
        })
    };
    assert!(alt(&mut g).is_err(), "nothing entered this turn");
    g.players[1].creatures_entered_this_turn.push(a);
    g.players[1].creatures_entered_this_turn.push(b);
    alt(&mut g).expect("alt cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Rashmi: the first spell each turn reveals the top; a cheaper spell is cast
/// free (CR 601.2), a land goes to hand.
#[test]
fn rashmi_casts_the_cheaper_top() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::rashmi_eternities_crafter());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    cast(&mut g, wurm, &[]);
    assert!(g.battlefield_find(bear).is_some() || g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Nascent Metamorph attacking becomes a copy of the opponent's first
/// creature card from the top.
#[test]
fn nascent_metamorph_shifts() {
    let mut g = pod(2);
    let nm = g.add_card_to_battlefield(0, catalog::nascent_metamorph());
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::craw_wurm());
    g.clear_sickness(nm);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: nm,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(nm).map(|c| (c.power, c.toughness)), Some((6, 4)));
}

/// Glademuse draws a caster a card off their own turn.
#[test]
fn glademuse_rewards_off_turn_spells() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::glademuse());
    g.add_card_to_library(1, catalog::island());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_by(&mut g, 1, bolt, &[Target::Player(0)], None);
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Curious Herd counts the target opponent's artifacts.
#[test]
fn curious_herd_counts_artifacts() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_battlefield(1, catalog::ornithopter());
    }
    let ch = g.add_card_to_hand(0, catalog::curious_herd());
    cast(&mut g, ch, &[Target::Player(1)]);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Beast").count(), 2);
}

/// Wort grants conspire (CR 702.78): a red instant copies off two tapped
/// red creatures.
#[test]
fn wort_grants_conspire() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::wort_the_raidmother());
    let goblins: Vec<CardId> = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Goblin Warrior")
        .map(|c| c.id)
        .collect();
    let (a, b) = if goblins.len() >= 2 {
        (goblins[0], goblins[1])
    } else {
        (
            g.add_card_to_battlefield(0, catalog::goblin_guide()),
            g.add_card_to_battlefield(0, catalog::goblin_guide()),
        )
    };
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellConspire {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
        conspire_creatures: [a, b],
    })
    .expect("conspire");
    drain_stack(&mut g);
    assert_eq!(lost(&g, 1), 6);
}

/// CR 702.16 — Eon Frolicker cast: the target opponent takes the extra turn,
/// and until your next turn you have protection from that player: its bolt
/// can't target you, the other opponent's can.
#[test]
fn cr_702_16_eon_frolicker_protects_you_from_the_extra_turn_player() {
    let mut g = pod(3);
    let eon = g.add_card_to_hand(0, catalog::eon_frolicker());
    cast(&mut g, eon, &[]);
    assert!(g.battlefield_find(eon).is_some());
    let from: Vec<usize> = (1..3).filter(|&q| g.players[0].protected_from_seat(q)).collect();
    assert_eq!(from.len(), 1, "protection from exactly the target opponent");
    for seat in [1, 2] {
        let bolt = g.add_card_to_hand(seat, catalog::lightning_bolt());
        flood(&mut g, seat);
        g.priority.player_with_priority = seat;
        let r = g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        });
        assert_eq!(r.is_err(), seat == from[0], "seat {seat}");
        drain_stack(&mut g);
    }
}

/// Lavabrink Floodgates: each upkeep the active player may put a doom
/// counter on it or remove one; at three it is sacrificed for 6 damage to
/// each creature.
#[test]
fn lavabrink_floodgates_doom_counter_either_way() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let setup = |answer: u32| {
        let mut g = pod(2);
        let gates = g.add_card_to_battlefield(0, catalog::lavabrink_floodgates());
        g.battlefield_find_mut(gates).unwrap().add_counters(CounterType::Doom, 2);
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(answer)]));
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        (g, gates, bear)
    };
    let (g, gates, bear) = setup(2);
    assert_eq!(g.battlefield_find(gates).unwrap().counter_count(CounterType::Doom), 1, "removed one");
    assert!(g.battlefield_find(bear).is_some());
    let (g, gates, bear) = setup(1);
    assert!(g.battlefield_find(gates).is_none(), "the third counter sacrifices it");
    assert!(g.battlefield_find(bear).is_none(), "6 damage to each creature");
}

/// Deflecting Swat — CR 115.7d "target spell or ability": an opponent's
/// Prodigal Sorcerer ping at our Bear, named by its stack id (CR 115.1),
/// goes back at the pinger's controller.
#[test]
fn deflecting_swat_redirects_an_ability() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(4);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pinger = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
    g.clear_sickness(pinger);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pinger, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("ping the bear");
    let ping = g.top_ability_of(pinger).expect("the ping is on the stack");
    let swat = g.add_card_to_hand(0, catalog::deflecting_swat());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: swat, target: Some(Target::Permanent(ping)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("swat the ping");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).map(|c| c.damage), Some(0), "the Bear was spared");
    assert_eq!(g.players[1].life, life - 1, "the ping hit its controller");
}

/// Pako, Arcane Retriever: attacking, it exiles the top card of each library
/// WITH A FETCH COUNTER on each and grows per NONCREATURE card; with Haldan
/// out the noncreature one is yours to cast, the creature card isn't.
#[test]
fn pako_exiles_with_fetch_counters_and_grows_per_noncreature() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = pod(2);
    let pako = g.add_card_to_battlefield(0, catalog::pako_arcane_retriever());
    g.add_card_to_battlefield(0, catalog::haldan_avid_arcanist());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    g.clear_sickness(pako);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: pako, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    for id in [bolt, bear] {
        let c = g.exile.iter().find(|c| c.id == id).expect("exiled");
        assert_eq!(c.counter_count(CounterType::Fetch), 1, "a fetch counter on each");
    }
    assert_eq!(g.battlefield_find(pako).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "one noncreature card");
    assert!(g.exile.iter().find(|c| c.id == bolt).unwrap().may_play_until.is_some(), "the Bolt is castable");
    assert!(g.exile.iter().find(|c| c.id == bear).unwrap().may_play_until.is_none(), "the creature card isn't");
}

/// CR 611.3a — Haldan's permission is its static's: Pako's exiles wait parked
/// until a Haldan arrives, and park again when it leaves.
#[test]
fn haldan_wakes_and_parks_pakos_grants() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = pod(2);
    let pako = g.add_card_to_battlefield(0, catalog::pako_arcane_retriever());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(1, catalog::island());
    g.clear_sickness(pako);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: pako, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let seat = |g: &GameState| g.exile.iter().find(|c| c.id == bolt).and_then(|c| c.may_play_until).map(|m| m.player);
    assert_eq!(seat(&g), Some(crabomination::card::MAY_PLAY_DORMANT), "no Haldan yet");
    g.step = TurnStep::PostCombatMain;
    let haldan = g.add_card_to_hand(0, catalog::haldan_avid_arcanist());
    cast(&mut g, haldan, &[]);
    assert_eq!(seat(&g), Some(0), "Haldan arrived");
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, &[Target::Permanent(haldan)]);
    assert!(g.battlefield_find(haldan).is_none());
    assert_eq!(seat(&g), Some(crabomination::card::MAY_PLAY_DORMANT), "Haldan left");
}
