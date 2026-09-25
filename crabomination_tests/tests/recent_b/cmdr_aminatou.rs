//! Commander: the Subjective Reality precon (C18, Aminatou,
//! `decks::cmdr_aminatou`). The primitives it needed are tested in
//! `core_rules/commander_cards.rs`.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod() -> GameState {
    let mut g = multi_player_game(3);
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
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
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    if let Some(c) = g.battlefield_find_mut(id) {
        c.loyalty_uses_this_turn = 0;
    }
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: None })
        .expect("loyalty");
    drain_stack(g);
}

fn connect(g: &mut GameState, attacker: CardId) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    g.step = TurnStep::PostCombatMain;
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

/// CR 606 — Aminatou's +1 draws and puts one back; −1 blinks a permanent
/// of yours.
#[test]
fn aminatou_filters_and_blinks() {
    let mut g = pod();
    library(&mut g, 0, 3);
    let a = g.add_card_to_battlefield(0, catalog::aminatou_the_fateshifter());
    let hand = g.players[0].hand.len();
    loyalty(&mut g, a, 0, None);
    assert_eq!(g.players[0].hand.len(), hand, "draw one, put one back");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    loyalty(&mut g, a, 1, Some(Target::Permanent(bear)));
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("returned");
    assert!(!back.tapped && back.controller == 0);
}

/// CR 701.34 — Aminatou's Augury: a land and one free spell per card type.
#[test]
fn aminatous_augury_offers_one_per_type() {
    let mut g = pod();
    for card in [catalog::forest(), catalog::grizzly_bears(), catalog::hill_giant(), catalog::sol_ring()] {
        g.add_card_to_library(0, card);
    }
    library(&mut g, 0, 4);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let aug = g.add_card_to_hand(0, catalog::aminatous_augury());
    cast(&mut g, aug, None).expect("cast");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Forest"));
    let free = g.exile.iter().filter(|c| c.may_play_until.is_some()).count();
    assert_eq!(free, 2, "one creature, one artifact");
}

/// CR 702.94 — Banishing Stroke bottoms a creature.
#[test]
fn banishing_stroke_bottoms() {
    let mut g = pod();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bs = g.add_card_to_hand(0, catalog::banishing_stroke());
    cast(&mut g, bs, Some(Target::Permanent(bear))).expect("cast");
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(bear));
}

/// CR 701.34 — Cloudform and Lightform manifest and enchant their 2/2.
#[test]
fn manifest_auras() {
    let mut g = pod();
    library(&mut g, 0, 2);
    let cf = g.add_card_to_hand(0, catalog::cloudform());
    cast(&mut g, cf, None).expect("cast");
    let host = g.battlefield_find(cf).and_then(|c| c.attached_to).expect("attached");
    assert!(g.battlefield_find(host).is_some_and(|c| c.face_down));
    assert!(g.permanent_has_keyword(host, &Keyword::Hexproof));
    // CR 704.5m — it *became* an Aura: its host dying takes it to the
    // graveyard, where it's the printed plain enchantment again.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(host))).expect("bolt the 2/2");
    let gone = g.players[0].graveyard.iter().find(|c| c.id == cf).expect("to the graveyard");
    assert!(!gone.definition.is_aura());
    let lf = g.add_card_to_hand(0, catalog::lightform());
    cast(&mut g, lf, None).expect("cast");
    let host = g.battlefield_find(lf).and_then(|c| c.attached_to).expect("attached");
    assert!(g.permanent_has_keyword(host, &Keyword::Lifelink));
}

/// Djinn of Wishes plays the top card free, or exiles it.
#[test]
fn djinn_of_wishes_wishes() {
    let mut g = pod();
    let d = g.add_card_to_battlefield(0, catalog::djinn_of_wishes());
    g.battlefield_find_mut(d).unwrap().counters.insert(CounterType::Wish, 3);
    g.add_card_to_library(0, catalog::sol_ring());
    activate(&mut g, d, 0, None).expect("wish");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Sol Ring"));
    g.add_card_to_library(0, catalog::forest());
    activate(&mut g, d, 0, None).expect("wish");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Forest"));
    g.add_card_to_library(0, catalog::forest());
    activate(&mut g, d, 0, None).expect("wish");
    assert!(g.exile.iter().any(|c| c.definition.name == "Forest"), "no land drop left");
}

/// CR 400.7 — Enigma Sphinx goes third from the top when it dies.
#[test]
fn enigma_sphinx_returns_third() {
    let mut g = pod();
    library(&mut g, 0, 4);
    let s = g.add_card_to_battlefield(0, catalog::enigma_sphinx());
    let wrath = g.add_card_to_hand(0, catalog::wrath_of_god());
    cast(&mut g, wrath, None).expect("wrath");
    assert_eq!(g.players[0].library.get(2).map(|c| c.id), Some(s));
}

/// CR 107.3 — Entreat the Dead returns X creature cards.
#[test]
fn entreat_the_dead_returns_x() {
    let mut g = pod();
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::hill_giant());
    let e = g.add_card_to_hand(0, catalog::entreat_the_dead());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: e,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("X = 2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some());
}

/// Isolated Watchtower only works two lands behind.
#[test]
fn isolated_watchtower_needs_to_be_behind() {
    let mut g = pod();
    let t = g.add_card_to_battlefield(0, catalog::isolated_watchtower());
    g.add_card_to_library(0, catalog::plains());
    assert!(activate(&mut g, t, 1, None).is_err(), "not behind");
    g.battlefield_find_mut(t).unwrap().tapped = false;
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::forest());
    }
    activate(&mut g, t, 1, None).expect("behind by two");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Plains" && c.tapped));
}

/// CR 701.34 — Jeskai Infiltrator re-manifests itself and the top card.
#[test]
fn jeskai_infiltrator_hides() {
    let mut g = pod();
    library(&mut g, 0, 2);
    let j = g.add_card_to_battlefield(0, catalog::jeskai_infiltrator());
    assert!(g.permanent_has_keyword(j, &Keyword::Unblockable), "alone");
    connect(&mut g, j);
    let face_down = g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).count();
    assert_eq!(face_down, 2);
}

/// CR 700.2 — Magus of the Balance evens out lands.
#[test]
fn magus_of_the_balance_balances() {
    let mut g = pod();
    let m = g.add_card_to_battlefield(0, catalog::magus_of_the_balance());
    g.clear_sickness(m);
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::forest());
    }
    g.add_card_to_battlefield(0, catalog::plains());
    activate(&mut g, m, 0, None).expect("balance");
    let lands = |s| g.battlefield.iter().filter(|c| c.controller == s && c.definition.is_land()).count();
    assert_eq!((lands(0), lands(1), lands(2)), (0, 0, 0), "seat 2 has none");
}

/// CR 702.74 — Night Incarnate's evoke sweeps -3/-3.
#[test]
fn night_incarnate_evoked() {
    let mut g = pod();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let n = g.add_card_to_hand(0, catalog::night_incarnate());
    flood(&mut g);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: n,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("evoke");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// Portent orders a library and draws at the next upkeep.
#[test]
fn portent_draws_later() {
    let mut g = pod();
    library(&mut g, 0, 4);
    let p = g.add_card_to_hand(0, catalog::portent());
    cast(&mut g, p, Some(Target::Player(0))).expect("cast");
    let hand = g.players[0].hand.len();
    g.turn_number += 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// CR 701.34 — Primordial Mist manifests; a face-down card may be played.
#[test]
fn primordial_mist_manifests() {
    let mut g = pod();
    library(&mut g, 0, 1);
    g.add_card_to_library(0, catalog::sol_ring());
    let pm = g.add_card_to_battlefield(0, catalog::primordial_mist());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let fd = g.battlefield.iter().find(|c| c.face_down).expect("manifested").id;
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, pm, 0, Some(Target::Permanent(fd))).expect("exile it");
    assert!(g.exile.iter().any(|c| c.id == fd && c.may_play_until.is_some()));
}

/// CR 702.49 — Silent-Blade Oni casts from the defender's hand.
#[test]
fn silent_blade_oni_steals() {
    let mut g = pod();
    let o = g.add_card_to_battlefield(0, catalog::silent_blade_oni());
    g.add_card_to_hand(1, catalog::sol_ring());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    connect(&mut g, o);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Sol Ring"));
}

/// CR 903.8 — Skull Storm copies per commander cast; creatureless opponents
/// lose half their life.
#[test]
fn skull_storm_storms() {
    let mut g = pod();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[2].life;
    let s = g.add_card_to_hand(0, catalog::skull_storm());
    cast(&mut g, s, None).expect("cast");
    assert!(g.battlefield.iter().all(|c| c.controller != 1));
    assert_eq!(g.players[2].life, life - life / 2);
}

/// Varina loots per attacking Zombie and makes tapped Zombies.
#[test]
fn varina_loots() {
    let mut g = pod();
    library(&mut g, 0, 4);
    let v = g.add_card_to_battlefield(0, catalog::varina_lich_queen());
    let life = g.players[0].life;
    connect(&mut g, v);
    assert_eq!(g.players[0].life, life + 1);
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    activate(&mut g, v, 0, None).expect("exile two");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Zombie" && c.tapped));
}

/// Yennett casts an odd top card free, or draws.
#[test]
fn yennett_reveals() {
    let mut g = pod();
    let y = g.add_card_to_battlefield(0, catalog::yennett_cryptic_sovereign());
    g.add_card_to_library(0, catalog::sol_ring());
    connect(&mut g, y);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Sol Ring"), "mana value 1");
    let hand = g.players[0].hand.len();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let y2 = g.battlefield_find_mut(y).unwrap();
    y2.tapped = false;
    connect(&mut g, y);
    assert_eq!(g.players[0].hand.len(), hand + 1, "a two-drop is drawn");
}
