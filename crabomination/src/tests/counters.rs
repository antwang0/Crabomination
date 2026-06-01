//! Functionality tests for the +1/+1-counter keyword mechanics added on
//! the `claude/modern_decks` branch: Modular (CR 702.43), Graft
//! (CR 702.57), and Melee (CR 702.121).

use crate::card::{CardType, CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::effect::Effect;
use crate::game::types::{AttackTarget, Target};
use crate::game::*;
use crate::game::{cast, drain_stack, two_player_game};
use crate::mana::Color;
use crate::TurnStep;

/// Bolt a creature `seat` controls so it dies, returning after resolution.
fn bolt(g: &mut GameState, seat: usize, target: CardId) {
    let b = g.add_card_to_hand(seat, catalog::lightning_bolt());
    g.players[seat].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: b, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(g);
}

// ── Modular (CR 702.43) ──────────────────────────────────────────────────

#[test]
fn arcbound_worker_enters_with_one_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::arcbound_worker());
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    let w = g.battlefield_find(id).expect("Worker resolved");
    assert_eq!(w.counter_count(CounterType::PlusOnePlusOne), 1,
        "Modular 1 enters with one +1/+1 counter");
    let view = g.compute_battlefield().into_iter().find(|c| c.id == id).unwrap();
    assert_eq!((view.power, view.toughness), (1, 1), "0/0 base + one counter = 1/1");
}

#[test]
fn modular_moves_counters_to_artifact_creature_on_death() {
    let mut g = two_player_game();
    // Recipient: an artifact creature already on the battlefield.
    let thopter = g.add_card_to_battlefield(0, catalog::ornithopter());
    let worker = g.add_card_to_hand(0, catalog::arcbound_worker());
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, worker);
    // Accept the "you may move counters" optional trigger.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    bolt(&mut g, 0, worker);
    let recip = g.battlefield_find(thopter).expect("Ornithopter survives");
    assert_eq!(recip.counter_count(CounterType::PlusOnePlusOne), 1,
        "Worker's counter migrates to the artifact creature on death");
}

#[test]
fn arcbound_ravager_sacs_artifact_for_a_counter() {
    let mut g = two_player_game();
    let ravager = g.add_card_to_battlefield(0, catalog::arcbound_ravager());
    // add_card_to_battlefield bypasses ETB, so stamp the Modular counter.
    g.battlefield_find_mut(ravager).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let fodder = g.add_card_to_battlefield(0, catalog::ornithopter());
    g.clear_sickness(ravager);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ravager, ability_index: 0, target: None, x_value: None,
    }).expect("Ravager ability activatable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ravager).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "sacrificing an artifact grows the Ravager");
    assert!(g.battlefield_find(fodder).is_none(), "the sacrificed artifact is gone");
}

// ── Graft (CR 702.57) ─────────────────────────────────────────────────────

#[test]
fn aquastrand_spider_enters_with_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::aquastrand_spider());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    let view = g.compute_battlefield().into_iter().find(|c| c.id == id).unwrap();
    assert_eq!((view.power, view.toughness), (2, 2), "Graft 2 → 0/0 + two counters");
    assert!(view.keywords.contains(&Keyword::Reach));
}

#[test]
fn graft_moves_a_counter_to_an_entering_creature() {
    let mut g = two_player_game();
    let spider = g.add_card_to_battlefield(0, catalog::aquastrand_spider());
    g.battlefield_find_mut(spider).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    // Accept the optional graft move when the new creature enters.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bear);
    assert_eq!(g.battlefield_find(spider).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "Spider gives up one of its counters");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "entering creature gains the moved counter");
}

#[test]
fn cytoplast_root_kin_pumps_creatures_with_counters() {
    let mut g = two_player_game();
    // A creature that already has a +1/+1 counter.
    let spider = g.add_card_to_battlefield(0, catalog::aquastrand_spider());
    g.battlefield_find_mut(spider).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let root_kin = g.add_card_to_hand(0, catalog::cytoplast_root_kin());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, root_kin);
    assert_eq!(g.battlefield_find(spider).unwrap().counter_count(CounterType::PlusOnePlusOne), 3,
        "ETB adds a counter to each other creature you control that has one");
    assert_eq!(g.battlefield_find(root_kin).unwrap().counter_count(CounterType::PlusOnePlusOne), 4,
        "Root-Kin itself entered with Graft 4");
}

// ── Melee (CR 702.121) ─────────────────────────────────────────────────────

#[test]
fn melee_pumps_the_attacker() {
    use crate::card::{CardDefinition, Subtypes};
    let mut g = two_player_game();
    // Inline 2/2 with Melee — exercises the shortcut directly.
    let def = CardDefinition {
        name: "Melee Tester",
        cost: crate::mana::cost(&[crate::mana::generic(2)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: 2,
        toughness: 2,
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::melee()],
        ..Default::default()
    };
    let id = g.add_card_to_battlefield(0, def);
    g.clear_sickness(id);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("declare attacker");
    drain_stack(&mut g);
    let view = g.compute_battlefield().into_iter().find(|c| c.id == id).unwrap();
    assert_eq!((view.power, view.toughness), (3, 3), "Melee grants +1/+1 on attack");
}

// ── Renown (CR 702.111) ────────────────────────────────────────────────────

#[test]
fn renown_adds_counters_on_first_combat_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::topan_freeblade());
    g.clear_sickness(id);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    // Run combat through to damage.
    for _ in 0..12 {
        if g.battlefield_find(id).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0) > 0 {
            break;
        }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "Renown 1 fires when it connects, becoming renowned");
    assert!(g.players[1].life < 20, "defender took combat damage");
}

// ── Outlast (CR 702.97) ────────────────────────────────────────────────────

#[test]
fn outlast_adds_a_counter_at_sorcery_speed() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ainok_bond_kin());
    g.clear_sickness(id);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("Outlast activatable at sorcery speed");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.tapped, "Outlast taps the creature");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "Outlast adds a +1/+1 counter");
    // The counter-anthem now grants first strike to this creature.
    let view = g.compute_battlefield().into_iter().find(|v| v.id == id).unwrap();
    assert!(view.keywords.contains(&Keyword::FirstStrike),
        "creatures with a +1/+1 counter gain first strike");
}

#[test]
fn outlast_anthem_only_buffs_creatures_with_counters() {
    let mut g = two_player_game();
    let falconer = g.add_card_to_battlefield(0, catalog::abzan_falconer());
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Stamp a counter on the bear only.
    g.battlefield_find_mut(plain).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let computed = g.compute_battlefield();
    let bear_view = computed.iter().find(|v| v.id == plain).unwrap();
    let falc_view = computed.iter().find(|v| v.id == falconer).unwrap();
    assert!(bear_view.keywords.contains(&Keyword::Flying), "countered creature flies");
    assert!(!falc_view.keywords.contains(&Keyword::Flying),
        "the Falconer itself has no counter yet, so it doesn't fly");
}

// ── Remaining bodies (stats + keyword coverage) ───────────────────────────

#[test]
fn arcbound_stinger_is_a_flying_modular_one_drop_body() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::arcbound_stinger());
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let view = g.compute_battlefield().into_iter().find(|c| c.id == id).unwrap();
    assert_eq!((view.power, view.toughness), (1, 1), "Modular 1 → 0/0 + one counter");
    assert!(view.keywords.contains(&Keyword::Flying));
}

#[test]
fn plaxcaster_frogling_enters_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::plaxcaster_frogling());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let view = g.compute_battlefield().into_iter().find(|c| c.id == id).unwrap();
    assert_eq!((view.power, view.toughness), (3, 3), "Graft 3 → 0/0 + three counters");
}

#[test]
fn stalwart_aven_and_skyraker_giant_renown_on_connect() {
    for (factory, renown, base, kw) in [
        (catalog::stalwart_aven as fn() -> _, 1, (2, 2), Keyword::Flying),
        (catalog::skyraker_giant as fn() -> _, 4, (4, 4), Keyword::Reach),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, factory());
        g.clear_sickness(id);
        let view = g.compute_battlefield().into_iter().find(|c| c.id == id).unwrap();
        assert_eq!((view.power, view.toughness), base);
        assert!(view.keywords.contains(&kw));
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: id, target: AttackTarget::Player(1),
        }])).expect("attack");
        for _ in 0..12 {
            if g.battlefield_find(id).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0) > 0 {
                break;
            }
            let _ = g.perform_action(GameAction::PassPriority);
            drain_stack(&mut g);
        }
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), renown,
            "renown count lands on first connection");
    }
}

#[test]
fn tuskguard_and_mer_ek_anthems_buff_countered_creatures() {
    for (factory, kw) in [
        (catalog::tuskguard_captain as fn() -> _, Keyword::Trample),
        (catalog::mer_ek_nightblade as fn() -> _, Keyword::Deathtouch),
    ] {
        let mut g = two_player_game();
        let lord = g.add_card_to_battlefield(0, factory());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        let computed = g.compute_battlefield();
        assert!(computed.iter().find(|v| v.id == bear).unwrap().keywords.contains(&kw),
            "countered creature gains the anthem keyword");
        assert!(!computed.iter().find(|v| v.id == lord).unwrap().keywords.contains(&kw),
            "the lord without a counter does not");
    }
}

// ── More Modular / Graft / Renown bodies ──────────────────────────────────

#[test]
fn arcbound_hybrid_and_bruiser_enter_with_counters() {
    for (factory, mana, pt, kw) in [
        (catalog::arcbound_hybrid as fn() -> _, 3, (2, 2), Some(Keyword::Haste)),
        (catalog::arcbound_bruiser as fn() -> _, 4, (3, 3), None),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, factory());
        g.players[0].mana_pool.add_colorless(mana);
        cast(&mut g, id);
        let view = g.compute_battlefield().into_iter().find(|c| c.id == id).unwrap();
        assert_eq!((view.power, view.toughness), pt, "Modular body enters with its counters");
        if let Some(k) = kw {
            assert!(view.keywords.contains(&k));
        }
    }
}

#[test]
fn simic_initiate_enters_as_a_one_one_graft() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::simic_initiate());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, id);
    let view = g.compute_battlefield().into_iter().find(|c| c.id == id).unwrap();
    assert_eq!((view.power, view.toughness), (1, 1), "Graft 1 → 1/1");
}

#[test]
fn vigean_graftmage_untaps_a_countered_creature() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::vigean_graftmage());
    g.clear_sickness(mage);
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(target).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.battlefield_find_mut(target).unwrap().tapped = true;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Permanent(target)), x_value: None,
    }).expect("Vigean ability activatable");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(target).unwrap().tapped, "untaps the countered creature");
}

#[test]
fn helium_squirter_grants_flying_to_a_countered_creature() {
    let mut g = two_player_game();
    let squirter = g.add_card_to_battlefield(0, catalog::helium_squirter());
    g.clear_sickness(squirter);
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(target).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: squirter, ability_index: 0, target: Some(Target::Permanent(target)), x_value: None,
    }).expect("Helium Squirter ability activatable");
    drain_stack(&mut g);
    let view = g.compute_battlefield().into_iter().find(|c| c.id == target).unwrap();
    assert!(view.keywords.contains(&Keyword::Flying), "countered creature gains flying");
}

#[test]
fn knight_and_consuls_lieutenant_are_renown_one_drops() {
    for (factory, pt, kw) in [
        (catalog::knight_of_the_pilgrims_road as fn() -> _, (2, 2), None),
        (catalog::consuls_lieutenant as fn() -> _, (2, 1), Some(Keyword::FirstStrike)),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, factory());
        g.clear_sickness(id);
        let view = g.compute_battlefield().into_iter().find(|c| c.id == id).unwrap();
        assert_eq!((view.power, view.toughness), pt);
        if let Some(k) = kw { assert!(view.keywords.contains(&k)); }
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: id, target: AttackTarget::Player(1),
        }])).expect("attack");
        for _ in 0..12 {
            if g.battlefield_find(id).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0) > 0 { break; }
            let _ = g.perform_action(GameAction::PassPriority);
            drain_stack(&mut g);
        }
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
            "Renown 1 on connect");
    }
}
