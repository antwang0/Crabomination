//! CR conformance for this run's state-based-action sweep: CR 704.5b (deckout
//! loss), CR 704.5d (a token in a non-battlefield zone ceases to exist), and
//! CR 704.5f (toughness ≤ 0 is put into the graveyard — even for an
//! indestructible creature, since it isn't a "destroy").

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, Value};
use crabomination::game::effects::EffectContext;
use crabomination::game::{drain_stack, two_player_game};

/// CR 704.5b — a player who attempts to draw from an empty library loses the
/// game (loss cause: decked).
#[test]
fn cr_704_5b_draw_from_empty_library_loses() {
    use crabomination::player::LossCause;
    let mut g = two_player_game();
    g.players[0].library.clear(); // no cards to draw
    let ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    let _ = g.resolve_effect(&Effect::Draw { who: Selector::You, amount: Value::ONE }, &ctx);
    // CR 104.3c — the loss is a state-based action, so it happens the next
    // time a player would receive priority, not inside the draw. Until then
    // the player is still in the game and "each opponent" still sees them.
    assert!(!g.players[0].eliminated, "CR 104.3c — not eliminated inside the draw");
    let _ = g.check_state_based_actions();
    assert!(g.players[0].eliminated, "empty-library draw eliminates the player");
    assert_eq!(g.players[0].loss_cause, Some(LossCause::Decked), "loss cause is decking out");
}

/// CR 704.5d — a token that leaves the battlefield (bounced to hand) ceases to
/// exist rather than lingering in the new zone.
#[test]
fn cr_704_5d_token_bounced_to_hand_ceases() {
    let mut g = two_player_game();
    let tok = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(tok).unwrap().is_token = true;
    let hand_before = g.players[0].hand.len();
    let ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    let mut events = Vec::new();
    g.move_card_to(
        tok,
        &crabomination::effect::ZoneDest::Hand(crabomination::effect::PlayerRef::OwnerOfMoved),
        &ctx,
        &mut events,
    );
    g.check_state_based_actions();
    assert!(g.battlefield_find(tok).is_none(), "token left the battlefield");
    assert_eq!(g.players[0].hand.len(), hand_before, "the token ceased to exist, not added to hand");
}

/// CR 704.5f — a creature reduced to toughness 0 is put into its owner's
/// graveyard even while indestructible (that SBA isn't a "destroy", so the
/// Indestructible counter doesn't save it).
#[test]
fn cr_704_5f_toughness_zero_kills_indestructible() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    {
        let c = g.battlefield_find_mut(bear).unwrap();
        c.add_counters(CounterType::Indestructible, 1);
        c.add_counters(CounterType::MinusOneMinusOne, 2); // → 0/0
    }
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "0-toughness creature dies through indestructibility");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear), "it went to the graveyard");
    drain_stack(&mut g);
}

/// CR 104.3c — the deck-out loss is a state-based action, so a player decked
/// *during* a resolution is still in the game for the rest of it. Before this
/// was deferred, `lose_to_empty_draw` set `eliminated` inside the draw and
/// `resolve_players` (which filters on `is_alive`) skipped the decked player
/// for the remainder of the same spell.
#[test]
fn cr_104_3c_decked_opponent_still_seen_by_the_same_resolution() {
    let mut g = two_player_game();
    g.players[1].library.clear();
    let start = g.players[1].life;
    let ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    // "Each opponent draws a card, then each opponent loses 2 life." The draw
    // decks P1; the life loss must still find them.
    let _ = g.resolve_effect(
        &Effect::Draw { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        &ctx,
    );
    assert!(!g.players[1].eliminated, "CR 104.3c — no elimination inside the draw");
    let _ = g.resolve_effect(
        &Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
        &ctx,
    );
    assert_eq!(g.players[1].life, start - 2, "the decked opponent is still an opponent");
    let _ = g.check_state_based_actions();
    assert!(g.players[1].eliminated, "and the SBA check performs the loss");
}

/// CR 800.4a — a player who loses to decking takes their permanents with
/// them. Eliminating inside the draw skipped `objects_leave_with_player`
/// entirely, so a decked player's board stayed on the battlefield for the
/// rest of the game.
#[test]
fn cr_800_4a_decked_players_permanents_leave_with_them() {
    let mut g = two_player_game();
    g.players[1].library.clear();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    let _ = g.resolve_effect(
        &Effect::Draw { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        &ctx,
    );
    assert!(g.battlefield.iter().any(|c| c.id == bear), "still theirs until the SBA");
    let _ = g.check_state_based_actions();
    assert!(g.players[1].eliminated);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "CR 800.4a — it left with them");
}
