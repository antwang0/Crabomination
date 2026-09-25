//! Commander: the Deadly Disguise precon (MKC, Kaust, Eyes of the Glade,
//! `decks::cmdr_kaust`).

use crabomination::card::{CardDefinition, CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
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

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
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

/// Cast `def` face down for seat 0 and return its id on the battlefield.
fn face_down(g: &mut GameState, def: CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("face down");
    drain_stack(g);
    assert!(g.battlefield_find(id).is_some_and(|c| c.face_down));
    id
}

fn turn_up(g: &mut GameState, id: CardId) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("face up");
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
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

/// Kaust — tapping turns a face-down attacker up, and that creature's combat
/// damage to a player draws.
#[test]
fn kaust_flips_an_attacker_and_draws_on_its_hit() {
    let mut g = pod(2);
    let k = g.add_card_to_battlefield(0, catalog::kaust_eyes_of_the_glade());
    g.clear_sickness(k);
    let m = face_down(&mut g, catalog::master_of_pearls());
    g.clear_sickness(m);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    attack(&mut g, &[m]);
    activate(&mut g, 0, k, 0, Some(Target::Permanent(m))).expect("kaust");
    assert!(!g.battlefield_find(m).unwrap().face_down, "turned face up for free");
    assert_eq!(pt(&g, m), (4, 4), "Master of Pearls' own +2/+2");
    let hand = g.players[0].hand.len();
    combat_damage(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew off its hit");
}

/// Ashcloud Phoenix — it comes back face down when it dies; turning it up
/// deals 2 to each player.
#[test]
fn ashcloud_phoenix_returns_face_down() {
    let mut g = pod(2);
    let ph = g.add_card_to_battlefield(0, catalog::ashcloud_phoenix());
    run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![ph]) }, ph);
    let c = g.battlefield_find(ph).expect("back on the battlefield");
    assert!(c.face_down);
    assert_eq!(pt(&g, ph), (2, 2));
    turn_up(&mut g, ph);
    assert_eq!(g.players[0].life, 18);
    assert_eq!(g.players[1].life, 18);
}

/// Duskana — draws per base-2/2 creature (face-down ones count) and pumps a
/// base-2/2 attacker.
#[test]
fn duskana_rewards_two_twos() {
    let mut g = pod(2);
    let fd = face_down(&mut g, catalog::master_of_pearls());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let d = g.add_card_to_hand(0, catalog::duskana_the_rage_mother());
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, d, None).expect("duskana");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "two base-2/2 creatures");
    g.clear_sickness(fd);
    attack(&mut g, &[fd]);
    assert_eq!(pt(&g, fd), (5, 5));
}

/// Hidden Dragonslayer — megamorph up with a counter and destroy a big
/// opposing creature.
#[test]
fn hidden_dragonslayer_kills_a_big_creature() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hd = face_down(&mut g, catalog::hidden_dragonslayer());
    turn_up(&mut g, hd);
    assert!(g.battlefield_find(angel).is_none(), "power 4 dies");
    assert!(g.battlefield_find(bears).is_some());
    assert_eq!(pt(&g, hd), (3, 2), "megamorph's counter");
}

/// Hooded Hydra — turned up with five counters (not a dying 0/0), and its
/// death makes a Snake per counter.
#[test]
fn hooded_hydra_turns_up_with_five_counters() {
    let mut g = pod(2);
    let hh = face_down(&mut g, catalog::hooded_hydra());
    turn_up(&mut g, hh);
    assert_eq!(pt(&g, hh), (5, 5));
    run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![hh]) }, hh);
    assert_eq!(named(&g, 0, "Snake").len(), 5);
}

/// Mastery of the Unseen — manifests for {3}{W}; turning a permanent up
/// gains life per creature.
#[test]
fn mastery_of_the_unseen_manifests_and_gains() {
    let mut g = pod(2);
    let mu = g.add_card_to_battlefield(0, catalog::mastery_of_the_unseen());
    g.add_card_to_library(0, catalog::grizzly_bears());
    activate(&mut g, 0, mu, 0, None).expect("manifest");
    let man = g.battlefield.iter().find(|c| c.controller == 0 && c.face_down).map(|c| c.id).expect("manifested");
    turn_up(&mut g, man);
    assert_eq!(g.players[0].life, 21, "one creature");
}

/// Neheb — afflict 3, and the second main phase adds {R} per life your
/// opponents lost.
#[test]
fn neheb_adds_red_for_life_lost() {
    let mut g = pod(2);
    let n = g.add_card_to_battlefield(0, catalog::neheb_the_eternal());
    run(&mut g, Effect::LoseLife { who: Selector::Player(crabomination::effect::PlayerRef::EachOpponent), amount: crabomination::effect::Value::Const(3) }, n);
    g.players[0].mana_pool = Default::default();
    g.step = TurnStep::PostCombatMain;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3);
}

/// Obscuring Aether and Panoptic Projektor — face-down spells get cheaper
/// (the Projektor only for the next one), and the Projektor doubles
/// turned-face-up triggers.
#[test]
fn face_down_discounts_and_projektor_doubling() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::obscuring_aether());
    assert_eq!(g.face_down_cast_cost(0), 2);
    let pp = g.add_card_to_battlefield(0, catalog::panoptic_projektor());
    activate(&mut g, 0, pp, 0, None).expect("projektor");
    assert_eq!(g.face_down_cast_cost(0), 0);
    let m = face_down(&mut g, catalog::master_of_pearls());
    assert_eq!(g.face_down_cast_cost(0), 2, "the Projektor's discount is spent");
    turn_up(&mut g, m);
    assert_eq!(pt(&g, m), (6, 6), "Master of Pearls triggered twice");
}

/// Obscuring Aether — it can turn itself face down into a 2/2.
#[test]
fn obscuring_aether_hides_itself() {
    let mut g = pod(2);
    let oa = g.add_card_to_battlefield(0, catalog::obscuring_aether());
    activate(&mut g, 0, oa, 0, None).expect("face down");
    assert!(g.battlefield_find(oa).unwrap().face_down);
    assert_eq!(pt(&g, oa), (2, 2));
}

/// Printlifter Ooze — a creature of yours turning up makes a trampling Ooze
/// with a counter per other creature.
#[test]
fn printlifter_ooze_makes_oozes() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::printlifter_ooze());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let m = face_down(&mut g, catalog::master_of_pearls());
    turn_up(&mut g, m);
    let ooze: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Ooze").map(|c| c.id).collect();
    assert_eq!(ooze.len(), 1);
    assert_eq!(g.battlefield_find(ooze[0]).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Salt Road Ambushers — another creature turning up gets two counters.
#[test]
fn salt_road_ambushers_grows_the_revealed() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::salt_road_ambushers());
    let hd = face_down(&mut g, catalog::hidden_dragonslayer());
    turn_up(&mut g, hd);
    assert_eq!(g.battlefield_find(hd).unwrap().counter_count(CounterType::PlusOnePlusOne), 3, "megamorph + two");
}

/// Showstopping Surprise — turns a creature up, then it deals its power to
/// each other creature.
#[test]
fn showstopping_surprise_flips_and_sweeps() {
    let mut g = pod(2);
    let fd = face_down(&mut g, catalog::ashcloud_phoenix());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ss = g.add_card_to_hand(0, catalog::showstopping_surprise());
    cast(&mut g, 0, ss, Some(Target::Permanent(fd))).expect("surprise");
    assert!(!g.battlefield_find(fd).unwrap().face_down);
    assert!(g.battlefield_find(bears).is_none(), "4 damage from the Phoenix");
}

/// Tesak — creatures with counters have haste; attacking adds {R} per
/// attacker.
#[test]
fn tesak_hastes_and_adds_mana() {
    let mut g = pod(2);
    let t = g.add_card_to_battlefield(0, catalog::tesak_judiths_hellhound());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Haste));
    g.clear_sickness(t);
    g.players[0].mana_pool = Default::default();
    attack(&mut g, &[t, bears]);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2);
}

/// True Identity — turning it (disguised) up scries and draws, once a turn.
#[test]
fn true_identity_draws_once_a_turn() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let ti = face_down(&mut g, catalog::true_identity());
    let m = face_down(&mut g, catalog::master_of_pearls());
    let hand = g.players[0].hand.len();
    turn_up(&mut g, ti);
    assert!(g.battlefield_find(ti).unwrap().definition.is_enchantment());
    turn_up(&mut g, m);
    assert_eq!(g.players[0].hand.len(), hand + 1, "only once each turn");
}

/// Unexplained Absence — an opponent's permanent is exiled and they cloak.
#[test]
fn unexplained_absence_exiles_and_cloaks() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_library(1, catalog::island());
    let ua = g.add_card_to_hand(0, catalog::unexplained_absence());
    cast(&mut g, 0, ua, Some(Target::Permanent(angel))).expect("absence");
    assert!(g.battlefield_find(angel).is_none());
    assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.face_down), "they cloaked");
}

/// Veiled Ascension — face-down creatures get flying counters, and the
/// upkeep cloaks.
#[test]
fn veiled_ascension_gives_face_downs_wings() {
    let mut g = pod(2);
    let before = face_down(&mut g, catalog::master_of_pearls());
    let va = g.add_card_to_battlefield(0, catalog::veiled_ascension());
    g.fire_self_etb_triggers(va, 0);
    drain_stack(&mut g);
    assert!(g.computed_permanent(before).unwrap().keywords().contains(&Keyword::Flying));
    let after = face_down(&mut g, catalog::hidden_dragonslayer());
    assert!(g.computed_permanent(after).unwrap().keywords().contains(&Keyword::Flying));
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).count(), 3);
}

/// Boltbender — turning it up re-aims a spell on the stack.
#[test]
fn boltbender_redirects_a_spell() {
    let mut g = pod(3);
    let bb = face_down(&mut g, catalog::boltbender());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    turn_up(&mut g, bb);
    assert_eq!(g.players[0].life, 20, "the Bolt went elsewhere");
}
