//! CR conformance for rules exercised by this run's WAR gap wave:
//! CR 614.16 (counter-placement replacements apply to proliferate — Doubling
//! Season / Mowu scale a proliferated counter), CR 603.6d/603.10a (an Aura's
//! leaves-the-battlefield trigger fires from last-known information when its
//! host is exiled — Kaya's Ghostform), and CR 122.1c (a "loyalty counters put
//! on planeswalkers you control" trigger reads the amount placed — Bioessence
//! Hydra).

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::effect::{Effect, Selector, ZoneDest};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, GameEvent};

/// CR 614.16 — proliferate is a counter *placement*, so replacements apply:
/// Doubling Season turns a proliferated +1/+1 into two.
#[test]
fn cr_614_16_proliferate_is_replaced_by_doubling_season() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::doubling_season());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let ctx = EffectContext::for_ability(bear, 0, None);
    g.resolve_effect(&Effect::Proliferate, &ctx).unwrap();
    // Started at 1; proliferate adds one, doubled to two → 3 total.
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3,
        "proliferated counter was doubled by Doubling Season",
    );
}

/// CR 603.6d / 603.10a — an Aura's "when the enchanted permanent leaves" trigger
/// fires from last-known information when its host is exiled (Kaya's Ghostform
/// returns the exiled card).
#[test]
fn cr_603_6d_aura_trigger_fires_when_host_exiled() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::kayas_ghostform());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    let evs = g
        .resolve_effect(&Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }, &ctx)
        .unwrap();
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    g.dispatch_triggers_for_events(&sba);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0),
        "the exiled creature returned to the battlefield under your control",
    );
}

/// CR 122.1c — Bioessence Hydra reads the number of loyalty counters put on
/// your planeswalkers and grows by exactly that many.
#[test]
fn cr_122_1c_loyalty_counters_added_scales_bioessence_hydra() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::the_wanderer());
    let hydra = g.add_card_to_battlefield(0, catalog::bioessence_hydra());
    g.battlefield_find_mut(pw).unwrap().add_counters(CounterType::Loyalty, 3);
    g.dispatch_triggers_for_events(&[GameEvent::CounterAdded {
        card_id: pw,
        counter_type: CounterType::Loyalty,
        count: 3,
    }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3,
        "grew by the number of loyalty counters placed",
    );
}
