//! Most-built commanders (`decks::cmdr_legends`, COMMANDER_BACKLOG §1).

use crabomination::card::{CardId, CounterType};
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
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

fn attack(g: &mut GameState, attacker: CardId) {
    g.clear_sickness(attacker);
    advance_to(g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(g);
}

/// CR 105.2 / 603.2: Aragorn's color triggers — Lightning Helix (red and
/// white) makes a Soldier *and* deals 3 to the targeted opponent.
#[test]
fn aragorn_fires_one_trigger_per_color_of_the_spell() {
    let mut g = pod(4);
    g.add_card_to_battlefield(0, catalog::aragorn_the_uniter());
    let helix = g.add_card_to_hand(0, catalog::lightning_helix());
    let before: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    cast(&mut g, 0, helix, Some(Target::Player(3)));
    let soldiers = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Human Soldier").count();
    assert_eq!(soldiers, 1, "the white trigger");
    let lost: i32 = (1..4).map(|p| before[p] - g.players[p].life).sum();
    assert_eq!(lost, 6, "the Helix's 3 plus the red trigger's 3 at an opponent");
    assert_eq!(g.players[0].life, before[0] + 3, "the Helix's lifegain");
}

/// Zur's attack trigger puts an enchantment with mana value 3 or less onto
/// the battlefield (CR 701.19a); a bigger one isn't findable.
#[test]
fn zur_fetches_a_small_enchantment_on_attack() {
    let mut g = two_player_game();
    let zur = g.add_card_to_battlefield(0, catalog::zur_the_enchanter());
    let small = g.add_card_to_library(0, catalog::honor_of_the_pure());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(small))]));
    attack(&mut g, zur);
    assert!(g.battlefield.iter().any(|c| c.id == small), "Honor of the Pure came from the library");
}

/// Flubs: with cards in hand a spell cast makes you discard; with an empty
/// hand it draws. A second land drop is allowed (CR 305.2).
#[test]
fn flubs_draws_on_empty_hand_and_discards_otherwise() {
    let mut g = pod(4);
    g.add_card_to_battlefield(0, catalog::flubs_the_fool());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::island());
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, 0, opt, None);
    // Opt drew one (hand = Island + drawn card), then Flubs discarded one.
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].graveyard.len(), 2, "Opt and the discard");
    // The last card is an Island: playing it empties the hand, so Flubs draws.
    let last = g.players[0].hand[0].id;
    g.perform_action(GameAction::PlayLand(last)).expect("land drop");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "empty hand → draw");
}

/// CR 700.2: Galadriel's alliance picks a mode not chosen this turn — three
/// creatures entering run all three modes, each once.
#[test]
fn galadriel_rotates_modes_within_a_turn() {
    let mut g = pod(4);
    let gal = g.add_card_to_battlefield(0, catalog::galadriel_light_of_valinor());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    for _ in 0..3 {
        let b = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast(&mut g, 0, b, None);
    }
    let picks = &g.battlefield_find(gal).unwrap().modes_chosen;
    let mut modes = picks[4..].to_vec();
    modes.sort_unstable();
    assert_eq!(modes, vec![0, 1, 2]);
    assert!(g.battlefield_find(gal).unwrap().counter_count(CounterType::PlusOnePlusOne) >= 1);
}

/// CR 510.1c / 702.3b: under Arcades a defender attacks and assigns combat
/// damage equal to its toughness; one entering draws a card.
#[test]
fn arcades_lets_walls_attack_for_toughness() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::arcades_the_strategist());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let wall = g.add_card_to_hand(0, catalog::wall_of_omens()); // 0/4, ETB draw
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, wall, None);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "Wall of Omens' draw plus Arcades'");
    let life = g.players[1].life;
    attack(&mut g, wall);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, life - 4, "a 0/4 wall hits for 4");
}

/// Tiamat cast from hand tutors up to five Dragons not named Tiamat; put
/// onto the battlefield without being cast, it doesn't (CR 603.4).
#[test]
fn tiamat_tutors_dragons_only_when_cast() {
    for was_cast in [true, false] {
        let mut g = pod(4);
        let d1 = g.add_card_to_library(0, catalog::shivan_dragon());
        g.add_card_to_library(0, catalog::island());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Search(Some(d1)),
            DecisionAnswer::Search(None),
        ]));
        if was_cast {
            let t = g.add_card_to_hand(0, catalog::tiamat());
            cast(&mut g, 0, t, None);
        } else {
            g.add_card_to_battlefield(0, catalog::tiamat());
            drain_stack(&mut g);
        }
        assert_eq!(g.players[0].hand.iter().any(|c| c.id == d1), was_cast, "was_cast={was_cast}");
    }
}

/// CR 701.47: each opponent's spell amasses Orcs 1 for Sauron's controller —
/// the first makes a 0/0 Orc Army with a counter, the second grows it.
#[test]
fn sauron_amasses_on_each_opponent_spell() {
    let mut g = pod(4);
    g.add_card_to_battlefield(0, catalog::sauron_the_dark_lord());
    for seat in [1, 2] {
        let s = g.add_card_to_hand(seat, catalog::opt());
        g.add_card_to_library(seat, catalog::island());
        cast(&mut g, seat, s, None);
    }
    let armies: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Army))
        .collect();
    assert_eq!(armies.len(), 1, "one Army, grown");
    assert_eq!(armies[0].counter_count(CounterType::PlusOnePlusOne), 2);
}

/// CR 701.68: Morcant entering makes each opponent blight 1 — a -1/-1
/// counter on a creature they control; an opponent with none is skipped.
#[test]
fn morcant_makes_each_opponent_blight() {
    let mut g = pod(4);
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::high_perfect_morcant());
    cast(&mut g, 0, m, None);
    for b in [b1, b2] {
        assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
    }
}
