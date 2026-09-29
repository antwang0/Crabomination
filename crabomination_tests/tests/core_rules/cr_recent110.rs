//! CR 205.3 / 702.73a — a permanent's creature types are its current ones: a
//! changeling is every creature type, and an animated land is a creature of
//! the types its animation gives. Battlefield walks that read the printed
//! type line got both wrong (`GameState::permanent_has_creature_type`).

use crabomination::card::{CreatureType, CardDefinition};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;
use crabomination::TurnStep;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

fn activate(g: &mut GameState, id: crabomination::card::CardId, index: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: index, target, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Crippling Fear naming Elf: a changeling keeps its size, an animated Treetop
/// Village (a creature that isn't an Elf) shrinks — the printed read had both
/// backwards (the Village isn't a printed creature, the Outcast isn't a
/// printed Elf).
#[test]
fn cr_702_73a_crippling_fear_reads_current_types() {
    let mut g = main_phase();
    let outcast = g.add_card_to_battlefield(1, catalog::changeling_outcast());
    let village = g.add_card_to_battlefield(1, catalog::treetop_village());
    g.clear_sickness(village);
    g.battlefield_find_mut(village).unwrap().controller = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, village, 1, None);
    g.battlefield_find_mut(village).unwrap().controller = 1;
    let fear = g.add_card_to_hand(0, catalog::crippling_fear());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Elf)]));
    g.perform_action(GameAction::CastSpell { card_id: fear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(outcast).is_some(), "a changeling is an Elf");
    assert!(g.battlefield_find(village).is_none(), "the 3/3 Ape took -3/-3");
}

/// Elvish Guidance ("an additional {G} for each Elf on the battlefield") and
/// Sliver Legion (+1/+1 per other Sliver) count a changeling.
#[test]
fn cr_702_73a_type_counts_include_changelings() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let guidance = g.add_card_to_battlefield(0, catalog::elvish_guidance());
    g.battlefield_find_mut(guidance).unwrap().attached_to = Some(land);
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.add_card_to_battlefield(1, catalog::changeling_outcast());
    activate(&mut g, land, 0, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3, "{{G}} + Elves + the changeling");

    let mut g = main_phase();
    let legion: CardDefinition = catalog::sliver_legion();
    let legion = g.add_card_to_battlefield(0, legion);
    g.add_card_to_battlefield(1, catalog::changeling_outcast());
    assert_eq!(g.computed_permanent(legion).unwrap().power, 8, "7/7 + one other Sliver");
}

/// CR 613.1d — "is a creature" on the battlefield is the layer view: Living
/// Death's "sacrifices all creatures they control" takes an animated Treetop
/// Village (the walk read the printed type line and left it).
#[test]
fn cr_613_1d_living_death_sacrifices_an_animated_land() {
    let mut g = main_phase();
    let village = g.add_card_to_battlefield(0, catalog::treetop_village());
    g.clear_sickness(village);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, village, 1, None);
    let ld = g.add_card_to_hand(0, catalog::living_death());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: ld, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(village).is_none(), "the 3/3 Ape was a creature");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == village));
}
