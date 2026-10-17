//! Commander: the 2026-10-17 (`019Y6J4R`) second closer batch
//! (`decks::cmdr_019y_b`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(6);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    cast_mode(g, id, target, None)
}

fn cast_mode(
    g: &mut GameState,
    id: CardId,
    target: Option<Target>,
    mode: Option<usize>,
) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode,
        x_value: None,
    })?;
    drain_stack(g);
    Ok(())
}


fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn activate(g: &mut GameState, id: CardId, idx: usize) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn name_of(g: &GameState, id: CardId) -> &'static str {
    g.battlefield_find(id).unwrap().definition.name
}

fn upkeep(g: &mut GameState) {
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(g);
}

/// Ravenous Demon transforms by sacrificing a Human (sorcery speed); the
/// Archdemon without a Human to feed it taps and deals 9 to its controller.
#[test]
fn ravenous_demon_eats_a_human_then_starves() {
    let mut g = pod(2);
    let demon = ready(&mut g, 0, catalog::ravenous_demon());
    assert!(activate(&mut g, demon, 0).is_err(), "no Human to sacrifice");
    let human = ready(&mut g, 0, catalog::elite_vanguard());
    activate(&mut g, demon, 0).expect("sacrifice a Human");
    assert!(g.battlefield_find(human).is_none());
    assert_eq!(name_of(&g, demon), "Archdemon of Greed");
    assert_eq!(pt(&g, demon), (9, 9));
    let life = g.players[0].life;
    upkeep(&mut g);
    assert_eq!(g.players[0].life, life - 9);
    assert!(g.battlefield_find(demon).unwrap().tapped);
}

/// Master's Guide-Mural makes a Golem, crafts with an artifact into the
/// Manufactory, which makes a Golem the turn an artifact entered.
#[test]
fn masters_guide_mural_crafts_into_a_golem_factory() {
    let golems =
        |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Golem").count();
    let mut g = pod(2);
    let mural = g.add_card_to_hand(0, catalog::masters_guide_mural());
    flood(&mut g, 0);
    cast(&mut g, mural, None).expect("cast");
    assert_eq!(golems(&g), 1);
    ready(&mut g, 0, catalog::sol_ring());
    activate(&mut g, mural, 0).expect("craft with Sol Ring");
    let factory = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Master's Manufactory")
        .map(|c| c.id)
        .expect("returned transformed");
    g.clear_sickness(factory);
    activate(&mut g, factory, 0).expect("an artifact entered this turn");
    assert_eq!(golems(&g), 2);
}

/// Foggy Swamp Visions (waterbend {X} = 1) exiles an opponent's creature card
/// and makes a token copy that's sacrificed at your next end step.
#[test]
fn foggy_swamp_visions_borrows_a_dead_creature_for_the_turn() {
    let mut g = pod(2);
    let dead = g.add_card_to_graveyard(1, catalog::serra_angel());
    let fsv = g.add_card_to_hand(0, catalog::foggy_swamp_visions());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellWaterbend {
        card_id: fsv,
        target: Some(Target::Permanent(dead)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(1),
        helpers: vec![],
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == dead));
    let angels = |g: &GameState| {
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Serra Angel").count()
    };
    assert_eq!(angels(&g), 1);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(angels(&g), 0, "sacrificed at the end step");
}

/// Living Breakthrough: after you cast a one-drop, opponents can't cast
/// mana-value-1 spells until your next turn; other values stay open.
#[test]
fn living_breakthrough_locks_the_cast_spells_mana_value() {
    let mut g = pod(2);
    let back = *catalog::inventive_iteration().back_face.unwrap();
    ready(&mut g, 0, back);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    assert_eq!(g.players[0].opponents_cant_cast_mv, vec![1]);
    let shock = g.add_card_to_hand(1, catalog::shock());
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    assert!(cast(&mut g, shock, Some(Target::Player(0))).is_err(), "MV 1 is locked");
    g.active_player_idx = 1;
    cast(&mut g, bears, None).expect("MV 2 is open");
}

/// Cemetery Illuminator exiles a creature card entering, then lets you cast a
/// creature spell from the top of your library (once a turn).
#[test]
fn cemetery_illuminator_casts_a_shared_type_from_the_top() {
    let mut g = multi_player_game(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let top = g.add_card_to_library(0, catalog::savannah_lions());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let ci = g.add_card_to_hand(0, catalog::cemetery_illuminator());
    flood(&mut g, 0);
    cast(&mut g, ci, None).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == dead && c.exiled_with == Some(ci)));
    cast(&mut g, top, None).expect("a creature, like the exiled Bears");
    assert!(g.battlefield_find(top).is_some());
    assert!(cast(&mut g, bolt, Some(Target::Player(1))).is_err(), "an instant shares no type");
}

/// Gwen Stacy exiles the top card she lets you play; transformed, playing it
/// from exile grows Ghost-Spider, whose two counters buy another exile.
#[test]
fn gwen_stacy_plays_from_exile_and_ghost_spider_grows() {
    let mut g = multi_player_game(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let gwen = g.add_card_to_hand(0, catalog::gwen_stacy());
    flood(&mut g, 0);
    cast(&mut g, gwen, None).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bears));
    g.clear_sickness(gwen);
    activate(&mut g, gwen, 0).expect("transform");
    assert_eq!(name_of(&g, gwen), "Ghost-Spider");
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast it from exile, paying its cost");
    drain_stack(&mut g);
    assert_eq!(pt(&g, gwen), (5, 5));
    g.battlefield_find_mut(gwen).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    activate(&mut g, gwen, 0).expect("remove two counters");
    assert_eq!(pt(&g, gwen), (4, 4));
    assert!(g.exile.iter().any(|c| c.owner == 0 && c.definition.name == "Forest"));
}
