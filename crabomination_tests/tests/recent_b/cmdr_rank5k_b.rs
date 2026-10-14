//! Commander: two-card gaps of EDHREC ranks 4501-5500 (`decks::cmdr_rank5k_b`).

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

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x });
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    );
}

fn keywords(g: &GameState, id: CardId) -> Vec<Keyword> {
    g.computed_permanent(id).unwrap().keywords().to_vec()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

/// Cast Away Doubt: draw two, then 2 damage to every player at the table.
#[test]
fn cast_away_doubt_hits_every_player() {
    let mut g = pod(4);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let lives: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    let id = g.add_card_to_hand(0, catalog::cast_away_doubt());
    cast(&mut g, id, None, None);
    assert_eq!(g.players[0].hand.len(), 2);
    for (p, l) in g.players.iter().zip(lives) {
        assert_eq!(p.life, l - 2);
    }
}

/// Machine Man: a noncreature spell grows him and gives him flying.
#[test]
fn machine_man_grows_on_noncreature_spells() {
    let mut g = pod(2);
    let mm = ready(&mut g, 0, catalog::machine_man_model_x_51());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None, None);
    assert_eq!(pt(&g, mm), (3, 4));
    assert!(keywords(&g, mm).contains(&Keyword::Flying));
}

/// The Echoverse Fulcrum's sorcery-speed exile ability wipes the board.
#[test]
fn echoverse_fulcrum_destroys_all_creatures() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::the_echoverse_fulcrum());
    cast(&mut g, id, None, None);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.clear_sickness(id);
    activate(&mut g, id, 0, None);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(id).is_none() && g.exile.iter().any(|c| c.id == id));
}

/// Okoye: only attacking *tokens* get first strike.
#[test]
fn okoye_gives_attacking_tokens_first_strike() {
    let mut g = pod(2);
    let id = g.add_card_to_hand(0, catalog::okoye_dora_milaje_leader());
    cast(&mut g, id, None, None);
    let soldiers: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.definition.name == "Soldier").map(|c| c.id).collect();
    assert_eq!(soldiers.len(), 2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.clear_sickness(soldiers[0]);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: soldiers[0], target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    assert!(keywords(&g, soldiers[0]).contains(&Keyword::FirstStrike));
    assert!(!keywords(&g, soldiers[1]).contains(&Keyword::FirstStrike), "not attacking");
    assert!(!keywords(&g, bear).contains(&Keyword::FirstStrike), "not a token");
}

/// Chase Stein: tap and discard to impulse the top card.
#[test]
fn chase_stein_impulses_for_a_discard() {
    let mut g = pod(2);
    let chase = ready(&mut g, 0, catalog::chase_stein_runaway());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_library(0, catalog::grizzly_bears());
    activate(&mut g, chase, 0, None);
    assert!(g.players[0].hand.is_empty());
    assert!(g.exile.iter().any(|c| c.definition.name == "Grizzly Bears" && c.may_play_until.is_some()));
}

/// Extract the Truth's first mode takes a creature card, never a land.
#[test]
fn extract_the_truth_takes_a_creature_card() {
    let mut g = pod(2);
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::extract_the_truth());
    act(
        &mut g,
        GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: Some(0),
            x_value: None,
        },
    );
    assert_eq!(g.players[1].hand.len(), 1);
    assert_eq!(g.players[1].hand[0].definition.name, "Island");
}

/// Profaner of the Dead: exploiting a 3-toughness creature bounces the
/// opponents' creatures with toughness 2 or less and leaves the rest.
#[test]
fn profaner_of_the_dead_bounces_below_the_exploited_toughness() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::profaner_of_the_dead());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // accept exploit
    cast(&mut g, id, None, None);
    let mine = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(mine, 0, "Profaner exploited itself (toughness 3)");
    assert!(g.battlefield_find(bear).is_none(), "toughness 2 < 3");
    assert!(g.battlefield_find(angel).is_some(), "toughness 4");
}

/// Hulkling: a noncreature spell turns him into the other creature until end
/// of turn, but he keeps his name, a 4/4 body and flying.
#[test]
fn hulkling_copies_but_keeps_his_name_and_body() {
    let mut g = pod(2);
    let hulk = ready(&mut g, 0, catalog::hulkling_young_avenger());
    ready(&mut g, 1, catalog::serra_angel());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None, None);
    let c = g.computed_permanent(hulk).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4));
    assert!(c.keywords().contains(&Keyword::Vigilance), "Serra Angel's vigilance");
    assert!(c.keywords().contains(&Keyword::Flying));
    assert_eq!(g.battlefield_find(hulk).unwrap().definition.name, "Hulkling, Young Avenger");
}

/// Night Nurse returns a permanent card put into the graveyard this turn.
#[test]
fn night_nurse_returns_a_card_that_died_this_turn() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let wrath = g.add_card_to_hand(0, catalog::wrath_of_god());
    cast(&mut g, wrath, None, None);
    let nurse = g.add_card_to_hand(0, catalog::night_nurse_healer_of_heroes());
    cast(&mut g, nurse, None, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Captain America, Skybound: battalion counters and indestructible on every attacker.
#[test]
fn captain_america_skybound_battalion() {
    let mut g = pod(2);
    let cap = ready(&mut g, 0, catalog::captain_america_skybound());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        [cap, a, b].into_iter().map(|attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(&mut g);
    for id in [cap, a, b] {
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert!(keywords(&g, id).contains(&Keyword::Indestructible));
    }
}

/// CR 704.5j — Spider-Verse turns the legend rule off for Spiders only.
#[test]
fn spider_verse_exempts_spiders_from_the_legend_rule() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::spider_verse());
    for _ in 0..2 {
        let id = g.add_card_to_hand(0, catalog::spinneret_and_spiderling());
        cast(&mut g, id, None, None);
    }
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Spinneret and Spiderling").count(), 2);
    for _ in 0..2 {
        let id = g.add_card_to_hand(0, catalog::okoye_dora_milaje_leader());
        cast(&mut g, id, None, None);
    }
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Okoye, Dora Milaje Leader").count(), 1);
}

/// Wingnut's alliance grants one of its three keywords when another creature enters.
#[test]
fn wingnut_alliance_grants_a_keyword() {
    let mut g = pod(2);
    let wing = ready(&mut g, 0, catalog::wingnut_bat_on_the_belfry());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None, None);
    let k = keywords(&g, wing);
    assert!([Keyword::Flying, Keyword::Menace, Keyword::Haste].iter().any(|x| k.contains(x)));
}
