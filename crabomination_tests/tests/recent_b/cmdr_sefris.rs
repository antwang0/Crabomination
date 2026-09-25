//! Commander: the Dungeons of Death precon (AFC, Sefris of the Hidden Ways,
//! `decks::cmdr_sefris`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector, Value};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_x(g, seat, id, target, None)
}

fn activate_x(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
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

fn run(g: &mut GameState, effect: Effect, source: CardId) {
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(source);
    let events = g.resolve_effect(&effect, &ctx).expect("effect");
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn combat_damage(g: &mut GameState) {
    g.step = TurnStep::CombatDamage;
    let events = g.resolve_combat().expect("combat damage");
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

fn in_dungeon_room(g: &GameState, seat: usize) -> Option<u8> {
    g.players[seat].dungeon.as_ref().map(|(_, room)| *room)
}

/// Sefris — creature cards milled together venture once; completing the
/// dungeon returns a creature card.
#[test]
fn sefris_ventures_once_a_turn_and_reanimates_on_completion() {
    let mut g = pod(2);
    let s = g.add_card_to_battlefield(0, catalog::sefris_of_the_hidden_ways());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    // The Temple's draw.
    g.add_card_to_library(0, catalog::island());
    run(&mut g, Effect::Mill { who: Selector::You, amount: Value::Const(2) }, s);
    assert_eq!(in_dungeon_room(&g, 0), Some(0), "one venture for the batch");
    run(&mut g, Effect::Mill { who: Selector::You, amount: Value::Const(1) }, s);
    assert_eq!(in_dungeon_room(&g, 0), Some(0), "once each turn");
    // Walk Lost Mine to the Temple: Create Undead brings a Bear back.
    for _ in 0..3 {
        run(&mut g, Effect::Venture, s);
    }
    assert_eq!(g.players[0].dungeons_completed, 1);
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1, "reanimated on completion");
}

/// Hama Pashar — room abilities trigger twice, and the dungeon still
/// completes once.
#[test]
fn hama_pashar_doubles_room_abilities() {
    let mut g = pod(2);
    let h = g.add_card_to_battlefield(0, catalog::hama_pashar_ruin_seeker());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    // Cave Entrance, then Goblin Lair.
    run(&mut g, Effect::Venture, h);
    run(&mut g, Effect::Venture, h);
    assert_eq!(named(&g, 0, "Goblin").len(), 2, "Goblin Lair twice");
    let hand = g.players[0].hand.len();
    run(&mut g, Effect::Venture, h);
    run(&mut g, Effect::Venture, h);
    assert_eq!(g.players[0].dungeons_completed, 1, "completed once");
    assert_eq!(g.players[0].hand.len(), hand + 2, "Temple drew twice");
    assert!(g.players[0].dungeon.is_none());
}

/// Arcane Endeavor — draw one die, cast a cheap instant from hand free off
/// the other.
#[test]
fn arcane_endeavor_draws_and_casts_free() {
    let mut g = pod(2);
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::island());
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let ae = g.add_card_to_hand(0, catalog::arcane_endeavor());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::DieRoll(2),
        DecisionAnswer::DieRoll(5),
        DecisionAnswer::Amount(1),
        DecisionAnswer::Cards(vec![bolt]),
    ]));
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, ae, None).expect("arcane endeavor");
    // −1 for the Endeavor, −1 for the Bolt, + the draw.
    let drew = g.players[0].hand.len() + 2 - hand;
    assert!(drew == 2 || drew == 5, "drew one result ({drew})");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "Bolt cast for free");
    assert_eq!(g.players[1].life, 17);
}

/// Bucknard's Everfull Purse — Treasures for the roll, then the Purse moves
/// to the player on your right.
#[test]
fn bucknards_purse_passes_right() {
    let mut g = pod(3);
    let p = g.add_card_to_battlefield(0, catalog::bucknards_everfull_purse());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(3)]));
    activate_x(&mut g, 0, p, 0, None, None).expect("purse");
    assert_eq!(named(&g, 0, "Treasure").len(), 3);
    assert_eq!(g.battlefield_find(p).unwrap().controller, 2, "seat 2 sits to seat 0's right");
}

/// Grave Endeavor — reanimate with counters off one die, drain off the
/// other.
#[test]
fn grave_endeavor_reanimates_and_drains() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ge = g.add_card_to_hand(0, catalog::grave_endeavor());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::DieRoll(4),
        DecisionAnswer::DieRoll(7),
        DecisionAnswer::Amount(0),
    ]));
    cast(&mut g, 0, ge, None).expect("grave endeavor");
    let counters = g.battlefield_find(bear).expect("back").counter_count(CounterType::PlusOnePlusOne);
    let drained = 20 - g.players[1].life;
    assert_eq!((counters + drained as u32), 11, "the two results split");
    assert_eq!(g.players[0].life, 20 + drained);
}

/// Immovable Rod — its tap strips another permanent while it stays tapped;
/// untapping ventures.
#[test]
fn immovable_rod_locks_while_tapped_and_ventures_on_untap() {
    let mut g = pod(2);
    let rod = g.add_card_to_battlefield(0, catalog::immovable_rod());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    activate_x(&mut g, 0, rod, 0, Some(Target::Permanent(angel)), None).expect("rod");
    let c = g.computed_permanent(angel).unwrap();
    assert!(!c.keywords().contains(&Keyword::Flying), "lost its abilities");
    assert!(c.keywords().contains(&Keyword::CantAttack));
    assert!(c.keywords().contains(&Keyword::CantBlock));
    run(&mut g, Effect::Untap { what: Selector::This, up_to: None }, rod);
    assert!(g.computed_permanent(angel).unwrap().keywords().contains(&Keyword::Flying), "back once untapped");
    assert_eq!(in_dungeon_room(&g, 0), Some(0), "untapping ventured");
}

/// Midnight Pathlighter — only legends block your creatures; connecting
/// ventures.
#[test]
fn midnight_pathlighter_evades_and_ventures() {
    let mut g = pod(2);
    let mp = g.add_card_to_battlefield(0, catalog::midnight_pathlighter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g
        .computed_permanent(bear)
        .unwrap()
        .keywords()
        .iter()
        .any(|k| matches!(k, Keyword::CantBeBlockedExceptBy(_))));
    g.clear_sickness(mp);
    g.clear_sickness(bear);
    attack(&mut g, &[mp, bear]);
    combat_damage(&mut g);
    assert_eq!(in_dungeon_room(&g, 0), Some(0), "one venture for the batch");
}

/// Minimus Containment — the enchanted creature is a Treasure artifact that
/// taps and sacrifices for mana.
#[test]
fn minimus_containment_makes_a_treasure() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let mc = g.add_card_to_hand(0, catalog::minimus_containment());
    cast(&mut g, 0, mc, Some(Target::Permanent(angel))).expect("containment");
    let c = g.computed_permanent(angel).unwrap();
    assert!(!c.card_types().contains(&crabomination::card::CardType::Creature), "no longer a creature");
    assert!(c.card_types().contains(&crabomination::card::CardType::Artifact));
    assert!(!c.keywords().contains(&Keyword::Flying));
    activate_x(&mut g, 1, angel, 0, None, None).expect("treasure ability");
    assert!(g.battlefield_find(angel).is_none(), "sacrificed for mana");
}

/// Minn — the second draw makes an Illusion; an Illusion dying puts a cheap
/// permanent from hand onto the battlefield.
#[test]
fn minn_makes_illusions_and_cheats_on_death() {
    let mut g = pod(2);
    let minn = g.add_card_to_battlefield(0, catalog::minn_wily_illusionist());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    run(&mut g, Effect::Draw { who: Selector::You, amount: Value::Const(2) }, minn);
    let ill = named(&g, 0, "Illusion");
    assert_eq!(ill.len(), 1);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    // A lone Illusion is 1/1: nothing with MV ≤ 1 in hand, so nothing moves.
    run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![ill[0]]) }, minn);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears));
    // Two Illusions pump each other to 2/1: the dying one lets the Bears in.
    let definition = illusion_def(&g, minn);
    run(&mut g, Effect::CreateToken { who: crabomination::effect::PlayerRef::You, count: Value::Const(2), definition }, minn);
    let ill = named(&g, 0, "Illusion");
    assert_eq!(g.computed_permanent(ill[0]).unwrap().power, 2);
    run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![ill[0]]) }, minn);
    assert!(g.battlefield_find(bears).is_some(), "MV 2 fits power 2");
    assert!(g.players[0].hand.iter().any(|c| c.id == giant), "MV 4 doesn't");
}

fn illusion_def(g: &GameState, minn: CardId) -> std::sync::Arc<crabomination::card::TokenDefinition> {
    let def = &g.battlefield_find(minn).unwrap().definition;
    match &def.triggered_abilities[0].effect {
        Effect::CreateToken { definition, .. } => definition.clone(),
        other => panic!("unexpected {other:?}"),
    }
}

/// Murder of Crows — another creature dying offers a loot.
#[test]
fn murder_of_crows_loots_on_deaths() {
    let mut g = pod(2);
    let m = g.add_card_to_battlefield(0, catalog::murder_of_crows());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![bear]) }, m);
    assert_eq!(g.players[0].graveyard.len(), 1, "drew then discarded");
    assert!(g.players[0].library.is_empty());
}

/// Nihiloor — steals a creature no stronger than itself; attacking with it
/// drains its owner.
#[test]
fn nihiloor_steals_and_drains_the_owner() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let n = g.add_card_to_hand(0, catalog::nihiloor());
    cast(&mut g, 0, n, None).expect("nihiloor");
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "stolen");
    assert!(g.battlefield_find(n).unwrap().tapped, "tapped for the steal");
    g.clear_sickness(bear);
    attack(&mut g, &[bear]);
    assert_eq!(g.players[0].life, 22);
    assert_eq!(g.players[1].life, 18);
}

/// Nimbus Maze — each colored ability needs the other basic type.
#[test]
fn nimbus_maze_gates_its_colors() {
    let mut g = pod(2);
    let maze = g.add_card_to_battlefield(0, catalog::nimbus_maze());
    g.priority.player_with_priority = 0;
    let act = |g: &mut GameState, i: usize| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: maze,
            ability_index: i,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(act(&mut g, 1).is_err(), "no Island: no {{W}}");
    g.add_card_to_battlefield(0, catalog::island());
    act(&mut g, 1).expect("{W} with an Island");
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
}

/// Obsessive Stitcher — tap to loot; sacrifice to reanimate.
#[test]
fn obsessive_stitcher_loots_and_reanimates() {
    let mut g = pod(2);
    let os = g.add_card_to_battlefield(0, catalog::obsessive_stitcher());
    g.clear_sickness(os);
    g.add_card_to_library(0, catalog::island());
    activate_x(&mut g, 0, os, 0, None, None).expect("loot");
    assert_eq!(g.players[0].graveyard.len(), 1);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.battlefield_find_mut(os).unwrap().tapped = false;
    activate_x(&mut g, 0, os, 1, Some(Target::Permanent(bear)), None).expect("reanimate");
    assert!(g.battlefield_find(bear).is_some());
    assert!(g.battlefield_find(os).is_none());
}

/// Phantom Steed — holds another creature in exile and brings an attacking
/// copy of it each attack.
#[test]
fn phantom_steed_attacks_with_a_copy() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ps = g.add_card_to_hand(0, catalog::phantom_steed());
    cast(&mut g, 0, ps, Some(Target::Permanent(bear))).expect("steed");
    assert!(g.battlefield_find(bear).is_none(), "exiled");
    g.clear_sickness(ps);
    attack(&mut g, &[ps]);
    let copies = named(&g, 0, "Grizzly Bears");
    assert_eq!(copies.len(), 1, "an attacking copy");
    assert!(g.battlefield_find(copies[0]).unwrap().is_token);
}

/// Revivify — a high roll returns this turn's dead creatures to the
/// battlefield, a low one to hand.
#[test]
fn revivify_returns_this_turns_dead() {
    for (roll, on_field) in [(3u8, false), (19, true)] {
        let mut g = pod(2);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![bear]) }, bear);
        let rv = g.add_card_to_hand(0, catalog::revivify());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(roll)]));
        cast(&mut g, 0, rv, None).expect("revivify");
        assert_eq!(g.battlefield_find(bear).is_some(), on_field, "roll {roll}");
        assert_eq!(g.players[0].hand.iter().any(|c| c.id == bear), !on_field, "roll {roll}");
    }
}

/// Rod of Absorption — resolving spells are exiled with it; sacrificing it
/// casts them up to a total mana value of X.
#[test]
fn rod_of_absorption_exiles_and_recasts() {
    let mut g = pod(2);
    let rod = g.add_card_to_battlefield(0, catalog::rod_of_absorption());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0))).expect("bolt");
    let div = g.add_card_to_hand(0, catalog::divination());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    cast(&mut g, 0, div, None).expect("divination");
    assert!(g.exile.iter().any(|c| c.id == bolt && c.exiled_with == Some(rod)));
    assert!(g.exile.iter().any(|c| c.id == div && c.exiled_with == Some(rod)));
    let hand = g.players[0].hand.len();
    activate_x(&mut g, 0, rod, 0, None, Some(1)).expect("rod");
    assert_eq!(g.players[1].life, 17, "the Bolt was cast for X = 1");
    assert_eq!(g.players[0].hand.len(), hand, "Divination (MV 3) didn't fit");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Rod gone: the Bolt hits a graveyard");
}

/// Thorough Investigation — attacking investigates; cracking the Clue
/// ventures.
#[test]
fn thorough_investigation_clues_and_ventures() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::thorough_investigation());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    attack(&mut g, &[bear]);
    let clue = named(&g, 0, "Clue");
    assert_eq!(clue.len(), 1);
    g.add_card_to_library(0, catalog::island());
    g.step = TurnStep::PostCombatMain;
    activate_x(&mut g, 0, clue[0], 0, None, None).expect("crack");
    assert_eq!(in_dungeon_room(&g, 0), Some(0));
}

/// Vanish into Memory — exile a creature and draw its power; it returns at
/// your next upkeep and you discard its toughness.
#[test]
fn vanish_into_memory_blinks_through_your_upkeep() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let vim = g.add_card_to_hand(0, catalog::vanish_into_memory());
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, vim, Some(Target::Permanent(angel))).expect("vanish");
    assert!(g.battlefield_find(angel).is_none());
    assert_eq!(g.players[0].hand.len(), hand - 1 + 4, "drew its power");
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(angel).unwrap().controller, 1, "back under its owner");
    assert_eq!(g.players[0].hand.len(), hand - 1, "discarded its toughness");
}

/// Wand of Orcus — the equipped attacker gains deathtouch and its combat
/// damage to a player makes that many Zombies.
#[test]
fn wand_of_orcus_makes_zombies() {
    let mut g = pod(2);
    let wand = g.add_card_to_battlefield(0, catalog::wand_of_orcus());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.clear_sickness(giant);
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: wand, target: giant }).expect("equip");
    attack(&mut g, &[giant]);
    assert!(g.computed_permanent(giant).unwrap().keywords().contains(&Keyword::Deathtouch));
    combat_damage(&mut g);
    assert_eq!(named(&g, 0, "Zombie").len(), 3);
}
