//! Commander: the Veloci-Ramp-Tor precon (LCC, Pantlaza, Sun-Favored,
//! `decks::cmdr_pantlaza`).

use crabomination::card::{CardId, CreatureType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Enrage draws: a player drawing from an empty library loses.
    for seat in 0..seats {
        for _ in 0..8 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    act(g, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    act(g, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: x,
        mode: None,
    })
}

fn shock(g: &mut GameState, at: CardId) {
    let s = g.add_card_to_hand(0, catalog::shock());
    cast(g, s, &[Target::Permanent(at)]).expect("shock");
}



/// CR 603.2d — Wayta: an enrage trigger of yours fires an additional time;
/// an opponent's enrage under the same damage does not.
#[test]
fn cr_603_2d_wayta_doubles_triggers_from_damage_to_your_creatures() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::wayta_trainer_prodigy());
    let mine = g.add_card_to_battlefield(0, catalog::ripjaw_raptor());
    let theirs = g.add_card_to_battlefield(1, catalog::ripjaw_raptor());
    let (hand0, hand1) = (g.players[0].hand.len(), g.players[1].hand.len());
    shock(&mut g, mine);
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "enrage fired twice");
    shock(&mut g, theirs);
    assert_eq!(g.players[1].hand.len(), hand1 + 1, "their enrage fired once");
}

/// Wayta's fight costs {2} less when both targets are yours.
#[test]
fn wayta_fight_discount_when_both_targets_are_yours() {
    let mut g = main_phase(2);
    let wayta = g.add_card_to_battlefield(0, catalog::wayta_trainer_prodigy());
    let a = g.add_card_to_battlefield(0, catalog::colossal_dreadmaw());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    activate(&mut g, wayta, 0, &[Target::Permanent(a), Target::Permanent(b)], None).expect("{G} only");
    assert!(g.battlefield_find(b).is_none(), "the Bear lost the fight");
}

/// CR 615 — Temple Altisaur prevents all but 1 of the damage to another
/// Dinosaur you control; itself and a non-Dinosaur take it all.
#[test]
fn cr_615_temple_altisaur_caps_damage_to_other_dinosaurs() {
    let mut g = main_phase(2);
    let temple = g.add_card_to_battlefield(0, catalog::temple_altisaur());
    let raptor = g.add_card_to_battlefield(0, catalog::ripjaw_raptor());
    shock(&mut g, raptor);
    assert_eq!(g.battlefield_find(raptor).unwrap().damage, 1);
    shock(&mut g, temple);
    assert_eq!(g.battlefield_find(temple).unwrap().damage, 2);
}

/// Progenitor's Icon — naming Dinosaur, a tap gives the next Dinosaur spell
/// flash; a non-Dinosaur is still sorcery-speed, and the grant is spent.
#[test]
fn progenitors_icon_grants_flash_to_the_chosen_type() {
    let mut g = main_phase(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Dinosaur)]));
    let icon = g.add_card_to_hand(0, catalog::progenitors_icon());
    cast(&mut g, icon, &[]).expect("cast the Icon");
    g.battlefield.iter_mut().for_each(|c| c.summoning_sick = false);
    activate(&mut g, icon, 1, &[], None).expect("tap for flash");
    g.step = TurnStep::BeginCombat;
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(cast(&mut g, bears, &[]).is_err(), "a Bear has no flash");
    let dino = g.add_card_to_hand(0, catalog::ripjaw_raptor());
    cast(&mut g, dino, &[]).expect("a Dinosaur has flash");
    assert!(g.players[0].next_typed_spell_flash_this_turn.is_empty(), "{:?}", g.players[0].next_typed_spell_flash_this_turn);
    let second = g.add_card_to_hand(0, catalog::ripjaw_raptor());
    assert!(cast(&mut g, second, &[]).is_err(), "the grant is spent");
}

/// CR 610.3 — Bronzebeak Foragers exiles one nonland permanent per
/// opponent until it leaves; {X}{W} sends an exiled card of mana value X to
/// its owner's graveyard for X life.
#[test]
fn cr_610_3_bronzebeak_foragers_exiles_per_opponent() {
    let mut g = main_phase(3);
    let a = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    let b = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let foragers = g.add_card_to_hand(0, catalog::bronzebeak_foragers());
    cast(&mut g, foragers, &[Target::Permanent(a), Target::Permanent(b)]).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == a) && g.exile.iter().any(|c| c.id == b));
    let life = g.players[0].life;
    flood(&mut g, 0);
    activate(&mut g, foragers, 0, &[Target::Permanent(b)], Some(2)).expect("X = 2");
    assert!(g.players[2].graveyard.iter().any(|c| c.id == b), "to its owner's graveyard");
    assert_eq!(g.players[0].life, life + 2);
}

/// Descendants' Path casts a revealed creature that shares a type with one
/// you control, for free.
#[test]
fn descendants_path_casts_a_kindred_top_card() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::descendants_path());
    g.add_card_to_battlefield(0, catalog::ripjaw_raptor());
    g.players[0].library.clear();
    let top = g.add_card_to_library(0, catalog::colossal_dreadmaw());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(top).is_some(), "the Dreadmaw (a Dinosaur) was cast");
}

/// Savage Stomp costs {G} on a Dinosaur of yours: +1/+1 counter, then it
/// fights.
#[test]
fn savage_stomp_discount_and_fight() {
    let mut g = main_phase(2);
    let raptor = g.add_card_to_battlefield(0, catalog::ripjaw_raptor());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let stomp = g.add_card_to_hand(0, catalog::savage_stomp());
    g.players[0].mana_pool.add(Color::Green, 1);
    act(&mut g, GameAction::CastSpell {
        card_id: stomp,
        target: Some(Target::Permanent(raptor)),
        additional_targets: vec![Target::Permanent(bears)],
        mode: None,
        x_value: None,
    })
    .expect("{G} on a Dinosaur");
    assert!(g.battlefield_find(bears).is_none());
    assert_eq!(g.computed_permanent(raptor).unwrap().power, 5);
}

/// Wakening Sun's Avatar cast from hand destroys every non-Dinosaur
/// creature; Zacama, cast, untaps your lands.
#[test]
fn wakening_suns_avatar_and_zacama_cast_triggers() {
    let mut g = main_phase(2);
    let raptor = g.add_card_to_battlefield(0, catalog::ripjaw_raptor());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let avatar = g.add_card_to_hand(0, catalog::wakening_suns_avatar());
    cast(&mut g, avatar, &[]).expect("cast");
    assert!(g.battlefield_find(bears).is_none());
    assert!(g.battlefield_find(raptor).is_some());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield.iter_mut().filter(|c| c.id == forest).for_each(|c| c.tapped = true);
    let zacama = g.add_card_to_hand(0, catalog::zacama_primal_calamity());
    cast(&mut g, zacama, &[]).expect("cast");
    assert!(!g.battlefield_find(forest).unwrap().tapped);
}

/// Marauding Raptor bites an entering Dinosaur for 2 and grows; Wrathful
/// Raptors sends a Dinosaur's damage on to a non-Dinosaur.
#[test]
fn marauding_and_wrathful_raptors() {
    let mut g = main_phase(2);
    let marauder = g.add_card_to_battlefield(0, catalog::marauding_raptor());
    g.add_card_to_battlefield(0, catalog::wrathful_raptors());
    let life = g.players[1].life;
    let dreadmaw = g.add_card_to_hand(0, catalog::colossal_dreadmaw());
    cast(&mut g, dreadmaw, &[]).expect("cast");
    assert_eq!(g.battlefield_find(dreadmaw).unwrap().damage, 2);
    assert_eq!(g.computed_permanent(marauder).unwrap().power, 4);
    assert_eq!(g.players[1].life, life - 2, "Wrathful Raptors sent the 2 to a player");
}

/// CR 702.100 evolve, then CR 701.57 — the Egg dying discovers its
/// last-known toughness (4 after one evolve counter).
#[test]
fn dinosaur_egg_evolves_then_discovers() {
    let mut g = main_phase(2);
    let egg = g.add_card_to_battlefield(0, catalog::dinosaur_egg());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bears, &[]).expect("cast");
    assert_eq!(g.computed_permanent(egg).unwrap().toughness, 4, "evolved");
    g.players[0].library.clear();
    let found = g.add_card_to_library(0, catalog::ripjaw_raptor());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(egg)]).expect("bolt");
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt2, &[Target::Permanent(egg)]).expect("bolt");
    assert!(g.battlefield_find(egg).is_none());
    assert!(!g.players[0].library.iter().any(|c| c.id == found), "the Raptor (MV 4) was discovered");
}
