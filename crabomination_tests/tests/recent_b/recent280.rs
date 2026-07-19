//! Functionality tests for `catalog::sets::decks::recent280`.

use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::{drain_stack, two_player_game, Target};

/// Brandywine Farmer makes a Food on entry and another on leaving.
#[test]
fn brandywine_farmer_food_on_etb_and_ltb() {
    let mut g = two_player_game();
    let bf = g.add_card_to_battlefield(0, catalog::brandywine_farmer());
    g.resolve_effect(&catalog::brandywine_farmer().triggered_abilities[0].effect.clone(), &EffectContext::for_ability(bf, 0, None)).unwrap();
    g.resolve_effect(&catalog::brandywine_farmer().triggered_abilities[1].effect.clone(), &EffectContext::for_ability(bf, 0, None)).unwrap();
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Food").count(), 2, "two Food tokens");
}

/// Captain of Umbar loots.
#[test]
fn captain_of_umbar_loots() {
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::captain_of_umbar());
    g.add_card_to_library(0, catalog::forest());
    let dump = g.add_card_to_hand(0, catalog::island());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Discard(vec![dump]),
    ]));
    let hand = g.players[0].hand.len();
    g.resolve_effect(&catalog::captain_of_umbar().activated_abilities[0].effect.clone(), &EffectContext::for_ability(cap, 0, None)).unwrap();
    assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one");
    assert_eq!(g.players[0].graveyard.len(), 1, "one card discarded");
}

/// Chance-Met Elves grows when its controller scries.
#[test]
fn chance_met_elves_grows_on_scry() {
    let mut g = two_player_game();
    let e = g.add_card_to_battlefield(0, catalog::chance_met_elves());
    g.resolve_effect(&catalog::chance_met_elves().triggered_abilities[0].effect.clone(), &EffectContext::for_ability(e, 0, None)).unwrap();
    assert_eq!(g.computed_permanent(e).unwrap().power, 4, "3/2 → 4/3");
}

/// Claim the Precious destroys and tempts the Ring.
#[test]
fn claim_the_precious_destroys_and_tempts() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext { targets: vec![Target::Permanent(victim)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::claim_the_precious().effect.clone(), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "creature destroyed");
    assert!(g.players[0].ring_temptations > 0, "the Ring tempted");
}

/// Dreadful as the Storm sets base P/T to 5/5.
#[test]
fn dreadful_as_the_storm_sets_base_pt() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::dreadful_as_the_storm().effect.clone(), &ctx).unwrap();
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!((p.power, p.toughness), (5, 5), "base P/T becomes 5/5");
}

/// Cirith Ungol Patrol sacrifices a creature to draw and make Food.
#[test]
fn cirith_ungol_patrol_sac_draws_and_makes_food() {
    let mut g = two_player_game();
    let patrol = g.add_card_to_battlefield(0, catalog::cirith_ungol_patrol());
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    g.resolve_effect(&catalog::cirith_ungol_patrol().activated_abilities[0].effect.clone(), &EffectContext::for_ability(patrol, 0, None)).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"), "made a Food");
}

/// Breaking of the Fellowship turns an opponent's creature on another.
#[test]
fn breaking_of_the_fellowship_forces_a_fight() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let ctx = EffectContext { targets: vec![Target::Permanent(attacker), Target::Permanent(victim)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::breaking_of_the_fellowship().effect.clone(), &ctx).unwrap();
    g.check_state_based_actions();
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "victim took 2 and died");
    assert!(g.players[0].ring_temptations > 0, "the Ring tempted");
}

/// Deceive the Messenger weakens a creature and amasses Orcs.
#[test]
fn deceive_the_messenger_debuffs_and_amasses() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::deceive_the_messenger().effect.clone(), &ctx).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, -1, "2/2 → -3/-0 leaves power -1");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Army)), "an Orc Army exists");
}
