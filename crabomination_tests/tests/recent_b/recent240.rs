//! Functionality tests for `catalog::sets::decks::recent240`.

use crabomination::card::AdditionalCastCost;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::Effect;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game};

/// Fear of Abduction exiles an opponent's creature until it leaves, then hands
/// it back to its owner — and it carries the exile-a-creature additional cost.
#[test]
fn fear_of_abduction_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let fear = g.add_card_to_battlefield(0, catalog::fear_of_abduction());
    assert!(matches!(
        catalog::fear_of_abduction().additional_cast_cost[0],
        AdditionalCastCost::ExilePermanent { count: 1, .. }
    ));
    g.fire_self_etb_triggers(fear, 0);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "opponent's creature exiled");
    g.remove_from_battlefield_to_graveyard_raw(fear);
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "returns to owner's hand on leave");
}

/// Say Its Name mills three, then returns a creature (or land) from the
/// graveyard to hand.
#[test]
fn say_its_name_mills_then_returns() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::say_its_name().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].graveyard.iter().filter(|c| c.definition.name == "Forest").count(),
        3,
        "milled three cards"
    );
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature returned to hand");
}

/// Veteran Survivor gains +3/+3 and hexproof once three cards are exiled with
/// it via its Survival ability.
#[test]
fn veteran_survivor_buffs_at_three_exiled() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let vet = g.add_card_to_battlefield(0, catalog::veteran_survivor());
    // Baseline: 2/1, no hexproof.
    let c = g.computed_permanent(vet).unwrap();
    assert_eq!((c.power, c.toughness), (2, 1));
    assert!(!c.keywords.contains(&Keyword::Hexproof));
    // Exile three cards stamped with the survivor as their source.
    for _ in 0..3 {
        let card = g.add_card_to_exile(1, catalog::grizzly_bears());
        g.exile.iter_mut().find(|c| c.id == card).unwrap().exiled_with = Some(vet);
    }
    let c = g.computed_permanent(vet).unwrap();
    assert_eq!((c.power, c.toughness), (5, 4), "+3/+3 at three exiled");
    assert!(c.keywords.contains(&Keyword::Hexproof), "hexproof at three exiled");
}

/// Coordinated Clobbering taps both chosen creatures and makes each deal its
/// power to the opponent's creature.
#[test]
fn coordinated_clobbering_two_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let opp = g.add_card_to_battlefield(1, catalog::avenger_of_zendikar()); // 5/5
    // Slots: 0 = a, 1 = opp, 2 = b.
    let ctx = EffectContext {
        targets: vec![Target::Permanent(a), Target::Permanent(opp), Target::Permanent(b)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    let body = match &catalog::coordinated_clobbering().effect {
        Effect::OptionalTargets { body, .. } => (**body).clone(),
        _ => panic!("not OptionalTargets"),
    };
    g.resolve_effect(&body, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped, "first creature tapped");
    assert!(g.battlefield_find(b).unwrap().tapped, "second creature tapped");
    assert_eq!(g.battlefield_find(opp).unwrap().damage, 4, "2 + 2 damage dealt");
}

/// Waltz of Rage's chosen creature deals its power to every other creature.
#[test]
fn waltz_of_rage_radiates() {
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::avenger_of_zendikar()); // 5/5
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let ctx = EffectContext {
        targets: vec![Target::Permanent(hero)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&catalog::waltz_of_rage().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(hero).is_some(), "source is not hit by itself");
    assert!(g.battlefield_find(ally).is_none(), "friendly creature took 5 and died");
    assert!(g.battlefield_find(enemy).is_none(), "enemy creature took 5 and died");
}

/// After Waltz of Rage resolves, a creature you control dying exiles the top of
/// your library (impulse-play window).
#[test]
fn waltz_of_rage_impulses_on_death() {
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::avenger_of_zendikar());
    let ctx = EffectContext {
        targets: vec![Target::Permanent(hero)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&catalog::waltz_of_rage().effect, &ctx).unwrap();
    drain_stack(&mut g);
    // Now a creature you control dies — the delayed trigger exiles the top card.
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let top = g.add_card_to_library(0, catalog::forest());
    g.battlefield_find_mut(victim).unwrap().damage = 5;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == top), "top card exiled on the death");
}
