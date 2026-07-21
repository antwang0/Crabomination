//! CR conformance for rules exercised by the Ravnica-block gap batches:
//! CR 615.1 (a "prevent all combat damage it would deal" effect zeroes an
//! attacker's outgoing damage but not the damage dealt *to* it),
//! CR 702.19e (a trampler assigns lethal to its blocker, remainder to the
//! player), and CR 104.3a / 800.4a (a conceding player leaves the game and
//! their objects leave with them).

use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::*;

/// CR 615.1 — Azorius Ploy prevents all combat damage its first target would
/// *deal*; the creature still takes combat damage dealt to it, so an attacker
/// whose damage is prevented still dies to its blocker's strike-back.
#[test]
fn cr_615_1_prevented_dealer_still_takes_damage() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(atk);
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2

    // slot0 = atk (prevent atk's outgoing), slot1 = blk (unused prevent-to slot
    // on a creature that takes no combat damage this test).
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(atk), Target::Permanent(blk)];
    let evs = g.resolve_effect(&catalog::azorius_ploy().effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.battlefield_find(blk).is_some(), "attacker dealt no combat damage");
    assert!(g.battlefield_find(atk).is_none(), "attacker still took its blocker's strike-back");
}

/// CR 702.19e — a trampling attacker assigns lethal damage to its blocker and
/// the remainder tramples over to the defending player.
#[test]
fn cr_702_19e_trample_assigns_lethal_then_over() {
    use crabomination::card::{CardDefinition, CardType, Keyword, Subtypes};
    let trampler = CardDefinition {
        name: "Trampler", card_types: vec![CardType::Creature],
        power: 4, toughness: 4, keywords: vec![Keyword::Trample],
        subtypes: Subtypes::default(), ..Default::default()
    };
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, trampler);
    g.clear_sickness(atk);
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let life1 = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.battlefield_find(blk).is_none(), "blocker took its 2 lethal");
    assert_eq!(g.players[1].life, life1 - 2, "the other 2 trampled over");
}

/// CR 104.3a / 800.4a — a conceding player is eliminated, records the loss
/// cause, and their permanents leave the battlefield with them.
#[test]
fn cr_104_3a_concession_removes_player_and_objects() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let evs = g.concede(1);
    assert!(g.players[1].eliminated, "conceding player eliminated");
    assert_eq!(g.players[1].loss_cause, Some(crabomination::player::LossCause::Conceded));
    assert!(g.battlefield_find(bear).is_none(), "their objects left with them");
    assert!(evs.iter().any(|e| matches!(e, crabomination::game::types::GameEvent::PlayerConceded { player: 1 })),
        "a PlayerConceded event fired");
    assert!(g.game_over.is_some(), "the last opponent standing ends the game");
}
