//! Commander: the 2026-10-17 (`019Y6J4R`) first closer batch
//! (`decks::cmdr_019y_a`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::card::Value;
use crabomination::effect::{Effect, Selector};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(6);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    cast_mode(g, id, target, None)
}

fn cast_mode(
    g: &mut GameState,
    id: CardId,
    target: Option<Target>,
    mode: Option<usize>,
) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode,
        x_value: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn in_exile(g: &GameState, id: CardId) -> bool {
    g.exile.iter().any(|c| c.id == id)
}

/// Gilt-Leaf Winnower's ETB destroys a non-Elf whose power and toughness
/// differ; a square creature and an unequal Elf are not legal targets.
#[test]
fn gilt_leaf_winnower_kills_only_unequal_non_elves() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let warrior = ready(&mut g, 1, catalog::elvish_warrior());
    let lions = ready(&mut g, 1, catalog::savannah_lions());
    let gw = g.add_card_to_hand(0, catalog::gilt_leaf_winnower());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, gw, None).expect("cast");
    assert!(g.battlefield_find(lions).is_none(), "2/1 non-Elf dies");
    assert!(g.battlefield_find(bears).is_some(), "2/2 is square");
    assert!(g.battlefield_find(warrior).is_some(), "2/3 Elf is an Elf");
}

/// Vicious Battlerager takes the initiative entering; each creature blocking
/// it costs its controller 5 life.
#[test]
fn vicious_battlerager_punishes_each_blocker() {
    let mut g = pod(2);
    let vb = g.add_card_to_hand(0, catalog::vicious_battlerager());
    flood(&mut g, 0);
    cast(&mut g, vb, None).expect("cast");
    assert_eq!(g.initiative, Some(0));
    g.clear_sickness(vb);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::savannah_lions());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: vb,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let life = g.players[1].life;
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(a, vb), (b, vb)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 10, "two blockers, 5 each");
}

/// Safana makes one Treasure at your end step with the initiative, three once
/// you've completed a dungeon, and none without the initiative (CR 603.4).
#[test]
fn safana_treasures_scale_with_a_completed_dungeon() {
    let treasures = |g: &GameState| {
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count()
    };
    let mut g = pod(2);
    ready(&mut g, 0, catalog::safana_calimport_cutthroat());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(treasures(&g), 0, "no initiative");
    g.initiative = Some(0);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(treasures(&g), 1);
    g.players[0].dungeons_completed = 1;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(treasures(&g), 4);
}

/// Fury Charm's third mode takes two time counters off a permanent.
#[test]
fn fury_charm_removes_two_time_counters() {
    let mut g = pod(2);
    let rock = ready(&mut g, 0, catalog::sol_ring());
    g.battlefield_find_mut(rock).unwrap().add_counters(CounterType::Time, 3);
    let fc = g.add_card_to_hand(0, catalog::fury_charm());
    flood(&mut g, 0);
    cast_mode(&mut g, fc, Some(Target::Permanent(rock)), Some(2)).expect("cast");
    assert_eq!(g.battlefield_find(rock).unwrap().counter_count(CounterType::Time), 1);
}

/// Pearl Lake Ancient returns itself to hand by bouncing three lands.
#[test]
fn pearl_lake_ancient_bounces_itself_for_three_lands() {
    let mut g = pod(2);
    let pla = ready(&mut g, 0, catalog::pearl_lake_ancient());
    let lands: Vec<_> = (0..3).map(|_| ready(&mut g, 0, catalog::island())).collect();
    g.perform_action(GameAction::ActivateAbility {
        card_id: pla,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("three lands to return");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == pla));
    assert!(lands.iter().all(|l| g.players[0].hand.iter().any(|c| c.id == *l)));
}

/// Warrior's Oath banks an extra turn; Alchemist's Gambit does too and exiles
/// itself, and its cleave cast drops the lose-the-game rider (CR 702.148).
#[test]
fn doomed_extra_turns_and_the_cleaved_gambit() {
    let mut g = pod(2);
    let oath = g.add_card_to_hand(0, catalog::warriors_oath());
    flood(&mut g, 0);
    cast(&mut g, oath, None).expect("cast");
    assert_eq!(g.players[0].extra_turns, 1);
    let lose_riders = g.delayed_triggers.len();
    assert!(lose_riders >= 1);

    let mut g = pod(2);
    let gambit = g.add_card_to_hand(0, catalog::alchemists_gambit());
    flood(&mut g, 0);
    cast(&mut g, gambit, None).expect("cast");
    assert_eq!(g.players[0].extra_turns, 1);
    assert!(in_exile(&g, gambit), "exiles itself");
    let uncleaved = g.delayed_triggers.len();

    let mut g = pod(2);
    let gambit = g.add_card_to_hand(0, catalog::alchemists_gambit());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: gambit,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cleave");
    drain_stack(&mut g);
    assert_eq!(g.players[0].extra_turns, 1);
    assert_eq!(g.delayed_triggers.len(), uncleaved - 1, "no lose-the-game delayed trigger");
}

/// Tamiyo Meets the Story Circle II: two Clues per card discarded.
#[test]
fn tamiyo_story_circle_investigates_twice_per_discard() {
    let mut g = pod(2);
    let h1 = g.add_card_to_hand(0, catalog::forest());
    let h2 = g.add_card_to_hand(0, catalog::forest());
    let saga = g.add_card_to_hand(0, catalog::tamiyo_meets_the_story_circle());
    flood(&mut g, 0);
    cast(&mut g, saga, None).expect("cast");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![h1, h2])]));
    let add = Effect::AddCounter {
        what: Selector::ExactObjects(vec![saga]),
        kind: CounterType::Lore,
        amount: Value::ONE,
    };
    let evs = g.resolve_effect(&add, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let clues =
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Clue").count();
    assert_eq!(clues, 4);
}

/// Eater of Virtue exiles the creature that dies wearing it, and the next
/// wearer has that card's flying (and only the listed keywords it has).
#[test]
fn eater_of_virtue_passes_on_the_dead_wearers_flying() {
    let mut g = pod(2);
    let eater = ready(&mut g, 0, catalog::eater_of_virtue());
    let angel = ready(&mut g, 0, catalog::serra_angel());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: eater, target: angel }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(pt(&g, angel), (6, 4));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(angel))).expect("bolt");
    cast(&mut g, bolt2, Some(Target::Permanent(angel))).expect("bolt");
    assert!(in_exile(&g, angel), "the dead wearer is exiled");
    g.perform_action(GameAction::Equip { equipment: eater, target: bear }).expect("equip");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    let kws = cp.keywords();
    assert!(kws.contains(&Keyword::Flying) && kws.contains(&Keyword::Vigilance));
    assert!(!kws.contains(&Keyword::Trample));
}
