//! Recover (CR 702.59) + the CR 121.2b/121.3 "can't choose to draw" rule.

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameEvent, Target};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameState, TurnStep};
use crabomination::mana::Color;

/// Kill `victim` through the real death funnel and dispatch its triggers.
fn kill(g: &mut GameState, victim: crabomination::game::CardId) {
    let mut evs = g.remove_to_graveyard_with_triggers(victim);
    evs.push(GameEvent::CreatureDied { card_id: victim });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// CR 702.59a — paying the recover cost returns the card to hand.
#[test]
fn cr_702_59_recover_returns_the_card_when_paid() {
    let mut g = two_player_game();
    let rot = g.add_card_to_graveyard(0, catalog::krovikan_rot());
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    kill(&mut g, victim);
    assert!(g.players[0].hand.iter().any(|c| c.id == rot), "recovered to hand");
}

/// CR 702.59a — declining exiles it instead.
#[test]
fn cr_702_59_recover_exiles_when_declined() {
    let mut g = two_player_game();
    let rot = g.add_card_to_graveyard(0, catalog::krovikan_rot());
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    kill(&mut g, victim);
    assert!(g.exile.iter().any(|c| c.id == rot), "declined recover exiles the card");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == rot));
}

/// CR 702.59a — "put into *your* graveyard": an opponent's creature dying
/// doesn't arm recover.
#[test]
fn cr_702_59_recover_ignores_an_opponents_creature_death() {
    let mut g = two_player_game();
    let rot = g.add_card_to_graveyard(0, catalog::krovikan_rot());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    kill(&mut g, theirs);
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == rot),
        "still in the graveyard — not returned, not exiled",
    );
}

#[test]
fn garzas_assassin_recovers_for_half_your_life() {
    let mut g = two_player_game();
    let garza = g.add_card_to_graveyard(0, catalog::garzas_assassin());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].life = 20;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    kill(&mut g, other);
    assert!(g.players[0].hand.iter().any(|c| c.id == garza), "recovered");
    assert_eq!(g.players[0].life, 10, "paid half of 20");
}

/// CR 121.2b — a draw cap truncates a forced draw.
#[test]
fn cr_121_2b_leovold_caps_an_opponents_draws() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leovold_emissary_of_trest());
    for _ in 0..5 {
        let id = g.next_id();
        g.players[1].library.push(crabomination::card::CardInstance::new(
            id,
            catalog::grizzly_bears(),
            1,
        ));
    }
    let before = g.players[1].hand.len();
    let mut events = vec![];
    g.draw_one(1, &mut events);
    g.draw_one(1, &mut events);
    assert_eq!(g.players[1].hand.len(), before + 1, "second draw is capped away");
}

/// CR 121.3 — a player who can't draw can't *choose* to draw either, so the
/// optional draw is never offered.
#[test]
fn cr_121_3_capped_player_is_not_offered_an_optional_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::leovold_emissary_of_trest());
    assert!(g.may_choose_to_draw(0, 1), "no draws yet — the choice stands");
    g.players[0].cards_drawn_this_turn = 1;
    assert!(!g.may_choose_to_draw(0, 1), "already at the cap — can't choose to draw");
    assert!(g.may_choose_to_draw(1, 1), "Leovold's controller is unaffected");
}

/// CR 121.3 — an empty library does NOT block the choice.
#[test]
fn cr_121_3_empty_library_still_allows_choosing_to_draw() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    assert!(g.may_choose_to_draw(0, 1));
}

#[test]
fn captains_defense_pumps_a_blocker_and_draws() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(0) }];
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, atk)])).expect("blocks");
    g.priority.player_with_priority = 0;
    let lib = g.next_id();
    g.players[0].library.push(crabomination::card::CardInstance::new(
        lib,
        catalog::grizzly_bears(),
        0,
    ));
    let cd = g.add_card_to_hand(0, catalog::captains_defense());
    g.players[0].mana_pool.add(Color::White, 1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: cd,
        target: Some(Target::Permanent(blocker)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(blocker).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert_eq!(g.players[0].hand.len(), hand, "spent one card, drew one");
}

#[test]
fn outflank_scales_with_your_creature_count() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(0) }];
    let of = g.add_card_to_hand(0, catalog::outflank());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: of,
        target: Some(Target::Permanent(atk)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(atk).is_none(), "3 damage kills the 2/2");
}
