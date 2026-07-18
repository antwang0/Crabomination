//! Comprehensive-Rules conformance for MKM-adjacent mechanics: granted Wither
//! in combat (CR 702.90), "can't be blocked by more than one creature"
//! (CR 509.1g), and Collect evidence firing the collect event (CR 701.59).

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Value};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::{two_player_game, GameEvent};

/// CR 702.90 — a creature granted Wither (Massacre Girl, Known Killer) deals its
/// combat damage to a blocker as -1/-1 counters, not marked damage.
#[test]
fn cr_702_90_granted_wither_combat_damage_is_counters() {
    let mut g = two_player_game();
    let _girl = g.add_card_to_battlefield(0, catalog::massacre_girl_known_killer());
    let attacker = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3, now has wither
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    let b = g.battlefield_find(blocker).unwrap();
    assert_eq!(
        b.counter_count(CounterType::MinusOneMinusOne),
        3,
        "3 wither damage lands as -1/-1 counters"
    );
    assert_eq!(b.damage, 0, "no damage is marked");
}

/// CR 509.1g — a creature that "can't be blocked by more than one creature"
/// (Rope's grant) rejects a two-blocker declaration.
#[test]
fn cr_509_1g_lone_block_rejects_two_blockers() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.clear_sickness(attacker);
    let rope = g.add_card_to_battlefield(0, catalog::rope());
    g.battlefield_find_mut(rope).unwrap().attached_to = Some(attacker);
    assert!(g
        .computed_permanent(attacker)
        .unwrap()
        .keywords
        .contains(&Keyword::CantBeBlockedByMoreThanOne));
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    let two = g.perform_action(GameAction::DeclareBlockers(vec![(b1, attacker), (b2, attacker)]));
    assert!(two.is_err(), "two blockers on a lone-block creature is illegal");
    // One blocker is fine.
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, attacker)])).expect("one blocker legal");
}

/// CR 701.59 — collecting evidence N exiles graveyard cards whose total mana
/// value is at least N (the engine takes the cheapest qualifying set) and emits
/// the collect event that drives "whenever you collect evidence" payoffs.
#[test]
fn cr_701_59_collect_evidence_exiles_at_least_n_and_emits_event() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::surveillance_monitor());
    // Graveyard: two MV-1 cards + one MV-6 card. Collecting evidence 3 should
    // exile the two cheap ones (total MV 2 < 3 → needs the third), landing on a
    // qualifying set with total MV ≥ 3.
    g.add_card_to_graveyard(0, catalog::llanowar_elves()); // MV 1
    g.add_card_to_graveyard(0, catalog::llanowar_elves()); // MV 1
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 6
    let gy_before = g.players[0].graveyard.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let effect = Effect::CollectEvidence { amount: Value::Const(3), then: Box::new(Effect::Noop) };
    let events = g.resolve_effect(&effect, &EffectContext::for_ability(src, 0, None)).unwrap();
    // Total MV of exiled cards must clear the threshold.
    let exiled: u32 = g
        .exile
        .iter()
        .filter(|c| c.owner == 0)
        .map(|c| c.definition.cost.cmc())
        .sum();
    assert!(exiled >= 3, "exiled a qualifying set (total MV ≥ 3), got {exiled}");
    assert!(g.players[0].graveyard.len() < gy_before, "cards left the graveyard");
    assert!(
        events.iter().any(|e| matches!(e, GameEvent::EvidenceCollected { player: 0 })),
        "collection emitted the collect event"
    );
}
