//! Commander: EDHREC ranks 1501-2000, one-card gaps, second batch
//! (`decks::cmdr_rank2k_b`).

use crabomination::card::{CardId, CounterType, Keyword};
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

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, more: Vec<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: more, mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g);
    cast_with(g, id, target, vec![]);
}

fn activate(g: &mut GameState, id: CardId, ability_index: usize) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn attack(g: &mut GameState, attackers: &[CardId], defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

/// Animist's Might costs {2} less aimed from a legendary creature, and deals
/// twice that creature's power.
#[test]
fn animists_might_discounts_for_a_legend_and_doubles_power() {
    let mut g = pod(2);
    let legend = ready(&mut g, 0, catalog::radha_heart_of_keld());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let might = g.add_card_to_hand(0, catalog::animists_might());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_with(&mut g, might, Some(Target::Permanent(legend)), vec![Target::Permanent(angel)]);
    assert!(g.battlefield_find(angel).is_none(), "six damage to the Angel");
}

/// Paladin Class: opponents pay {1} more on your turn; level 2 is an anthem;
/// level 3 gives an attacker +1/+1 per other attacker and double strike.
#[test]
fn paladin_class_levels_up() {
    let mut g = pod(2);
    let class = g.add_card_to_hand(0, catalog::paladin_class());
    cast(&mut g, class, None);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(a).unwrap().power, 2);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let taxed = g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(taxed.is_err(), "{{R}} doesn't cover the {{1}} tax on your turn");
    activate(&mut g, class, 0);
    assert_eq!(g.computed_permanent(a).unwrap().power, 3, "level 2 anthem");
    activate(&mut g, class, 1);
    attack(&mut g, &[a, b], 1);
    let pumped = [a, b].into_iter().filter(|&id| {
        let cp = g.computed_permanent(id).unwrap();
        cp.power == 4 && cp.keywords().contains(&Keyword::DoubleStrike)
    });
    assert_eq!(pumped.count(), 1, "one attacker gets +1/+1 and double strike");
}

/// Path of Bravery pumps only at or above starting life, and gains a life per
/// attacker.
#[test]
fn path_of_bravery_reads_starting_life() {
    let mut g = pod(2);
    let start = g.players[0].life;
    ready(&mut g, 0, catalog::path_of_bravery());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    g.players[0].life = start - 1;
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "below starting life");
    attack(&mut g, &[bear], 1);
    assert_eq!(g.players[0].life, start, "one attacker, one life");
}

/// Lake of the Dead enters only by sacrificing a Swamp, and its big ability
/// eats another for four black.
#[test]
fn lake_of_the_dead_costs_swamps() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::swamp());
    let lake = g.add_card_to_hand(0, catalog::lake_of_the_dead());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(lake)).expect("play");
    drain_stack(&mut g);
    let swamps = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Swamp").count();
    assert_eq!(swamps(&g), 1, "one Swamp paid its way in");
    assert!(g.battlefield_find(lake).is_some());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: lake,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sac a Swamp");
    assert_eq!(swamps(&g), 0);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 4);
}

/// Nazgûl Battle-Mace: the equipped attacker's annihilator makes the defender
/// sacrifice, and the sacrificed nontoken permanent comes over (the headless
/// opponent declines to pay 3 life).
#[test]
fn nazgul_battle_mace_steals_the_annihilated_permanent() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let mace = ready(&mut g, 0, catalog::nazgul_battle_mace());
    flood(&mut g);
    g.perform_action(GameAction::Equip { equipment: mace, target: bear }).expect("equip");
    drain_stack(&mut g);
    let land = g.add_card_to_battlefield(1, catalog::forest());
    attack(&mut g, &[bear], 1);
    let stolen = g.battlefield_find(land).expect("the Forest came back");
    assert_eq!(stolen.controller, 0);
}

/// Karn, Gilded Guardian counts colors among your other artifacts.
#[test]
fn karn_gilded_guardian_draws_per_artifact_color() {
    let mut g = pod(2);
    library(&mut g, 0, 5);
    ready(&mut g, 0, catalog::sol_ring());
    ready(&mut g, 0, catalog::jhoiras_timebug());
    ready(&mut g, 0, catalog::etherium_sculptor());
    let karn = g.add_card_to_hand(0, catalog::karn_gilded_guardian());
    let hand = g.players[0].hand.len();
    cast(&mut g, karn, None);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "the Sculptor is blue; the others are colorless");
}

/// Headliner Scarlett stops the targeted player's creatures blocking, not
/// another opponent's (the auto-picker names an opponent).
#[test]
fn headliner_scarlett_names_one_player() {
    let mut g = pod(3);
    let theirs = ready(&mut g, 2, catalog::grizzly_bears());
    let other = ready(&mut g, 1, catalog::grizzly_bears());
    let scarlett = g.add_card_to_hand(0, catalog::headliner_scarlett());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    cast(&mut g, scarlett, None);
    let blocks = |g: &GameState, id| g.computed_permanent(id).unwrap().keywords().contains(&Keyword::CantBlock);
    assert!(blocks(&g, theirs) != blocks(&g, other), "exactly one opponent's creatures");
    assert!(!blocks(&g, mine));
}

/// Irencrag Pyromancer fires on the second draw of the turn only.
#[test]
fn irencrag_pyromancer_burns_on_the_second_draw() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    ready(&mut g, 0, catalog::irencrag_pyromancer());
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    let cards = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, cards, None);
    assert_eq!(g.players[1].life, life - 3);
}

/// Jedit Ojanen, Mercenary pays {G} for a Cat Warrior when it or another
/// legend enters.
#[test]
fn jedit_ojanen_mercenary_makes_cats_for_legends() {
    let mut g = pod(2);
    let jedit = g.add_card_to_hand(0, catalog::jedit_ojanen_mercenary());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    cast(&mut g, jedit, None);
    let radha = g.add_card_to_hand(0, catalog::radha_heart_of_keld());
    cast(&mut g, radha, None);
    let cats = g.battlefield.iter().filter(|c| c.definition.name == "Cat Warrior").count();
    assert_eq!(cats, 2);
}

/// Misty Knight draws one per card discarded this turn, its own cost included.
#[test]
fn misty_knight_counts_this_turns_discards() {
    let mut g = pod(2);
    library(&mut g, 0, 5);
    let knight = ready(&mut g, 0, catalog::misty_knight_hero_for_hire());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let tolarian = g.add_card_to_hand(0, catalog::tolarian_winds());
    cast(&mut g, tolarian, None);
    let hand = g.players[0].hand.len();
    activate(&mut g, knight, 0);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3, "two from the Winds, one from the cost");
}

/// Tales of Master Seshiro counters a creature twice, then flips into a 5/5.
#[test]
fn tales_of_master_seshiro_grows_then_transforms() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let tales = g.add_card_to_hand(0, catalog::tales_of_master_seshiro());
    cast(&mut g, tales, None);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    for _ in 0..2 {
        g.saga_advance(tales);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    let cp = g.computed_permanent(tales).expect("returned transformed");
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// CR 603.10a — "unless that player pays 3 life" asks the sacrificer: a
/// stolen Forest the defender sacrifices lands in its owner's graveyard, but
/// the defender pays (`Triggerer` asked the owner).
#[test]
fn nazgul_battle_mace_asks_the_sacrificer_not_the_owner() {
    let mut g = pod(3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let mace = ready(&mut g, 0, catalog::nazgul_battle_mace());
    flood(&mut g);
    g.perform_action(GameAction::Equip { equipment: mace, target: bear }).expect("equip");
    drain_stack(&mut g);
    let land = g.add_card_to_battlefield(2, catalog::forest());
    g.battlefield_find_mut(land).unwrap().controller = 1;
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, &[bear], 1);
    assert_eq!((g.players[1].life, g.players[2].life), (l1 - 3, l2), "the defender paid");
    assert!(g.players[2].graveyard.iter().any(|c| c.id == land), "the Forest stayed in its owner's graveyard");
}
