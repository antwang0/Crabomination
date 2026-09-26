//! Commander: the Nature's Vengeance precon (C18, Lord Windgrace,
//! `decks::cmdr_windgrace`) and the primitives it needed.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn loyalty(g: &mut GameState, pw: CardId, index: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: pw, ability_index: index, target, x_value: None })
        .expect("loyalty ability");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn declare(g: &mut GameState, seat: usize, attacks: Vec<Attack>) -> Result<(), String> {
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.would_accept(GameAction::DeclareAttackers(attacks)).then_some(()).ok_or_else(|| "rejected".to_string())
}

/// Charnelhoard Wurm triggers on damage to an opponent, not to its own
/// controller.
#[test]
fn charnelhoard_wurm_triggers_only_on_an_opponent() {
    use crabomination::game::effects::EntityRef;
    let mut g = pod(2);
    let wurm = g.add_card_to_battlefield(0, catalog::charnelhoard_wurm());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let mut ev = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 2, Some(wurm), &mut ev);
    g.dispatch_triggers_for_events(&ev);
    assert!(g.stack.is_empty(), "damage to you isn't damage to an opponent");
    let mut ev = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(1), 2, Some(wurm), &mut ev);
    g.dispatch_triggers_for_events(&ev);
    assert_eq!(g.stack.len(), 1, "the opponent was dealt damage");
}

/// CR 606.3 — Lord Windgrace's +2 discards, then draws; a land discarded
/// this way draws one more.
#[test]
fn lord_windgrace_plus_two_draws_an_extra_card_for_a_land() {
    let mut g = pod(2);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let pw = g.add_card_to_battlefield(0, catalog::lord_windgrace());
    g.add_card_to_hand(0, catalog::forest());
    loyalty(&mut g, pw, 0, None);
    assert_eq!(g.players[0].hand.len(), 2, "a land went: two cards back");
    assert_eq!(g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty), 7);

    let mut g = pod(2);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let pw = g.add_card_to_battlefield(0, catalog::lord_windgrace());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    loyalty(&mut g, pw, 0, None);
    assert_eq!(g.players[0].hand.len(), 1, "a nonland went: one card back");
}

/// Lord Windgrace's −3 returns a land card from the graveyard to the
/// battlefield; CR 903.3a — it can be a commander.
#[test]
fn lord_windgrace_minus_three_returns_lands_and_can_command() {
    assert!(catalog::lord_windgrace().can_be_commander);
    let mut g = pod(2);
    let pw = g.add_card_to_battlefield(0, catalog::lord_windgrace());
    let forest = g.add_card_to_graveyard(0, catalog::forest());
    loyalty(&mut g, pw, 1, Some(Target::Permanent(forest)));
    assert!(g.battlefield_find(forest).is_some_and(|c| c.controller == 0));
}

/// CR 107.3 / 601.2h — Gyrus enters with a counter per mana spent (X plus
/// its three pips). Attacking, it exiles a lesser-power creature card for a
/// token copy put onto the battlefield tapped and attacking (CR 508.4 — never
/// declared) that is exiled at end of combat.
#[test]
fn gyrus_counts_mana_spent_and_raises_an_attacking_copy() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    let gyrus = g.add_card_to_hand(0, catalog::gyrus_waker_of_corpses());
    cast(&mut g, gyrus, None, Some(2)).expect("cast Gyrus for X=2");
    assert_eq!(g.battlefield_find(gyrus).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::serra_angel());
    g.clear_sickness(gyrus);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: gyrus, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bears), "the Bears card was exiled");
    let copy = named(&g, 0, "Grizzly Bears");
    assert_eq!(copy.len(), 1, "one token copy");
    assert!(g.battlefield_find(copy[0]).is_some_and(|c| c.tapped && c.is_token));
    assert!(g.attacking.iter().any(|a| a.attacker == copy[0] && a.target == AttackTarget::Player(1)));
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    let life = g.players[1].life;
    for _ in 0..6 {
        if g.step == TurnStep::PostCombatMain {
            break;
        }
        if let Ok(ev) = g.advance_step(Vec::new()) {
            g.dispatch_triggers_for_events(&ev);
        }
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, life - 5 - 2, "Gyrus and the copy both hit");
    assert!(named(&g, 0, "Grizzly Bears").is_empty(), "the copy left at end of combat");
}

/// CR 508.1a — Xantcha enters under an opponent's control, must attack, and
/// can't attack its owner; CR 113.3d — any player may activate its ability.
#[test]
fn xantcha_changes_sides_and_never_attacks_its_owner() {
    let mut g = pod(3);
    flood(&mut g, 0);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let x = g.add_card_to_hand(0, catalog::xantcha_sleeper_agent());
    cast(&mut g, x, None, None).expect("cast Xantcha");
    let holder = g.battlefield_find(x).unwrap().controller;
    assert_ne!(holder, 0, "an opponent controls it");
    let other = if holder == 1 { 2 } else { 1 };
    g.clear_sickness(x);
    g.turn_number += 1;
    let at = |t| vec![Attack { attacker: x, target: AttackTarget::Player(t) }];
    assert!(declare(&mut g, holder, at(0)).is_err(), "not its owner");
    assert!(declare(&mut g, holder, vec![]).is_err(), "it attacks each combat");
    declare(&mut g, holder, at(other)).expect("the third seat");
    // Seat 0 (not the controller) pays {3}: the controller loses 2, seat 0 draws.
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let (life, hand) = (g.players[holder].life, g.players[0].hand.len());
    activate(&mut g, 0, x, 0, None).expect("any player may activate");
    assert_eq!(g.players[holder].life, life - 2);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// CR 508.1d — Thantis makes every creature attack each combat if able, and
/// each creature attacking its controller grows it.
#[test]
fn thantis_forces_attacks_and_grows_when_attacked() {
    let mut g = pod(2);
    let thantis = g.add_card_to_battlefield(0, catalog::thantis_the_warweaver());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let elves = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    for id in [bear, elves] {
        g.clear_sickness(id);
    }
    assert!(declare(&mut g, 1, vec![]).is_err(), "they must attack");
    let at = |id| Attack { attacker: id, target: AttackTarget::Player(0) };
    g.perform_action(GameAction::DeclareAttackers(vec![at(bear), at(elves)])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(thantis).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Reality Scramble puts a permanent on the bottom and reveals until a card
/// that shares a card type with it, which enters (the rest go to the bottom).
#[test]
fn reality_scramble_trades_a_permanent_for_the_next_of_its_type() {
    let mut g = pod(2);
    flood(&mut g, 0);
    g.add_card_to_library(0, catalog::island());
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rs = g.add_card_to_hand(0, catalog::reality_scramble());
    cast(&mut g, rs, Some(Target::Permanent(bear)), None).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(angel).is_some(), "the next creature entered");
    assert_eq!(g.players[0].library.len(), 3, "Bear and the two Islands below");
    assert!(catalog::reality_scramble().keywords.contains(&Keyword::Retrace));
}

/// CR 903.3 — Forge of Heroes counters a commander that entered this turn:
/// +1/+1 on a creature, loyalty on a planeswalker.
#[test]
fn forge_of_heroes_counters_a_fresh_commander() {
    let mut g = pod(2);
    let forge = g.add_card_to_battlefield(0, catalog::forge_of_heroes());
    let pw = g.add_card_to_battlefield(0, catalog::lord_windgrace());
    g.players[0].commanders.push(pw);
    g.battlefield_find_mut(pw).unwrap().entered_turn = Some(g.turn_number);
    activate(&mut g, 0, forge, 1, Some(Target::Permanent(pw))).expect("forge");
    assert_eq!(g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty), 6);
    // A commander from an earlier turn isn't a legal target.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].commanders.push(bear);
    g.battlefield_find_mut(bear).unwrap().entered_turn = Some(g.turn_number - 1);
    g.battlefield_find_mut(forge).unwrap().tapped = false;
    assert!(activate(&mut g, 0, forge, 1, Some(Target::Permanent(bear))).is_err());
}

/// CR 903.8 — Fury Storm copies itself once per command-zone cast of your
/// commander; each copy copies the target spell again.
#[test]
fn fury_storm_copies_per_commander_cast() {
    let mut g = pod(2);
    flood(&mut g, 0);
    let cmdr = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmdr);
    g.commander_cast_count.insert(cmdr, 2);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let storm = g.add_card_to_hand(0, catalog::fury_storm());
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    g.perform_action(GameAction::CastSpell {
        card_id: storm,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Fury Storm");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 4 * 3, "the Bolt and three copies");
}

/// Hunting Wilds, kicked: the two Forests enter, untap, and stay 3/3 haste
/// creatures (CR 702.33 kicker; CR 611.2c — they stay creatures).
#[test]
fn hunting_wilds_kicked_animates_its_forests() {
    let mut g = pod(2);
    flood(&mut g, 0);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hw = g.add_card_to_hand(0, catalog::hunting_wilds());
    g.perform_action(GameAction::CastSpellKicked {
        card_id: hw,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked");
    drain_stack(&mut g);
    let forests = named(&g, 0, "Forest");
    assert_eq!(forests.len(), 2);
    for f in forests {
        let c = g.computed_permanent(f).unwrap();
        assert!(c.card_types().contains(&crabomination::card::CardType::Creature));
        assert_eq!((c.power, c.toughness), (3, 3));
        assert!(!g.battlefield_find(f).unwrap().tapped);
        assert_eq!(c.colors.to_vec(), vec![Color::Green], "they become green");
    }
}

/// Nesting Dragon's landfall lays a Dragon Egg; the Egg, dying, hatches a
/// 2/2 flying Dragon.
#[test]
fn nesting_dragon_eggs_hatch() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::nesting_dragon());
    let land = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    let egg = named(&g, 0, "Dragon Egg");
    assert_eq!(egg.len(), 1);
    flood(&mut g, 0);
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(egg[0])), None).expect("kill the egg");
    let dragon = named(&g, 0, "Dragon");
    assert_eq!(dragon.len(), 1);
    assert!(g.computed_permanent(dragon[0]).unwrap().keywords().contains(&Keyword::Flying));
}

/// Crash of Rhino Beetles gets +10/+10 at ten lands; Zendikar Incarnate's
/// power is its controller's land count (CR 604.3).
#[test]
fn land_count_creatures() {
    let mut g = pod(2);
    let crash = g.add_card_to_battlefield(0, catalog::crash_of_rhino_beetles());
    let zi = g.add_card_to_battlefield(0, catalog::zendikar_incarnate());
    for _ in 0..9 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    assert_eq!(g.computed_permanent(crash).unwrap().power, 5);
    assert_eq!(g.computed_permanent(zi).unwrap().power, 9);
    g.add_card_to_battlefield(0, catalog::forest());
    assert_eq!(g.computed_permanent(crash).unwrap().power, 15);
    assert_eq!(g.computed_permanent(zi).unwrap().toughness, 4);
}

/// Turntimber Sower makes a Plant when a land card hits your graveyard, and
/// sacrifices three creatures to fetch a land card back.
#[test]
fn turntimber_sower_plants_and_recurs() {
    let mut g = pod(2);
    flood(&mut g, 0);
    let sower = g.add_card_to_battlefield(0, catalog::turntimber_sower());
    let land = g.add_card_to_battlefield(0, catalog::warped_landscape());
    activate(&mut g, 0, land, 1, None).expect("crack the Landscape");
    assert_eq!(named(&g, 0, "Plant").len(), 1, "the Landscape hit the graveyard");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, sower, 0, Some(Target::Permanent(land))).expect("sacrifice three");
    assert!(g.players[0].hand.iter().any(|c| c.id == land));
}

/// Flameblast Dragon pays {X}{R} as one payment: with four Mountains, X = 3
/// and the {R} both come out of them (X used to be paid first, leaving no
/// {R}, so the damage never happened).
#[test]
fn flameblast_dragon_pays_x_and_red_together() {
    let mut g = pod(2);
    let dragon = g.add_card_to_battlefield(0, catalog::flameblast_dragon());
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    g.clear_sickness(dragon);
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(3)]));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: dragon, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "X = 3 to the defending player");
    assert!(g.battlefield.iter().filter(|c| c.definition.name == "Mountain").all(|c| c.tapped), "all four paid");
}
