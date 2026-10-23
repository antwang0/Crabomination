//! Commander: two-card gaps of EDHREC ranks 5501-6500
//! (`decks::cmdr_rank6k_b`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
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

fn kill(g: &mut GameState, id: CardId) {
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(g, murder, Some(Target::Permanent(id)));
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks
            .into_iter()
            .map(|(attacker, p)| Attack {
                attacker,
                target: AttackTarget::Player(p),
            })
            .collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn finish_combat(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![]))
        .expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}
/// CR 702.193 — White Tiger's power-up summons The Tiger God; Greenhouse
/// Propagator gains life as it enters.
#[test]
fn white_tiger_summons_the_tiger_god() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::greenhouse_propagator());
    let wt = ready(&mut g, 0, catalog::white_tiger_ava_ayala());
    activate(&mut g, wt, 0, None);
    assert_eq!(plus(&g, wt), 1);
    let god = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "The Tiger God")
        .expect("token")
        .id;
    assert!(
        g.computed_permanent(god)
            .unwrap()
            .keywords()
            .contains(&Keyword::CantBeBlockedByMoreThanOne)
    );
    assert_eq!(g.players[0].life, 21);
}

/// CR 107.3 — Kayla's Reconstruction puts up to X cheap artifacts/creatures
/// onto the battlefield.
#[test]
fn kaylas_reconstruction_deploys_x() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.add_card_to_library(0, catalog::serra_angel());
    let k = g.add_card_to_hand(0, catalog::kaylas_reconstruction());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: k,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Grizzly Bears"), 2);
    assert_eq!(named(&g, 0, "Serra Angel"), 0, "mana value 5");
}

/// CR 602.5b — Fixer draws only once an artifact entered this turn; Glaze
/// Fiend pumps on it.
#[test]
fn fixer_needs_an_artifact_this_turn() {
    let mut g = pod(2);
    let fixer = ready(&mut g, 0, catalog::fixer_techno_terror());
    let fiend = ready(&mut g, 0, catalog::glaze_fiend());
    g.add_card_to_library(0, catalog::island());
    flood(&mut g);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: fixer,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None
        })
        .is_err()
    );
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None);
    assert_eq!(pt(&g, fiend), (2, 3));
    activate(&mut g, fixer, 0, None);
    assert_eq!(g.players[0].life, 18);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// CR 702.193 — Goliath's power-up draws per creature with power 4+.
#[test]
fn goliath_draws_for_big_creatures() {
    let mut g = pod(2);
    let gol = ready(&mut g, 0, catalog::goliath_mass_manipulator());
    ready(&mut g, 0, catalog::garruks_harbinger());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    activate(&mut g, gol, 0, None);
    assert_eq!(pt(&g, gol), (4, 4));
    assert_eq!(g.players[0].hand.len(), 2, "Goliath and the Harbinger");
}

/// CR 702.11d — Garruk's Harbinger has hexproof from black and digs on hits.
#[test]
fn garruks_harbinger_digs_on_damage() {
    let mut g = pod(2);
    let h = ready(&mut g, 0, catalog::garruks_harbinger());
    assert!(
        g.computed_permanent(h)
            .unwrap()
            .keywords()
            .contains(&Keyword::HexproofFromColor(Color::Black))
    );
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    declare(&mut g, vec![(h, 1)]);
    finish_combat(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Garruk's Harbinger digs off combat damage to a planeswalker too ("a
/// player or planeswalker").
#[test]
fn garruks_harbinger_digs_off_a_planeswalker_hit() {
    let mut g = pod(2);
    let h = ready(&mut g, 0, catalog::garruks_harbinger());
    let jace = ready(&mut g, 1, catalog::jace_the_mind_sculptor());
    g.battlefield_find_mut(jace).unwrap().counters.insert(CounterType::Loyalty, 9);
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: h, target: AttackTarget::Planeswalker(jace) }]))
        .expect("attack");
    drain_stack(&mut g);
    finish_combat(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// CR 111.1 — Namora's power-up and Aquatic Incursion make Merfolk.
#[test]
fn namora_and_aquatic_incursion_make_merfolk() {
    let mut g = pod(2);
    let n = ready(&mut g, 0, catalog::namora_the_sea_queen());
    activate(&mut g, n, 0, None);
    let ai = g.add_card_to_hand(0, catalog::aquatic_incursion());
    cast(&mut g, ai, None);
    assert_eq!(named(&g, 0, "Merfolk"), 4);
    let incursion = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Aquatic Incursion")
        .unwrap()
        .id;
    activate(&mut g, incursion, 0, Some(Target::Permanent(n)));
    assert!(
        g.computed_permanent(n)
            .unwrap()
            .keywords()
            .contains(&Keyword::Unblockable)
    );
}

/// CR 603.10a — Jack of Hearts' death damage reads his last power.
#[test]
fn jack_of_hearts_explodes() {
    let mut g = pod(2);
    let jack = ready(&mut g, 0, catalog::jack_of_hearts_volatile_hero());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    kill(&mut g, jack);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(angel).is_some());
}

/// CR 605.1b — Gauntlet of Might pumps red creatures and doubles up Mountains.
#[test]
fn gauntlet_of_might_pumps_and_ramps() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::gauntlet_of_might());
    let jack = ready(&mut g, 0, catalog::jack_of_hearts_volatile_hero());
    assert_eq!(pt(&g, jack), (3, 3));
    let m = ready(&mut g, 1, catalog::mountain());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: m,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap");
    assert_eq!(
        g.players[1].mana_pool.total(),
        2,
        "its controller gets the extra"
    );
}

/// CR 700.2 — White Widow's ETB counters up to two creatures.
#[test]
fn white_widow_counters_two() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    let ww = ready(&mut g, 0, catalog::white_widow_free_agent());
    let etb = catalog::white_widow_free_agent().triggered_abilities[0]
        .effect
        .clone();
    let mut ctx = EffectContext::for_spell(0, Some(Target::Permanent(a)), 0, 0);
    ctx.targets.push(Target::Permanent(b));
    ctx.source = Some(ww);
    let evs = g.resolve_effect(&etb, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(plus(&g, a) + plus(&g, b), 2);
}

/// CR 607.2a — Promise of Tomorrow exiles your dead and returns them when your
/// board is empty at an end step.
#[test]
fn promise_of_tomorrow_brings_them_back() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::promise_of_tomorrow());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    kill(&mut g, bear);
    assert!(g.exile.iter().any(|c| c.definition.name == "Grizzly Bears"));
    while g.step != TurnStep::End {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(named(&g, 0, "Grizzly Bears"), 1);
    assert_eq!(named(&g, 0, "Promise of Tomorrow"), 0);
}

/// CR 702.49 — Covert Technician drops a cheap artifact on a hit; Replication
/// Specialist copies it.
#[test]
fn covert_technician_and_replication_specialist() {
    let mut g = pod(2);
    let ct = ready(&mut g, 0, catalog::covert_technician());
    ready(&mut g, 0, catalog::replication_specialist());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Cards(vec![ring]),
        DecisionAnswer::Bool(true),
    ]));
    flood(&mut g);
    declare(&mut g, vec![(ct, 1)]);
    finish_combat(&mut g);
    assert_eq!(named(&g, 0, "Sol Ring"), 2, "put in, then copied");
}
