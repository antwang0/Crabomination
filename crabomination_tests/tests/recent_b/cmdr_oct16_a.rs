//! Commander: the 2026-10-16 rescan's shared gaps (`decks::cmdr_oct16_a`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef};
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
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

/// Seat 0 scries `n` as its own effect, triggers and all.
fn scry(g: &mut GameState, n: i32) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Scry { who: PlayerRef::You, amount: crabomination::card::Value::Const(n) }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn cast_from_exile(g: &mut GameState, id: CardId) -> Result<(), GameError> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

/// CR 701.22a — The Temporal Anchor: the two cards bottomed by a scry are
/// exiled from the library's bottom, linked to the Anchor, and playable on
/// your turn at sorcery speed (no flash, unlike Azula's grant).
#[test]
fn temporal_anchor_exiles_what_you_bottom_and_lets_you_play_it() {
    let mut g = pod(2);
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    let elves = g.add_card_to_library(0, catalog::llanowar_elves());
    stock_libraries(&mut g, 5);
    let anchor = ready(&mut g, 0, catalog::the_temporal_anchor());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder { kept_top: vec![], bottom: vec![bears, elves] }]));
    scry(&mut g, 2);
    for id in [bears, elves] {
        assert!(g.exile.iter().any(|c| c.id == id && c.exiled_with == Some(anchor)), "bottomed then exiled");
    }
    assert_eq!(g.players[0].library.len(), 5);
    g.step = TurnStep::BeginCombat;
    assert!(cast_from_exile(&mut g, bears).is_err(), "no flash");
    g.step = TurnStep::PreCombatMain;
    cast_from_exile(&mut g, bears).expect("play it on your turn");
    assert!(g.battlefield_find(bears).is_some());
}

/// A scry that keeps everything on top bottoms nothing, so the Anchor's
/// second ability doesn't trigger.
#[test]
fn temporal_anchor_ignores_a_scry_that_bottoms_nothing() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    ready(&mut g, 0, catalog::the_temporal_anchor());
    let top: Vec<CardId> = g.players[0].library.iter().take(2).map(|c| c.id).collect();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder { kept_top: top, bottom: vec![] }]));
    scry(&mut g, 2);
    assert!(g.exile.is_empty());
}

/// Glorfindel: a scry pumps it and grants the chosen evasion rider.
#[test]
fn glorfindel_scry_pumps_and_grants_the_chosen_mode() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let glorfindel = ready(&mut g, 0, catalog::glorfindel_dauntless_rescuer());
    let top: Vec<CardId> = g.players[0].library.iter().take(1).map(|c| c.id).collect();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::ScryOrder { kept_top: top, bottom: vec![] },
        DecisionAnswer::Mode(1),
    ]));
    scry(&mut g, 1);
    let c = g.computed_permanent(glorfindel).unwrap();
    assert_eq!((c.power, c.toughness), (4, 3));
    assert!(c.keywords().contains(&Keyword::CantBeBlockedByMoreThanOne));
}

/// Track Down draws the revealed top card only when it's a creature or land.
#[test]
fn track_down_draws_a_creature_or_land() {
    for (def, drawn) in [(catalog::grizzly_bears(), true), (catalog::lightning_bolt(), false)] {
        let mut g = pod(2);
        let top = g.add_card_to_library(0, def);
        stock_libraries(&mut g, 5);
        let spell = g.add_card_to_hand(0, catalog::track_down());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder { kept_top: vec![top], bottom: vec![] }]));
        flood(&mut g);
        g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.iter().any(|c| c.id == top), drawn);
    }
}

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, additional: Vec<Target>) -> Result<(), GameError> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: additional, mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn step_to(g: &mut GameState, seat: usize, step: TurnStep) {
    for _ in 0..200 {
        if g.active_player_idx == seat && g.step == step {
            return;
        }
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
    panic!("never reached seat {seat} {step:?}");
}

fn attack_unblocked(g: &mut GameState, attacker: CardId, defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(defender) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

/// Michelangelo's Technique: up to two creatures, but only while their mana
/// values sum to 6 or less — a 5 and a 2 can't both come.
#[test]
fn michelangelos_technique_caps_total_mana_value_at_six() {
    let mut g = pod(2);
    let big = g.add_card_to_library(0, catalog::craw_wurm());
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    let elves = g.add_card_to_library(0, catalog::llanowar_elves());
    stock_libraries(&mut g, 5);
    let spell = g.add_card_to_hand(0, catalog::michelangelos_technique());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![big, bears, elves])]));
    cast_with(&mut g, spell, None, vec![]).expect("cast");
    assert!(g.battlefield_find(big).is_some(), "Craw Wurm (6) first");
    assert!(g.battlefield_find(bears).is_none() && g.battlefield_find(elves).is_none(), "nothing fits beside it");
}

/// Koya: the exiled creature comes back at the next end step unless {3}{B}
/// is paid.
#[test]
fn koya_returns_the_exiled_creature_unless_paid() {
    for pay in [false, true] {
        let mut g = pod(2);
        stock_libraries(&mut g, 5);
        let theirs = ready(&mut g, 1, catalog::grizzly_bears());
        let koya = g.add_card_to_hand(0, catalog::koya_death_from_above());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(pay)]));
        cast_with(&mut g, koya, None, vec![]).expect("cast");
        assert!(g.exile.iter().any(|c| c.id == theirs), "the only other creature is exiled");
        flood(&mut g);
        step_to(&mut g, 0, TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(theirs).is_some(), !pay, "pay {pay}");
    }
}

/// Ooze Spill counters the spell and leaves a Mutagen behind.
#[test]
fn ooze_spill_counters_and_makes_a_mutagen() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    let life = g.players[0].life;
    let spill = g.add_card_to_hand(0, catalog::ooze_spill());
    cast_with(&mut g, spill, Some(Target::Permanent(bolt)), vec![]).expect("spill");
    assert_eq!(g.players[0].life, life);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Mutagen"));
}

/// Hog-Monkey Rampage adds the counter only to a creature with power 4+.
#[test]
fn hog_monkey_rampage_counter_needs_power_four() {
    for (def, counter) in [(catalog::craw_wurm(), 1), (catalog::grizzly_bears(), 0)] {
        let mut g = pod(2);
        stock_libraries(&mut g, 5);
        let mine = ready(&mut g, 0, def);
        let theirs = ready(&mut g, 1, catalog::llanowar_elves());
        let spell = g.add_card_to_hand(0, catalog::hog_monkey_rampage());
        cast_with(&mut g, spell, Some(Target::Permanent(mine)), vec![Target::Permanent(theirs)]).expect("cast");
        assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), counter);
        assert!(g.battlefield_find(theirs).is_none(), "the Elves lose the fight");
    }
}

/// Michelangelo, Improviser: combat damage lets him drop a creature and a
/// land from hand.
#[test]
fn michelangelo_improviser_drops_a_creature_and_a_land() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let mikey = ready(&mut g, 0, catalog::michelangelo_improviser());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    attack_unblocked(&mut g, mikey, 1);
    assert!(g.battlefield_find(bears).is_some() && g.battlefield_find(forest).is_some());
}

/// Pirate Ship is sacrificed once you control no Island (a state trigger,
/// CR 603.8).
#[test]
fn pirate_ship_dies_without_islands() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let island = ready(&mut g, 0, catalog::island());
    let ship = ready(&mut g, 0, catalog::pirate_ship());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Destroy { what: crabomination::effect::Selector::ExactObjects(vec![island]) }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(ship).is_none());
}

/// CR 702.190a — Splinter's Technique sneaks for {1}{B} in the declare
/// blockers step, returning the unblocked attacker.
#[test]
fn splinters_technique_sneaks_from_an_unblocked_attacker() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let tech = g.add_card_to_hand(0, catalog::splinters_technique());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative { card_id: tech, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("sneak");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears), "the attacker went back to hand");
}

// ── cmdr_oct16_b ────────────────────────────────────────────────────────────

/// Final-Sting Faerie and Hooded Assassin destroy only a creature that was
/// dealt damage this turn.
#[test]
fn final_sting_faerie_kills_a_damaged_creature() {
    for damaged in [true, false] {
        let mut g = pod(2);
        stock_libraries(&mut g, 5);
        let theirs = ready(&mut g, 1, catalog::grizzly_bears());
        if damaged {
            let ctx = EffectContext::for_spell(0, None, 0, 0);
            let dmg = Effect::DealDamage { to: crabomination::effect::Selector::ExactObjects(vec![theirs]), amount: crabomination::card::Value::ONE };
            let evs = g.resolve_effect(&dmg, &ctx).unwrap();
            g.dispatch_triggers_for_events(&evs);
        }
        let faerie = g.add_card_to_hand(0, catalog::final_sting_faerie());
        cast_with(&mut g, faerie, None, vec![]).expect("cast");
        assert_eq!(g.battlefield_find(theirs).is_none(), damaged);
    }
}

/// Abomination grows by the combat damage another Villain deals a player.
#[test]
fn abomination_counts_a_villains_combat_damage() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let abom = ready(&mut g, 0, catalog::abomination_irradiated_brute());
    attack_unblocked(&mut g, abom, 1);
    assert_eq!(g.battlefield_find(abom).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

/// Repulsor Blast: with teamwork paid it also hits the creature's
/// controller for 2.
#[test]
fn repulsor_blast_teamwork_hits_the_controller() {
    for kicked in [false, true] {
        let mut g = pod(2);
        stock_libraries(&mut g, 5);
        ready(&mut g, 0, catalog::grizzly_bears());
        let theirs = ready(&mut g, 1, catalog::craw_wurm());
        let blast = g.add_card_to_hand(0, catalog::repulsor_blast());
        let life = g.players[1].life;
        flood(&mut g);
        let (target, additional_targets, mode, x_value) = (Some(Target::Permanent(theirs)), vec![], None, None);
        let action = if kicked {
            GameAction::CastSpellKicked { card_id: blast, target, additional_targets, mode, x_value }
        } else {
            GameAction::CastSpell { card_id: blast, target, additional_targets, mode, x_value }
        };
        g.perform_action(action).expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(theirs).is_none());
        assert_eq!(life - g.players[1].life, if kicked { 2 } else { 0 });
    }
}

/// CR 506.4 / 702.190a — The Last Ronin's Technique: sneak returns the only
/// attacker, but the combat goes on, so the Spirits still enter tapped and
/// attacking (they did nothing with no attacker left). Bug fix.
#[test]
fn last_ronins_technique_sneaked_spirits_attack() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let tech = g.add_card_to_hand(0, catalog::the_last_ronins_technique());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative { card_id: tech, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("sneak");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Ninja Turtle Spirit").collect();
    assert_eq!(spirits.len(), 3);
    assert!(spirits.iter().all(|c| c.tapped));
    assert_eq!(g.attacking.len(), 3, "the Bears went home, the Spirits attack");
}

/// Colossus is indestructible only during its controller's turn, and pumps
/// other Mutants.
#[test]
fn colossus_indestructible_on_your_turn_only() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let colossus = ready(&mut g, 0, catalog::colossus_steel_stalwart());
    let quick = ready(&mut g, 0, catalog::quicksilver_brash_blur());
    assert!(g.computed_permanent(colossus).unwrap().keywords().contains(&Keyword::Indestructible));
    assert_eq!(g.computed_permanent(quick).unwrap().power, 2);
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(colossus).unwrap().keywords().contains(&Keyword::Indestructible));
}

/// Legolas pings when you cast a spell targeting a creature you don't
/// control.
#[test]
fn legolas_pings_off_a_spell_targeting_theirs() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    ready(&mut g, 0, catalog::legolas_master_archer());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let elves = ready(&mut g, 1, catalog::llanowar_elves());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Permanent(wurm)), vec![]).expect("bolt");
    // Bolt's 3 alone leaves the 6/4 Wurm alive; one creature took Legolas's 1.
    assert!(g.battlefield_find(elves).is_none() || g.battlefield_find(wurm).is_none(), "Legolas dealt his 1");
}

/// Life and Limb: a Forest is a 1/1 green Saproling creature, and a Saproling
/// token is a Forest land, both in addition to their types (CR 613.1d).
#[test]
fn life_and_limb_makes_forests_saprolings_and_saprolings_forests() {
    use crabomination::card::{CardType, CreatureType, LandType};
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    ready(&mut g, 0, catalog::life_and_limb());
    let forest = ready(&mut g, 0, catalog::forest());
    let swarm = g.add_card_to_hand(0, catalog::spore_swarm());
    cast_with(&mut g, swarm, None, vec![]).expect("swarm");
    let f = g.computed_permanent(forest).unwrap();
    assert!(f.card_types().contains(&CardType::Creature) && f.subtypes().creature_types.contains(&CreatureType::Saproling));
    assert_eq!((f.power, f.toughness), (1, 1));
    assert!(f.colors.contains(&Color::Green));
    let token = g.battlefield.iter().find(|c| c.definition.name == "Saproling").map(|c| c.id).unwrap();
    let t = g.computed_permanent(token).unwrap();
    assert!(t.card_types().contains(&CardType::Land) && t.subtypes().land_types.contains(&LandType::Forest));
}

/// Savage Thallid: sacrificing a Saproling (an activation cost) regenerates
/// target Fungus.
#[test]
fn savage_thallid_sacrifices_a_saproling_to_regenerate() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let thallid = ready(&mut g, 0, catalog::savage_thallid());
    let swarm = g.add_card_to_hand(0, catalog::spore_swarm());
    cast_with(&mut g, swarm, None, vec![]).expect("swarm");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: thallid, ability_index: 1, target: Some(Target::Permanent(thallid)), additional_targets: vec![], x_value: None, mode: None })
        .expect("regenerate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count(), 2);
    assert_eq!(g.battlefield_find(thallid).unwrap().regeneration_shields, 1);
}
