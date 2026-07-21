//! CR conformance for rules exercised by the GPT gap wave 4: CR 704.5j (the
//! legend rule reads *current* supertypes, so a continuous Legendary grant
//! collapses duplicates — Leyline of Singularity), CR 701.15b (a goaded
//! creature must attack a player other than its goader if able), and CR 611.2c
//! ("during your turn" anthems switch off outside the controller's turn).

use crabomination::card::{
    CardDefinition, CardType, SelectionRequirement as R, StaticAbility,
};
use crabomination::catalog;
use crabomination::effect::StaticEffect;
use crabomination::game::types::AttackTarget;
use crabomination::game::{drain_stack, multi_player_game, two_player_game, Attack, GameAction};
use crabomination::game::{GameState, TurnStep};

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// CR 704.5j — Leyline of Singularity grants the Legendary supertype to every
/// nonland permanent (layer 4). Two same-named creatures then violate the
/// legend rule and one is put into the graveyard.
#[test]
fn cr_704_5j_continuous_legendary_grant_collapses_duplicates() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leyline_of_singularity());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    let bears = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Grizzly Bears" && c.controller == 0)
        .count();
    assert_eq!(bears, 1, "the legend rule collapsed the duplicate");
    // Removing the Leyline drops the supertype grant; a second copy is legal.
    g.battlefield.retain(|c| c.definition.name != "Leyline of Singularity");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    let bears = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Grizzly Bears" && c.controller == 0)
        .count();
    assert_eq!(bears, 2, "without the grant the duplicates coexist");
}

/// CR 701.15b — a creature goaded by P1 must attack a player other than P1 when
/// it's able to. Attacking the goader while a non-goader opponent (P2) is
/// available is illegal; attacking the non-goader is legal.
#[test]
fn cr_701_15b_goaded_must_attack_non_goader() {
    let mut g = multi_player_game(3);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.battlefield_find_mut(bear).unwrap().goaded_by = vec![1]; // goaded by P1
    advance_to(&mut g, TurnStep::DeclareAttackers);
    // Attacking the goader (P1) is illegal while P2 is a valid alternative.
    assert!(
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
            .is_err(),
        "can't attack the goader when another opponent is available"
    );
    // Attacking the non-goader (P2) satisfies the requirement.
    assert!(
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(2) }])
            .is_ok(),
        "attacking the non-goader is legal"
    );
}

/// CR 611.2c — a "during your turn" anthem is live only while its controller is
/// the active player.
#[test]
fn cr_611_2c_during_your_turn_anthem_turn_gated() {
    fn turn_gated_lord() -> CardDefinition {
        CardDefinition {
            name: "Turn Lord",
            card_types: vec![CardType::Enchantment],
            static_abilities: vec![StaticAbility {
                description: "During your turn, creatures you control get +2/+2.",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::Creature,
                    power: 2,
                    toughness: 2,
                    keywords: vec![],
                    opponents: false,
                    only_your_turn: true,
                    scale_by_counters_on_self: None,
                },
            }],
            ..Default::default()
        }
    }
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, turn_gated_lord());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    // Player 0's turn (active) — anthem live.
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+2 on your turn");
    // Advance to player 1's turn — anthem off.
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    g.perform_action(GameAction::PassPriority).ok();
    while g.active_player_idx == 0 {
        g.perform_action(GameAction::PassPriority).expect("pass to next turn");
    }
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "no bonus off your turn");
}
