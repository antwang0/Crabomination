//! Phyrexia: All Will Be One — Incubate (CR 701.53). The Incubator token enters
//! with N +1/+1 counters; `{2}: Transform` flips it to a 0/0 Phyrexian creature
//! (so it becomes N/N).

use crate::card::{CardType, CounterType};
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};

/// Resolve `effect` as though `player` were its controller.
fn resolve_for(g: &mut GameState, player: usize, effect: crate::effect::Effect) {
    let src = g.add_card_to_battlefield(player, catalog::grizzly_bears());
    let ctx = crate::game::effects::EffectContext::for_ability(src, player, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(g);
}

/// Incubate 3 mints an Incubator with three +1/+1 counters; transforming it
/// yields a 3/3 Phyrexian artifact creature (counters persist, CR 712).
#[test]
fn incubate_then_transform_to_n_over_n() {
    let mut g = two_player_game();
    resolve_for(&mut g, 0, crate::effect::Effect::Incubate {
        who: crate::effect::PlayerRef::You,
        amount: crate::effect::Value::Const(3),
    });
    let inc = g.battlefield.iter().find(|c| c.definition.name == "Incubator").expect("Incubator minted");
    let inc_id = inc.id;
    assert_eq!(inc.counter_count(CounterType::PlusOnePlusOne), 3, "three +1/+1 counters");
    let cp = g.computed_permanent(inc_id).unwrap();
    assert!(cp.card_types.contains(&CardType::Artifact) && !cp.card_types.contains(&CardType::Creature),
        "front is a noncreature artifact");
    // {2}: Transform.
    g.players[0].mana_pool.add_colorless(2);
    g.step = crate::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: inc_id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
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
    let ctx = crate::game::effects::EffectContext::for_ability(croc, 0, None);
    g.resolve_effect(
        &crate::effect::Effect::SacrificePermanent { what: crate::effect::Selector::Target(0) },
        &crate::game::effects::EffectContext { targets: vec![crate::game::types::Target::Permanent(croc)], ..ctx },
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
    use crate::card::Keyword;
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
    g.dispatch_triggers_for_events(&[crate::game::GameEvent::PermanentEntered { card_id: croc }]);
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
    g.step = crate::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: hunt, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
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
    use crate::card::Keyword;
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
    use crate::game::types::{Attack, AttackTarget};
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
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    // No poison → the {W} Corrupted ability (index 1) is rejected.
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: dancer, ability_index: 1,
        target: Some(Target::Permanent(foe)), additional_targets: vec![], x_value: None,
    });
    assert!(err.is_err(), "Corrupted ability blocked below 3 poison");
    // Grant Corrupted and retry — now it taps the target.
    g.players[1].poison_counters = 3;
    g.perform_action(GameAction::ActivateAbility {
        card_id: dancer, ability_index: 1,
        target: Some(Target::Permanent(foe)), additional_targets: vec![], x_value: None,
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
        g.players[0].mana_pool.add(crate::mana::Color::White, 1);
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
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
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
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
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life0 = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: glad, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
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
    use crate::game::types::{Attack, AttackTarget};
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
    use crate::card::Keyword;
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
    use crate::game::types::{Attack, AttackTarget};
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
    let ctx = crate::game::effects::EffectContext::for_ability(watcher, 0, None);
    g.resolve_effect(
        &crate::effect::Effect::SacrificePermanent { what: crate::effect::Selector::Target(0) },
        &crate::game::effects::EffectContext { targets: vec![Target::Permanent(watcher)], ..ctx },
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
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Chimney Rabble");
    drain_stack(&mut g);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&crate::card::Keyword::Haste));
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
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
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
    use crate::card::Keyword;
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
        card_id: cent, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Cutthroat Centurion");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.computed_permanent(cent).unwrap().power, 4, "2/2 + 2/2 = 4/4");
}

/// Shrapnel Slinger sacrifices a creature on ETB to destroy an opponent artifact.
#[test]
fn shrapnel_slinger_sac_destroys_artifact() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::millstone()); // opponent artifact
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let slinger = g.add_card_to_battlefield(0, catalog::shrapnel_slinger());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let eff = catalog::shrapnel_slinger().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(slinger, 0, None, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the creature");
    assert!(g.battlefield_find(art).is_none(), "opponent's artifact destroyed");
}

// ── Modern_decks ONE wave (toxic / corrupted / oil / for-Mirrodin payoffs) ──

use crate::card::Keyword;
use crate::effect::{Effect, Value};
use crate::game::types::Target;

/// Resolve a targeted `effect` for `player` against `targets`.
fn resolve_targeted(g: &mut GameState, player: usize, effect: Effect, targets: Vec<Target>) {
    let src = g.add_card_to_battlefield(player, catalog::grizzly_bears());
    let base = crate::game::effects::EffectContext::for_ability(src, player, None);
    let ctx = crate::game::effects::EffectContext { targets, ..base };
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(g);
}

fn add_oil(g: &mut GameState, id: crate::card::CardId, n: u32) {
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
    let token = crate::card::TokenDefinition {
        name: "Test Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Red],
        ..Default::default()
    };
    resolve_for(&mut g, 0, Effect::CreateToken {
        who: crate::effect::PlayerRef::You,
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
    g.perform_action(GameAction::DeclareAttackers(vec![crate::game::types::Attack {
        attacker: cackler, target: crate::game::types::AttackTarget::Player(1),
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
    g.dispatch_triggers_for_events(&[crate::game::GameEvent::PermanentEntered { card_id: big }]);
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
    let ctx = crate::game::effects::EffectContext::for_ability(cc, 0, None);
    g.resolve_effect(
        &Effect::SacrificePermanent { what: crate::effect::Selector::Target(0) },
        &crate::game::effects::EffectContext { targets: vec![Target::Permanent(cc)], ..ctx },
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
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.step = crate::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: z, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
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
    g.step = crate::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: migloz, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
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
    g.dispatch_triggers_for_events(&[crate::game::GameEvent::CreatureDied { card_id: fodder }]);
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
    g.perform_action(GameAction::DeclareAttackers(vec![crate::game::types::Attack {
        attacker: chorus, target: crate::game::types::AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(chorus).unwrap().power, 2, "1/1 toxic attacker gets +1/+1");
}

/// Ichor Drinker incubates 2 from the graveyard (exile-self cost).
#[test]
fn ichor_drinker_gy_incubates() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::ichor_drinker());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.step = crate::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
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
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.step = crate::game::types::TurnStep::PreCombatMain;
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
    g.players[1].mana_pool.add(crate::mana::Color::White, 2);
    g.step = crate::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts angel");
    // Bring the Ending checks the caster's (player 0's) Corrupted — an opponent
    // with three or more poison — so poison the opponent (player 1).
    g.players[1].poison_counters = 3;
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let base = crate::game::effects::EffectContext::for_ability(src, 0, None);
    let ctx = crate::game::effects::EffectContext { targets: vec![Target::Permanent(angel)], ..base };
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
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.step = crate::game::types::TurnStep::PreCombatMain;
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
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    g.clear_sickness(acro);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crate::game::types::Attack {
        attacker: acro, target: crate::game::types::AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert!(g.computed_permanent(acro).unwrap().keywords.contains(&Keyword::Flying), "gained flying via sacrifice");
}
