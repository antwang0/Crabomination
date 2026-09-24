//! Commander: the Exquisite Invention precon (C18, Saheeli, the Gifted,
//! `decks::cmdr_saheeli`). The primitives it needed are tested in
//! `core_rules/commander_cards.rs`.

use crabomination::card::{CardDefinition, CardId, CardType, CounterType, CreatureType, Keyword, Subtypes, Supertype};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::{Color, cost, generic};

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

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
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
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn attack(g: &mut GameState, attacker: CardId, at: usize) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(at) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
}

fn bear_commander() -> CardDefinition {
    CardDefinition {
        name: "Test Bear Commander",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// Saheeli's +1s: a Servo, and affinity for artifacts on the next spell.
#[test]
fn saheeli_makes_servos_and_discounts_the_next_spell() {
    let mut g = pod(2);
    let s = g.add_card_to_battlefield(0, catalog::saheeli_the_gifted());
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: s, ability_index: 0, target: None, x_value: None })
        .expect("+1 Servo");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Servo").len(), 1);
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::ornithopter());
    }
    g.battlefield_find_mut(s).unwrap().loyalty_uses_this_turn = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: s, ability_index: 1, target: None, x_value: None })
        .expect("+1 affinity");
    drain_stack(&mut g);
    // Five artifacts: Hexavus's {6} costs {1}.
    let hex = g.add_card_to_hand(0, catalog::hexavus());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: hex, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("one colorless is enough");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hex).is_some());
}

/// Brudiclad makes a Myr each combat and may turn every other token into a
/// copy of the biggest one.
#[test]
fn brudiclad_makes_every_token_the_biggest() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::brudiclad_telchor_engineer());
    let foundry = g.add_card_to_battlefield(0, catalog::retrofitter_foundry());
    activate(&mut g, foundry, 1, None).expect("a Servo");
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(named(&g, 0, "Phyrexian Myr").len(), 2, "the Servo became a 2/1 Myr");
    assert!(named(&g, 0, "Servo").is_empty());
}

/// Echo Storm copies itself once per command-zone cast of your commander.
#[test]
fn echo_storm_scales_with_commander_casts() {
    let mut g = pod(2);
    g.seat_commanders(0, vec![bear_commander()]);
    let commander = g.players[0].commanders[0];
    g.commander_cast_count.insert(commander, 2);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let es = g.add_card_to_hand(0, catalog::echo_storm());
    cast(&mut g, 0, es, Some(Target::Permanent(ring))).expect("cast");
    assert_eq!(named(&g, 0, "Sol Ring").len(), 4, "the Ring, the spell's token, two copies' tokens");
}

/// Forge of Heroes puts a +1/+1 counter on a commander that entered this turn.
#[test]
fn forge_of_heroes_boosts_a_fresh_commander() {
    let mut g = pod(2);
    let forge = g.add_card_to_battlefield(0, catalog::forge_of_heroes());
    g.seat_commanders(0, vec![bear_commander()]);
    let id = g.players[0].commanders[0];
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None, alternative: false, pitch_card: None })
        .expect("cast the commander");
    drain_stack(&mut g);
    activate(&mut g, forge, 1, Some(Target::Permanent(id))).expect("forge");
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Geode Golem — connecting, the commander is cast from the command zone
/// without its mana cost; only the tax is owed.
#[test]
fn geode_golem_casts_the_commander_free() {
    let mut g = pod(2);
    g.seat_commanders(0, vec![bear_commander()]);
    let id = g.players[0].commanders[0];
    let golem = g.add_card_to_battlefield(0, catalog::geode_golem());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    attack(&mut g, golem, 1);
    assert!(g.battlefield_find(id).is_some(), "cast with no mana, no tax yet");
    assert_eq!(g.commander_cast_count.get(&id), Some(&1));
}

/// Highland Lake enters tapped; Inkwell Leviathan has its three keywords.
#[test]
fn highland_lake_and_inkwell_leviathan() {
    let mut g = pod(2);
    let lake = g.add_card_to_hand(0, catalog::highland_lake());
    g.perform_action(GameAction::PlayLand(lake)).expect("land");
    assert!(g.battlefield_find(lake).unwrap().tapped);
    let lev = g.add_card_to_battlefield(0, catalog::inkwell_leviathan());
    let kws = g.computed_permanent(lev).unwrap().keywords().to_vec();
    assert!(kws.contains(&Keyword::Trample) && kws.contains(&Keyword::Shroud));
    assert!(kws.contains(&Keyword::Landwalk(crabomination::card::LandType::Island)));
}

/// Lieutenant — Loyal Drake draws at your combat only with your commander out.
#[test]
fn loyal_drake_needs_its_commander() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_battlefield(0, catalog::loyal_drake());
    let hand = g.players[0].hand.len();
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(g.players[0].hand.len(), hand);
    let c = g.add_card_to_battlefield(0, bear_commander());
    g.players[0].commanders.push(c);
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Prototype Portal imprints an artifact card and copies it for its mana value.
#[test]
fn prototype_portal_copies_its_imprint() {
    let mut g = pod(2);
    let master = g.add_card_to_hand(0, catalog::master_of_etherium());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let portal = g.add_card_to_hand(0, catalog::prototype_portal());
    cast(&mut g, 0, portal, None).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == master && c.exiled_with == Some(portal)));
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::ActivateAbility { card_id: portal, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None }).is_err(), "X is three");
    activate(&mut g, portal, 0, None).expect("copy");
    assert_eq!(named(&g, 0, "Master of Etherium").len(), 1);
}

/// Retrofitter Foundry climbs Servo → Thopter → Construct.
#[test]
fn retrofitter_foundry_upgrades_tokens() {
    let mut g = pod(2);
    let f = g.add_card_to_battlefield(0, catalog::retrofitter_foundry());
    activate(&mut g, f, 1, None).expect("servo");
    activate(&mut g, f, 0, None).expect("untap");
    activate(&mut g, f, 2, None).expect("thopter");
    assert!(named(&g, 0, "Servo").is_empty());
    activate(&mut g, f, 0, None).expect("untap");
    activate(&mut g, f, 3, None).expect("construct");
    assert!(named(&g, 0, "Thopter").is_empty());
    assert_eq!(named(&g, 0, "Construct").len(), 1);
}

/// Reverse Engineer draws three; Saheeli's Artistry copies an artifact and a
/// creature (the latter an artifact too).
#[test]
fn reverse_engineer_and_saheelis_artistry() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    let re = g.add_card_to_hand(0, catalog::reverse_engineer());
    cast(&mut g, 0, re, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand + 3);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let art = g.add_card_to_hand(0, catalog::saheelis_artistry());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: art,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("both modes");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Sol Ring").len(), 2);
    let copy = named(&g, 0, "Grizzly Bears")[0];
    assert!(g.computed_permanent(copy).unwrap().card_types().contains(&CardType::Artifact));
}

/// Saheeli's Directive deploys the cheap artifacts among the top X and bins
/// the rest.
#[test]
fn saheelis_directive_deploys_artifacts() {
    let mut g = pod(2);
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let ring = g.add_card_to_library(0, catalog::sol_ring());
    let hex = g.add_card_to_library(0, catalog::hexavus());
    let d = g.add_card_to_hand(0, catalog::saheelis_directive());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: d, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("X = 3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_some());
    let gy: Vec<CardId> = g.players[0].graveyard.iter().map(|c| c.id).collect();
    assert!(gy.contains(&bear) && gy.contains(&hex), "Hexavus is over X; the Bears aren't an artifact");
}

/// Tawnos copies an artifact's activated ability on the stack.
#[test]
fn tawnos_copies_an_artifact_ability() {
    let mut g = pod(2);
    let t = g.add_card_to_battlefield(0, catalog::tawnos_urzas_apprentice());
    let f = g.add_card_to_battlefield(0, catalog::retrofitter_foundry());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility { card_id: f, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("servo");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: t, ability_index: 0, target: Some(Target::Permanent(f)), additional_targets: vec![], x_value: None, mode: None })
        .expect("copy it");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Servo").len(), 2);
}

/// CR 611.2b — Treasure Nabber keeps an opponent's mana rock through the end
/// of its controller's next turn, then gives it back.
#[test]
fn treasure_nabber_borrows_until_the_end_of_your_next_turn() {
    let mut g = pod(2);
    for s in 0..2 {
        for _ in 0..6 {
            g.add_card_to_library(s, catalog::island());
        }
    }
    g.add_card_to_battlefield(0, catalog::treasure_nabber());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility { card_id: ring, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ring).unwrap().controller, 0);
    let pass_until = |g: &mut GameState, seat: usize| {
        for _ in 0..400 {
            if g.active_player_idx == seat && g.step == TurnStep::PreCombatMain {
                return;
            }
            let _ = g.perform_action(GameAction::PassPriority);
        }
        panic!("never reached seat {seat}'s main phase");
    };
    pass_until(&mut g, 0);
    assert_eq!(g.battlefield_find(ring).unwrap().controller, 0, "still ours on our turn");
    pass_until(&mut g, 1);
    assert_eq!(g.battlefield_find(ring).unwrap().controller, 1, "back after our turn ended");
}

/// Varchild's victim makes Survivors that can't block; when Varchild leaves,
/// its controller takes them.
#[test]
fn varchild_hands_out_and_takes_survivors() {
    let mut g = pod(2);
    let v = g.add_card_to_battlefield(0, catalog::varchild_betrayer_of_kjeldor());
    attack(&mut g, v, 1);
    let survivors = named(&g, 1, "Survivor");
    assert_eq!(survivors.len(), 3);
    assert!(g.computed_permanent(survivors[0]).unwrap().keywords().contains(&Keyword::CantBlock));
    g.remove_to_graveyard_with_triggers(v);
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Survivor").len(), 3);
}

/// Vessel of Endless Rest bottoms a graveyard card and taps for any color.
#[test]
fn vessel_of_endless_rest_bottoms_a_card() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let v = g.add_card_to_hand(0, catalog::vessel_of_endless_rest());
    cast(&mut g, 0, v, None).expect("cast");
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(bear));
}
