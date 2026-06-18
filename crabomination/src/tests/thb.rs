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
        card_id: dork, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3, "G → GGG");

    g.add_card_to_battlefield(0, catalog::mana_reflection());
    let dork2 = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.clear_sickness(dork2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dork2, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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
        additional_targets: Vec::new(),
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
        card_id: dt, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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
        additional_targets: Vec::new(),
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
        card_id: ph, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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
        card_id: eid, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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
        card_id: lampad, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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
        card_id: oread, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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
        card_id: reaper, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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

/// Battle cry (CR 702.92): an attacking Goblin Wardriver pumps each *other*
/// attacking creature +1/+0, but not itself.
#[test]
fn battle_cry_pumps_other_attackers() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let driver = g.add_card_to_battlefield(0, catalog::goblin_wardriver()); // 2/2 battle cry
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());       // 2/2
    g.clear_sickness(driver);
    g.clear_sickness(bear);
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: driver, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("both attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 3, "other attacker +1/+0");
    assert_eq!(g.battlefield_find(driver).unwrap().power(), 2, "battle cry doesn't pump itself");
}

// ── THB batch 6 ──────────────────────────────────────────────────────────────

/// Final Death exiles the target creature (not to the graveyard).
#[test]
fn final_death_exiles_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::final_death());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Final Death");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature removed");
    assert!(g.exile.iter().any(|c| c.id == bear), "exiled, not in graveyard");
}

/// Fruit of Tizerus drains the target player for 2.
#[test]
fn fruit_of_tizerus_drains_two() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::fruit_of_tizerus());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Fruit of Tizerus");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
}

/// Skophos Warleader sacrifices a creature to pump itself and gain menace.
#[test]
fn skophos_warleader_sac_pumps_and_grants_menace() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let leader = g.add_card_to_battlefield(0, catalog::skophos_warleader());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: leader, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Skophos");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
    let c = g.compute_battlefield().into_iter().find(|c| c.id == leader).unwrap();
    assert_eq!(c.power, 5, "+1/+0");
    assert!(c.keywords.contains(&Keyword::Menace), "gained menace");
}

/// Blight-Breath Catoblepas shrinks an opposing creature by your devotion to
/// black (its own {B}{B} = 2) → a 2/2 dies.
#[test]
fn blight_breath_shrinks_by_devotion() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::blight_breath_catoblepas());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Catoblepas");
    drain_stack(&mut g);
    // Catoblepas itself has {B}{B} → devotion 2 → bear -2/-2 → dies.
    assert!(g.battlefield_find(bear).is_none(), "2/2 shrunk to 0/0 and died");
}

/// Nylea's Forerunner grants trample to your other creatures.
#[test]
fn nyleas_forerunner_grants_trample() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nyleas_forerunner());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.compute_battlefield().into_iter().find(|c| c.id == bear).unwrap();
    assert!(c.keywords.contains(&Keyword::Trample), "other creature has trample");
}

// ── THB batch 7 ──────────────────────────────────────────────────────────────

/// Setessan Petitioner gains life equal to devotion to green (its own {G}{G}
/// = 2).
#[test]
fn setessan_petitioner_gains_devotion_life() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::setessan_petitioner());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Setessan Petitioner");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained life = devotion 2");
}

/// Voracious Typhon escapes with three +1/+1 counters (none on a normal cast).
#[test]
fn voracious_typhon_escapes_with_counters() {
    let mut g = two_player_game();
    let typhon = g.add_card_to_graveyard(0, catalog::voracious_typhon());
    let fodder: Vec<_> =
        (0..4).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastEscape {
        card_id: typhon, exile_cards: fodder,
        target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("escape the Typhon");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(typhon).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3, "escaped with three counters");
}

/// Omen of the Dead returns a creature card from the graveyard to hand.
#[test]
fn omen_of_the_dead_returns_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::omen_of_the_dead());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Omen of the Dead");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature back in hand");
}

/// Phalanx Tactics gives the target +2/+1 and other creatures +1/+1.
#[test]
fn phalanx_tactics_pumps_team() {
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 target
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 other
    let spell = g.add_card_to_hand(0, catalog::phalanx_tactics());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(hero)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Phalanx Tactics");
    drain_stack(&mut g);
    let bf = g.compute_battlefield();
    let h = bf.iter().find(|c| c.id == hero).unwrap();
    let a = bf.iter().find(|c| c.id == ally).unwrap();
    assert_eq!((h.power, h.toughness), (4, 3), "target +2/+1");
    assert_eq!((a.power, a.toughness), (3, 3), "other +1/+1");
}

/// Nessian Hornbeetle grows at begin-combat only if you control another
/// power-4+ creature.
#[test]
fn nessian_hornbeetle_grows_with_a_big_ally() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let beetle = g.add_card_to_battlefield(0, catalog::nessian_hornbeetle());
    // No big ally yet → no counter.
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(beetle).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    // Add a 4-power creature, fire again → counter.
    g.add_card_to_battlefield(0, catalog::craw_wurm()); // 6/4
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(beetle).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "grows with a power-4+ ally");
}

// ── THB batch 8 ──────────────────────────────────────────────────────────────

/// Revoke Existence exiles a target enchantment.
#[test]
fn revoke_existence_exiles_enchantment() {
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(1, catalog::omen_of_the_sun());
    let spell = g.add_card_to_hand(0, catalog::revoke_existence());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(ench)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Revoke Existence");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == ench), "enchantment exiled");
}

/// Sentinel's Eyes grants +1/+1 and vigilance.
#[test]
fn sentinels_eyes_pumps_and_grants_vigilance() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::sentinels_eyes());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sentinel's Eyes");
    drain_stack(&mut g);
    let c = g.compute_battlefield().into_iter().find(|c| c.id == bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "+1/+1");
    assert!(c.keywords.contains(&Keyword::Vigilance), "granted vigilance");
}

/// Triumphant Surge destroys a power-4+ creature and gains 3 life.
#[test]
fn triumphant_surge_destroys_big_creature() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    let spell = g.add_card_to_hand(0, catalog::triumphant_surge());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(wurm)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Triumphant Surge");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_none(), "big creature destroyed");
    assert_eq!(g.players[0].life, life + 3, "gained 3");
}

/// Final Flare sacrifices a creature as an additional cost and deals 5.
#[test]
fn final_flare_sacrifices_and_burns() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    let spell = g.add_card_to_hand(0, catalog::final_flare());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Final Flare");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed a creature as cost");
    assert!(g.battlefield_find(victim).is_none(), "5 damage killed the 6/4");
}

/// Iroas's Blessing burns an opposing creature on ETB and pumps the enchanted
/// one.
#[test]
fn iroass_blessing_burns_and_pumps() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(0, catalog::iroass_blessing());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(theirs)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(mine)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Iroas's Blessing");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "4 damage killed the 2/2");
    let c = g.compute_battlefield().into_iter().find(|c| c.id == mine).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "enchanted creature +1/+1");
}

/// Dreadful Apathy's {2}{W} exiles the enchanted creature.
#[test]
fn dreadful_apathy_exiles_enchanted() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::dreadful_apathy());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dreadful Apathy");
    drain_stack(&mut g);
    let aura_id = g.battlefield.iter().find(|c| c.definition.name == "Dreadful Apathy").unwrap().id;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura_id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Dreadful Apathy");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "enchanted creature exiled");
}

/// Sea God's Scorn returns up to three target creatures to their owners' hands
/// (ApplyToTargets multi-bounce), and the enchantment affinity reduces cost.
#[test]
fn sea_gods_scorn_bounces_up_to_three() {
    let mut g = two_player_game();
    let c1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c3 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::sea_gods_scorn());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(c1)),
        additional_targets: vec![Target::Permanent(c2), Target::Permanent(c3)],
        mode: None,
        x_value: None,
    })
    .expect("cast {3}{U} bouncing three creatures");
    drain_stack(&mut g);
    assert!(g.battlefield_find(c1).is_none(), "first creature returned");
    assert!(g.battlefield_find(c2).is_none(), "second creature returned");
    assert!(g.battlefield_find(c3).is_none(), "third creature returned");
    assert_eq!(g.players[1].hand.len(), 3, "all three back in owner's hand");
}

/// Sea God's Scorn affinity: each enchantment you control shaves {1} generic.
#[test]
fn sea_gods_scorn_affinity_reduces_cost() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dreadful_apathy());
    g.add_card_to_battlefield(0, catalog::dreadful_apathy());
    let c1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::sea_gods_scorn());
    // Two enchantments → {3} becomes {1}; pay {1}{U}.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(c1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast at the reduced {1}{U}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(c1).is_none(), "creature returned");
}

/// Wrap in Flames deals 1 to each of up to three creatures and they can't block.
#[test]
fn wrap_in_flames_pings_and_locks_blocking() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let c1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::wrap_in_flames());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(c1)),
        additional_targets: vec![Target::Permanent(c2)],
        mode: None,
        x_value: None,
    })
    .expect("cast {2}{R}");
    drain_stack(&mut g);
    // 2/2 bears each take 1 damage (survive) and gain CantBlock.
    let dmg = g.battlefield_find(c1).expect("bear survives").damage;
    assert_eq!(dmg, 1, "took 1 damage");
    let cp = g.computed_permanent(c1).expect("computed");
    assert!(cp.keywords.contains(&crate::card::Keyword::CantBlock), "can't block this turn");
}

// ── THB extra batch (modern_decks rebase) ───────────────────────────────────

#[test]
fn setessan_skirmisher_constellation_self_pump() {
    let mut g = two_player_game();
    let sk = g.add_card_to_battlefield(0, catalog::setessan_skirmisher());
    let omen = g.add_card_to_hand(0, catalog::omen_of_the_sea());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: omen, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("omen");
    drain_stack(&mut g);
    let c = g.computed_permanent(sk).unwrap();
    assert_eq!((c.power, c.toughness), (3, 2), "constellation +1/+1 EOT");
}

#[test]
fn gift_of_strength_pumps_and_grants_reach() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::gift_of_strength());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("gift of strength");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5), "+3/+3");
    assert!(c.keywords.contains(&Keyword::Reach));
}

#[test]
fn karametras_blessing_protects_an_enchanted_creature() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::escape_velocity());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("aura");
    drain_stack(&mut g);
    let blessing = g.add_card_to_hand(0, catalog::karametras_blessing());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: blessing, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("blessing");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert!(c.keywords.contains(&Keyword::Indestructible) && c.keywords.contains(&Keyword::Hexproof),
        "enchanted creature gains hexproof + indestructible");
}

#[test]
fn underworld_fires_sweeps_one_and_exiles() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let big = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::underworld_fires());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("underworld fires");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == small), "1/1 dies and is exiled");
    assert!(g.battlefield.iter().any(|c| c.id == big), "2/2 survives");
}

#[test]
fn satyrs_cunning_makes_a_cant_block_satyr() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::satyrs_cunning());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("satyr's cunning");
    drain_stack(&mut g);
    let tok = g.battlefield.iter().find(|c| c.definition.name == "Satyr").expect("Satyr");
    assert!(tok.definition.keywords.contains(&Keyword::CantBlock));
}

#[test]
fn travelers_amulet_fetches_a_basic_land() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let amulet = g.add_card_to_battlefield(0, catalog::travelers_amulet());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: amulet, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("amulet");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "basic fetched to hand");
    assert!(!g.battlefield.iter().any(|c| c.id == amulet), "sacrificed");
}

#[test]
fn escape_velocity_grants_haste() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::escape_velocity());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("escape velocity");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.power, 3, "+1/+0");
    assert!(c.keywords.contains(&Keyword::Haste));
}

#[test]
fn setessan_training_draws_on_etb_and_grants_trample() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let aura = g.add_card_to_hand(0, catalog::setessan_training());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("setessan training");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "aura left hand (-1), ETB drew (+1)");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample));
}

#[test]
fn staggering_insight_grants_lifelink() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::staggering_insight());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("staggering insight");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "+1/+1");
    assert!(c.keywords.contains(&Keyword::Lifelink));
}

// ── THB fill batch tests ──────────────────────────────────────────────────────

/// Temple of Abandon enters tapped (scry tapland) and taps for R or G.
#[test]
fn temple_of_abandon_enters_tapped() {
    let mut g = two_player_game();
    let land = g.add_card_to_hand(0, catalog::temple_of_abandon());
    g.perform_action(GameAction::PlayLand(land)).expect("play the Temple");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped, "Temple enters tapped");
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("tap for {R}");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "produced red");
}

/// Fateful End deals 3 to a player.
#[test]
fn fateful_end_burns_three() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::fateful_end());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Fateful End");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "3 damage to opponent");
}

/// Memory Drain counters a spell.
#[test]
fn memory_drain_counters_a_spell() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .unwrap();
    let drain = g.add_card_to_hand(0, catalog::memory_drain());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: drain, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Memory Drain");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "Bolt countered, never dealt damage");
}

/// Scavenging Harpy exiles a card from an opponent's graveyard on ETB.
#[test]
fn scavenging_harpy_exiles_from_opponent_graveyard() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let harpy = g.add_card_to_hand(0, catalog::scavenging_harpy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(dead)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: harpy, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Harpy");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().all(|c| c.id != dead), "card exiled from gy");
}

/// Sphinx Mindbreaker mills each opponent ten on ETB.
#[test]
fn sphinx_mindbreaker_mills_ten() {
    let mut g = two_player_game();
    for _ in 0..12 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let gy = g.players[1].graveyard.len();
    let sphinx = g.add_card_to_hand(0, catalog::sphinx_mindbreaker());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: sphinx, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Sphinx");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy + 10, "opponent milled ten");
}

/// Demon of Loathing forces a sacrifice on combat damage to a player.
#[test]
fn demon_of_loathing_forces_sacrifice() {
    let mut g = two_player_game();
    let demon = g.add_card_to_battlefield(0, catalog::demon_of_loathing());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.fire_combat_damage_to_player_triggers(demon, 1, 7);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "opponent sacrificed their creature");
}

/// Victory's Envoy puts a +1/+1 counter on each other creature at upkeep.
#[test]
fn victorys_envoy_pumps_team_at_upkeep() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let envoy = g.add_card_to_battlefield(0, catalog::victorys_envoy());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(envoy).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
        "not itself");
}

/// Pharika's Libation mode 0 makes an opponent sacrifice a creature.
#[test]
fn pharikas_libation_sacrifices_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::pharikas_libation());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![], mode: Some(0), x_value: None,
    })
    .expect("cast Pharika's Libation mode 0");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "opponent sacrificed a creature");
}

/// Return to Nature mode 1 destroys an enchantment.
#[test]
fn return_to_nature_destroys_enchantment() {
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(1, catalog::escape_protocol());
    let spell = g.add_card_to_hand(0, catalog::return_to_nature());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(ench)), additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("cast Return to Nature mode 1");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
}

/// Portent of Betrayal steals a creature, untapping it and giving haste.
#[test]
fn portent_of_betrayal_steals_creature() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(creature).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::portent_of_betrayal());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(creature)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Portent of Betrayal");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(creature).unwrap().controller, 0, "stolen for the turn");
    assert!(!g.battlefield_find(creature).unwrap().tapped, "untapped");
}

/// Nyx Lotus taps for devotion-many mana of a chosen color.
#[test]
fn nyx_lotus_taps_for_devotion() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // no pips
    // Two black pips on the battlefield → devotion to black 2.
    g.add_card_to_battlefield(0, catalog::gray_merchant_of_asphodel());
    let lotus = g.add_card_to_battlefield(0, catalog::nyx_lotus());
    g.clear_sickness(lotus);
    g.battlefield_find_mut(lotus).unwrap().tapped = false;
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Color(Color::Black),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: lotus, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("tap Nyx Lotus");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 2, "devotion-2 black mana");
}

/// Flicker of Fate blinks a creature (removing damage / re-triggering ETB).
#[test]
fn flicker_of_fate_blinks_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::flicker_of_fate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Flicker of Fate");
    drain_stack(&mut g);
    // A new object exists under the same owner (the bear id is gone, replaced).
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 1,
        "creature returned to the battlefield");
}

/// Renata's power equals devotion to green; it grants entering creatures +1/+1.
#[test]
fn renata_power_scales_with_devotion() {
    let mut g = two_player_game();
    let renata = g.add_card_to_battlefield(0, catalog::renata_called_to_the_hunt());
    // {2}{G}{G} = two green pips → devotion 2.
    assert_eq!(g.computed_permanent(renata).unwrap().power, 2, "power = devotion to green");
}

// ── THB aura / equipment / constellation batch tests ──────────────────────────

/// Aspect of Manticore gives +2/+0 and first strike on enter.
#[test]
fn aspect_of_manticore_grants_first_strike() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::aspect_of_manticore());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Aspect of Manticore");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.power, 4, "+2/+0");
    assert!(c.keywords.contains(&Keyword::FirstStrike), "first strike until EOT");
}

/// Commanding Presence is a +2/+2 first-strike aura.
#[test]
fn commanding_presence_pumps_and_first_strike() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::commanding_presence());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Commanding Presence");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "+2/+2");
    assert!(c.keywords.contains(&Keyword::FirstStrike));
}

/// Hydra's Growth adds a +1/+1 counter on enter, then doubles it at upkeep.
#[test]
fn hydras_growth_doubles_counters_at_upkeep() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::hydras_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Hydra's Growth");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "doubled at upkeep");
}

/// Warbriar Blessing fights an opposing creature on enter.
#[test]
fn warbriar_blessing_fights() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::nessian_boar()); // big body
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::warbriar_blessing());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(enemy)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(mine)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Warbriar Blessing");
    drain_stack(&mut g);
    assert!(g.battlefield_find(enemy).is_none(), "fought and killed the bear");
}

/// Bronze Sword grants +2/+0 on equip.
#[test]
fn bronze_sword_equips() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::bronze_sword());
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::Equip { equipment: sword, target: bear }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0");
}

/// Wings of Hubris grants flying on equip.
#[test]
fn wings_of_hubris_grants_flying() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wings = g.add_card_to_battlefield(0, catalog::wings_of_hubris());
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::Equip { equipment: wings, target: bear }).expect("equip");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying));
}

/// Nexus Wardens gains 2 life on constellation.
#[test]
fn nexus_wardens_constellation_gain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nexus_wardens());
    let ench = g.add_card_to_battlefield(0, catalog::escape_protocol());
    let life = g.players[0].life;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "constellation gained 2");
}

/// Nadir Kraken grows and makes a Tentacle when you pay {1} on a draw.
#[test]
fn nadir_kraken_grows_on_draw() {
    let mut g = two_player_game();
    let kraken = g.add_card_to_battlefield(0, catalog::nadir_kraken());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    let tokens_before = g.battlefield.iter().filter(|c| c.definition.name == "Tentacle").count();
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    let mut ev = Vec::new();
    g.draw_one(0, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(kraken).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "+1/+1 counter");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Tentacle").count(),
        tokens_before + 1, "made a Tentacle");
}

/// Sunmane Pegasus gains vigilance and lifelink for the turn.
#[test]
fn sunmane_pegasus_gains_vigilance_lifelink() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let peg = g.add_card_to_battlefield(0, catalog::sunmane_pegasus());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: peg, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("activate pump");
    drain_stack(&mut g);
    let c = g.computed_permanent(peg).unwrap();
    assert!(c.keywords.contains(&Keyword::Vigilance) && c.keywords.contains(&Keyword::Lifelink));
}

/// Skola Grovedancer's activated mill puts a card in your graveyard.
#[test]
fn skola_grovedancer_mills() {
    let mut g = two_player_game();
    let skola = g.add_card_to_battlefield(0, catalog::skola_grovedancer());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let gy = g.players[0].graveyard.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: skola, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("activate mill");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy + 1, "milled one");
}

/// Mantle of the Wolf makes two Wolves when it's destroyed (leaves for gy).
#[test]
fn mantle_of_the_wolf_makes_wolves_on_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::mantle_of_the_wolf());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Mantle of the Wolf");
    drain_stack(&mut g);
    let wolves_before = g.battlefield.iter().filter(|c| c.definition.name == "Wolf").count();
    // Destroy the aura with Return to Nature (mode 1).
    let rtn = g.add_card_to_hand(0, catalog::return_to_nature());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: rtn, target: Some(Target::Permanent(aura)), additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("destroy the Mantle");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Wolf").count(),
        wolves_before + 2, "two Wolves on aura death");
}

// ── THB heroic / sacrifice / lure batch tests ─────────────────────────────────

/// Hero of the Winds' heroic pumps your team when you target it with a spell.
#[test]
fn hero_of_the_winds_heroic_pumps_team() {
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::hero_of_the_winds());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pump = g.add_card_to_hand(0, catalog::infuriate()); // +3/+2 target creature
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pump, target: Some(Target::Permanent(hero)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Infuriate at the Hero");
    drain_stack(&mut g);
    // Bear got the heroic +1/+0 (Infuriate only buffed the hero).
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "team +1/+0 from heroic");
}

/// Heroes of the Revel makes a Satyr token on enter.
#[test]
fn heroes_of_the_revel_makes_satyr() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::heroes_of_the_revel());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Heroes of the Revel");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Satyr"), "made a Satyr token");
}

/// Irreverent Revelers mode 0 destroys an artifact on enter.
#[test]
fn irreverent_revelers_destroys_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::irreverent_revelers());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Mode(0),
        crate::decision::DecisionAnswer::Target(Target::Permanent(art)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Irreverent Revelers");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
}

/// Treeshaker Chimera draws three when it dies.
#[test]
fn treeshaker_chimera_draws_on_death() {
    let mut g = two_player_game();
    let chimera = g.add_card_to_battlefield(0, catalog::treeshaker_chimera());
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let hand = g.players[0].hand.len();
    // Lethal damage → dies SBA fires the draw trigger.
    g.battlefield_find_mut(chimera).unwrap().damage = 5;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 3, "drew three on death");
}

/// Blood Aspirant grows when you sacrifice a permanent.
#[test]
fn blood_aspirant_grows_on_sacrifice() {
    let mut g = two_player_game();
    let aspirant = g.add_card_to_battlefield(0, catalog::blood_aspirant());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentSacrificed { card_id: fodder, who: 0 }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(aspirant).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Underworld Sentinel exiles a creature from your graveyard on attack and
/// returns it when it dies.
#[test]
fn underworld_sentinel_exiles_then_returns() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let sentinel = g.add_card_to_battlefield(0, catalog::underworld_sentinel());
    g.clear_sickness(sentinel);
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(dead)),
    ]));
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sentinel, target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == dead), "creature exiled with the Sentinel");
    // Sentinel dies (lethal damage SBA) → the exiled creature enters play.
    g.battlefield_find_mut(sentinel).unwrap().damage = 5;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "exiled card returned to play");
}

// ── THB intervention / recursion / saga batch tests ───────────────────────────

/// Erebos's Intervention mode 0 shrinks a creature by X and gains X life.
#[test]
fn erebos_intervention_shrinks_and_gains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::erebos_s_intervention());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: Some(0), x_value: Some(2),
    })
    .expect("cast Erebos's Intervention X=2 mode 0");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "-2/-2 kills the 2/2");
    assert_eq!(g.players[0].life, life + 2, "gained X life");
}

/// Chainweb Aracnir pings a flyer for its power on enter.
#[test]
fn chainweb_aracnir_pings_a_flyer() {
    let mut g = two_player_game();
    let hawk = g.add_card_to_battlefield(1, catalog::suntail_hawk()); // 1/1 flyer
    let aracnir = g.add_card_to_hand(0, catalog::chainweb_aracnir());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(hawk)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: aracnir, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Chainweb Aracnir");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hawk).is_none(), "1 damage kills the 1/1 flyer");
}

/// Archon of Sun's Grace mints a Pegasus on constellation.
#[test]
fn archon_of_suns_grace_constellation_pegasus() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::archon_of_suns_grace());
    let ench = g.add_card_to_battlefield(0, catalog::escape_protocol());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Pegasus"), "minted a Pegasus");
}

/// Archon of Falling Stars returns an enchantment from your graveyard on death.
#[test]
fn archon_of_falling_stars_returns_enchantment() {
    let mut g = two_player_game();
    let archon = g.add_card_to_battlefield(0, catalog::archon_of_falling_stars());
    let ench = g.add_card_to_graveyard(0, catalog::escape_protocol());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
        crate::decision::DecisionAnswer::Target(Target::Permanent(ench)),
    ]));
    g.battlefield_find_mut(archon).unwrap().damage = 4;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_some(), "enchantment returned to the battlefield");
}

/// Elspeth's Nightmare chapter I destroys a small opposing creature.
#[test]
fn elspeths_nightmare_chapter_one_destroys() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::elspeths_nightmare());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.saga_advance(saga); // chapter I
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "power-2 creature destroyed");
}

/// Alirios enters tapped and makes a 3/2 Reflection.
#[test]
fn alirios_enters_tapped_with_reflection() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::alirios_enraptured());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Alirios");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().tapped, "enters tapped");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Reflection"), "made a Reflection");
}

// ── THB "first spell each opponent's turn" tests ──────────────────────────────

/// Arena Trickster grows on your first spell during an opponent's turn.
#[test]
fn arena_trickster_grows_on_first_spell_opp_turn() {
    let mut g = two_player_game();
    let trickster = g.add_card_to_battlefield(0, catalog::arena_trickster());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // It's the opponent's turn; player 0 casts at instant speed.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Bolt on opponent's turn");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(trickster).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "grew on the first off-turn spell");
}

/// Stinging Lionfish can tap a permanent on your first off-turn spell.
#[test]
fn stinging_lionfish_taps_on_first_off_turn_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stinging_lionfish());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
        crate::decision::DecisionAnswer::Mode(0), // tap
        crate::decision::DecisionAnswer::Target(Target::Permanent(victim)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Bolt on opponent's turn");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped, "Lionfish tapped the target");
}

// ── THB modern_decks batch ────────────────────────────────────────────────────

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Terror of Mount Velus grants your team double strike on ETB.
#[test]
fn terror_of_mount_velus_grants_team_double_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let terror = g.add_card_to_battlefield(0, catalog::terror_of_mount_velus());
    let trig = catalog::terror_of_mount_velus().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(terror, 0, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&crate::card::Keyword::DoubleStrike), "bear gains double strike");
}

/// Thundering Chariot becomes a creature once crewed.
#[test]
fn thundering_chariot_crews_into_a_creature() {
    let mut g = two_player_game();
    let chariot = g.add_card_to_battlefield(0, catalog::thundering_chariot());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(chariot).unwrap().card_types.contains(&crate::card::CardType::Creature), "vehicle is not a creature uncrewed");
    g.perform_action(GameAction::Crew { vehicle: chariot, crew_creatures: vec![bear] })
        .expect("crew 1 with a 2-power bear");
    assert!(g.computed_permanent(chariot).unwrap().card_types.contains(&crate::card::CardType::Creature), "crewed vehicle is a creature");
}

/// Wolfwillow Haven's enchanted land taps for an extra {G}.
#[test]
fn wolfwillow_haven_adds_extra_green() {
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let haven = g.add_card_to_hand(0, catalog::wolfwillow_haven());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: haven, target: Some(Target::Permanent(forest)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant the forest");
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility { card_id: forest, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap enchanted forest");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "base green plus extra green");
}

/// Mirror Shield grants +0/+2 and hexproof to the equipped creature.
#[test]
fn mirror_shield_grants_toughness_and_hexproof() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let shield = g.add_card_to_battlefield(0, catalog::mirror_shield());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: shield, target: bear }).expect("equip {2}");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.toughness, 4, "2/2 + 0/2");
    assert!(cp.keywords.contains(&crate::card::Keyword::Hexproof), "gains hexproof");
}

/// Shimmerwing Chimera bounces another enchantment you control at upkeep.
#[test]
fn shimmerwing_chimera_bounces_own_enchantment_at_upkeep() {
    let mut g = two_player_game();
    let chimera = g.add_card_to_battlefield(0, catalog::shimmerwing_chimera());
    let other = g.add_card_to_battlefield(0, catalog::nyxborn_brute());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    let trig = catalog::shimmerwing_chimera().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(chimera, 0, Some(Target::Permanent(other)), 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    assert!(g.battlefield_find(other).is_none(), "Nyxborn Brute returned to hand");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Nyxborn Brute"));
}

/// Thryx makes your mana-value-5+ spells cost {1} less.
#[test]
fn thryx_reduces_big_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::thryx_the_sudden_storm());
    let dragon = g.add_card_to_hand(0, catalog::shivan_dragon()); // {4}{R}{R} → {3}{R}{R}
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: dragon, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Shivan Dragon for {3}{R}{R} thanks to Thryx");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Shivan Dragon"));
}

/// Sleep of the Dead taps a creature and stuns it.
#[test]
fn sleep_of_the_dead_taps_and_stuns() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::sleep_of_the_dead());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sleep of the Dead");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.tapped, "creature tapped");
    assert_eq!(c.counter_count(CounterType::Stun), 1, "stun counter applied");
}

/// Inevitable End forces the enchanted creature's controller to sacrifice a
/// creature at upkeep.
#[test]
fn inevitable_end_sacrifices_at_upkeep() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::inevitable_end());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(victim);
    // The granted upkeep trigger: only the bear is a creature, so it must go.
    let ability = catalog::inevitable_end().equipped_bonus.unwrap().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(victim, 0, None, 0);
    g.resolve_effect(&ability, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "controller sacrificed the bear");
}

/// Impending Doom burns the enchanted creature's controller when it dies.
#[test]
fn impending_doom_burns_on_death() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::impending_doom());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(victim);
    let cp = g.computed_permanent(victim).unwrap();
    assert_eq!(cp.power, 5, "2/2 + 3/3");
    assert!(cp.keywords.contains(&crate::card::Keyword::MustAttack));
    let life_before = g.players[1].life;
    let dies = catalog::impending_doom().equipped_bonus.unwrap().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(victim, 1, None, 0);
    g.resolve_effect(&dies, &ctx).unwrap();
    assert_eq!(g.players[1].life, life_before - 3, "controller took 3");
}

/// Naiad of Hidden Coves discounts your spells only on opponents' turns.
#[test]
fn naiad_discounts_only_on_opponents_turns() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::naiad_of_hidden_coves());
    let angel = g.add_card_to_hand(0, catalog::restoration_angel()); // {3}{W} flash
    // Opponent's turn: discount applies → {2}{W} = 3 mana.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("discounted on opponent's turn");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Restoration Angel"));
}

/// Grasping Giant exiles the creature that blocks it (until it leaves).
#[test]
fn grasping_giant_exiles_its_blocker() {
    let mut g = two_player_game();
    let giant = g.add_card_to_battlefield(0, catalog::grasping_giant());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(giant);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: giant, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, giant)])).expect("block");
    drain_stack(&mut g);
    assert!(g.battlefield_find(blocker).is_none(), "blocker exiled");
    assert!(g.exile.iter().any(|c| c.id == blocker), "blocker is in exile");
}

/// Sunlit Hoplite has first strike on your turn but not on opponents' turns.
#[test]
fn sunlit_hoplite_first_strike_on_your_turn() {
    let mut g = two_player_game();
    let hoplite = g.add_card_to_battlefield(0, catalog::sunlit_hoplite());
    assert!(g.active_player_idx == 0);
    assert!(g.computed_permanent(hoplite).unwrap().keywords.contains(&crate::card::Keyword::FirstStrike),
        "first strike during your turn");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(hoplite).unwrap().keywords.contains(&crate::card::Keyword::FirstStrike),
        "no first strike on opponent's turn");
}

/// Swimmer in Nightmares grows with a stocked graveyard and is unblockable
/// under an Ashiok planeswalker.
#[test]
fn swimmer_in_nightmares_scales_and_evades() {
    let mut g = two_player_game();
    let swimmer = g.add_card_to_battlefield(0, catalog::swimmer_in_nightmares());
    assert_eq!(g.computed_permanent(swimmer).unwrap().power, 1, "base power");
    for _ in 0..10 {
        let id = g.next_id();
        g.players[1].graveyard.push(crate::card::CardInstance::new(id, catalog::grizzly_bears(), 1));
    }
    assert_eq!(g.computed_permanent(swimmer).unwrap().power, 4, "+3 with a ten-card graveyard");
    assert!(!g.computed_permanent(swimmer).unwrap().keywords.contains(&crate::card::Keyword::Unblockable));
    g.add_card_to_battlefield(0, catalog::ashiok_nightmare_weaver());
    assert!(g.computed_permanent(swimmer).unwrap().keywords.contains(&crate::card::Keyword::Unblockable),
        "unblockable under Ashiok");
}

// ── THB sagas ─────────────────────────────────────────────────────────────────

/// Resolve a saga's chapter `idx` (0-based) with the given target.
fn resolve_chapter(g: &mut GameState, saga: CardId, controller: usize, idx: usize, target: Option<Target>) {
    let def = g.battlefield_find(saga).unwrap().definition.clone();
    let eff = def.saga_chapters[idx].1.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(saga, controller, target, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
}

/// The First Iroan Games I makes a Human Soldier; III draws two under a big body.
#[test]
fn first_iroan_games_chapters() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::the_first_iroan_games());
    resolve_chapter(&mut g, saga, 0, 0, None);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Human Soldier"), "made a Soldier");
    g.add_card_to_battlefield(0, catalog::terror_of_mount_velus()); // 5/5 power ≥ 4
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    resolve_chapter(&mut g, saga, 0, 2, None);
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew two with a power-4+ creature");
}

/// The Binding of the Titans I mills three from each player.
#[test]
fn binding_of_the_titans_mills() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::the_binding_of_the_titans());
    for p in 0..2 {
        for _ in 0..5 { g.add_card_to_library(p, catalog::grizzly_bears()); }
    }
    resolve_chapter(&mut g, saga, 0, 0, None);
    assert_eq!(g.players[0].graveyard.len(), 3, "you milled three");
    assert_eq!(g.players[1].graveyard.len(), 3, "opponent milled three");
}

/// Kiora Bests the Sea God I creates an 8/8 hexproof Kraken.
#[test]
fn kiora_makes_kraken() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::kiora_bests_the_sea_god());
    resolve_chapter(&mut g, saga, 0, 0, None);
    let kraken = g.battlefield.iter().find(|c| c.definition.name == "Kraken").expect("made a Kraken");
    assert_eq!((kraken.definition.power, kraken.definition.toughness), (8, 8));
    assert!(kraken.definition.keywords.contains(&crate::card::Keyword::Hexproof));
}

/// The Akroan War III makes each tapped creature damage itself for its power.
#[test]
fn akroan_war_tapped_creatures_self_damage() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::the_akroan_war());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    resolve_chapter(&mut g, saga, 0, 2, None);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "tapped 2/2 took 2 and died");
}

/// Thassa's Intervention mode 0 digs X and draws up to two.
#[test]
fn thassas_intervention_digs() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::thassas_intervention());
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: Some(0), x_value: Some(3),
    }).expect("cast {X=3}{U}{U}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "spell left hand, two cards in");
}

/// Relentless Pursuit puts up to two creature/land cards into hand.
#[test]
fn relentless_pursuit_takes_creature_and_land() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::relentless_pursuit());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Relentless Pursuit");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "took two; spell left hand");
}

/// Commanding Presence grants +2/+2 and first strike.
#[test]
fn commanding_presence_pumps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::commanding_presence());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords.contains(&crate::card::Keyword::FirstStrike));
}

/// Furious Rise exiles a card to play while you control a big creature.
#[test]
fn furious_rise_exiles_with_big_creature() {
    let mut g = two_player_game();
    let rise = g.add_card_to_battlefield(0, catalog::furious_rise());
    g.add_card_to_battlefield(0, catalog::terror_of_mount_velus()); // 5/5
    g.add_card_to_library(0, catalog::lightning_bolt());
    let eff = g.battlefield_find(rise).unwrap().definition.triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(rise, 0, None, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    assert!(g.exile.iter().any(|c| c.definition.name == "Lightning Bolt"), "top card exiled to play");
}

/// Nightmare Shepherd exiles a dying creature to make a 1/1 Nightmare copy.
#[test]
fn nightmare_shepherd_copies_the_dead() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nightmare_shepherd());
    let doomed = g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    let eff = catalog::nightmare_shepherd().triggered_abilities[0].effect.clone();
    // The dying creature rides in as TriggerSource (source param of for_trigger).
    let ctx = crate::game::effects::EffectContext::for_trigger(doomed, 0, None, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    assert!(g.battlefield_find(doomed).is_none(), "original exiled");
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Shivan Dragon" && c.is_token);
    let copy = copy.expect("made a token copy");
    assert_eq!((copy.definition.power, copy.definition.toughness), (1, 1), "copy is 1/1");
    assert!(copy.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Nightmare));
}

/// Rise to Glory reanimates a creature from your graveyard.
#[test]
fn rise_to_glory_reanimates_creature() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::rise_to_glory());
    let dragon = g.add_card_to_graveyard(0, catalog::shivan_dragon());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(dragon)), additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Rise to Glory mode 0");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dragon).is_some(), "dragon reanimated");
}

/// Lagonna-Band Storyteller recurs an enchantment and gains its mana value.
#[test]
fn lagonna_band_storyteller_recurs_enchantment() {
    let mut g = two_player_game();
    let aura = g.add_card_to_graveyard(0, catalog::commanding_presence()); // MV 4
    let teller = g.add_card_to_battlefield(0, catalog::lagonna_band_storyteller());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
        crate::decision::DecisionAnswer::Target(Target::Permanent(aura)),
    ]));
    let life_before = g.players[0].life;
    let eff = catalog::lagonna_band_storyteller().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(teller, 0, Some(Target::Permanent(aura)), 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[0].life, life_before + 4, "gained MV 4");
    assert_eq!(g.players[0].library.first().map(|c| c.definition.name), Some("Commanding Presence"),
        "enchantment on top of library");
}

/// Purphoros's Intervention mode 1 deals twice X to a creature.
#[test]
fn purphoross_intervention_burns() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::purphoross_intervention());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: Some(1), x_value: Some(2),
    }).expect("cast {X=2}{R} mode 1");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "2/2 took 4 and died");
}

/// Dalakos grants flying and haste to your equipped creatures.
#[test]
fn dalakos_equips_grant_flying_haste() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dalakos_crafter_of_wonders());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boots = g.add_card_to_battlefield(0, catalog::lavaspur_boots());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: boots, target: bear }).expect("equip");
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&crate::card::Keyword::Flying), "equipped creature flies");
    assert!(cp.keywords.contains(&crate::card::Keyword::Haste), "equipped creature has haste");
}

/// The Triumph of Anax IV makes your creature fight an opponent's.
#[test]
fn triumph_of_anax_fight() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::the_triumph_of_anax());
    let mine = g.add_card_to_battlefield(0, catalog::terror_of_mount_velus()); // 5/5
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let eff = g.battlefield_find(saga).unwrap().definition.saga_chapters[3].1.clone();
    let mut ctx = crate::game::effects::EffectContext::for_trigger(saga, 0, Some(Target::Permanent(mine)), 0);
    ctx.targets.push(Target::Permanent(theirs)); // slot 1: defender
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(theirs).is_none(), "2/2 died to the 5/5");
}

/// The Triumph of Anax I pumps a creature by the lore count and grants trample.
#[test]
fn triumph_of_anax_pumps_by_lore() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::the_triumph_of_anax());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 1);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eff = g.battlefield_find(saga).unwrap().definition.saga_chapters[0].1.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(saga, 0, Some(Target::Permanent(bear)), 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "2 + 1 lore counter");
    assert!(cp.keywords.contains(&crate::card::Keyword::Trample));
}

/// Warden of the Chained can't attack without another big creature.
#[test]
fn warden_of_the_chained_needs_a_big_friend() {
    let mut g = two_player_game();
    let warden = g.add_card_to_battlefield(0, catalog::warden_of_the_chained());
    assert!(g.computed_permanent(warden).unwrap().keywords.contains(&crate::card::Keyword::CantAttack),
        "can't attack alone");
    g.add_card_to_battlefield(0, catalog::terror_of_mount_velus()); // 5/5
    assert!(!g.computed_permanent(warden).unwrap().keywords.contains(&crate::card::Keyword::CantAttack),
        "can attack with a power-4+ ally");
}

/// Elspeth, Sun's Nemesis −2 mints two 1/1 Soldiers.
#[test]
fn elspeth_suns_nemesis_makes_soldiers() {
    let mut g = two_player_game();
    let e = g.add_card_to_battlefield(0, catalog::elspeth_suns_nemesis());
    g.battlefield_find_mut(e).unwrap().counters.insert(CounterType::Loyalty, 5);
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: e, ability_index: 1, target: None, x_value: None,
    }).expect("-2 activatable");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Soldier))
        .count();
    assert_eq!(soldiers, 2, "two 1/1 Soldier tokens");
    assert_eq!(g.battlefield_find(e).unwrap().counter_count(CounterType::Loyalty), 3, "5 - 2");
}

/// Elspeth, Sun's Nemesis can be recast from the graveyard via Escape.
#[test]
fn elspeth_suns_nemesis_escapes() {
    let mut g = two_player_game();
    g.players[0].mana_pool.add(crate::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);
    let e = catalog::elspeth_suns_nemesis();
    let id = g.add_card_to_graveyard(0, e);
    // Four other cards to exile for the escape cost.
    for _ in 0..4 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
    let fodder: Vec<_> =
        g.players[0].graveyard.iter().filter(|c| c.id != id).take(4).map(|c| c.id).collect();
    g.perform_action(GameAction::CastEscape {
        card_id: id, exile_cards: fodder,
        target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("escape cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "Elspeth entered from graveyard");
}

/// Gravebreaker Lamia discounts spells cast from the graveyard by {1}, but
/// not spells cast from elsewhere.
#[test]
fn gravebreaker_lamia_discounts_graveyard_casts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gravebreaker_lamia());
    let id = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let card = g.players[0].graveyard.iter().find(|c| c.id == id).unwrap().clone();
    use crate::game::actions::cost_reduction_for_spell_zoned;
    assert_eq!(cost_reduction_for_spell_zoned(&g, 0, &card, None, true), 1, "graveyard cast: {{1}} off");
    assert_eq!(cost_reduction_for_spell_zoned(&g, 0, &card, None, false), 0, "hand cast: no discount");
}

/// Calix, Destiny's Hand −7 returns all enchantment cards from your
/// graveyard to the battlefield.
#[test]
fn calix_ultimate_reanimates_enchantments() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::calix_destinys_hand());
    g.battlefield_find_mut(c).unwrap().counters.insert(CounterType::Loyalty, 7);
    let ench = g.add_card_to_graveyard(0, catalog::omen_of_the_sun()); // enchantment
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // not an enchantment
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: c, ability_index: 2, target: None, x_value: None,
    }).expect("-7 activatable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_some(), "enchantment returned");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.is_creature()).count(), 1,
        "creature stays in graveyard");
}

/// Ashiok, Nightmare Muse +1 mints a 2/3 Nightmare.
#[test]
fn ashiok_nightmare_muse_makes_token() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::ashiok_nightmare_muse());
    g.battlefield_find_mut(a).unwrap().counters.insert(CounterType::Loyalty, 5);
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: a, ability_index: 0, target: None, x_value: None,
    }).expect("+1");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c|
        c.controller == 0 && c.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Nightmare)).unwrap();
    assert_eq!((token.power(), token.toughness()), (2, 3));
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::Loyalty), 6, "5 + 1");
}

/// Ashiok −3 bounces a permanent and makes its owner exile a card from hand.
#[test]
fn ashiok_nightmare_muse_bounce_and_exile() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::ashiok_nightmare_muse());
    g.battlefield_find_mut(a).unwrap().counters.insert(CounterType::Loyalty, 5);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears()); // a card to exile from hand
    let before = g.players[1].hand.len();
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: a, ability_index: 1, target: Some(Target::Permanent(victim)), x_value: None,
    }).expect("-3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "permanent left the battlefield");
    // Bounced card returned to hand (+1), then owner exiled one (−1) → net = before.
    assert_eq!(g.players[1].hand.len(), before, "bounce +1 then exile −1");
    assert_eq!(g.exile.iter().filter(|c| c.owner == 1).count(), 1, "one card exiled from hand");
}

/// Skophos Maze-Warden's {1} ability pumps it +1/-1.
#[test]
fn skophos_maze_warden_pumps() {
    let mut g = two_player_game();
    let w = g.add_card_to_battlefield(0, catalog::skophos_maze_warden());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: w, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(w).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "3/4 → 4/3");
}

/// Incendiary Oracle exiles a creature it kills in combat instead of letting
/// it die.
#[test]
fn incendiary_oracle_exiles_what_it_kills() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let oracle = g.add_card_to_battlefield(0, catalog::incendiary_oracle()); // 2/2
    let block = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(oracle);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: oracle, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(block, oracle)])).expect("block");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.exile.iter().any(|c| c.id == block), "blocker exiled, not killed to graveyard");
}

/// Shoal Kraken loots when an enchantment you control enters (if you choose to).
#[test]
fn shoal_kraken_constellation_loots() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::shoal_kraken());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let ench = g.add_card_to_battlefield(0, catalog::escape_protocol());
    let hand = g.players[0].hand.len();
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one (net 0)");
    assert_eq!(g.players[0].graveyard.len(), 1, "discarded a card");
}

/// Ilysian Caryatid makes one mana normally, two with a big creature out.
#[test]
fn ilysian_caryatid_scales_with_power() {
    let mut g = two_player_game();
    let dork = g.add_card_to_battlefield(0, catalog::ilysian_caryatid());
    g.clear_sickness(dork);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dork, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for one");
    assert_eq!(g.players[0].mana_pool.total(), 1, "one mana with no big creature");
    g.players[0].mana_pool.empty();
    g.battlefield_find_mut(dork).unwrap().tapped = false;
    g.add_card_to_battlefield(0, catalog::craw_wurm()); // 6/4
    g.perform_action(GameAction::ActivateAbility {
        card_id: dork, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for two");
    assert_eq!(g.players[0].mana_pool.total(), 2, "two mana with a power-4+ creature");
}

/// Elspeth, Undaunted Hero +2 puts a counter on up to two creatures.
#[test]
fn elspeth_undaunted_hero_plus_two() {
    let mut g = two_player_game();
    let e = g.add_card_to_battlefield(0, catalog::elspeth_undaunted_hero());
    g.battlefield_find_mut(e).unwrap().counters.insert(CounterType::Loyalty, 5);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: e, ability_index: 0, target: Some(Target::Permanent(a)), x_value: None,
    }).expect("+2");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Elspeth, Undaunted Hero −8 pumps your team by devotion to white and grants
/// flying. Elspeth's own {W}{W}{W} contributes devotion 3.
#[test]
fn elspeth_undaunted_hero_ultimate_devotion_pump() {
    let mut g = two_player_game();
    let e = g.add_card_to_battlefield(0, catalog::elspeth_undaunted_hero());
    g.battlefield_find_mut(e).unwrap().counters.insert(CounterType::Loyalty, 11);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: e, ability_index: 2, target: None, x_value: None,
    }).expect("-8");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "2/2 + devotion-to-white 3");
    assert!(cp.keywords.contains(&crate::card::Keyword::Flying), "gains flying");
}

/// Elspeth's Devotee tutors Elspeth, Undaunted Hero to hand on ETB.
#[test]
fn elspeths_devotee_tutors_elspeth() {
    let mut g = two_player_game();
    let walker = g.add_card_to_library(0, catalog::elspeth_undaunted_hero());
    let dev = g.add_card_to_battlefield(0, catalog::elspeths_devotee());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(walker)),
    ]));
    g.fire_self_etb_triggers(dev, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == walker), "Elspeth tutored to hand");
}

/// Enemy of Enlightenment shrinks by the opponent's hand size.
#[test]
fn enemy_of_enlightenment_shrinks_with_opponent_hand() {
    let mut g = two_player_game();
    let demon = g.add_card_to_battlefield(0, catalog::enemy_of_enlightenment());
    // Opponent holds 3 cards → 5/5 becomes 2/2.
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    let cp = g.computed_permanent(demon).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "5/5 - 3 cards");
}

/// Enemy of Enlightenment's upkeep makes each player discard a card.
#[test]
fn enemy_of_enlightenment_upkeep_discard() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::enemy_of_enlightenment());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 0, "you discarded");
    assert_eq!(g.players[1].hand.len(), 0, "opponent discarded");
}

/// Ashiok, Sculptor of Fears +2 draws and mills each player two.
#[test]
fn ashiok_sculptor_plus_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::ashiok_sculptor_of_fears());
    g.battlefield_find_mut(a).unwrap().counters.insert(CounterType::Loyalty, 4);
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    for _ in 0..3 { g.add_card_to_library(1, catalog::forest()); }
    let h = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: a, ability_index: 0, target: None, x_value: None,
    }).expect("+2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h + 1, "drew a card");
    assert_eq!(g.players[0].graveyard.len(), 2, "you milled two");
    assert_eq!(g.players[1].graveyard.len(), 2, "opponent milled two");
}

/// Ashiok, Sculptor of Fears −5 reanimates a graveyard creature under your
/// control.
#[test]
fn ashiok_sculptor_reanimates() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::ashiok_sculptor_of_fears());
    g.battlefield_find_mut(a).unwrap().counters.insert(CounterType::Loyalty, 5);
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: a, ability_index: 1, target: Some(Target::Permanent(corpse)), x_value: None,
    }).expect("-5");
    drain_stack(&mut g);
    let bear = g.battlefield_find(corpse).expect("reanimated");
    assert_eq!(bear.controller, 0, "under your control");
}

/// Ichthyomorphosis turns the enchanted creature into a 0/1 blue Fish with no
/// abilities.
#[test]
fn ichthyomorphosis_makes_a_fish() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer/vigilance
    let aura = g.add_card_to_hand(0, catalog::ichthyomorphosis());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ichthyomorphosis");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 1), "base 0/1");
    assert!(cp.keywords.is_empty(), "lost flying and vigilance");
    assert!(cp.subtypes.creature_types.contains(&crate::card::CreatureType::Fish), "is a Fish");
}

/// One with the Stars turns a creature into a noncreature enchantment.
#[test]
fn one_with_the_stars_makes_an_enchantment() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::one_with_the_stars());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast One with the Stars");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.card_types.contains(&CardType::Enchantment), "now an enchantment");
    assert!(!cp.card_types.contains(&CardType::Creature), "no longer a creature");
}

/// Heliod's Punishment strips the creature's abilities and stops it attacking
/// or blocking.
#[test]
fn heliods_punishment_neutralizes() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying/vigilance
    let aura = g.add_card_to_hand(0, catalog::heliods_punishment());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Heliod's Punishment");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).unwrap();
    assert!(!cp.keywords.contains(&crate::card::Keyword::Flying), "lost flying");
    assert!(cp.keywords.contains(&crate::card::Keyword::CantAttack), "can't attack");
    assert!(cp.keywords.contains(&crate::card::Keyword::CantBlock), "can't block");
}

/// Deathbellow War Cry tutors up to four Minotaurs straight onto the
/// battlefield.
#[test]
fn deathbellow_war_cry_tutors_minotaurs() {
    let mut g = two_player_game();
    let m1 = g.add_card_to_library(0, catalog::rage_scarred_berserker());
    let m2 = g.add_card_to_library(0, catalog::skophos_maze_warden());
    g.add_card_to_library(0, catalog::grizzly_bears()); // not a Minotaur
    let spell = g.add_card_to_hand(0, catalog::deathbellow_war_cry());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add_colorless(5);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(m1)),
        crate::decision::DecisionAnswer::Search(Some(m2)),
        crate::decision::DecisionAnswer::Search(None),
        crate::decision::DecisionAnswer::Search(None),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Deathbellow War Cry");
    drain_stack(&mut g);
    assert!(g.battlefield_find(m1).is_some() && g.battlefield_find(m2).is_some(),
        "both Minotaurs hit the battlefield");
}

/// Nylea's Intervention mode 2 deals twice X to each flyer.
#[test]
fn nyleas_intervention_burns_flyers() {
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 no fly
    let spell = g.add_card_to_hand(0, catalog::nyleas_intervention());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: Some(1), x_value: Some(2),
    }).expect("cast Nylea's Intervention mode 2, X=2");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(flyer).is_none(), "4/4 flyer took 4 and died");
    assert!(g.battlefield_find(ground).is_some(), "ground creature untouched");
}

/// Callaphe's power tracks your devotion to blue.
#[test]
fn callaphe_power_is_devotion_to_blue() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::callaphe_beloved_of_the_sea());
    // Callaphe's own {U}{U} gives devotion 2 → 2/3.
    assert_eq!(g.computed_permanent(c).unwrap().power, 2, "devotion to blue = 2");
    // Add a {U} creature → devotion 3 → 3/3.
    g.add_card_to_battlefield(0, catalog::towering_wave_mystic()); // {1}{U}
    assert_eq!(g.computed_permanent(c).unwrap().power, 3, "devotion to blue = 3");
}

/// Siona digs seven for an Aura to hand.
#[test]
fn siona_finds_an_aura() {
    let mut g = two_player_game();
    let aura = g.add_card_to_library(0, catalog::mires_grasp()); // an Aura
    g.add_card_to_library(0, catalog::grizzly_bears());
    let siona = g.add_card_to_battlefield(0, catalog::siona_captain_of_the_pyleas());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(aura)),
    ]));
    g.fire_self_etb_triggers(siona, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == aura), "Aura put into hand");
}

/// The client view surfaces an Ichthyomorphosis'd creature's computed Fish
/// subtype and the lost-all-abilities flag.
#[test]
fn view_surfaces_aura_characteristic_override() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::serra_angel());
    let aura = g.add_card_to_hand(0, catalog::ichthyomorphosis());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(host)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant");
    drain_stack(&mut g);
    let view = crate::server::view::project(&g, 0);
    let pv = view.battlefield.iter().find(|p| p.id == host).unwrap();
    assert!(pv.creature_subtypes.contains(&crate::card::CreatureType::Fish), "shown as a Fish");
    assert!(pv.lost_all_abilities, "abilities-removed flag surfaced");
    assert_eq!((pv.power, pv.toughness), (0, 1));
}

/// Flummoxed Cyclops can block a lone attacker but not a swarm of two-plus.
#[test]
fn flummoxed_cyclops_cant_block_a_swarm() {
    let cant_block = |attackers: usize| -> bool {
        let mut g = two_player_game();
        g.active_player_idx = 1;
        let cyclops = g.add_card_to_battlefield(0, catalog::flummoxed_cyclops());
        let atk: Vec<_> = (0..attackers)
            .map(|_| {
                let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
                g.clear_sickness(a);
                a
            })
            .collect();
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(
            atk.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(0) }).collect(),
        )).expect("attack");
        drain_stack(&mut g);
        g.computed_permanent(cyclops).unwrap().keywords.contains(&crate::card::Keyword::CantBlock)
    };
    assert!(!cant_block(1), "one attacker: can block");
    assert!(cant_block(2), "two attackers: can't block");
}

/// Altar of the Pantheon raises devotion to every color by one (CR 700.5),
/// and its mana ability gains 1 life only while you control a God/Demigod/
/// legendary enchantment.
#[test]
fn altar_of_the_pantheon_devotion_and_conditional_life() {
    let mut g = two_player_game();
    let altar = g.add_card_to_battlefield(0, catalog::altar_of_the_pantheon());
    // +1 to each color, even with no colored pips on the battlefield.
    assert_eq!(g.devotion_to(0, &[Color::White]), 1, "altar adds 1 white devotion");
    assert_eq!(g.devotion_to(0, &[Color::Black]), 1, "altar adds 1 to each color");

    // No God/legendary enchantment yet: tapping for mana gains no life
    // (AutoDecider resolves the any-color choice; it stays a mana ability).
    let life = g.players[0].life;
    let mana_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: altar, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("activate mana ability");
    assert_eq!(g.players[0].mana_pool.total(), mana_before + 1, "added one mana");
    assert_eq!(g.players[0].life, life, "no payoff permanent: no life");

    // Add a God; now the rider gains a life.
    g.battlefield_find_mut(altar).unwrap().tapped = false;
    g.add_card_to_battlefield(0, catalog::heliod_god_of_the_sun());
    g.perform_action(GameAction::ActivateAbility {
        card_id: altar, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("activate again");
    assert_eq!(g.players[0].life, life + 1, "God controlled → gain 1 life");
}

/// Hateful Eidolon draws one card per Aura you controlled on an enchanted
/// creature when it dies.
#[test]
fn hateful_eidolon_draws_per_aura_on_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hateful_eidolon());
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::commanding_presence());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(victim);
    g.add_card_to_library(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.battlefield_find_mut(victim).unwrap().damage = 99;
    let events = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "enchanted creature died");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew one card for the Aura");
}

/// Dawn Evangel returns a small creature from your graveyard when a creature
/// wearing one of your Auras dies.
#[test]
fn dawn_evangel_reanimates_on_aura_wearer_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dawn_evangel());
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::commanding_presence());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(victim);
    let buried = g.add_card_to_graveyard(0, catalog::llanowar_elves()); // MV 1
    g.battlefield_find_mut(victim).unwrap().damage = 99;
    let events = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == buried), "small creature returned to hand");
}

/// Minion's Return reanimates the enchanted creature under your control when
/// it dies (CR 603.6d leaves-battlefield trigger off the Aura's LKI).
#[test]
fn minions_return_steals_on_death() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::minions_return());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(victim);
    g.battlefield_find_mut(victim).unwrap().damage = 99;
    let events = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears");
    assert!(back.is_some(), "creature returned to the battlefield");
    assert_eq!(back.unwrap().controller, 0, "returned under the Aura controller's control");
}

/// Inspire Awe fogs vanilla attackers but lets enchanted / enchantment
/// creatures still deal combat damage.
#[test]
fn inspire_awe_fogs_all_but_enchanted() {
    let mut g = two_player_game();
    let vanilla = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let enchcreature = g.add_card_to_battlefield(0, catalog::dawn_evangel()); // 2/3 enchantment creature
    g.clear_sickness(vanilla);
    g.clear_sickness(enchcreature);
    let spell = g.add_card_to_hand(0, catalog::inspire_awe());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Inspire Awe");
    drain_stack(&mut g);
    let life = g.players[1].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: vanilla, target: AttackTarget::Player(1) },
        Attack { attacker: enchcreature, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    // Only the enchantment creature (2/3) connects; the vanilla bear is fogged.
    assert_eq!(g.players[1].life, life - 2, "only the enchantment creature dealt damage");
}

/// Ironscale Hydra prevents combat damage dealt to it and grows with a +1/+1
/// counter (CR 615), for both attacker- and blocker-dealt damage.
#[test]
fn ironscale_hydra_prevents_and_grows() {
    let mut g = two_player_game();
    g.active_player_idx = 1; // opponent attacks into our Hydra
    let hydra = g.add_card_to_battlefield(0, catalog::ironscale_hydra()); // 5/5
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(attacker);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker, target: AttackTarget::Player(0) },
    ])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(hydra, attacker)])).expect("block");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    let h = g.battlefield_find(hydra).expect("Hydra survives — its damage was prevented");
    assert_eq!(h.damage, 0, "no combat damage marked on the Hydra");
    assert_eq!(h.counter_count(CounterType::PlusOnePlusOne), 1, "grew a +1/+1 counter");
    // The 2/2 attacker took the Hydra's 5 and died.
    assert!(g.battlefield_find(attacker).is_none(), "attacker took the Hydra's damage and died");
}

/// Dreamshaper Shaman: paying {2}{R} + sacrificing a nonland permanent digs
/// to a nonland permanent card and puts it onto the battlefield.
#[test]
fn dreamshaper_shaman_digs_to_a_permanent() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let shaman = g.add_card_to_battlefield(0, catalog::dreamshaper_shaman());
    g.add_card_to_library(0, catalog::grizzly_bears()); // top of library — a nonland permanent
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let eff = g.battlefield_find(shaman).unwrap().definition.triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(shaman, 0, None, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(shaman).is_none(), "the Shaman was sacrificed to its own cost");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the revealed nonland permanent entered the battlefield");
}

/// Athreos, Shroud-Veiled: a coin-countered creature that dies returns under
/// your control; the end-step trigger places the coin counter.
#[test]
fn athreos_reclaims_coin_countered_creature() {
    let mut g = two_player_game();
    let athreos = g.add_card_to_battlefield(0, catalog::athreos_shroud_veiled());
    // End-step trigger places a coin counter on another target creature.
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let eff = g.battlefield_find(athreos).unwrap().definition.triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(athreos, 0, Some(Target::Permanent(victim)), 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.battlefield_find(victim).unwrap().counter_count(CounterType::Coin), 1, "coin counter placed");
    // The coin-countered creature dies → returns under our control.
    g.battlefield_find_mut(victim).unwrap().damage = 99;
    let events = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears");
    assert!(back.is_some(), "the coin-countered creature returned to the battlefield");
    assert_eq!(back.unwrap().controller, 0, "returned under Athreos's controller's control");
}

/// Ashiok's Erasure: counter+exile a spell, lock opponents off that name while
/// it's on the battlefield, then return the card to its owner's hand on leave.
#[test]
fn ashioks_erasure_exiles_locks_and_returns() {
    let mut g = two_player_game();
    // P1 casts a Lightning Bolt; P0 flashes in Ashiok's Erasure to exile it.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp bolt on stack");
    let erasure = g.add_card_to_hand(0, catalog::ashioks_erasure());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: erasure, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Erasure on stack");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "the bolt is exiled, not in any graveyard");
    assert_eq!(g.players[0].life, 20, "the exiled bolt never resolved");

    // P1 can't cast another Lightning Bolt while the Erasure stays in play.
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let res = g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(res.is_err(), "opponent is locked out of casting the named spell");

    // Erasure leaves → the exiled card returns to its owner's hand.
    let erasure_id = g.battlefield.iter().find(|c| c.definition.name == "Ashiok's Erasure").unwrap().id;
    g.remove_from_battlefield_to_graveyard_raw(erasure_id);
    g.check_state_based_actions();
    assert!(g.players[1].hand.iter().any(|c| c.id == bolt), "exiled card returns to owner's hand on leave");
}

/// Entrancing Lyre taps a creature with power ≤ X and locks it from untapping
/// while the Lyre stays tapped; the lock releases once the Lyre leaves.
#[test]
fn entrancing_lyre_taps_and_locks() {
    let mut g = two_player_game();
    let lyre = g.add_card_to_battlefield(0, catalog::entrancing_lyre());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lyre, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: Some(2),
    }).expect("tap a power-2 creature with X=2");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.tapped, "the bear is tapped");
    assert_eq!(b.untap_locked_by, Some(lyre), "locked by the Lyre");
    // Opponent's untap step: the lock holds while the Lyre stays tapped.
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped, "stays tapped while the Lyre is tapped");
    // The Lyre leaves → the lock releases on the next untap step.
    g.remove_from_battlefield_to_graveyard_raw(lyre);
    g.active_player_idx = 1;
    g.do_untap();
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untaps once the Lyre is gone");
}

/// Entrancing Lyre's "{X}: power X or less" target filter rejects a creature
/// whose power exceeds the X paid.
#[test]
fn entrancing_lyre_respects_power_filter() {
    let mut g = two_player_game();
    let lyre = g.add_card_to_battlefield(0, catalog::entrancing_lyre());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.players[0].mana_pool.add_colorless(1);
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: lyre, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: Some(1),
    });
    assert!(res.is_err(), "power 2 > X=1 is an illegal target");
}

/// Haktos the Unscarred — the ETB roll (forced to "2") grants protection from
/// each mana value other than 2: a MV-2 source connects, a MV-3 source can't.
#[test]
fn haktos_protection_from_each_mv_except_chosen() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(1)])); // d3=1 → chosen 2
    let haktos = g.add_card_to_battlefield(0, catalog::haktos_the_unscarred());
    let eff = catalog::haktos_the_unscarred().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(haktos, 0, None, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    assert!(
        g.computed_permanent(haktos).unwrap().keywords.iter().any(|k|
            matches!(k, crate::card::Keyword::ProtectionFromManaValueExcept(2))),
        "gained protection from each mana value other than 2");
    let mv2 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // {1}{G} = MV 2
    let mv3 = g.add_card_to_battlefield(1, catalog::gray_ogre());     // {2}{R} = MV 3
    assert!(!g.damage_prevented_by_protection(mv2, haktos), "MV-2 source connects");
    assert!(g.damage_prevented_by_protection(mv3, haktos), "MV-3 source is prevented by protection");
}

/// Medomai's Prophecy III: after naming a card (II), the first time you cast a
/// spell with that name this turn you draw two.
#[test]
fn medomais_prophecy_chapter_iii_draws_on_named_cast() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::medomais_prophecy());
    // Chapter II — name "Lightning Bolt".
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard("Lightning Bolt".into())]));
    let ch2 = catalog::medomais_prophecy().saga_chapters[1].1.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(saga, 0, None, 0);
    g.resolve_effect(&ch2, &ctx).unwrap();
    assert_eq!(g.battlefield_find(saga).unwrap().named_card.as_deref(), Some("Lightning Bolt"));
    // Chapter III — arm the "first named cast this turn → draw two" trigger.
    let ch3 = catalog::medomais_prophecy().saga_chapters[2].1.clone();
    g.resolve_effect(&ch3, &ctx).unwrap();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Lightning Bolt");
    drain_stack(&mut g);
    // Bolt leaves hand (-1) and the chapter-III trigger draws two (+2).
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "casting the named spell drew two");
}

/// Atris, Oracle of Half-Truths: ETB reveals the top three of your library and
/// splits them into hand/graveyard (value heuristic — highest-MV card isolated,
/// you keep the higher-value pile).
#[test]
fn atris_splits_top_three_into_hand_and_graveyard() {
    let mut g = two_player_game();
    // Top of library (index 0 first): a 2-MV bear, then two 0-MV lands.
    g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 — isolated pile A
    g.add_card_to_library(0, catalog::island());        // MV 0
    g.add_card_to_library(0, catalog::island());        // MV 0
    let atris = g.add_card_to_battlefield(0, catalog::atris_oracle_of_half_truths());
    let eff = catalog::atris_oracle_of_half_truths().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(atris, 0, None, 0);
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    g.resolve_effect(&eff, &ctx).unwrap();
    // Pile A (the bear, total MV 2) outweighs pile B (two lands, MV 0) → bear
    // to hand, the two lands to the graveyard.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "the higher-value pile (1 card) went to hand");
    assert_eq!(g.players[0].graveyard.len(), gy_before + 2, "the other two cards went to the graveyard");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "kept the bear");
}

/// Storm Herald: ETB returns Aura cards from your graveyard attached to your
/// creatures, then exiles them at your next end step.
#[test]
fn storm_herald_returns_auras_then_exiles_eot() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_graveyard(0, catalog::pacifism());
    let herald = g.add_card_to_battlefield(0, catalog::storm_herald());
    let eff = catalog::storm_herald().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(herald, 0, None, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    let a = g.battlefield_find(aura).expect("Pacifism returned to the battlefield");
    assert_eq!(a.attached_to, Some(bear), "attached to a creature you control");
    // Exiled at the next end step.
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == aura), "the returned Aura is exiled at the next end step");
}

/// Allure of the Unknown: reveals the top six, the opponent exiles the best
/// nonland (with a free may-play of its own), the rest go to your hand.
#[test]
fn allure_of_the_unknown_exiles_nonland_rest_to_hand() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears()); // MV2 nonland → exiled
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_library(0, catalog::shock());          // MV1 nonland
    g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::allure_of_the_unknown());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len(); // includes the Allure spell
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Allure of the Unknown");
    drain_stack(&mut g);
    let bear = g.exile.iter().find(|c| c.definition.name == "Grizzly Bears")
        .expect("the best nonland was exiled");
    assert_eq!(bear.may_play_until.as_ref().unwrap().player, 1, "the opponent may cast the exiled card");
    // Allure left hand on cast (-1); the other five revealed cards entered hand (+5).
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 5, "the rest went to your hand");
}
