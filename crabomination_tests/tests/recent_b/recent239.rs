//! Functionality tests for `catalog::sets::decks::recent239`.

use crabomination::card::{AdditionalCastCost, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef, Predicate};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game};

/// Betrayer's Bargain deals 5 and exiles the lethal creature instead of
/// burying it, and carries the sacrifice-or-pay additional cost.
#[test]
fn betrayers_bargain_exiles_lethal() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let def = catalog::betrayers_bargain();
    assert!(matches!(
        def.additional_cast_cost[0],
        AdditionalCastCost::SacrificeOrPay { pay: 2, .. }
    ));
    let ctx = EffectContext { targets: vec![Target::Permanent(victim)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&def.effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "lethal creature exiled, not buried");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == victim));
}

/// Untimely Malfunction's third mode keeps one or two creatures from blocking.
#[test]
fn untimely_malfunction_cant_block_mode() {
    let mut g = two_player_game();
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let modes = match &catalog::untimely_malfunction().effect {
        Effect::ChooseMode(m) => m.clone(),
        _ => panic!("not modal"),
    };
    let ctx = EffectContext { targets: vec![Target::Permanent(blocker)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&modes[2], &ctx).unwrap();
    assert!(g.computed_permanent(blocker).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Untimely Malfunction's first mode destroys an artifact.
#[test]
fn untimely_malfunction_destroy_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    let modes = match &catalog::untimely_malfunction().effect {
        Effect::ChooseMode(m) => m.clone(),
        _ => panic!("not modal"),
    };
    let ctx = EffectContext { targets: vec![Target::Permanent(art)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&modes[0], &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
}

/// With delirium, Omnivorous Flytrap's ETB distributes two +1/+1 counters; at
/// six card types it doubles them on the same creatures.
#[test]
fn omnivorous_flytrap_delirium_counters() {
    let mut g = two_player_game();
    // Six distinct card types in the graveyard.
    g.add_card_to_graveyard(0, catalog::forest()); // Land
    g.add_card_to_graveyard(0, catalog::lightning_strike()); // Instant
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // Creature
    g.add_card_to_graveyard(0, catalog::sol_ring()); // Artifact
    g.add_card_to_graveyard(0, catalog::divination()); // Sorcery
    g.add_card_to_graveyard(0, catalog::pacifism()); // Enchantment
    assert!(g.distinct_card_types_in_graveyard(0) >= 6);
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let etb = catalog::omnivorous_flytrap().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(target)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&etb, &ctx).unwrap();
    // Two counters distributed onto the single target, then doubled to four.
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

/// The trigger only fires with delirium active (four+ card types).
#[test]
fn omnivorous_flytrap_delirium_gate() {
    let filter = catalog::omnivorous_flytrap().triggered_abilities[0].event.filter.clone();
    assert!(matches!(filter, Some(Predicate::DeliriumActive { who: PlayerRef::You })));
}

/// Norin can't block, and his blocked-creature trigger exiles the trigger
/// source and grants a play-from-exile window.
#[test]
fn norin_exiles_blocked_creature() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let norin = catalog::norin_swift_survivalist();
    assert!(norin.keywords.contains(&Keyword::CantBlock));
    let effect = norin.triggered_abilities[0].effect.clone();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = EffectContext::for_trigger(ally, 0, None, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.exile.iter().any(|c| c.id == ally), "blocked creature exiled");
}
