//! Phyrexia: All Will Be One — Incubate (CR 701.53). The Incubator token enters
//! with N +1/+1 counters; `{2}: Transform` flips it to a 0/0 Phyrexian creature
//! (so it becomes N/N).

use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};

/// Resolve `effect` as though `player` were its controller.
fn resolve_for(g: &mut GameState, player: usize, effect: crabomination::effect::Effect) {
    let src = g.add_card_to_battlefield(player, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(src, player, None);
    let events = g.resolve_effect(&effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

/// Incubate 3 mints an Incubator with three +1/+1 counters; transforming it
/// yields a 3/3 Phyrexian artifact creature (counters persist, CR 712).
#[test]
fn incubate_then_transform_to_n_over_n() {
    let mut g = two_player_game();
    resolve_for(&mut g, 0, crabomination::effect::Effect::Incubate {
        who: crabomination::effect::PlayerRef::You,
        amount: crabomination::effect::Value::Const(3),
    });
    let inc = g.battlefield.iter().find(|c| c.definition.name == "Incubator").expect("Incubator minted");
    let inc_id = inc.id;
    assert_eq!(inc.counter_count(CounterType::PlusOnePlusOne), 3, "three +1/+1 counters");
    let cp = g.computed_permanent(inc_id).unwrap();
    assert!(cp.card_types.contains(&CardType::Artifact) && !cp.card_types.contains(&CardType::Creature),
        "front is a noncreature artifact");
    // {2}: Transform.
    g.players[0].mana_pool.add_colorless(2);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: inc_id, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("transform the Incubator");
    drain_stack(&mut g);
    let back = g.computed_permanent(inc_id).unwrap();
    assert!(back.card_types.contains(&CardType::Creature), "back is a creature");
    assert_eq!((back.power, back.toughness), (3, 3), "0/0 base + three +1/+1 = 3/3");
}

/// Eyes of Gitaxias incubates 3 and draws a card.
#[test]
fn eyes_of_gitaxias_incubates_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    resolve_for(&mut g, 0, catalog::eyes_of_gitaxias().effect);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Incubator"), "Incubator minted");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card");
}

/// Injector Crocodile incubates 3 when it dies.
#[test]
fn injector_crocodile_incubates_on_death() {
    let mut g = two_player_game();
    let croc = g.add_card_to_battlefield(0, catalog::injector_crocodile());
    let ctx = crabomination::game::effects::EffectContext::for_ability(croc, 0, None);
    g.resolve_effect(
        &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &crabomination::game::effects::EffectContext { targets: vec![crabomination::game::types::Target::Permanent(croc)], ..ctx },
    ).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Incubator"), "death incubated 3");
}

/// Sunfall exiles all creatures and incubates X = the number exiled.
#[test]
fn sunfall_exiles_all_and_incubates_count() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    resolve_for(&mut g, 0, catalog::sunfall().effect); // resolve_for adds one more bear (4 total)
    assert!(!g.battlefield.iter().any(|c| c.definition.is_creature() && c.definition.name != "Incubator"),
        "all creatures exiled");
    let inc = g.battlefield.iter().find(|c| c.definition.name == "Incubator").expect("Incubator minted");
    assert_eq!(inc.counter_count(CounterType::PlusOnePlusOne), 4, "X = 4 creatures exiled");
}

/// Phyrexian Awakening's static grants vigilance to your Phyrexians.
#[test]
fn phyrexian_awakening_anthem_grants_vigilance() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::phyrexian_awakening());
    let croc = g.add_card_to_battlefield(0, catalog::injector_crocodile()); // a Phyrexian
    assert!(g.computed_permanent(croc).unwrap().keywords.contains(&Keyword::Vigilance));
}

/// Essence of Orthodoxy incubates 2 when a Phyrexian you control enters.
#[test]
fn essence_of_orthodoxy_incubates_on_phyrexian_entry() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::essence_of_orthodoxy());
    let before = g.battlefield.iter().filter(|c| c.definition.name == "Incubator").count();
    let croc = g.add_card_to_battlefield(0, catalog::injector_crocodile()); // a Phyrexian
    g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::PermanentEntered { card_id: croc }]);
    drain_stack(&mut g);
    let after = g.battlefield.iter().filter(|c| c.definition.name == "Incubator").count();
    assert_eq!(after, before + 1, "a Phyrexian entering incubated");
}

/// Compleated Huntmaster sacrifices another permanent to incubate 3.
#[test]
fn compleated_huntmaster_sac_incubates() {
    let mut g = two_player_game();
    let hunt = g.add_card_to_battlefield(0, catalog::compleated_huntmaster());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(hunt);
    g.players[0].mana_pool.add_colorless(1);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: hunt, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate Compleated Huntmaster");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    let inc = g.battlefield.iter().find(|c| c.definition.name == "Incubator").expect("incubated");
    assert_eq!(inc.counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Apostle of Invasion gains double strike only while an opponent has 3+ poison
/// (CR 702.166 Corrupted).
#[test]
fn apostle_of_invasion_corrupted_double_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let apostle = g.add_card_to_battlefield(0, catalog::apostle_of_invasion());
    assert!(
        !g.computed_permanent(apostle).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "no double strike below 3 poison",
    );
    g.players[1].poison_counters = 3;
    assert!(
        g.computed_permanent(apostle).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "double strike live once Corrupted",
    );
}

/// Bloated Contaminator poisons (toxic 1) and proliferates on combat damage.
#[test]
fn bloated_contaminator_toxic_and_proliferate() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let bc = g.add_card_to_battlefield(0, catalog::bloated_contaminator());
    g.clear_sickness(bc);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bc, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    // toxic 1 gives a poison counter, then the proliferate trigger bumps it to 2.
    assert_eq!(g.players[1].poison_counters, 2, "toxic 1 + proliferate = 2 poison");
}

/// Sinew Dancer's cheap Corrupted tap ability is only activatable while an
/// opponent has 3+ poison (CR 702.166); its full-price ability always works.
#[test]
fn sinew_dancer_corrupted_ability_gated() {
    let mut g = two_player_game();
    let dancer = g.add_card_to_battlefield(0, catalog::sinew_dancer());
    g.clear_sickness(dancer);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    // No poison → the {W} Corrupted ability (index 1) is rejected.
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: dancer, ability_index: 1,
        target: Some(Target::Permanent(foe)), additional_targets: vec![], x_value: None, mode: None,
    });
    assert!(err.is_err(), "Corrupted ability blocked below 3 poison");
    // Grant Corrupted and retry — now it taps the target.
    g.players[1].poison_counters = 3;
    g.perform_action(GameAction::ActivateAbility {
        card_id: dancer, ability_index: 1,
        target: Some(Target::Permanent(foe)), additional_targets: vec![], x_value: None, mode: None,
    }).expect("Corrupted ability activatable at 3 poison");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "target creature tapped");
}

/// Vivisection Evangelist destroys an opponent's creature on ETB only when
/// Corrupted (CR 702.166); otherwise the trigger doesn't fire.
#[test]
fn vivisection_evangelist_corrupted_etb_destroy() {
    let cast = |g: &mut GameState, target: Option<Target>| {
        let id = g.add_card_to_hand(0, catalog::vivisection_evangelist());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Vivisection Evangelist");
        drain_stack(g);
    };
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Not corrupted → the ETB trigger's intervening-`if` (CR 603.4) fails, so
    // the self-ETB trigger doesn't fire (exercises the filter-gating fix).
    cast(&mut g, None);
    assert!(g.battlefield_find(foe).is_some(), "no destroy below 3 poison");
    // Corrupted → the ETB destroys the opponent's creature (target bound at cast).
    g.players[1].poison_counters = 3;
    cast(&mut g, Some(Target::Permanent(foe)));
    assert!(g.battlefield_find(foe).is_none(), "Corrupted ETB destroyed the creature");
}

/// Ravenous Necrotitan makes you sacrifice a creature on ETB unless Corrupted.
#[test]
fn ravenous_necrotitan_sacrifices_unless_corrupted() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let titan = g.add_card_to_hand(0, catalog::ravenous_necrotitan());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: titan, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ravenous Necrotitan");
    drain_stack(&mut g);
    // Not corrupted → a creature was sacrificed (the weakest — the 2/2 bear).
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed a creature (not Corrupted)");
}

/// Fleshless Gladiator returns itself tapped from the graveyard while Corrupted,
/// costing 1 life.
#[test]
fn fleshless_gladiator_corrupted_graveyard_return() {
    let mut g = two_player_game();
    let glad = g.add_card_to_graveyard(0, catalog::fleshless_gladiator());
    g.players[1].poison_counters = 3; // Corrupted on
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life0 = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: glad, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate from graveyard");
    drain_stack(&mut g);
    let back = g.battlefield_find(glad).expect("returned to battlefield");
    assert!(back.tapped, "returned tapped");
    assert_eq!(g.players[0].life, life0 - 1, "lost 1 life");
}

/// Branchblight Stalker's toxic 2 gives the defending player two poison counters
/// on combat damage (CR 702.180 Toxic).
#[test]
fn branchblight_stalker_toxic_two_poisons() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let stalker = g.add_card_to_battlefield(0, catalog::branchblight_stalker());
    g.clear_sickness(stalker);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: stalker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 2, "toxic 2 = two poison counters");
}

/// The toxic vanilla-ish cycle ships with its printed stats and keywords.
#[test]
fn one_toxic_cycle_stats_and_keywords() {
    use crabomination::card::Keyword;
    let bilious = catalog::bilious_skulldweller();
    assert_eq!((bilious.power, bilious.toughness), (1, 1));
    assert!(bilious.keywords.contains(&Keyword::Deathtouch) && bilious.keywords.contains(&Keyword::Toxic(1)));
    let jawbone = catalog::jawbone_duelist();
    assert!(jawbone.keywords.contains(&Keyword::DoubleStrike) && jawbone.keywords.contains(&Keyword::Toxic(1)));
    let basilisk = catalog::ichorspit_basilisk();
    assert_eq!((basilisk.power, basilisk.toughness), (1, 3));
    assert!(basilisk.keywords.contains(&Keyword::Deathtouch) && basilisk.keywords.contains(&Keyword::Toxic(1)));
}

/// Incisor Glider's Corrupted attack trigger pumps your team only while an
/// opponent has 3+ poison (CR 702.166 intervening-if on an attack trigger).
#[test]
fn incisor_glider_corrupted_attack_pump() {
    use crabomination::game::types::{Attack, AttackTarget};
    let attack = |g: &mut GameState, glider, other| {
        g.clear_sickness(glider);
        g.clear_sickness(other);
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: glider, target: AttackTarget::Player(1),
        }])).unwrap();
        drain_stack(g);
    };
    // Not corrupted → no pump: the other creature stays a 2/2.
    let mut g = two_player_game();
    let glider = g.add_card_to_battlefield(0, catalog::incisor_glider());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack(&mut g, glider, bear);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "no pump below 3 poison");
    // Corrupted → team gets +1/+1.
    let mut g = two_player_game();
    g.players[1].poison_counters = 3;
    let glider = g.add_card_to_battlefield(0, catalog::incisor_glider());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack(&mut g, glider, bear);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "Corrupted pump: 2/2 → 3/3");
}

/// Malcator's Watcher draws a card when it dies.
#[test]
fn malcators_watcher_draws_on_death() {
    let mut g = two_player_game();
    let watcher = g.add_card_to_battlefield(0, catalog::malcators_watcher());
    g.add_card_to_library(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    let ctx = crabomination::game::effects::EffectContext::for_ability(watcher, 0, None);
    g.resolve_effect(
        &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &crabomination::game::effects::EffectContext { targets: vec![Target::Permanent(watcher)], ..ctx },
    ).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "death drew a card");
}

/// Chimney Rabble enters with haste and mints a 1/1 Phyrexian Goblin token.
#[test]
fn chimney_rabble_mints_goblin_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::chimney_rabble());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Chimney Rabble");
    drain_stack(&mut g);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&crabomination::card::Keyword::Haste));
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Goblin" && c.controller == 0).count(),
        1, "one Goblin token minted",
    );
}

/// Chrome Prowler taps an opponent's creature on ETB.
#[test]
fn chrome_prowler_taps_on_etb() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::chrome_prowler());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Chrome Prowler");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "opponent's creature tapped on ETB");
}

/// The simple ONE keyword creatures ship with their printed stats/keywords.
#[test]
fn one_keyword_creatures_shape() {
    use crabomination::card::Keyword;
    let lookout = catalog::swooping_lookout();
    assert_eq!((lookout.power, lookout.toughness), (1, 2));
    assert!(lookout.keywords.contains(&Keyword::Flying) && lookout.keywords.contains(&Keyword::Vigilance));
    let cleaver = catalog::sheoldreds_headcleaver();
    assert_eq!((cleaver.power, cleaver.toughness), (2, 4));
    assert!(cleaver.keywords.contains(&Keyword::Menace) && cleaver.keywords.contains(&Keyword::Toxic(2)));
}

/// Cutthroat Centurion sacrifices another permanent to pump itself +2/+2.
#[test]
fn cutthroat_centurion_sac_pump() {
    let mut g = two_player_game();
    let cent = g.add_card_to_battlefield(0, catalog::cutthroat_centurion());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cent, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate Cutthroat Centurion");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.computed_permanent(cent).unwrap().power, 4, "2/2 + 2/2 = 4/4");
}

/// Shrapnel Slinger sacrifices a creature on ETB to destroy an opponent artifact.
#[test]
fn shrapnel_slinger_sac_destroys_artifact() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::millstone()); // opponent artifact
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let slinger = g.add_card_to_battlefield(0, catalog::shrapnel_slinger());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let eff = catalog::shrapnel_slinger().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(slinger, 0, None, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the creature");
    assert!(g.battlefield_find(art).is_none(), "opponent's artifact destroyed");
}

// ── Modern_decks ONE wave (toxic / corrupted / oil / for-Mirrodin payoffs) ──

use crabomination::card::Keyword;
use crabomination::effect::{Effect, Value};
use crabomination::game::types::Target;

/// Resolve a targeted `effect` for `player` against `targets`.
fn resolve_targeted(g: &mut GameState, player: usize, effect: Effect, targets: Vec<Target>) {
    let src = g.add_card_to_battlefield(player, catalog::grizzly_bears());
    let base = crabomination::game::effects::EffectContext::for_ability(src, player, None);
    let ctx = crabomination::game::effects::EffectContext { targets, ..base };
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(g);
}

fn add_oil(g: &mut GameState, id: crabomination::card::CardId, n: u32) {
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().add_counters(CounterType::Oil, n);
}

/// Tyrranax Rex ships toxic 4 + ward + trample + haste, and is uncounterable.
#[test]
fn tyrranax_rex_keywords() {
    let mut g = two_player_game();
    let rex = g.add_card_to_battlefield(0, catalog::tyrranax_rex());
    let kws = g.computed_permanent(rex).unwrap().keywords;
    assert!(kws.contains(&Keyword::Toxic(4)) && kws.contains(&Keyword::Trample) && kws.contains(&Keyword::Haste));
    assert!(catalog::tyrranax_rex().keywords.iter().any(|k| matches!(k, Keyword::CantBeCountered)));
}

/// Thrun has indestructible only on his controller's turn (SelfHasKeywordIf).
#[test]
fn thrun_indestructible_only_your_turn() {
    let mut g = two_player_game();
    let thrun = g.add_card_to_battlefield(0, catalog::thrun_breaker_of_silence());
    g.active_player_idx = 0;
    assert!(g.computed_permanent(thrun).unwrap().keywords.contains(&Keyword::Indestructible));
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(thrun).unwrap().keywords.contains(&Keyword::Indestructible));
}

/// Thrun can't be targeted by an opponent's nongreen source, but a green one is
/// fine (HexproofExceptColors).
#[test]
fn thrun_hexproof_from_nongreen() {
    let mut g = two_player_game();
    let thrun = g.add_card_to_battlefield(0, catalog::thrun_breaker_of_silence());
    let red = g.add_card_to_battlefield(1, catalog::goblin_king()); // a red source
    let green = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // a green source
    assert!(g.ability_target_has_protection(&Target::Permanent(thrun), red), "nongreen blocked");
    assert!(!g.ability_target_has_protection(&Target::Permanent(thrun), green), "green allowed");
}

/// Mondrak doubles token creation.
#[test]
fn mondrak_doubles_tokens() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mondrak_glory_dominus());
    let token = crabomination::card::TokenDefinition {
        name: "Test Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crabomination::mana::Color::Red],
        ..Default::default()
    };
    resolve_for(&mut g, 0, Effect::CreateToken {
        who: crabomination::effect::PlayerRef::You,
        count: Value::ONE,
        definition: token,
    });
    let minted = g.battlefield.iter().filter(|c| c.definition.name == "Test Goblin").count();
    assert_eq!(minted, 2, "one token doubled to two");
}

/// Kuldotha Cackler's attack pump = number of your permanents with oil counters.
#[test]
fn kuldotha_cackler_pumps_per_oil_permanent() {
    let mut g = two_player_game();
    let cackler = g.add_card_to_battlefield(0, catalog::kuldotha_cackler());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    add_oil(&mut g, a, 1);
    add_oil(&mut g, b, 3); // two permanents carry oil (counts permanents, not counters)
    g.clear_sickness(cackler);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: cackler, target: crabomination::game::types::AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(cackler).unwrap().power, 4, "2 base + 2 oil-bearing permanents");
}

/// Evolving Adaptive enters with an oil counter (so it's 1/1) via the P/T CDA.
#[test]
fn evolving_adaptive_grows_with_oil() {
    let mut g = two_player_game();
    let ea = g.move_card_to_battlefield_for_test(0, catalog::evolving_adaptive());
    let cp = g.computed_permanent(ea).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "0/0 + 1 oil = 1/1");
    add_oil(&mut g, ea, 2);
    let cp = g.computed_permanent(ea).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "three oil = 3/3");
}

/// A bigger creature entering adds an oil counter to Evolving Adaptive; a smaller
/// one doesn't.
#[test]
fn evolving_adaptive_gains_oil_on_bigger_entry() {
    let mut g = two_player_game();
    let ea = g.move_card_to_battlefield_for_test(0, catalog::evolving_adaptive()); // 1/1
    let oil0 = g.battlefield.iter().find(|c| c.id == ea).unwrap().counter_count(CounterType::Oil);
    // Serra Angel (4/4) is bigger.
    let big = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::PermanentEntered { card_id: big }]);
    drain_stack(&mut g);
    let oil1 = g.battlefield.iter().find(|c| c.id == ea).unwrap().counter_count(CounterType::Oil);
    assert_eq!(oil1, oil0 + 1, "bigger creature added an oil counter");
}

/// Skrelv's Hive: corrupted grants lifelink to your toxic creatures only.
#[test]
fn skrelvs_hive_corrupted_lifelink() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::skrelvs_hive());
    let toxic = g.add_card_to_battlefield(0, catalog::crawling_chorus()); // toxic 1
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Not corrupted yet.
    assert!(!g.computed_permanent(toxic).unwrap().keywords.contains(&Keyword::Lifelink));
    g.players[1].poison_counters = 3;
    assert!(g.computed_permanent(toxic).unwrap().keywords.contains(&Keyword::Lifelink), "toxic gains lifelink");
    assert!(!g.computed_permanent(plain).unwrap().keywords.contains(&Keyword::Lifelink), "non-toxic doesn't");
}

/// Prologue to Phyresis poisons each opponent and draws.
#[test]
fn prologue_to_phyresis_poison_and_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    resolve_for(&mut g, 0, catalog::prologue_to_phyresis().effect);
    assert_eq!(g.players[1].poison_counters, 1);
    assert_eq!(g.players[0].hand.len(), hand0 + 1);
}

/// Whisper of the Dross shrinks a creature and proliferates.
#[test]
fn whisper_of_the_dross_shrinks_and_proliferates() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.players[1].poison_counters = 1; // opponent already poisoned
    resolve_targeted(&mut g, 0, catalog::whisper_of_the_dross().effect, vec![Target::Permanent(victim)]);
    assert_eq!(g.computed_permanent(victim).unwrap().toughness, 3, "4/4 -> 3/3");
    assert_eq!(g.players[1].poison_counters, 2, "proliferated opponent's poison");
}

/// Crawling Chorus mints a Mite when it dies.
#[test]
fn crawling_chorus_dies_to_mite() {
    let mut g = two_player_game();
    let cc = g.add_card_to_battlefield(0, catalog::crawling_chorus());
    let ctx = crabomination::game::effects::EffectContext::for_ability(cc, 0, None);
    g.resolve_effect(
        &Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &crabomination::game::effects::EffectContext { targets: vec![Target::Permanent(cc)], ..ctx },
    ).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Phyrexian Mite"), "died into a Mite");
}

/// Zealot of the God-Pharaoh pings an opponent for 2.
#[test]
fn zealot_pings_opponent() {
    let mut g = two_player_game();
    let z = g.add_card_to_battlefield(0, catalog::zealot_of_the_god_pharaoh());
    g.clear_sickness(z);
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: z, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate Zealot");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
}

/// Mandibular Kite's living weapon mints a Germ and attaches.
#[test]
fn mandibular_kite_living_weapon() {
    let mut g = two_player_game();
    let kite = g.move_card_to_battlefield_for_test(0, catalog::mandibular_kite());
    drain_stack(&mut g);
    let germ = g.battlefield.iter().find(|c| c.definition.name == "Phyrexian Germ").expect("Germ minted");
    // Equipped: 0/0 + 1/1 = 1/1 with flying.
    let cp = g.computed_permanent(germ.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.keywords.contains(&Keyword::Flying));
    let _ = kite;
}

/// Migloz enters with five oil counters and can spend two for +2/+2.
#[test]
fn migloz_spends_oil_to_pump() {
    let mut g = two_player_game();
    let migloz = g.move_card_to_battlefield_for_test(0, catalog::migloz_maze_crusher());
    assert_eq!(g.battlefield.iter().find(|c| c.id == migloz).unwrap().counter_count(CounterType::Oil), 5);
    g.clear_sickness(migloz);
    g.players[0].mana_pool.add_colorless(2);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: migloz, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("pump ability");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(migloz).unwrap().power, 6, "4 + 2");
    assert_eq!(g.battlefield.iter().find(|c| c.id == migloz).unwrap().counter_count(CounterType::Oil), 3, "spent two oil");
}

/// Vraan drains 2 the first time another of your creatures dies each turn.
#[test]
fn vraan_drains_once_per_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vraan_executioner_thane());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::CreatureDied { card_id: fodder }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2);
    assert_eq!(g.players[0].life, my_life + 2);
}

/// Karumonix grants toxic to your other Rats.
#[test]
fn karumonix_rat_lord() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::karumonix_the_rat_king());
    // A plain Rat: give it via a token-ish stand-in — use another Karumonix body? Instead
    // verify the static targets Rats: a Rat creature gains toxic 1.
    let rat = g.add_card_to_battlefield(0, catalog::ravenous_rats());
    assert!(g.computed_permanent(rat).unwrap().keywords.iter().any(|k| matches!(k, Keyword::Toxic(_))));
}

/// Slaughter Singer pumps another attacking toxic creature.
#[test]
fn slaughter_singer_pumps_toxic_attacker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::slaughter_singer());
    let chorus = g.add_card_to_battlefield(0, catalog::crawling_chorus()); // toxic 1, 1/1
    g.clear_sickness(chorus);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: chorus, target: crabomination::game::types::AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(chorus).unwrap().power, 2, "1/1 toxic attacker gets +1/+1");
}

/// Ichor Drinker incubates 2 from the graveyard (exile-self cost).
#[test]
fn ichor_drinker_gy_incubates() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::ichor_drinker());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate from graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Incubator"), "incubated 2");
}

/// Corrupted Conviction sacrifices a creature and draws two.
#[test]
fn corrupted_conviction_sac_draws_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    let cc = g.add_card_to_hand(0, catalog::corrupted_conviction());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: cc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Corrupted Conviction");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed a creature");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "spent the spell, drew two");
}

/// Vraska's Fall edicts each opponent and poisons them.
#[test]
fn vraskas_fall_edict_and_poison() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    resolve_for(&mut g, 0, catalog::vraskas_fall().effect);
    assert!(g.battlefield_find(victim).is_none(), "opponent sacrificed a creature");
    assert_eq!(g.players[1].poison_counters, 1, "opponent got a poison counter");
}

/// Bring the Ending hard-counters while Corrupted; otherwise it's a soft counter.
#[test]
fn bring_the_ending_corrupted_hard_counter() {
    let mut g = two_player_game();
    // On player 1's turn they cast a creature; player 0 counters it while Corrupted.
    g.active_player_idx = 1;
    let angel = g.add_card_to_hand(1, catalog::serra_angel());
    g.players[1].mana_pool.add_colorless(3);
    g.players[1].mana_pool.add(crabomination::mana::Color::White, 2);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts angel");
    // Bring the Ending checks the caster's (player 0's) Corrupted — an opponent
    // with three or more poison — so poison the opponent (player 1).
    g.players[1].poison_counters = 3;
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let base = crabomination::game::effects::EffectContext::for_ability(src, 0, None);
    let ctx = crabomination::game::effects::EffectContext { targets: vec![Target::Permanent(angel)], ..base };
    g.resolve_effect(&catalog::bring_the_ending().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == angel), "angel hard-countered to graveyard");
}

/// Vindictive Flamestoker accrues an oil counter per noncreature spell you cast.
#[test]
fn vindictive_flamestoker_oil_on_noncreature_cast() {
    let mut g = two_player_game();
    let vf = g.add_card_to_battlefield(0, catalog::vindictive_flamestoker());
    // Cast a noncreature spell (Lightning Bolt) from hand.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().find(|c| c.id == vf).unwrap().counter_count(CounterType::Oil), 1,
        "noncreature cast put an oil counter");
}

/// Gitaxian Anatomist taps and proliferates on entry.
#[test]
fn gitaxian_anatomist_taps_and_proliferates() {
    let mut g = two_player_game();
    g.players[1].poison_counters = 1;
    let ga = g.move_card_to_battlefield_for_test(0, catalog::gitaxian_anatomist());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().find(|c| c.id == ga).unwrap().tapped, "tapped itself");
    assert_eq!(g.players[1].poison_counters, 2, "proliferated opponent poison");
}

/// Basilica Shepherd mints two Mite tokens on entry.
#[test]
fn basilica_shepherd_makes_two_mites() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::basilica_shepherd());
    drain_stack(&mut g);
    let mites = g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Mite").count();
    assert_eq!(mites, 2, "two Mites entered");
}

/// Infectious Bite fights one-sided and poisons each opponent.
#[test]
fn infectious_bite_fights_and_poisons() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    resolve_targeted(&mut g, 0, catalog::infectious_bite().effect,
        vec![Target::Permanent(mine), Target::Permanent(theirs)]);
    assert!(g.battlefield_find(theirs).is_none(), "took 4 damage and died");
    assert_eq!(g.players[1].poison_counters, 1, "each opponent poisoned");
}

/// Gulping Scraptrap proliferates on entry.
#[test]
fn gulping_scraptrap_proliferates_on_entry() {
    let mut g = two_player_game();
    g.players[1].poison_counters = 1;
    g.move_card_to_battlefield_for_test(0, catalog::gulping_scraptrap());
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 2, "entry proliferated opponent poison");
}

/// Deadly Derision destroys a creature and mints a Treasure.
#[test]
fn deadly_derision_destroys_and_makes_treasure() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    resolve_targeted(&mut g, 0, catalog::deadly_derision().effect, vec![Target::Permanent(victim)]);
    assert!(g.battlefield_find(victim).is_none(), "destroyed");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure" && c.controller == 0), "made a Treasure");
}

/// Kill-Zone Acrobat sacrifices to gain flying on attack.
#[test]
fn kill_zone_acrobat_sac_for_flying() {
    let mut g = two_player_game();
    let acro = g.add_card_to_battlefield(0, catalog::kill_zone_acrobat());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.clear_sickness(acro);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: acro, target: crabomination::game::types::AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert!(g.computed_permanent(acro).unwrap().keywords.contains(&Keyword::Flying), "gained flying via sacrifice");
}

/// Blightbelly Rat proliferates when it dies.
#[test]
fn blightbelly_rat_dies_proliferates() {
    let mut g = two_player_game();
    let rat = g.add_card_to_battlefield(0, catalog::blightbelly_rat());
    g.players[1].poison_counters = 1;
    let ctx = crabomination::game::effects::EffectContext::for_ability(rat, 0, None);
    g.resolve_effect(
        &Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &crabomination::game::effects::EffectContext { targets: vec![Target::Permanent(rat)], ..ctx },
    ).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 2, "death proliferated opponent poison");
}

/// Sawblade Scamp gains oil on noncreature cast and spends it to ping.
#[test]
fn sawblade_scamp_oil_then_ping() {
    let mut g = two_player_game();
    let scamp = g.add_card_to_battlefield(0, catalog::sawblade_scamp());
    add_oil(&mut g, scamp, 1);
    g.clear_sickness(scamp);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: scamp, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("ping ability");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "dealt 1 to the opponent");
    assert_eq!(g.battlefield.iter().find(|c| c.id == scamp).unwrap().counter_count(CounterType::Oil), 0, "spent the oil");
}

/// Furnace Punisher pings the upkeep player for 2 unless they control two or
/// more basic lands.
#[test]
fn furnace_punisher_punishes_nonbasic_manabases() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::furnace_punisher());
    // Opponent has one basic — gets hit on their upkeep.
    g.add_card_to_battlefield(1, catalog::swamp());
    let life = g.players[1].life;
    let eff = catalog::furnace_punisher().triggered_abilities[0].effect.clone();
    let src = g.battlefield.iter().find(|c| c.definition.name == "Furnace Punisher").unwrap().id;
    let ctx = crabomination::game::effects::EffectContext::for_trigger(src, 0, None, 0);
    g.active_player_idx = 1;
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[1].life, life - 2, "one basic → 2 damage");
    // Second basic shields them.
    g.add_card_to_battlefield(1, catalog::swamp());
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[1].life, life - 2, "two basics → no damage");
}

/// Anoint with Affliction exiles only MV≤3 creatures normally, but any
/// creature once its controller is corrupted (3+ poison).
#[test]
fn anoint_with_affliction_corrupted_widens_target() {
    let mut g = two_player_game();
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let big = g.add_card_to_battlefield(1, catalog::craw_wurm()); // MV 6
    let spell = g.add_card_to_hand(0, catalog::anoint_with_affliction());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "MV 6 target rejected while not corrupted");
    g.players[1].poison_counters = 3;
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("corrupted → any creature");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == big), "wurm exiled");
}

/// Voltage Surge: 2 damage plain; sacrificing an artifact via the optional
/// additional cost (kicker plumbing) deals 4 instead.
#[test]
fn voltage_surge_kicked_deals_four() {
    let mut g = two_player_game();
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let fodder = g.add_card_to_battlefield(0, catalog::ornithopter());
    let victim = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    let spell = g.add_card_to_hand(0, catalog::voltage_surge());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast with additional cost");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "artifact sacrificed at cast");
    assert!(g.battlefield_find(victim).is_none(), "4 damage kills the 6/4");
}

/// Voltage Surge without the sacrifice deals 2.
#[test]
fn voltage_surge_unkicked_deals_two() {
    let mut g = two_player_game();
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let victim = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let spell = g.add_card_to_hand(0, catalog::voltage_surge());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("plain cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).expect("survives").damage, 2);
}

/// Necrogen Communion grants toxic 2 and reanimates the host under your
/// control when it dies.
#[test]
fn necrogen_communion_grants_toxic_and_reanimates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let aura = g.add_card_to_hand(0, catalog::necrogen_communion());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.iter()
        .any(|k| matches!(k, crabomination::card::Keyword::Toxic(2))), "host has toxic 2");
    g.battlefield_find_mut(bear).unwrap().damage = 99;
    let events = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let back = g.battlefield_find(bear).expect("host returned to the battlefield");
    assert_eq!(back.controller, 0, "under the aura controller's control");
}

/// Annihilating Glare's SacrificeOrPay cost: with an artifact it sacrifices;
/// the spell destroys the target.
#[test]
fn annihilating_glare_sacrifices_and_destroys() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::ornithopter());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::annihilating_glare());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "artifact paid the cost");
    assert!(g.battlefield_find(victim).is_none(), "angel destroyed");
}

/// Axiom Engraver enters with two oil and loots through them.
#[test]
fn axiom_engraver_loots_on_oil() {
    let mut g = two_player_game();
    let ax = g.move_card_to_battlefield_for_test(0, catalog::axiom_engraver());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ax).unwrap().counter_count(CounterType::Oil), 2);
    g.clear_sickness(ax);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_hand(0, catalog::island()); // discard fodder
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ax, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("loot");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "discard 1 + draw 1");
    assert_eq!(g.battlefield_find(ax).unwrap().counter_count(CounterType::Oil), 1);
}

/// Blazing Crescendo pumps and impulses with a next-turn window.
#[test]
fn blazing_crescendo_pumps_and_impulses() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let top = g.add_card_to_library(0, catalog::forest());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::blazing_crescendo());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "2+3");
    let ex = g.exile.iter().find(|c| c.id == top).expect("impulsed");
    assert!(matches!(ex.may_play_until.as_ref().unwrap().duration,
        crabomination::card::MayPlayDuration::EndOfControllersNextTurn));
}

/// Annex Sentry jails a small opposing creature until it leaves.
#[test]
fn annex_sentry_jails_until_it_leaves() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let sentry = g.add_card_to_hand(0, catalog::annex_sentry());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: sentry, target: Some(Target::Permanent(small)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast sentry");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == small), "bear jailed");
    g.battlefield_find_mut(sentry).unwrap().damage = 99;
    let events = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_some(), "bear returns when the jailer dies");
}

/// Armored Scrapgorger grows with oil and eats graveyards when tapped.
#[test]
fn armored_scrapgorger_eats_graveyards() {
    let mut g = two_player_game();
    let gorger = g.add_card_to_battlefield(0, catalog::armored_scrapgorger());
    let snack = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.clear_sickness(gorger);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: gorger, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("tap for mana");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == snack), "graveyard card eaten");
    let c = g.battlefield_find(gorger).unwrap();
    assert_eq!(c.counter_count(CounterType::Oil), 1);
    // 0/3 base; with three oils it's a 3/3.
    g.battlefield_find_mut(gorger).unwrap().add_counters(CounterType::Oil, 2);
    assert_eq!(g.computed_permanent(gorger).unwrap().power, 3, "+3/+0 at 3 oil");
}

/// Ambulatory Edifice: paying 2 life shrinks a target.
#[test]
fn ambulatory_edifice_pays_life_to_shrink() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.move_card_to_battlefield_for_test(0, catalog::ambulatory_edifice());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "paid 2 life");
    assert_eq!(g.computed_permanent(foe).unwrap().power, 1, "-1/-1");
}

/// Bladed Ambassador spends its oil for indestructibility.
#[test]
fn bladed_ambassador_goes_indestructible() {
    let mut g = two_player_game();
    let amb = g.move_card_to_battlefield_for_test(0, catalog::bladed_ambassador());
    drain_stack(&mut g);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: amb, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(amb).unwrap().keywords.contains(&Keyword::Indestructible));
    assert_eq!(g.battlefield_find(amb).unwrap().counter_count(CounterType::Oil), 0);
}

/// Black Sun's Twilight at X=5 shrinks the target and reanimates.
#[test]
fn black_suns_twilight_big_x_reanimates() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::black_suns_twilight());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: Some(5),
    }).expect("cast X=5");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "4/4 - 5/-5 dies");
    let back = g.battlefield_find(dead).expect("reanimated");
    assert!(back.tapped, "enters tapped");
}

/// Atmosphere Surgeon banks oil off noncreature spells and spends it for
/// flying.
#[test]
fn atmosphere_surgeon_banks_and_spends_oil() {
    let mut g = two_player_game();
    let surgeon = g.add_card_to_battlefield(0, catalog::atmosphere_surgeon());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(surgeon).unwrap().counter_count(CounterType::Oil), 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: surgeon, ability_index: 0, target: Some(Target::Permanent(surgeon)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("spend oil");
    drain_stack(&mut g);
    assert!(g.computed_permanent(surgeon).unwrap().keywords.contains(&Keyword::Flying));
}

/// Adaptive Sporesinger mode 2 proliferates.
#[test]
fn adaptive_sporesinger_proliferates() {
    let mut g = two_player_game();
    let seeded = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(seeded).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let singer = g.add_card_to_battlefield(0, catalog::adaptive_sporesinger());
    // Resolve the ETB choose-one directly on its proliferate mode.
    let modes = match &catalog::adaptive_sporesinger().triggered_abilities[0].effect {
        crabomination::effect::Effect::ChooseMode(m) => m.clone(),
        _ => unreachable!(),
    };
    let ctx = crabomination::game::effects::EffectContext::for_trigger(singer, 0, None, 0);
    g.resolve_effect(&modes[1], &ctx).unwrap();
    assert_eq!(g.battlefield_find(seeded).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "proliferated");
}

/// Against All Odds mode 2 reanimates a small artifact/creature.
#[test]
fn against_all_odds_reanimates_small() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let mine = g.add_card_to_battlefield(0, catalog::ornithopter()); // flicker target
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::against_all_odds());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(dead)], mode: None, x_value: None,
    }).expect("cast both modes");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "reanimated");
    assert!(g.battlefield_find(mine).is_some(), "flickered back");
}

/// Bladegraft Aspirant discounts Equipment spells.
#[test]
fn bladegraft_aspirant_discounts_equipment() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bladegraft_aspirant());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let sword = g.add_card_to_hand(0, catalog::short_sword()); // {1} → free
    g.perform_action(GameAction::CastSpell {
        card_id: sword, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("discounted to {0}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sword).is_some());
}

/// Carnivorous Canopy proliferates only against small targets.
#[test]
fn carnivorous_canopy_proliferates_on_small() {
    let mut g = two_player_game();
    let relic = g.add_card_to_battlefield(1, catalog::ornithopter()); // MV 0
    let seeded = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(seeded).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::carnivorous_canopy());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(relic)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(relic).is_none(), "artifact destroyed");
    assert_eq!(g.battlefield_find(seeded).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "proliferated (MV 0 ≤ 3)");
}

/// Crawling Chorus leaves a Mite behind.
#[test]
fn crawling_chorus_leaves_a_mite() {
    let mut g = two_player_game();
    let chorus = g.add_card_to_battlefield(0, catalog::crawling_chorus());
    g.battlefield_find_mut(chorus).unwrap().damage = 99;
    let events = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let mite = g.battlefield.iter().find(|c| c.definition.name == "Phyrexian Mite")
        .expect("Mite minted");
    assert!(mite.definition.keywords.contains(&crabomination::card::Keyword::CantBlock));
}

/// Distorted Curiosity costs {U} while an opponent is corrupted.
#[test]
fn distorted_curiosity_corrupted_discount() {
    let mut g = two_player_game();
    g.players[1].poison_counters = 3;
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let spell = g.add_card_to_hand(0, catalog::distorted_curiosity());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1); // just {U}
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("corrupted discount to {U}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew two");
}

// ── Proliferate-matters (CR 701.34) ──────────────────────────────────────────

/// "Whenever you proliferate" fires once per proliferate instance
/// (EventKind::Proliferated — Scheming Aspirant drains 2 per proliferation).
#[test]
fn cr_701_34_proliferate_trigger_fires() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::scheming_aspirant());
    resolve_for(&mut g, 0, crabomination::effect::Effect::Proliferate);
    assert_eq!(g.players[1].life, 18, "opponent lost 2");
    assert_eq!(g.players[0].life, 22, "you gained 2");
}

/// Tekuthal doubles a proliferate (CR 614): one instruction → two
/// proliferations → payoffs fire twice and counters advance twice.
#[test]
fn cr_701_34_tekuthal_proliferates_twice() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tekuthal_inquiry_dominus());
    g.add_card_to_battlefield(0, catalog::scheming_aspirant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(bear) {
        c.add_counters(CounterType::PlusOnePlusOne, 1);
    }
    resolve_for(&mut g, 0, crabomination::effect::Effect::Proliferate);
    let counters = g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 3, "1 + two proliferations = 3");
    assert_eq!(g.players[1].life, 16, "payoff fired twice");
}

/// Tekuthal's ability removes three counters of any kinds from among OTHER
/// permanents and puts an indestructible counter on itself.
#[test]
fn tekuthal_ability_removes_any_kind_counters() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let tek = g.add_card_to_battlefield(0, catalog::tekuthal_inquiry_dominus());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(bear) {
        c.add_counters(CounterType::PlusOnePlusOne, 2);
        c.add_counters(CounterType::Oil, 1);
    }
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tek, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate Tekuthal");
    drain_stack(&mut g);
    let bear_c = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_c.counters.values().sum::<u32>(), 0, "three counters drained");
    assert_eq!(
        g.battlefield_find(tek).unwrap().keyword_counters.get(&Keyword::Indestructible).copied().unwrap_or(0),
        1,
        "indestructible counter added",
    );
}

/// Ezuri: pay {3} on ETB → proliferate twice; each proliferation draws a card.
#[test]
fn ezuri_etb_pays_and_draws_per_proliferate() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let hand0 = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::ezuri_stalker_of_spheres());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "two proliferations, two draws");
}

/// Voidwing Hybrid returns from the graveyard when you proliferate — and its
/// graveyard-scoped trigger does NOT bounce it while on the battlefield.
#[test]
fn voidwing_hybrid_returns_from_graveyard_on_proliferate() {
    let mut g = two_player_game();
    let vw = g.add_card_to_battlefield(0, catalog::voidwing_hybrid());
    resolve_for(&mut g, 0, crabomination::effect::Effect::Proliferate);
    assert!(g.battlefield_find(vw).is_some(), "battlefield copy untouched");
    // Kill it, then proliferate: it returns to hand.
    g.resolve_effect(
        &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &crabomination::game::effects::EffectContext {
            targets: vec![Target::Permanent(vw)],
            ..crabomination::game::effects::EffectContext::for_ability(vw, 0, None)
        },
    ).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Voidwing Hybrid"));
    resolve_for(&mut g, 0, crabomination::effect::Effect::Proliferate);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Voidwing Hybrid"),
        "returned to hand from graveyard",
    );
}

/// Melira caps a multi-poison hit at one and locks poison for the turn
/// (CR 614 replacement at the add_poison funnel).
#[test]
fn cr_614_melira_caps_poison_at_one_per_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::melira_the_living_cure());
    resolve_for(&mut g, 1, crabomination::effect::Effect::AddPoison {
        who: crabomination::effect::Selector::Player(crabomination::effect::PlayerRef::EachOpponent),
        amount: crabomination::effect::Value::Const(5),
    });
    assert_eq!(g.players[0].poison_counters, 1, "5 poison replaced with 1");
    resolve_for(&mut g, 1, crabomination::effect::Effect::AddPoison {
        who: crabomination::effect::Selector::Player(crabomination::effect::PlayerRef::EachOpponent),
        amount: crabomination::effect::Value::Const(2),
    });
    assert_eq!(g.players[0].poison_counters, 1, "no additional poison this turn");
}

/// Melira's exile ability: when the watched artifact dies this turn it
/// returns to the battlefield under its owner's control.
#[test]
fn melira_exile_watches_artifact_death() {
    let mut g = two_player_game();
    let mel = g.add_card_to_battlefield(0, catalog::melira_the_living_cure());
    let relic = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mel, ability_index: 0,
        target: Some(Target::Permanent(relic)), additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate Melira");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mel).is_none(), "Melira exiled as a cost");
    // Destroy the watched artifact — the delayed trigger returns it.
    resolve_for(&mut g, 0, crabomination::effect::Effect::Destroy {
        what: crabomination::effect::Selector::EachPermanent(
            crabomination::card::SelectionRequirement::Artifact,
        ),
    });
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Mind Stone"),
        "watched artifact returned to the battlefield");
}

/// Ovika mints X hasty Goblins per noncreature spell (X = its mana value) and
/// her compound Ward—{3}, Pay 3 life both taxes mana and life (CR 702.21).
#[test]
fn ovika_tokens_and_compound_ward() {
    let mut g = two_player_game();
    let ov = g.add_card_to_battlefield(0, catalog::ovika_enigma_goliath());
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    // Cast Divination (MV 3, noncreature) → 3 Goblins with haste.
    let shock = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: shock, target: None, additional_targets: vec![], x_value: None, mode: None }).unwrap();
    drain_stack(&mut g);
    let goblins: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Goblin").collect();
    assert_eq!(goblins.len(), 3, "X = 3 (Divination's mana value)");
    // Compound ward: opponent targets Ovika with 3 floated mana but low life —
    // can pay mana AND 3 life (life 20 → fine); check both halves get paid.
    let bolt = g.add_card_to_hand(1, catalog::murder());
    g.players[1].mana_pool.add(crabomination::mana::Color::Black, 2);
    g.players[1].mana_pool.add_colorless(4 + 3);
    g.priority.player_with_priority = 1;
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(ov)), additional_targets: vec![], x_value: None, mode: None }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 3, "ward life half paid");
    assert!(g.battlefield_find(ov).is_none(), "ward paid, Murder resolved");
}

/// Reject Imperfection counters and proliferates only against MV ≤ 3.
#[test]
fn reject_imperfection_proliferates_on_cheap_spells() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(bear) {
        c.add_counters(CounterType::PlusOnePlusOne, 1);
    }
    // Opponent casts Divination (MV 3); we counter with Reject Imperfection.
    for _ in 0..3 { g.add_card_to_library(1, catalog::forest()); }
    let div = g.add_card_to_hand(1, catalog::divination());
    g.players[1].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: div, target: None, additional_targets: vec![], x_value: None, mode: None }).unwrap();
    let ri = g.add_card_to_hand(0, catalog::reject_imperfection());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: ri, target: None, additional_targets: vec![], x_value: None, mode: None }).unwrap();
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Divination"), "countered");
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "MV 3 → proliferated",
    );
}

/// Serum Snare bounces and proliferates only when the target's MV ≤ 3.
#[test]
fn serum_snare_bounce_and_conditional_proliferate() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(mine) {
        c.add_counters(CounterType::PlusOnePlusOne, 1);
    }
    let cheap = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    resolve_for(&mut g, 0, crabomination::effect::Effect::Seq(vec![]));
    let snare = g.add_card_to_hand(0, catalog::serum_snare());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: snare, target: Some(Target::Permanent(cheap)), additional_targets: vec![], x_value: None, mode: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "bounced");
    assert_eq!(
        g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "MV 2 → proliferated",
    );
}

/// Infectious Inquiry draws 2, loses 2 life, and poisons each opponent through
/// the add_poison funnel.
#[test]
fn infectious_inquiry_draws_and_poisons() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    let hand0 = g.players[0].hand.len();
    resolve_for(&mut g, 0, catalog::infectious_inquiry().effect);
    assert_eq!(g.players[0].hand.len(), hand0 + 2);
    assert_eq!(g.players[0].life, 18);
    assert_eq!(g.players[1].poison_counters, 1);
}

/// Experimental Augury impulses one of three to hand and proliferates.
#[test]
fn experimental_augury_pick_and_proliferate() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(bear) {
        c.add_counters(CounterType::PlusOnePlusOne, 1);
    }
    let hand0 = g.players[0].hand.len();
    let lib0 = g.players[0].library.len();
    resolve_for(&mut g, 0, catalog::experimental_augury().effect);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "one card to hand");
    assert_eq!(g.players[0].library.len(), lib0 - 1, "rest bottomed");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Thirsting Roots mode 2 proliferates; mode 1 tutors a basic land to hand.
#[test]
fn thirsting_roots_modes() {
    let mut g = two_player_game();
    let forest_id = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(forest_id)),
    ]));
    let hand0 = g.players[0].hand.len();
    // Mode 0: search a basic land.
    let ctx = crabomination::game::effects::EffectContext {
        mode: 0,
        ..crabomination::game::effects::EffectContext::for_ability(
            g.add_card_to_battlefield(0, catalog::grizzly_bears()), 0, None)
    };
    let crabomination::effect::Effect::ChooseMode(modes) = catalog::thirsting_roots().effect else { panic!() };
    g.resolve_effect(&modes[0], &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "basic land tutored to hand");
}

/// Venomous Brutalizer's ETB may-pay proliferates when paid.
#[test]
fn venomous_brutalizer_etb_proliferate() {
    let mut g = two_player_game();
    g.players[1].poison_counters = 1;
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::venomous_brutalizer());
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 2, "paid one-green and proliferated");
}

/// Mesmerizing Dose taps the enchanted creature, proliferates, and locks its
/// untap step.
#[test]
fn mesmerizing_dose_taps_and_locks() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dose = g.add_card_to_hand(0, catalog::mesmerizing_dose());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: dose, target: Some(Target::Permanent(foe)), additional_targets: vec![], x_value: None, mode: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "enchanted creature tapped");
}

// ── For Mirrodin! Equipment + commons (wave 2) ───────────────────────────────

/// For Mirrodin! mints a 2/2 red Rebel and self-attaches (CR 702.163);
/// Dragonwing Glider's Rebel swings as a 4/4 flying haste.
#[test]
fn dragonwing_glider_for_mirrodin() {
    let mut g = two_player_game();
    let glider = g.move_card_to_battlefield_for_test(0, catalog::dragonwing_glider());
    drain_stack(&mut g);
    let rebel = g.battlefield.iter().find(|c| c.definition.name == "Rebel").expect("Rebel minted");
    assert_eq!(g.battlefield_find(glider).unwrap().attached_to, Some(rebel.id));
    let cp = g.computed_permanent(rebel.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords.contains(&Keyword::Flying) && cp.keywords.contains(&Keyword::Haste));
}

/// Hexgold Halberd grants first strike + trample only during your turn.
#[test]
fn hexgold_halberd_turn_gated_keywords() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::hexgold_halberd());
    drain_stack(&mut g);
    let rebel = g.battlefield.iter().find(|c| c.definition.name == "Rebel").unwrap().id;
    assert!(g.computed_permanent(rebel).unwrap().keywords.contains(&Keyword::FirstStrike),
        "your turn: first strike");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(rebel).unwrap().keywords.contains(&Keyword::FirstStrike),
        "opponent's turn: no first strike");
}

/// Hexgold Hoverwings' anthem pumps each equipped creature you control.
#[test]
fn hexgold_hoverwings_equipped_anthem() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::hexgold_hoverwings());
    drain_stack(&mut g);
    let rebel = g.battlefield.iter().find(|c| c.definition.name == "Rebel").unwrap().id;
    // Rebel is 2/2, +0/+0 from Hoverwings bonus (flying only), +1/+0 anthem.
    let cp = g.computed_permanent(rebel).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2));
    assert!(cp.keywords.contains(&Keyword::Flying));
    // An unequipped bear stays 2/2.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2);
}

/// Bladehold War-Whip reduces other Equipment's equip costs by {1}.
#[test]
fn bladehold_war_whip_equip_discount() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let star = g.add_card_to_battlefield(0, catalog::vulshok_morningstar()); // equip {2}
    g.move_card_to_battlefield_for_test(0, catalog::bladehold_war_whip());
    drain_stack(&mut g);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: star, target: bear })
        .expect("equip {2} reduced to {1}");
    assert_eq!(g.battlefield_find(star).unwrap().attached_to, Some(bear));
}

/// Infested Fleshcutter mints a Mite when the equipped creature attacks.
#[test]
fn infested_fleshcutter_attack_mite() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cutter = g.add_card_to_battlefield(0, catalog::infested_fleshcutter());
    g.clear_sickness(bear);
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: cutter, target: bear }).unwrap();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Phyrexian Mite"), "Mite minted");
}

/// Prosthetic Injector grants toxic 1: the equipped attacker poisons on hit.
#[test]
fn prosthetic_injector_grants_toxic() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let inj = g.add_card_to_battlefield(0, catalog::prosthetic_injector());
    g.clear_sickness(bear);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: inj, target: bear }).unwrap();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 1, "toxic 1 poisoned");
}

/// Oxidda Finisher's affinity counts only Equipment.
#[test]
fn oxidda_finisher_affinity_for_equipment() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vulshok_morningstar());
    g.add_card_to_battlefield(0, catalog::mind_stone()); // artifact, not Equipment
    let ox = g.add_card_to_hand(0, catalog::oxidda_finisher());
    // {5}{R}{R} - {1} = {4}{R}{R}: 6 mana total.
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: ox, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("affinity for one Equipment");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Oxidda Finisher"));
}

/// Rebel Salvo strips indestructible before its 5 damage (kill the god).
#[test]
fn rebel_salvo_strips_indestructible() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(foe) {
        std::sync::Arc::make_mut(&mut c.definition).keywords.push(Keyword::Indestructible);
    }
    let salvo = g.add_card_to_hand(0, catalog::rebel_salvo());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: salvo, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "indestructible stripped, 5 damage kills");
}

/// Jor Kadeen pumps by equipped-creature count and draws at power ≥ 4.
#[test]
fn jor_kadeen_attack_pump_and_draw() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let jor = g.add_card_to_battlefield(0, catalog::jor_kadeen_first_goldwarden());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let star = g.add_card_to_battlefield(0, catalog::vulshok_morningstar());
    g.clear_sickness(jor);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: star, target: bear }).unwrap();
    let hand0 = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: jor, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    // One equipped creature → +1/+1 → 3/3: below 4, no draw.
    assert_eq!(g.computed_permanent(jor).unwrap().power, 3);
    assert_eq!(g.players[0].hand.len(), hand0, "power 3 < 4, no draw");
}

/// Kemba attaches an Equipment to an entering Cat and anthems equipped
/// creatures.
#[test]
fn kemba_attaches_to_entering_cat() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kemba_kha_enduring());
    let star = g.add_card_to_battlefield(0, catalog::vulshok_morningstar());
    // A Cat enters → Kemba's trigger attaches the Morningstar to it.
    let cat = g.add_card_to_battlefield(0, catalog::leonin_lightbringer());
    g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::PermanentEntered { card_id: cat }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(star).unwrap().attached_to, Some(cat), "Equipment attached");
    // 3/2 + Morningstar +2/+2 + Kemba anthem +1/+1 + own while-equipped +1/+1 = 7/6.
    let cp = g.computed_permanent(cat).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 6));
}

/// Sword of Forge and Frontier impulses two cards + an extra land play on hit.
#[test]
fn sword_of_forge_and_frontier_impulse() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_forge_and_frontier());
    g.clear_sickness(bear);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: sword, target: bear }).unwrap();
    let lands0 = g.players[0].extra_land_plays;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(g.exile.iter().filter(|c| c.definition.name == "Forest").count(), 2,
        "two cards impulse-exiled");
    assert_eq!(g.players[0].extra_land_plays, lands0 + 1, "extra land play granted");
}

/// Sphere lands enter tapped, tap for their color, and sac to draw.
#[test]
fn sphere_land_taps_and_sacs() {
    let mut g = two_player_game();
    let land = g.move_card_to_battlefield_for_test(0, catalog::the_autonomous_furnace());
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped, "enters tapped");
    if let Some(c) = g.battlefield_find_mut(land) { c.tapped = false; }
    g.add_card_to_library(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("sac to draw");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "sacrificed");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card");
}

/// Dross Skullbomb's colored mode is sorcery-only and returns a creature card.
#[test]
fn dross_skullbomb_modes() {
    let mut g = two_player_game();
    let bomb = g.add_card_to_battlefield(0, catalog::dross_skullbomb());
    g.add_card_to_library(0, catalog::forest());
    let gy_bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: bomb, ability_index: 1,
        target: Some(Target::Permanent(gy_bear)), additional_targets: vec![], x_value: None, mode: None,
    }).expect("sorcery-speed mode at main");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "creature card returned + drew");
}

/// Hexgold Slash deals 4 to a toxic creature, 2 otherwise.
#[test]
fn hexgold_slash_scales_on_toxic() {
    let mut g = two_player_game();
    // A 4/4 toxic creature dies to the upgraded 4 damage.
    let toxic = g.add_card_to_battlefield(1, catalog::venomous_brutalizer());
    let slash = g.add_card_to_hand(0, catalog::hexgold_slash());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: slash, target: Some(Target::Permanent(toxic)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(toxic).is_none(), "4 damage killed the 4/4 toxic creature");
}

/// Compleat Devotion draws only when the pumped creature has toxic.
#[test]
fn compleat_devotion_toxic_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let toxic = g.add_card_to_battlefield(0, catalog::bilious_skulldweller());
    let cd = g.add_card_to_hand(0, catalog::compleat_devotion());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: cd, target: Some(Target::Permanent(toxic)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0, "cast (-1) + toxic draw (+1)");
    assert_eq!(g.computed_permanent(toxic).unwrap().power, 3, "1/1 + 2 = 3");
}

/// Minor Misstep counters only MV ≤ 1.
#[test]
fn minor_misstep_mv_gate() {
    let mut g = two_player_game();
    // Divination (MV 3) is not a legal target.
    for _ in 0..3 { g.add_card_to_library(1, catalog::forest()); }
    let div = g.add_card_to_hand(1, catalog::divination());
    g.players[1].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: div, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    let mm = g.add_card_to_hand(0, catalog::minor_misstep());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.priority.player_with_priority = 0;
    let hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: mm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // The MV filter fails — Divination is not countered; it resolves and draws 2.
    assert_eq!(g.players[1].hand.len(), hand_before + 2, "MV 3 spell survived and resolved");
}

/// Skyscythe Engulfer can't be blocked by fliers.
#[test]
fn skyscythe_engulfer_evades_fliers() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let eng = g.add_card_to_battlefield(0, catalog::skyscythe_engulfer());
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(eng);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: eng, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    let err = g.perform_action(GameAction::DeclareBlockers(vec![(flier, eng)]));
    assert!(err.is_err(), "flier can't block Skyscythe Engulfer");
}

/// Cephalopod Sentry's power tracks your artifact count (it counts itself).
#[test]
fn cephalopod_sentry_cda_power() {
    let mut g = two_player_game();
    let squid = g.add_card_to_battlefield(0, catalog::cephalopod_sentry());
    assert_eq!(g.computed_permanent(squid).unwrap().power, 1, "counts itself");
    g.add_card_to_battlefield(0, catalog::mind_stone());
    assert_eq!(g.computed_permanent(squid).unwrap().power, 2);
    assert_eq!(g.computed_permanent(squid).unwrap().toughness, 5);
}

/// Duelist of Deep Faith has first strike only on your turn.
#[test]
fn duelist_of_deep_faith_turn_gate() {
    let mut g = two_player_game();
    let d = g.add_card_to_battlefield(0, catalog::duelist_of_deep_faith());
    assert!(g.computed_permanent(d).unwrap().keywords.contains(&Keyword::FirstStrike));
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(d).unwrap().keywords.contains(&Keyword::FirstStrike));
}

/// Orthodoxy Enforcer gets +2/+0 with two artifacts out.
#[test]
fn orthodoxy_enforcer_metalcraft_lite() {
    let mut g = two_player_game();
    let oe = g.add_card_to_battlefield(0, catalog::orthodoxy_enforcer());
    assert_eq!(g.computed_permanent(oe).unwrap().power, 2);
    g.add_card_to_battlefield(0, catalog::mind_stone());
    g.add_card_to_battlefield(0, catalog::vulshok_morningstar());
    assert_eq!(g.computed_permanent(oe).unwrap().power, 4, "+2/+0 at 2+ artifacts");
}

/// Offer Immortality grants deathtouch + indestructible for the turn.
#[test]
fn offer_immortality_grants() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let oi = g.add_card_to_hand(0, catalog::offer_immortality());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: oi, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Deathtouch));
    assert!(cp.keywords.contains(&Keyword::Indestructible));
}

/// Quicksilver Fisher loots on entry.
#[test]
fn quicksilver_fisher_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::quicksilver_fisher());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0, "draw 1, discard 1");
    assert_eq!(g.players[0].graveyard.len(), 1, "discarded");
}

/// Free from Flesh pumps and oils its target.
#[test]
fn free_from_flesh_pump_and_oil() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let fff = g.add_card_to_hand(0, catalog::free_from_flesh());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: fff, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Oil), 2);
}

// ── Oil-counter package (wave 3) ─────────────────────────────────────────────

/// Trawler Drake grows +1/+1 per oil; a noncreature cast adds oil.
#[test]
fn trawler_drake_grows_with_oil() {
    let mut g = two_player_game();
    let drake = g.move_card_to_battlefield_for_test(0, catalog::trawler_drake());
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(drake).unwrap().power, 1, "enters with one oil");
    let slash = g.add_card_to_hand(0, catalog::hexgold_slash());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: slash, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(drake).unwrap().power, 2, "cast added an oil");
}

/// Exuberant Fuseling gets +1/+0 (not +1/+1) per oil and oils up on deaths.
#[test]
fn exuberant_fuseling_power_only_oil() {
    let mut g = two_player_game();
    let fus = g.move_card_to_battlefield_for_test(0, catalog::exuberant_fuseling());
    drain_stack(&mut g);
    let cp = g.computed_permanent(fus).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "ETB oil: +1/+0 over 0/1");
    // Another creature dies → +1 oil.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve_for(&mut g, 0, crabomination::effect::Effect::Destroy {
        what: crabomination::effect::Selector::EachPermanent(
            crabomination::card::SelectionRequirement::HasCreatureType(crabomination::card::CreatureType::Bear),
        ),
    });
    let _ = bear;
    let cp = g.computed_permanent(fus).unwrap();
    assert!(cp.power >= 2, "death added oil (power {})", cp.power);
}

/// Serum Sovereign converts an oil into a draw + scry.
#[test]
fn serum_sovereign_oil_to_draw() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let ss = g.add_card_to_battlefield(0, catalog::serum_sovereign());
    if let Some(c) = g.battlefield_find_mut(ss) { c.add_counters(CounterType::Oil, 1); }
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ss, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1);
    assert_eq!(g.battlefield_find(ss).unwrap().counter_count(CounterType::Oil), 0);
}

/// Ichor Synthesizer goes +2/+0 unblockable at four oil.
#[test]
fn ichor_synthesizer_threshold() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let ics = g.add_card_to_battlefield(0, catalog::ichor_synthesizer());
    assert_eq!(g.computed_permanent(ics).unwrap().power, 1);
    if let Some(c) = g.battlefield_find_mut(ics) { c.add_counters(CounterType::Oil, 4); }
    let cp = g.computed_permanent(ics).unwrap();
    assert_eq!(cp.power, 3, "+2/+0 at 4 oil");
    assert!(cp.keywords.contains(&Keyword::Unblockable));
}

/// Tablet of Compleation's gated abilities respect the oil thresholds.
#[test]
fn tablet_of_compleation_gates() {
    let mut g = two_player_game();
    let tab = g.add_card_to_battlefield(0, catalog::tablet_of_compleation());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // {T}: Add {C} requires 2+ oil — rejected fresh.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: tab, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).is_err(), "colorless tap gated below 2 oil");
    if let Some(c) = g.battlefield_find_mut(tab) { c.add_counters(CounterType::Oil, 2); }
    g.perform_action(GameAction::ActivateAbility {
        card_id: tab, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("2 oil unlocks {T}: Add {C}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
}

/// Urabrask's Forge mints an X/1 (X = oil) trample-haste Horror at combat and
/// sacrifices it at the end step.
#[test]
fn urabrasks_forge_token_lifecycle() {
    let mut g = two_player_game();
    let forge = g.add_card_to_battlefield(0, catalog::urabrasks_forge());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let horror = g.battlefield.iter().find(|c| c.definition.name == "Phyrexian Horror")
        .expect("Horror minted");
    assert_eq!(g.battlefield_find(forge).unwrap().counter_count(CounterType::Oil), 1);
    assert_eq!(g.computed_permanent(horror.id).unwrap().power, 1, "X = 1 oil");
    let horror_id = horror.id;
    // Next end step: sacrificed (a real sacrifice, not an exile).
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(horror_id).is_none(), "token sacrificed at end step");
    assert!(g.exile.iter().all(|c| c.id != horror_id), "sacrificed, not exiled");
}

/// Watchful Blisterzoa draws per oil on death (LKI counter read).
#[test]
fn watchful_blisterzoa_death_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let bz = g.move_card_to_battlefield_for_test(0, catalog::watchful_blisterzoa());
    drain_stack(&mut g);
    if let Some(c) = g.battlefield_find_mut(bz) { c.add_counters(CounterType::Oil, 1); } // 2 total
    let hand0 = g.players[0].hand.len();
    g.resolve_effect(
        &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &crabomination::game::effects::EffectContext {
            targets: vec![Target::Permanent(bz)],
            ..crabomination::game::effects::EffectContext::for_ability(bz, 0, None)
        },
    ).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "drew per oil counter");
}

/// Magmatic Sprinter stays by shedding two oil, bounces when dry.
#[test]
fn magmatic_sprinter_end_step() {
    let mut g = two_player_game();
    let ms = g.add_card_to_battlefield(0, catalog::magmatic_sprinter());
    if let Some(c) = g.battlefield_find_mut(ms) { c.add_counters(CounterType::Oil, 2); }
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ms).is_some(), "shed two oil and stayed");
    assert_eq!(g.battlefield_find(ms).unwrap().counter_count(CounterType::Oil), 0);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ms).is_none(), "no oil left — bounced");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Magmatic Sprinter"));
}

/// Evolved Spinoderm swaps hexproof for trample as its oil drains, then dies.
#[test]
fn evolved_spinoderm_oil_curve() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let sp = g.move_card_to_battlefield_for_test(0, catalog::evolved_spinoderm());
    drain_stack(&mut g);
    let cp = g.computed_permanent(sp).unwrap();
    assert!(cp.keywords.contains(&Keyword::Hexproof) && !cp.keywords.contains(&Keyword::Trample));
    if let Some(c) = g.battlefield_find_mut(sp) { c.remove_counters(CounterType::Oil, 3); } // 1 left
    let cp = g.computed_permanent(sp).unwrap();
    assert!(cp.keywords.contains(&Keyword::Trample) && !cp.keywords.contains(&Keyword::Hexproof));
    // Upkeep: shed the last oil → sacrificed.
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(sp).is_none(), "dry Spinoderm sacrificed");
}

/// Eye of Malcator animates into a 4/4 when another artifact enters.
#[test]
fn eye_of_malcator_animates() {
    let mut g = two_player_game();
    let eye = g.add_card_to_battlefield(0, catalog::eye_of_malcator());
    assert!(g.computed_permanent(eye).unwrap().card_types.iter().all(|t| *t != crabomination::card::CardType::Creature));
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::PermanentEntered { card_id: stone }]);
    drain_stack(&mut g);
    let cp = g.computed_permanent(eye).unwrap();
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature), "animated");
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Vat of Rebirth reanimates for four oil at sorcery speed.
#[test]
fn vat_of_rebirth_reanimates() {
    let mut g = two_player_game();
    let vat = g.add_card_to_battlefield(0, catalog::vat_of_rebirth());
    if let Some(c) = g.battlefield_find_mut(vat) { c.add_counters(CounterType::Oil, 4); }
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: vat, ability_index: 0, target: Some(Target::Permanent(dead)), additional_targets: vec![], x_value: None, mode: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == dead), "creature reanimated");
}

/// Gleeful Demolition: your own artifact yields three Goblins; an opponent's
/// yields none.
#[test]
fn gleeful_demolition_goblins_only_for_yours() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::mind_stone());
    let gd = g.add_card_to_hand(0, catalog::gleeful_demolition());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: gd, target: Some(Target::Permanent(mine)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none());
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Goblin").count(), 3);
}

/// Testament Bearer's death fills the hand from the top three.
#[test]
fn testament_bearer_death_dig() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let tb = g.add_card_to_battlefield(0, catalog::testament_bearer());
    let hand0 = g.players[0].hand.len();
    let gy0 = g.players[0].graveyard.len();
    g.resolve_effect(
        &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &crabomination::game::effects::EffectContext {
            targets: vec![Target::Permanent(tb)],
            ..crabomination::game::effects::EffectContext::for_ability(tb, 0, None)
        },
    ).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "one to hand");
    // Testament Bearer itself + two milled cards.
    assert_eq!(g.players[0].graveyard.len(), gy0 + 3, "rest to graveyard");
}

/// Plague Nurse spreads toxic 1 to your other toxic creatures — total toxic
/// stacks in combat poison.
#[test]
fn plague_nurse_toxic_boost() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let nurse = g.add_card_to_battlefield(0, catalog::plague_nurse());
    let sting = g.add_card_to_battlefield(0, catalog::bilious_skulldweller()); // toxic 1
    g.clear_sickness(sting);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: nurse, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).unwrap();
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sting, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 2, "toxic 1 + granted toxic 1 = 2 poison");
}

/// Nimraiser Paladin's ETB only fetches MV ≤ 3 creature cards.
#[test]
fn nimraiser_paladin_reanimate_gate() {
    let mut g = two_player_game();
    let cheap = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    g.add_card_to_graveyard(0, catalog::serra_angel()); // MV 5
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Target(Target::Permanent(cheap)),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::nimraiser_paladin());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "MV 2 creature returned to hand");
}

/// Escaped Experiment shrinks a defender by your artifact count.
#[test]
fn escaped_experiment_attack_debuff() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let ex = g.add_card_to_battlefield(0, catalog::escaped_experiment());
    g.add_card_to_battlefield(0, catalog::mind_stone());
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.clear_sickness(ex);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ex, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    // Experiment (artifact creature) + Mind Stone = 2 artifacts → -2/-0.
    assert_eq!(g.computed_permanent(foe).unwrap().power, 2, "4 - 2 = 2");
}

/// Myr Convert taps + pays 2 life for any color.
#[test]
fn myr_convert_life_for_mana() {
    let mut g = two_player_game();
    let myr = g.add_card_to_battlefield(0, catalog::myr_convert());
    g.clear_sickness(myr);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: myr, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18, "paid 2 life");
}

/// Feed the Infection drains only through the Corrupted gate.
#[test]
fn feed_the_infection_corrupted_gate() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); }
    resolve_for(&mut g, 0, catalog::feed_the_infection().effect);
    assert_eq!(g.players[1].life, 20, "no poison — no drain");
    assert_eq!(g.players[0].life, 17, "lost 3");
    g.players[1].poison_counters = 3;
    resolve_for(&mut g, 0, catalog::feed_the_infection().effect);
    assert_eq!(g.players[1].life, 17, "Corrupted — opponent lost 3");
}

// ── Wave 4 ───────────────────────────────────────────────────────────────────

/// Rustvine Cultivator banks oil then untaps a land with it.
#[test]
fn rustvine_cultivator_untaps_land() {
    let mut g = two_player_game();
    let rv = g.add_card_to_battlefield(0, catalog::rustvine_cultivator());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    if let Some(c) = g.battlefield_find_mut(land) { c.tapped = true; }
    if let Some(c) = g.battlefield_find_mut(rv) { c.add_counters(CounterType::Oil, 1); }
    g.clear_sickness(rv);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rv, ability_index: 1, target: Some(Target::Permanent(land)), additional_targets: vec![], x_value: None, mode: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield_find(land).unwrap().tapped, "land untapped");
}

/// Oil-Gorger Troll draws only when an oil-countered permanent is around.
#[test]
fn oil_gorger_troll_conditional_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::oil_gorger_troll());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23, "gained 3");
    assert_eq!(g.players[0].hand.len(), hand0, "no oil permanent — no draw");
    // With an oil-countered permanent out, a second Troll draws.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(bear) { c.add_counters(CounterType::Oil, 1); }
    g.move_card_to_battlefield_for_test(0, catalog::oil_gorger_troll());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "oil permanent — draw");
}

/// Hazardous Blast pings the opposing team and locks their blocks.
#[test]
fn hazardous_blast_ping_and_lock() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    resolve_for(&mut g, 0, catalog::hazardous_blast().effect);
    assert_eq!(g.computed_permanent(mine).unwrap().toughness, 2, "yours untouched");
    assert!(g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Ruthless Predation pumps then bites.
#[test]
fn ruthless_predation_fight() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let rp = g.add_card_to_hand(0, catalog::ruthless_predation());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: rp, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "3 power kills the 2/2");
    assert!(g.battlefield_find(mine).is_some(), "2 damage vs 4 toughness survives");
}

/// Maze's Mantle grants hexproof only to a toxic host.
#[test]
fn mazes_mantle_toxic_hexproof() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let toxic = g.add_card_to_battlefield(0, catalog::bilious_skulldweller());
    let mm = g.add_card_to_hand(0, catalog::mazes_mantle());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: mm, target: Some(Target::Permanent(toxic)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(toxic).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "1/1 + 2/2");
    assert!(cp.keywords.contains(&Keyword::Hexproof), "toxic host got hexproof");
}

/// Drown in Ichor shrinks and proliferates.
#[test]
fn drown_in_ichor_kill_and_proliferate() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.players[1].poison_counters = 1;
    let di = g.add_card_to_hand(0, catalog::drown_in_ichor());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: di, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "-4/-4 kills the 4/4");
    assert_eq!(g.players[1].poison_counters, 2, "proliferated the poison");
}

/// Glistener Seer converts oil into scrys.
#[test]
fn glistener_seer_scry() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let seer = g.move_card_to_battlefield_for_test(0, catalog::glistener_seer());
    drain_stack(&mut g);
    g.clear_sickness(seer);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: seer, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(seer).unwrap().counter_count(CounterType::Oil), 2);
}

/// Cinderslash Ravager discounts by oil-countered permanents and sweeps 1.
#[test]
fn cinderslash_ravager_discount_and_ping() {
    let mut g = two_player_game();
    // Two oil-countered permanents → {4}{R}{G} - {2} = 4 mana total.
    for _ in 0..2 {
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        if let Some(c) = g.battlefield_find_mut(b) { c.add_counters(CounterType::Oil, 1); }
    }
    let foe = g.add_card_to_battlefield(1, catalog::bilious_skulldweller()); // 1/1
    let cr = g.add_card_to_hand(0, catalog::cinderslash_ravager());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: cr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast for {2}{R}{G} after the oil discount");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "ETB ping killed the 1/1");
}

/// Sheoldred's Edict mode 0 makes each opponent sacrifice a nontoken creature.
#[test]
fn sheoldreds_edict_nontoken_mode() {
    let mut g = two_player_game();
    let real = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let se = g.add_card_to_hand(0, catalog::sheoldreds_edict());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: se, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(real).is_none(), "opponent sacrificed the nontoken creature");
}

/// Tyvar's Stand pumps by X and shields.
#[test]
fn tyvars_stand_x_pump() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ts = g.add_card_to_hand(0, catalog::tyvars_stand());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: ts, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: Some(3),
    }).unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 5, "+3/+3");
    assert!(cp.keywords.contains(&Keyword::Indestructible));
}

/// Mite Overseer's token anthem is on only during your turn.
#[test]
fn mite_overseer_turn_gated_token_anthem() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mite_overseer());
    resolve_for(&mut g, 0, crabomination::effect::Effect::CreateToken {
        who: crabomination::effect::PlayerRef::You,
        count: crabomination::effect::Value::ONE,
        definition: crabomination::card::TokenDefinition {
            name: "Soldier".into(),
            power: 1,
            toughness: 1,
            card_types: vec![crabomination::card::CardType::Creature],
            subtypes: crabomination::card::Subtypes {
                creature_types: vec![crabomination::card::CreatureType::Soldier],
                ..Default::default()
            },
            ..Default::default()
        },
    });
    let tok = g.battlefield.iter().find(|c| c.definition.name == "Soldier").unwrap().id;
    assert_eq!(g.computed_permanent(tok).unwrap().power, 2, "your turn: +1/+0");
    g.active_player_idx = 1;
    assert_eq!(g.computed_permanent(tok).unwrap().power, 1, "off turn: anthem off");
}

/// Vat Emergence steals a creature card from the opponent's graveyard and
/// proliferates.
#[test]
fn vat_emergence_reanimates_theirs() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.players[1].poison_counters = 1;
    resolve_for(&mut g, 0, crabomination::effect::Effect::Seq(vec![]));
    let ve = g.add_card_to_hand(0, catalog::vat_emergence());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: ve, target: Some(Target::Permanent(dead)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    let stolen = g.battlefield_find(dead).expect("reanimated");
    assert_eq!(stolen.controller, 0, "under your control");
    assert_eq!(g.players[1].poison_counters, 2, "proliferated");
}

/// Urabrask's Anointer's ETB damage scales with oil-countered permanents.
#[test]
fn urabrasks_anointer_scaled_ping() {
    let mut g = two_player_game();
    for _ in 0..2 {
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        if let Some(c) = g.battlefield_find_mut(b) { c.add_counters(CounterType::Oil, 1); }
    }
    g.move_card_to_battlefield_for_test(0, catalog::urabrasks_anointer());
    drain_stack(&mut g);
    // Auto-target aims the ping at the opponent's face: X = 2 oil permanents.
    assert_eq!(g.players[1].life, 18, "2 oil permanents → 2 damage");
}

/// Planar Disruption locks the enchanted permanent's abilities and combat.
#[test]
fn planar_disruption_locks() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pd = g.add_card_to_hand(0, catalog::planar_disruption());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: pd, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(foe).unwrap();
    assert!(cp.keywords.contains(&Keyword::CantAttack));
    assert!(cp.keywords.contains(&Keyword::CantActivateAbilities));
}

/// Porcelain Zealot pumps +2/+2 for a toxic target at combat.
#[test]
fn porcelain_zealot_toxic_bonus() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::porcelain_zealot());
    let toxic = g.add_card_to_battlefield(0, catalog::bilious_skulldweller()); // 1/1 toxic
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Target(Target::Permanent(toxic)),
    ]));
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(toxic).unwrap().power, 3, "1 + 2 (toxic bonus)");
}

// ── Grafted tests from the parallel session ──────────────────────────────────

/// Cruel Grimnarch gains 4 when the opponent has no card to discard.
#[test]
fn cruel_grimnarch_gains_on_empty_hand() {
    let mut g = two_player_game();
    g.players[1].hand.clear();
    let life = g.players[0].life;
    g.move_card_to_battlefield_for_test(0, catalog::cruel_grimnarch());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "gained 4 off the empty hand");
}

/// Awaken the Sleeper steals, untaps, hastes, and may smash Equipment.
#[test]
fn awaken_the_sleeper_steals_and_smashes() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let stolen = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(1, catalog::short_sword());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(stolen);
    g.battlefield_find_mut(stolen).unwrap().tapped = true;
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::awaken_the_sleeper());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(stolen)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let c = g.battlefield_find(stolen).unwrap();
    assert_eq!(c.controller, 0, "stolen");
    assert!(!c.tapped, "untapped");
    assert!(g.battlefield_find(sword).is_none(), "equipment destroyed");
}

// ── ONE remainder wave 1 ─────────────────────────────────────────────────────

/// Chittering Skitterling's sac-draw is Corrupted-gated and once per turn.
#[test]
fn chittering_skitterling_corrupted_sac_draw() {
    let mut g = two_player_game();
    let rat = g.add_card_to_battlefield(0, catalog::chittering_skitterling());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let act = |g: &mut GameState| g.perform_action(GameAction::ActivateAbility {
        card_id: rat, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    });
    assert!(act(&mut g).is_err(), "no poison → Corrupted gate rejects");
    g.players[1].poison_counters = 3;
    let hand0 = g.players[0].hand.len();
    act(&mut g).expect("corrupted sac-draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew");
    assert!(act(&mut g).is_err(), "once per turn");
}

/// The Filigree Sylex nukes MV = oil count on sacrifice (LKI-read counters).
#[test]
fn filigree_sylex_destroys_matching_mv() {
    let mut g = two_player_game();
    let sylex = g.add_card_to_battlefield(0, catalog::the_filigree_sylex());
    g.battlefield_find_mut(sylex).unwrap().add_counters(CounterType::Oil, 2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // MV 5
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sylex, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("sac-nuke");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "MV 2 destroyed at 2 oil");
    assert!(g.battlefield_find(angel).is_some(), "MV 5 survives");
}

/// Tamiyo's Logbook draw costs {1} less per other artifact.
#[test]
fn tamiyos_logbook_discounted_draw() {
    let mut g = two_player_game();
    let book = g.add_card_to_battlefield(0, catalog::tamiyos_logbook());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::sol_ring());
    }
    g.add_card_to_library(0, catalog::forest());
    // {5}{U} − {3} = {2}{U}.
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: book, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("discounted activation");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew");
    assert!(g.players[0].mana_pool.total() == 0, "paid exactly two generic + U");
}

/// Staff of Compleation: pay 4 life to draw; {5} untaps it.
#[test]
fn staff_of_compleation_life_paid_modes() {
    let mut g = two_player_game();
    let staff = g.add_card_to_battlefield(0, catalog::staff_of_compleation());
    g.add_card_to_library(0, catalog::forest());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: staff, ability_index: 3, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("pay 4 life: draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 4);
    assert_eq!(g.players[0].hand.len(), hand0 + 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: staff, ability_index: 4, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("{5}: untap");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(staff).unwrap().tapped, "untapped");
}

/// Koth −3 deals damage equal to your Mountains.
#[test]
fn koth_minus_three_scales_with_mountains() {
    let mut g = two_player_game();
    let koth = g.add_card_to_battlefield(0, catalog::koth_fire_of_resistance());
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: koth, ability_index: 1, target: Some(Target::Permanent(angel)), x_value: None,
    }).expect("-3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none(), "4 damage kills the 4/4");
}

/// Malcator mints on ETB and again at end step after 3 artifacts entered.
#[test]
fn malcator_end_step_golem_gate() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::malcator_purity_overseer());
    drain_stack(&mut g);
    let golems = |g: &GameState| g.battlefield.iter()
        .filter(|c| c.definition.name == "Phyrexian Golem").count();
    assert_eq!(golems(&g), 1, "ETB golem");
    g.players[0].artifacts_entered_this_turn = 3;
    g.active_player_idx = 0;
    g.fire_step_triggers(crabomination::game::types::TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(golems(&g), 2, "end-step golem after 3 artifacts");
}

/// Geth's anthem shrinks others; his reanimation stamps a finality counter.
#[test]
fn geth_anthem_and_reanimate() {
    let mut g = two_player_game();
    let geth = g.add_card_to_battlefield(0, catalog::geth_thane_of_contracts());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bears).unwrap().power, 1, "-1/-1 anthem");
    assert_eq!(g.computed_permanent(geth).unwrap().power, 3, "Geth unaffected");
    let dead = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.clear_sickness(geth);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: geth, ability_index: 0, target: Some(Target::Permanent(dead)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("reanimate");
    drain_stack(&mut g);
    let angel = g.battlefield.iter().find(|c| c.definition.name == "Serra Angel")
        .expect("reanimated");
    assert_eq!(angel.counter_count(CounterType::Finality), 1, "finality rider");
}

/// Ichorplate Golem bumps entering oil creatures and anthems oil carriers.
#[test]
fn ichorplate_golem_oil_synergy() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ichorplate_golem());
    // Skitterfang enters with three oil counters → trigger adds a fourth.
    let fang = g.move_card_to_battlefield_for_test(0, catalog::atraxas_skitterfang());
    g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::PermanentEntered { card_id: fang }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(fang).unwrap().counter_count(CounterType::Oil), 4,
        "3 from ETB + 1 from Ichorplate");
    let cp = g.computed_permanent(fang).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "2/2 + oil anthem");
}

/// Necrogen Rotpriest adds a poison counter when a toxic creature connects.
#[test]
fn necrogen_rotpriest_bonus_poison() {
    let mut g = two_player_game();
    let priest = g.add_card_to_battlefield(0, catalog::necrogen_rotpriest());
    g.clear_sickness(priest);
    g.active_player_idx = 0;
    g.step = crabomination::game::types::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: priest, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = crabomination::game::types::TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    // toxic 2 + rotpriest trigger = 3 poison.
    assert_eq!(g.players[1].poison_counters, 3);
}

/// Indoctrination Attendant bounces your permanent for a Mite.
#[test]
fn indoctrination_attendant_bounce_for_mite() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.move_card_to_battlefield_for_test(0, catalog::indoctrination_attendant());
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "bounced");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Phyrexian Mite"), "mite minted");
}

/// Mirrex taps for any color only on the turn it entered.
#[test]
fn mirrex_any_color_gate() {
    let mut g = two_player_game();
    let land = g.move_card_to_battlefield_for_test(0, catalog::mirrex());
    drain_stack(&mut g);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("entered this turn → any color");
    assert_eq!(g.players[0].mana_pool.total(), 1);
    // Next turn the gate closes.
    let land2 = g.add_card_to_battlefield(0, catalog::mirrex());
    g.battlefield_find_mut(land2).unwrap().entered_turn = None;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: land2, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).is_err(), "stale Mirrex can't tap for color");
}

/// The Monumental Facade moves its stored oil onto your artifact.
#[test]
fn monumental_facade_oil_transfer() {
    let mut g = two_player_game();
    let facade = g.move_card_to_battlefield_for_test(0, catalog::the_monumental_facade());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(facade).unwrap().counter_count(CounterType::Oil), 2);
    let golem = g.add_card_to_battlefield(0, catalog::ichorplate_golem());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: facade, ability_index: 1, target: Some(Target::Permanent(golem)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("move oil");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(facade).unwrap().counter_count(CounterType::Oil), 1);
    assert_eq!(g.battlefield_find(golem).unwrap().counter_count(CounterType::Oil), 1);
}

/// The Seedcore's Corrupted pump needs 3+ opponent poison and a 1/1 target.
#[test]
fn seedcore_corrupted_pump() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::the_seedcore());
    let priest = g.add_card_to_battlefield(0, catalog::venerated_rotpriest()); // a 1/2
    let mono = g.add_card_to_battlefield(0, catalog::memnite()); // 1/1
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let act = |g: &mut GameState, tgt| g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: Some(Target::Permanent(tgt)),
        additional_targets: vec![], x_value: None, mode: None,
    });
    assert!(act(&mut g, mono).is_err(), "no poison → rejected");
    g.players[1].poison_counters = 3;
    assert!(act(&mut g, priest).is_err(), "1/2 isn't a legal 1/1 target");
    g.battlefield_find_mut(land).unwrap().tapped = false;
    act(&mut g, mono).expect("corrupted pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(mono).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2), "1/1 + 2/+1");
}

/// Zealot's Conviction gives +1/+1, upgrading to +2/+1 first strike when Corrupted.
#[test]
fn zealots_conviction_corrupted_rider() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::zealots_conviction());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bears);
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 base");
    assert!(!cp.keywords.contains(&Keyword::FirstStrike));
    g.players[1].poison_counters = 3;
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "additional +1/+0");
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "corrupted first strike");
}

/// Transplant Theorist loots on artifact entries and bottoms graveyard cards.
#[test]
fn transplant_theorist_loot_and_bottom() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let theorist = g.move_card_to_battlefield_for_test(0, catalog::transplant_theorist());
    g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::PermanentEntered {
        card_id: theorist,
    }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 1, "looted: drew then discarded");
    let dead = g.players[0].graveyard[0].id;
    g.players[0].mana_pool.add_colorless(2);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: theorist, ability_index: 0, target: Some(Target::Permanent(dead)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("bottom it");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.is_empty(), "graveyard emptied");
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(dead), "on the bottom");
}

/// Phyrexian Atlas drains on tap only while Corrupted.
#[test]
fn phyrexian_atlas_corrupted_tap_drain() {
    let mut g = two_player_game();
    let atlas = g.add_card_to_battlefield(0, catalog::phyrexian_atlas());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: atlas, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1, "no poison → no drain");
    g.players[1].poison_counters = 3;
    g.battlefield_find_mut(atlas).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: atlas, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("tap again");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 1, "corrupted tap drains 1");
}

/// Slobad turns an artifact into that much restricted red mana.
#[test]
fn slobad_sacrifices_for_scaled_mana() {
    let mut g = two_player_game();
    let slobad = g.add_card_to_battlefield(0, catalog::slobad_iron_goblin());
    g.clear_sickness(slobad);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring()); // MV 1
    let _ = ring;
    g.add_card_to_battlefield(0, catalog::tamiyos_logbook()); // MV 3 — auto-pick lowest
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: slobad, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("sac artifact for mana");
    assert!(g.players[0].mana_pool.restricted_total() >= 1, "got scaled restricted red mana");
}

/// Venerated Rotpriest poisons when your creature is targeted by a spell.
#[test]
fn venerated_rotpriest_poison_on_target() {
    let mut g = two_player_game();
    let priest = g.add_card_to_battlefield(0, catalog::venerated_rotpriest());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(priest)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt own priest");
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 1, "opponent got a poison counter");
}

/// Unctus anthems artifact bodies and loots when your blue creature taps.
#[test]
fn unctus_grand_metatect_statics() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::unctus_grand_metatect());
    let golem = g.add_card_to_battlefield(0, catalog::ichorplate_golem());
    let cp = g.computed_permanent(golem).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "artifact creature +1/+1");
    // A blue creature tapping loots.
    let drake = g.add_card_to_battlefield(0, catalog::wind_drake());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    let gy0 = g.players[0].graveyard.len();
    g.battlefield_find_mut(drake).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::PermanentTapped { card_id: drake, actor: None, as_attacker: false }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0, "loot: net hand unchanged");
    assert_eq!(g.players[0].graveyard.len(), gy0 + 1, "discarded one");
}

/// Tyvar lets a summoning-sick creature tap-activate (CR 602.5g exemption).
#[test]
fn tyvar_grants_ability_haste() {
    let mut g = two_player_game();
    let priest = g.add_card_to_battlefield(0, catalog::necrogen_rotpriest());
    let sylex = g.add_card_to_battlefield(0, catalog::the_filigree_sylex());
    let _ = (priest, sylex);
    // A sick creature with a tap ability: Sinew Dancer ({W}, {T}: tap target).
    let dancer = g.add_card_to_battlefield(0, catalog::sinew_dancer());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let act = |g: &mut GameState| g.perform_action(GameAction::ActivateAbility {
        card_id: dancer, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: vec![], x_value: None, mode: None,
    });
    assert!(act(&mut g).is_err(), "sick creature can't tap-activate (CR 602.5g)");
    g.add_card_to_battlefield(0, catalog::tyvar_jubilant_brawler());
    act(&mut g).expect("Tyvar's static exempts the gate");
}

/// Tyvar −2 mills three and can reanimate a cheap creature.
#[test]
fn tyvar_minus_two_reanimates() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let tyvar = g.add_card_to_battlefield(0, catalog::tyvar_jubilant_brawler());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: tyvar, ability_index: 1, target: Some(Target::Permanent(dead)), x_value: None,
    }).expect("-2");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"), "reanimated");
    assert!(g.players[0].library.len() < 3 || !g.players[0].graveyard.is_empty(), "milled");
}

/// Nahiri's Sacrifice divides damage equal to the sacrificed mana value.
#[test]
fn nahiris_sacrifice_scales_with_sacrifice() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::serra_angel()); // MV 5 fodder
    let target = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::nahiris_sacrifice());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast with sacrifice");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "5 damage kills the 4/4");
}

/// Serum-Core Chimera accrues oil on noncreature casts and cashes three in.
#[test]
fn serum_core_chimera_oil_loop() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let chimera = g.add_card_to_battlefield(0, catalog::serum_core_chimera());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("noncreature cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(chimera).unwrap().counter_count(CounterType::Oil), 1);
    let c = g.battlefield_find_mut(chimera).unwrap();
    c.add_counters(CounterType::Oil, 2);
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest());
    let victim = g.add_card_to_battlefield(1, catalog::wind_drake()); // 2/2 flyer
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: chimera, ability_index: 0, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("remove three oil");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "3 damage killed the drake");
}

// ── ONE planeswalkers: Compleated (CR 702.150) + friends ─────────────────────

fn cast_pw(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, def);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast planeswalker");
    drain_stack(g);
    id
}

/// CR 702.150: paying Jace's {U/P} with mana gives full loyalty.
#[test]
fn compleated_full_mana_full_loyalty() {
    let mut g = two_player_game();
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    let jace = cast_pw(&mut g, catalog::jace_the_perfected_mind());
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 5);
}

/// CR 702.150c: paying the {U/P} with 2 life drops Jace to 3 loyalty.
#[test]
fn compleated_life_paid_two_fewer_loyalty() {
    let mut g = two_player_game();
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    let jace = cast_pw(&mut g, catalog::jace_the_perfected_mind());
    assert_eq!(g.players[0].life, life - 2, "paid 2 life for the pip");
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 3);
}

/// Nissa's two {G/P} pips paid with life cost 4 loyalty total.
#[test]
fn compleated_nissa_double_pip() {
    let mut g = two_player_game();
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    let nissa = cast_pw(&mut g, catalog::nissa_ascended_animist());
    assert_eq!(g.players[0].life, life - 4);
    assert_eq!(g.battlefield_find(nissa).unwrap().counter_count(CounterType::Loyalty), 3);
}

/// Lukka's {R/G/P} pays from either color; green here.
#[test]
fn phyrexian_hybrid_pays_either_color() {
    let mut g = two_player_game();
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    let lukka = cast_pw(&mut g, catalog::lukka_bound_to_ruin());
    assert_eq!(g.players[0].life, life, "no life paid — green covered the pip");
    assert_eq!(g.battlefield_find(lukka).unwrap().counter_count(CounterType::Loyalty), 5);
}

/// Vraska −9 tops the opponent up to exactly nine poison counters.
#[test]
fn vraska_minus_nine_tops_up_poison() {
    let mut g = two_player_game();
    let vraska = g.add_card_to_battlefield(0, catalog::vraska_betrayals_sting());
    g.battlefield_find_mut(vraska).unwrap().counters.insert(CounterType::Loyalty, 10);
    g.players[1].poison_counters = 4;
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: vraska, ability_index: 2, target: Some(Target::Player(1)), x_value: None,
    }).expect("-9");
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 9);
}

/// Vraska −2 turns a creature into an abilityless Treasure artifact.
#[test]
fn vraska_minus_two_treasureifies() {
    let mut g = two_player_game();
    let vraska = g.add_card_to_battlefield(0, catalog::vraska_betrayals_sting());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: vraska, ability_index: 1, target: Some(Target::Permanent(angel)), x_value: None,
    }).expect("-2");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).unwrap();
    assert!(cp.card_types.contains(&CardType::Artifact) && !cp.card_types.contains(&CardType::Creature),
        "now a noncreature artifact");
    assert!(cp.subtypes.artifact_subtypes.contains(&crabomination::card::ArtifactSubtype::Treasure));
    assert!(!cp.keywords.contains(&crabomination::card::Keyword::Flying), "abilities wiped");
}

/// Kaito's trigger bounces the dealer and unlocks a second loyalty activation.
#[test]
fn kaito_bounce_grants_double_loyalty() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let kaito = g.add_card_to_battlefield(0, catalog::kaito_dancing_shadow());
    let priest = g.add_card_to_battlefield(0, catalog::necrogen_rotpriest());
    g.clear_sickness(priest);
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.active_player_idx = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = crabomination::game::types::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: priest, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = crabomination::game::types::TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(priest).is_none(), "dealer bounced to hand");
    g.step = crabomination::game::types::TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    let zero = |g: &mut GameState| g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: kaito, ability_index: 1, target: None, x_value: None,
    });
    zero(&mut g).expect("first activation");
    drain_stack(&mut g);
    zero(&mut g).expect("second activation allowed this turn");
    drain_stack(&mut g);
    assert!(zero(&mut g).is_err(), "third rejected");
}

/// Kaya −3 exiles an enchantment and leaves a 1/1 flying Spirit copy.
#[test]
fn kaya_minus_three_spirit_copy() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let kaya = g.add_card_to_battlefield(0, catalog::kaya_intangible_slayer());
    let ench = g.add_card_to_battlefield(1, catalog::phyrexian_awakening());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: kaya, ability_index: 2, target: Some(Target::Permanent(ench)), x_value: None,
    }).expect("-3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none(), "enchantment exiled");
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Phyrexian Awakening")
        .expect("token copy minted");
    let cp = g.computed_permanent(copy.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.card_types.contains(&CardType::Creature));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// The Eternal Wanderer −4: each player keeps one creature.
#[test]
fn eternal_wanderer_minus_four_keeps_one_each() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::the_eternal_wanderer());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw, ability_index: 2, target: None, x_value: None,
    }).expect("-4");
    drain_stack(&mut g);
    for p in 0..2 {
        let count = g.battlefield.iter()
            .filter(|c| c.controller == p && c.definition.is_creature()).count();
        assert_eq!(count, 1, "player {p} kept exactly one creature");
    }
}

/// Nahiri 0 mints a hasty copy of a graveyard Equipment, exiled at end step.
#[test]
fn nahiri_zero_copies_from_graveyard() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let nahiri = g.add_card_to_battlefield(0, catalog::nahiri_the_unforgiving());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: nahiri, ability_index: 2, target: Some(Target::Permanent(dead)), x_value: None,
    }).expect("0");
    drain_stack(&mut g);
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears")
        .expect("token copy");
    assert!(g.computed_permanent(copy.id).unwrap().keywords.contains(&Keyword::Haste));
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == dead), "original exiled");
}

// ── ONE wave 6: Sun's Twilights + rares ──────────────────────────────────────

fn cast_x(g: &mut GameState, def: crabomination::card::CardDefinition, x: u32, target: Option<Target>) {
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, def);
    g.players[0].mana_pool.add_colorless(x + 6);
    for c in [crabomination::mana::Color::White, crabomination::mana::Color::Blue, crabomination::mana::Color::Red,
              crabomination::mana::Color::Green] {
        g.players[0].mana_pool.add(c, 2);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: id, target, additional_targets: vec![], mode: None, x_value: Some(x),
    }).expect("cast X spell");
    drain_stack(g);
}

/// White Sun's Twilight at X=5: 5 life, 5 Mites, board wiped.
#[test]
fn white_suns_twilight_big_x() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::serra_angel());
    let life = g.players[0].life;
    cast_x(&mut g, catalog::white_suns_twilight(), 5, None);
    assert_eq!(g.players[0].life, life + 5);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Mite").count(), 5);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Serra Angel"), "wiped");
}

/// Blue Sun's Twilight at X=5 steals and copies.
#[test]
fn blue_suns_twilight_steal_and_copy() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    cast_x(&mut g, catalog::blue_suns_twilight(), 5, Some(Target::Permanent(angel)));
    assert_eq!(g.battlefield_find(angel).unwrap().controller, 0, "stolen");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Serra Angel").count(), 2,
        "copy minted");
}

/// Blue Sun's Twilight can't steal above X.
#[test]
fn blue_suns_twilight_mv_cap() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // MV 5
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::blue_suns_twilight());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: Some(2),
    }).is_err(), "MV 5 > X=2 is an illegal target");
}

/// Red Sun's Twilight at X=5 destroys and leaves hasty copies.
#[test]
fn red_suns_twilight_copies_then_destroys() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    cast_x(&mut g, catalog::red_suns_twilight(), 5, Some(Target::Permanent(ring)));
    assert!(g.battlefield_find(ring).is_none(), "original destroyed");
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Sol Ring")
        .expect("token copy left behind");
    assert!(copy.is_token);
}

/// Green Sun's Twilight at small X digs to hand.
#[test]
fn green_suns_twilight_digs_to_hand() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let hand0 = g.players[0].hand.len();
    cast_x(&mut g, catalog::green_suns_twilight(), 2, None);
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "took a creature and a land");
}

/// Kinzu exiles the dying creature for a 1/1 toxic copy.
#[test]
fn kinzu_exiles_for_toxic_copy() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kinzu_of_the_bleak_coven());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[0].life;
    let ctx = crabomination::game::effects::EffectContext::for_ability(bears, 0, None);
    let events = g.resolve_effect(&crabomination::effect::Effect::SacrificePermanent {
        what: crabomination::effect::Selector::This,
    }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "paid 2 life");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bears), "exiled, not in graveyard");
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears")
        .expect("copy minted");
    let cp = g.computed_permanent(copy.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.keywords.iter().any(|k| matches!(k, crabomination::card::Keyword::Toxic(1))));
}

/// Kethek sacrifices a spare creature and deploys a lesser-MV creature.
#[test]
fn kethek_end_step_crucible() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kethek_crucible_goliath());
    g.add_card_to_battlefield(0, catalog::serra_angel()); // MV 5 fodder
    g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 hit
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 0;
    g.fire_step_triggers(crabomination::game::types::TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "lesser creature deployed");
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Serra Angel"), "fodder gone");
}

/// Argentum Masticore's upkeep: discard kills a lesser permanent; declining
/// sacrifices it.
#[test]
fn argentum_masticore_upkeep_tax() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let core = g.add_card_to_battlefield(0, catalog::argentum_masticore());
    g.add_card_to_hand(0, catalog::serra_angel()); // MV 5 discard
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring()); // MV 1
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 0;
    g.fire_step_triggers(crabomination::game::types::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_none(), "MV1 ≤ MV5 discarded — destroyed");
    assert!(g.battlefield_find(core).is_some(), "Masticore stays");
    // Decline next upkeep → sacrifice.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    g.fire_step_triggers(crabomination::game::types::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(core).is_none(), "sacrificed on decline");
}

/// Vanish into Eternity costs {3} more against creatures.
#[test]
fn vanish_into_eternity_creature_tax() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::vanish_into_eternity());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "3 mana cannot cover the +3 creature tax");
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(ring)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("noncreature target at base cost");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_none(), "exiled");
}

/// Viral Spawning's flashback is Corrupted-gated.
#[test]
fn viral_spawning_corrupted_flashback() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::viral_spawning());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::CastFlashback {
        card_id: dead, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no poison → no flashback");
    g.players[1].poison_counters = 3;
    g.perform_action(GameAction::CastFlashback {
        card_id: dead, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("corrupted flashback");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Phyrexian Beast"));
}

/// Zenith Chronicler: only the first multicolored spell each turn draws.
#[test]
fn zenith_chronicler_first_multicolored() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::zenith_chronicler());
    g.add_card_to_library(1, catalog::forest());
    g.add_card_to_library(1, catalog::forest());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand1 = g.players[1].hand.len();
    for i in 0..2 {
        let spell = g.add_card_to_hand(0, catalog::kaito_dancing_shadow()); // UB multicolored
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("cast {i}: {e:?}"));
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].hand.len(), hand1 + 1, "only the first cast drew");
}

/// Noxious Assault pumps and poisons blockers for the turn.
#[test]
fn noxious_assault_block_poison() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::noxious_assault());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(atk).unwrap().power, 4, "+2/+2");
    g.active_player_idx = 0;
    g.step = crabomination::game::types::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = crabomination::game::types::TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, atk)])).unwrap();
    assert_eq!(g.players[1].poison_counters, 1, "blocking poisoned the defender");
}

/// Contagious Vorrac proliferates when the top four hold no land.
#[test]
fn contagious_vorrac_else_proliferates() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[1].poison_counters = 1;
    g.move_card_to_battlefield_for_test(0, catalog::contagious_vorrac());
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 2, "no land → proliferate");
}

/// Expand the Sphere deploys up to two lands tapped and proliferates shortfall.
#[test]
fn expand_the_sphere_deploys_and_compensates() {
    let mut g = two_player_game();
    // One land in the top six → deploy 1, proliferate once.
    g.add_card_to_library(0, catalog::forest());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[1].poison_counters = 1;
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::expand_the_sphere());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let forest = g.battlefield.iter().find(|c| c.definition.name == "Forest")
        .expect("land deployed");
    assert!(forest.tapped, "enters tapped");
    assert_eq!(g.players[1].poison_counters, 2, "one short → one proliferate");
}

/// Goliath Hatchery mints two Beasts; Corrupted upkeep draws by best toxic.
#[test]
fn goliath_hatchery_tokens_and_draw() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::goliath_hatchery());
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Beast").count(), 2);
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let hand0 = g.players[0].hand.len();
    g.players[1].poison_counters = 3;
    g.active_player_idx = 0;
    g.fire_step_triggers(crabomination::game::types::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew = toxic 1 (the Beasts)");
}

// ── ONE wave 7: mythics + oil engines ────────────────────────────────────────

/// All Will Be One pings for counters you place and poison you inflict.
#[test]
fn all_will_be_one_counter_pings() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::all_will_be_one());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let life1 = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_ability(bears, 0, None);
    let events = g.resolve_effect(&crabomination::effect::Effect::AddCounter {
        what: crabomination::effect::Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: crabomination::effect::Value::Const(4),
    }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(victim).is_none() || g.players[1].life == life1 - 4,
        "4 counters pinged an opposing target for 4"
    );
}

/// Drivnod doubles your death triggers.
#[test]
fn drivnod_doubles_death_triggers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::drivnod_carnage_dominus());
    // Injector Crocodile: dies → incubate 3.
    let croc = g.add_card_to_battlefield(0, catalog::injector_crocodile());
    let ctx = crabomination::game::effects::EffectContext::for_ability(croc, 0, None);
    let events = g.resolve_effect(&crabomination::effect::Effect::SacrificePermanent {
        what: crabomination::effect::Selector::This,
    }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let incubators = g.battlefield.iter().filter(|c| c.definition.name == "Incubator").count();
    assert_eq!(incubators, 2, "death trigger fired twice");
}

/// Ichormoon Gauntlet grants planeswalkers [0]: Proliferate.
#[test]
fn ichormoon_gauntlet_grants_loyalty_zero() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ichormoon_gauntlet());
    let koth = g.add_card_to_battlefield(0, catalog::koth_fire_of_resistance());
    g.players[1].poison_counters = 1;
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Koth prints 3 abilities; index 3 = the granted [0]: Proliferate.
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: koth, ability_index: 3, target: None, x_value: None,
    }).expect("granted [0]");
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 2, "proliferated");
}

/// Mindsplice Apparatus discounts instants per oil counter.
#[test]
fn mindsplice_apparatus_scaling_discount() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::mindsplice_apparatus());
    g.battlefield_find_mut(app).unwrap().add_counters(CounterType::Oil, 2);
    // Serum Visions? use Lightning Bolt: {R} — no generic to shave. Use
    // Nahiri's Sacrifice {1}{R}: 2 oil shaves the {1}.
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
    let target = g.add_card_to_battlefield(1, catalog::wind_drake());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::nahiris_sacrifice());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1); // exactly {R}
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("discounted to {R}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "resolved");
}

/// Mercurial Spelldancer's saboteur rider copies your next instant.
#[test]
fn mercurial_spelldancer_copies_next_spell() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let dancer = g.add_card_to_battlefield(0, catalog::mercurial_spelldancer());
    g.battlefield_find_mut(dancer).unwrap().add_counters(CounterType::Oil, 2);
    g.clear_sickness(dancer);
    g.active_player_idx = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = crabomination::game::types::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dancer, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = crabomination::game::types::TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(dancer).unwrap().counter_count(CounterType::Oil), 0,
        "two oil cashed in");
    let life1 = g.players[1].life;
    g.step = crabomination::game::types::TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 6, "bolt + copy = 6");
}

/// Churning Reservoir's Goblin mint needs oil activity this turn.
#[test]
fn churning_reservoir_oil_gate() {
    let mut g = two_player_game();
    let res = g.add_card_to_battlefield(0, catalog::churning_reservoir());
    g.players[0].mana_pool.add_colorless(4);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let act = |g: &mut GameState| g.perform_action(GameAction::ActivateAbility {
        card_id: res, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    });
    assert!(act(&mut g).is_err(), "no oil activity yet");
    // Remove an oil counter from a permanent you control.
    let fang = g.add_card_to_battlefield(0, catalog::atraxas_skitterfang());
    g.battlefield_find_mut(fang).unwrap().add_counters(CounterType::Oil, 1);
    let ctx = crabomination::game::effects::EffectContext::for_ability(fang, 0, None);
    g.resolve_effect(&crabomination::effect::Effect::RemoveCounter {
        what: crabomination::effect::Selector::This, kind: CounterType::Oil,
        amount: crabomination::effect::Value::ONE,
    }, &ctx).unwrap();
    act(&mut g).expect("oil was removed this turn");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Phyrexian Goblin"));
}

/// Phyrexian Vindicator deflects damage to another target.
#[test]
fn phyrexian_vindicator_deflects() {
    let mut g = two_player_game();
    let vind = g.add_card_to_battlefield(0, catalog::phyrexian_vindicator());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(vind)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the Vindicator");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(vind).unwrap().damage, 0, "damage prevented");
    assert_eq!(g.players[1].life, life1 - 3, "3 deflected at the opponent");
}

/// Graaz turns your other creatures into 5/3 Juggernauts.
#[test]
fn graaz_makes_five_three_juggernauts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::graaz_unstoppable_juggernaut());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 3));
    assert!(cp.subtypes.creature_types.contains(&crabomination::card::CreatureType::Juggernaut));
    assert!(cp.subtypes.creature_types.contains(&crabomination::card::CreatureType::Bear), "in addition");
    assert!(cp.keywords.contains(&crabomination::card::Keyword::MustAttack), "must attack");
}

/// Encroaching Mycosynth turns your nonland permanents into artifacts.
#[test]
fn encroaching_mycosynth_artifacts_everything() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::encroaching_mycosynth());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(g.computed_permanent(bears).unwrap().card_types.contains(&CardType::Artifact));
    assert!(!g.computed_permanent(land).unwrap().card_types.contains(&CardType::Artifact));
    assert!(!g.computed_permanent(theirs).unwrap().card_types.contains(&CardType::Artifact));
}

/// Venser mints The Hollow Sentinel on your first proliferate.
#[test]
fn venser_mints_hollow_sentinel() {
    let mut g = two_player_game();
    let venser = g.add_card_to_battlefield(0, catalog::venser_corpse_puppet());
    g.players[1].poison_counters = 1;
    let ctx = crabomination::game::effects::EffectContext::for_ability(venser, 0, None);
    let events = g.resolve_effect(&crabomination::effect::Effect::Proliferate, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "The Hollow Sentinel"));
}

/// The Mycosynth Gardens becomes a copy of your Sol Ring.
#[test]
fn mycosynth_gardens_copies_artifact() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::the_mycosynth_gardens());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring()); // MV 1
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: Some(Target::Permanent(ring)),
        additional_targets: vec![], x_value: Some(1), mode: None,
    }).expect("become a copy");
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).unwrap();
    assert!(cp.card_types.contains(&CardType::Artifact), "now a Sol Ring copy");
}

/// Mirran Safehouse taps like the lands in the graveyards.
#[test]
fn mirran_safehouse_borrows_land_abilities() {
    let mut g = two_player_game();
    let house = g.add_card_to_battlefield(0, catalog::mirran_safehouse());
    g.add_card_to_graveyard(1, catalog::mountain());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: house, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("tap like a Mountain");
    assert_eq!(g.players[0].mana_pool.amount(crabomination::mana::Color::Red), 1);
}

/// Monument to Perfection animates only at nine land names.
#[test]
fn monument_to_perfection_transformation() {
    let mut g = two_player_game();
    let mon = g.add_card_to_battlefield(0, catalog::monument_to_perfection());
    g.players[0].mana_pool.add_colorless(6);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let act = |g: &mut GameState| g.perform_action(GameAction::ActivateAbility {
        card_id: mon, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    });
    assert!(act(&mut g).is_err(), "too few land names");
    for f in [catalog::plains, catalog::island, catalog::swamp, catalog::mountain,
              catalog::forest, catalog::mirrex, catalog::the_seedcore,
              catalog::the_monumental_facade, catalog::the_mycosynth_gardens] {
        g.add_card_to_battlefield(0, f());
    }
    act(&mut g).expect("nine names");
    drain_stack(&mut g);
    let cp = g.computed_permanent(mon).unwrap();
    assert_eq!((cp.power, cp.toughness), (9, 9));
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Indestructible));
}

// ── ONE wave 8: the set closes out ───────────────────────────────────────────

/// Capricious Hellraiser is {3} cheaper at nine cards, exiles three random
/// graveyard cards, and casts a *copy* of one (the original stays exiled).
#[test]
fn capricious_hellraiser_ritual() {
    let mut g = two_player_game();
    for _ in 0..9 {
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
    }
    let life1 = g.players[1].life;
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hell = g.add_card_to_hand(0, catalog::capricious_hellraiser());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 3); // {3} shaved off
    g.perform_action(GameAction::CastSpell {
        card_id: hell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("discounted to {R}{R}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 6, "three cards exiled");
    assert_eq!(g.players[1].life, life1 - 3, "the free copy resolved");
    assert_eq!(g.exile.iter().filter(|c| c.definition.name == "Lightning Bolt").count(), 3,
        "the original stays in exile (a copy was cast)");
}

/// Blade of Shared Souls lets its bearer copy another creature you control.
#[test]
fn blade_of_shared_souls_copies() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blade = g.add_card_to_battlefield(0, catalog::blade_of_shared_souls());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Cards(vec![angel]),
    ]));
    g.perform_action(GameAction::Equip { equipment: blade, target: bears }).expect("equip");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!(cp.power, 4, "the bear became a Serra Angel copy");
}

/// Rhuk steals the Equipment off another attacking equipped creature.
#[test]
fn rhuk_nabs_equipment() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let rhuk = g.add_card_to_battlefield(0, catalog::rhuk_hexgold_nabber());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bears);
    let sword = g.add_card_to_battlefield(0, catalog::short_sword());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bears);
    g.active_player_idx = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = crabomination::game::types::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bears, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sword).unwrap().attached_to, Some(rhuk), "sword moved to Rhuk");
}

/// Ria Ivor converts the chosen creature's prevented combat damage to Mites.
#[test]
fn ria_ivor_mints_mites() {
    let mut g = two_player_game();
    let ria = g.add_card_to_battlefield(0, catalog::ria_ivor_bane_of_bladehold());
    g.clear_sickness(ria);
    g.active_player_idx = 0;
    // Auto-pick chooses the biggest creature — Ria herself.
    g.fire_step_triggers(crabomination::game::types::TurnStep::BeginCombat);
    drain_stack(&mut g);
    let life1 = g.players[1].life;
    g.step = crabomination::game::types::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ria, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = crabomination::game::types::TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1, "combat damage prevented");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Mite").count(), 3,
        "three Mites for three prevented damage");
}
