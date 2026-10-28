//! Star Trek Commander: the We Are the Borg precon's new cards
//! (`decks::trek_borg`) — assimilate and the artifact-creature grants.

use crabomination::card::{CardId, CardType, CounterType, CreatureType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::{Color, SpellKind, SpendRestriction};

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn assert_assimilated(g: &GameState, id: CardId, seat: usize) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    assert_eq!(c.controller, seat);
    assert!(c.card_types().contains(&CardType::Artifact) && c.card_types().contains(&CardType::Creature));
    assert_eq!(c.subtypes().creature_types, vec![CreatureType::Borg], "loses all other creature types");
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Assimilate (TRC keyword action) on a permanent: control, a +1/+1 counter,
/// a Borg artifact creature. Chapter III blinks that Borg back to its owner as
/// a new object, assimilation gone.
#[test]
fn best_of_both_worlds_assimilates_then_returns_the_borg() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let saga = g.add_card_to_hand(0, catalog::the_best_of_both_worlds());
    flood(&mut g, 0);
    cast(&mut g, saga, Some(Target::Permanent(giant)));
    assert_assimilated(&g, giant, 0);
    let saga_id = g.battlefield.iter().find(|c| c.definition.name == "The Best of Both Worlds").unwrap().id;
    for _ in 0..2 {
        g.saga_advance(saga_id);
        drain_stack(&mut g);
    }
    let back = g.battlefield.iter().find(|c| c.definition.name == "Hill Giant").expect("returned");
    assert_eq!(back.controller, 1, "under its owner's control");
    let c = g.computed_permanent(back.id).unwrap();
    assert!(!c.card_types().contains(&CardType::Artifact), "a new object, not assimilated");
}

/// The Borg Queen's arrival: the targeted opponent reveals down to a creature
/// card, which enters assimilated under her controller; the reveals above it
/// go to the bottom.
#[test]
fn borg_queen_assimilates_the_first_creature_card_down() {
    let mut g = pod(2);
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::lightning_bolt());
    let giant = g.add_card_to_library(1, catalog::hill_giant());
    g.add_card_to_library(1, catalog::forest());
    let queen = g.add_card_to_hand(0, catalog::the_borg_queen());
    flood(&mut g, 0);
    cast(&mut g, queen, None);
    assert_assimilated(&g, giant, 0);
    let names: Vec<_> = g.players[1].library.iter().map(|c| c.definition.name).collect();
    assert_eq!(names[0], "Forest", "the unrevealed card stays on top");
    assert_eq!(names.len(), 3);
    // The Queen's anthem: the assimilated 3/3 Giant, its counter, +2/+2.
    let c = g.computed_permanent(giant).unwrap();
    assert_eq!((c.power, c.toughness), (6, 6));
}

/// Gift of Perfection: every opponent reveals, but only one creature card is
/// assimilated — the biggest.
#[test]
fn gift_of_perfection_assimilates_one_of_all_the_reveals() {
    let mut g = pod(4);
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    let wurm = g.add_card_to_library(2, catalog::craw_wurm());
    let giant = g.add_card_to_library(3, catalog::hill_giant());
    let gift = g.add_card_to_hand(0, catalog::gift_of_perfection());
    flood(&mut g, 0);
    cast(&mut g, gift, None);
    assert_assimilated(&g, wurm, 0);
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(giant).is_none());
    assert_eq!(g.players[1].library.len(), 1, "the others go back to the bottom");
}

/// Join the Collective counts the nonartifact creatures it destroyed, tokens
/// included, and not the artifact ones.
#[test]
fn join_the_collective_counts_nonartifact_creatures_destroyed() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::hill_giant());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::stratagem_emulator());
    let spell = g.add_card_to_hand(0, catalog::join_the_collective());
    flood(&mut g, 0);
    cast(&mut g, spell, None);
    let borgs = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Borg").count();
    // Two nonartifact creatures, plus Stratagem Emulator's dying 2 power.
    assert_eq!(borgs, 4);
}

/// Optimization Annex: a charge counter put on your artifact comes with one
/// more; a +1/+1 counter does not.
#[test]
fn optimization_annex_adds_a_charge_counter() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::optimization_annex());
    let ship = g.add_card_to_battlefield(0, catalog::class_4_tactical_cube());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // Station: tap the 2-power Bear for two charge counters, plus one.
    g.perform_action(GameAction::ActivateAbility {
        card_id: ship,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("station");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ship).unwrap().counter_count(CounterType::Charge), 3);
}

/// Queen's Diamond's mana pays only for its controller's commander.
#[test]
fn queens_diamond_mana_is_commander_only() {
    let r = SpendRestriction::CommanderSpellsOnly;
    assert!(r.allows(&SpellKind { commander: true, ..Default::default() }));
    assert!(!r.allows(&SpellKind::default()));
    assert!(!r.allows(&SpellKind { commander: true, activating_ability: true, ..Default::default() }));
}

/// Hugh: an artifact spell replicates for {2}{U}; each copy becomes a token.
#[test]
fn hugh_gives_artifact_spells_replicate() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::hugh_first_of_many());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellReplicate {
        card_id: ring,
        times: 2,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("replicated cast");
    drain_stack(&mut g);
    let rings: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Sol Ring").collect();
    assert_eq!(rings.len(), 3);
    assert_eq!(rings.iter().filter(|c| c.is_token).count(), 2);
}

/// Võx: in a four-player game the next artifact creature spell costs {3}
/// less (undaunted, CR 702.125), once.
#[test]
fn vox_grants_undaunted_to_the_next_artifact_creature() {
    let mut g = pod(4);
    let vox = g.add_card_to_hand(0, catalog::vox_child_of_the_collective());
    flood(&mut g, 0);
    cast(&mut g, vox, None);
    g.players[0].mana_pool.empty();
    let a = g.add_card_to_hand(0, catalog::hive_mind_coprocessor());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, a, None);
    assert!(g.battlefield_find(a).is_some(), "{{4}}{{B}} paid with {{1}}{{B}}");
    let b = g.add_card_to_hand(0, catalog::extraction_integrant());
    g.players[0].mana_pool.add(Color::Black, 1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: b,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the grant is spent"
    );
}
