//! CR conformance for the modern_decks batch's engine work:
//! - CR 508.1a — a team-wide "creatures you control can attack as though they
//!   didn't have defender" static lets a Wall attack (High Alert).
//! - CR 510.1c — a creature that "assigns combat damage equal to its toughness"
//!   deals toughness, not power, to the defending player (High Alert).
//! - CR 701.15b — a milled card is put into a graveyard, so "whenever a card is
//!   put into a graveyard from anywhere" triggers fire on mills (The Haunt of
//!   Hightower).

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

/// CR 508.1a + 510.1c — under High Alert, a 0/4 defender Wall can attack and
/// deals damage equal to its toughness (4) to the defending player.
#[test]
fn cr_508_510_high_alert_wall_attacks_for_toughness() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::high_alert());
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_lost_thoughts()); // 0/4 Defender
    g.clear_sickness(wall);
    assert!(
        g.computed_permanent(wall).unwrap().keywords.contains(&Keyword::AssignsCombatDamageByToughness),
        "High Alert grants toughness-damage to your creatures"
    );
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: wall, target: AttackTarget::Player(1) }]))
        .expect("Wall attacks despite Defender under High Alert");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert_eq!(g.players[1].life, 16, "0/4 Wall dealt 4 (its toughness), not 0");
}

/// CR 508.1a — without the team static, the same Wall can't be declared as an
/// attacker (the control on the feature above).
#[test]
fn cr_508_1a_defender_cant_attack_without_grant() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_lost_thoughts());
    g.clear_sickness(wall);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: wall, target: AttackTarget::Player(1) }])).is_err(),
        "a Defender can't attack without a permission-granting effect"
    );
}

/// CR 701.15b — a milled card is "put into a graveyard from a library", firing
/// The Haunt of Hightower's "whenever a card is put into an opponent's
/// graveyard from anywhere" counter trigger.
#[test]
fn cr_701_15b_mill_fires_put_into_graveyard_triggers() {
    let mut g = two_player_game();
    let haunt = g.add_card_to_battlefield(0, catalog::the_haunt_of_hightower());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(
            &crabomination::effect::Effect::Mill {
                who: crabomination::effect::Selector::Player(crabomination::effect::PlayerRef::EachOpponent),
                amount: crabomination::effect::Value::ONE,
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(haunt).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "The Haunt gains a +1/+1 counter from the milled card"
    );
}
