//! CR conformance for this run's RAV/RTR work: CR 614.6 (replacement "instead"
//! — Phytohydra converts *combat* damage to counters), CR 702.98 (Unleash —
//! a creature that entered unleashed can't block), and CR 702.96 (Scavenge is
//! sorcery-speed only).

use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game};

/// CR 614.6 — Phytohydra's "if damage would be dealt to it, put that many
/// +1/+1 counters instead" replaces *combat* damage too: a blocked attacker
/// marks no damage on it, and it grows by the assigned amount.
#[test]
fn cr_614_6_phytohydra_replaces_combat_damage() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let hydra = g.add_card_to_battlefield(0, catalog::phytohydra()); // 1/1
    let attacker = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(crabomination::game::GameAction::PassPriority).expect("pass");
    }
    g.perform_action(crabomination::game::GameAction::DeclareBlockers(vec![(hydra, attacker)]))
        .expect("block");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(crabomination::game::GameAction::PassPriority).expect("pass");
    }
    let c = g.battlefield_find(hydra).expect("Phytohydra survives — damage was replaced");
    assert_eq!(c.damage, 0, "no combat damage marked");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 6, "grew by the 6 assigned");
}

/// CR 702.98 — a creature that entered with an unleash counter can't be
/// declared as a blocker.
#[test]
fn cr_702_98_unleashed_creature_cant_block() {
    use crabomination::card::CounterType;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::mana::Color;
    let mut g = two_player_game();
    // Player 1 casts Dead Reveler unleashed (enters with a +1/+1 counter).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let rev = g.add_card_to_hand(1, catalog::dead_reveler());
    g.players[1].mana_pool.add_colorless(2);
    g.players[1].mana_pool.add(Color::Black, 1);
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.perform_action(crabomination::game::GameAction::CastSpell {
        card_id: rev, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast unleashed");
    drain_stack(&mut g);
    let reveler = g.battlefield.iter().find(|c| c.definition.name == "Dead Reveler").unwrap().id;
    assert_eq!(g.battlefield_find(reveler).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    // Player 0 attacks; the unleashed reveler can't be declared blocking it.
    g.active_player_idx = 0;
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(crabomination::game::GameAction::PassPriority).expect("pass");
    }
    assert!(
        g.declare_blockers(vec![(reveler, attacker)]).is_err(),
        "an unleashed creature can't block",
    );
}

/// CR 702.96 — Scavenge is a sorcery-speed activated ability: it can't be used
/// during an opponent's combat.
#[test]
fn cr_702_96_scavenge_is_sorcery_speed() {
    let mut g = two_player_game();
    let mon = g.add_card_to_graveyard(0, catalog::korozda_monitor());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    // Player 1's turn, combat step — not player 0's main phase.
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(crabomination::game::GameAction::ActivateAbility {
            card_id: mon, ability_index: 0, target: Some(Target::Permanent(target)),
            additional_targets: vec![], x_value: None, mode: None,
        }).is_err(),
        "scavenge can't be activated at instant speed",
    );
}
