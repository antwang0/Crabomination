//! Commander: the 2026-10-13 rescan's two-card gaps, third pass
//! (`decks::cmdr_edhrec26`).

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
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

fn cast(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
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

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let cp = g.computed_permanent(id).expect("on the battlefield");
    (cp.power, cp.toughness)
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn damage(g: &mut GameState) {
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

/// Necklace of Girion: green spells and Forests feed a creature.
#[test]
fn necklace_of_girion_feeds_a_creature() {
    let mut g = pod(2);
    let blor = ready(&mut g, 0, catalog::blor_the_impervious());
    ready(&mut g, 0, catalog::necklace_of_girion());
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    cast(&mut g, 0, elf, &[Target::Permanent(blor)]).expect("green spell");
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(forest)).expect("forest");
    drain_stack(&mut g);
    let total: u32 = g.battlefield.iter().map(|c| c.counter_count(CounterType::PlusOnePlusOne)).sum();
    assert_eq!(total, 2);
}

/// Brought Back returns this turn's dead, tapped; Bishop of Rebirth
/// reanimates a cheap creature on attack.
#[test]
fn brought_back_and_bishop_of_rebirth() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, &[Target::Permanent(bear)]).expect("bolt");
    g.check_state_based_actions();
    let bb = g.add_card_to_hand(0, catalog::brought_back());
    cast(&mut g, 0, bb, &[Target::Permanent(bear)]).expect("brought back");
    assert!(g.battlefield_find(bear).is_some_and(|c| c.tapped));
    let mut g = pod(2);
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bishop = ready(&mut g, 0, catalog::bishop_of_rebirth());
    attack(&mut g, &[bishop]);
    assert!(g.battlefield_find(dead).is_some());
}

/// Minsc makes Boo, then makes a creature an X/X Giant.
#[test]
fn minsc_beloved_ranger_boo_and_giant() {
    let mut g = pod(2);
    let minsc = g.add_card_to_hand(0, catalog::minsc_beloved_ranger());
    cast(&mut g, 0, minsc, &[]).expect("cast");
    let boo = g.battlefield.iter().find(|c| c.definition.name == "Boo").map(|c| c.id).expect("Boo");
    activate(&mut g, minsc, 0, Some(Target::Permanent(boo)), Some(6)).expect("x");
    assert_eq!(pt(&g, boo), (6, 6));
    assert!(g.computed_permanent(boo).unwrap().subtypes().creature_types.contains(&CreatureType::Giant));
}

/// Arena Rector trades itself for a planeswalker from the library.
#[test]
fn arena_rector_fetches_a_planeswalker() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::jace_reality_sculptor());
    let rector = ready(&mut g, 0, catalog::arena_rector());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, &[Target::Permanent(rector)]).expect("bolt");
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(named(&g, "Jace, Reality Sculptor"), 1);
    assert!(g.exile.iter().any(|c| c.id == rector));
}

/// Eternal of Harsh Truths: afflict 2 when blocked (CR 702.130).
#[test]
fn eternal_of_harsh_truths_afflicts() {
    let mut g = pod(2);
    let eternal = ready(&mut g, 0, catalog::eternal_of_harsh_truths());
    let wall = ready(&mut g, 1, catalog::grizzly_bears());
    attack(&mut g, &[eternal]);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, eternal)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

/// Surrakar Spellblade banks charge counters and draws that many.
#[test]
fn surrakar_spellblade_draws_per_counter() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let blade = ready(&mut g, 0, catalog::surrakar_spellblade());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, 0, bolt, &[Target::Player(1)]).expect("bolt");
    }
    assert_eq!(g.battlefield_find(blade).unwrap().counter_count(CounterType::Charge), 2);
    attack(&mut g, &[blade]);
    damage(&mut g);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Commander Mustard's Soldiers trample and hit for 1 on attack once
/// activated; Baird makes a Soldier when a creature is pumped.
#[test]
fn commander_mustard_and_baird() {
    let mut g = pod(2);
    let mustard = ready(&mut g, 0, catalog::commander_mustard());
    let baird = ready(&mut g, 0, catalog::baird_argivian_recruiter());
    assert!(g.computed_permanent(baird).unwrap().keywords().contains(&Keyword::Trample));
    activate(&mut g, mustard, 0, None, None).expect("activate");
    attack(&mut g, &[baird]);
    assert_eq!(g.players[1].life, 19, "Baird's granted ping");
    let rush = g.add_card_to_hand(0, catalog::rush_of_blood());
    cast(&mut g, 0, rush, &[Target::Permanent(baird)]).expect("pump");
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(named(&g, "Soldier"), 1);
}

/// Kassandra fetches The Spear of Leonidas; Eivor burns per Equipment.
#[test]
fn kassandra_and_eivor() {
    let mut g = pod(3);
    g.add_card_to_library(0, catalog::the_spear_of_leonidas());
    let k = g.add_card_to_hand(0, catalog::kassandra_eagle_bearer());
    cast(&mut g, 0, k, &[]).expect("cast");
    assert_eq!(named(&g, "The Spear of Leonidas"), 1);
    let eivor = ready(&mut g, 0, catalog::eivor_battle_ready());
    attack(&mut g, &[eivor]);
    assert_eq!((g.players[1].life, g.players[2].life), (19, 19));
}

/// Kassandra draws only off a creature with a LEGENDARY Equipment attached:
/// the Spear counts, Bonesplitter doesn't.
#[test]
fn kassandra_wants_a_legendary_equipment() {
    for legendary in [true, false] {
        let mut g = pod(2);
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        ready(&mut g, 0, catalog::kassandra_eagle_bearer());
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        let eq = ready(&mut g, 0, if legendary { catalog::the_spear_of_leonidas() } else { catalog::bonesplitter() });
        g.battlefield_find_mut(eq).unwrap().attached_to = Some(bear);
        let hand = g.players[0].hand.len();
        attack(&mut g, &[bear]);
        g.step = TurnStep::CombatDamage;
        let ev = g.resolve_combat().expect("damage");
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + usize::from(legendary), "legendary: {legendary}");
    }
}

/// Shadow of Mortality costs X less for life lost below the start.
#[test]
fn shadow_of_mortality_discount() {
    let mut g = pod(2);
    g.players[0].life = 7;
    g.players[0].mana_pool.add(Color::Black, 2);
    let id = g.add_card_to_hand(0, catalog::shadow_of_mortality());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{B}{B} at 13 life below a 20 start");
}

/// Cabal Paladin burns each opponent for a historic spell.
#[test]
fn cabal_paladin_pings_on_historic() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::cabal_paladin());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, 0, ring, &[]).expect("cast");
    assert_eq!((g.players[1].life, g.players[2].life), (18, 18));
}

/// Freelance Muscle grows by the biggest stat among your other creatures.
#[test]
fn freelance_muscle_grows_by_others() {
    let mut g = pod(2);
    let muscle = ready(&mut g, 0, catalog::freelance_muscle());
    ready(&mut g, 0, catalog::craw_wurm());
    attack(&mut g, &[muscle]);
    assert_eq!(pt(&g, muscle), (10, 10));
}

/// Magebane Armor: +2/+4, no flying, and noncombat damage is prevented.
#[test]
fn magebane_armor_grounds_and_shields() {
    let mut g = pod(2);
    let bird = ready(&mut g, 0, catalog::silver_raven());
    let armor = ready(&mut g, 0, catalog::magebane_armor());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: armor, target: bird }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bird), (3, 5));
    assert!(!g.computed_permanent(bird).unwrap().keywords().contains(&Keyword::Flying));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, &[Target::Permanent(bird)]).expect("bolt");
    assert_eq!(g.battlefield_find(bird).unwrap().damage, 0);
}

/// Dual Casting lets the enchanted creature copy your instant.
#[test]
fn dual_casting_copies_your_spell() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::dual_casting());
    cast(&mut g, 0, aura, &[Target::Permanent(bear)]).expect("aura");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("copy");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14);
}

/// Oblivion's Hunger: indestructible, and a card for a countered creature.
#[test]
fn oblivions_hunger_draws_with_a_counter() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let id = g.add_card_to_hand(0, catalog::oblivions_hunger());
    cast(&mut g, 0, id, &[Target::Permanent(bear)]).expect("cast");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Indestructible));
    assert_eq!(g.players[0].hand.len(), 1);
}
