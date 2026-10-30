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
    assert!(f.colors.contains(Color::Green));
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

/// Fear, Fire, Foes!: X to the target, 1 to each of its controller's other
/// creatures, and none to anyone else's.
#[test]
fn fear_fire_foes_splashes_the_targets_controller_only() {
    let mut g = pod(3);
    stock_libraries(&mut g, 5);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let elves = ready(&mut g, 1, catalog::llanowar_elves());
    let bystander = ready(&mut g, 2, catalog::llanowar_elves());
    let mine = ready(&mut g, 0, catalog::llanowar_elves());
    let spell = g.add_card_to_hand(0, catalog::fear_fire_foes());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: spell, target: Some(Target::Permanent(wurm)), additional_targets: vec![], mode: None, x_value: Some(4) })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_none(), "4 to the 6/4");
    assert!(g.battlefield_find(elves).is_none(), "1 to its controller's other creature");
    assert!(g.battlefield_find(bystander).is_some() && g.battlefield_find(mine).is_some());
}

// ── cmdr_oct16_c ────────────────────────────────────────────────────────────

fn cast(g: &mut GameState, id: CardId) {
    cast_with(g, id, None, vec![]).expect("cast");
}

/// Cowabunga! takes a Turtle over a bigger non-TMNT card; with neither a
/// Mutant/Ninja/Turtle nor a land among the four, nothing comes.
#[test]
fn cowabunga_takes_only_tmnt_or_land() {
    for (def, taken) in [(catalog::craw_wurm(), false), (catalog::forest(), true)] {
        let mut g = pod(2);
        let card = g.add_card_to_library(0, def);
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::lightning_bolt());
        }
        stock_libraries(&mut g, 5);
        let spell = g.add_card_to_hand(0, catalog::cowabunga());
        cast(&mut g, spell);
        assert_eq!(g.players[0].hand.iter().any(|c| c.id == card), taken);
        assert_eq!(g.players[0].hand.len(), taken as usize);
    }
}

/// Pictures of Spider-Man takes up to two creatures from five on entering,
/// then cashes itself in for a Treasure.
#[test]
fn pictures_of_spider_man_takes_two_creatures_then_a_treasure() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::forest());
    stock_libraries(&mut g, 5);
    let pics = g.add_card_to_hand(0, catalog::pictures_of_spider_man());
    cast(&mut g, pics);
    assert_eq!(g.players[0].hand.len(), 2);
    assert!(g.players[0].hand.iter().all(|c| c.definition.name == "Grizzly Bears"));
    g.clear_sickness(pics);
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility { card_id: pics, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None }).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(pics).is_none());
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 1);
}

/// Origin of Spider-Man: chapter I's Spider is chapter II's only target and
/// stays a legendary Spider Hero with a counter afterwards (no duration).
#[test]
fn origin_of_spider_man_makes_a_legendary_hero() {
    use crabomination::card::CreatureType;
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let saga = g.add_card_to_hand(0, catalog::origin_of_spider_man());
    cast(&mut g, saga);
    let spider = g.battlefield.iter().find(|c| c.definition.name == "Spider").expect("chapter I").id;
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    let cp = g.computed_permanent(spider).unwrap();
    assert!(cp.supertypes().contains(&crabomination::card::Supertype::Legendary));
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Hero));
    assert_eq!(g.battlefield_find(spider).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Auditore Ambush, both modes: bounce a creature and fetch Ezio — from the
/// graveyard here, the "and/or graveyard" half.
#[test]
fn auditore_ambush_bounces_and_fetches_ezio_from_the_graveyard() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let ezio = g.add_card_to_graveyard(0, catalog::ezio_blade_of_vengeance());
    let spell = g.add_card_to_hand(0, catalog::auditore_ambush());
    flood(&mut g);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: spell,
        spree_modes: vec![0, 1],
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![Target::Player(0)],
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bears));
    assert!(g.players[0].hand.iter().any(|c| c.id == ezio));
}

/// Bortuk Bonerattle: domain 2 returns a two-drop to the battlefield, sends a
/// six-drop to hand, and does nothing when Bortuk wasn't cast.
#[test]
fn bortuk_returns_by_domain_when_cast() {
    for (def, onto) in [(catalog::grizzly_bears(), true), (catalog::craw_wurm(), false)] {
        let mut g = pod(2);
        stock_libraries(&mut g, 5);
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(0, catalog::swamp());
        let dead = g.add_card_to_graveyard(0, def);
        let bortuk = g.add_card_to_hand(0, catalog::bortuk_bonerattle());
        cast(&mut g, bortuk);
        assert_eq!(g.battlefield_find(dead).is_some(), onto);
        assert_eq!(g.players[0].hand.iter().any(|c| c.id == dead), !onto);
    }
    let mut g = pod(2);
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::bortuk_bonerattle());
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == dead), "not cast: no trigger");
}

/// Sailors' Bane counts instants/sorceries in exile and the graveyard, both.
#[test]
fn sailors_bane_costs_less_per_spell_in_exile_and_graveyard() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
    }
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_exile(0, catalog::lightning_bolt());
    let bane = g.add_card_to_hand(0, catalog::sailors_bane());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: bane, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{2}{U}{U} after five spells");
}

/// Symbiote Spider-Man digs as deep as its combat damage; Find New Host
/// (from the graveyard, exiling it) hands that ability to another creature.
#[test]
fn symbiote_spider_man_digs_then_finds_a_new_host() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let sym = ready(&mut g, 0, catalog::symbiote_spider_man());
    attack_unblocked(&mut g, sym, 1);
    assert_eq!((g.players[0].hand.len(), g.players[0].graveyard.len()), (1, 1), "2 damage: one kept, one binned");

    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let host = ready(&mut g, 0, catalog::grizzly_bears());
    let sym = g.add_card_to_graveyard(0, catalog::symbiote_spider_man());
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sym,
        ability_index: 0,
        target: Some(Target::Permanent(host)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("find new host");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == sym));
    assert_eq!(g.battlefield_find(host).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    attack_unblocked(&mut g, host, 1);
    assert_eq!((g.players[0].hand.len(), g.players[0].graveyard.len()), (1, 2), "the 3/3 host dug three");
}

/// Jasmine Boreal of the Seven: its {G}{W} pays for a vanilla creature only,
/// and a vanilla attacker can be blocked by a vanilla creature but not by
/// one with abilities.
#[test]
fn jasmine_boreal_feeds_and_shields_vanilla_creatures() {
    let mut g = pod(2);
    let jas = ready(&mut g, 0, catalog::jasmine_boreal_of_the_seven());
    let elves = g.add_card_to_hand(0, catalog::llanowar_elves());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    let tap = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility { card_id: jas, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
            .expect("tap for GW")
    };
    tap(&mut g);
    let cast = |g: &mut GameState, id| {
        g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
    };
    assert!(cast(&mut g, elves).is_err(), "Llanowar Elves has an ability");
    cast(&mut g, bears).expect("Grizzly Bears has none");
    drain_stack(&mut g);

    for (blocker, legal) in [(catalog::llanowar_elves(), false), (catalog::grizzly_bears(), true)] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::jasmine_boreal_of_the_seven());
        let attacker = ready(&mut g, 0, catalog::grizzly_bears());
        let blocker = ready(&mut g, 1, blocker);
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])).expect("attack");
        drain_stack(&mut g);
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let r = g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]));
        assert_eq!(r.is_ok(), legal);
    }
}

/// Web of Life and Destiny drops a creature from the top five at the
/// beginning of combat on its controller's turn.
#[test]
fn web_of_life_and_destiny_deploys_at_beginning_of_combat() {
    let mut g = pod(2);
    let wurm = g.add_card_to_library(0, catalog::craw_wurm());
    stock_libraries(&mut g, 10);
    ready(&mut g, 0, catalog::web_of_life_and_destiny());
    step_to(&mut g, 0, TurnStep::BeginCombat);
    assert!(g.battlefield_find(wurm).is_some());
}

/// Cantankerous Keepers: two Elves out cut its cost by two, and every Elf
/// among the four milled goes to hand while the rest stay binned — "all",
/// so a seat answering an empty pick still gets them (it was an up-to-four ask).
#[test]
fn cantankerous_keepers_takes_every_milled_elf() {
    let mut g = pod(2);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Cards(vec![]),
    ]));
    let elves: Vec<_> = (0..2).map(|_| g.add_card_to_library(0, catalog::llanowar_elves())).collect();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    stock_libraries(&mut g, 5);
    ready(&mut g, 0, catalog::llanowar_elves());
    ready(&mut g, 0, catalog::llanowar_elves());
    let keepers = g.add_card_to_hand(0, catalog::cantankerous_keepers());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: keepers, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{3}{G} with two Elves");
    drain_stack(&mut g);
    assert!(elves.iter().all(|e| g.players[0].hand.iter().any(|c| c.id == *e)));
    assert_eq!(g.players[0].graveyard.len(), 2);
}

/// Sentinel Totem exiles every graveyard, not just one.
#[test]
fn sentinel_totem_exiles_all_graveyards() {
    let mut g = pod(3);
    for s in 0..3 {
        g.add_card_to_graveyard(s, catalog::grizzly_bears());
    }
    let totem = ready(&mut g, 0, catalog::sentinel_totem());
    g.perform_action(GameAction::ActivateAbility { card_id: totem, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("activate");
    drain_stack(&mut g);
    assert!(g.players.iter().all(|p| p.graveyard.is_empty()));
    assert!(g.exile.iter().any(|c| c.id == totem));
}

/// Nissa's Revelation reads the revealed creature's power and toughness
/// before the draw takes it: Craw Wurm (6/4) draws six and gains four.
#[test]
fn nissas_revelation_draws_power_gains_toughness() {
    let mut g = pod(2);
    let wurm = g.add_card_to_library(0, catalog::craw_wurm());
    stock_libraries(&mut g, 10);
    let spell = g.add_card_to_hand(0, catalog::nissas_revelation());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder { kept_top: vec![wurm], bottom: vec![] }]));
    let life = g.players[0].life;
    cast(&mut g, spell);
    assert_eq!(g.players[0].hand.len(), 6);
    assert_eq!(g.players[0].life, life + 4);
}

/// Along the Crooked Way's return fires both "creature card leaves your
/// graveyard" triggers: Undead Hand Ninja drains one, the Way amasses one.
#[test]
fn along_the_crooked_way_return_drains_and_amasses() {
    let mut g = pod(3);
    stock_libraries(&mut g, 5);
    ready(&mut g, 0, catalog::undead_hand_ninja());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let way = g.add_card_to_hand(0, catalog::along_the_crooked_way());
    let life = (g.players[0].life, g.players[1].life, g.players[2].life);
    cast(&mut g, way);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears));
    assert_eq!((g.players[0].life, g.players[1].life, g.players[2].life), (life.0 + 1, life.1 - 1, life.2 - 1));
    let army = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name.contains("Army")).expect("amassed");
    assert_eq!(army.counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Balin, Loremaster with an enduring story: wheel the hand, then hit each
/// opponent for the number discarded.
#[test]
fn balin_wheels_and_burns_with_an_enduring_story() {
    let mut g = pod(3);
    stock_libraries(&mut g, 10);
    ready(&mut g, 0, catalog::sol_ring());
    ready(&mut g, 0, catalog::sol_ring());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    let balin = g.add_card_to_hand(0, catalog::balin_loremaster());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    cast(&mut g, balin);
    assert_eq!(g.players[0].hand.len(), 3, "three discarded, three drawn");
    assert_eq!(g.players[1].life, life - 3);
    assert_eq!(g.players[2].life, life - 3);
}

/// K'un-Lun Warrior: sacrifices an artifact to draw when it has one.
#[test]
fn kun_lun_warrior_sacrifices_an_artifact_to_draw() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let w = g.add_card_to_hand(0, catalog::kun_lun_warrior());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0), DecisionAnswer::Bool(true)]));
    cast(&mut g, w);
    assert!(g.battlefield_find(ring).is_none());
    assert_eq!(g.players[0].hand.len(), 1);
}

/// CR 702.193 — power-up is once per object; Wonder Man, Hollywood Hero lets
/// each of your power-ups go once more, and no further.
#[test]
fn cr_702_193_wonder_man_allows_a_second_power_up() {
    let mut g = pod(2);
    let wm = ready(&mut g, 0, catalog::wonder_man_hollywood_hero());
    let up = |g: &mut GameState| {
        flood(g);
        let r = g.perform_action(GameAction::ActivateAbility { card_id: wm, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None });
        drain_stack(g);
        r
    };
    up(&mut g).expect("first");
    up(&mut g).expect("second, Wonder Man's");
    assert!(up(&mut g).is_err(), "no third");
    assert_eq!(g.battlefield_find(wm).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

/// Shaun & Rebecca fetch The Animus from the graveyard on entering, and
/// their {C} mills two through a reflexive trigger.
#[test]
fn shaun_and_rebecca_fetch_the_animus_and_mill_for_mana() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let animus = g.add_card_to_graveyard(0, catalog::the_animus());
    let sr = g.add_card_to_hand(0, catalog::shaun_rebecca_agents());
    cast(&mut g, sr);
    assert!(g.battlefield_find(animus).is_some());
    g.clear_sickness(sr);
    let gy = g.players[0].graveyard.len();
    g.perform_action(GameAction::ActivateAbility { card_id: sr, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("tap for C");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy + 2);
}

/// Pull from Eternity moves a face-up exiled card to its owner's graveyard.
#[test]
fn pull_from_eternity_returns_an_exiled_card_to_the_graveyard() {
    let mut g = pod(2);
    let exiled = g.add_card_to_exile(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::pull_from_eternity());
    cast_with(&mut g, spell, Some(Target::Permanent(exiled)), vec![]).expect("cast");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == exiled));
}

/// Way of the Mind Sculptor: its Jace token's −3 removes three loyalty and
/// draws an extra card; the −1 removes one and doesn't.
#[test]
fn way_of_the_mind_sculptor_draws_off_a_big_minus() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let way = g.add_card_to_hand(0, catalog::way_of_the_mind_sculptor());
    cast(&mut g, way);
    let jace = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name.contains("Jace")).expect("Jace token").id;
    let act = |g: &mut GameState, i| {
        g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: jace, ability_index: i, target: None, x_value: None }).expect("loyalty");
        drain_stack(g);
    };
    act(&mut g, 0);
    assert_eq!(g.players[0].hand.len(), 0, "the −1 surveils; no draw");
    g.turn_number += 1;
    for c in g.battlefield.iter_mut() {
        c.loyalty_uses_this_turn = 0;
    }
    act(&mut g, 1);
    assert_eq!(g.players[0].hand.len(), 2, "the −3 draws one, the Way one more");
}

/// The Fall of Lord Konda: chapter II hands every stolen permanent back to its
/// owner; chapter III returns the Saga as Fragment of Konda.
#[test]
fn fall_of_lord_konda_returns_stolen_permanents_then_transforms() {
    let mut g = pod(2);
    stock_libraries(&mut g, 20);
    let stolen = ready(&mut g, 1, catalog::grizzly_bears());
    let steal = Effect::GainControl {
        what: crabomination::effect::Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature),
        to: None,
        duration: crabomination::effect::Duration::Permanent,
    };
    g.resolve_effect(&steal, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    let saga = g.add_card_to_hand(0, catalog::the_fall_of_lord_konda());
    cast(&mut g, saga);
    assert_eq!(g.battlefield_find(stolen).unwrap().controller, 0);
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    assert_eq!(g.battlefield_find(stolen).unwrap().controller, 1, "chapter II");
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Fragment of Konda"), "chapter III");
}

/// Polukranos Reborn transforms at sorcery speed for {6}{W/P}; the Engine of
/// Ruin's death makes its two 3/3 Hydras.
#[test]
fn polukranos_transforms_and_leaves_two_hydras() {
    let mut g = pod(2);
    let p = ready(&mut g, 0, catalog::polukranos_reborn());
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility { card_id: p, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("transform");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(p).unwrap().power, 6);
    let bolt = Effect::Destroy { what: crabomination::effect::Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature) };
    let evs = g.resolve_effect(&bolt, &EffectContext::for_spell(1, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Hydra").count(), 2);
}

/// Patch Up returns up to three creature cards whose mana values total 3 or
/// less: two two-drops can't both come.
#[test]
fn patch_up_respects_the_total_mana_value() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::llanowar_elves());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::patch_up());
    cast(&mut g, spell);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 2, "elves + one bears");
}

/// Korlash's grandeur: discard another Korlash to put two Swamps onto the
/// battlefield tapped, and Korlash grows with them.
#[test]
fn korlash_grandeur_fetches_two_swamps() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::swamp());
    }
    stock_libraries(&mut g, 5);
    g.add_card_to_battlefield(0, catalog::swamp());
    let k = ready(&mut g, 0, catalog::korlash_heir_to_blackblade());
    g.add_card_to_hand(0, catalog::korlash_heir_to_blackblade());
    g.perform_action(GameAction::ActivateAbility { card_id: k, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("grandeur");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Swamp" && c.tapped).count(), 2);
    assert_eq!(g.computed_permanent(k).unwrap().power, 3);
}

/// Activated Sleeper copies a creature that died this turn, not one that was
/// milled.
#[test]
fn activated_sleeper_copies_a_creature_that_died_this_turn() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    g.add_card_to_graveyard(1, catalog::craw_wurm());
    let bears = ready(&mut g, 1, catalog::serra_angel());
    let kill = Effect::Destroy { what: crabomination::effect::Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature) };
    g.resolve_effect(&kill, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears));
    let s = g.add_card_to_hand(0, catalog::activated_sleeper());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, s);
    let cp = g.computed_permanent(s).expect("on the battlefield");
    assert_eq!((cp.power, cp.toughness), (4, 4), "a Serra Angel, not the Wurm");
}

/// Reach of Branches comes back from the graveyard when a Forest enters
/// under its owner's control (a graveyard-scoped landfall trigger).
#[test]
fn reach_of_branches_returns_on_a_forest() {
    let mut g = pod(2);
    let reach = g.add_card_to_graveyard(0, catalog::reach_of_branches());
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::PlayLand(forest)).expect("land");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == reach));
    // "Enters", not "is played": a Forest an effect puts there counts too.
    let mut put = pod(2);
    let reach = put.add_card_to_graveyard(0, catalog::reach_of_branches());
    let forest = put.add_card_to_hand(0, catalog::forest());
    put.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let drop = crabomination::effect::Effect::Move {
        what: crabomination::effect::Selector::ExactObjects(vec![forest]),
        to: crabomination::effect::ZoneDest::Battlefield {
            controller: crabomination::effect::PlayerRef::You,
            tapped: false,
        },
    };
    let evs = put.resolve_effect(&drop, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    put.dispatch_triggers_for_events(&evs);
    drain_stack(&mut put);
    assert!(put.players[0].hand.iter().any(|c| c.id == reach), "put onto the battlefield");
}

/// Unstoppable Ash: a blocked creature of yours gets +0/+5.
#[test]
fn unstoppable_ash_toughens_a_blocked_creature() {
    let mut g = pod(2);
    let ash = ready(&mut g, 0, catalog::unstoppable_ash());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(wurm, bears)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bears).unwrap().toughness, 7);
    assert!(g.battlefield_find(ash).is_some());
}

/// Grond is a creature only on its controller's turn with an Army out.
#[test]
fn grond_animates_on_your_turn_with_an_army() {
    use crabomination::card::CardType;
    let mut g = pod(2);
    let grond = ready(&mut g, 0, catalog::grond_the_gatebreaker());
    let is_creature = |g: &GameState| g.computed_permanent(grond).unwrap().card_types().contains(&CardType::Creature);
    assert!(!is_creature(&g), "no Army");
    let amass = Effect::Amass { who: PlayerRef::You, count: crabomination::card::Value::ONE, extra_type: None };
    g.resolve_effect(&amass, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert!(is_creature(&g), "your turn, an Army");
    g.active_player_idx = 1;
    assert!(!is_creature(&g), "not your turn");
}

/// Gorbag: an Orc that connects may be sacrificed for a card.
#[test]
fn gorbag_cashes_in_a_connecting_orc() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let gorbag = ready(&mut g, 0, catalog::gorbag_of_minas_morgul());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack_unblocked(&mut g, gorbag, 1);
    assert!(g.battlefield_find(gorbag).is_none(), "sacrificed");
    assert_eq!(g.players[0].hand.len() + g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 1);
}

/// Llanowar Empath keeps a creature it reveals after the scry.
#[test]
fn llanowar_empath_takes_a_revealed_creature() {
    let mut g = pod(2);
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    stock_libraries(&mut g, 5);
    let e = g.add_card_to_hand(0, catalog::llanowar_empath());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder { kept_top: vec![bears], bottom: vec![] }]));
    cast(&mut g, e);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears));
}

/// Galion makes another attacker's base P/T its own 4/4.
#[test]
fn galion_copies_its_size_onto_a_creature() {
    let mut g = pod(2);
    let galion = ready(&mut g, 0, catalog::galion_elvenkings_butler());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: galion, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(elf).unwrap().power, 4);
}

/// Training Compound's colored mana needs a basic land (or the turn it
/// entered); colorless is always there.
#[test]
fn training_compound_colors_need_a_basic_or_a_fresh_entry() {
    let mut g = pod(2);
    let tc = ready(&mut g, 0, catalog::training_compound());
    g.battlefield.iter_mut().for_each(|c| c.entered_turn = None);
    let tap = |g: &mut GameState, i| {
        g.perform_action(GameAction::ActivateAbility { card_id: tc, ability_index: i, target: None, additional_targets: vec![], x_value: None, mode: None })
    };
    assert!(tap(&mut g, 1).is_err(), "no basic, entered earlier");
    g.add_card_to_battlefield(0, catalog::forest());
    tap(&mut g, 1).expect("with a basic");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
}

/// Rhino's Rampage: a 3/3 (+1/+0) fighting a 1/1 deals excess damage, so the
/// reflexive trigger destroys a cheap noncreature artifact.
#[test]
fn rhinos_rampage_excess_destroys_an_artifact() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let spell = g.add_card_to_hand(0, catalog::rhinos_rampage());
    cast_with(&mut g, spell, Some(Target::Permanent(bears)), vec![Target::Permanent(elf)]).expect("cast");
    assert!(g.battlefield_find(elf).is_none());
    assert!(g.battlefield_find(ring).is_none(), "excess damage → Sol Ring destroyed");
}

/// Rhino's Rampage's reflexive destroy is "up to one": with only its caster's
/// own Sol Ring eligible, it declines instead of destroying it (CR 601.2c).
#[test]
fn rhinos_rampage_spares_its_own_artifact() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let mine = ready(&mut g, 0, catalog::sol_ring());
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    let spell = g.add_card_to_hand(0, catalog::rhinos_rampage());
    cast_with(&mut g, spell, Some(Target::Permanent(bears)), vec![Target::Permanent(elf)]).expect("cast");
    assert!(g.battlefield_find(elf).is_none());
    assert!(g.battlefield_find(mine).is_some(), "own Sol Ring survives");
}

/// Sméagol: Mirrormere Guardian dying tempts you, so an opponent reveals until
/// a land — the land is yours tapped, the cards above it go to their graveyard.
#[test]
fn smeagol_steals_a_land_when_the_ring_tempts() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let bolt = g.add_card_to_library(1, catalog::lightning_bolt());
    let land = g.add_card_to_library(1, catalog::forest());
    g.players[1].library.retain(|c| c.id == bolt || c.id == land);
    ready(&mut g, 0, catalog::smeagol_helpful_guide());
    ready(&mut g, 0, catalog::mirrormere_guardian());
    let kill = Effect::Destroy {
        what: crabomination::effect::Selector::EachPermanent(crabomination::card::SelectionRequirement::HasName("Mirrormere Guardian".into())),
    };
    let evs = g.resolve_effect(&kill, &EffectContext::for_spell(1, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let l = g.battlefield_find(land).expect("stolen land");
    assert!(l.controller == 0 && l.tapped);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

/// The Destined Thief loots once when creatures connect with two different
/// players in the same damage step (CR 603.2c "one or more players").
#[test]
fn destined_thief_loots_once_across_players() {
    let mut g = pod(3);
    stock_libraries(&mut g, 5);
    let thief = ready(&mut g, 0, catalog::the_destined_thief());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: thief, target: AttackTarget::Player(1) },
        Attack { attacker: bears, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!((g.players[0].hand.len(), g.players[0].graveyard.len()), (1, 1), "one loot");
}

/// Nimble Trapfinder can't be blocked once another Rogue entered this turn.
#[test]
fn nimble_trapfinder_unblockable_after_a_rogue_enters() {
    let mut g = pod(2);
    let t = ready(&mut g, 0, catalog::nimble_trapfinder());
    let has = |g: &GameState| g.computed_permanent(t).unwrap().keywords().contains(&crabomination::card::Keyword::Unblockable);
    assert!(!has(&g));
    let thief = g.add_card_to_hand(0, catalog::the_destined_thief());
    cast(&mut g, thief);
    assert!(has(&g));
}

/// Tresserhorn's Lord stays only by sacrificing three other creatures (which
/// its own sacrifice trigger turns into three cards), paying 3 life and
/// feeding an opponent three cards; with two others it is sacrificed.
#[test]
fn tresserhorns_lord_demands_three_creatures() {
    for (others, stays) in [(3, true), (2, false)] {
        let mut g = pod(2);
        stock_libraries(&mut g, 10);
        for _ in 0..others {
            ready(&mut g, 0, catalog::grizzly_bears());
        }
        let lord = g.add_card_to_hand(0, catalog::tresserhorns_lord_returned());
        let life = g.players[0].life;
        cast(&mut g, lord);
        assert_eq!(g.battlefield_find(lord).is_some(), stays);
        if stays {
            assert_eq!(g.players[0].life, life - 3);
            assert_eq!(g.players[1].hand.len(), 3);
            assert_eq!(g.players[0].hand.len(), 3, "three sacrifices, three draws");
        }
    }
}

/// Dismiss into Dream: an opponent's creature targeted by anything (a Bolt
/// here) is sacrificed; yours aren't Illusions and keep no such trigger.
#[test]
fn dismiss_into_dream_sacrifices_targeted_opposing_creatures() {
    use crabomination::card::CreatureType;
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::dismiss_into_dream());
    assert!(g.computed_permanent(theirs).unwrap().subtypes().creature_types.contains(&CreatureType::Illusion));
    assert!(!g.computed_permanent(mine).unwrap().subtypes().creature_types.contains(&CreatureType::Illusion));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(theirs)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == theirs));
}

/// Psychic Pickpocket connives, then bounces a nonland permanent.
#[test]
fn psychic_pickpocket_connives_and_bounces() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let pp = g.add_card_to_hand(0, catalog::psychic_pickpocket());
    cast(&mut g, pp);
    assert!(g.players[1].hand.iter().any(|c| c.id == angel));
}

/// Psychic Pickpocket's bounce is "up to one": alone with its own permanents it
/// bounces nothing.
#[test]
fn psychic_pickpocket_spares_its_own_permanents() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let pp = g.add_card_to_hand(0, catalog::psychic_pickpocket());
    cast(&mut g, pp);
    assert!(g.battlefield_find(ring).is_some() && g.battlefield_find(pp).is_some());
}

/// King of the Oathbreakers phases out when a spell targets it (the Bolt
/// fizzles), then makes a tapped Spirit when it phases back in.
#[test]
fn king_of_the_oathbreakers_phases_out_from_a_spell() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let king = ready(&mut g, 0, catalog::king_of_the_oathbreakers());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(king)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert!(g.phased_out.iter().any(|c| c.id == king), "phased out, not dead");
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    assert!(g.battlefield_find(king).is_some(), "phased back in");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Spirit" && c.tapped));
}

/// Cauldron Haze: a persisted creature comes back with a -1/-1 counter.
#[test]
fn cauldron_haze_grants_persist() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let haze = g.add_card_to_hand(0, catalog::cauldron_haze());
    cast_with(&mut g, haze, Some(Target::Permanent(bears)), vec![]).expect("haze");
    let kill = Effect::Destroy { what: crabomination::effect::Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature) };
    let evs = g.resolve_effect(&kill, &EffectContext::for_spell(1, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("persisted");
    assert_eq!(back.counter_count(CounterType::MinusOneMinusOne), 1);
}
