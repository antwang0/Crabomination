//! Commander: the Lorehold Legacies precon (C21, Osgir,
//! `decks::cmdr_lorehold`) and the primitives it needed.

use crabomination::card::{CardDefinition, CardId, CardType, SelectionRequirement as R, StaticAbility};
use crabomination::catalog;
use crabomination::effect::{Effect, StaticEffect};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// Attack with `attacker` at seat 1, have `blocker` block it, and run combat.
fn blocked_combat(g: &mut GameState, attacker: CardId, blocker: CardId) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// CR 615 — "prevent all combat damage that would be dealt to attacking
/// artifact creatures you control": the blocked attacker survives, the
/// blocker still takes its damage, and an opponent's artifact isn't covered.
#[test]
fn cr_615_a_filtered_shield_prevents_combat_damage_to_its_matches() {
    let shield = || CardDefinition {
        name: "Test Shield",
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "prevent combat damage to attacking artifact creatures you control",
            effect: StaticEffect::PreventAllCombatDamageToMatching {
                filter: R::Artifact.and(R::Creature).and(R::IsAttacking).and(R::ControlledByYou),
            },
        }],
        ..Default::default()
    };
    let mut g = pod(2);
    g.add_card_to_battlefield(0, shield());
    let sable = g.add_card_to_battlefield(0, catalog::bronze_sable());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    blocked_combat(&mut g, sable, bear);
    assert!(g.battlefield_find(sable).is_some(), "no damage to the attacking artifact");
    assert!(g.battlefield_find(bear).is_none(), "the blocker still dies");

    let mut g = pod(2);
    g.add_card_to_battlefield(1, shield());
    let sable = g.add_card_to_battlefield(0, catalog::bronze_sable());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    blocked_combat(&mut g, sable, bear);
    assert!(g.battlefield_find(sable).is_none(), "an opponent's shield doesn't cover it");
}

/// Reveal until an artifact: it enters, the misses go to the bottom, and the
/// controller takes damage equal to every card revealed.
#[test]
fn reveal_until_an_artifact_bottoms_the_rest_and_hurts_you() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::bronze_sable());
    g.add_card_to_library(0, catalog::island());
    let top: Vec<&str> = g.players[0].library.iter().map(|c| c.definition.name).collect();
    assert_eq!(top[0], "Grizzly Bears", "add_card_to_library puts cards in order: {top:?}");
    let life = g.players[0].life;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::RevealUntilOneToBattlefieldRestBottom { filter: R::Artifact, damage_controller: true },
        &ctx,
    )
    .expect("resolve");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Bronze Sable"));
    assert_eq!(g.players[0].life, life - 3);
    let order: Vec<&str> = g.players[0].library.iter().map(|c| c.definition.name).collect();
    assert_eq!(order, vec!["Island", "Grizzly Bears", "Grizzly Bears"], "the misses went to the bottom");
}
