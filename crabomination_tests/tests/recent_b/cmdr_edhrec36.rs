//! Commander: Melek's, Trazyn's, O-Kagachi's, Orah's and Meria's last gaps
//! (`decks::cmdr_edhrec36`).

use crabomination::card::{CardId, CardType};
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
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g);
    act(g, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), String> {
    act(g, GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn kill(g: &mut GameState, id: CardId) {
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(g, murder, &[Target::Permanent(id)]).expect("murder");
}

/// CR 603.10a — Orah: a destroyed Cleric, Orah herself or another, brings
/// back a Cleric card of lesser mana value. The destroy funnel bound no event
/// amount for Orah's own death, so no card was ever a legal target.
#[test]
fn orah_returns_a_smaller_cleric() {
    let mut g = pod(2);
    let orah = ready(&mut g, 0, catalog::orah_skyclave_hierophant());
    let bond = ready(&mut g, 0, catalog::cleric_of_lifes_bond());
    let one = g.add_card_to_graveyard(0, catalog::souls_attendant());
    kill(&mut g, bond);
    assert!(g.battlefield_find(one).is_some(), "mana value 1 < 2");
    let big = g.add_card_to_graveyard(0, catalog::orah_skyclave_hierophant());
    kill(&mut g, orah);
    assert!(g.battlefield_find(bond).is_some(), "Orah's own death returns the 2");
    assert!(g.battlefield_find(big).is_none(), "mana value 4 isn't lesser");
}

/// Kura's death mode two: an X/X Spirit, X = lands you control.
#[test]
fn kura_leaves_a_spirit_sized_by_your_lands() {
    let mut g = pod(2);
    let kura = ready(&mut g, 0, catalog::kura_the_boundless_sky());
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    kill(&mut g, kura);
    let spirit = g.battlefield.iter().find(|c| c.definition.name == "Spirit").map(|c| c.id).expect("spirit");
    let cp = g.computed_permanent(spirit).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Pyre of Heroes: sacrifice a creature, fetch one that shares a creature
/// type with it at one more mana value.
#[test]
fn pyre_of_heroes_climbs_the_curve_by_type() {
    let mut g = pod(2);
    let pyre = ready(&mut g, 0, catalog::pyre_of_heroes());
    let fodder = ready(&mut g, 0, catalog::cleric_of_lifes_bond());
    let wrong = g.add_card_to_library(0, catalog::hill_giant());
    let right = g.add_card_to_library(0, catalog::vampire_nighthawk());
    flood(&mut g);
    activate(&mut g, pyre, 0, None).expect("pyre");
    assert!(g.battlefield_find(fodder).is_none());
    assert!(g.battlefield_find(right).is_some(), "a 3-mana Vampire");
    assert!(g.battlefield_find(wrong).is_none());
}

/// Meria taps two nontoken artifacts to exile the top card, playable this
/// turn.
#[test]
fn meria_turns_artifacts_into_cards() {
    let mut g = pod(2);
    let meria = ready(&mut g, 0, catalog::meria_scholar_of_antiquity());
    ready(&mut g, 0, catalog::herbal_poultice());
    ready(&mut g, 0, catalog::sol_ring());
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    activate(&mut g, meria, 1, None).expect("tap two artifacts");
    let c = g.exile.iter().find(|c| c.id == top).expect("exiled");
    assert!(c.may_play_until.is_some());
    assert!(activate(&mut g, meria, 0, None).is_err(), "no untapped artifact left");
}

/// Traxos doesn't untap in your untap step; an artifact or creature spell
/// untaps it.
#[test]
fn traxos_untaps_on_artifact_or_creature_spells() {
    let mut g = pod(2);
    let traxos = ready(&mut g, 0, catalog::traxos_scourge_eternal());
    g.battlefield_find_mut(traxos).unwrap().tapped = true;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]).expect("bolt");
    assert!(g.battlefield_find(traxos).unwrap().tapped, "an instant doesn't untap it");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, &[]).expect("bear");
    assert!(!g.battlefield_find(traxos).unwrap().tapped);
}

/// Gravitic Punch: your creature hits a player for its power; Immolating Gyre
/// burns only what you don't control, X = instants and sorceries in your
/// graveyard.
#[test]
fn gravitic_punch_and_immolating_gyre() {
    let mut g = pod(3);
    let wurm = ready(&mut g, 0, catalog::craw_wurm());
    let punch = g.add_card_to_hand(0, catalog::gravitic_punch());
    cast(&mut g, punch, &[Target::Permanent(wurm), Target::Player(2)]).expect("punch");
    assert_eq!(g.players[2].life, 20 - 6);
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    for _ in 0..1 {
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
    }
    let gyre = g.add_card_to_hand(0, catalog::immolating_gyre());
    cast(&mut g, gyre, &[]).expect("gyre");
    assert!(g.battlefield_find(theirs).is_none(), "punch + bolt in the graveyard: 2 damage");
    assert!(g.battlefield_find(wurm).is_some());
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.card_types.contains(&CardType::Sorcery)));
}

/// CR 611.2b — Kyodai's indestructible lasts "for as long as you control
/// Kyodai": it ends when an opponent takes Kyodai (it lasted while Kyodai
/// stayed on the battlefield).
#[test]
fn kyodai_shield_ends_when_kyodai_changes_control() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let kyodai = g.add_card_to_hand(0, catalog::kyodai_soul_of_kamigawa());
    cast(&mut g, kyodai, &[]).expect("Kyodai");
    let has = |g: &GameState| g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Indestructible);
    assert!(has(&g));
    g.battlefield_find_mut(kyodai).unwrap().controller = 1;
    let _ = g.check_state_based_actions();
    assert!(!has(&g), "an opponent controls Kyodai");
}
