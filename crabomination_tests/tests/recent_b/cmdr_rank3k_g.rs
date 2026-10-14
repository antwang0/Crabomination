//! Commander: one- and two-card gaps of EDHREC ranks 2501-3500
//! (`decks::cmdr_rank3k_g`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::Effect;
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

fn resolve(g: &mut GameState, effect: &Effect) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
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
/// CR 702.15b — Butcher's Cleaver grants lifelink only to a Human.
#[test]
fn butchers_cleaver_gives_a_human_lifelink() {
    let mut g = pod(2);
    let cleaver = ready(&mut g, 0, catalog::butchers_cleaver());
    let human = ready(&mut g, 0, catalog::grizzly_bears());
    act(
        &mut g,
        GameAction::Equip {
            equipment: cleaver,
            target: human,
        },
    );
    assert_eq!(pt(&g, human), (5, 2));
    assert!(
        !g.computed_permanent(human)
            .unwrap()
            .keywords()
            .contains(&Keyword::Lifelink),
        "a Bear is not a Human"
    );
    let man = ready(&mut g, 0, catalog::steward_of_solidarity());
    act(
        &mut g,
        GameAction::Equip {
            equipment: cleaver,
            target: man,
        },
    );
    assert!(
        g.computed_permanent(man)
            .unwrap()
            .keywords()
            .contains(&Keyword::Lifelink)
    );
}

/// CR 614.1a — Doctor Strange doubles your life gain; MJ grows on each gain.
#[test]
fn doctor_strange_doubles_life_gain_and_mj_grows() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::doctor_strange_surgeon());
    let mj = ready(&mut g, 0, catalog::mj_rising_star());
    resolve(
        &mut g,
        &Effect::GainLife {
            who: crabomination::effect::Selector::You,
            amount: crabomination::effect::Value::Const(3),
        },
    );
    assert_eq!(g.players[0].life, 26);
    assert_eq!(plus(&g, mj), 1);
}

/// CR 122.1 — Sauron's saurian counter makes a green 5/5 Dinosaur while it stays.
#[test]
fn sauron_saurian_counter_makes_a_dinosaur() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let sauron = ready(&mut g, 0, catalog::sauron_dino_devotee());
    let etb = catalog::sauron_dino_devotee().triggered_abilities[0]
        .effect
        .clone();
    resolve_at(&mut g, &etb, sauron, Some(Target::Permanent(bear)), 1);
    assert_eq!(
        g.battlefield_find(bear)
            .unwrap()
            .counter_count(CounterType::Saurian),
        1
    );
    assert_eq!(pt(&g, bear), (5, 5));
    let p = g.computed_permanent(bear).unwrap();
    assert!(p.colors.contains(Color::Green));
    assert!(
        p.subtypes()
            .creature_types
            .contains(&crabomination::card::CreatureType::Dinosaur)
    );
    resolve_at(&mut g, &etb, sauron, None, 0);
    assert_eq!(g.players[0].life, 23, "the gain-3 mode");
}

/// CR 607.2a — Nyla exiles a creature card linked to itself, losing life and
/// making Clues equal to its mana value.
#[test]
fn nyla_exiles_and_makes_clues() {
    let mut g = pod(2);
    let nyla = ready(&mut g, 0, catalog::nyla_shirshu_sleuth());
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    let etb = catalog::nyla_shirshu_sleuth().triggered_abilities[0]
        .effect
        .clone();
    resolve_at(&mut g, &etb, nyla, Some(Target::Permanent(angel)), 0);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != angel));
    assert_eq!(g.players[0].life, 15);
    assert_eq!(named(&g, 0, "Clue"), 5);
}

/// CR 506.4 — Technodrome can't attack while its power is below 6.
#[test]
fn technodrome_is_gated_on_power() {
    let mut g = pod(2);
    let t = ready(&mut g, 0, catalog::technodrome());
    assert!(
        g.computed_permanent(t)
            .unwrap()
            .keywords()
            .contains(&Keyword::CantAttack)
    );
    for _ in 0..3 {
        let rock = ready(&mut g, 0, catalog::sol_ring());
        let _ = rock;
        g.add_card_to_library(0, catalog::island());
        g.battlefield_find_mut(t).unwrap().tapped = false;
        activate(&mut g, t, 0, None);
    }
    assert_eq!(plus(&g, t), 3);
    assert_eq!(pt(&g, t), (6, 6));
    assert!(
        !g.computed_permanent(t)
            .unwrap()
            .keywords()
            .contains(&Keyword::CantAttack)
    );
}

/// CR 701.21a — Culling Dais stores charge counters, then draws for them.
#[test]
fn culling_dais_draws_per_charge_counter() {
    let mut g = pod(2);
    let dais = ready(&mut g, 0, catalog::culling_dais());
    for _ in 0..2 {
        ready(&mut g, 0, catalog::grizzly_bears());
        g.battlefield_find_mut(dais).unwrap().tapped = false;
        activate(&mut g, dais, 0, None);
    }
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    activate(&mut g, dais, 1, None);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// CR 601.2b — Stir Up Trouble pays {4} when there's nothing to sacrifice.
#[test]
fn stir_up_trouble_destroys_a_creature() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let stir = g.add_card_to_hand(0, catalog::stir_up_trouble());
    cast(&mut g, stir, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none());
}

/// CR 119.7 — Gríma stops opponents gaining life; a legendary sacrifice amasses.
#[test]
fn grima_blocks_gain_and_amasses_on_legends() {
    let mut g = pod(2);
    let grima = ready(&mut g, 0, catalog::grima_wormtongue());
    let mut ctx = EffectContext::for_spell(1, None, 0, 0);
    ctx.source = None;
    let gain = Effect::GainLife {
        who: crabomination::effect::Selector::You,
        amount: crabomination::effect::Value::Const(5),
    };
    g.resolve_effect(&gain, &ctx).unwrap();
    assert_eq!(g.players[1].life, 20);
    ready(&mut g, 0, catalog::mj_rising_star());
    activate(&mut g, grima, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 19);
    assert_eq!(
        g.battlefield
            .iter()
            .filter(|c| c.definition.name.contains("Army"))
            .count(),
        1
    );
}

/// CR 701.21a — Angelic Purge sacrifices a permanent and exiles the target.
#[test]
fn angelic_purge_exiles_after_a_sacrifice() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::island());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let purge = g.add_card_to_hand(0, catalog::angelic_purge());
    cast(&mut g, purge, Some(Target::Permanent(bear)));
    assert!(g.exile.iter().any(|c| c.id == bear));
    assert_eq!(named(&g, 0, "Island"), 0);
}

/// CR 702.6a — Conquering Manticore steals a creature with haste and attacks.
#[test]
fn conquering_manticore_steals_and_swings() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    g.battlefield_find_mut(angel).unwrap().tapped = true;
    let m = ready(&mut g, 0, catalog::conquering_manticore());
    let etb = catalog::conquering_manticore().triggered_abilities[0]
        .effect
        .clone();
    resolve_at(&mut g, &etb, m, Some(Target::Permanent(angel)), 0);
    assert_eq!(g.battlefield_find(angel).unwrap().controller, 0);
    declare(&mut g, vec![(angel, 1)]);
    finish_combat(&mut g);
    assert_eq!(g.players[1].life, 16);
}

/// CR 702.29e — Apex Witchstalker basic-landcycles for {2}.
#[test]
fn apex_witchstalker_basic_landcycles() {
    let mut g = pod(2);
    let w = g.add_card_to_hand(0, catalog::apex_witchstalker());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let forest = g.add_card_to_library(0, catalog::forest());
    act(&mut g, GameAction::Landcycle { card_id: w });
    assert!(g.players[0].hand.iter().any(|c| c.id == forest));
    assert!(g.players[0].hand.iter().all(|c| c.id != bear));
}

/// CR 508.1 — Moon-Boy gets +1/+1 attacking while you control a Dinosaur.
#[test]
fn moon_boy_pumps_with_a_dinosaur() {
    let mut g = pod(2);
    let moon = ready(&mut g, 0, catalog::moon_boy_dino_rider());
    let sauron = ready(&mut g, 0, catalog::sauron_dino_devotee());
    declare(&mut g, vec![(moon, 1)]);
    assert_eq!(pt(&g, moon), (3, 3));
    kill(&mut g, sauron);
    assert!(g.battlefield_find(sauron).is_none());
}
