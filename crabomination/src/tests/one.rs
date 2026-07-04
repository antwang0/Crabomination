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
    // Not corrupted → the ETB's intervening-if fails, no destroy.
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
    let syphoner = catalog::pestilent_syphoner();
    assert!(syphoner.keywords.contains(&Keyword::Flying) && syphoner.keywords.contains(&Keyword::Toxic(1)));
    let basilisk = catalog::ichorspit_basilisk();
    assert_eq!((basilisk.power, basilisk.toughness), (1, 3));
    assert!(basilisk.keywords.contains(&Keyword::Deathtouch) && basilisk.keywords.contains(&Keyword::Toxic(1)));
}
