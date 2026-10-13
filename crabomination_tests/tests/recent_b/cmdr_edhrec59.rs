//! Commander: EDHREC ranks 2001-2500, one-card gaps, fifth batch
//! (`decks::cmdr_edhrec59`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn fill_pool(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
    g.priority.player_with_priority = 0;
}

fn cast_at(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    fill_pool(g);
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let atks = attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect();
    g.perform_action(GameAction::DeclareAttackers(atks)).expect("attack");
    drain_stack(g);
}

fn combat_damage(g: &mut GameState) {
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

/// Ancestral Katana hops onto a Warrior attacking alone for {1}.
#[test]
fn ancestral_katana_follows_a_lone_warrior() {
    let mut g = pod(2);
    let katana = ready(&mut g, 0, catalog::ancestral_katana());
    let warrior = ready(&mut g, 0, catalog::honored_crop_captain());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add_colorless(1);
    attack(&mut g, &[warrior]);
    assert_eq!(g.battlefield_find(katana).unwrap().attached_to, Some(warrior));
    assert_eq!(g.computed_permanent(warrior).unwrap().power, 5);
}

/// Myojin of Towering Might enters indestructible when cast, and spends the
/// counter on eight +1/+1 counters.
#[test]
fn myojin_of_towering_might_distributes_eight() {
    let mut g = pod(2);
    let myojin = g.add_card_to_hand(0, catalog::myojin_of_towering_might());
    cast_at(&mut g, myojin, None).expect("cast");
    assert_eq!(g.battlefield_find(myojin).unwrap().counter_count(CounterType::Indestructible), 1);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    fill_pool(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: myojin,
        ability_index: 0,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("remove the counter");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 8);
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Trample));
}

/// Ghostly Touch's attacker may tap a blocker-to-be.
#[test]
fn ghostly_touch_taps_on_attack() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let touch = g.add_card_to_hand(0, catalog::ghostly_touch());
    cast_at(&mut g, touch, Some(Target::Permanent(bears))).expect("aura");
    let wall = ready(&mut g, 1, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true), DecisionAnswer::Target(Target::Permanent(wall))]));
    attack(&mut g, &[bears]);
    assert!(g.battlefield_find(wall).unwrap().tapped);
}

/// Ogre-Head Helm: when the equipped creature connects, sacrifice the Helm to
/// wheel for three.
#[test]
fn ogre_head_helm_wheels_on_damage() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let helm = ready(&mut g, 0, catalog::ogre_head_helm());
    g.add_card_to_hand(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    attack(&mut g, &[helm]);
    combat_damage(&mut g);
    assert!(g.battlefield_find(helm).is_none(), "sacrificed");
    assert_eq!(g.players[0].hand.len(), 3, "discarded one, drew three");
}

/// Torgaar sets a player's life to half their starting life.
#[test]
fn torgaar_halves_a_life_total() {
    let mut g = pod(2);
    let start = g.players[1].life;
    let torgaar = g.add_card_to_hand(0, catalog::torgaar_famine_incarnate());
    cast_at(&mut g, torgaar, Some(Target::Player(1))).expect("cast");
    assert_eq!(g.players[1].life, start / 2);
}

/// Jace's −2 bounces a creature to its owner's hand.
#[test]
fn jace_unraveler_bounces() {
    let mut g = pod(2);
    let jace = ready(&mut g, 0, catalog::jace_unraveler_of_secrets());
    let maw = ready(&mut g, 1, catalog::colossal_dreadmaw());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: jace,
        ability_index: 1,
        target: Some(Target::Permanent(maw)),
        x_value: None,
    })
    .expect("-2");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == maw));
}

/// Vengeful Pharaoh answers combat damage from the graveyard: it destroys an
/// attacker and goes back on top of the library.
#[test]
fn vengeful_pharaoh_strikes_from_the_grave() {
    let mut g = pod(2);
    let pharaoh = g.add_card_to_graveyard(1, catalog::vengeful_pharaoh());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    attack(&mut g, &[bears]);
    combat_damage(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "the attacker was destroyed");
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(pharaoh));
}
