//! Functionality tests for War of the Spark (WAR) — `catalog::sets::war`.

use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// Stat / keyword lines for the simple beaters.
#[test]
fn war_stat_and_keyword_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::ironclad_krovod, 2, 5, &[]),
        (catalog::naga_eternal, 3, 2, &[]),
        (catalog::lazotep_behemoth, 5, 4, &[]),
        (catalog::goblin_assailant, 2, 2, &[]),
        (catalog::enforcer_griffin, 3, 4, &[Keyword::Flying]),
        (catalog::banehound, 1, 1, &[Keyword::Lifelink, Keyword::Haste]),
        (catalog::charity_extractor, 1, 5, &[Keyword::Lifelink]),
        (catalog::sunblade_angel, 3, 3, &[Keyword::Flying, Keyword::FirstStrike, Keyword::Vigilance, Keyword::Lifelink]),
        (catalog::raging_kronch, 4, 3, &[Keyword::CantAttackAlone]),
    ];
    for (f, p, t, kws) in table {
        let c = f();
        assert_eq!((c.power, c.toughness), (*p, *t), "{} P/T", c.name);
        for kw in *kws {
            assert!(c.keywords.contains(kw), "{} should have {:?}", c.name, kw);
        }
    }
}

/// Bulwark Giant gains 5 life on entry.
#[test]
fn bulwark_giant_gains_life() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    g.move_card_to_battlefield_for_test(0, catalog::bulwark_giant());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 5);
}

/// Loxodon Sergeant grants other creatures vigilance until end of turn.
#[test]
fn loxodon_sergeant_grants_vigilance() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::loxodon_sergeant());
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Vigilance));
}

/// Kiora's Dambreaker proliferates on entry.
#[test]
fn kioras_dambreaker_proliferates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.move_card_to_battlefield_for_test(0, catalog::kioras_dambreaker());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Martyr for the Cause proliferates when it dies.
#[test]
fn martyr_for_the_cause_proliferates_on_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let martyr = g.add_card_to_battlefield(0, catalog::martyr_for_the_cause()); // 2/2
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(martyr), 2, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Rising Populace grows when another of your permanents dies.
#[test]
fn rising_populace_grows_on_ally_death() {
    let mut g = two_player_game();
    let pop = g.add_card_to_battlefield(0, catalog::rising_populace());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(bear), 2, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(pop).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Sky Theater Strix pumps on a noncreature spell.
#[test]
fn sky_theater_strix_pumps_on_noncreature_cast() {
    let mut g = two_player_game();
    let strix = g.add_card_to_battlefield(0, catalog::sky_theater_strix());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bolt");
    drain_stack(&mut g); // resolve the prowess-style pump trigger
    assert_eq!(g.computed_permanent(strix).unwrap().power, 2, "+1/+0 until end of turn");
}

/// Erratic Visionary loots (draw then discard).
#[test]
fn erratic_visionary_loots() {
    let mut g = two_player_game();
    let viz = g.add_card_to_battlefield(0, catalog::erratic_visionary());
    g.battlefield_find_mut(viz).unwrap().summoning_sick = false;
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: viz, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("loot");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "drew one and discarded one → net zero");
}

/// Vampire Opportunist drains 2.
#[test]
fn vampire_opportunist_drains() {
    let mut g = two_player_game();
    let vamp = g.add_card_to_battlefield(0, catalog::vampire_opportunist());
    g.battlefield_find_mut(vamp).unwrap().summoning_sick = false;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(6);
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::ActivateAbility {
        card_id: vamp, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("drain");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 2, "opponent loses 2");
    assert_eq!(g.players[0].life, l0 + 2, "you gain 2");
}

/// Ashiok's Skulker makes itself unblockable until end of turn.
#[test]
fn ashioks_skulker_unblockable() {
    let mut g = two_player_game();
    let skulker = g.add_card_to_battlefield(0, catalog::ashioks_skulker());
    g.battlefield_find_mut(skulker).unwrap().summoning_sick = false;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: skulker, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("unblockable");
    drain_stack(&mut g);
    assert!(g.computed_permanent(skulker).unwrap().keywords.contains(&Keyword::Unblockable));
}

/// Grim Initiate amasses Zombies 1 when it dies.
#[test]
fn grim_initiate_amasses_on_death() {
    let mut g = two_player_game();
    let grim = g.add_card_to_battlefield(0, catalog::grim_initiate()); // 1/1
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(grim), 1, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    let army = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)
    });
    let army = army.expect("an Army token exists");
    assert_eq!(army.counter_count(CounterType::PlusOnePlusOne), 1, "amass 1 → one counter");
    assert!(army.definition.subtypes.creature_types.contains(&CreatureType::Zombie), "Army is also a Zombie");
}

/// Pouncing Lynx has first strike only during its controller's turn.
#[test]
fn pouncing_lynx_first_strike_your_turn() {
    let mut g = two_player_game();
    let lynx = g.add_card_to_battlefield(0, catalog::pouncing_lynx());
    g.active_player_idx = 0;
    assert!(g.computed_permanent(lynx).unwrap().keywords.contains(&Keyword::FirstStrike), "first strike on your turn");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(lynx).unwrap().keywords.contains(&Keyword::FirstStrike), "not on opponent's turn");
}
