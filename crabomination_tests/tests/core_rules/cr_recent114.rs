//! CR 115.1 / 400.3 — "target … card from your graveyard" can't target a card
//! in another player's graveyard. Found by a strict debug pod: Mystic
//! Sanctuary put an opponent's Valorous Stance on top of its controller's
//! library (`scripts/audit_your_graveyard_target.py` is the census).

use crabomination::card::CardDefinition;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::TurnStep;
use crabomination::game::*;

fn pod() -> GameState {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // Every "you may" here is a yes.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    g
}

/// Each enters-trigger, with the only matching card in an opponent's
/// graveyard, leaves that card where it is.
#[test]
fn cr_115_1_a_your_graveyard_target_never_reaches_an_opponents_graveyard() {
    type Setup = fn(&mut GameState);
    let none: Setup = |_| {};
    let islands: Setup = |g| {
        for _ in 0..3 {
            g.add_card_to_battlefield(0, catalog::island());
        }
    };
    let two_to_exile: Setup = |g| {
        for _ in 0..2 {
            g.add_card_to_graveyard(0, catalog::forest());
        }
    };
    type Case = (&'static str, fn() -> CardDefinition, fn() -> CardDefinition, Setup);
    let cases: [Case; 5] = [
        ("Mystic Sanctuary", catalog::mystic_sanctuary, catalog::lightning_bolt, islands),
        ("Torrential Gearhulk", catalog::torrential_gearhulk, catalog::lightning_bolt, none),
        ("Neva", catalog::neva_stalked_by_nightmares, catalog::grizzly_bears, none),
        ("Titania", catalog::titania_protector_of_argoth, catalog::forest, none),
        ("Young Necromancer", catalog::young_necromancer, catalog::grizzly_bears, two_to_exile),
    ];
    for (name, card, theirs, setup) in cases {
        let mut g = pod();
        setup(&mut g);
        let other = g.add_card_to_graveyard(1, theirs());
        g.move_card_to_battlefield_for_test(0, card());
        drain_stack(&mut g);
        assert!(
            g.players[1].graveyard.iter().any(|c| c.id == other),
            "{name} took seat 1's card out of its graveyard",
        );
    }
}

/// And the card the trigger may take is still found in its own graveyard.
#[test]
fn cr_115_1_mystic_sanctuary_still_tops_your_own_instant() {
    let mut g = pod();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let mine = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.move_card_to_battlefield_for_test(0, catalog::mystic_sanctuary());
    drain_stack(&mut g);
    assert!(
        g.players[0].library.iter().any(|c| c.id == mine),
        "your instant went on top of your library: gy {:?} stack {} pending {:?}",
        g.players[0].graveyard.iter().map(|c| c.definition.name).collect::<Vec<_>>(),
        g.stack.len(),
        g.pending_decision.as_ref().map(|p| &p.decision),
    );
}

/// CR 400.3 — a card bound for a hand or library other than its owner's goes
/// to its owner's: seat 0's "put it on top of your library" / "return it to
/// your hand" on seat 1's permanent (a stolen Sensei's Divining Top) puts it
/// in seat 1's. Found by a debug-pod invariant.
#[test]
fn cr_400_3_a_card_goes_to_its_owners_hand_or_library() {
    use crabomination::effect::{Effect, LibraryPosition, PlayerRef, Selector, ZoneDest};
    use crabomination::game::effects::EffectContext;
    let dests = [
        ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
        ZoneDest::Hand(PlayerRef::You),
    ];
    for to in dests {
        let mut g = pod();
        let top = g.add_card_to_battlefield(1, catalog::senseis_divining_top());
        g.battlefield_find_mut(top).unwrap().controller = 0;
        let ctx = EffectContext::for_ability(top, 0, None);
        g.resolve_effect(&Effect::Move { what: Selector::ExactObjects(vec![top]), to: to.clone() }, &ctx)
            .expect("resolves");
        let p1 = &g.players[1];
        assert!(
            p1.library.iter().chain(p1.hand.iter()).any(|c| c.id == top),
            "{to:?}: the Top is in its owner's zone",
        );
        let p0 = &g.players[0];
        assert!(!p0.library.iter().chain(p0.hand.iter()).any(|c| c.id == top), "{to:?}: not its controller's");
    }
}
