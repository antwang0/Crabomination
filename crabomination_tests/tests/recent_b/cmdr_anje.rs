//! Commander: the Merciless Rage precon (C19, Anje, `decks::cmdr_anje`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn cast_granted(g: &mut GameState, id: CardId) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// An untapped, unsick Anje with a few cards to draw.
fn anje(g: &mut GameState) -> CardId {
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::swamp());
    }
    let a = g.add_card_to_battlefield(0, catalog::anje_falkenrath());
    g.clear_sickness(a);
    a
}

/// Anje discards Dark Withering: it is cast for madness {B} (CR 702.35),
/// kills the nonblack creature, and Anje untaps.
#[test]
fn anje_discards_into_madness_and_untaps() {
    let mut g = main_phase(2);
    let a = anje(&mut g);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::dark_withering());
    g.players[0].mana_pool.add(Color::Black, 1);
    activate(&mut g, a, 0).expect("rummage");
    assert!(g.battlefield_find(bear).is_none(), "Dark Withering cast for its madness cost");
    assert!(!g.battlefield_find(a).unwrap().tapped, "a card with madness untaps Anje");
    assert_eq!(g.players[0].hand.len(), 1, "drew one");
}

/// From Under the Floorboards for madness {X}{B}{B}: X is the mana left over,
/// four Zombies and 4 life instead of three.
#[test]
fn from_under_the_floorboards_madness_x() {
    let mut g = main_phase(2);
    let a = anje(&mut g);
    g.add_card_to_hand(0, catalog::from_under_the_floorboards());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    let life = g.players[0].life;
    activate(&mut g, a, 0).expect("rummage");
    let zombies = named(&g, 0, "Zombie");
    assert_eq!(zombies.len(), 4);
    assert!(zombies.iter().all(|z| g.battlefield_find(*z).unwrap().tapped));
    assert_eq!(g.players[0].life, life + 4);
}

/// Cast from hand, From Under the Floorboards makes the printed three.
#[test]
fn from_under_the_floorboards_hard_cast() {
    let mut g = main_phase(2);
    let spell = g.add_card_to_hand(0, catalog::from_under_the_floorboards());
    flood(&mut g, 0);
    cast(&mut g, spell, &[]).expect("cast");
    assert_eq!(named(&g, 0, "Zombie").len(), 3);
}

/// Grave Scrabbler's return only happens when its madness cost was paid.
#[test]
fn grave_scrabbler_needs_madness() {
    let mut g = main_phase(2);
    let dead = g.add_card_to_graveyard(1, catalog::craw_wurm());
    let hard = g.add_card_to_hand(0, catalog::grave_scrabbler());
    flood(&mut g, 0);
    cast(&mut g, hard, &[]).expect("cast");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == dead), "no madness, no return");
    g.players[0].mana_pool = Default::default();
    let a = anje(&mut g);
    g.add_card_to_hand(0, catalog::grave_scrabbler());
    g.decider = Box::new(ScriptedDecider::new((0..4).map(|_| DecisionAnswer::Bool(true))));
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, a, 0).expect("rummage");
    assert!(g.players[1].hand.iter().any(|c| c.id == dead), "back to its owner's hand");
}

/// Aeon Engine reverses the turn order (CR 101.4 / the Aeon Engine rulings):
/// seat 0's next seat becomes the one on its other side.
#[test]
fn aeon_engine_reverses_turn_order() {
    let mut g = main_phase(4);
    let engine = g.add_card_to_battlefield(0, catalog::aeon_engine());
    assert_eq!(g.next_alive_seat(0), 1);
    activate(&mut g, engine, 0).expect("reverse");
    assert!(g.battlefield_find(engine).is_none(), "exiled as a cost");
    assert_eq!(g.next_alive_seat(0), 3);
    assert_eq!(g.next_alive_seat(3), 2);
}

/// K'rrik: 2 life pays each {B} of a spell's cost the pool can't.
#[test]
fn krrik_pays_black_with_life() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::krrik_son_of_yawgmoth());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::dark_withering());
    g.players[0].mana_pool.add_colorless(4);
    let life = g.players[0].life;
    cast(&mut g, spell, &[Target::Permanent(bear)]).expect("cast with life");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].life, life - 4);
    let k = named(&g, 0, "K'rrik, Son of Yawgmoth")[0];
    assert_eq!(g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "a black spell");

    // "For each {B} in a cost" covers an activation cost too (2024-06-07
    // ruling, CR 107.4f): Nantuko Shade's {B} pump for 2 life, no mana.
    let shade = g.add_card_to_battlefield(0, catalog::nantuko_shade());
    let life = g.players[0].life;
    activate(&mut g, shade, 0).expect("pump with life");
    assert_eq!(g.players[0].life, life - 2);
    assert_eq!(g.computed_permanent(shade).unwrap().power, 3);
}

/// Bone Miser: a discarded creature makes a 2/2 Zombie.
#[test]
fn bone_miser_pays_for_a_discarded_creature() {
    let mut g = main_phase(2);
    let a = anje(&mut g);
    g.add_card_to_battlefield(0, catalog::bone_miser());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    activate(&mut g, a, 0).expect("rummage");
    assert_eq!(named(&g, 0, "Zombie").len(), 1);
}

/// Greven gets +X/+0 for the life you've lost this turn.
#[test]
fn greven_grows_with_life_lost() {
    let mut g = main_phase(2);
    let greven = g.add_card_to_battlefield(0, catalog::greven_predator_captain());
    let spell = g.add_card_to_hand(0, catalog::dark_withering());
    g.add_card_to_battlefield(0, catalog::krrik_son_of_yawgmoth());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, spell, &[Target::Permanent(bear)]).expect("cast with life");
    assert_eq!(g.computed_permanent(greven).unwrap().power, 5 + 4);
}

/// Chainer: a creature cast from the graveyard through its permission gains
/// haste; one cast from hand doesn't.
#[test]
fn chainer_hastes_what_you_did_not_cast_from_hand() {
    let mut g = main_phase(2);
    let chainer = g.add_card_to_battlefield(0, catalog::chainer_nightmare_adept());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::swamp());
    activate(&mut g, chainer, 0).expect("discard");
    flood(&mut g, 0);
    cast_granted(&mut g, dead).expect("cast from the graveyard");
    let cp = g.computed_permanent(dead).expect("on the battlefield");
    assert!(cp.keywords().contains(&Keyword::Haste));
    let fresh = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, fresh, &[]).expect("cast from hand");
    assert!(!g.computed_permanent(fresh).unwrap().keywords().contains(&Keyword::Haste));

    // "A creature spell": any one of them, the player's pick when it's cast
    // (CR 601.2) — the later one here — and then no second.
    let mut g = main_phase(2);
    let chainer = g.add_card_to_battlefield(0, catalog::chainer_nightmare_adept());
    let first = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let second = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.add_card_to_hand(0, catalog::swamp());
    activate(&mut g, chainer, 0).expect("discard");
    flood(&mut g, 0);
    cast_granted(&mut g, second).expect("the angel");
    assert!(g.battlefield_find(second).is_some());
    flood(&mut g, 0);
    assert!(cast_granted(&mut g, first).is_err(), "one creature spell");
}

/// Hedonist's Trove exiles an opponent's graveyard, and you may cast from it.
#[test]
fn hedonists_trove_plays_their_graveyard() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let trove = g.add_card_to_hand(0, catalog::hedonists_trove());
    flood(&mut g, 0);
    cast(&mut g, trove, &[]).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bear));
    flood(&mut g, 0);
    cast_granted(&mut g, bear).expect("cast from exile");
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
}

/// Hedonist's Trove: lands from among its cards freely, but one spell a turn
/// (re-armed as the turn ends), and nothing once it has left the battlefield
/// (CR 611.3a — a static ability's effect ends with its source).
#[test]
fn hedonists_trove_one_spell_a_turn_while_it_stays() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let angel = g.add_card_to_graveyard(1, catalog::serra_angel());
    let land = g.add_card_to_graveyard(1, catalog::swamp());
    let trove = g.add_card_to_hand(0, catalog::hedonists_trove());
    flood(&mut g, 0);
    cast(&mut g, trove, &[]).expect("cast");
    flood(&mut g, 0);
    cast_granted(&mut g, bear).expect("the first spell");
    flood(&mut g, 0);
    assert!(cast_granted(&mut g, angel).is_err(), "one spell this way each turn");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("lands aren't capped");
    assert!(g.battlefield_find(land).is_some());

    let granted_to = |g: &GameState| g.exile.iter().find(|c| c.id == angel).and_then(|c| c.may_play_until).map(|m| m.player);
    let started = g.turn_number;
    while g.turn_number == started {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(granted_to(&g), Some(0), "a new turn, a new spell");

    let naturalize = g.add_card_to_hand(0, catalog::naturalize());
    flood(&mut g, 0);
    cast(&mut g, naturalize, &[Target::Permanent(trove)]).expect("destroy the Trove");
    assert!(g.battlefield_find(trove).is_none());
    assert_eq!(granted_to(&g), None, "the permission left with the Trove");
}

/// Boneyard Parley brings creature cards from graveyards back under your
/// control.
#[test]
fn boneyard_parley_takes_a_pile() {
    let mut g = main_phase(2);
    let a = g.add_card_to_graveyard(1, catalog::craw_wurm());
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    // CR 115.1 — "up to five TARGET creature cards": the untargeted one stays.
    let left = g.add_card_to_graveyard(1, catalog::shivan_dragon());
    let spell = g.add_card_to_hand(0, catalog::boneyard_parley());
    flood(&mut g, 0);
    cast(&mut g, spell, &[Target::Permanent(a), Target::Permanent(b)]).expect("cast");
    let mine = [a, b].iter().filter(|id| g.battlefield_find(**id).is_some_and(|c| c.controller == 0)).count();
    assert!(mine >= 1, "the chosen pile came back under your control");
    assert!([a, b].iter().all(|id| g.battlefield_find(*id).is_some()
        || g.players.iter().any(|p| p.graveyard.iter().any(|c| c.id == *id))));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == left), "not a target, not exiled");
}

/// Asylum Visitor draws on the upkeep of a player with no cards in hand.
#[test]
fn asylum_visitor_punishes_an_empty_hand() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::asylum_visitor());
    g.add_card_to_library(0, catalog::swamp());
    g.players[1].hand.clear();
    g.active_player_idx = 1;
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].life, life - 1);
}


/// Archfiend of Spite: "unless they sacrifice that many permanents" is the
/// damaging player's choice — headless they sacrifice when they can; a seat
/// that declines loses the life instead.
#[test]
fn archfiend_of_spite_damager_chooses_life_or_permanents() {
    for decline in [false, true] {
        let mut g = main_phase(2);
        let fiend = g.add_card_to_battlefield(0, catalog::archfiend_of_spite());
        let pinger = g.add_card_to_battlefield(1, catalog::prodigal_pyromancer());
        g.clear_sickness(pinger);
        for _ in 0..3 {
            g.add_card_to_battlefield(1, catalog::island());
        }
        if decline {
            g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(1)]));
        }
        let life = g.players[1].life;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::ActivateAbility {
            card_id: pinger,
            ability_index: 0,
            target: Some(Target::Permanent(fiend)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("ping");
        drain_stack(&mut g);
        let mine = g.battlefield.iter().filter(|c| c.controller == 1).count();
        if decline {
            assert_eq!((g.players[1].life, mine), (life - 1, 4), "declined: 1 life");
        } else {
            assert_eq!((g.players[1].life, mine), (life, 3), "one permanent sacrificed");
        }
    }
}

/// CR 608.2h — Archfiend of Spite's "a source an opponent controls" is read
/// as the source last existed: a Lightning Bolt already in its owner's
/// graveyard still triggers it (it never did).
#[test]
fn archfiend_of_spite_sees_a_resolved_burn_spell() {
    let mut g = main_phase(2);
    let fiend = g.add_card_to_battlefield(0, catalog::archfiend_of_spite());
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::island());
    }
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(fiend)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    let lands = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(lands, 0, "three permanents sacrificed for 3 damage");
}
