//! Commander: the 2026-10-13 rescan's two-card gaps, second half
//! (`decks::cmdr_edhrec25`).

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

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
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

/// Calamity Bearer doubles a Giant's damage.
#[test]
fn calamity_bearer_doubles_giants() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::calamity_bearer());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    attack(&mut g, &[giant]);
    damage(&mut g);
    assert_eq!(g.players[1].life, 14);
}

/// Evra trades your life total for her power.
#[test]
fn evra_exchanges_life_with_power() {
    let mut g = pod(2);
    let evra = ready(&mut g, 0, catalog::evra_halcyon_witness());
    activate(&mut g, evra, None).expect("exchange");
    assert_eq!(g.players[0].life, 4);
    assert_eq!(pt(&g, evra), (20, 4));
}

/// Staff of the Sun Magus: a white spell and a Plains each gain 1.
#[test]
fn staff_of_the_sun_magus_gains() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::staff_of_the_sun_magus());
    let asc = g.add_card_to_hand(0, catalog::enlightened_ascetic());
    cast(&mut g, 0, asc, &[]).expect("white spell");
    let plains = g.add_card_to_hand(0, catalog::plains());
    g.perform_action(GameAction::PlayLand(plains)).expect("land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 22);
}

/// Urdnan: one counter grants first strike, two double strike.
#[test]
fn urdnan_strikes_by_counters() {
    for (n, kw) in [(1, Keyword::FirstStrike), (2, Keyword::DoubleStrike)] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::urdnan_dromoka_warrior());
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, n);
        attack(&mut g, &[bear]);
        assert!(g.computed_permanent(bear).unwrap().keywords().contains(&kw), "{n} counters");
    }
}

/// Lich's Relic: paying {2} on entry kills a creature per opponent.
#[test]
fn lichs_relic_executes_per_opponent() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::hill_giant());
    let relic = g.add_card_to_hand(0, catalog::lichs_relic());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, 0, relic, &[]).expect("cast");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Gix's Command's default pair: counters + lifelink, and each opponent
/// sacrifices their biggest creature.
#[test]
fn gixs_command_default_modes() {
    let mut g = pod(3);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let small = ready(&mut g, 1, catalog::grizzly_bears());
    let big = ready(&mut g, 1, catalog::hill_giant());
    let other = ready(&mut g, 2, catalog::craw_wurm());
    let cmd = g.add_card_to_hand(0, catalog::gixs_command());
    cast(&mut g, 0, cmd, &[]).expect("cast");
    assert_eq!(pt(&g, mine), (4, 4));
    assert!(g.computed_permanent(mine).unwrap().keywords().contains(&Keyword::Lifelink));
    assert!(g.battlefield_find(big).is_none() && g.battlefield_find(other).is_none());
    assert!(g.battlefield_find(small).is_some());
}

/// "Up to one creature" is any creature, chosen as the mode resolves (it was
/// an engine pick among yours); with none of its own a seat picks nothing
/// rather than growing an opponent's.
#[test]
fn gixs_command_never_grows_an_opponents_creature_by_default() {
    let mut g = pod(2);
    let small = ready(&mut g, 1, catalog::grizzly_bears());
    let big = ready(&mut g, 1, catalog::hill_giant());
    let cmd = g.add_card_to_hand(0, catalog::gixs_command());
    cast(&mut g, 0, cmd, &[]).expect("cast");
    assert!(g.battlefield_find(big).is_none(), "their biggest is sacrificed");
    assert_eq!(pt(&g, small), (2, 2));
}

/// Katilda and Lier: casting a Human gives a graveyard instant flashback.
#[test]
fn katilda_grants_flashback_on_humans() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::katilda_and_lier());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let human = g.add_card_to_hand(0, catalog::thieves_guild_enforcer());
    cast(&mut g, 0, human, &[]).expect("human");
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFlashback {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flashback");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Whisper sacrifices two creatures to reanimate one.
#[test]
fn whisper_trades_two_for_one() {
    let mut g = pod(2);
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let whisper = ready(&mut g, 0, catalog::whisper_blood_liturgist());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, whisper, Some(Target::Permanent(wurm))).expect("activate");
    assert!(g.battlefield_find(wurm).is_some());
}

/// Soul Exchange exiles a creature to reanimate another.
#[test]
fn soul_exchange_exiles_to_reanimate() {
    let mut g = pod(2);
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::soul_exchange());
    cast(&mut g, 0, id, &[Target::Permanent(wurm)]).expect("cast");
    assert!(g.battlefield_find(wurm).is_some());
    assert!(g.exile.iter().any(|c| c.id == bear));
}

/// Soulbright Flamekin's third resolution in a turn adds eight red.
#[test]
fn soulbright_flamekin_third_time() {
    let mut g = pod(2);
    let kin = ready(&mut g, 0, catalog::soulbright_flamekin());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    for _ in 0..3 {
        g.players[0].mana_pool = Default::default();
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: kin,
            ability_index: 0,
            target: Some(Target::Permanent(kin)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("activate");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 8);
}

/// Fang returns as a Spirit the first time he dies, then stays dead.
#[test]
fn fang_returns_once_as_a_spirit() {
    let mut g = pod(2);
    let fang = ready(&mut g, 0, catalog::fang_rokus_companion());
    for round in 0..2 {
        let kill = g.add_card_to_hand(1, catalog::murder());
        let id = g.battlefield.iter().find(|c| c.definition.name == "Fang, Roku's Companion").map(|c| c.id);
        let Some(id) = id else { panic!("round {round}: Fang missing") };
        cast(&mut g, 1, kill, &[Target::Permanent(id)]).expect("murder");
        g.check_state_based_actions();
        drain_stack(&mut g);
        let back = g.battlefield.iter().find(|c| c.definition.name == "Fang, Roku's Companion").map(|c| c.id);
        if round == 0 {
            let back = back.expect("returned");
            assert!(g.computed_permanent(back).unwrap().subtypes().creature_types.contains(&CreatureType::Spirit));
        } else {
            assert!(back.is_none(), "a Spirit stays dead");
        }
    }
    let _ = fang;
}

/// Drana: the defending player picks the creature that returns, with a
/// counter, as a Vampire.
#[test]
fn drana_returns_a_creature_as_a_vampire() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let drana = ready(&mut g, 0, catalog::drana_the_last_bloodchief());
    attack(&mut g, &[drana]);
    assert!(g.battlefield_find(bear).is_some());
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Vampire));
}

/// Olivia: a discard turns an entering creature into a hasty Vampire.
#[test]
fn olivia_mobilizes_entering_creatures() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::olivia_mobilized_for_war());
    g.add_card_to_hand(0, catalog::island());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, 0, bear, &[]).expect("cast");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords().contains(&Keyword::Haste));
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Vampire));
}

/// Amarant Coral's combat damage is dealt again to each other opponent.
#[test]
fn amarant_coral_spreads_its_damage() {
    let mut g = pod(4);
    let amarant = ready(&mut g, 0, catalog::amarant_coral());
    attack(&mut g, &[amarant]);
    damage(&mut g);
    assert_eq!((g.players[1].life, g.players[2].life, g.players[3].life), (15, 15, 15));
}

/// Strength of Will: indestructible, and damage becomes counters.
#[test]
fn strength_of_will_turns_damage_into_counters() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::strength_of_will());
    cast(&mut g, 0, id, &[Target::Permanent(bear)]).expect("cast");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, &[Target::Permanent(bear)]).expect("bolt");
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}
