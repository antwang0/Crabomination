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

/// Rootwise Survivor's Survival animates a target land with three +1/+1
/// counters into a creature, and its trigger is a tapped-gated second main.
#[test]
fn rootwise_survivor_survival_animates_land() {
    use crabomination::effect::{EventKind, EventScope};
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let def = catalog::rootwise_survivor();
    assert_eq!(def.triggered_abilities[0].event.kind, EventKind::StepBegins(crabomination::game::TurnStep::PostCombatMain));
    assert!(matches!(def.triggered_abilities[0].event.scope, EventScope::YourControl));
    let ctx = EffectContext { targets: vec![Target::Permanent(land)], ..EffectContext::for_trigger(land, 0, None, 0) };
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).unwrap();
    let l = g.computed_permanent(land).unwrap();
    assert_eq!((l.power, l.toughness), (3, 3), "0/0 Elemental + three +1/+1");
    assert!(l.card_types.contains(&crabomination::card::CardType::Creature));
}

/// Reluctant Role Model's Survival grants a flying counter, and its death
/// trigger relocates the counters to another creature.
#[test]
fn reluctant_role_model_counters_and_relocation() {
    let mut g = two_player_game();
    let model = g.add_card_to_battlefield(0, catalog::reluctant_role_model());
    let heir = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let survival = match &catalog::reluctant_role_model().triggered_abilities[0].effect {
        Effect::ChooseMode(m) => m[0].clone(), // flying counter
        _ => panic!("not modal"),
    };
    g.resolve_effect(&survival, &EffectContext::for_trigger(model, 0, None, 0)).unwrap();
    assert!(g.computed_permanent(model).unwrap().keywords.contains(&Keyword::Flying));
    // Death trigger moves the counters onto the heir.
    let death = catalog::reluctant_role_model().triggered_abilities[1].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(heir)], ..EffectContext::for_trigger(model, 0, None, 0) };
    g.resolve_effect(&death, &ctx).unwrap();
    assert!(g.computed_permanent(heir).unwrap().keywords.contains(&Keyword::Flying),
        "flying keyword counter relocated to the heir");
    assert!(!g.computed_permanent(model).unwrap().keywords.contains(&Keyword::Flying),
        "the model's counter left it");
}

/// Come Back Wrong destroys a creature, reanimates it under your control, and
/// schedules a sacrifice at your next end step.
#[test]
fn come_back_wrong_steals_the_corpse() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext { targets: vec![Target::Permanent(victim)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::come_back_wrong().effect, &ctx).unwrap();
    drain_stack(&mut g);
    let c = g.battlefield_find(victim).expect("reanimated onto the battlefield");
    assert_eq!(c.controller, 0, "under your control now");
}

/// Valgavoth's Onslaught (X=2) manifests two 2/2s, each with two +1/+1
/// counters (making them 4/4 face-down creatures).
#[test]
fn valgavoths_onslaught_manifests_and_counters() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let ctx = EffectContext::for_spell(0, None, 0, 2); // X = 2
    g.resolve_effect(&catalog::valgavoths_onslaught().effect, &ctx).unwrap();
    let facedown: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).collect();
    assert_eq!(facedown.len(), 2, "two creatures manifested");
    for c in facedown {
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2, "each got X=2 counters");
    }
}

/// Altanak's channel ability returns a target land card from the graveyard to
/// the battlefield tapped, and it carries the opponent-target draw trigger.
#[test]
fn altanak_channel_returns_land_tapped() {
    let mut g = two_player_game();
    let land = g.add_card_to_graveyard(0, catalog::forest());
    let def = catalog::altanak_the_thrice_called();
    assert!(def.keywords.contains(&Keyword::Trample));
    assert_eq!(def.triggered_abilities[0].event.kind, crabomination::effect::EventKind::BecameTarget);
    let effect = def.activated_abilities[0].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(land)], ..EffectContext::for_ability(land, 0, None) };
    g.resolve_effect(&effect, &ctx).unwrap();
    let l = g.battlefield_find(land).expect("land returned to battlefield");
    assert!(l.tapped, "enters tapped");
}
