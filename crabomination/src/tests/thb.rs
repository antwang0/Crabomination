//! Functionality tests for the Theros Beyond Death batch
//! (`catalog::sets::thb`).

use crate::card::CounterType;
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use crate::TurnStep;

/// Heliod's Intervention mode 0 destroys X chosen artifact/enchantment targets.
#[test]
fn heliods_intervention_destroys_x_targets() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::mind_stone());
    let b = g.add_card_to_battlefield(1, catalog::mind_stone());
    let spell = g.add_card_to_hand(0, catalog::heliods_intervention());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: Some(0),
        x_value: Some(2),
    })
    .expect("cast {X=2}{W}{W} destroying two artifacts");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none(), "first artifact destroyed");
    assert!(g.battlefield_find(b).is_none(), "second artifact destroyed");
}

/// Heliod's Intervention mode 1: target player gains twice X life.
#[test]
fn heliods_intervention_gains_twice_x() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::heliods_intervention());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: Some(3),
    })
    .expect("cast for X=3 gaining 6");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 6, "gained twice X life");
}

/// Shark Typhoon mints an X/X Shark when you cast a noncreature spell
/// (X = that spell's mana value).
#[test]
fn shark_typhoon_mints_shark_on_noncreature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::shark_typhoon());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Bolt");
    drain_stack(&mut g);
    let shark = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Shark")
        .expect("Shark token minted");
    assert_eq!((shark.power(), shark.toughness()), (1, 1), "X = Bolt's mana value");
}

/// Shark Typhoon does not trigger on creature spells.
#[test]
fn shark_typhoon_ignores_creature_casts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::shark_typhoon());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Bears");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Shark"));
}

/// Cycling Shark Typhoon for {X}{1}{U} mints an X/X Shark and draws.
#[test]
fn shark_typhoon_cycles_for_an_x_x_shark() {
    let mut g = two_player_game();
    let st = g.add_card_to_hand(0, catalog::shark_typhoon());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::Cycle { card_id: st, x_value: Some(3) })
        .expect("cycle for {3}{1}{U}");
    drain_stack(&mut g);
    let shark = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Shark")
        .expect("Shark token minted on cycle");
    assert_eq!((shark.power(), shark.toughness()), (3, 3), "X = 3 paid to the cycle");
    assert_eq!(g.players[0].hand.len(), hand, "discarded Typhoon, drew a card");
}

/// Nyxbloom Ancient triples a tapped-for-mana ability; composes with Mana
/// Reflection multiplicatively (CR 614.5).
#[test]
fn nyxbloom_ancient_triples_tapped_mana() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nyxbloom_ancient());
    let dork = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.clear_sickness(dork);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dork, ability_index: 0, target: None, x_value: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3, "G → GGG");

    g.add_card_to_battlefield(0, catalog::mana_reflection());
    let dork2 = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.clear_sickness(dork2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dork2, ability_index: 0, target: None, x_value: None,
    })
    .expect("tap for mana");
    assert_eq!(
        g.players[0].mana_pool.amount(Color::Green),
        3 + 6,
        "doubler × tripler = 6×"
    );
}

/// Polukranos enters with six +1/+1 counters from a regular cast.
#[test]
fn polukranos_enters_with_six_counters() {
    let mut g = two_player_game();
    let p = g.add_card_to_hand(0, catalog::polukranos_unchained());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: p, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Polukranos");
    drain_stack(&mut g);
    let c = g.battlefield_find(p).expect("on battlefield");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 6);
}

/// Polukranos escapes with twelve counters instead (CR 614.12 "escapes with").
#[test]
fn polukranos_escapes_with_twelve_counters() {
    let mut g = two_player_game();
    let p = g.add_card_to_graveyard(0, catalog::polukranos_unchained());
    let fodder: Vec<_> =
        (0..6).map(|_| g.add_card_to_graveyard(0, catalog::lightning_bolt())).collect();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastEscape {
        card_id: p,
        exile_cards: fodder,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("escape for {4}{B}{G} + exile six");
    drain_stack(&mut g);
    let c = g.battlefield_find(p).expect("on battlefield");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 12, "escaped → twelve");
}

/// Damage to Polukranos is prevented by removing that many +1/+1 counters.
#[test]
fn polukranos_prevents_damage_by_removing_counters() {
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::polukranos_unchained());
    g.battlefield_find_mut(p).unwrap().add_counters(CounterType::PlusOnePlusOne, 6);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(p)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt Polukranos");
    drain_stack(&mut g);
    let c = g.battlefield_find(p).expect("still alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 3, "3 counters removed");
    assert_eq!(c.damage, 0, "damage prevented");
}

/// Polukranos's {1}{B}{G} fight activation.
#[test]
fn polukranos_fight_activation() {
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::polukranos_unchained());
    g.battlefield_find_mut(p).unwrap().add_counters(CounterType::PlusOnePlusOne, 6);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: p,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        x_value: None,
    })
    .expect("fight the bear");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "6-power fight kills the bear");
    // The bear's 2 strike-back was prevented by removing two counters.
    assert_eq!(
        g.battlefield_find(p).unwrap().counter_count(CounterType::PlusOnePlusOne),
        4,
        "two counters paid for the bear's strike-back"
    );
}

/// Elspeth Conquers Death chapter I exiles an MV≥3 opponent permanent.
#[test]
fn elspeth_conquers_death_chapter_one_exiles() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::mind_stone()); // MV 2 — illegal
    let stone = g.add_card_to_battlefield(1, catalog::pyxis_of_pandemonium()); // MV 1 — illegal
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // MV 5 — the pick
    let saga = g.add_card_to_hand(0, catalog::elspeth_conquers_death());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: saga, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast ECD");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == serra), "MV 5 Angel exiled by chapter I");
    assert!(g.battlefield_find(big).is_some());
    assert!(g.battlefield_find(stone).is_some());
}

/// ECD chapter II taxes opponents' noncreature spells {2} until your next
/// turn (and the tax expires at your untap).
#[test]
fn elspeth_conquers_death_chapter_two_taxes_opponents() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::elspeth_conquers_death());
    g.saga_advance(saga); // I (no legal target — fizzles)
    drain_stack(&mut g);
    g.saga_advance(saga); // II
    drain_stack(&mut g);

    // P1's Lightning Bolt now costs {R} + {2}: one red floating is short.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "taxed cast rejected without the extra {{2}}"
    );
    g.players[1].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("paying {R} + {2} works");
    drain_stack(&mut g);

    // The tax expires at the controller's untap.
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.turn_scoped_spell_taxes.is_empty(), "tax cleared at your untap");
}

/// ECD chapter III reanimates a creature with a +1/+1 counter.
#[test]
fn elspeth_conquers_death_chapter_three_reanimates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let saga = g.add_card_to_battlefield(0, catalog::elspeth_conquers_death());
    g.saga_advance(saga); // I
    drain_stack(&mut g);
    g.saga_advance(saga); // II
    drain_stack(&mut g);
    g.saga_advance(saga); // III
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear reanimated");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "+1/+1 counter added");
    g.check_state_based_actions();
    assert!(g.battlefield_find(saga).is_none(), "saga sacrificed after III");
}

/// Dream Trawler grows +1/+0 per draw and draws when attacking.
#[test]
fn dream_trawler_draw_pump_and_attack_draw() {
    let mut g = two_player_game();
    let dt = g.add_card_to_battlefield(0, catalog::dream_trawler());
    g.add_card_to_library(0, catalog::forest());
    let mut events = vec![];
    g.draw_one(0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(dt).unwrap().power, 4, "+1/+0 from the draw");
}

/// Dream Trawler's discard-a-card activation grants hexproof.
#[test]
fn dream_trawler_discard_grants_hexproof() {
    let mut g = two_player_game();
    let dt = g.add_card_to_battlefield(0, catalog::dream_trawler());
    g.add_card_to_hand(0, catalog::forest());
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: dt, ability_index: 0, target: None, x_value: None,
    })
    .expect("discard a card: hexproof");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1, "card discarded as the cost");
    assert!(
        g.computed_permanent(dt).unwrap().keywords.contains(&crate::card::Keyword::Hexproof),
        "hexproof until end of turn"
    );
}

/// Arasta mints a 1/2 reach Spider when an opponent casts an instant.
#[test]
fn arasta_mints_spider_on_opponent_instant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::arasta_of_the_endless_web());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("opponent casts Bolt");
    drain_stack(&mut g);
    let spider = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Spider" && c.controller == 0)
        .expect("Spider token for Arasta's controller");
    assert_eq!((spider.power(), spider.toughness()), (1, 2));
}

/// Daxos's toughness equals devotion to white.
#[test]
fn daxos_toughness_tracks_white_devotion() {
    let mut g = two_player_game();
    let daxos = g.add_card_to_battlefield(0, catalog::daxos_blessed_by_the_sun());
    // Daxos's own {W}{W} counts for 2.
    assert_eq!(g.computed_permanent(daxos).unwrap().toughness, 2);
    g.add_card_to_battlefield(0, catalog::serra_angel()); // {3}{W}{W}
    assert_eq!(g.computed_permanent(daxos).unwrap().toughness, 4);
}

/// Daxos gains 1 life when another creature you control enters or dies.
#[test]
fn daxos_gains_on_other_creature_enter_and_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::daxos_blessed_by_the_sun());
    let life = g.players[0].life;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bear }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "+1 on the bear entering");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt the bear");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "+1 on the bear dying");
}

/// Tymaret Calls the Dead chapter I mills three and exiles a creature for a
/// 2/2 Zombie; chapter III gains life equal to your Zombies.
#[test]
fn tymaret_calls_the_dead_chapters() {
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let saga = g.add_card_to_hand(0, catalog::tymaret_calls_the_dead());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: saga, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast saga");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Zombie").count(),
        1,
        "chapter I: milled creature exiled for a Zombie"
    );
    g.saga_advance(saga); // II — another mill + Zombie
    drain_stack(&mut g);
    let life = g.players[0].life;
    g.saga_advance(saga); // III — gain life equal to Zombies (2)
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "chapter III gains per Zombie");
}

/// Thirst for Meaning draws three; without an enchantment you discard two.
#[test]
fn thirst_for_meaning_draws_three_discards_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::thirst_for_meaning());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Thirst");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "drew 3, discarded 2 (no enchantment)");
}

/// Shatter the Sky: a player with a power-4+ creature draws, then all
/// creatures die.
#[test]
fn shatter_the_sky_draws_then_wraths() {
    let mut g = two_player_game();
    g.add_card_to_library(1, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let spell = g.add_card_to_hand(0, catalog::shatter_the_sky());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    let p0_hand = g.players[0].hand.len();
    let p1_hand = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast wrath");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(serra).is_none());
    assert_eq!(g.players[0].hand.len(), p0_hand - 1, "no 4-power creature → no draw");
    assert_eq!(g.players[1].hand.len(), p1_hand + 1, "Serra's controller drew");
}

/// Alseid sacrifices to grant protection from a chosen color.
#[test]
fn alseid_grants_protection_from_chosen_color() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let alseid = g.add_card_to_battlefield(0, catalog::alseid_of_lifes_bounty());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: alseid,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        x_value: None,
    })
    .expect("sac Alseid");
    drain_stack(&mut g);
    assert!(g.battlefield_find(alseid).is_none(), "Alseid sacrificed");
    assert!(
        g.computed_permanent(bear)
            .unwrap()
            .keywords
            .contains(&crate::card::Keyword::Protection(Color::Red)),
        "bear protected from red"
    );
}

/// Mire Triton mills two and gains 2 on ETB.
#[test]
fn mire_triton_etb() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::forest());
    }
    let life = g.players[0].life;
    let mt = g.add_card_to_battlefield(0, catalog::mire_triton());
    g.fire_self_etb_triggers(mt, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 2, "milled two");
    assert_eq!(g.players[0].life, life + 2);
}

/// Aphemia exiles an enchantment from your graveyard at end step for a Zombie.
#[test]
fn aphemia_end_step_zombie() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::aphemia_the_cacophony());
    let ench = g.add_card_to_graveyard(0, catalog::shark_typhoon());
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == ench), "enchantment exiled");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Zombie"));
}

/// Underworld Rage-Hound escapes with a +1/+1 counter (none on a normal cast).
#[test]
fn underworld_rage_hound_escape_counter() {
    let mut g = two_player_game();
    let hound = g.add_card_to_graveyard(0, catalog::underworld_rage_hound());
    let fodder: Vec<_> =
        (0..3).map(|_| g.add_card_to_graveyard(0, catalog::lightning_bolt())).collect();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastEscape {
        card_id: hound,
        exile_cards: fodder,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("escape the hound");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(hound).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "escaped with a +1/+1 counter"
    );
}

/// Nessian Boar: each creature that blocks it lets its controller draw.
#[test]
fn nessian_boar_blocker_controller_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(1, catalog::forest());
    let boar = g.add_card_to_battlefield(0, catalog::nessian_boar());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(boar);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: boar,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let p1_hand = g.players[1].hand.len();
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, boar)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), p1_hand + 1, "blocker's controller drew");
}

/// Mystic Repeal bottoms an enchantment.
#[test]
fn mystic_repeal_bottoms_enchantment() {
    let mut g = two_player_game();
    g.add_card_to_library(1, catalog::forest());
    let ench = g.add_card_to_battlefield(1, catalog::shark_typhoon());
    let spell = g.add_card_to_hand(0, catalog::mystic_repeal());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(ench)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Mystic Repeal");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none());
    assert_eq!(
        g.players[1].library.last().map(|c| c.id),
        Some(ench),
        "on the bottom of its owner's library"
    );
}


/// Phoenix of Ash's firebreathing pump.
#[test]
fn phoenix_of_ash_pump() {
    let mut g = two_player_game();
    let ph = g.add_card_to_battlefield(0, catalog::phoenix_of_ash());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ph, ability_index: 0, target: None, x_value: None,
    })
    .expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ph).unwrap().power, 4, "+2/+0");
}

/// Agonizing Remorse exiles a chosen nonland hand card; you lose 1.
#[test]
fn agonizing_remorse_exiles_from_hand() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::agonizing_remorse());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "nonland exiled");
    assert_eq!(g.players[0].life, life - 1);
}

/// Eat to Extinction exiles a creature and surveils.
#[test]
fn eat_to_extinction_exiles_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::eat_to_extinction());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear));
}

/// Taranika's attack trigger makes the untapped target a 4/4 indestructible.
#[test]
fn taranika_attack_trigger_untaps_and_buffs() {
    let mut g = two_player_game();
    let t = g.add_card_to_battlefield(0, catalog::taranika_akroan_veteran());
    let goat = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(goat).unwrap().tapped = true;
    g.clear_sickness(t);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: t,
        target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let c = g.computed_permanent(goat).unwrap();
    assert!(!g.battlefield_find(goat).unwrap().tapped, "untapped");
    assert_eq!((c.power, c.toughness), (4, 4), "base 4/4 until end of turn");
    assert!(c.keywords.contains(&crate::card::Keyword::Indestructible));
}

/// Sweet Oblivion mills four and escapes from the graveyard.
#[test]
fn sweet_oblivion_mills_and_escapes() {
    let mut g = two_player_game();
    for _ in 0..8 { g.add_card_to_library(1, catalog::forest()); }
    let so = g.add_card_to_graveyard(0, catalog::sweet_oblivion());
    let fodder: Vec<_> =
        (0..4).map(|_| g.add_card_to_graveyard(0, catalog::lightning_bolt())).collect();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastEscape {
        card_id: so, exile_cards: fodder,
        target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("escape");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 4, "milled four");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == so),
        "escaped sorcery returns to the graveyard");
}

/// Klothys's Design pumps the team by green devotion.
#[test]
fn klothyss_design_pumps_by_devotion() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // {G} = 1 devotion
    let spell = g.add_card_to_hand(0, catalog::klothyss_design());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // Devotion at resolution: the bear's {G} = 1 → +1/+1.
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// Escape Protocol flickers an artifact when you cycle and pay {1}.
#[test]
fn escape_protocol_flickers_on_cycle() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::escape_protocol());
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.battlefield_find_mut(stone).unwrap().tapped = true;
    let cycler = g.add_card_to_hand(0, catalog::shark_typhoon());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3); // {X=0}{1}{U} cycle + {1} flicker
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    g.perform_action(GameAction::Cycle { card_id: cycler, x_value: Some(0) })
        .expect("cycle");
    drain_stack(&mut g);
    let c = g.battlefield_find(stone).expect("flickered back");
    assert!(!c.tapped, "returned untapped (new object)");
}

/// Protean Thaumaturge copies another creature on constellation.
#[test]
fn protean_thaumaturge_constellation_copy() {
    let mut g = two_player_game();
    let pt = g.add_card_to_battlefield(0, catalog::protean_thaumaturge());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    let ench = g.add_card_to_battlefield(0, catalog::escape_protocol());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
    drain_stack(&mut g);
    let c = g.battlefield_find(pt).unwrap();
    assert_eq!(c.definition.name, "Serra Angel", "became a copy");
}

/// Enigmatic Incarnation sacrifices an enchantment for a creature with
/// MV = 1 + the sacrifice's MV.
#[test]
fn enigmatic_incarnation_fetches_on_end_step() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::enigmatic_incarnation());
    let ench = g.add_card_to_battlefield(0, catalog::escape_protocol()); // MV 2
    let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 — wrong
    let wurm = g.add_card_to_library(0, catalog::serra_angel()); // MV 5 — wrong
    let triton = g.add_card_to_library(0, catalog::mire_triton()); // wrong (MV 2)
    let target_mv3 = g.add_card_to_library(0, catalog::dream_trawler()); // MV 6 — wrong
    let hill = g.add_card_to_library(0, catalog::phoenix_of_ash()); // MV 3 — the pick
    let _ = (bears, wurm, triton, target_mv3);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
        crate::decision::DecisionAnswer::Search(Some(hill)),
    ]));
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none(), "enchantment sacrificed");
    assert!(g.battlefield_find(hill).is_some(), "MV-3 creature fetched onto the battlefield");
}

/// Gallia pumps other Satyrs and draws two off a random discard when
/// attacking with three creatures.
#[test]
fn gallia_satyr_anthem_and_attack_payoff() {
    let mut g = two_player_game();
    let gallia = g.add_card_to_battlefield(0, catalog::gallia_of_the_endless_dance());
    let satyr = g.add_card_to_battlefield(0, catalog::voyaging_satyr());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(satyr).unwrap().power, 2, "+1/+1 to other Satyrs");
    assert_eq!(g.computed_permanent(gallia).unwrap().power, 2, "not itself");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "not non-Satyrs");

    for id in [gallia, satyr, bear] {
        g.clear_sickness(id);
    }
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: gallia, target: AttackTarget::Player(1) },
        Attack { attacker: satyr, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("attack with three");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "discarded one, drew two");
}

/// Kunoros locks reanimation and graveyard casts, but not library plays.
#[test]
fn kunoros_locks_graveyards_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::kunoros_hound_of_athreos());
    // Creature can't enter from a graveyard (Dread Return fizzles).
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let dr = g.add_card_to_hand(0, catalog::dread_return());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: dr, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dread Return");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "reanimation locked");
    // Flashback (cast from graveyard) is locked too.
    assert!(g.cast_from_zone_blocked(
        &catalog::lightning_bolt(),
        crate::card::Zone::Graveyard
    ));
    // Library casts are NOT locked (unlike Grafdigger's Cage).
    assert!(!g.cast_from_zone_blocked(
        &catalog::lightning_bolt(),
        crate::card::Zone::Library
    ));
}

/// Tectonic Giant's attack trigger mode 0 hits each opponent for 3.
#[test]
fn tectonic_giant_attack_burn_mode() {
    let mut g = two_player_game();
    let tg = g.add_card_to_battlefield(0, catalog::tectonic_giant());
    g.clear_sickness(tg);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: tg,
        target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "mode 0: 3 to each opponent");
}

// ── THB batch (modern_decks) — vanilla/ETB/death/constellation/activated ──────

/// Nyxborn Brute is a 7/3 enchantment creature.
#[test]
fn nyxborn_brute_is_enchantment_creature() {
    let c = catalog::nyxborn_brute();
    assert!(c.card_types.contains(&crate::card::CardType::Enchantment));
    assert!(c.card_types.contains(&crate::card::CardType::Creature));
    assert_eq!((c.power, c.toughness), (7, 3));
}

/// Moss Viper has deathtouch.
#[test]
fn moss_viper_has_deathtouch() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::moss_viper());
    assert!(g.battlefield_find(v).unwrap().has_keyword(&crate::card::Keyword::Deathtouch));
}

/// Discordant Piper dies → a 0/1 white Goat token appears.
#[test]
fn discordant_piper_dies_makes_goat() {
    let mut g = two_player_game();
    let piper = g.add_card_to_battlefield(0, catalog::discordant_piper());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(piper)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the piper");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Goat"),
        "Goat token minted on death");
}

/// Grim Physician dies → target opponent creature gets -1/-1.
#[test]
fn grim_physician_dies_shrinks_opponent() {
    let mut g = two_player_game();
    let phys = g.add_card_to_battlefield(0, catalog::grim_physician());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(phys)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt own physician");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (1, 1), "opponent bear shrunk -1/-1");
}

/// Careless Celebrant dies → 2 damage kills an opposing 2/2.
#[test]
fn careless_celebrant_dies_pings() {
    let mut g = two_player_game();
    let celeb = g.add_card_to_battlefield(0, catalog::careless_celebrant());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(celeb)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt own celebrant");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2 damage kills the opposing 2/2");
}

/// Elite Instructor ETB loots: library −1, graveyard +1 (drew then discarded).
#[test]
fn elite_instructor_loots_on_etb() {
    let mut g = two_player_game();
    let _draw = g.add_card_to_library(0, catalog::grizzly_bears());
    let inst = g.add_card_to_hand(0, catalog::elite_instructor());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib_before = g.players[0].library.len();
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: inst, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Elite Instructor");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 1, "drew one");
    assert_eq!(g.players[0].graveyard.len(), gy_before + 1, "discarded one");
}

/// Hyrax Tower Scout untaps a target creature on ETB.
#[test]
fn hyrax_tower_scout_untaps() {
    let mut g = two_player_game();
    let dork = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(dork).unwrap().tapped = true;
    let scout = g.add_card_to_hand(0, catalog::hyrax_tower_scout());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(dork)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: scout, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hyrax Tower Scout");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(dork).unwrap().tapped, "target creature untapped");
}

/// Eidolon of Philosophy: {6}{U}, sac → draw three.
#[test]
fn eidolon_of_philosophy_draws_three() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let eid = g.add_card_to_battlefield(0, catalog::eidolon_of_philosophy());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(6);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: eid, ability_index: 0, target: None, x_value: None,
    }).expect("activate Eidolon");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 3, "drew three");
    assert!(g.battlefield_find(eid).is_none(), "sacrificed itself");
}

/// Lampad of Death's Vigil: {1}, sac a creature → each opp loses 1, you gain 1.
#[test]
fn lampad_drains_on_sacrifice() {
    let mut g = two_player_game();
    let lampad = g.add_card_to_battlefield(0, catalog::lampad_of_deaths_vigil());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    let my_life = g.players[0].life;
    let opp_life = g.players[1].life;
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(fodder)),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: lampad, ability_index: 0, target: None, x_value: None,
    }).expect("activate Lampad");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
    assert_eq!(g.players[0].life, my_life + 1, "gained 1");
    assert_eq!(g.players[1].life, opp_life - 1, "opponent lost 1");
}

/// Captivating Unicorn's constellation taps an opposing creature.
#[test]
fn captivating_unicorn_constellation_taps() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::captivating_unicorn());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ench = g.add_card_to_battlefield(0, catalog::escape_protocol());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(opp)),
    ]));
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp).unwrap().tapped, "opposing creature tapped");
}

/// Sage of Mysteries' constellation mills a target player two cards.
#[test]
fn sage_of_mysteries_constellation_mills() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sage_of_mysteries());
    for _ in 0..4 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let ench = g.add_card_to_battlefield(0, catalog::escape_protocol());
    let lib = g.players[1].library.len();
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Player(1)),
    ]));
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib - 2, "milled two");
}

/// The remaining Nyxborn vanillas are enchantment creatures with the printed P/T.
#[test]
fn nyxborn_vanillas_are_enchantment_creatures() {
    for (def, pt) in [
        (catalog::nyxborn_colossus(), (6, 7)),
        (catalog::nyxborn_courser(), (2, 4)),
        (catalog::nyxborn_marauder(), (4, 3)),
        (catalog::nyxborn_seaguard(), (2, 5)),
    ] {
        assert!(def.card_types.contains(&crate::card::CardType::Enchantment), "{}", def.name);
        assert!(def.card_types.contains(&crate::card::CardType::Creature), "{}", def.name);
        assert_eq!((def.power, def.toughness), pt, "{}", def.name);
    }
}

/// Rumbling Sentry scries 1 on ETB (top card kept, library size unchanged).
#[test]
fn rumbling_sentry_scries_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let sentry = g.add_card_to_hand(0, catalog::rumbling_sentry());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    let lib = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: sentry, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Rumbling Sentry");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib, "scry keeps card count");
    assert_eq!(g.battlefield_find(sentry).map(|c| (c.power(), c.toughness())), Some((3, 6)));
}

/// Oread of Mountain's Blaze: {2}{R}, discard a card → draw a card.
#[test]
fn oread_rummages() {
    let mut g = two_player_game();
    let oread = g.add_card_to_battlefield(0, catalog::oread_of_mountains_blaze());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib = g.players[0].library.len();
    let gy = g.players[0].graveyard.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: oread, ability_index: 0, target: None, x_value: None,
    }).expect("activate Oread");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 1, "drew one");
    assert_eq!(g.players[0].graveyard.len(), gy + 1, "discarded one");
}

/// Pious Wayfarer's constellation pumps a target creature +1/+1.
#[test]
fn pious_wayfarer_constellation_pumps() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pious_wayfarer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ench = g.add_card_to_battlefield(0, catalog::escape_protocol());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 3), "+1/+1 until end of turn");
}

/// Daybreak Chimera's devotion cost reduction: with devotion 2 to white its
/// {3}{W}{W} cost drops to {1}{W}{W} (CR 700.5 / SelfCostReducedByDevotion).
#[test]
fn daybreak_chimera_devotion_reduces_generic() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::serra_angel()); // {3}{W}{W} → +2 white devotion
    let chimera = g.add_card_to_hand(0, catalog::daybreak_chimera());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1); // only {1} generic — exact after the {2} discount
    g.perform_action(GameAction::CastSpell {
        card_id: chimera, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast for {1}{W}{W} after devotion discount");
    drain_stack(&mut g);
    assert!(g.battlefield_find(chimera).is_some(), "Daybreak Chimera resolved");
}

// ── THB batch 2 tests ─────────────────────────────────────────────────────────

/// Hero of the Games' Heroic pumps the team +1/+0 when a spell targets it.
#[test]
fn hero_of_the_games_heroic_pumps_team() {
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::hero_of_the_games());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::infuriate());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(hero)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Infuriate on the hero");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 3, "team +1/+0");
}

/// Eidolon of Inspiration pumps a creature you control +2/+0 at begin-combat.
#[test]
fn eidolon_of_inspiration_begin_combat_pump() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.add_card_to_battlefield(0, catalog::eidolon_of_inspiration());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 4, "+2/+0 at begin-combat");
}

/// Favored of Iroas gains double strike on constellation.
#[test]
fn favored_of_iroas_constellation_double_strike() {
    let mut g = two_player_game();
    let fav = g.add_card_to_battlefield(0, catalog::favored_of_iroas());
    let ench = g.add_card_to_battlefield(0, catalog::escape_protocol());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
    drain_stack(&mut g);
    assert!(g.computed_permanent(fav).unwrap().keywords.contains(&crate::card::Keyword::DoubleStrike));
}

/// Pheres-Band Brawler fights an opposing creature on ETB.
#[test]
fn pheres_band_brawler_fights_on_etb() {
    let mut g = two_player_game();
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let brawler = g.add_card_to_hand(0, catalog::pheres_band_brawler());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(opp)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: brawler, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Pheres-Band Brawler");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp).is_none(), "4/4 fights and kills the 2/2");
}

/// Reverent Hoplite mints devotion-to-white 1/1 Soldiers on ETB.
#[test]
fn reverent_hoplite_makes_devotion_soldiers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::serra_angel()); // {3}{W}{W} → 2 white devotion
    let hoplite = g.add_card_to_hand(0, catalog::reverent_hoplite());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    let before = g.battlefield.iter().filter(|c| c.definition.name == "Human Soldier").count();
    g.perform_action(GameAction::CastSpell {
        card_id: hoplite, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Reverent Hoplite");
    drain_stack(&mut g);
    // Devotion = Serra (WW=2) + Hoplite's own {W} (1) = 3.
    let after = g.battlefield.iter().filter(|c| c.definition.name == "Human Soldier").count();
    assert_eq!(after - before, 3, "one token per white pip on the battlefield");
}

/// Rage-Scarred Berserker grants +1/+0 and indestructible on ETB.
#[test]
fn rage_scarred_berserker_grants_indestructible() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let zerk = g.add_card_to_hand(0, catalog::rage_scarred_berserker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(bear)),
        crate::decision::DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: zerk, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Rage-Scarred Berserker");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.power, 3, "+1/+0");
    assert!(c.keywords.contains(&crate::card::Keyword::Indestructible), "granted indestructible");
}

/// Leonin of the Lost Pride exiles an opposing graveyard card on death.
#[test]
fn leonin_dies_exiles_opponent_graveyard() {
    let mut g = two_player_game();
    let leonin = g.add_card_to_battlefield(0, catalog::leonin_of_the_lost_pride());
    let gy = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(gy)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(leonin)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt own Leonin");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == gy), "opponent graveyard card exiled");
}

/// Eutropia's constellation adds a +1/+1 counter and grants flying.
#[test]
fn eutropia_constellation_counter_and_flying() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::eutropia_the_twice_favored());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ench = g.add_card_to_battlefield(0, catalog::escape_protocol());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(bear)),
        crate::decision::DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "+1/+1 counter");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&crate::card::Keyword::Flying),
        "gains flying");
}

/// Brine Giant's affinity for enchantments cuts its generic cost by one per
/// enchantment you control (here {6}{U} → {4}{U} with two enchantments).
#[test]
fn brine_giant_affinity_for_enchantments() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::escape_protocol());
    g.add_card_to_battlefield(0, catalog::escape_protocol());
    let giant = g.add_card_to_hand(0, catalog::brine_giant());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4); // {4}{U} after the {2} discount
    g.perform_action(GameAction::CastSpell {
        card_id: giant, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Brine Giant for {4}{U}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_some(), "Brine Giant resolved at the reduced cost");
}

/// Loathsome Chimera can be cast from the graveyard for its Escape cost.
#[test]
fn loathsome_chimera_escapes() {
    let mut g = two_player_game();
    let chimera = g.add_card_to_graveyard(0, catalog::loathsome_chimera());
    let fodder: Vec<_> =
        (0..3).map(|_| g.add_card_to_graveyard(0, catalog::grizzly_bears())).collect();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastEscape {
        card_id: chimera, exile_cards: fodder, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("escape Loathsome Chimera for {4}{G} + exile three");
    drain_stack(&mut g);
    assert!(g.battlefield_find(chimera).is_some(), "escaped onto the battlefield");
}

// ── THB batch 5 — Omens, devotion, escape, blue tempo ────────────────────────

/// Omen of the Sun ETB makes two tokens and gains 2 life.
#[test]
fn omen_of_the_sun_etb() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::omen_of_the_sun());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    let creatures = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Omen of the Sun");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained 2");
    let now = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count();
    assert_eq!(now, creatures + 2, "two Soldier tokens");
}

/// Omen of the Forge ETB deals 2 damage to the opponent.
#[test]
fn omen_of_the_forge_etb_burns() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::omen_of_the_forge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Omen of the Forge");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "2 damage");
}

/// Mire's Grasp shrinks a 2/2 to -1/-1 and it dies.
#[test]
fn mires_grasp_kills_small_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::mires_grasp());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mire's Grasp");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear shrunk to -1/-1 and died");
}

/// Funeral Rites draws two, loses 2 life, and mills two.
#[test]
fn funeral_rites_draws_loses_mills() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::funeral_rites());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let (hand, life, gy) = (g.players[0].hand.len(), g.players[0].life, g.players[0].graveyard.len());
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Funeral Rites");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "lost 2");
    // -1 (cast) +2 (draw) = +1 hand; graveyard gains the spell + 2 milled.
    assert_eq!(g.players[0].hand.len(), hand + 1, "net +1 in hand");
    assert_eq!(g.players[0].graveyard.len(), gy + 3, "spell + 2 milled");
}

/// Soulreaper of Mogis sacrifices a creature to draw.
#[test]
fn soulreaper_sacrifices_to_draw() {
    let mut g = two_player_game();
    let reaper = g.add_card_to_battlefield(0, catalog::soulreaper_of_mogis());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: reaper, ability_index: 0, target: None, x_value: None,
    }).expect("activate Soulreaper");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Drag to the Underworld's devotion discount lets it be cast for {B}{B} with
/// two black devotion on board; destroys the target.
#[test]
fn drag_to_the_underworld_devotion_discount() {
    let mut g = two_player_game();
    // Two Gray-Merchant-ish black pips of devotion (use Mire Triton: {1}{B}).
    g.add_card_to_battlefield(0, catalog::mire_triton());
    g.add_card_to_battlefield(0, catalog::mire_triton());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::drag_to_the_underworld());
    // Only {B}{B}: the {2} generic is fully discounted by devotion 2.
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Drag discounted to {B}{B}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature destroyed");
}

/// Deny the Divine counters a creature spell and exiles it.
#[test]
fn deny_the_divine_counters_and_exiles() {
    let mut g = two_player_game();
    // Active player 0 casts a creature; player 1 counters it in response.
    let creature = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: creature, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a creature");
    let deny = g.add_card_to_hand(1, catalog::deny_the_divine());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: deny, target: Some(Target::Permanent(creature)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Deny the Divine");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == creature), "countered spell exiled");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != creature), "not in graveyard");
}

/// Venomous Hierophant ETB mills three.
#[test]
fn venomous_hierophant_mills_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::venomous_hierophant());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let gy = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Venomous Hierophant");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy + 3, "milled three");
}

/// Chain to Memory shrinks a creature's power by 4.
#[test]
fn chain_to_memory_shrinks_power() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::chain_to_memory());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Chain to Memory");
    drain_stack(&mut g);
    let c = g.compute_battlefield().into_iter().find(|c| c.id == bear).unwrap();
    assert!(c.power <= -2, "power dropped by 4 (got {})", c.power);
}

/// Whirlwind of Thought draws when you cast a noncreature spell.
#[test]
fn whirlwind_of_thought_draws_on_noncreature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::whirlwind_of_thought());
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a noncreature spell");
    drain_stack(&mut g);
    // -1 bolt + 1 drawn = net 0.
    assert_eq!(g.players[0].hand.len(), hand, "drew off the noncreature cast");
}

/// Underworld Charger escapes with two +1/+1 counters.
#[test]
fn underworld_charger_escapes_with_counters() {
    let mut g = two_player_game();
    let charger = g.add_card_to_graveyard(0, catalog::underworld_charger());
    let fodder: Vec<_> =
        (0..3).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastEscape {
        card_id: charger, exile_cards: fodder,
        target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("escape the charger");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(charger).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2, "escaped with two counters");
}

/// Tymaret's toughness equals devotion to black.
#[test]
fn tymaret_toughness_tracks_devotion() {
    let mut g = two_player_game();
    let tym = g.add_card_to_battlefield(0, catalog::tymaret_chosen_from_death());
    // Tymaret itself is {B}{B} = 2 devotion.
    let c = g.compute_battlefield().into_iter().find(|c| c.id == tym).unwrap();
    assert_eq!((c.power, c.toughness), (2, 2), "2/* with devotion 2");
}
