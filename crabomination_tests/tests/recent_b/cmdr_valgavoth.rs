//! Commander: the Endless Punishment precon (DSC, Valgavoth,
//! `decks::cmdr_valgavoth`) and the primitives it needed.

use crabomination::card::{CardDefinition, CardId, CardType, CreatureType, Subtypes};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::{Color, SpendRestriction, cost, generic};

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// CR 106.6 — "spend this mana only to cast instant, sorcery, Demon, and
/// Spirit spells": a sorcery and a Demon may use it, a Bear may not.
#[test]
fn cr_106_6_instant_sorcery_demon_or_spirit_mana() {
    let restricted = SpendRestriction::InstantSorceryOrTypes([CreatureType::Demon, CreatureType::Spirit]);
    let demon = || CardDefinition {
        name: "Test Demon",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    };
    let bear2 = || CardDefinition { name: "Test Bear", subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() }, ..demon() };
    let mut g = pod(2);
    let b = g.add_card_to_hand(0, bear2());
    g.players[0].mana_pool.add_restricted(Color::Black, 2, restricted);
    assert!(cast(&mut g, 0, b, None).is_err(), "a Bear can't spend it");
    let d = g.add_card_to_hand(0, demon());
    cast(&mut g, 0, d, None).expect("a Demon can");
    g.players[0].mana_pool.add_restricted(Color::Red, 1, restricted);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Player(1))).expect("an instant can");
}
