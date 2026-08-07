//! CR conformance for this run's engine work:
//! - CR 120.8 — a source that would deal 0 damage deals none, so
//!   damage-triggered abilities don't fire.
//! - CR 514.2 — cleanup removes damage marked on *phased-out* permanents too.
//! - CR 121.5 — moving cards from library to hand without the word "draw"
//!   isn't a draw: no draw triggers, no draw tally.

use crabomination::card::{
    CardDefinition, CardId, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crabomination::game::*;
use crabomination::mana::{cost, generic};

/// A 2/2 that counts every time it's dealt damage.
fn damage_watcher() -> CardDefinition {
    CardDefinition {
        name: "Damage Watcher",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

fn resolve(g: &mut GameState, source: CardId, effect: &Effect) {
    let ctx = crabomination::game::effects::EffectContext::for_ability(source, 0, None);
    let mut evs = g.resolve_effect(effect, &ctx).expect("resolve");
    evs.extend(g.check_state_based_actions());
    g.dispatch_triggers_for_events(&evs);
    while !g.stack.is_empty() {
        g.resolve_top_of_stack().expect("resolve stack");
    }
}

/// CR 120.8 — zero damage is no damage at all; nothing triggers.
#[test]
fn cr_120_8_zero_damage_does_not_trigger() {
    let mut g = two_player_game();
    let watcher = g.add_card_to_battlefield(0, damage_watcher());
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for (amount, want) in [(0, 0u32), (1, 1)] {
        resolve(
            &mut g,
            src,
            &Effect::DealDamage {
                to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Golem)),
                amount: Value::Const(amount),
            },
        );
        assert_eq!(
            g.battlefield_find(watcher).unwrap().counter_count(CounterType::Charge),
            want,
            "{amount} damage"
        );
    }
}

/// CR 514.2 — a phased-out permanent's marked damage clears at cleanup too.
#[test]
fn cr_514_2_cleanup_clears_phased_out_damage() {
    let mut g = two_player_game();
    let hidden = g.add_card_to_battlefield(0, catalog::serra_angel());
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve(
        &mut g,
        src,
        &Effect::DealDamage {
            to: Selector::EachPermanent(R::HasKeyword(Keyword::Flying)),
            amount: Value::Const(2),
        },
    );
    assert_eq!(g.battlefield_find(hidden).unwrap().damage, 2);
    resolve(
        &mut g,
        src,
        &Effect::PhaseOut {
            what: Selector::EachPermanent(R::HasKeyword(Keyword::Flying)),
            until_source_leaves: false,
        },
    );
    assert!(g.battlefield_find(hidden).is_none(), "it's out of phase");
    let mut evs = Vec::new();
    g.do_cleanup(&mut evs);
    let out = g.phased_out.iter().find(|c| c.id == hidden).expect("still phased out");
    assert_eq!(out.damage, 0, "cleanup reaches the phased-out zone");
}

/// CR 121.5 — library → hand without "draw" is not a draw.
#[test]
fn cr_121_5_moving_to_hand_is_not_a_draw() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::plains());
    let before = g.players[0].cards_drawn_this_turn;
    resolve(
        &mut g,
        src,
        &Effect::Move {
            what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
            to: ZoneDest::Hand(PlayerRef::You),
        },
    );
    assert_eq!(g.players[0].hand.len(), 1, "the card moved");
    assert_eq!(g.players[0].cards_drawn_this_turn, before, "but it wasn't drawn");
    assert!(g.players[0].last_drawn_card.is_none());
}
