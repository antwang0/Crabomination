//! Commander: the Graveyard Overdrive precon (M3C, Disa, `decks::cmdr_disa`).

use crabomination::card::{CardId, CounterType, CreatureType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
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

fn cast_action(id: CardId, target: Option<Target>, mode: Option<usize>, x: Option<u32>) -> GameAction {
    GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: x }
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g, 0);
    g.perform_action(cast_action(id, target, None, None)).expect("cast");
    drain_stack(g);
}

fn cast_mode(g: &mut GameState, id: CardId, target: Option<Target>, mode: usize) {
    flood(g, 0);
    g.perform_action(cast_action(id, target, Some(mode), None)).expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))
}

fn attack(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    for (a, _) in &attacks {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn connect(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    attack(g, attacks);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

// ── Lhurgoyfs ──────────────────────────────────────────────────────────────

/// A Lhurgoyf card discarded to your graveyard returns; one that dies
/// stays. Connecting makes a Tarmogoyf (CR 604.3 — its P/T track the
/// graveyards).
#[test]
fn disa_returns_lhurgoyfs_and_makes_tarmogoyfs() {
    let mut g = main_phase();
    let disa = g.add_card_to_battlefield(0, catalog::disa_the_restless());
    let goyf = g.add_card_to_hand(0, catalog::pyrogoyf());
    let dies = g.add_card_to_battlefield(0, catalog::barrowgoyf());
    for c in [catalog::lightning_bolt(), catalog::grizzly_bears()] {
        g.add_card_to_graveyard(1, c);
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(dies)));
    assert!(g.battlefield_find(dies).is_none(), "died from the battlefield: stays");
    // Milled from the library (Thought Scour on yourself): it comes back.
    let hand_goyf = goyf;
    g.players[0].hand.retain(|c| c.id != hand_goyf);
    let milled = g.add_card_to_library(0, catalog::pyrogoyf());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let scour = g.add_card_to_hand(0, catalog::thought_scour());
    cast(&mut g, scour, Some(Target::Player(0)));
    assert!(g.battlefield_find(milled).is_some(), "milled Lhurgoyf came back");
    connect(&mut g, vec![(disa, 1)]);
    let tarmo = named(&g, 0, "Tarmogoyf");
    assert_eq!(tarmo.len(), 1);
    // Graveyards hold an instant, a creature and a sorcery — three types.
    assert_eq!(pt(&g, tarmo[0]), (3, 4));
}

/// Connecting may mill that many and take a creature card.
#[test]
fn barrowgoyf_mills_and_keeps_a_creature() {
    let mut g = main_phase();
    let goyf = g.add_card_to_battlefield(0, catalog::barrowgoyf());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Cards(vec![bears]),
    ]));
    connect(&mut g, vec![(goyf, 1)]);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears), "the milled bears came to hand");
}

/// Another Lhurgoyf entering deals its power to any target.
#[test]
fn pyrogoyf_throws_each_lhurgoyf() {
    let mut g = main_phase();
    // A bot seat aims damage at opponents (the pods' default profile).
    g.players[0].hostile_player_targets = true;
    g.add_card_to_battlefield(0, catalog::pyrogoyf());
    for c in [catalog::lightning_bolt(), catalog::grizzly_bears()] {
        g.add_card_to_graveyard(1, c);
    }
    let other = g.add_card_to_hand(0, catalog::polygoyf());
    flood(&mut g, 0);
    g.perform_action(cast_action(other, None, None, None)).expect("cast");
    // resolve the creature spell only
    let _ = g.perform_action(GameAction::PassPriority);
    let _ = g.perform_action(GameAction::PassPriority);
    drain_stack(&mut g);
    // Instant, creature, and now nothing new: two types → Polygoyf is 2/3.
    assert_eq!(g.players[1].life, 18);
}

/// The enchanted land makes Tarmogoyf tokens.
#[test]
fn tarmogoyf_nest_breeds_goyfs() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let nest = g.add_card_to_hand(0, catalog::tarmogoyf_nest());
    cast(&mut g, nest, Some(Target::Permanent(land)));
    flood(&mut g, 0);
    activate(&mut g, land, 1, None).expect("the granted ability");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Tarmogoyf").len(), 1);
}

/// Myriad (CR 702.116): a copy attacks each other opponent.
#[test]
fn polygoyf_attacks_everyone() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    let goyf = g.add_card_to_battlefield(0, catalog::polygoyf());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, vec![(goyf, 1)]);
    assert_eq!(g.attacking.len(), 2, "a token copy attacks the other opponent");
}

// ── Commanders and creatures ───────────────────────────────────────────────

/// +X/+0 for the biggest creature card in any graveyard; attacking mills
/// everyone; a milled spell from ANY graveyard can be cast, once a turn.
#[test]
fn coram_plays_from_whatever_was_milled() {
    let mut g = main_phase();
    let coram = g.add_card_to_battlefield(0, catalog::coram_the_undertaker());
    g.add_card_to_graveyard(1, catalog::hill_giant());
    assert_eq!(pt(&g, coram), (3, 5));
    let bolt = g.add_card_to_library(1, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::divination());
    attack(&mut g, vec![(coram, 1)]);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "each player milled one");
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    flood(&mut g, 0);
    g.perform_action(cast_action(bolt, Some(Target::Player(1)), None, None)).expect("the opponent's milled Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "back to its owner's graveyard");
    let mine = g.players[0].graveyard.iter().find(|c| c.definition.name == "Divination").unwrap().id;
    flood(&mut g, 0);
    assert!(g.perform_action(cast_action(mine, None, None, None)).is_err(), "one spell a turn");
}

/// An instant aimed at one opponent's nonland permanent is copied onto a
/// different opponent's (CR 707.10c — the copy is yours).
#[test]
fn exterminator_magmarch_spreads_removal() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::exterminator_magmarch());
    let first = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let second = g.add_card_to_battlefield(2, catalog::hill_giant());
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(first)));
    assert!(g.battlefield_find(first).is_none() && g.battlefield_find(second).is_none());
}

/// Cast, each player sacrifices X creatures; two counters per sacrifice.
#[test]
fn gluttonous_hellkite_eats_the_table() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let kite = g.add_card_to_hand(0, catalog::gluttonous_hellkite());
    flood(&mut g, 0);
    g.perform_action(cast_action(kite, None, None, Some(1))).expect("X = 1");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(kite).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 0);
}

/// This turn, a nontoken creature of yours dying leaves Saprolings equal to
/// its power (read from its last known information).
#[test]
fn infested_thrinax_seeds_the_dead() {
    let mut g = main_phase();
    let thrinax = g.add_card_to_hand(0, catalog::infested_thrinax());
    cast(&mut g, thrinax, None);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(giant)));
    assert_eq!(named(&g, 0, "Saproling").len(), 3);
}

/// An Insect per creature card in your graveyard; sacrifice another for a
/// card and a life.
#[test]
fn izoni_hatches_and_feeds() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.add_card_to_library(0, catalog::forest());
    let izoni = g.add_card_to_hand(0, catalog::izoni_thousand_eyed());
    cast(&mut g, izoni, None);
    assert_eq!(named(&g, 0, "Insect").len(), 3);
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    activate(&mut g, izoni, 0, None).expect("sacrifice an Insect");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Insect").len(), 2);
    assert_eq!((g.players[0].hand.len(), g.players[0].life), (hand + 1, 21));
}

/// Choose a player: damage to them and their permanents is doubled (CR
/// 614.5); anyone else takes normal damage.
#[test]
fn sawhorn_nemesis_doubles_on_its_mark() {
    let mut g = main_phase();
    let saw = g.add_card_to_hand(0, catalog::sawhorn_nemesis());
    cast(&mut g, saw, None);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 14);
}

/// At your end step, sacrifice another creature: its power to any target and
/// three Treasures.
#[test]
fn ziatora_flings_and_pays() {
    let mut g = main_phase();
    g.players[0].hostile_player_targets = true;
    g.add_card_to_battlefield(0, catalog::ziatora_the_incinerator());
    g.add_card_to_battlefield(0, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Treasure").len(), 3);
    assert_eq!(g.players[1].life, 17);
}

/// A 5/5 Dragon comes along.
#[test]
fn broodmate_tyrant_brings_a_mate() {
    let mut g = main_phase();
    let tyrant = g.add_card_to_hand(0, catalog::broodmate_tyrant());
    cast(&mut g, tyrant, None);
    assert_eq!(named(&g, 0, "Dragon").len(), 1);
}

// ── Spells and a walker ────────────────────────────────────────────────────

/// Find returns up to two creature cards; Finality counters one of yours and
/// shrinks everything by 4.
#[test]
fn find_finality_both_halves() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let card = g.add_card_to_hand(0, catalog::find_finality());
    cast(&mut g, card, None);
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 2);
    let giant = g.add_card_to_battlefield(0, catalog::serra_angel());
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant());
    let card = g.add_card_to_hand(0, catalog::find_finality());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSplitRight {
        card_id: card,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Finality");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "-4/-4 kills the Hill Giant");
    assert_eq!(pt(&g, giant), (2, 2), "Serra Angel 4/4 +2/+2 -4/-4");
}

/// +1 Zombie and mill two; -3 returns a creature as a black Zombie.
#[test]
fn liliana_deaths_majesty_raises_zombies() {
    let mut g = main_phase();
    let lili = g.add_card_to_battlefield(0, catalog::liliana_deaths_majesty());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: lili,
        ability_index: 0,
        target: None,
        x_value: None,
    })
    .expect("+1");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Zombie").len(), 1);
    assert_eq!(g.players[0].graveyard.len(), 2);
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.battlefield_find_mut(lili).unwrap().loyalty_uses_this_turn = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: lili,
        ability_index: 1,
        target: Some(Target::Permanent(angel)),
        x_value: None,
    })
    .expect("-3");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).expect("returned");
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Zombie));
    assert!(cp.colors.contains(Color::Black) && cp.colors.contains(Color::White));
}

/// Mode one edicts the biggest; mode two's impulse lasts until the end step
/// begins; mode three exiles a graveyard.
#[test]
fn riveteers_charm_modes() {
    let mut g = main_phase();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::serra_angel());
    let charm = g.add_card_to_hand(0, catalog::riveteers_charm());
    cast_mode(&mut g, charm, Some(Target::Player(1)), 0);
    assert!(g.battlefield_find(big).is_none() && g.battlefield_find(small).is_some());
    let top: Vec<CardId> = (0..3).map(|_| g.add_card_to_library(0, catalog::lightning_bolt())).collect();
    let charm = g.add_card_to_hand(0, catalog::riveteers_charm());
    cast_mode(&mut g, charm, None, 1);
    assert!(top.iter().all(|id| g.exile.iter().any(|c| c.id == *id && c.may_play_until.is_some())));
    g.step = TurnStep::PostCombatMain;
    let _ = g.advance_step(Vec::new());
    assert_eq!(g.step, TurnStep::End);
    assert!(top.iter().all(|id| g.exile.iter().any(|c| c.id == *id && c.may_play_until.is_none())), "gone as the end step begins");
    g.add_card_to_graveyard(1, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    let charm = g.add_card_to_hand(0, catalog::riveteers_charm());
    cast_mode(&mut g, charm, Some(Target::Player(1)), 2);
    assert!(g.players[1].graveyard.is_empty());
}

/// Tempting offer (CR 207.2c): you copy the spell; an opponent who declines
/// adds nothing.
#[test]
fn tempt_with_mayhem_copies_a_spell() {
    let mut g = main_phase();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    g.perform_action(cast_action(bolt, Some(Target::Player(1)), None, None)).expect("bolt");
    let tempt = g.add_card_to_hand(0, catalog::tempt_with_mayhem());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    flood(&mut g, 0);
    g.perform_action(cast_action(tempt, Some(Target::Permanent(bolt)), None, None)).expect("tempt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "the Bolt and one copy");
}
