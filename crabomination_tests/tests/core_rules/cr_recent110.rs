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

/// CR 702.14c / 305.7 — landwalk reads the defender's current land types:
/// Urborg makes their Forest a Swamp, so Bog Wraith's swampwalk makes it
/// unblockable (the check read the printed type line).
#[test]
fn cr_702_14c_swampwalk_sees_urborg() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::urborg_tomb_of_yawgmoth());
    let wraith = g.add_card_to_battlefield(0, catalog::bog_wraith());
    g.clear_sickness(wraith);
    g.add_card_to_battlefield(1, catalog::forest());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: wraith, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(bear, wraith)])).is_err(), "the Forest is a Swamp");
}

/// CR 105.2 — "colors among permanents you control" are current colours: an
/// animated Treetop Village is a green Ape, so Shimmercreep (black) drains 2,
/// not 1 (the count read the Village's printed, colourless line).
#[test]
fn cr_105_2_vivid_counts_an_animated_lands_colour() {
    let mut g = main_phase();
    let village = g.add_card_to_battlefield(0, catalog::treetop_village());
    g.clear_sickness(village);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, village, 1, None);
    let opp = g.players[1].life;
    g.move_card_to_battlefield_for_test(0, catalog::shimmercreep());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "black + green");
}

/// CR 613.1d / 702.73a — Shared Animosity counts an animated Mutavault (a
/// creature with every creature type while animated) as an attacker sharing
/// the Elf's type: the walk read the printed, noncreature line.
#[test]
fn cr_613_1d_shared_animosity_counts_an_animated_mutavault() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::shared_animosity());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let vault = g.add_card_to_battlefield(0, catalog::mutavault());
    g.clear_sickness(elf);
    g.clear_sickness(vault);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, vault, 1, None);
    g.step = TurnStep::DeclareAttackers;
    let at = |attacker| Attack { attacker, target: AttackTarget::Player(1) };
    g.perform_action(GameAction::DeclareAttackers(vec![at(elf), at(vault)])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(elf).unwrap().power, 2, "1 + the other attacking Elf-typed creature");
}
