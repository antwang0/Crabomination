//! Commander: one-card gaps of EDHREC ranks 2501-3500
//! (`decks::cmdr_rank3k_h`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::Effect;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    );
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    );
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne)
}

fn resolve_at(
    g: &mut GameState,
    effect: &Effect,
    source: CardId,
    target: Option<Target>,
    mode: usize,
) {
    let mut ctx = EffectContext::for_spell(0, target, mode, 0);
    ctx.source = Some(source);
    let evs = g.resolve_effect(effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}
/// CR 601.2f — No One Left Behind costs {3} less for a cheap creature card.
#[test]
fn no_one_left_behind_reanimates_cheaply() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::no_one_left_behind());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("two mana is enough");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
}

/// CR 122.1 — The Darkhold draws per sin counter, then charges for the hand.
#[test]
fn the_darkhold_draws_and_drains() {
    let mut g = pod(2);
    let book = ready(&mut g, 0, catalog::the_darkhold());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    activate(&mut g, book, 0, None);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].life, 19);
    g.battlefield_find_mut(book).unwrap().tapped = false;
    activate(&mut g, book, 0, None);
    assert_eq!(g.players[0].hand.len(), 3);
    assert_eq!(g.players[0].life, 16);
}

/// CR 119.5 — The Darkhold's second ability sets every life to the lowest.
#[test]
fn the_darkhold_levels_life_totals() {
    let mut g = pod(3);
    let book = ready(&mut g, 0, catalog::the_darkhold());
    g.players[2].life = 7;
    activate(&mut g, book, 1, None);
    assert!(g.players.iter().all(|p| p.life == 7));
}

/// CR 111.1 — Reckless Crew makes a Dwarf per Vehicle and Equipment and can
/// suit them up.
#[test]
fn reckless_crew_counts_and_equips() {
    let mut g = pod(2);
    let sword = ready(&mut g, 0, catalog::butchers_cleaver());
    ready(&mut g, 0, catalog::smugglers_copter());
    let crew = g.add_card_to_hand(0, catalog::reckless_crew());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    cast(&mut g, crew, None);
    assert_eq!(named(&g, 0, "Dwarf Berserker"), 2);
    let host = g
        .battlefield_find(sword)
        .unwrap()
        .attached_to
        .expect("attached");
    assert_eq!(
        g.battlefield_find(host).unwrap().definition.name,
        "Dwarf Berserker"
    );
    assert_eq!(pt(&g, host), (5, 1));

    // An Equipment already on another creature moves to a Dwarf: the printed
    // text says "an Equipment you control", not an unattached one.
    let mut g = pod(2);
    let cleaver = ready(&mut g, 0, catalog::butchers_cleaver());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(cleaver).unwrap().attached_to = Some(bear);
    let crew = g.add_card_to_hand(0, catalog::reckless_crew());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, crew, None);
    let host = g.battlefield_find(cleaver).unwrap().attached_to.expect("still attached");
    assert_eq!(g.battlefield_find(host).unwrap().definition.name, "Dwarf Berserker", "moved onto the Dwarf");
}

/// CR 702.84a — Mishra's Research Desk impulses two cards and unearths.
#[test]
fn mishras_research_desk_impulses_and_unearths() {
    let mut g = pod(2);
    let desk = ready(&mut g, 0, catalog::mishras_research_desk());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    activate(&mut g, desk, 0, None);
    assert_eq!(g.exile.len(), 2);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == desk));
    activate(&mut g, desk, 1, None);
    assert!(g.battlefield_find(desk).is_some(), "unearthed");
}

/// CR 714.2 — Roads Go Ever, Ever On exiles two Plains and returns them.
#[test]
fn roads_go_ever_ever_on_fetches_plains() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::plains());
    }
    g.add_card_to_library(0, catalog::island());
    let saga = g.add_card_to_hand(0, catalog::roads_go_ever_ever_on());
    cast(&mut g, saga, None);
    assert_eq!(g.players[0].life, 22);
    assert_eq!(
        g.exile
            .iter()
            .filter(|c| c.definition.name == "Plains")
            .count(),
        2
    );
    let ch2 = catalog::roads_go_ever_ever_on().saga_chapters[1].1.clone();
    resolve_at(&mut g, &ch2, saga, None, 0);
    assert_eq!(
        g.players[0]
            .hand
            .iter()
            .filter(|c| c.definition.name == "Plains")
            .count(),
        1
    );
}

/// CR 606.3 — Ajani's +1 takes a creature; −3 spreads counters with vigilance.
#[test]
fn ajani_sleeper_agent_digs_and_distributes() {
    let mut g = pod(2);
    let ajani = ready(&mut g, 0, catalog::ajani_sleeper_agent());
    g.add_card_to_library(0, catalog::grizzly_bears());
    act(
        &mut g,
        GameAction::ActivateLoyaltyAbility {
            card_id: ajani,
            ability_index: 0,
            target: None,
            x_value: None,
        },
    );
    assert_eq!(g.players[0].hand.len(), 1);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let minus = catalog::ajani_sleeper_agent().loyalty_abilities[1]
        .effect
        .clone();
    resolve_at(&mut g, &minus, ajani, Some(Target::Permanent(bear)), 0);
    assert_eq!(plus(&g, bear), 3);
    assert!(
        g.computed_permanent(bear)
            .unwrap()
            .keywords()
            .contains(&Keyword::Vigilance)
    );
}

/// CR 613.1b — Magus of the Unseen borrows an opponent's artifact with haste.
#[test]
fn magus_of_the_unseen_borrows_an_artifact() {
    let mut g = pod(2);
    let magus = ready(&mut g, 0, catalog::magus_of_the_unseen());
    let ring = ready(&mut g, 1, catalog::sol_ring());
    g.battlefield_find_mut(ring).unwrap().tapped = true;
    activate(&mut g, magus, 0, Some(Target::Permanent(ring)));
    let c = g.battlefield_find(ring).unwrap();
    assert_eq!(c.controller, 0);
    assert!(!c.tapped);
    // "When you lose control of the artifact, tap it" — as the steal ends in
    // the cleanup step (CR 514.2).
    while g.step != TurnStep::End {
        g.advance_step(vec![]).expect("advance");
        drain_stack(&mut g);
    }
    g.advance_step(vec![]).expect("to cleanup");
    drain_stack(&mut g);
    let c = g.battlefield_find(ring).unwrap();
    assert_eq!(c.controller, 1, "back to its owner");
    assert!(c.tapped, "tapped when control was lost");
}
