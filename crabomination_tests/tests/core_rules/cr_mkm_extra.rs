//! Comprehensive-Rules conformance for MKM-adjacent mechanics: granted Wither
//! in combat (CR 702.90), "can't be blocked by more than one creature"
//! (CR 509.1g), Collect evidence firing the collect event and as an activated-
//! ability cost (CR 701.59), Disguise face-down body + turn-face-up trigger
//! (CR 702.166), and a discard/sacrifice-or-lose-life Punisher dodge
//! (CR 701.55 / 601.2).

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

/// CR 701.59 — collect evidence can be an *activated-ability* cost. Forensic
/// Researcher's second ability pays "Collect evidence 3" by exiling graveyard
/// cards worth ≥ 3, then taps an opponent's creature; it's rejected when the
/// graveyard can't afford the cost (so the tap isn't burned).
#[test]
fn cr_701_59_collect_evidence_activation_cost() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let researcher = g.add_card_to_battlefield(0, catalog::forensic_researcher());
    g.clear_sickness(researcher);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Empty graveyard → the collect-evidence cost can't be paid.
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: researcher,
            ability_index: 1,
            target: Some(Target::Permanent(foe)),
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .is_err(),
        "rejected with no evidence to collect",
    );
    assert!(!g.battlefield_find(researcher).unwrap().tapped, "tap not burned");

    // Two MV-6 cards more than cover evidence 3; now the tap resolves.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: researcher,
        ability_index: 1,
        target: Some(Target::Permanent(foe)),
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("collect-evidence tap resolves");
    crabomination::game::drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "opponent's creature tapped");
    assert!(g.players[0].graveyard.len() < gy_before, "graveyard cards exiled to collect evidence");
}

/// CR 702.166 — a Disguise card cast face down is a 2/2 creature with ward {2};
/// turning it face up for its disguise cost restores its real characteristics
/// and fires its "when turned face up" trigger.
#[test]
fn cr_702_166_disguise_face_down_body_and_flip() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let card = g.add_card_to_hand(0, catalog::bubble_smuggler());
    // Disguise's face-down cast costs {3}.
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: card }).expect("cast face down");
    crabomination::game::drain_stack(&mut g);
    let fd = g.battlefield_find(card).expect("face-down permanent on the battlefield");
    assert_eq!((fd.definition.power, fd.definition.toughness), (2, 2), "face-down 2/2");
    let cp = g.computed_permanent(card).unwrap();
    assert!(
        cp.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))),
        "face-down body has ward",
    );
    // Turn face up for the disguise cost {5}{U}; the counters trigger fires.
    g.clear_sickness(card);
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::TurnFaceUp { card_id: card }).expect("turn face up");
    crabomination::game::drain_stack(&mut g);
    let up = g.battlefield_find(card).unwrap();
    assert_eq!(up.counter_count(CounterType::PlusOnePlusOne), 4, "turn-up trigger added four counters");
}

/// CR 701.55 / 601.2 — a "loses life unless they discard or sacrifice" Punisher
/// lets the punished player dodge with any affordable option: an opponent
/// holding a creature sacrifices it rather than losing 3 life.
#[test]
fn cr_701_55_punisher_opponent_dodges_by_sacrificing() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let orb = g.add_card_to_battlefield(0, catalog::polygraph_orb());
    // Evidence for the cost; the opponent has no cards but a spare creature.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[1].hand.clear();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: orb,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("activate the punisher");
    crabomination::game::drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "opponent sacrificed a creature to dodge");
    assert_eq!(g.players[1].life, life_before, "no life lost when the sacrifice was chosen");
}
