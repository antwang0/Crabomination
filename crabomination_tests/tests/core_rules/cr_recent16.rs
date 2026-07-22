//! CR conformance for this run's state-based-action sweep: CR 704.5b (deckout
//! loss), CR 704.5d (a token in a non-battlefield zone ceases to exist), and
//! CR 704.5f (toughness ≤ 0 is put into the graveyard — even for an
//! indestructible creature, since it isn't a "destroy").

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::effect::{Effect, Selector, Value};
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
