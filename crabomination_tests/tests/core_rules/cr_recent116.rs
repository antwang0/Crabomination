//! CR 603.10a / 603.2 — a "whenever a creature dies" listener that the same
//! resolution removes: one destroyed WITH the creatures saw them die (a
//! leaves-the-battlefield ability looks back), one removed AFTER a death saw
//! it, and one removed BEFORE it did not.

use crabomination::card::SelectionRequirement as R;
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::TurnStep;
use crabomination::game::*;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

fn resolve(g: &mut GameState, source: crabomination::card::CardId, e: Effect) {
    let events = g
        .resolve_effect(&e, &EffectContext::for_ability(source, 0, None))
        .expect("resolves");
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

/// CR 603.10a — Planar Cleansing destroys Bastion of Remembrance with the two
/// Bears; the enchantment's death trigger looks back and drains twice.
#[test]
fn cr_603_10a_a_noncreature_listener_destroyed_with_the_creatures_saw_them_die() {
    let mut g = main_phase();
    let bastion = g.add_card_to_battlefield(0, catalog::bastion_of_remembrance());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve(&mut g, bastion, catalog::planar_cleansing().effect);
    assert!(g.battlefield.is_empty());
    assert_eq!((g.players[0].life, g.players[1].life), (22, 18));
}

/// CR 603.2 — a Bear dies, then Blood Artist is bounced in the same
/// resolution: it was there for the death and drains once.
#[test]
fn cr_603_2_a_listener_bounced_after_a_death_saw_it() {
    let mut g = main_phase();
    let artist = g.add_card_to_battlefield(0, catalog::blood_artist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    resolve(
        &mut g,
        artist,
        Effect::Seq(vec![
            Effect::Destroy {
                what: Selector::ExactObjects(vec![bear]),
            },
            Effect::Move {
                what: Selector::ExactObjects(vec![artist]),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
    );
    assert!(g.players[0].hand.iter().any(|c| c.id == artist));
    assert_eq!((g.players[0].life, g.players[1].life), (21, 19));
}

/// And not a death after it left: bounced first, then the Bear dies.
#[test]
fn cr_603_2_a_listener_bounced_before_a_death_did_not_see_it() {
    let mut g = main_phase();
    let artist = g.add_card_to_battlefield(0, catalog::blood_artist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    resolve(
        &mut g,
        artist,
        Effect::Seq(vec![
            Effect::Move {
                what: Selector::ExactObjects(vec![artist]),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Destroy {
                what: Selector::ExactObjects(vec![bear]),
            },
        ]),
    );
    assert_eq!((g.players[0].life, g.players[1].life), (20, 20));
}

/// CR 603.10a — an enchantment exiled together with the creatures is not a
/// death of its own, but its "whenever a creature you control dies" still
/// does not see an exile: nothing died.
#[test]
fn cr_603_10a_an_exile_sweep_is_no_death() {
    let mut g = main_phase();
    let bastion = g.add_card_to_battlefield(0, catalog::bastion_of_remembrance());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve(
        &mut g,
        bastion,
        Effect::Move {
            what: Selector::EachPermanent(R::Enchantment.or(R::Creature)),
            to: ZoneDest::Exile,
        },
    );
    assert_eq!((g.players[0].life, g.players[1].life), (20, 20));
}
