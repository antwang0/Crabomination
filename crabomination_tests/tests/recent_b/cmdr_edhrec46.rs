//! Commander: Samut's, The Walls of Ba Sing Se's and Sasaya's last gaps
//! (`decks::cmdr_edhrec46`).

use crabomination::card::{CardId, CardType, Keyword};
use crabomination::catalog;
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

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, mode: Option<usize>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Ardoz and Primal Forcemage pump each creature entering under you; Ardoz
/// counts itself, the Forcemage only others.
#[test]
fn entering_creatures_get_pumped() {
    let mut g = pod(2);
    let ardoz = g.add_card_to_hand(0, catalog::ardoz_cobbler_of_war());
    cast(&mut g, ardoz, None, None).expect("ardoz");
    assert_eq!(g.computed_permanent(ardoz).unwrap().power, 1 + 2);
    g.add_card_to_battlefield(0, catalog::primal_forcemage());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None, None).expect("bear");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (2 + 2 + 3, 2 + 3));
}

/// CR 508.4 — Stampede Surfer's attack puts a Boar into combat against each
/// opponent.
#[test]
fn cr_508_4_stampede_surfer_sends_a_boar_at_each_opponent() {
    let mut g = pod(4);
    let surfer = g.add_card_to_battlefield(0, catalog::stampede_surfer());
    g.clear_sickness(surfer);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: surfer, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let boars: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Boar").map(|c| c.id).collect();
    assert_eq!(boars.len(), 3);
    let mut defenders: Vec<_> = g
        .attacking
        .iter()
        .filter(|a| boars.contains(&a.attacker))
        .map(|a| a.target)
        .collect();
    defenders.sort_by_key(|t| format!("{t:?}"));
    defenders.dedup();
    assert_eq!(defenders.len(), 3, "one Boar per opponent");
}

/// Warmonger's Chariot lets a Wall attack; Towering Viewpoint grants flying.
#[test]
fn a_chariot_wall_attacks() {
    let mut g = pod(2);
    let gate = g.add_card_to_battlefield(0, catalog::consulate_skygate());
    g.clear_sickness(gate);
    let chariot = g.add_card_to_battlefield(0, catalog::warmongers_chariot());
    flood(&mut g);
    g.perform_action(GameAction::Equip { equipment: chariot, target: gate }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(gate).unwrap().power, 2);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: gate, target: AttackTarget::Player(1) }]))
        .expect("a defender with the Chariot attacks");
    let view = g.add_card_to_battlefield(0, catalog::towering_viewpoint());
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, view, 0, Some(Target::Permanent(gate))).expect("leap of faith");
    assert!(g.computed_permanent(gate).unwrap().keywords().contains(&Keyword::Flying));
}

/// Wakeroot Elemental wakes a land for good; Silvanus's Invoker only this
/// turn — both stay lands.
#[test]
fn awakened_lands_stay_lands() {
    let mut g = pod(2);
    let wakeroot = g.add_card_to_battlefield(0, catalog::wakeroot_elemental());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    activate(&mut g, wakeroot, 0, Some(Target::Permanent(land))).expect("awaken");
    let l = g.battlefield_find(land).unwrap();
    assert!(!l.tapped);
    let cp = g.computed_permanent(land).unwrap();
    assert!(cp.card_types().contains(&CardType::Creature) && cp.card_types().contains(&CardType::Land));
    assert_eq!((cp.power, cp.toughness), (5, 5));
    let invoker = g.add_card_to_battlefield(0, catalog::silvanuss_invoker());
    let other = g.add_card_to_battlefield(0, catalog::forest());
    activate(&mut g, invoker, 0, Some(Target::Permanent(other))).expect("conjure");
    let cp = g.computed_permanent(other).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8));
    assert!(cp.keywords().contains(&Keyword::Trample));
}

/// You Happen On a Glade's Make Camp returns a permanent card.
#[test]
fn glade_make_camp_returns_a_permanent_card() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let glade = g.add_card_to_hand(0, catalog::you_happen_on_a_glade());
    cast(&mut g, glade, Some(Target::Permanent(bear)), Some(1)).expect("glade");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}
