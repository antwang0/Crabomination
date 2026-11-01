//! Star Trek: the sixth batch of EDHREC closers (`decks::trek_singles6`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn faeries(g: &GameState) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Faerie").map(|c| c.id).collect()
}

/// Data, Professor of Humanities (seek out): the SECOND draw of a turn makes
/// two Faeries; the first and third don't.
#[test]
fn data_makes_faeries_on_the_second_draw() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::data_professor_of_humanities());
    for n in [0usize, 2, 2] {
        let mut ev = Vec::new();
        g.draw_one(0, &mut ev);
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert_eq!(faeries(&g).len(), n);
    }
}

/// Jean-Luc, Mon Capitaine (federation, CR 614.12): a Faerie enters with a
/// +1/+1 counter per creature type among the non-Borg creatures you ALREADY
/// control — Human, Officer, Bear; the Faerie's own type doesn't count.
#[test]
fn jean_luc_grows_entering_faeries_by_federation() {
    use crabomination::card::{CardType, CreatureType, Subtypes, TokenDefinition, Value};
    let mut g = pod(2);
    ready(&mut g, 0, catalog::jean_luc_mon_capitaine());
    ready(&mut g, 0, catalog::grizzly_bears());
    let faerie = TokenDefinition {
        name: "Faerie".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        ..Default::default()
    };
    let make = crabomination::effect::Effect::CreateToken {
        who: crabomination::effect::PlayerRef::You,
        count: Value::ONE,
        definition: std::sync::Arc::new(faerie),
    };
    let ev = g.resolve_effect(&make, &EffectContext::for_spell(0, None, 0, 0)).expect("mint");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let fs = faeries(&g);
    assert_eq!(fs.len(), 1);
    assert_eq!(g.battlefield_find(fs[0]).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Uhura, Xenolinguist: a non-Human creature entering (an opponent's too)
/// offers {1} for a card; a Human doesn't.
#[test]
fn uhura_pays_to_draw_off_non_humans() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::uhura_xenolinguist());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    flood(&mut g, 2);
    g.active_player_idx = 2;
    for def in [catalog::grizzly_bears(), catalog::uhura_xenolinguist()] {
        let id = g.add_card_to_hand(2, def);
        g.priority.player_with_priority = 2;
        g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("cast");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Wesley Crusher: the default pair — a creature can't be blocked, and the
/// target player draws two then discards two.
#[test]
fn wesley_crusher_chooses_two_modes() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let wes = g.add_card_to_hand(0, catalog::wesley_crusher_starfleet_cadet());
    flood(&mut g, 0);
    let lib = g.players[1].library.len();
    g.perform_action(GameAction::CastSpell { card_id: wes, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let unblockable = |id| g.computed_permanent(id).unwrap().keywords().contains(&crabomination::card::Keyword::Unblockable);
    assert!(unblockable(bear) || unblockable(wes));
    let drew = g.players[0].library.len() + 2 == 10 || g.players[1].library.len() + 2 == lib;
    assert!(drew, "someone looted two");
}

/// Steal Artifact: you control the enchanted artifact; it returns when the
/// Aura leaves.
#[test]
fn steal_artifact_takes_an_artifact() {
    let mut g = pod(2);
    let rock = ready(&mut g, 1, catalog::sol_ring());
    let aura = g.add_card_to_hand(0, catalog::steal_artifact());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(rock)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rock).unwrap().controller, 0);
}

/// The TRK campus lands (CR 614.1c / 603.2): with two other lands Ni'Var
/// enters untapped and its "enters untapped" trigger fires; with fewer it is
/// tapped and nothing triggers.
#[test]
fn campus_lands_enter_untapped_with_two_other_lands() {
    for (others, untapped) in [(1usize, false), (2, true)] {
        let mut g = pod(2);
        for _ in 0..others {
            g.add_card_to_battlefield(0, catalog::island());
        }
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        let land = g.add_card_to_hand(0, catalog::ni_var_science_academy());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
        g.perform_action(GameAction::PlayLand(land)).expect("land");
        drain_stack(&mut g);
        assert_eq!(!g.battlefield_find(land).unwrap().tapped, untapped);
        let flying = g.computed_permanent(bear).unwrap().keywords().contains(&crabomination::card::Keyword::Flying);
        assert_eq!(flying, untapped);
    }
}

/// Spock, Voice of Reason: draws one per NONTOKEN permanent you control that
/// entered this turn.
#[test]
fn spock_counts_nontoken_arrivals() {
    let mut g = pod(2);
    let spock = ready(&mut g, 0, catalog::spock_voice_of_reason());
    let hand = g.players[0].hand.len();
    for def in [catalog::grizzly_bears(), catalog::island()] {
        g.move_card_to_battlefield_for_test(0, def);
    }
    ready(&mut g, 0, catalog::data_professor_of_humanities());
    let mut ev = Vec::new();
    g.draw_one(0, &mut ev);
    g.draw_one(0, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let drawn = g.players[0].hand.len() - hand;
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: spock,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    // Bears, Island, Data (and Spock himself, put in this turn): not the Faeries.
    let entered = g.battlefield.iter().filter(|c| c.controller == 0 && !c.is_token && c.entered_turn == Some(g.turn_number)).count();
    assert_eq!(g.players[0].hand.len() - hand - drawn, entered);
}

/// Q, Your Illustrious Host: tapping three Faeries digs to the first nonland
/// card and casts it free.
#[test]
fn q_host_taps_three_faeries_to_cast_free() {
    let mut g = pod(2);
    let q = ready(&mut g, 0, catalog::q_your_illustrious_host());
    ready(&mut g, 0, catalog::data_professor_of_humanities());
    for _ in 0..2 {
        let mut ev = Vec::new();
        g.draw_one(0, &mut ev);
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
    }
    let extra = ready(&mut g, 0, catalog::spellstutter_sprite());
    let fs: Vec<CardId> = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Faerie").map(|c| c.id).collect();
    for f in fs.iter().copied().chain([extra]) {
        g.clear_sickness(f);
    }
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].library.rotate_right(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: q,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "cast for free");
    let tapped = g.battlefield.iter().filter(|c| c.controller == 0 && c.tapped && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Faerie)).count();
    assert_eq!(tapped, 3);
}
