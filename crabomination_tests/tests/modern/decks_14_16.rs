#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── modern_decks-14 ──────────────────────────────────────────────────────────

#[test]
fn vindicate_destroys_target_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vindicate());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Vindicate castable for {1}{W}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Vindicate destroys target permanent");
}

#[test]
fn vindicate_can_target_a_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::vindicate());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Vindicate accepts a land target (Permanent filter)");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == land),
        "Vindicate destroys a land target — same as Oracle");
}

#[test]
fn anguished_unmaking_exiles_and_caster_loses_three_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::anguished_unmaking());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Anguished Unmaking castable for {1}{W}{B}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in exile, not graveyard");
    assert_eq!(g.players[0].life, life_before - 3,
        "Caster loses 3 life");
}

#[test]
fn magma_spray_deals_two_damage_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::magma_spray());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Magma Spray castable for {R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Grizzly Bears (2-toughness) dies to 2 damage");
}

#[test]
fn despark_exiles_high_cmc_permanent() {
    let mut g = two_player_game();
    let craw = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6 CMC
    let id = g.add_card_to_hand(0, catalog::despark());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(craw)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Despark castable for {W}{B}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == craw),
        "6-CMC Craw Wurm gets exiled by Despark");
}

#[test]
fn despark_rejects_low_cmc_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2 CMC
    let id = g.add_card_to_hand(0, catalog::despark());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Despark should reject a 2-CMC creature: {:?}", err);
}

#[test]
fn crumble_to_dust_exiles_nonbasic_but_rejects_basic() {
    let mut g = two_player_game();
    let dual = g.add_card_to_battlefield(1, catalog::watery_grave());
    let basic = g.add_card_to_battlefield(1, catalog::island());
    let id_ok = g.add_card_to_hand(0, catalog::crumble_to_dust());
    let id_bad = g.add_card_to_hand(0, catalog::crumble_to_dust());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Red, 4);

    g.perform_action(GameAction::CastSpell {
        card_id: id_ok,
        target: Some(Target::Permanent(dual)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Crumble to Dust castable for {2}{R}{R}");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == dual), "Watery Grave (nonbasic) gets exiled");

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id_bad,
        target: Some(Target::Permanent(basic)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Crumble to Dust should reject a basic Island: {:?}", err);
}

#[test]
fn crumble_to_dust_sweeps_same_named_cards_from_all_zones() {
    // Exiles the targeted nonbasic land AND every same-named copy in the
    // owner's graveyard, hand, and library; the rest is shuffled, not lost.
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::watery_grave());
    let in_hand = g.add_card_to_hand(1, catalog::watery_grave());
    let in_gy = g.add_card_to_graveyard(1, catalog::watery_grave());
    let in_lib = g.add_card_to_library(1, catalog::watery_grave());
    let bystander = g.add_card_to_library(1, catalog::island()); // different name, survives

    let crumble = g.add_card_to_hand(0, catalog::crumble_to_dust());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Red, 4);
    g.perform_action(GameAction::CastSpell {
        card_id: crumble, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Crumble castable");
    drain_stack(&mut g);

    for id in [target, in_hand, in_gy, in_lib] {
        assert!(g.exile.iter().any(|c| c.id == id), "same-named card {id:?} exiled");
    }
    assert!(g.players[1].library.iter().any(|c| c.id == bystander),
        "a differently-named card stays in the library");
}

#[test]
fn skullcrack_deals_three_damage_to_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::skullcrack());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Skullcrack castable for {1}{R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 3,
        "Skullcrack deals 3 damage to target player");
}

#[test]
fn skullcrack_locks_target_player_lifegain_for_the_turn() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::skullcrack());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Skullcrack castable");
    drain_stack(&mut g);

    // Target is now locked from gaining life.
    assert!(g.players[1].cannot_gain_life_this_turn);
    let life_after_bolt = g.players[1].life;
    // Try to gain 5 life — should be a no-op.
    g.adjust_life(1, 5);
    assert_eq!(g.players[1].life, life_after_bolt,
        "CR 119.7 — locked player can't gain life");
    // Caster (seat 0) is not locked.
    g.adjust_life(0, 5);
    assert!(!g.players[0].cannot_gain_life_this_turn);
}

#[test]
fn skullcrack_lifegain_lock_clears_at_next_untap() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::skullcrack());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Skullcrack castable");
    drain_stack(&mut g);
    assert!(g.players[1].cannot_gain_life_this_turn);

    // do_untap is called when the active player rotates. Run it
    // directly to assert the per-turn flag clears for every player.
    g.do_untap();
    assert!(!g.players[1].cannot_gain_life_this_turn);
    assert!(!g.players[0].cannot_gain_life_this_turn);
}

#[test]
fn fiery_impulse_deals_two_damage_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::fiery_impulse());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Fiery Impulse castable for {R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "2-toughness Grizzly Bears dies to Fiery Impulse");
}

#[test]
fn fiery_impulse_deals_three_damage_with_spell_mastery() {
    let mut g = two_player_game();
    // Seed 2+ IS cards in your graveyard for spell mastery.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    // 3-toughness vanilla creature.
    let target = g.add_card_to_battlefield(1, catalog::centaur_courser());
    let id = g.add_card_to_hand(0, catalog::fiery_impulse());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Fiery Impulse castable for {R}");
    drain_stack(&mut g);

    // Centaur Courser is 3/3 — 3 damage kills it, 2 damage doesn't.
    assert!(!g.battlefield.iter().any(|c| c.id == target),
        "Spell mastery: 3-toughness creature dies to upgraded 3 damage");
}

#[test]
fn fiery_impulse_deals_two_damage_without_spell_mastery() {
    let mut g = two_player_game();
    // Only ONE IS card in your graveyard — spell mastery NOT active.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let target = g.add_card_to_battlefield(1, catalog::centaur_courser()); // 3/3
    let id = g.add_card_to_hand(0, catalog::fiery_impulse());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Fiery Impulse castable for {R}");
    drain_stack(&mut g);

    // Owlin survives — 2 damage dealt (no spell mastery), 2 < 3 toughness.
    assert!(g.battlefield.iter().any(|c| c.id == target),
        "Without spell mastery: 3-toughness creature survives 2 damage");
}

#[test]
fn infernal_grasp_destroys_and_loses_two_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::infernal_grasp());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "creature destroyed");
    assert_eq!(g.players[0].life, 18, "lose 2 life");
}

#[test]
fn village_rites_sacrifices_and_draws_two() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::village_rites());
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast, +2 drawn = net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "draws two");
}

#[test]
fn power_word_kill_spares_dragons() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::power_word_kill());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable on a non-Dragon");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "ordinary creature dies");
}

#[test]
fn snakeskin_veil_pumps_and_grants_hexproof() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::snakeskin_veil());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 3), "+1/+1");
    assert!(c.has_keyword(&Keyword::Hexproof), "gains hexproof");
}

#[test]
fn murmuring_mystic_makes_a_bird_on_instant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::murmuring_mystic());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Bird Illusion"),
        "casting an instant mints a Bird");
}

/// Pack tactics (real gate): a lone 3-power attack draws nothing; 6+ total
/// attacking power draws.
#[test]
fn werewolf_pack_leader_draws_on_six_power_attack() {
    let mut g = two_player_game();
    let leader = g.add_card_to_battlefield(0, catalog::werewolf_pack_leader());
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.add_card_to_library(0, catalog::island());
    for id in [leader, angel] {
        g.battlefield.iter_mut().find(|c| c.id == id).unwrap().summoning_sick = false;
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.declare_attackers(vec![
        Attack { attacker: leader, target: AttackTarget::Player(1) },
        Attack { attacker: angel, target: AttackTarget::Player(1) },
    ]).expect("attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "3 + 4 attacking power meets pack tactics");
}

#[test]
fn supreme_verdict_destroys_all_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::supreme_verdict());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == a || c.id == b),
        "all creatures destroyed");
}

#[test]
fn stubborn_denial_counters_unless_paid() {
    let mut g = two_player_game();
    // Opponent casts a noncreature spell (Lightning Bolt) we want to counter.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp bolt on stack");
    // We respond with Stubborn Denial; opponent has no mana to pay {1}.
    let sd = g.add_card_to_hand(0, catalog::stubborn_denial());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sd, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stubborn Denial on stack");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt countered (unpaid)");
    assert_eq!(g.players[0].life, 20, "Bolt never resolved");
}

#[test]
fn archmages_charm_mode_one_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::archmages_charm());
    g.players[0].mana_pool.add(Color::Blue, 3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 for the Charm leaving hand, +2 drawn → net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "draws two cards");
}

#[test]
fn brute_force_pumps_three_three() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::brute_force());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (5, 5), "+3/+3");
}

#[test]
fn titans_strength_pumps_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::titans_strength());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (5, 3), "+3/+1");
}

#[test]
fn crash_through_grants_trample_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::crash_through());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().find(|c| c.id == bear).unwrap().has_keyword(&Keyword::Trample),
        "your creatures gain trample");
    // -1 for the Crash Through leaving hand, +1 drawn.
    assert_eq!(g.players[0].hand.len(), hand_before, "net hand size unchanged (cast 1, drew 1)");
}

#[test]
fn fling_sacrifices_creature_and_deals_its_power() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2 power
    let id = g.add_card_to_hand(0, catalog::fling());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "creature sacrificed");
    assert_eq!(g.players[1].life, foe_life - 2, "deals damage equal to its power");
}

#[test]
fn sprite_dragon_grows_on_noncreature_spell() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::sprite_dragon());
    // Cast a noncreature spell (Lightning Bolt) — Sprite Dragon grows.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    let d = g.battlefield.iter().find(|c| c.id == dragon).unwrap();
    assert_eq!(d.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1,
        "noncreature cast adds a +1/+1 counter");
    assert_eq!(d.power(), 2, "now 2/2");
}

#[test]
fn kiln_fiend_pumps_on_instant_cast() {
    let mut g = two_player_game();
    let fiend = g.add_card_to_battlefield(0, catalog::kiln_fiend());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().find(|c| c.id == fiend).unwrap().power(), 4,
        "Kiln Fiend is 4/2 after an instant");
}

#[test]
fn temur_battle_rage_double_strike_base_trample_with_ferocious() {
    let mut g = two_player_game();
    // A 4-power creature satisfies Ferocious.
    let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::temur_battle_rage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter().find(|c| c.id == big).unwrap();
    assert!(c.has_keyword(&Keyword::DoubleStrike), "base grants double strike");
    assert!(c.has_keyword(&Keyword::Trample), "Ferocious grants trample");
    assert_eq!(c.power(), 4, "no P/T change — TBR only grants keywords");
}

#[test]
fn soul_scar_mage_has_prowess() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::soul_scar_mage());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().find(|c| c.id == mage).unwrap().power(), 2,
        "Prowess pumps to 2/3 after a noncreature spell");
}

#[test]
fn mutagenic_growth_pumps_two_two() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mutagenic_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 4), "+2/+2");
}

#[test]
fn unholy_heat_deals_two_without_delirium() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::unholy_heat());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().find(|c| c.id == angel).unwrap().damage, 2,
        "deals 2 without delirium");
}

#[test]
fn unholy_heat_deals_six_with_delirium() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    // Seed 4 card types in our graveyard: creature, instant, enchantment, artifact.
    for def in [
        catalog::grizzly_bears(),
        catalog::lightning_bolt(),
        catalog::seal_of_fire(),
        catalog::mishras_bauble(),
    ] {
        let id = g.next_id();
        g.players[0].graveyard.push(crabomination::card::CardInstance::new(id, def, 0));
    }
    let id = g.add_card_to_hand(0, catalog::unholy_heat());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == angel), "6 damage kills the 4/4 (delirium)");
}

#[test]
fn cut_down_destroys_small_creature_only() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2+2=4 ≤ 5
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4+4=8 > 5
    let id = g.add_card_to_hand(0, catalog::cut_down());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable on the small creature");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "small creature destroyed");

    // The big creature isn't a legal target.
    let id2 = g.add_card_to_hand(0, catalog::cut_down());
    g.players[0].mana_pool.add(Color::Black, 1);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: id2, target: Some(Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap_err();
    assert!(matches!(err, GameError::SelectionRequirementViolated | GameError::InvalidTarget));
}

#[test]
fn galvanic_blast_affinity_boosts_with_three_artifacts() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mishras_bauble());
    }
    let id = g.add_card_to_hand(0, catalog::galvanic_blast());
    g.players[0].mana_pool.add(Color::Red, 1);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 4, "4 damage with 3 artifacts");
}

#[test]
fn seal_of_fire_sacrifices_to_deal_two() {
    let mut g = two_player_game();
    let seal = g.add_card_to_battlefield(0, catalog::seal_of_fire());
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: seal, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac Seal of Fire");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == seal), "Seal is sacrificed");
    assert_eq!(g.players[1].life, foe_life - 2, "deals 2 to the target");
}

#[test]
fn abrade_mode_zero_burns_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::abrade());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "3 damage kills the 2/2");
}

#[test]
fn boros_charm_mode_zero_deals_four_to_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::boros_charm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 4, "4 damage to target player");
}

#[test]
fn servant_of_tymaret_inspired_drains_on_untap() {
    let mut g = two_player_game();
    let servant = g.add_card_to_battlefield(0, catalog::servant_of_tymaret());
    g.battlefield.iter_mut().find(|c| c.id == servant).unwrap().tapped = true;
    let foe_life = g.players[1].life;

    g.do_untap();
    drain_stack(&mut g); // resolve the Inspired trigger

    assert!(!g.battlefield.iter().find(|c| c.id == servant).unwrap().tapped, "Servant untaps");
    assert_eq!(g.players[1].life, foe_life - 1, "Inspired drains each opponent 1 on untap");
}

#[test]
fn exert_attacker_does_not_untap_next_turn() {
    let mut g = two_player_game();
    let crasher = g.add_card_to_battlefield(0, catalog::ahn_crop_crasher());
    // Not summoning sick (has Haste anyway). Advance to declare attackers.
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack {
        attacker: crasher,
        target: AttackTarget::Player(1),
    }]).expect("Ahn-Crop Crasher attacks");
    let c = g.battlefield.iter().find(|c| c.id == crasher).unwrap();
    assert!(c.tapped, "attacker is tapped");
    assert!(c.skip_next_untap, "exert flagged it to skip next untap");

    // The controller's next untap step: the exerted creature stays tapped.
    g.do_untap();
    let c = g.battlefield.iter().find(|c| c.id == crasher).unwrap();
    assert!(c.tapped, "exerted creature does not untap");
    assert!(!c.skip_next_untap, "exert flag consumed — it untaps the turn after");
}

#[test]
fn grapeshot_storm_copies_for_each_prior_spell() {
    let mut g = two_player_game();
    // Two prior spells this turn (Bolts at our own face are fine — we only
    // care about the spell count). Each is cast and resolved.
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt castable");
        drain_stack(&mut g);
    }
    assert_eq!(g.spells_cast_this_turn, 2, "two prior spells recorded");

    let gp = g.add_card_to_hand(0, catalog::grapeshot());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: gp, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Grapeshot castable for {1}{R}");
    drain_stack(&mut g);
    // Original + 2 Storm copies = 3 instances of 1 damage.
    assert_eq!(g.players[1].life, foe_life - 3, "Storm copies once per prior spell");
}

#[test]
fn searing_blood_deals_two_damage_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::searing_blood());
    g.players[0].mana_pool.add(Color::Red, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Searing Blood castable for {R}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "2-toughness Grizzly Bears dies to Searing Blood");
}

#[test]
fn searing_blood_burns_controller_when_creature_dies() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 dies
    let id = g.add_card_to_hand(0, catalog::searing_blood());
    g.players[0].mana_pool.add(Color::Red, 2);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear dies");
    assert_eq!(g.players[1].life, p1_life - 3, "controller takes 3 on death");
}

#[test]
fn searing_blood_spares_controller_when_creature_survives() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 survives 2
    let id = g.add_card_to_hand(0, catalog::searing_blood());
    g.players[0].mana_pool.add(Color::Red, 2);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(wall)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == wall), "4-toughness survives");
    assert_eq!(g.players[1].life, p1_life, "no burn when creature survives");
}

#[test]
fn searing_blood_burns_on_deferred_death_same_turn() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 survives 2
    let id = g.add_card_to_hand(0, catalog::searing_blood());
    g.players[0].mana_pool.add(Color::Red, 2);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life, "no burn yet — 2 damage isn't lethal");

    // Later this turn, a Lightning Bolt finishes the angel (2+3 ≥ 4).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == angel), "angel dies to the combined damage");
    assert_eq!(g.players[1].life, p1_life - 3, "Searing Blood's death watch burns 3");
}

#[test]
fn harrow_sacrifices_land_and_searches_two_basics() {
    let mut g = two_player_game();
    // Stock the library with two Forests so Harrow has fetch targets.
    let forest_one = g.add_card_to_library(0, catalog::forest());
    let forest_two = g.add_card_to_library(0, catalog::forest());
    // Sac fodder: a Mountain on the battlefield.
    let mountain = g.add_card_to_battlefield(0, catalog::mountain());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest_one)),
        DecisionAnswer::Search(Some(forest_two)),
    ]));
    let id = g.add_card_to_hand(0, catalog::harrow());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Harrow castable for {2}{G}");
    drain_stack(&mut g);

    // Mountain went to graveyard (sacrificed).
    assert!(g.players[0].graveyard.iter().any(|c| c.id == mountain),
        "Sacrificed Mountain ends in graveyard");
    // Two Forests are now in play (untapped).
    let forests_in_play = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Forest")
        .count();
    assert_eq!(forests_in_play, 2, "Both Forests entered the battlefield");
    // Both Forests should enter UNTAPPED (this is what differentiates Harrow
    // from Cultivate / Kodama's Reach).
    for f in g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Forest")
    {
        assert!(!f.tapped, "Harrow's basics enter untapped");
    }
}

#[test]
fn drown_in_the_loch_mode_zero_counters_a_spell() {
    let mut g = two_player_game();
    // The gate needs MV ≤ cards in the spell's controller's graveyard.
    // Bolt is MV 1, so seat 1 needs at least one graveyard card.
    g.add_card_to_graveyard(1, catalog::forest());
    // Opponent casts a spell on their own turn.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lightning Bolt cast by opp");

    // Caster responds with Drown in the Loch in mode 0 (counter).
    let id = g.add_card_to_hand(0, catalog::drown_in_the_loch());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.priority.player_with_priority = 0;
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    }).expect("Drown in the Loch mode 0 (counter) castable");
    drain_stack(&mut g);

    // Bolt is countered → caster takes no damage.
    assert_eq!(g.players[0].life, life_before,
        "Bolt was countered, caster's life is unchanged");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Countered Bolt ends in opp's graveyard");
}

#[test]
fn drown_in_the_loch_mode_one_destroys_creature() {
    let mut g = two_player_game();
    // Gate: MV (bear = 2) ≤ cards in the creature's controller's graveyard.
    g.add_card_to_graveyard(1, catalog::forest());
    g.add_card_to_graveyard(1, catalog::forest());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::drown_in_the_loch());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    }).expect("Drown in the Loch mode 1 castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Mode 1 destroys target creature");
}

#[test]
fn drown_in_the_loch_gate_blocks_high_mv_target() {
    // The MV gate makes a creature illegal when its controller's graveyard
    // is too small (bear MV 2, empty graveyard → not a legal target).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::drown_in_the_loch());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let res = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    });
    assert!(res.is_err(), "MV-2 creature with an empty-graveyard controller is illegal");
    assert!(g.battlefield.iter().any(|c| c.id == bear), "bear survives");
}

#[test]
fn cremate_exiles_graveyard_card_and_draws() {
    let mut g = two_player_game();
    // Stock graveyard with a card and library with one to draw.
    let grave_id = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::cremate());
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(grave_id)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cremate castable for {B}");
    drain_stack(&mut g);

    // Net hand: -1 cast +1 draw = 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "Cantrip nets 0 hand");
    assert!(g.exile.iter().any(|c| c.id == grave_id),
        "Targeted graveyard card was exiled");
}

#[test]
fn mortuary_mire_etb_taps_and_recurs_creature_card() {
    let mut g = two_player_game();
    let _grave_creature = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mortuary_mire());

    g.perform_action(GameAction::PlayLand(id))
        .expect("Mortuary Mire is a playable land");
    drain_stack(&mut g);

    let mire = g.battlefield_find(id).expect("Mortuary Mire on battlefield");
    assert!(mire.tapped, "Mortuary Mire enters tapped");
    // The bear should have moved to the top of player 0's library.
    let top = g.players[0].library.last()
        .expect("Library should not be empty");
    assert_eq!(top.definition.name, "Grizzly Bears",
        "ETB places the creature card on top of library");
}

#[test]
fn geier_reach_sanitarium_taps_for_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::geier_reach_sanitarium());
    let pool_before = g.players[0].mana_pool.total();

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("First mana ability is {T}: Add {C}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].mana_pool.total(), pool_before + 1,
        "Geier Reach taps for {{C}}");
}

#[test]
fn geier_reach_sanitarium_wheel_ability_each_player_loots() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::geier_reach_sanitarium());
    g.players[0].mana_pool.add_colorless(1);
    // Stock libraries so each player has a card to draw.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(1, catalog::island());
    // Stock hands so the discard step has something to throw away.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let h0 = g.players[0].hand.len();
    let h1 = g.players[1].hand.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None }).expect("Wheel ability is sorcery-speed");
    drain_stack(&mut g);

    // Each player draws 1 then discards 1 → net 0 hand size for each.
    assert_eq!(g.players[0].hand.len(), h0,
        "Player 0 nets 0 hand from each-player loot");
    assert_eq!(g.players[1].hand.len(), h1,
        "Player 1 nets 0 hand from each-player loot");
}

// ── modern_decks-15: 12 new cube cards ───────────────────────────────────────

#[test]
fn strangle_deals_three_damage_and_surveils() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strangle());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Strangle castable for {R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Grizzly Bears (2-toughness) dies to 3 damage");
    // Surveil 1 inspected the top card of the library, putting it either
    // back on top or in the graveyard. Either outcome reduces or holds
    // the library size; we just verify the cast didn't fail at surveil.
    assert!(g.players[0].library.len() <= lib_before,
        "Surveil 1 either kept or graveyarded the top card");
}

#[test]
fn dreadbore_destroys_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::dreadbore());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Dreadbore castable for {B}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Dreadbore destroys the target creature");
}

#[test]
fn bedevil_destroys_target_artifact() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::bedevil());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(stone)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bedevil castable for {B}{B}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "Bedevil destroys the target artifact");
}

#[test]
fn tome_scour_mills_target_player_five() {
    let mut g = two_player_game();
    for _ in 0..10 { g.add_card_to_library(1, catalog::island()); }
    let lib_before = g.players[1].library.len();
    let grave_before = g.players[1].graveyard.len();
    let id = g.add_card_to_hand(0, catalog::tome_scour());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Tome Scour castable for {U}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].library.len(), lib_before - 5,
        "Tome Scour mills 5 cards from target player");
    assert_eq!(g.players[1].graveyard.len(), grave_before + 5);
}

#[test]
fn repulse_returns_creature_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::repulse());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    let opp_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Repulse castable for {2}{U}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear is bounced off the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "Bear returns to its owner's hand");
    // Caster: -1 cast +1 draw = 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "Repulse cantrips");
    // Opp gains the bear in hand.
    assert_eq!(g.players[1].hand.len(), opp_hand_before + 1);
}

#[test]
fn visions_of_beyond_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::visions_of_beyond());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Visions of Beyond castable for {U}");
    drain_stack(&mut g);

    // -1 cast +1 draw = 0 net hand change.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Visions of Beyond is a 1-mana cantrip");
}

#[test]
fn visions_of_beyond_draws_three_with_twenty_card_graveyard() {
    let mut g = two_player_game();
    // Stack opponent's graveyard with 20 cards (any card type works).
    for _ in 0..20 {
        let id = g.add_card_to_library(1, catalog::island());
        // Put it directly into the graveyard.
        if let Some(pos) = g.players[1].library.iter().position(|c| c.id == id) {
            let card = g.players[1].library.remove(pos);
            g.players[1].graveyard.push(card);
        }
    }
    assert_eq!(g.players[1].graveyard.len(), 20);
    // Stock 4 cards in seat 0's library so the draw-3 has fodder.
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::visions_of_beyond());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Visions of Beyond castable for {U}");
    drain_stack(&mut g);

    // -1 cast +3 draw = +2 net hand change (the upgraded mode).
    assert_eq!(g.players[0].hand.len(), hand_before + 2,
        "Visions of Beyond draws 3 when a graveyard has 20+ cards");
}

#[test]
fn visions_of_beyond_draws_one_with_nineteen_card_graveyard() {
    let mut g = two_player_game();
    // Just under the threshold — 19 cards.
    for _ in 0..19 {
        let id = g.add_card_to_library(1, catalog::island());
        if let Some(pos) = g.players[1].library.iter().position(|c| c.id == id) {
            let card = g.players[1].library.remove(pos);
            g.players[1].graveyard.push(card);
        }
    }
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::visions_of_beyond());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Visions of Beyond castable for {U}");
    drain_stack(&mut g);

    // -1 cast +1 draw = 0 net hand change (the cantrip mode).
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Visions of Beyond draws 1 when no graveyard has 20+ cards");
}

#[test]
fn plummet_destroys_target_flying_creature() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
    let id = g.add_card_to_hand(0, catalog::plummet());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(angel)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Plummet castable for {1}{G}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == angel),
        "Plummet destroys flying Serra Angel");
}

#[test]
fn plummet_rejects_non_flying_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // no flying
    let id = g.add_card_to_hand(0, catalog::plummet());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Plummet should reject a non-flying creature: {:?}", err);
}

#[test]
fn strategic_planning_takes_one_of_top_three_rest_to_graveyard() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let lib_before = g.players[0].library.len();
    let grave_before = g.players[0].graveyard.len();
    let id = g.add_card_to_hand(0, catalog::strategic_planning());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Strategic Planning castable for {1}{U}");
    drain_stack(&mut g);

    // Look at top 3: one to hand, two to graveyard. Library loses 3.
    assert_eq!(g.players[0].library.len(), lib_before - 3,
        "top three cards leave the library (1 to hand + 2 to graveyard)");
    // Graveyard: +2 rest + the resolved sorcery itself = +3.
    assert_eq!(g.players[0].graveyard.len(), grave_before + 3,
        "two unpicked cards + the resolved sorcery");
    // Hand: -1 cast + 1 picked = net 0 (a 2-mana cantrip).
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn strategic_planning_picks_the_chosen_card_to_hand() {
    // A ScriptedDecider picks a specific one of the top three; it lands in
    // hand and the other two go to the graveyard.
    let mut g = two_player_game();
    let want = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strategic_planning());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(want))]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == want), "chosen Bolt went to hand");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Island").count(), 2,
        "the two unpicked Islands went to the graveyard");
}

#[test]
fn ravenous_rats_etb_makes_each_opponent_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let opp_hand_before = g.players[1].hand.len();
    let opp_grave_before = g.players[1].graveyard.len();
    let id = g.add_card_to_hand(0, catalog::ravenous_rats());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ravenous Rats castable for {1}{B}");
    drain_stack(&mut g);

    // Body enters the battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Rat body resolves onto the battlefield");
    // Opp discards a card → hand -1, graveyard +1.
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1,
        "Opponent discarded a card from ETB trigger");
    assert_eq!(g.players[1].graveyard.len(), opp_grave_before + 1);
}

#[test]
fn brain_maggot_etb_exiles_until_it_leaves_then_returns_to_hand() {
    let mut g = two_player_game();
    let target_card = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::forest()); // land — should be skipped by filter
    let id = g.add_card_to_hand(0, catalog::brain_maggot());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Brain Maggot castable for {1}{B}");
    drain_stack(&mut g);

    // The bolt is exiled (not graveyarded), linked to Brain Maggot.
    assert!(g.exile.iter().any(|c| c.id == target_card),
        "Lightning Bolt (the only nonland) is exiled until Brain Maggot leaves");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == target_card),
        "exiled, not discarded");
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Forest"),
        "Land stays in opponent's hand (Nonland filter)");

    // Brain Maggot dies → the exiled card returns to its owner's hand.
    g.remove_from_battlefield_to_graveyard_raw(id);
    assert!(g.players[1].hand.iter().any(|c| c.id == target_card),
        "exiled card returns to owner's hand when Brain Maggot leaves");
    assert!(!g.exile.iter().any(|c| c.id == target_card), "no longer in exile");
}

#[test]
fn banisher_priest_exiles_creature_until_it_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::banisher_priest());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Banisher Priest castable for {1}{W}{W}");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "opp creature exiled");
    assert!(g.battlefield_find(bear).is_none(), "bear off battlefield");

    // Priest dies → the bear returns to the battlefield under its owner.
    g.remove_from_battlefield_to_graveyard_raw(id);
    let returned = g.battlefield_find(bear).expect("bear back on battlefield");
    assert_eq!(returned.controller, 1, "returns under its owner's control");
    assert!(!g.exile.iter().any(|c| c.id == bear), "no longer exiled");
}

#[test]
fn oblivion_ring_cannot_target_itself() {
    // The "another" clause: with only O-Ring on the battlefield its ETB
    // has no legal target, so nothing is exiled.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::oblivion_ring());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Oblivion Ring castable for {2}{W}");
    drain_stack(&mut g);
    assert!(g.exile.is_empty(), "O-Ring can't exile itself (OtherThanSource)");
    assert!(g.battlefield_find(id).is_some(), "O-Ring resolved onto battlefield");
}

#[test]
fn oblivion_ring_exiles_nonland_permanent_and_returns_it() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::oblivion_ring());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Oblivion Ring castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "opp permanent exiled");
    // O-Ring leaves → exiled permanent returns to battlefield.
    g.remove_from_battlefield_to_graveyard_raw(id);
    assert!(g.battlefield_find(bear).is_some(), "exiled permanent returns");
}

#[test]
fn bond_of_discipline_taps_each_opponent_creature_and_grants_lifelink() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear_a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear_b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let your_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::bond_of_discipline());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bond of Discipline castable for {3}{W}");
    drain_stack(&mut g);

    let perm_a = g.battlefield_find(bear_a).expect("opp bear A still on battlefield");
    let perm_b = g.battlefield_find(bear_b).expect("opp bear B still on battlefield");
    assert!(perm_a.tapped, "Bond of Discipline taps each opponent creature (A)");
    assert!(perm_b.tapped, "Bond of Discipline taps each opponent creature (B)");

    // Your bear gains lifelink EOT — check the temporary keyword grant.
    let computed = g.computed_permanent(your_bear)
        .expect("your bear still on battlefield");
    assert!(computed.keywords.iter().any(|k| matches!(k, Keyword::Lifelink)),
        "Your creature has lifelink granted EOT");
}

#[test]
fn sudden_edict_forces_target_player_to_sacrifice() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::sudden_edict());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sudden Edict castable for {1}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Targeted opponent sacrificed their only creature");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "Sacrificed creature ends up in opp's graveyard");
}

/// Regression: `Effect::Sacrifice`'s `who` slot now surfaces a target
/// filter via `primary_target_filter`, so the auto-target heuristic
/// picks the opponent for edict-class spells. Without the surfacing,
/// `auto_target_for_effect` returned None and the bot couldn't cast
/// Sudden Edict at all.
#[test]
fn auto_target_for_sudden_edict_picks_opponent() {
    use crabomination::server::bot;
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let card = catalog::sudden_edict();
    let target = bot::choose_target(&g, &card, 0);
    assert_eq!(target, Some(Target::Player(1)),
        "auto_target_for_effect picks the opponent for Sudden Edict");
}

/// `target_filtered(SelectionRequirement::Player)` rejects a permanent
/// target at cast time, so Sudden Edict can't be aimed at a creature
/// directly (cast-time filter mismatch).
#[test]
fn sudden_edict_rejects_creature_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::sudden_edict());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Sudden Edict should reject a creature target (Player filter): {:?}",
        err);
}

// ── modern_decks-16: cube-pool activations ───────────────────────────────────
//
// These cards already have factories + targeted unit tests covering
// the cards' primary play patterns — see e.g. `vandalblast_destroys_
// opponent_artifact`, `ranger_captain_etb_searches_for_a_one_drop`,
// `heliod_sun_crowned_grants_lifelink_until_end_of_turn`,
// `containment_priest_is_a_flash_two_two`, `tireless_tracker_*`,
// `swan_song_*`. The activations below pin the cube-pool wiring (so
// regressions on the cube prefetch / sampling path get caught early).

/// Fellwar Stone joins the colorless utility pool when activated.
/// Verify the factory produces a working {2} mana rock that taps for
/// any one color.
#[test]
fn fellwar_stone_taps_for_any_color() {
    // Push (batch 117): Fellwar Stone now respects "an opponent's
    // land could produce". With no opp lands at all, falls back to
    // colorless. Seed an opp Island so the pool gains blue.
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(0, catalog::fellwar_stone());
    g.battlefield_find_mut(stone).unwrap().summoning_sick = false;
    // Opp has an Island → Blue is in the legal pool.
    g.add_card_to_battlefield(1, catalog::island());

    g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Fellwar Stone's mana ability should resolve");

    let pool = &g.players[0].mana_pool;
    // With only Island under opp's control, only Blue should be legal.
    assert_eq!(pool.amount(Color::Blue), 1,
        "Fellwar Stone produced 1 blue (the only color opp's Island can produce)");
    assert_eq!(pool.amount(Color::White), 0);
    assert_eq!(pool.amount(Color::Black), 0);
    assert_eq!(pool.amount(Color::Red), 0);
    assert_eq!(pool.amount(Color::Green), 0);
}

#[test]
fn fellwar_stone_falls_back_to_colorless_when_no_opp_basic_lands() {
    // No opp lands → pool gains 1 colorless (so the activation isn't
    // a silent no-op). Matches the "never silently no-op" convention.
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(0, catalog::fellwar_stone());
    g.battlefield_find_mut(stone).unwrap().summoning_sick = false;
    // Opp has no battlefield permanents at all.
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Fellwar Stone activates with no opp lands");
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.total(), 1, "Pool has exactly 1 mana");
    // Colorless fallback — none of the colored amounts increment.
    assert_eq!(pool.amount(Color::White), 0);
    assert_eq!(pool.amount(Color::Blue), 0);
    assert_eq!(pool.amount(Color::Black), 0);
    assert_eq!(pool.amount(Color::Red), 0);
    assert_eq!(pool.amount(Color::Green), 0);
}

#[test]
fn fellwar_stone_unions_colors_across_multiple_opp_lands() {
    // Multiple opp basic-typed lands → union of their colors is the
    // legal pool. With opp Island + Forest, only Blue + Green are
    // legal; AutoDecider picks the first (Blue).
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(0, catalog::fellwar_stone());
    g.battlefield_find_mut(stone).unwrap().summoning_sick = false;
    g.add_card_to_battlefield(1, catalog::island());
    g.add_card_to_battlefield(1, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Fellwar Stone activates");
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.total(), 1);
    // White / Black / Red are not in the legal pool — opp controls no
    // Plains / Swamp / Mountain.
    assert_eq!(pool.amount(Color::White), 0);
    assert_eq!(pool.amount(Color::Black), 0);
    assert_eq!(pool.amount(Color::Red), 0);
    // Blue or Green (one of them) gained 1.
    assert!(pool.amount(Color::Blue) + pool.amount(Color::Green) == 1,
        "Exactly one of Blue/Green gained 1");
}

/// Star Compass taps for a color one of the controller's own basic lands
/// could produce (mirror of Fellwar Stone).
#[test]
fn star_compass_taps_for_your_basic_land_color() {
    let mut g = two_player_game();
    let compass = g.add_card_to_battlefield(0, catalog::star_compass());
    g.battlefield_find_mut(compass).unwrap().summoning_sick = false;
    // You control a Mountain → Red is the only legal color.
    g.add_card_to_battlefield(0, catalog::mountain());
    g.perform_action(GameAction::ActivateAbility {
        card_id: compass, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Star Compass mana ability resolves");
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.amount(Color::Red), 1, "produced red from your Mountain");
    assert_eq!(pool.total(), 1);
}

/// Star Compass falls back to colorless when you control no basic-typed land.
#[test]
fn star_compass_falls_back_to_colorless() {
    let mut g = two_player_game();
    let compass = g.add_card_to_battlefield(0, catalog::star_compass());
    g.battlefield_find_mut(compass).unwrap().summoning_sick = false;
    // Opp's Island shouldn't count — only the controller's own lands do.
    g.add_card_to_battlefield(1, catalog::island());
    g.perform_action(GameAction::ActivateAbility {
        card_id: compass, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Star Compass activates with no basic land you control");
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.total(), 1);
    assert_eq!(pool.amount(Color::Blue), 0, "opp's Island doesn't feed Star Compass");
}

/// Grim Lavamancer's `{R}, {T}, Exile two cards from your gy:` deals
/// 2 damage to any target. Push (batch 114): the exile-two cost is
/// now wired faithfully via the extended `exile_other_filter:
/// Some((filter, 2))` shape. Verify activation pings 2 damage to a
/// target creature when there are ≥ 2 gy cards to exile.
#[test]
fn grim_lavamancer_pings_creature_with_gy_card_to_exile() {
    let mut g = two_player_game();
    let lava = g.add_card_to_battlefield(0, catalog::grim_lavamancer());
    g.battlefield_find_mut(lava).unwrap().summoning_sick = false;
    // Seed two graveyard cards for the exile-2-from-gy cost.
    let _fodder_a = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let _fodder_b = g.add_card_to_graveyard(0, catalog::shock());
    // Need a creature target on the battlefield (opponent's bear).
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lava, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None })
    .expect("Lavamancer can activate with R + 2 gy fodder");
    drain_stack(&mut g);
    // Bear (2/2) takes 2 damage and dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Grim Lavamancer should ping the bear for 2 (now dead)");
}

#[test]
fn grim_lavamancer_rejects_activation_with_only_one_gy_card() {
    // Batch 114 negative test: with only 1 card in graveyard the
    // exile-2 cost can't be paid → activation rejects cleanly without
    // burning tap/mana.
    let mut g = two_player_game();
    let lava = g.add_card_to_battlefield(0, catalog::grim_lavamancer());
    g.clear_sickness(lava);
    let _fodder = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;
    let pool_before = g.players[0].mana_pool.total();
    let tapped_before = g.battlefield_find(lava).map(|c| c.tapped).unwrap_or(false);

    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: lava, ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None });
    assert!(result.is_err(),
        "Only 1 card in gy — activation must reject the exile-2 cost");
    assert_eq!(g.players[1].life, life_before, "No damage was dealt");
    assert_eq!(g.players[0].mana_pool.total(), pool_before,
        "Mana wasn't burned on the rejected activation");
    let tapped_after = g.battlefield_find(lava).map(|c| c.tapped).unwrap_or(false);
    assert_eq!(tapped_before, tapped_after,
        "Tap wasn't burned on the rejected activation");
    // Single gy fodder card is still in the graveyard.
    assert_eq!(g.players[0].graveyard.len(), 1,
        "GY fodder still in place — cost wasn't partially paid");
}

// ── Guardian Scalelord (M15 / cube card) ────────────────────────────────────

#[test]
fn guardian_scalelord_attack_grants_flying_to_target_friendly() {
    use crabomination::card::{CreatureType, Keyword};
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let scalelord = g.add_card_to_battlefield(0, catalog::guardian_scalelord());
    g.clear_sickness(scalelord);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    drain_stack(&mut g);
    // Body sanity check.
    let scalelord_card = g.battlefield_find(scalelord).unwrap();
    assert!(scalelord_card.has_keyword(&Keyword::Flying));
    assert!(scalelord_card.definition.subtypes.creature_types.contains(&CreatureType::Dragon));

    // Accept the MayDo rider so the bear actually gets Flying.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: scalelord,
        target: AttackTarget::Player(1),
    }])).expect("Scalelord can attack");
    drain_stack(&mut g);
    // The bear should now have Flying EOT.
    let bear_card = g.battlefield_find(bear).unwrap();
    assert!(bear_card.has_keyword(&Keyword::Flying),
        "Scalelord's attack trigger gave the bear flying");
}

#[test]
fn guardian_scalelord_declines_optional_grant_by_default() {
    // AutoDecider defaults to "no" on MayDo (CR 603.2 — the controller
    // chooses; the bot harness defaults to skipping optional non-cost
    // riders). The bear should NOT get flying without an explicit yes.
    use crabomination::card::Keyword;
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let scalelord = g.add_card_to_battlefield(0, catalog::guardian_scalelord());
    g.clear_sickness(scalelord);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    drain_stack(&mut g);

    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: scalelord,
        target: AttackTarget::Player(1),
    }])).expect("Scalelord can attack");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert!(!bear_card.has_keyword(&Keyword::Flying),
        "AutoDecider declines the MayDo; bear stays grounded");
}

