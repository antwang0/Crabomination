//! Commander: one-card EDHREC gaps ranked 1001-1500 (`decks::cmdr_edhrec14`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::grizzly_bears());
    }
}

fn act(g: &mut GameState, a: GameAction) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(a).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
}

/// Fury of the Horde (CR 118.9, 601.2b): "exile two red cards from your hand
/// rather than pay this spell's mana cost" takes the pitch card and one more
/// red card — and with only one other red card the alternative cost is
/// unpayable.
#[test]
fn fury_of_the_horde_exiles_two_red_cards_for_its_alternative_cost() {
    let mut g = pod(4);
    let fury = g.add_card_to_hand(0, catalog::fury_of_the_horde());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let alt = |pitch| GameAction::CastSpellAlternative {
        card_id: fury,
        pitch_card: Some(pitch),
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };
    assert!(act(&mut g, alt(bolt)).is_err(), "one red card can't pay for two");
    let second = g.add_card_to_hand(0, catalog::lightning_bolt());
    act(&mut g, alt(bolt)).expect("two red cards pay it");
    assert!(g.exile.iter().any(|c| c.id == bolt) && g.exile.iter().any(|c| c.id == second));
    assert!(g.players[0].hand.is_empty());
}

/// Plate Armor: "Equip {3}. This ability costs {1} less to activate for each
/// other Equipment you control" — two others make it {1}.
#[test]
fn plate_armor_equip_costs_less_per_other_equipment() {
    let mut g = pod(2);
    let armor = ready(&mut g, 0, catalog::plate_armor());
    ready(&mut g, 0, catalog::bonesplitter());
    ready(&mut g, 0, catalog::bonesplitter());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    act(&mut g, GameAction::Equip { equipment: armor, target: bear }).expect("equip for {1}");
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5));
    assert!(c.keywords().iter().any(|k| matches!(k, Keyword::Ward(_))));
}

/// Heartwood Storyteller (CR 102.2 at four seats): a noncreature spell draws
/// each of its caster's opponents a card — the caster draws nothing.
#[test]
fn heartwood_storyteller_draws_each_of_the_casters_opponents() {
    let mut g = pod(4);
    ready(&mut g, 2, catalog::heartwood_storyteller());
    for s in 0..4 {
        library(&mut g, s, 3);
    }
    let hands: Vec<usize> = (0..4).map(|s| g.players[s].hand.len()).collect();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    assert_eq!(g.players[0].hand.len(), hands[0]);
    for s in 1..4 {
        assert_eq!(g.players[s].hand.len(), hands[s] + 1, "seat {s} draws");
    }
}

/// Qala, Ajani's Pridemate: life gained grows Qala; attacking pumps the other
/// attackers by the counters on it.
#[test]
fn qala_counts_its_counters_into_the_other_attackers() {
    let mut g = pod(2);
    let qala = ready(&mut g, 0, catalog::qala_ajanis_pridemate());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(6);
    g.players[0].mana_pool.add(Color::White, 2);
    for _ in 0..2 {
        act(&mut g, GameAction::ActivateAbility {
            card_id: qala,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("gain 1");
    }
    assert_eq!(g.battlefield_find(qala).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    g.step = TurnStep::DeclareAttackers;
    act(&mut g, GameAction::DeclareAttackers(vec![
        crabomination::game::types::Attack { attacker: qala, target: crabomination::game::types::AttackTarget::Player(1) },
        crabomination::game::types::Attack { attacker: bear, target: crabomination::game::types::AttackTarget::Player(1) },
    ]))
    .expect("attack");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0 from two counters");
}
