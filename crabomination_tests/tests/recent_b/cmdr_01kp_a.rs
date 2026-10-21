//! Commander: the 2026-10-21 (`01KPxZ3H`) single-commander closers
//! (`decks::cmdr_01kp_a`).

use crabomination::card::{ArtifactSubtype, CardDefinition, CardId, CardType, Keyword};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x_value: Option<u32>) {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value })
        .expect("cast");
    drain_stack(g);
}

fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn attack(g: &mut GameState, seat: usize, attacks: &[(CardId, usize)]) {
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.iter().map(|&(a, d)| Attack { attacker: a, target: AttackTarget::Player(d) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

/// CR 508.1 + 603.4 — Squall triggers when anyone's creatures attack one of
/// its controller's opponents (once per opponent attacked), only if one of
/// that opponent's attackers has power or toughness equal to the chosen
/// number; an attack on Squall's controller never triggers it.
#[test]
fn cr_603_4_squall_hits_each_attacked_opponent_with_a_matching_attacker() {
    let mut g = pod(4);
    let squall = g.add_card_to_hand(0, catalog::squall_gunblade_duelist());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    cast_x(&mut g, 0, squall, None, None);
    assert_eq!(g.battlefield_find(squall).and_then(|c| c.chosen_number), Some(2), "the commonest P/T it owns");
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let bears2 = ready(&mut g, 1, catalog::grizzly_bears());
    let life = [g.players[0].life, g.players[2].life, g.players[3].life];
    // Seat 1 attacks seat 2 with a 2/2 (a match), seat 3 with a 3/3 (none) and
    // seat 0 — Squall's controller — with a 2/2.
    attack(&mut g, 1, &[(bears, 2), (giant, 3), (bears2, 0)]);
    assert_eq!(g.players[2].life, life[1] - 3, "a 2/2 attacked seat 2");
    assert_eq!(g.players[3].life, life[2], "a 3/3 matches neither half");
    assert_eq!(g.players[0].life, life[0], "an attack on Squall's controller doesn't trigger it");
}

/// Duelist's Flame needs a blocked creature (CR 509.1h); with X = 2 the
/// trampling Hill Giant puts 4 past a 2/1 blocker, and the granted trigger
/// looks at that many cards, exiles Divination and casts it free.
#[test]
fn duelists_flame_pumps_a_blocked_creature_and_casts_off_its_damage() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let div = g.add_card_to_library(0, catalog::divination());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let lions = g.add_card_to_battlefield(1, catalog::savannah_lions());
    let flame = g.add_card_to_hand(0, catalog::duelists_flame());
    attack(&mut g, 0, &[(giant, 1)]);
    flood(&mut g, 0);
    let early = GameAction::CastSpell {
        card_id: flame,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    };
    assert!(g.perform_action(early).is_err(), "an unblocked attacker isn't a legal target");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(lions, giant)])).expect("block");
    drain_stack(&mut g);
    cast_x(&mut g, 0, flame, Some(Target::Permanent(giant)), Some(2));
    let c = g.battlefield_find(giant).unwrap();
    assert_eq!(c.power(), 5);
    assert!(g.permanent_has_keyword(giant, &Keyword::Trample));
    let (life, hand) = (g.players[1].life, g.players[0].hand.len());
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, life - 4, "trample past the 2/1");
    assert_eq!(g.players[0].hand.len(), hand + 2, "Divination was cast free");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == div));
    assert_eq!(g.players[0].library.len(), 1, "two drawn, the rest bottomed");
}

fn rex_trigger(g: &mut GameState, rex: CardId, target: CardId) {
    let fx = catalog::captain_rex_nebula().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(rex, 0, Some(Target::Permanent(target)), 0);
    let evs = g.resolve_effect(&fx, &ctx).expect("rex");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// Captain Rex Nebula (2022 ruling): the target loses its card types and
/// creature types, becomes a Vehicle artifact with base P/T its mana value and
/// crew 2; crewing it adds Creature but no creature types back; it all ends at
/// cleanup (CR 514.2).
#[test]
fn rex_nebula_turns_a_permanent_into_a_crewable_mana_value_vehicle() {
    let mut g = pod(2);
    let rex = ready(&mut g, 0, catalog::captain_rex_nebula());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    rex_trigger(&mut g, rex, giant);
    let cp = g.computed_permanent(giant).unwrap();
    assert_eq!(cp.card_types(), &[CardType::Artifact], "no longer a creature");
    assert_eq!(cp.subtypes().artifact_subtypes, vec![ArtifactSubtype::Vehicle]);
    assert!(cp.subtypes().creature_types.is_empty(), "loses Giant");
    g.perform_action(GameAction::Crew { vehicle: giant, crew_creatures: vec![rex] }).expect("crew 2");
    let cp = g.computed_permanent(giant).unwrap();
    assert!(cp.card_types().contains(&CardType::Creature) && cp.card_types().contains(&CardType::Artifact));
    assert!(cp.subtypes().creature_types.is_empty(), "crewing doesn't bring Giant back");
    assert_eq!((cp.power, cp.toughness), (4, 4), "base P/T = mana value 4");
    g.do_cleanup(&mut vec![]);
    let cp = g.computed_permanent(giant).unwrap();
    assert_eq!(cp.card_types(), &[CardType::Creature]);
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Crash Land: the Vehicle deals combat damage, the d6 comes up equal to its
/// mana value, so it is sacrificed and then deals that much damage.
#[test]
fn rex_nebula_crash_land_sacrifices_on_its_mana_value() {
    let mut g = pod(2);
    let rex = ready(&mut g, 0, catalog::captain_rex_nebula());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    rex_trigger(&mut g, rex, bears);
    g.perform_action(GameAction::Crew { vehicle: bears, crew_creatures: vec![rex] }).expect("crew 2");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(2)]));
    let life = g.players[1].life;
    attack(&mut g, 0, &[(bears, 1)]);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bears), "sacrificed on a 2");
    assert_eq!(g.players[1].life, life - 4, "2 combat + 2 crash");
}

/// Mysterious Limousine: each entry/attack exiles a creature until it leaves
/// (CR 610.3) and swaps the previous one back; when it leaves, the last one
/// returns too.
#[test]
fn mysterious_limousine_swaps_its_prisoner_and_frees_it_on_leaving() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let limo = g.add_card_to_hand(0, catalog::mysterious_limousine());
    cast_x(&mut g, 0, limo, None, None);
    assert!(g.exile.iter().any(|c| c.id == bears), "the ETB took the only other creature");
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let fx = catalog::mysterious_limousine().triggered_abilities[1].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(limo, 0, Some(Target::Permanent(giant)), 0);
    g.resolve_effect(&fx, &ctx).expect("swap");
    assert!(g.exile.iter().any(|c| c.id == giant));
    assert!(g.battlefield_find(bears).is_some(), "the old prisoner came back");
    let destroy = crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::ExactObjects(vec![limo]) };
    g.resolve_effect(&destroy, &crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0)).expect("destroy");
    assert!(g.battlefield_find(giant).is_some(), "freed when the Limousine left");
}
