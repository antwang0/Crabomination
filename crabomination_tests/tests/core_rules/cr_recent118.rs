//! CR 400.7 / 607 — a card that leaves exile is a new object: the exiler's
//! "until this leaves" link (`exiled_by`) does not follow it into its next
//! zone, nor into a later exile.

use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::*;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

fn resolve(g: &mut GameState, source: crabomination::card::CardId, target: Option<Target>, e: &Effect) {
    let events = g.resolve_effect(e, &EffectContext::for_ability(source, 0, target)).expect("resolves");
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

/// Foreboding Steamboat exiles a Bear, its attack trigger puts the Bear into
/// the graveyard, and a later exile of that card is a plain exile: the
/// Steamboat leaving doesn't bring it back. (Two 6-seat debug pods, seed
/// 130026 games 10 and 27, found the card still "exiled until the Steamboat
/// leaves" with the Steamboat long gone.)
#[test]
fn cr_400_7_a_card_out_of_exile_loses_its_exilers_link() {
    let mut g = main_phase();
    let boat = g.add_card_to_battlefield(0, catalog::foreboding_steamboat());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let etb = catalog::foreboding_steamboat().triggered_abilities[0].effect.clone();
    resolve(&mut g, boat, None, &etb);
    assert!(g.exile.iter().any(|c| c.id == bear && c.exiled_by.is_some()));
    let to_gy = Effect::Move { what: Selector::CardExiledWithSource, to: ZoneDest::Graveyard };
    resolve(&mut g, boat, None, &to_gy);
    let in_gy = g.players[1].graveyard.iter().find(|c| c.id == bear).expect("in the graveyard");
    assert!(in_gy.exiled_by.is_none() && in_gy.exiled_with.is_none());
    resolve(&mut g, boat, None, &Effect::Move { what: Selector::ExactObjects(vec![bear]), to: ZoneDest::Exile });
    resolve(&mut g, boat, None, &Effect::Destroy { what: Selector::ExactObjects(vec![boat]) });
    assert!(g.exile.iter().any(|c| c.id == bear), "a plain exile: the Steamboat leaving doesn't return it");
}

/// Hostage Taker exiles a Bear; the Bear comes back by another route, is
/// exiled again by something else, and Hostage Taker leaving must not return
/// it (CR 610.3c reads only the object it exiled).
#[test]
fn cr_400_7_a_reexiled_card_does_not_return_with_its_old_exiler() {
    let mut g = main_phase();
    let taker = g.add_card_to_battlefield(0, catalog::hostage_taker());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let etb = catalog::hostage_taker().triggered_abilities[0].effect.clone();
    resolve(&mut g, taker, Some(Target::Permanent(bear)), &etb);
    assert!(g.exile.iter().any(|c| c.id == bear), "the Taker holds it");
    resolve(
        &mut g,
        taker,
        None,
        &Effect::Move {
            what: Selector::ExactObjects(vec![bear]),
            to: ZoneDest::Battlefield { controller: PlayerRef::Seat(1), tapped: false },
        },
    );
    assert!(g.battlefield_find(bear).is_some_and(|c| c.exiled_by.is_none()));
    resolve(&mut g, taker, None, &Effect::Move { what: Selector::ExactObjects(vec![bear]), to: ZoneDest::Exile });
    resolve(&mut g, taker, None, &Effect::Destroy { what: Selector::ExactObjects(vec![taker]) });
    assert!(g.battlefield_find(bear).is_none(), "the Taker's leave returns only what it holds");
}
