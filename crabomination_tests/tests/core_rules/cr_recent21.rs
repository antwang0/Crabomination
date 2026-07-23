//! CR conformance for rules exercised by this run's GTC wave 16:
//! CR 601.3e (a "can't cast noncreature spells" lock blocks noncreature casts
//! but not creatures — near Aurelia's Fury's new bound-recipient rider), CR
//! 508.1a (a summoning-sick creature can't be declared as an attacker —
//! random), and CR 601.3a / 608.2 (a card exiled with a while-exiled may-play
//! grant can be cast from exile by the grantee — near Nightveil Specter's
//! play-from-exile grant).

use crabomination::card::{MayPlayDuration, MayPlayPermission};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameError};

/// CR 601.3e — a player told they "can't cast noncreature spells this turn"
/// (Aurelia's Fury) is blocked from casting an instant, but may still cast a
/// creature spell.
#[test]
fn cr_601_3e_noncreature_lock_blocks_instant_allows_creature() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::shock()); // instant
    let dude = g.add_card_to_hand(0, catalog::gutter_skulk()); // {1}{B} creature
    g.players[0].cant_cast_noncreature_this_turn = true;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for c in [Color::Red, Color::Black] {
        g.players[0].mana_pool.add(c, 2);
    }
    g.players[0].mana_pool.add_colorless(2);
    let blocked = g.cast_spell(bolt, Some(Target::Player(1)), vec![], None, None);
    assert!(matches!(blocked, Err(GameError::CantCastNoncreature)),
        "the noncreature lock blocks an instant");
    g.cast_spell(dude, None, vec![], None, None).expect("a creature spell is still castable");
}

/// CR 508.1a — a creature that hasn't been controlled continuously since the
/// turn began (summoning sick, no haste) can't be declared as an attacker.
#[test]
fn cr_508_1a_summoning_sick_creature_cant_attack() {
    let mut g = two_player_game();
    let fresh = g.add_card_to_battlefield(0, catalog::gutter_skulk()); // summoning sick
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let err = g.declare_attackers(vec![Attack { attacker: fresh, target: AttackTarget::Player(1) }]);
    assert!(matches!(err, Err(GameError::SummoningSickness(_))),
        "a summoning-sick creature can't attack");
}

/// CR 601.3a / 608.2 — a card exiled with a while-exiled "you may play it"
/// grant (Nightveil Specter) can be cast from exile by the grantee, entering
/// under that player's control even though its owner is the opponent.
#[test]
fn cr_601_3a_play_card_exiled_with_grant() {
    use crabomination::mana::Color;
    // The opponent (owner) has the card; player 0 holds a while-exiled grant.
    let mut g = two_player_game();
    let card = g.add_card_to_exile(1, catalog::gutter_skulk()); // {1}{B} 2/2, owned by p1
    {
        let c = g.exile.iter_mut().find(|c| c.id == card).unwrap();
        c.may_play_until = Some(MayPlayPermission {
            player: 0,
            granted_turn: g.turn_number,
            duration: MayPlayDuration::WhileExiled,
            exile_after: false,
            miracle: false,
        });
        // "You may play them" pays the card's own cost (Nightveil's pay_own_cost).
        c.granted_alt_cast_cost_eot = Some(c.definition.cost.clone());
    }
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: card, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast the exiled card from exile");
    drain_stack(&mut g);
    let perm = g.battlefield_find(card).expect("card resolved onto the battlefield");
    assert_eq!(perm.controller, 0, "it enters under the grantee's control");
    assert!(!g.exile.iter().any(|c| c.id == card), "it left exile as it was cast");
}
