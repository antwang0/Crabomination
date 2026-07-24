//! CR conformance for rules exercised by this run's WAR batch-9 wave:
//! CR 509.1c (Menace — a menacing attacker can't be blocked by exactly one
//! creature; here menace is *granted* by Angrath's anthem), CR 701.43e (Amass —
//! amassing again grows the same Army rather than minting a second), and
//! CR 120.10 (excess damage over lethal is tracked for a resolution).

use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Value};
use crabomination::game::effects::{EffectContext, EntityRef};
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// CR 509.1c — a creature granted menace (by Angrath's anthem) can't be legally
/// blocked by a single creature, but two blockers is fine.
#[test]
fn cr_509_1c_granted_menace_requires_two_blockers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::angrath_captain_of_chaos());
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // gains menace from Angrath
    assert!(g.computed_permanent(atk).unwrap().keywords.contains(&Keyword::Menace));
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(b1, atk)])).is_err(), "one blocker is illegal against menace");
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(b1, atk), (b2, atk)])).is_ok(), "two blockers is legal");
}

/// CR 701.43e — if you already control an Army, Amass adds counters to it
/// instead of minting a second Army; the Army also gains the amassed type.
#[test]
fn cr_701_43e_amass_grows_existing_army() {
    let mut g = two_player_game();
    let ctx = EffectContext::for_ability(g.add_card_to_battlefield(0, catalog::grizzly_bears()), 0, None);
    let amass2 = Effect::Amass { who: PlayerRef::You, count: Value::Const(2), extra_type: Some(CreatureType::Zombie) };
    g.resolve_effect(&amass2, &ctx).unwrap();
    g.resolve_effect(&Effect::Amass { who: PlayerRef::You, count: Value::ONE, extra_type: Some(CreatureType::Zombie) }, &ctx).unwrap();
    let armies: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army))
        .collect();
    assert_eq!(armies.len(), 1, "a single Army, grown twice");
    assert_eq!(armies[0].counter_count(CounterType::PlusOnePlusOne), 3, "2 + 1 counters on the same Army");
    assert!(armies[0].definition.subtypes.creature_types.contains(&CreatureType::Zombie), "Army is also a Zombie");
}

/// CR 120.10 — damage dealt to a creature beyond what's lethal is "excess"; the
/// engine tracks it for the current resolution.
#[test]
fn cr_120_10_excess_damage_tracked() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.excess_damage_this_resolution = 0;
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(bear), 5, None, &mut evs);
    // 5 dealt, 2 lethal → 3 excess.
    assert_eq!(g.excess_damage_this_resolution, 3, "excess over lethal is tracked");
}
