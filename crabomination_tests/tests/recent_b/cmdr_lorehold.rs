//! Commander: the Lorehold Legacies precon (C21, Osgir,
//! `decks::cmdr_lorehold`) and the primitives it needed.

use crabomination::card::{CardDefinition, CardId, CardType, SelectionRequirement as R, StaticAbility};
use crabomination::catalog;
use crabomination::effect::{Effect, StaticEffect};
use crabomination::game::effects::EffectContext;
use crabomination::card::{CounterType, Keyword};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::mana::Color;
use crabomination::game::*;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// Attack with `attacker` at seat 1, have `blocker` block it, and run combat.
fn blocked_combat(g: &mut GameState, attacker: CardId, blocker: CardId) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// CR 615 — "prevent all combat damage that would be dealt to attacking
/// artifact creatures you control": the blocked attacker survives, the
/// blocker still takes its damage, and an opponent's artifact isn't covered.
#[test]
fn cr_615_a_filtered_shield_prevents_combat_damage_to_its_matches() {
    let shield = || CardDefinition {
        name: "Test Shield",
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "prevent combat damage to attacking artifact creatures you control",
            effect: StaticEffect::PreventAllCombatDamageToMatching {
                filter: R::Artifact.and(R::Creature).and(R::IsAttacking).and(R::ControlledByYou),
            },
        }],
        ..Default::default()
    };
    let mut g = pod(2);
    g.add_card_to_battlefield(0, shield());
    let sable = g.add_card_to_battlefield(0, catalog::bronze_sable());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    blocked_combat(&mut g, sable, bear);
    assert!(g.battlefield_find(sable).is_some(), "no damage to the attacking artifact");
    assert!(g.battlefield_find(bear).is_none(), "the blocker still dies");

    let mut g = pod(2);
    g.add_card_to_battlefield(1, shield());
    let sable = g.add_card_to_battlefield(0, catalog::bronze_sable());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    blocked_combat(&mut g, sable, bear);
    assert!(g.battlefield_find(sable).is_none(), "an opponent's shield doesn't cover it");
}

/// Reveal until an artifact: it enters, the misses go to the bottom, and the
/// controller takes damage equal to every card revealed.
#[test]
fn reveal_until_an_artifact_bottoms_the_rest_and_hurts_you() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::bronze_sable());
    g.add_card_to_library(0, catalog::island());
    let top: Vec<&str> = g.players[0].library.iter().map(|c| c.definition.name).collect();
    assert_eq!(top[0], "Grizzly Bears", "add_card_to_library puts cards in order: {top:?}");
    let life = g.players[0].life;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::RevealUntilOneToBattlefieldRestBottom { filter: R::Artifact, damage_controller: true },
        &ctx,
    )
    .expect("resolve");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Bronze Sable"));
    assert_eq!(g.players[0].life, life - 3);
    let order: Vec<&str> = g.players[0].library.iter().map(|c| c.definition.name).collect();
    assert_eq!(order, vec!["Island", "Grizzly Bears", "Grizzly Bears"], "the misses went to the bottom");
}

// ── The cards ──

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

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    let seat = g.battlefield_find(id).expect("source").controller;
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: x,
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

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn yes(g: &mut GameState) {
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 8]));
}

/// {X}, {T}, exile an artifact card with mana value X: two token copies; a
/// card of another mana value can't pay.
#[test]
fn osgir_copies_an_artifact_from_the_graveyard() {
    let mut g = pod(2);
    let osgir = g.add_card_to_battlefield(0, catalog::osgir_the_reconstructor());
    g.clear_sickness(osgir);
    g.add_card_to_graveyard(0, catalog::mind_stone());
    assert!(activate(&mut g, osgir, 1, None, Some(3)).is_err(), "no artifact with mana value 3");
    activate(&mut g, osgir, 1, None, Some(2)).expect("X = 2");
    assert_eq!(named(&g, 0, "Mind Stone").len(), 2);
    assert!(g.players[0].graveyard.is_empty());
}

/// Alibou: other artifact creatures have haste; an artifact attack deals X
/// (the tapped artifacts you control).
#[test]
fn alibou_burns_for_each_tapped_artifact() {
    let mut g = pod(2);
    let alibou = g.add_card_to_battlefield(0, catalog::alibou_ancient_witness());
    let thopter = g.add_card_to_battlefield(0, catalog::ornithopter());
    assert!(g.computed_permanent(thopter).unwrap().keywords().contains(&Keyword::Haste));
    assert!(!g.computed_permanent(alibou).unwrap().keywords().contains(&Keyword::Haste), "other");
    let rock = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.battlefield_find_mut(rock).unwrap().tapped = true;
    let life = g.players[1].life;
    attack(&mut g, &[alibou, thopter]);
    assert_eq!(g.players[1].life, life - 3, "Alibou, the Ornithopter and the tapped Mind Stone");
}

/// Enters: up to two basic Plains to hand.
#[test]
fn archaeomancers_map_fetches_plains() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    let map = g.add_card_to_hand(0, catalog::archaeomancers_map());
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, map, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand + 1, "the Map left, two Plains came");
}

/// {T}, sacrifice an artifact: the next artifact enters; damage per reveal.
#[test]
fn audacious_reshapers_digs_for_an_artifact() {
    let mut g = pod(2);
    let r = g.add_card_to_battlefield(0, catalog::audacious_reshapers());
    g.clear_sickness(r);
    g.add_card_to_battlefield(0, catalog::mind_stone());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::sol_ring());
    let life = g.players[0].life;
    activate(&mut g, r, 0, None, None).expect("activate");
    assert_eq!(named(&g, 0, "Sol Ring").len(), 1);
    assert!(named(&g, 0, "Mind Stone").is_empty(), "sacrificed");
    assert_eq!(g.players[0].life, life - 2);
}

/// Equipped creature has haste, and paying {1} copies its ability.
#[test]
fn battlemages_bracers_copy_the_equipped_ability() {
    let mut g = pod(2);
    let pyro = g.add_card_to_battlefield(0, catalog::prodigal_pyromancer());
    let bracers = g.add_card_to_battlefield(0, catalog::battlemages_bracers());
    g.battlefield_find_mut(bracers).unwrap().attached_to = Some(pyro);
    assert!(g.computed_permanent(pyro).unwrap().keywords().contains(&Keyword::Haste));
    yes(&mut g);
    let life = g.players[1].life;
    activate(&mut g, pyro, 0, Some(Target::Player(1)), None).expect("ping");
    assert_eq!(g.players[1].life, life - 2, "the ping and its copy");
}

/// Power counts your artifacts; other artifacts have ward {2}.
#[test]
fn bronze_guardian_counts_and_wards_artifacts() {
    let mut g = pod(2);
    let guard = g.add_card_to_battlefield(0, catalog::bronze_guardian());
    let rock = g.add_card_to_battlefield(0, catalog::mind_stone());
    assert_eq!(g.computed_permanent(guard).unwrap().power, 2);
    let kws = g.computed_permanent(rock).unwrap().keywords().to_vec();
    assert!(kws.iter().any(|k| matches!(k, Keyword::Ward(_))), "{kws:?}");
}

/// The opponent declines: you mill three and they take the milled mana value.
#[test]
fn combustible_gearhulk_punishes_a_refusal() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::sol_ring());
    g.add_card_to_library(0, catalog::mind_stone());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(false)]));
    let hulk = g.add_card_to_hand(0, catalog::combustible_gearhulk());
    let life = g.players[1].life;
    cast(&mut g, 0, hulk, None).expect("cast");
    assert_eq!(g.players[0].graveyard.len(), 3, "milled three");
    assert_eq!(g.players[1].life, life - 3, "Sol Ring 1 + Mind Stone 2 + Island 0");
}

/// Casting an artifact spell and paying {2}: a Construct that counts
/// artifacts.
#[test]
fn digsite_engineer_builds_constructs() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::digsite_engineer());
    yes(&mut g);
    let rock = g.add_card_to_hand(0, catalog::mind_stone());
    cast(&mut g, 0, rock, None).expect("cast");
    let c = named(&g, 0, "Construct");
    assert_eq!(c.len(), 1);
    assert_eq!(g.computed_permanent(c[0]).unwrap().power, 2, "Mind Stone and itself");
}

/// {2}{W}, {T}, sacrifice: destroy target artifact or enchantment.
#[test]
fn dispellers_capsule_pops_an_enchantment() {
    let mut g = pod(2);
    let cap = g.add_card_to_battlefield(0, catalog::dispellers_capsule());
    let aura = g.add_card_to_battlefield(1, catalog::goblin_oriflamme());
    activate(&mut g, cap, 0, Some(Target::Permanent(aura)), None).expect("activate");
    assert!(g.battlefield_find(aura).is_none());
    assert!(g.battlefield_find(cap).is_none(), "sacrificed");
}

/// {T}, discard: target creature can't be blocked this turn.
#[test]
fn key_to_the_city_opens_the_gates() {
    let mut g = pod(2);
    let key = g.add_card_to_battlefield(0, catalog::key_to_the_city());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, key, 0, Some(Target::Permanent(bear)), None).expect("activate");
    assert!(g.players[0].hand.is_empty(), "discarded");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Unblockable));
}

/// Attacking exiles the top card, and the exile grows Laelia.
#[test]
fn laelia_exiles_and_grows() {
    let mut g = pod(2);
    let laelia = g.add_card_to_battlefield(0, catalog::laelia_the_blade_reforged());
    library(&mut g, 0, 2);
    attack(&mut g, &[laelia]);
    assert_eq!(g.exile.len(), 1);
    assert_eq!(g.battlefield_find(laelia).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// An artifact creature entering draws, once a turn.
#[test]
fn losheel_draws_once_a_turn() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::losheel_clockwork_scholar());
    library(&mut g, 0, 3);
    let hand = g.players[0].hand.len();
    for _ in 0..2 {
        let t = g.add_card_to_hand(0, catalog::ornithopter());
        cast(&mut g, 0, t, None).expect("cast");
    }
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// An opponent's second spell in a turn makes a Treasure; their first and
/// third don't.
#[test]
fn monologue_tax_taxes_the_second_spell() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::monologue_tax());
    g.active_player_idx = 1;
    for _ in 0..3 {
        let t = g.add_card_to_hand(1, catalog::ornithopter());
        cast(&mut g, 1, t, None).expect("cast");
    }
    assert_eq!(named(&g, 0, "Treasure").len(), 1);
}

/// An artifact of yours entering lets you rummage.
#[test]
fn quicksmith_genius_rummages() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::quicksmith_genius());
    library(&mut g, 0, 2);
    g.add_card_to_hand(0, catalog::plains());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let t = g.add_card_to_hand(0, catalog::ornithopter());
    cast(&mut g, 0, t, None).expect("cast");
    assert_eq!(g.players[0].graveyard.len(), 1, "discarded the Plains");
    assert_eq!(g.players[0].hand.len(), 1, "drew one back");
}

/// Dying: each player who accepts discards their hand and draws seven.
#[test]
fn ruin_grinder_offers_a_wheel() {
    let mut g = pod(2);
    let grinder = g.add_card_to_battlefield(0, catalog::ruin_grinder());
    library(&mut g, 0, 10);
    g.add_card_to_hand(0, catalog::plains());
    // Seat 0 accepts, seat 1 declines.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true), DecisionAnswer::Bool(false)]));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.battlefield_find_mut(grinder).unwrap().damage = 3;
    cast(&mut g, 1, bolt, Some(Target::Permanent(grinder))).expect("bolt");
    assert!(g.battlefield_find(grinder).is_none());
    assert_eq!(g.players[0].hand.len(), 7);
    assert!(catalog::ruin_grinder().keywords.iter().any(|k| matches!(k, Keyword::Landcycling(..))));
}

/// Dying leaves three keyworded Golems.
#[test]
fn triplicate_titan_splits_into_golems() {
    let mut g = pod(2);
    let titan = g.add_card_to_battlefield(0, catalog::triplicate_titan());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.battlefield_find_mut(titan).unwrap().damage = 8;
    cast(&mut g, 1, bolt, Some(Target::Permanent(titan))).expect("bolt");
    let golems = named(&g, 0, "Golem");
    assert_eq!(golems.len(), 3);
    for kw in [Keyword::Flying, Keyword::Vigilance, Keyword::Trample] {
        assert!(golems.iter().any(|&id| g.computed_permanent(id).unwrap().keywords().contains(&kw)));
    }
}

/// Every artifact card returns, hasty.
#[test]
fn wake_the_past_returns_artifacts_with_haste() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::ornithopter());
    g.add_card_to_graveyard(0, catalog::mind_stone());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let wake = g.add_card_to_hand(0, catalog::wake_the_past());
    cast(&mut g, 0, wake, None).expect("cast");
    let t = named(&g, 0, "Ornithopter");
    assert_eq!(t.len(), 1);
    assert_eq!(named(&g, 0, "Mind Stone").len(), 1);
    assert!(named(&g, 0, "Grizzly Bears").is_empty());
    assert!(g.computed_permanent(t[0]).unwrap().keywords().contains(&Keyword::Haste));
}
