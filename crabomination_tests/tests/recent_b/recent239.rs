//! Functionality tests for `catalog::sets::decks::recent239`.

use crabomination::card::{AdditionalCastCost, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef, Predicate};
use crabomination::game::GameAction;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

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

/// Kutzil's Flanker's second mode gains life and scries; its third exiles a
/// target player's graveyard.
#[test]
fn kutzils_flanker_modes() {
    let mut g = two_player_game();
    let flanker = g.add_card_to_battlefield(0, catalog::kutzils_flanker());
    assert!(catalog::kutzils_flanker().keywords.contains(&Keyword::Flash));
    let modes = match &catalog::kutzils_flanker().triggered_abilities[0].effect {
        Effect::ChooseMode(m) => m.clone(),
        _ => panic!("not modal"),
    };
    let life0 = g.players[0].life;
    g.resolve_effect(&modes[1], &EffectContext::for_trigger(flanker, 0, None, 0)).unwrap();
    assert_eq!(g.players[0].life, life0 + 2, "gained 2 life");
    // Mode 3 exiles a target player's graveyard.
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.resolve_effect(&modes[2], &EffectContext::for_trigger(flanker, 0, Some(Target::Player(1)), 0)).unwrap();
    assert!(g.players[1].graveyard.is_empty(), "opponent graveyard exiled");
}

/// Stubborn Burrowfiend's saddle trigger mills two and pumps by graveyard
/// creatures, and it fires only once per turn.
#[test]
fn stubborn_burrowfiend_saddle_mill_and_pump() {
    use crabomination::effect::EventKind;
    let mut g = two_player_game();
    let fiend = g.add_card_to_battlefield(0, catalog::stubborn_burrowfiend()); // 2/2
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // 2 creatures → X=2
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); } // mill lands, X unchanged
    let def = catalog::stubborn_burrowfiend();
    assert_eq!(def.triggered_abilities[0].event.kind, EventKind::CrewsOrSaddles);
    assert!(def.triggered_abilities[0].event.once_per_turn, "first-time-each-turn gate");
    g.resolve_effect(&def.triggered_abilities[0].effect, &EffectContext::for_trigger(fiend, 0, None, 0)).unwrap();
    let c = g.computed_permanent(fiend).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "2/2 + X/X (X = 2 graveyard creatures)");
}

/// Unscrupulous Contractor's ETB sacrifice draws two and drains the target.
#[test]
fn unscrupulous_contractor_sacrifice_draws_and_drains() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::unscrupulous_contractor());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let (life0, hand0) = (g.players[0].life, g.players[0].hand.len());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let etb = catalog::unscrupulous_contractor().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_trigger(src, 0, Some(Target::Player(0)), 0);
    g.resolve_effect(&etb, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "target drew 2");
    assert_eq!(g.players[0].life, life0 - 2, "target lost 2");
}

/// Outlaw Stitcher makes a 2/2 Zombie Rogue that grows by two per extra spell
/// cast this turn, and it's plottable.
#[test]
fn outlaw_stitcher_token_scales_with_spells() {
    let mut g = two_player_game();
    g.players[0].spells_cast_this_turn = 3; // Stitcher + 2 others → 2 extra
    assert!(catalog::outlaw_stitcher().plot_cost.is_some());
    let src = g.add_card_to_battlefield(0, catalog::outlaw_stitcher());
    let etb = catalog::outlaw_stitcher().triggered_abilities[0].effect.clone();
    g.resolve_effect(&etb, &EffectContext::for_trigger(src, 0, None, 0)).unwrap();
    let token = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Zombie Rogue").expect("token made");
    // base 2/2 + 2 counters * (3 - 1) = four counters.
    assert_eq!(token.counter_count(CounterType::PlusOnePlusOne), 4);
}

/// Tumbleweed Rising makes an Elemental whose power tracks your biggest
/// creature, and it's plottable.
#[test]
fn tumbleweed_rising_makes_dynamic_token() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 → X = 4
    assert!(catalog::tumbleweed_rising().plot_cost.is_some());
    g.resolve_effect(&catalog::tumbleweed_rising().effect, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    let token = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Elemental").expect("token made");
    let id = token.id;
    assert_eq!(g.computed_permanent(id).unwrap().power, 4, "X/X = greatest power you control");
}

/// Bite Down on Crime pumps your creature and fights an enemy for its power.
#[test]
fn bite_down_on_crime_pumps_and_fights() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 4/2
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies to 4
    let ctx = EffectContext {
        targets: vec![Target::Permanent(mine), Target::Permanent(enemy)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&catalog::bite_down_on_crime().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 4, "+2/+0");
    assert!(g.battlefield_find(enemy).is_none(), "took 4 and died");
}

/// Trial of Agony burns one creature and locks the other out of blocking.
#[test]
fn trial_of_agony_burns_and_locks() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies to 5
    let b = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 survives
    let ctx = EffectContext {
        targets: vec![Target::Permanent(a), Target::Permanent(b)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&catalog::trial_of_agony().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none(), "took 5 and died");
    assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Getaway Glamer's first mode blinks a creature (exiles it now).
#[test]
fn getaway_glamer_blink_mode() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blink = match &catalog::getaway_glamer().effect {
        Effect::Spree { modes } => modes[0].effect.clone(),
        _ => panic!("not spree"),
    };
    let ctx = EffectContext { targets: vec![Target::Permanent(c)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&blink, &ctx).unwrap();
    assert!(g.battlefield_find(c).is_none(), "creature exiled by the blink");
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

/// Bite Down on Crime collects evidence 6 for a {2} discount: with 6+ mana
/// value in the graveyard the {3}{G} sorcery casts for {1}{G}, exiling the
/// evidence, and its collect flag is stamped.
#[test]
fn bite_down_on_crime_evidence_discount() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); } // MV 2 × 3 = 6
    let spell = g.add_card_to_hand(0, catalog::bite_down_on_crime());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1); // only {1}{G} — 2 short of {3}{G}
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    }).expect("collect-evidence discount pays for the cast");
    drain_stack(&mut g);
    // Evidence exiled (3 grizzlies) and the pumped 4/2 killed the 2/2.
    assert_eq!(g.exile.iter().filter(|c| c.owner == 0).count(), 3, "evidence exiled");
    assert!(g.battlefield_find(theirs).is_none(), "4-power hit killed the 2/2");
}

/// Without graveyard fuel the discount can't apply: {1}{G} is short of {3}{G}.
#[test]
fn bite_down_on_crime_no_evidence_no_discount() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::bite_down_on_crime());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    }).is_err(), "empty graveyard means the full three-generic-plus-green cost");
}

/// Behind the Mask makes the target 4/3 with no evidence, 1/1 with evidence.
#[test]
fn behind_the_mask_evidence_flips_pt() {
    // No evidence → 4/3.
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::behind_the_mask());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Behind the Mask");
    drain_stack(&mut g);
    let c = g.computed_permanent(target).unwrap();
    assert_eq!((c.power, c.toughness), (4, 3), "4/3 without evidence");

    // Evidence collected → 1/1.
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); } // MV 6
    let spell = g.add_card_to_hand(0, catalog::behind_the_mask());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Behind the Mask with evidence");
    drain_stack(&mut g);
    let c = g.computed_permanent(target).unwrap();
    assert_eq!((c.power, c.toughness), (1, 1), "1/1 with evidence");
}

/// Analyze the Pollen fetches a basic land normally, but with evidence its
/// `If` branch widens the search filter to creature-or-land — so a Grizzly
/// Bears in the library becomes a legal pick.
#[test]
fn analyze_the_pollen_evidence_widens_search() {
    use crabomination::effect::Effect;
    // Evidence collected → creature is a legal search pick.
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.cast_collected_evidence = true;
    g.resolve_effect(&catalog::analyze_the_pollen().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature fetched with evidence");

    // Without evidence the same creature pick is illegal (basic-land only).
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
    let else_ = match catalog::analyze_the_pollen().effect {
        Effect::If { else_, .. } => *else_,
        _ => panic!("not an If"),
    };
    g.resolve_effect(&else_, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    drain_stack(&mut g);
    assert!(!g.players[0].hand.iter().any(|c| c.id == bear), "creature is not a basic land");
}

/// Paranormal Analyst returns the card milled by manifest dread to hand.
#[test]
fn paranormal_analyst_returns_milled_card() {
    use crabomination::effect::{Effect, PlayerRef};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::paranormal_analyst());
    // Two cards on top: the analyst manifests one, mills the other, and its
    // trigger returns that milled card to hand.
    let manifested = g.add_card_to_library(0, catalog::grizzly_bears());
    let milled = g.add_card_to_library(0, catalog::forest());
    // Library is a stack — ensure `manifested` is on top so `forest` is the mill.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![manifested])]));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&Effect::ManifestDread { who: PlayerRef::You }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == milled),
        "milled card returned to hand by Paranormal Analyst");
    assert!(g.battlefield.iter().any(|c| c.id == manifested && c.face_down),
        "the chosen card is manifested face down");
}

/// Oblivious Bookworm draws then discards when you had no face-down activity,
/// but keeps the drawn card when a permanent entered face down this turn.
#[test]
fn oblivious_bookworm_discard_unless_face_down_activity() {
    // No face-down activity → draw then discard (net hand unchanged).
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let effect = catalog::oblivious_bookworm().triggered_abilities[0].effect.clone();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let before = g.players[0].hand.len();
    g.resolve_effect(&effect, &EffectContext::for_trigger(crabomination::card::CardId(0), 0, None, 0)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before, "drew one, discarded one");

    // A permanent entered face down this turn → no discard (net +1 hand).
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].face_down_activity_this_turn = true;
    let effect = catalog::oblivious_bookworm().triggered_abilities[0].effect.clone();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let before = g.players[0].hand.len();
    g.resolve_effect(&effect, &EffectContext::for_trigger(crabomination::card::CardId(0), 0, None, 0)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "kept the drawn card");
}

/// Monstrous Emergence deals damage equal to the chosen creature's power (its
/// choose-a-creature additional cost picks the highest-power creature you
/// control).
#[test]
fn monstrous_emergence_deals_chosen_power() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let _ = big;
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let spell = g.add_card_to_hand(0, catalog::monstrous_emergence());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Monstrous Emergence");
    drain_stack(&mut g);
    // Highest-power creature is the 3/3 Hill Giant → 3 damage kills the 3/3.
    assert!(g.battlefield_find(victim).is_none(), "3 damage (chosen creature's power) killed the 3/3");
}

/// The additional cost is unpayable with no creature to choose or reveal.
#[test]
fn monstrous_emergence_needs_a_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::monstrous_emergence());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no creature to choose or reveal");
}

/// Leyline of Hope may begin in play, boosts life gain by 1, and anthems your
/// team once you're 7+ life above your starting total.
#[test]
fn leyline_of_hope_lifegain_and_anthem() {
    use crabomination::effect::OpeningHandEffect;
    let def = catalog::leyline_of_hope();
    assert!(matches!(def.opening_hand, Some(OpeningHandEffect::StartInPlay { .. })));
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leyline_of_hope());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    // Life gain of 3 becomes 4 (bonus +1).
    let start = g.players[0].life;
    g.adjust_life(0, 3);
    assert_eq!(g.players[0].life, start + 4, "life gain boosted by 1");
    // Below the +7 threshold the team isn't pumped.
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "no anthem yet");
    // Push to 7+ above starting → +2/+2 anthem.
    g.players[0].life = g.players[0].starting_life + 7;
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "anthem online at 7+ above starting");
}

/// Creeping Peeper taps for {U} that only casts enchantment spells.
#[test]
fn creeping_peeper_enchantment_only_mana() {
    use crabomination::effect::{Effect, ManaPayload};
    use crabomination::mana::SpendRestriction;
    let def = catalog::creeping_peeper();
    // The ability adds enchantment-restricted blue mana.
    match &def.activated_abilities[0].effect {
        Effect::AddMana { pool: ManaPayload::Restricted(_, r), .. } => {
            assert_eq!(*r, SpendRestriction::EnchantmentSpell);
        }
        _ => panic!("not enchantment-restricted mana"),
    }
    // The restriction admits an enchantment spell but not a creature spell.
    assert!(SpendRestriction::EnchantmentSpell.allows(&catalog::pacifism().spell_kind()));
    assert!(!SpendRestriction::EnchantmentSpell.allows(&catalog::grizzly_bears().spell_kind()));
}

/// Fear of Burning Alive burns each opponent for 4 on ETB, and with delirium its
/// noncombat-damage trigger copies the damage onto an opponent's creature.
#[test]
fn fear_of_burning_alive_etb_and_delirium_copy() {
    use crabomination::effect::{EventKind, EventScope};
    let mut g = two_player_game();
    let etb = catalog::fear_of_burning_alive().triggered_abilities[0].effect.clone();
    let before = g.players[1].life;
    g.resolve_effect(&etb, &EffectContext::for_trigger(crabomination::card::CardId(0), 0, None, 0)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 4, "each opponent burned for 4");

    // Delirium trigger is a noncombat-damage listener; it copies the amount.
    let def = catalog::fear_of_burning_alive();
    assert_eq!(def.triggered_abilities[1].event.kind, EventKind::PlayerDealtNoncombatDamage);
    assert!(matches!(def.triggered_abilities[1].event.scope, EventScope::OpponentControl));
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::fear_of_burning_alive());
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let copy = def.triggered_abilities[1].effect.clone();
    let mut ctx = EffectContext { targets: vec![Target::Permanent(victim)], ..EffectContext::for_trigger(src, 0, None, 0) };
    ctx.event_amount = 5; // an earlier source dealt 5 noncombat
    g.resolve_effect(&copy, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "5 copied damage killed the 3/3");
}

/// Mudflat Village's sac ability returns a Rat card from the graveyard to hand;
/// its {B} ability is creature-spell restricted.
#[test]
fn mudflat_village_returns_kindred_and_restricts_mana() {
    use crabomination::effect::{Effect, ManaPayload};
    use crabomination::mana::SpendRestriction;
    let def = catalog::mudflat_village();
    // The {B} ability is creature-only.
    match &def.activated_abilities[1].effect {
        Effect::AddMana { pool: ManaPayload::Restricted(_, r), .. } => {
            assert_eq!(*r, SpendRestriction::CreatureOnly);
        }
        _ => panic!("not creature-restricted mana"),
    }
    // Sac ability (tap + sacrifice + {1}{B}) returns a matching Rat to hand.
    let ab = &def.activated_abilities[2];
    assert!(ab.tap_cost && ab.sac_cost, "tap + sacrifice cost");
    assert_eq!(ab.mana_cost.cmc(), 2, "one-generic-plus-black activation");
    let mut g = two_player_game();
    let rat = g.add_card_to_graveyard(0, catalog::typhoid_rats());
    let ctx = EffectContext { targets: vec![Target::Permanent(rat)], ..EffectContext::for_ability(rat, 0, None) };
    g.resolve_effect(&ab.effect, &ctx).unwrap();
    assert!(g.players[0].hand.iter().any(|c| c.id == rat), "Rat returned to hand");
}

/// Oakhollow Village's {G} ability counters only your kindred creatures that
/// entered this turn.
#[test]
fn oakhollow_village_counters_kindred_entered_this_turn() {
    let mut g = two_player_game();
    g.turn_number = 5;
    // A Frog that entered this turn (kindred), a Rat this turn (not kindred),
    // and a Frog that entered earlier.
    let fresh_frog = g.add_card_to_battlefield(0, catalog::spore_frog());
    let rat = g.add_card_to_battlefield(0, catalog::typhoid_rats());
    let old_frog = g.add_card_to_battlefield(0, catalog::spore_frog());
    g.battlefield_find_mut(fresh_frog).unwrap().entered_turn = Some(5);
    g.battlefield_find_mut(rat).unwrap().entered_turn = Some(5);
    g.battlefield_find_mut(old_frog).unwrap().entered_turn = Some(1);
    let ab = catalog::oakhollow_village().activated_abilities[2].effect.clone();
    g.resolve_effect(&ab, &EffectContext::for_ability(crabomination::card::CardId(0), 0, None)).unwrap();
    assert_eq!(g.battlefield_find(fresh_frog).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "kindred creature that entered this turn gets a counter");
    assert_eq!(g.battlefield_find(rat).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
        "non-kindred creature untouched");
    assert_eq!(g.battlefield_find(old_frog).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
        "kindred creature that entered earlier untouched");
}

/// Lupinflower Village's sac ability digs six deep for a Rabbit and bottoms the
/// rest; its {W} ability is creature-restricted.
#[test]
fn lupinflower_village_digs_for_kindred() {
    use crabomination::effect::{Effect, ManaPayload};
    use crabomination::mana::SpendRestriction;
    let def = catalog::lupinflower_village();
    match &def.activated_abilities[1].effect {
        Effect::AddMana { pool: ManaPayload::Restricted(_, r), .. } => {
            assert_eq!(*r, SpendRestriction::CreatureOnly);
        }
        _ => panic!("not creature-restricted mana"),
    }
    // Sac ability digs six deep for a kindred card, bottoming the rest at random.
    let ab = &def.activated_abilities[2];
    assert!(ab.tap_cost && ab.sac_cost);
    match &ab.effect {
        Effect::LookPickToHand { count, pick_filter: Some(_), rest_bottom_random: true, optional: true, .. } => {
            assert!(matches!(count, crabomination::effect::Value::Const(6)));
        }
        _ => panic!("not a six-deep kindred dig"),
    }
}

/// Lilypad Village's surveil ability is gated on controlling a kindred creature
/// that entered this turn.
#[test]
fn lilypad_village_surveil_gate() {
    use crabomination::effect::Effect;
    let def = catalog::lilypad_village();
    let ab = &def.activated_abilities[2];
    assert!(matches!(ab.effect, Effect::Surveil { .. }));
    assert!(ab.condition.is_some(), "gated on a kindred entered-this-turn");
    // The gate is unmet with no kindred on the board.
    let mut g = two_player_game();
    g.turn_number = 3;
    let ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    assert!(!g.evaluate_predicate(ab.condition.as_ref().unwrap(), &ctx), "no kindred → gate closed");
    // With a Frog that entered this turn, the gate opens.
    let frog = g.add_card_to_battlefield(0, catalog::spore_frog());
    g.battlefield_find_mut(frog).unwrap().entered_turn = Some(3);
    assert!(g.evaluate_predicate(ab.condition.as_ref().unwrap(), &ctx), "kindred present → gate open");
}

/// Rockface Village's sorcery-speed ability pumps and hastes a kindred creature.
#[test]
fn rockface_village_pumps_and_hastes_kindred() {
    let mut g = two_player_game();
    let lizard = g.add_card_to_battlefield(0, catalog::viashino_pyromancer()); // Lizard 2/1
    let ab = catalog::rockface_village().activated_abilities[2].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(lizard)], ..EffectContext::for_ability(crabomination::card::CardId(0), 0, None) };
    g.resolve_effect(&ab, &ctx).unwrap();
    let c = g.computed_permanent(lizard).unwrap();
    assert_eq!(c.power, 3, "+1/+0 applied");
    assert!(c.keywords.contains(&Keyword::Haste), "gained haste");
}

/// Whiskervale Forerunner's Valiant trigger digs five deep for a small creature
/// and deploys it; the trigger is once-per-turn on becoming your target.
#[test]
fn whiskervale_forerunner_valiant_dig() {
    use crabomination::effect::{Effect, EventKind};
    let def = catalog::whiskervale_forerunner();
    let t = &def.triggered_abilities[0];
    assert_eq!(t.event.kind, EventKind::BecameTarget);
    assert!(t.event.once_per_turn, "first time each turn");
    match &t.effect {
        Effect::LookPickToHand { count, pick_filter: Some(_), to_battlefield: true, rest_bottom_random: true, .. } => {
            assert!(matches!(count, crabomination::effect::Value::Const(5)));
        }
        _ => panic!("not a five-deep creature dig"),
    }
    // The dig deploys a small creature from the top of the library.
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::whiskervale_forerunner());
    let small = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 creature
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![small])]));
    g.resolve_effect(&t.effect, &EffectContext::for_trigger(src, 0, None, 0)).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == small), "small creature deployed");
}

/// Hollow Marauder costs less per graveyard creature, and its ETB draws only
/// when the opponent's discard was cheap (MV ≤ 3).
#[test]
fn hollow_marauder_cost_and_conditional_draw() {
    use crabomination::card::SelectionRequirement as R;
    let def = catalog::hollow_marauder();
    assert_eq!(def.affinity_graveyard_filter, Some(R::Creature));
    assert!(def.keywords.contains(&Keyword::Flying));
    // Cheap discard → draw.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears()); // MV 2
    let src = g.add_card_to_battlefield(0, catalog::hollow_marauder());
    let etb = def.triggered_abilities[0].effect.clone();
    let before = g.players[0].hand.len();
    g.resolve_effect(&etb, &EffectContext::for_trigger(src, 0, None, 0)).unwrap();
    assert_eq!(g.players[0].hand.len(), before + 1, "drew after a cheap discard");
    // Expensive discard → no draw.
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::serra_angel()); // MV 5
    let src = g.add_card_to_battlefield(0, catalog::hollow_marauder());
    let before = g.players[0].hand.len();
    g.resolve_effect(&def.triggered_abilities[0].effect.clone(), &EffectContext::for_trigger(src, 0, None, 0)).unwrap();
    assert_eq!(g.players[0].hand.len(), before, "no draw after an expensive discard");
}

/// Feed the Cycle forages (exiling three graveyard cards) to pay its additional
/// cost, then destroys a creature; with no forage material the pay is folded.
#[test]
fn feed_the_cycle_forage_or_pay() {
    use crabomination::card::AdditionalCastCost;
    let def = catalog::feed_the_cycle();
    assert!(matches!(def.additional_cast_cost[0], AdditionalCastCost::ForageOrPay { pay: 1 }));
    // With three graveyard cards, cast forages (no extra mana) and destroys.
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::feed_the_cycle());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1); // exactly {1}{B}
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("forage pays the additional cost");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature destroyed");
    assert_eq!(g.exile.iter().filter(|c| c.owner == 0).count(), 3, "three cards foraged");

    // With no forage material the pay ({1}) is folded — {1}{B} is one short.
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::feed_the_cycle());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no forage material means the folded generic makes it unaffordable");
}

/// Freestrider Commando: a hardcast (mana spent) enters as a vanilla 3/3; a
/// free/plotted cast (no mana spent) enters with two +1/+1 counters.
#[test]
fn freestrider_commando_counters_only_when_free() {
    // Hardcast for {2}{G} → no counters.
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let spell = g.add_card_to_hand(0, catalog::freestrider_commando());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("hardcast");
    drain_stack(&mut g);
    let c = g.battlefield_find(spell).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 0, "hardcast is a vanilla 3/3");

    // The ETB gates the counters on "no mana spent" (Not CastSpellManaSpentAtLeast
    // 1) — so a free/plotted cast (0 spent) or a non-cast entry qualifies. That
    // hardcast above proves the gate reads the cast's mana spend at ETB time.
    let def = catalog::freestrider_commando();
    assert!(def.plot_cost.is_some(), "has a Plot cost");
    match &def.triggered_abilities[0].effect {
        Effect::If { cond: Predicate::Not(inner), .. } => {
            assert!(matches!(**inner, Predicate::CastSpellManaSpentAtLeast(1)));
        }
        _ => panic!("not a no-mana-spent gate"),
    }
}
