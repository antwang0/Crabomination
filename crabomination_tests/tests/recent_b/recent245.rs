//! Functionality tests for `catalog::sets::decks::recent245` (MKM Clue
//! Equipment cycle + collect-evidence payoffs).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::GameAction;
use crabomination::game::{drain_stack, two_player_game, GameEvent};

fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
}

/// Wrench grants +1/+1, vigilance, and a "{3}, {T}: Tap target creature"
/// activated ability to the equipped creature.
#[test]
fn wrench_buffs_and_grants_tap_ability() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wrench = g.add_card_to_battlefield(0, catalog::wrench());
    g.battlefield_find_mut(wrench).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "gains vigilance");
    assert_eq!(g.granted_abilities_for(bear).len(), 1, "granted the tap ability");
}

/// Rope grants +1/+2, reach, and can't-be-blocked-by-more-than-one.
#[test]
fn rope_buffs_reach_and_lone_block() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rope = g.add_card_to_battlefield(0, catalog::rope());
    g.battlefield_find_mut(rope).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "+1/+2");
    assert!(cp.keywords.contains(&Keyword::Reach));
    assert!(cp.keywords.contains(&Keyword::CantBeBlockedByMoreThanOne));
}

/// Knife's +1/+0 and first strike apply only during the controller's turn.
#[test]
fn knife_only_during_your_turn() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let knife = g.add_card_to_battlefield(0, catalog::knife());
    g.battlefield_find_mut(knife).unwrap().attached_to = Some(bear);
    // Player 0's turn: bonus active.
    g.active_player_idx = 0;
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "+1/+0 during your turn");
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
    // Opponent's turn: bonus gone.
    g.active_player_idx = 1;
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 2, "no bonus off your turn");
    assert!(!cp.keywords.contains(&Keyword::FirstStrike));
}

/// The shared "{2}, Sacrifice this Equipment: Draw a card" ability draws and
/// sacrifices the Equipment.
#[test]
fn clue_equipment_sac_draws() {
    let mut g = two_player_game();
    let candlestick = g.add_card_to_battlefield(0, catalog::candlestick());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: candlestick,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("sac Candlestick to draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert!(g.battlefield_find(candlestick).is_none(), "Equipment sacrificed");
}

/// Surveillance Monitor mints a Thopter whenever its controller collects
/// evidence.
#[test]
fn surveillance_monitor_thopter_on_evidence() {
    let mut g = two_player_game();
    let _mon = g.add_card_to_battlefield(0, catalog::surveillance_monitor());
    g.dispatch_triggers_for_events(&[GameEvent::EvidenceCollected { player: 0 }]);
    drain_stack(&mut g);
    let thopters = g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count();
    assert_eq!(thopters, 1, "collecting evidence made a Thopter");
}

/// Evidence Examiner investigates whenever its controller collects evidence.
#[test]
fn evidence_examiner_investigates_on_collection() {
    let mut g = two_player_game();
    let _examiner = g.add_card_to_battlefield(0, catalog::evidence_examiner());
    g.dispatch_triggers_for_events(&[GameEvent::EvidenceCollected { player: 0 }]);
    drain_stack(&mut g);
    assert_eq!(clues(&g, 0), 1, "collecting evidence investigated");
}

/// Collecting evidence via `Effect::CollectEvidence` emits an
/// `EvidenceCollected` event (the wiring behind the two payoffs above).
#[test]
fn collect_evidence_emits_event() {
    use crabomination::effect::{Effect, Value};
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::surveillance_monitor());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 6, ample for 4
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let effect = Effect::CollectEvidence { amount: Value::Const(4), then: Box::new(Effect::Noop) };
    let events = g.resolve_effect(&effect, &EffectContext::for_ability(src, 0, None)).unwrap();
    assert!(
        events.iter().any(|e| matches!(e, GameEvent::EvidenceCollected { player: 0 })),
        "collection emitted the EvidenceCollected event"
    );
}

/// Unscrupulous Agent makes an opponent exile a card from hand on ETB.
#[test]
fn unscrupulous_agent_exiles_from_hand() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let opp_hand = g.players[1].hand.len();
    let agent = g.add_card_to_battlefield(0, catalog::unscrupulous_agent());
    g.fire_self_etb_triggers(agent, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent exiled a card");
}

/// Undercity Eliminator sacrifices a permanent to exile an opponent's creature.
#[test]
fn undercity_eliminator_sacs_to_exile() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _fodder = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let elim = g.add_card_to_battlefield(0, catalog::undercity_eliminator());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_self_etb_triggers(elim, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "opponent's creature exiled");
}

/// Furtive Courier loots (draw then discard) when it attacks.
#[test]
fn furtive_courier_attack_loots() {
    let mut g = two_player_game();
    let courier = g.add_card_to_battlefield(0, catalog::furtive_courier());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let effect = catalog::furtive_courier().triggered_abilities[0].effect.clone();
    let before = g.players[0].graveyard.len();
    g.resolve_effect(&effect, &EffectContext::for_trigger(courier, 0, None, 0)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), before + 1, "discarded one card after drawing");
}
