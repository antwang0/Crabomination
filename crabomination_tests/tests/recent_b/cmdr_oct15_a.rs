//! Commander: `decks::cmdr_oct15_a`'s later cards (`012vc8CU`), and the
//! Rulebreaker deck-construction exceptions they print. The run's earlier
//! cards are tested at the end of `cmdr_scan7k_a.rs`.

use crabomination::card::{CardDefinition, CardId};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::format::{validate_commander_deck, CommanderDeckError, Deck, DeckError};
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
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
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn named_count(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// `commander` plus `extra` and enough of `basic` to make `total` cards.
fn deck(commander: CardDefinition, extra: Vec<CardDefinition>, basic: fn() -> CardDefinition, total: usize) -> Deck {
    let mut main = extra;
    while main.len() + 1 < total {
        main.push(basic());
    }
    Deck { main, commanders: vec![commander], sideboard: vec![] }
}

fn off_identity(d: &Deck) -> Vec<&'static str> {
    match validate_commander_deck(d) {
        Ok(()) => vec![],
        Err((_, cmd)) => cmd
            .into_iter()
            .filter_map(|e| match e {
                CommanderDeckError::OffColorIdentity { card_name, .. } => Some(card_name),
                _ => None,
            })
            .collect(),
    }
}

/// Rulebreaker (CR 903.4's exception on Maular): a mana value 7+ creature and
/// a basic land of any color fit; a cheap off-color spell still doesn't.
#[test]
fn maular_rulebreaker_admits_big_creatures_and_basics() {
    let d = deck(
        catalog::maular_the_next_evolution(),
        vec![catalog::stormtide_leviathan(), catalog::island(), catalog::counterspell()],
        catalog::forest,
        100,
    );
    assert_eq!(off_identity(&d), vec!["Counterspell"]);
    let plain = deck(catalog::marwyn_the_nurturer(), vec![catalog::stormtide_leviathan()], catalog::forest, 100);
    assert_eq!(off_identity(&plain), vec!["Stormtide Leviathan"], "no Rulebreaker, no exception");
}

/// Rulebreaker: Seluma's deck may hold Angels of any color; The Everforger's,
/// artifact creatures and Equipment.
#[test]
fn seluma_and_everforger_rulebreakers() {
    let angels = deck(catalog::seluma_light_of_aysen(), vec![catalog::serra_angel(), catalog::swamp()], catalog::plains, 100);
    assert!(off_identity(&angels).is_empty());
    let forge = deck(
        catalog::the_everforger(),
        vec![catalog::myr_enforcer(), catalog::bonesplitter(), catalog::mountain(), catalog::counterspell()],
        catalog::wastes,
        100,
    );
    assert_eq!(off_identity(&forge), vec!["Counterspell"]);
}

/// Rulebreaker (Whtz): no maximum deck size — 120 cards pass, 99 still don't.
#[test]
fn whtz_has_no_maximum_deck_size() {
    let too_many = |d: &Deck| match validate_commander_deck(d) {
        Ok(()) => false,
        Err((generic, _)) => generic.iter().any(|e| matches!(e, DeckError::TooManyCards { .. })),
    };
    let big = deck(catalog::whtz_the_bibliophile(), vec![], catalog::plains, 120);
    assert!(validate_commander_deck(&big).is_ok());
    let small = deck(catalog::whtz_the_bibliophile(), vec![], catalog::plains, 99);
    assert!(validate_commander_deck(&small).is_err(), "still a 100-card floor");
    let other = deck(catalog::seluma_light_of_aysen(), vec![], catalog::plains, 120);
    assert!(too_many(&other));
}

/// Maular doubles a mana value 7+ attacker's power and toughness.
#[test]
fn maular_doubles_a_big_attacker() {
    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::maular_the_next_evolution());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: m, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(pt(&g, m), (20, 20));
}

/// Seluma's combat damage returns an Angel from the graveyard.
#[test]
fn seluma_returns_an_angel_on_combat_damage() {
    let mut g = pod(2);
    let s = ready(&mut g, 0, catalog::seluma_light_of_aysen());
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: s, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(angel).is_some());
}

/// The Everforger copies the first artifact creature spell each turn.
#[test]
fn the_everforger_copies_one_artifact_creature_a_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_everforger());
    flood(&mut g);
    for _ in 0..2 {
        let o = g.add_card_to_hand(0, catalog::ornithopter());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell { card_id: o, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("cast");
        drain_stack(&mut g);
    }
    assert_eq!(named_count(&g, 0, "Ornithopter"), 3, "two cast, one copy");
}

/// CR 614 — Dralnu, Lich Lord: a Bolt at him sacrifices three other
/// permanents instead, and he survives; {T} gives a graveyard Bolt flashback.
#[test]
fn dralnu_sacrifices_instead_of_taking_damage() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let dralnu = ready(&mut g, 0, catalog::dralnu_lich_lord());
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(dralnu)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dralnu).is_some(), "no damage dealt");
    assert_eq!(named_count(&g, 0, "Swamp"), 1, "three Swamps sacrificed");
    let mine = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: dralnu, ability_index: 0, target: Some(Target::Permanent(mine)), additional_targets: vec![], x_value: None, mode: None })
        .expect("grant flashback");
    drain_stack(&mut g);
    let card = g.players[0].graveyard.iter().find(|c| c.id == mine).expect("still in the graveyard");
    assert!(card.granted_flashback_eot.is_some(), "flashback until end of turn");
}

/// CR 614 — combat damage to a blocking Dralnu is replaced the same way.
#[test]
fn dralnu_blocking_sacrifices_for_combat_damage() {
    let mut g = pod(2);
    let dralnu = ready(&mut g, 1, catalog::dralnu_lich_lord());
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::swamp());
    }
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(dralnu, bear)])).expect("block");
    drain_stack(&mut g);
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(dralnu).is_some_and(|c| c.damage == 0), "no damage marked");
    assert_eq!(named_count(&g, 1, "Swamp"), 1, "two Swamps for the Bears' two");
    assert!(g.battlefield_find(bear).is_none(), "Dralnu's 3 kills the Bears");
}
