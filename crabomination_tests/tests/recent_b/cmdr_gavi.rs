//! Commander: the Timeless Wisdom precon (C20, Gavi, Nest Warden,
//! `decks::cmdr_gavi`).

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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
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

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Cycle from an empty pool unless `mana`, then resolve what it triggered.
fn cycle(g: &mut GameState, id: CardId, mana: bool) -> Result<(), String> {
    if mana {
        flood(g, 0);
    } else {
        g.players[0].mana_pool = Default::default();
    }
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::plains());
    }
}

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn in_graveyard(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].graveyard.iter().any(|c| c.id == id)
}

fn in_hand(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].hand.iter().any(|c| c.id == id)
}

fn discard(g: &mut GameState, seat: usize, id: CardId) {
    let mut evs = Vec::new();
    assert!(g.discard_card(seat, id, &mut evs));
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// CR 702.29a / 118.9 — Gavi: the first card cycled each turn may cycle for
/// {0}; the second needs its cost. The second card drawn that turn makes a
/// 2/2 Dinosaur Cat.
#[test]
fn gavi_cycles_the_first_card_free_and_rewards_the_second_draw() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::gavi_nest_warden());
    library(&mut g, 0, 4);
    let a = g.add_card_to_hand(0, catalog::desert_of_the_true());
    let b = g.add_card_to_hand(0, catalog::desert_of_the_mindful());
    cycle(&mut g, a, false).expect("the first cycle is free");
    assert_eq!(count_named(&g, 0, "Dinosaur Cat"), 0, "one card drawn");
    assert!(cycle(&mut g, b, false).is_err(), "the second cycle costs {{1}}{{U}}");
    cycle(&mut g, b, true).expect("paid");
    assert_eq!(count_named(&g, 0, "Dinosaur Cat"), 1, "the second draw");
    assert_eq!(g.players[0].cards_cycled_this_turn, 2);
}

/// New Perspectives: draw three on entry; with seven cards in hand, cycling
/// costs {0}, and not below that.
#[test]
fn new_perspectives_frees_cycling_with_a_full_hand() {
    let mut g = main_phase(2);
    library(&mut g, 0, 6);
    let np = g.add_card_to_hand(0, catalog::new_perspectives());
    cast(&mut g, np, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), 3);
    let few = g.players[0].hand[0].id;
    let desert = g.add_card_to_hand(0, catalog::desert_of_the_true());
    assert!(cycle(&mut g, desert, false).is_err(), "four cards in hand");
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::plains());
    }
    assert_eq!(g.players[0].hand.len(), 7);
    cycle(&mut g, desert, false).expect("free with seven");
    assert!(in_graveyard(&g, 0, desert));
    assert!(in_hand(&g, 0, few));
}

/// CR 614.1a — Abandoned Sarcophagus: a cycling card that dies is exiled, a
/// cycled one reaches the graveyard, and a cycling spell there can be cast —
/// at instant speed with flash, on an opponent's turn.
#[test]
fn abandoned_sarcophagus_exiles_uncycled_cards_and_casts_cycled_ones() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::abandoned_sarcophagus());
    library(&mut g, 0, 2);
    let body = g.add_card_to_battlefield(0, catalog::nimble_obstructionist());
    let kill = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, kill, &[Target::Permanent(body)]).expect("murder");
    assert!(g.exile.iter().any(|c| c.id == body), "died uncycled: exiled");
    let ob = g.add_card_to_hand(0, catalog::nimble_obstructionist());
    cycle(&mut g, ob, true).expect("cycle");
    assert!(in_graveyard(&g, 0, ob), "cycled: kept");
    g.active_player_idx = 1;
    g.step = TurnStep::PostCombatMain;
    cast(&mut g, ob, &[]).expect("flash it from the graveyard");
    assert!(g.battlefield_find(ob).is_some());
    let plain = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    assert!(cast(&mut g, plain, &[]).is_err(), "no cycling: no grant");
}

/// CR 603.4 — Spellpyre Phoenix returns from the graveyard at an end step only
/// after two cycles that turn.
#[test]
fn spellpyre_phoenix_returns_after_two_cycles() {
    let mut g = main_phase(2);
    library(&mut g, 0, 3);
    let phoenix = g.add_card_to_graveyard(0, catalog::spellpyre_phoenix());
    let a = g.add_card_to_hand(0, catalog::desert_of_the_true());
    cycle(&mut g, a, true).expect("cycle");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(in_graveyard(&g, 0, phoenix), "one cycle");
    g.step = TurnStep::PreCombatMain;
    let b = g.add_card_to_hand(0, catalog::desert_of_the_mindful());
    cycle(&mut g, b, true).expect("cycle");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(in_hand(&g, 0, phoenix), "two cycles");
}

/// Spellpyre Phoenix's entry returns an instant or sorcery with cycling.
#[test]
fn spellpyre_phoenix_returns_a_cycling_spell_on_entry() {
    let mut g = main_phase(2);
    let spell = g.add_card_to_graveyard(0, catalog::descend_upon_the_sinful());
    let cyc = g.add_card_to_graveyard(0, catalog::decree_of_justice());
    let p = g.add_card_to_hand(0, catalog::spellpyre_phoenix());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, p, &[Target::Permanent(cyc)]).expect("cast");
    assert!(in_hand(&g, 0, cyc));
    assert!(in_graveyard(&g, 0, spell), "no cycling");
}

/// Drake Haven: a cycle (a discard, CR 702.29a) may pay {1} for a Drake.
#[test]
fn drake_haven_pays_for_a_drake_on_a_cycle() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::drake_haven());
    library(&mut g, 0, 1);
    let d = g.add_card_to_hand(0, catalog::desert_of_the_true());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cycle(&mut g, d, true).expect("cycle");
    assert_eq!(count_named(&g, 0, "Drake"), 1);
}

/// Tectonic Reformation gives land cards cycling {R}; Surly Badgersaur turns a
/// discarded land into a Treasure and a discarded creature into a counter.
#[test]
fn tectonic_reformation_cycles_lands_into_badgersaur_treasure() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::tectonic_reformation());
    let badger = g.add_card_to_battlefield(0, catalog::surly_badgersaur());
    library(&mut g, 0, 1);
    let land = g.add_card_to_hand(0, catalog::plains());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::Cycle { card_id: land, x_value: None }).expect("cycle for {R}");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Treasure"), 1);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    discard(&mut g, 0, bear);
    assert_eq!(g.battlefield_find(badger).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// CR 701.14 — a discarded noncreature, nonland card has Surly Badgersaur
/// fight up to one creature an opponent controls.
#[test]
fn surly_badgersaur_fights_on_a_spell_discard() {
    let mut g = main_phase(2);
    let badger = g.add_card_to_battlefield(0, catalog::surly_badgersaur());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::murder());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    discard(&mut g, 0, spell);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.battlefield_find(badger).unwrap().damage, 2);
}

/// Astral Drift: cycling another card while it's out exiles target creature
/// until the next end step.
#[test]
fn astral_drift_flickers_on_a_cycle() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::astral_drift());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    library(&mut g, 0, 1);
    let d = g.add_card_to_hand(0, catalog::desert_of_the_true());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    cycle(&mut g, d, true).expect("cycle");
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Grizzly Bears"), "exiled");
    g.step = TurnStep::PostCombatMain;
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(count_named(&g, 1, "Grizzly Bears"), 1, "back at the end step");
}

/// Hostile Desert: {2}, exile a land card from your graveyard — a 3/4 until
/// end of turn; with no land card there, it can't.
#[test]
fn hostile_desert_animates_by_exiling_a_land_card() {
    let mut g = main_phase(2);
    let desert = g.add_card_to_battlefield(0, catalog::hostile_desert());
    assert!(activate(&mut g, desert, 1, None).is_err(), "no land card to exile");
    let land = g.add_card_to_graveyard(0, catalog::plains());
    activate(&mut g, desert, 1, None).expect("animate");
    assert!(g.exile.iter().any(|c| c.id == land));
    assert_eq!(pt(&g, desert), (3, 4));
}

/// CR 603.4 — Herald of the Forgotten, cast, returns any number of permanent
/// cards with cycling from your graveyard.
#[test]
fn herald_of_the_forgotten_returns_cycling_permanents() {
    let mut g = main_phase(2);
    let a = g.add_card_to_graveyard(0, catalog::desert_of_the_true());
    let b = g.add_card_to_graveyard(0, catalog::astral_drift());
    let herald = g.add_card_to_hand(0, catalog::herald_of_the_forgotten());
    cast(&mut g, herald, &[Target::Permanent(a), Target::Permanent(b)]).expect("cast");
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some());
}

/// Rooting Moloch exiles a cycling card from your graveyard; you may play it.
#[test]
fn rooting_moloch_lets_you_play_an_exiled_cycling_card() {
    let mut g = main_phase(2);
    let land = g.add_card_to_graveyard(0, catalog::desert_of_the_true());
    let m = g.add_card_to_hand(0, catalog::rooting_moloch());
    cast(&mut g, m, &[Target::Permanent(land)]).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == land));
    g.perform_action(GameAction::PlayLand(land)).expect("play it from exile");
    assert!(g.battlefield_find(land).is_some());
}

/// Brallin: each discard grows it and deals 1 to each opponent — every one at
/// a four-seat table.
#[test]
fn brallin_pings_each_opponent_per_discard() {
    let mut g = main_phase(4);
    let brallin = g.add_card_to_battlefield(0, catalog::brallin_skyshark_rider());
    let c = g.add_card_to_hand(0, catalog::plains());
    let life = g.players[1].life;
    discard(&mut g, 0, c);
    for p in 1..4 {
        assert_eq!(g.players[p].life, life - 1);
    }
    assert_eq!(pt(&g, brallin), (4, 4));
}

/// CR 702.124c — Partner with: Shabraz's entry fetches Brallin; each draw
/// grows it and gains 1.
#[test]
fn shabraz_fetches_its_partner_and_grows_on_draws() {
    let mut g = main_phase(2);
    library(&mut g, 0, 2);
    let brallin = g.add_card_to_library(0, catalog::brallin_skyshark_rider());
    let s = g.add_card_to_hand(0, catalog::shabraz_the_skyshark());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Search(Some(brallin))]));
    cast(&mut g, s, &[Target::Player(0)]).expect("cast");
    assert!(in_hand(&g, 0, brallin));
    let life = g.players[0].life;
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(pt(&g, s), (4, 4));
    assert_eq!(g.players[0].life, life + 1);
}

/// Vizier of Tumbling Sands: cycled, it untaps target permanent.
#[test]
fn vizier_untaps_a_permanent_when_cycled() {
    let mut g = main_phase(2);
    let land = g.add_card_to_battlefield(0, catalog::plains());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    library(&mut g, 0, 1);
    let v = g.add_card_to_hand(0, catalog::vizier_of_tumbling_sands());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(land))]));
    cycle(&mut g, v, true).expect("cycle");
    assert!(!g.battlefield_find(land).unwrap().tapped);
}

/// CR 603.2 / 603.3d — Akim's Bird comes once per turn however many tokens
/// are made; {3}{U}{R}{W} gives creature tokens double strike.
#[test]
fn akim_makes_one_bird_a_turn_and_doubles_tokens() {
    let mut g = main_phase(2);
    let akim = g.add_card_to_battlefield(0, catalog::akim_the_soaring_wind());
    for _ in 0..2 {
        let tok = g.add_card_to_hand(0, catalog::raise_the_alarm());
        cast(&mut g, tok, &[]).expect("two Soldiers");
    }
    assert_eq!(count_named(&g, 0, "Bird"), 1, "once this turn");
    activate(&mut g, akim, 0, None).expect("double strike");
    let soldier = g.battlefield.iter().find(|c| c.definition.name == "Soldier").unwrap().id;
    assert!(g.permanent_has_keyword(soldier, &Keyword::DoubleStrike));
    assert!(!g.permanent_has_keyword(akim, &Keyword::DoubleStrike), "tokens only");
}

/// Descend upon the Sinful exiles every creature; with delirium (CR 207.2c)
/// an Angel is made.
#[test]
fn descend_upon_the_sinful_leaves_an_angel_with_delirium() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for def in [catalog::plains(), catalog::murder(), catalog::sol_ring(), catalog::grizzly_bears()] {
        g.add_card_to_graveyard(0, def);
    }
    let d = g.add_card_to_hand(0, catalog::descend_upon_the_sinful());
    cast(&mut g, d, &[]).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bear));
    assert_eq!(count_named(&g, 0, "Angel"), 1);
}

/// CR 702.66 — Ethereal Forager delves; attacking, it may return a delved
/// instant or sorcery to its owner's hand.
#[test]
fn ethereal_forager_returns_a_delved_spell_on_attack() {
    let mut g = main_phase(2);
    let spell = g.add_card_to_graveyard(0, catalog::murder());
    let f = g.add_card_to_hand(0, catalog::ethereal_forager());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellDelve {
        card_id: f,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        delve_cards: vec![spell],
    })
    .expect("delve");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == spell));
    g.clear_sickness(f);
    g.step = TurnStep::DeclareAttackers;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: f, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(in_hand(&g, 0, spell));
}

/// CR 702.29c — Nimble Obstructionist, cycled, counters an opponent's
/// triggered ability.
#[test]
fn nimble_obstructionist_counters_an_opponents_trigger() {
    let mut g = main_phase(2);
    library(&mut g, 0, 1);
    let theirs = g.add_card_to_battlefield(1, catalog::drake_haven());
    library(&mut g, 1, 1);
    let tc = g.add_card_to_hand(1, catalog::desert_of_the_true());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::Cycle { card_id: tc, x_value: None }).expect("their cycle");
    assert!(!g.stack.is_empty(), "Drake Haven's trigger waits");
    let ob = g.add_card_to_hand(0, catalog::nimble_obstructionist());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(theirs))]));
    cycle(&mut g, ob, true).expect("cycle");
    assert_eq!(count_named(&g, 1, "Drake"), 0, "countered");
}

/// CR 707.2 — Crystalline Resonance, on a cycle, becomes a copy of another
/// permanent and keeps its own trigger.
#[test]
fn crystalline_resonance_copies_on_a_cycle() {
    let mut g = main_phase(2);
    let cr = g.add_card_to_battlefield(0, catalog::crystalline_resonance());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    library(&mut g, 0, 1);
    let d = g.add_card_to_hand(0, catalog::desert_of_the_true());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    cycle(&mut g, d, true).expect("cycle");
    assert_eq!(pt(&g, cr), (2, 2));
    assert_eq!(g.battlefield_find(cr).unwrap().definition.name, "Grizzly Bears");
}
