//! Functionality tests for `catalog::sets::decks::recent121`.

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EntityRef;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Fill a graveyard with cards of four distinct types to satisfy Delirium.
fn make_delirium(g: &mut GameState, seat: usize) {
    g.add_card_to_graveyard(seat, catalog::grizzly_bears()); // creature
    g.add_card_to_graveyard(seat, catalog::lightning_bolt()); // instant
    g.add_card_to_graveyard(seat, catalog::forest()); // land
    g.add_card_to_graveyard(seat, catalog::divination()); // sorcery
}

/// Barkform Harvester tucks a graveyard card onto the bottom of the library.
#[test]
fn barkform_harvester_tucks_graveyard_card() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let harvester = g.add_card_to_battlefield(0, catalog::barkform_harvester());
    g.clear_sickness(harvester);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: harvester, ability_index: 0, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], x_value: None,
    }).expect("tuck");
    drain_stack(&mut g);
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt), "left the graveyard");
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bolt), "on the bottom");
}

/// Bonebind Orator returns another creature card from the graveyard, exiling itself.
#[test]
fn bonebind_orator_graveyard_recursion() {
    let mut g = two_player_game();
    let orator = g.add_card_to_graveyard(0, catalog::bonebind_orator());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: orator, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("recur");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bear returned to hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == orator), "orator exiled itself");
}

/// Clifftop Lookout digs a land onto the battlefield tapped on entry.
#[test]
fn clifftop_lookout_ramps_a_land() {
    let mut g = two_player_game();
    // Top of library: two nonlands then a Forest.
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let lookout = g.add_card_to_battlefield(0, catalog::clifftop_lookout());
    g.fire_self_etb_triggers(lookout, 0);
    drain_stack(&mut g);
    let land = g.battlefield.iter().find(|c| c.definition.name == "Forest");
    assert!(land.is_some(), "a Forest hit the battlefield");
    assert!(land.unwrap().tapped, "and it entered tapped");
}

/// Brambleguard Captain pumps a creature by its own power at combat.
#[test]
fn brambleguard_captain_begin_combat_pump() {
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::brambleguard_captain()); // 2/3
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(cap);
    g.active_player_idx = 0;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "2/2 + captain's power 2 → 4/2");
}

/// Downwind Ambusher's first mode shrinks an opponent's creature.
#[test]
fn downwind_ambusher_minus_mode() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ambusher = g.add_card_to_battlefield(0, catalog::downwind_ambusher());
    g.fire_self_etb_triggers(ambusher, 0);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 1, "2/2 → 1/1");
}

/// Cracked Skull destroys the enchanted creature when it's dealt damage.
#[test]
fn cracked_skull_destroys_on_damage() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let skull = g.add_card_to_hand(0, catalog::cracked_skull());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: skull, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cracked Skull on the bear");
    drain_stack(&mut g);
    // Now deal a point of damage to the enchanted creature.
    let mut ev = vec![];
    g.deal_damage_to_from(EntityRef::Permanent(bear), 1, None, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "enchanted creature destroyed");
}

/// Beastie Beatdown: a delirium fight buffs your creature and kills theirs.
#[test]
fn beastie_beatdown_delirium_fight() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    make_delirium(&mut g, 0);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 4/4 delirium
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::beastie_beatdown());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast Beastie Beatdown");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 4, "delirium → two +1/+1 counters");
    assert!(!g.battlefield.iter().any(|c| c.id == theirs), "4 damage kills the 2/2");
}

/// Balustrade Wurm reanimates itself from the graveyard under Delirium.
#[test]
fn balustrade_wurm_delirium_reanimates() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    make_delirium(&mut g, 0);
    let wurm = g.add_card_to_graveyard(0, catalog::balustrade_wurm());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wurm, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("reanimate");
    drain_stack(&mut g);
    let onbf = g.battlefield.iter().find(|c| c.definition.name == "Balustrade Wurm");
    assert!(onbf.is_some(), "wurm returned to the battlefield");
    assert!(onbf.unwrap().counter_count(crabomination::card::CounterType::Finality) >= 1, "with a finality counter");
}

/// Drag to the Roots costs {2} less while Delirium is active.
#[test]
fn drag_to_the_roots_delirium_discount() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    make_delirium(&mut g, 0);
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::drag_to_the_roots());
    // Discounted cost is {B}{G} instead of {2}{B}{G}.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast at the delirium discount");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == target), "nonland permanent destroyed");
}
