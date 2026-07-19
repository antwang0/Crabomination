//! Functionality tests for `catalog::sets::decks::recent281`.

use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::{drain_stack, two_player_game, Target};

/// Enraged Huorn tempts the Ring on entry.
#[test]
fn enraged_huorn_tempts_the_ring() {
    let mut g = two_player_game();
    let h = g.add_card_to_battlefield(0, catalog::enraged_huorn());
    g.resolve_effect(&catalog::enraged_huorn().triggered_abilities[0].effect.clone(), &EffectContext::for_ability(h, 0, None)).unwrap();
    assert!(g.players[0].ring_temptations > 0, "the Ring tempted");
}

/// Ithilien Kingfisher cantrips on death.
#[test]
fn ithilien_kingfisher_death_draws() {
    let mut g = two_player_game();
    let k = g.add_card_to_battlefield(0, catalog::ithilien_kingfisher());
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    g.battlefield_find_mut(k).unwrap().damage = 100;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew on death");
}

/// Escape from Orthanc pumps toughness, grants flying, and untaps.
#[test]
fn escape_from_orthanc_pumps_and_untaps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::escape_from_orthanc().effect.clone(), &ctx).unwrap();
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!((p.power, p.toughness), (3, 5), "+1/+3");
    assert!(p.keywords.contains(&crabomination::card::Keyword::Flying), "gains flying");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
}

/// Gimli's Fury grants trample only to a legendary target.
#[test]
fn gimlis_fury_trample_for_legends_only() {
    // Nonlegendary: no trample.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::gimlis_fury().effect.clone(), &ctx).unwrap();
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!(p.power, 5, "+3/+2");
    assert!(!p.keywords.contains(&crabomination::card::Keyword::Trample), "no trample for a nonlegend");
}

/// East-Mark Cavalier destroys the Goblin/Orc it damages in combat, and its
/// trigger is gated to those types.
#[test]
fn east_mark_cavalier_slays_orcs() {
    let def = catalog::east_mark_cavalier();
    // Event is filtered to a Goblin or Orc trigger target.
    assert!(
        def.triggered_abilities[0].event.filter.is_some(),
        "the destroy is gated on the damaged creature being a Goblin or Orc",
    );
    // The destroy body kills the damaged creature (an Orc Soldier).
    let mut g = two_player_game();
    let cav = g.add_card_to_battlefield(0, catalog::east_mark_cavalier());
    let orc = g.add_card_to_battlefield(1, catalog::cirith_ungol_patrol()); // Orc Soldier
    assert!(catalog::cirith_ungol_patrol().subtypes.creature_types.contains(&crabomination::card::CreatureType::Orc));
    let ctx = EffectContext { targets: vec![Target::Permanent(orc)], ..EffectContext::for_ability(cav, 0, None) };
    g.resolve_effect(&def.triggered_abilities[0].effect.clone(), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == orc), "the damaged Orc is destroyed");
}
