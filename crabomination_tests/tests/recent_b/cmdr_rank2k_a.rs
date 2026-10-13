//! Commander: EDHREC ranks 1501-2000, one-card gaps, first batch
//! (`decks::cmdr_rank2k_a`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
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

fn cast_at(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, ability_index: usize, target: Option<Target>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index,
        target,
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

/// Merrow Bonegnawer's "target player exiles a card from their graveyard"
/// reaches the targeted opponent at a four-seat table, not seat 1.
#[test]
fn merrow_bonegnawer_targets_a_graveyard_by_player() {
    let mut g = pod(4);
    let gnawer = ready(&mut g, 0, catalog::merrow_bonegnawer());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(2, catalog::grizzly_bears());
    activate(&mut g, gnawer, 0, Some(Target::Player(2)));
    assert_eq!(g.players[2].graveyard.len(), 0, "seat 2 exiled theirs");
    assert_eq!(g.players[1].graveyard.len(), 1, "seat 1 untouched");
}

/// Pestilence Rats counts the other Rats on the whole battlefield.
#[test]
fn pestilence_rats_counts_every_other_rat() {
    let mut g = pod(3);
    let rats = ready(&mut g, 0, catalog::pestilence_rats());
    ready(&mut g, 1, catalog::pestilence_rats());
    ready(&mut g, 2, catalog::pestilence_rats());
    let cp = g.computed_permanent(rats).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3));
}

/// Skanos Dragonheart's attack pump reads the larger of a Dragon you control
/// and a Dragon card in your graveyard.
#[test]
fn skanos_reads_dragons_on_the_battlefield_and_in_the_graveyard() {
    let mut g = pod(2);
    let skanos = ready(&mut g, 0, catalog::skanos_dragonheart());
    ready(&mut g, 0, catalog::adult_gold_dragon());
    g.add_card_to_graveyard(0, catalog::wrathful_red_dragon());
    attack(&mut g, &[skanos], 1);
    assert_eq!(g.computed_permanent(skanos).unwrap().power, 4 + 5, "the 5-power graveyard Dragon wins");
}

/// Yorvo gets one counter for a green creature, and a second when that
/// creature is bigger than Yorvo.
#[test]
fn yorvo_grows_twice_for_a_bigger_green_creature() {
    let mut g = pod(2);
    let yorvo = g.add_card_to_hand(0, catalog::yorvo_lord_of_garenbrig());
    cast_at(&mut g, yorvo, None);
    let counters = |g: &GameState| g.battlefield_find(yorvo).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters(&g), 4);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, bears, None);
    assert_eq!(counters(&g), 5, "a 2/2 is not bigger");
    let ghalta = g.add_card_to_hand(0, catalog::ghalta_primal_hunger());
    cast_at(&mut g, ghalta, None);
    assert_eq!(counters(&g), 7, "a 12-power creature is");
}

/// Cavalier of Night: the reflexive destroy needs the sacrifice; dying
/// returns a small creature card.
#[test]
fn cavalier_of_night_sacrifices_destroys_and_returns() {
    let mut g = pod(2);
    let fodder = ready(&mut g, 0, catalog::grizzly_bears());
    let victim = ready(&mut g, 1, catalog::serra_angel());
    let cav = g.add_card_to_hand(0, catalog::cavalier_of_night());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    cast_at(&mut g, cav, None);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed");
    assert!(g.battlefield_find(victim).is_none(), "destroyed");
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast_at(&mut g, murder, Some(Target::Permanent(cav)));
    assert!(g.battlefield_find(fodder).is_some(), "the Bears came back");
}

/// Transmogrant's Crown draws when the equipped creature dies; Hawkeye's Bow
/// pings each opponent when the equipped creature becomes tapped.
#[test]
fn transmogrants_crown_and_hawkeyes_bow_ride_the_host() {
    let mut g = pod(3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let crown = ready(&mut g, 0, catalog::transmogrants_crown());
    let bow = ready(&mut g, 0, catalog::hawkeyes_bow());
    for eq in [crown, bow] {
        flood(&mut g);
        g.perform_action(GameAction::Equip { equipment: eq, target: bear }).expect("equip");
        drain_stack(&mut g);
    }
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5);
    attack(&mut g, &[bear], 1);
    assert_eq!((g.players[1].life, g.players[2].life), (19, 19), "the Bow pinged each opponent");
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    let blade = g.add_card_to_hand(0, catalog::doom_blade());
    g.step = TurnStep::PostCombatMain;
    cast_at(&mut g, blade, Some(Target::Permanent(bear)));
    assert_eq!(g.players[0].hand.len(), hand + 1, "the Crown drew");
}

/// Molten Man fetches a basic Mountain, counts Mountains, and costs a land
/// when it leaves (with no Mountain out it dies as a 0/0 before the fetch).
#[test]
fn molten_man_fetches_counts_and_costs_a_land() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::mountain());
    g.add_card_to_library(0, catalog::mountain());
    let man = g.add_card_to_hand(0, catalog::molten_man_inferno_incarnate());
    cast_at(&mut g, man, None);
    assert_eq!(g.computed_permanent(man).unwrap().power, 2);
    let blade = g.add_card_to_hand(0, catalog::doom_blade());
    cast_at(&mut g, blade, Some(Target::Permanent(man)));
    let lands = g.battlefield.iter().filter(|c| c.definition.name == "Mountain").count();
    assert_eq!(lands, 1, "one land went with it");
}

/// Lovisa Coldeyes pumps every Warrior on the table, an opponent's included.
#[test]
fn lovisa_pumps_every_warrior() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::lovisa_coldeyes());
    let theirs = ready(&mut g, 1, catalog::radha_heart_of_keld());
    let cp = g.computed_permanent(theirs).unwrap();
    assert_eq!(cp.power, 5);
    assert!(cp.keywords().contains(&Keyword::Haste));
}

/// Jaheira's Respite fetches one basic per creature attacking you and fogs.
#[test]
fn jaheiras_respite_counts_attackers_and_prevents_combat_damage() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(0) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    let respite = g.add_card_to_hand(0, catalog::jaheiras_respite());
    cast_at(&mut g, respite, None);
    let forests = g.battlefield.iter().filter(|c| c.owner == 0 && c.definition.name == "Forest").count();
    assert_eq!(forests, 2);
}
