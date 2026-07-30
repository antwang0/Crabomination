//! Mirrodin gap batch (`decks::recent316`).

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// Swing `attacker` (seat 0's) into seat 1 and run combat out.
fn swing(g: &mut GameState, attacker: crabomination::card::CardId) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

/// The Myr cycle taps for its colour.
#[test]
fn myr_cycle_taps_for_its_color() {
    let mut g = main_phase();
    for (factory, color) in [
        (catalog::copper_myr as fn() -> _, Color::Green),
        (catalog::silver_myr, Color::Blue),
        (catalog::iron_myr, Color::Red),
        (catalog::leaden_myr, Color::Black),
    ] {
        let myr = g.add_card_to_battlefield(0, factory());
        g.clear_sickness(myr);
        g.perform_action(GameAction::ActivateAbility {
            card_id: myr, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None,
        })
        .expect("tap for mana");
        assert_eq!(g.players[0].mana_pool.amount(color), 1, "{color:?}");
        g.players[0].mana_pool.empty();
    }
}

/// A Slith grows permanently off combat damage.
#[test]
fn slith_firewalker_grows_on_combat_damage() {
    let mut g = main_phase();
    let slith = g.add_card_to_battlefield(0, catalog::slith_firewalker());
    swing(&mut g, slith);
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.battlefield_find(slith).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let cp = g.computed_permanent(slith).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// Slith Strider draws when it becomes blocked.
#[test]
fn slith_strider_draws_when_blocked() {
    let mut g = main_phase();
    let slith = g.add_card_to_battlefield(0, catalog::slith_strider());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(slith);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: slith, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, slith)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Cobalt Golem buys flying for the turn.
#[test]
fn cobalt_golem_buys_flying() {
    let mut g = main_phase();
    let golem = g.add_card_to_battlefield(0, catalog::cobalt_golem());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: golem, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(golem).unwrap().keywords.contains(&Keyword::Flying));
}

/// Grid Monitor locks its controller out of creature spells but not others.
#[test]
fn grid_monitor_locks_creature_spells() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grid_monitor());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "creature spells are locked",
    );
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("noncreature spells are fine");
}

/// Leonin Abunas shields your artifacts from opposing removal.
#[test]
fn leonin_abunas_grants_artifact_hexproof() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::leonin_abunas());
    let rock = g.add_card_to_battlefield(0, catalog::tanglebloom());
    assert!(g.computed_permanent(rock).unwrap().keywords.contains(&Keyword::Hexproof));
    let naturalize = g.add_card_to_hand(1, catalog::goblin_replica());
    let _ = naturalize;
    assert!(g.check_target_legality(&Target::Permanent(rock), 1).is_err(), "hexproof holds");
}

/// Leonin Elder pays out for any artifact entering, on either side.
#[test]
fn leonin_elder_gains_life_off_any_artifact() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::leonin_elder());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let bloom = g.add_card_to_hand(1, catalog::tanglebloom());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bloom, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("they cast an artifact");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21);
}

/// Loxodon Punisher scales with the Equipment strapped to it.
#[test]
fn loxodon_punisher_scales_per_equipment() {
    let mut g = main_phase();
    let punisher = g.add_card_to_battlefield(0, catalog::loxodon_punisher());
    let cp = g.computed_permanent(punisher).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "bare");
    let armor = g.add_card_to_battlefield(0, catalog::slagwurm_armor());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: armor, target: punisher }).expect("equip");
    let cp = g.computed_permanent(punisher).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 10), "+2/+2 from the rider, +0/+6 from the Armor");
}

/// Leonin Den-Guard only wakes up once it's holding something.
#[test]
fn leonin_den_guard_needs_equipment() {
    let mut g = main_phase();
    let guard = g.add_card_to_battlefield(0, catalog::leonin_den_guard());
    let cp = g.computed_permanent(guard).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 3));
    assert!(!cp.keywords.contains(&Keyword::Vigilance));
    let gear = g.add_card_to_battlefield(0, catalog::vulshok_battlegear());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: gear, target: guard }).expect("equip");
    let cp = g.computed_permanent(guard).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 7), "+1/+1 self, +3/+3 gear");
    assert!(cp.keywords.contains(&Keyword::Vigilance));
}

/// Empyrial Plate reads the equipped creature's controller's hand.
#[test]
fn empyrial_plate_scales_with_hand_size() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let plate = g.add_card_to_battlefield(0, catalog::empyrial_plate());
    g.players[0].hand.clear();
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: plate, target: bear }).expect("equip");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "empty hand, no bonus");
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Vulshok Gauntlets trade untapping for a big body.
#[test]
fn vulshok_gauntlets_lock_untap() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gauntlets = g.add_card_to_battlefield(0, catalog::vulshok_gauntlets());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: gauntlets, target: bear }).expect("equip");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 4));
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.step = TurnStep::End;
    let _ = g.advance_step(Vec::new());
    while g.step != TurnStep::Upkeep {
        let _ = g.advance_step(Vec::new());
    }
    assert!(g.battlefield_find(bear).unwrap().tapped, "it never untaps");
}

/// Viridian Longbow turns its host into a pinger.
#[test]
fn viridian_longbow_grants_a_ping() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bow = g.add_card_to_battlefield(0, catalog::viridian_longbow());
    g.clear_sickness(bear);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: bow, target: bear }).expect("equip");
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    })
    .expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
}

/// Viridian Joiner scales its mana with its power.
#[test]
fn viridian_joiner_taps_for_its_power() {
    let mut g = main_phase();
    let joiner = g.add_card_to_battlefield(0, catalog::viridian_joiner());
    g.clear_sickness(joiner);
    g.battlefield_find_mut(joiner).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: joiner, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("tap");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3);
}

/// Vedalken Archmage draws off artifact casts only.
#[test]
fn vedalken_archmage_draws_on_artifact_casts() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::vedalken_archmage());
    let bloom = g.add_card_to_hand(0, catalog::tanglebloom());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bloom, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast an artifact");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "-1 cast +1 draw");
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast a Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1, "no draw off a nonartifact");
}

/// Wizard Replica taxes a spell out of existence.
#[test]
fn wizard_replica_counters_unless_paid() {
    let mut g = main_phase();
    let replica = g.add_card_to_battlefield(0, catalog::wizard_replica());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("they Bolt you");
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: replica, ability_index: 0, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], x_value: None,
    })
    .expect("sac the Replica");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "they had no {{2}} to pay");
    assert!(g.battlefield_find(replica).is_none(), "the Replica was the cost");
}

/// Rustspore Ram eats an Equipment on the way in.
#[test]
fn rustspore_ram_destroys_equipment() {
    let mut g = main_phase();
    let gear = g.add_card_to_battlefield(1, catalog::vulshok_battlegear());
    let ram = g.add_card_to_hand(0, catalog::rustspore_ram());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: ram, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast the Ram");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gear).is_none());
}

/// Irradiate scales off your artifact count.
#[test]
fn irradiate_shrinks_by_artifact_count() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::tanglebloom());
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::irradiate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "-3/-3 kills a 2/2");
}

/// Deconstruct refunds three green.
#[test]
fn deconstruct_destroys_and_refunds() {
    let mut g = main_phase();
    let rock = g.add_card_to_battlefield(1, catalog::tanglebloom());
    let spell = g.add_card_to_hand(0, catalog::deconstruct());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(rock)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none());
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3);
}

/// Fabricate tutors an artifact to hand.
#[test]
fn fabricate_finds_an_artifact() {
    let mut g = main_phase();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let rock = g.add_card_to_library(0, catalog::tanglebloom());
    let spell = g.add_card_to_hand(0, catalog::fabricate());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(rock)),
    ]));
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == rock));
}

/// Tempest of Light sweeps every enchantment, both sides.
#[test]
fn tempest_of_light_destroys_all_enchantments() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::sphere_of_safety());
    let theirs = g.add_card_to_battlefield(1, catalog::sphere_of_safety());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::tempest_of_light());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none());
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(bear).is_some(), "creatures are untouched");
}

/// Barter in Blood taxes both players two creatures.
#[test]
fn barter_in_blood_hits_every_player() {
    let mut g = main_phase();
    for seat in 0..2 {
        for _ in 0..3 {
            g.add_card_to_battlefield(seat, catalog::grizzly_bears());
        }
    }
    let spell = g.add_card_to_hand(0, catalog::barter_in_blood());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    for seat in 0..2 {
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == seat).count(),
            1,
            "seat {seat} sacrificed two",
        );
    }
}

/// Tel-Jilad Chosen can't be targeted by an artifact source's ability.
#[test]
fn tel_jilad_chosen_has_protection_from_artifacts() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::tel_jilad_chosen());
    assert!(
        g.computed_permanent(elf)
            .unwrap()
            .keywords
            .contains(&Keyword::ProtectionFromCardType(CardType::Artifact))
    );
    let tower = g.add_card_to_battlefield(0, catalog::tower_of_champions());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(tower);
    g.players[0].mana_pool.add_colorless(16);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: tower, ability_index: 0, target: Some(Target::Permanent(elf)),
            additional_targets: vec![], x_value: None,
        })
        .is_err(),
        "an artifact source can't target it",
    );
    g.perform_action(GameAction::ActivateAbility {
        card_id: tower, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    })
    .expect("but an ordinary creature is fine");
}

/// Krark's Thumb hands its controller the coin-flip advantage (CR 705.3).
#[test]
fn krarks_thumb_gives_flip_advantage() {
    let mut g = main_phase();
    assert_eq!(g.coin_flip_advantage_now(0), 0);
    g.add_card_to_battlefield(0, catalog::krarks_thumb());
    assert!(g.coin_flip_advantage_now(0) > 0);
    assert_eq!(g.coin_flip_advantage_now(1), 0, "only its controller");
}

/// Tel-Jilad Stylus bottoms a permanent you own.
#[test]
fn tel_jilad_stylus_bottoms_your_permanent() {
    let mut g = main_phase();
    let stylus = g.add_card_to_battlefield(0, catalog::tel_jilad_stylus());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(stylus);
    g.perform_action(GameAction::ActivateAbility {
        card_id: stylus, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bear));
}

/// Woebearer's combat damage may reanimate a creature card to hand.
#[test]
fn woebearer_returns_a_creature_to_hand() {
    let mut g = main_phase();
    let woe = g.add_card_to_battlefield(0, catalog::woebearer());
    let buried = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Target(Target::Permanent(buried)),
    ]));
    swing(&mut g, woe);
    assert!(g.players[0].hand.iter().any(|c| c.id == buried));
}

// ── Mirrodin gap batch 2 (`decks::recent317`) ──

/// The Nim scale with your artifact count.
#[test]
fn nim_lasher_grows_per_artifact() {
    let mut g = main_phase();
    let nim = g.add_card_to_battlefield(0, catalog::nim_lasher());
    let cp = g.computed_permanent(nim).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "no artifacts yet");
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::tanglebloom());
    }
    let cp = g.computed_permanent(nim).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 1), "+1/+0 each");
}

/// Nim Shambler eats a creature to regenerate.
#[test]
fn nim_shambler_regenerates_by_sacrifice() {
    let mut g = main_phase();
    let shambler = g.add_card_to_battlefield(0, catalog::nim_shambler());
    let chump = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: shambler, ability_index: 0, target: None, additional_targets: vec![],
        x_value: None,
    })
    .expect("sacrifice the Bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(chump).is_none());
    assert!(g.battlefield_find(shambler).unwrap().regeneration_shields > 0);
}

/// Myr Adapter tracks the Equipment strapped to it.
#[test]
fn myr_adapter_scales_per_equipment() {
    let mut g = main_phase();
    let myr = g.add_card_to_battlefield(0, catalog::myr_adapter());
    let gear = g.add_card_to_battlefield(0, catalog::vulshok_battlegear());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: gear, target: myr }).expect("equip");
    let cp = g.computed_permanent(myr).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "+1/+1 rider, +3/+3 gear");
}

/// The Shard cycle offers both a generic and a coloured activation.
#[test]
fn granite_shard_fires_off_either_cost() {
    let mut g = main_phase();
    let shard = g.add_card_to_battlefield(0, catalog::granite_shard());
    g.clear_sickness(shard);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shard, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    })
    .expect("{3} half");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
    g.battlefield_find_mut(shard).unwrap().tapped = false;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shard, ability_index: 1, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    })
    .expect("{R} half");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

/// Serum Tank charges off every artifact and cashes in for a card.
#[test]
fn serum_tank_charges_then_draws() {
    let mut g = main_phase();
    let tank = g.add_card_to_battlefield(0, catalog::serum_tank());
    g.clear_sickness(tank);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let bloom = g.add_card_to_hand(0, catalog::tanglebloom());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bloom, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast an artifact");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(tank).unwrap().counter_count(CounterType::Charge), 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: tank, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("cash in");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.battlefield_find(tank).unwrap().counter_count(CounterType::Charge), 0);
}

/// Affinity for artifacts discounts the Chiss-Goria relics.
#[test]
fn scale_of_chiss_goria_has_affinity() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::tanglebloom());
    }
    let scale = g.add_card_to_hand(0, catalog::scale_of_chiss_goria());
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: scale, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "{{3}} minus two artifacts is still {{1}}",
    );
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: scale, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("two artifacts shave {2}");
}

/// Mass Hysteria hands haste to everything, both sides.
#[test]
fn mass_hysteria_grants_haste_globally() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::mass_hysteria());
    for id in [mine, theirs] {
        assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Haste));
    }
}

/// Necrogen Mists taxes the active player each upkeep.
#[test]
fn necrogen_mists_discards_at_each_upkeep() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::necrogen_mists());
    g.players[1].hand.clear();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.step, TurnStep::Upkeep);
    assert_eq!(g.players[1].hand.len(), 0, "they pitched their card");
}

/// Molder Slug taxes the active player an artifact each upkeep.
#[test]
fn molder_slug_eats_an_artifact_each_upkeep() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::molder_slug());
    let rock = g.add_card_to_battlefield(1, catalog::tanglebloom());
    g.active_player_idx = 1;
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none());
}

/// Megatog trades an artifact for a big trampling turn.
#[test]
fn megatog_eats_an_artifact_for_a_pump() {
    let mut g = main_phase();
    let tog = g.add_card_to_battlefield(0, catalog::megatog());
    let rock = g.add_card_to_battlefield(0, catalog::tanglebloom());
    g.perform_action(GameAction::ActivateAbility {
        card_id: tog, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("eat the rock");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none());
    let cp = g.computed_permanent(tog).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 7));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Krark-Clan Shaman sweeps the ground but spares fliers.
#[test]
fn krark_clan_shaman_spares_fliers() {
    let mut g = main_phase();
    let shaman = g.add_card_to_battlefield(0, catalog::krark_clan_shaman());
    g.add_card_to_battlefield(0, catalog::tanglebloom());
    let ground = g.add_card_to_battlefield(1, catalog::ornithopter());
    let flier = g.add_card_to_battlefield(1, catalog::wizard_replica());
    // Ornithopter flies; use a 1-toughness ground body instead.
    let _ = ground;
    let grounded = g.add_card_to_battlefield(1, catalog::nim_replica());
    g.perform_action(GameAction::ActivateAbility {
        card_id: shaman, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("sweep");
    drain_stack(&mut g);
    assert!(g.battlefield_find(grounded).is_none(), "the 3/1 ground body dies");
    assert!(g.battlefield_find(flier).is_some(), "the flier is spared");
    assert!(g.battlefield_find(shaman).is_none(), "the 1/1 Shaman shoots itself too");
}

/// Ogre Leadfoot punishes an artifact creature that blocks it.
#[test]
fn ogre_leadfoot_kills_an_artifact_blocker() {
    let mut g = main_phase();
    let ogre = g.add_card_to_battlefield(0, catalog::ogre_leadfoot());
    let robot = g.add_card_to_battlefield(1, catalog::alpha_myr());
    g.clear_sickness(ogre);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: ogre, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(robot, ogre)])).expect("block");
    drain_stack(&mut g);
    assert!(g.battlefield_find(robot).is_none());
}

/// A flesh blocker doesn't set Ogre Leadfoot off.
#[test]
fn ogre_leadfoot_ignores_a_nonartifact_blocker() {
    let mut g = main_phase();
    let ogre = g.add_card_to_battlefield(0, catalog::ogre_leadfoot());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(ogre);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: ogre, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, ogre)])).expect("block");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "no trigger, so it survives the block");
}

/// Override taxes by your artifact count.
#[test]
fn override_taxes_by_artifact_count() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::tanglebloom());
    }
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("they Bolt you");
    let counter = g.add_card_to_hand(0, catalog::override_card());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: counter, target: Some(Target::Permanent(bolt)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Override it");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "they had no {{2}} to pay");
}

/// Awe Strike blanks the next damage a creature deals and banks it as life.
#[test]
fn awe_strike_prevents_and_gains() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::awe_strike());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    swing(&mut g, bear);
    assert_eq!(g.players[1].life, 20, "the Bear's damage is prevented");
    assert_eq!(g.players[0].life, 22, "2 prevented, 2 gained");
    assert!(g.damage_prevented_sources_debug().is_empty(), "CR 615.8 — one instance only");
}

/// Bloodscent drags every legal blocker onto one creature.
#[test]
fn bloodscent_forces_every_block() {
    let mut g = main_phase();
    let lure = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::bloodscent());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(lure)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.computed_permanent(lure).unwrap().keywords.contains(&Keyword::AllMustBlock));
}

/// Goblin War Wagon needs an upkeep toll to untap.
#[test]
fn goblin_war_wagon_pays_to_untap() {
    let mut g = main_phase();
    let wagon = g.add_card_to_battlefield(0, catalog::goblin_war_wagon());
    g.battlefield_find_mut(wagon).unwrap().tapped = true;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert!(!g.battlefield_find(wagon).unwrap().tapped, "the toll bought an untap");
}

/// Neurok Familiar keeps an artifact and pitches anything else.
#[test]
fn neurok_familiar_sorts_the_top_card() {
    let mut g = main_phase();
    g.players[0].library.clear();
    let rock = g.add_card_to_library(0, catalog::tanglebloom());
    let familiar = g.add_card_to_hand(0, catalog::neurok_familiar());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: familiar, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == rock), "an artifact is kept");
}

/// Inertia Bubble shuts down the artifact it lands on.
#[test]
fn inertia_bubble_locks_an_artifact() {
    let mut g = main_phase();
    let rock = g.add_card_to_battlefield(1, catalog::tanglebloom());
    let bubble = g.add_card_to_hand(0, catalog::inertia_bubble());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bubble, target: Some(Target::Permanent(rock)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.untap_prevented_by_static(rock));
}

/// Contaminated Bond bleeds the enchanted creature's controller on an attack.
#[test]
fn contaminated_bond_drains_on_attack() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bond = g.add_card_to_hand(0, catalog::contaminated_bond());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bond, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Leveler trades your library for a 10/10.
#[test]
fn leveler_exiles_your_library() {
    let mut g = main_phase();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let leveler = g.add_card_to_hand(0, catalog::leveler());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: leveler, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].library.is_empty());
    let cp = g.computed_permanent(leveler).unwrap();
    assert_eq!((cp.power, cp.toughness), (10, 10));
}
