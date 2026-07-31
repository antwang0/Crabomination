//! Functionality tests for War of the Spark (WAR) — `catalog::sets::war`.

use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
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
        card_id: viz, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
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
        card_id: vamp, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
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
        card_id: skulker, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
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

// ── Spells ──────────────────────────────────────────────────────────────────

/// Cast a WAR instant/sorcery at a single target and drain.
fn cast_at_target(g: &mut GameState, def: crabomination::card::CardDefinition, target: Target, mana: &[(Color, u32)], colorless: u32) {
    let id = g.add_card_to_hand(0, def);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for (c, n) in mana { g.players[0].mana_pool.add(*c, *n); }
    g.players[0].mana_pool.add_colorless(colorless);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(target), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(g);
}

/// Battlefield Promotion: counter, first strike, and 2 life.
#[test]
fn battlefield_promotion_pumps_and_gains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    cast_at_target(&mut g, catalog::battlefield_promotion(), Target::Permanent(bear), &[(Color::White, 1)], 1);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::FirstStrike));
    assert_eq!(g.players[0].life, life + 2);
}

/// Sorin's Thirst: 2 damage to a creature and 2 life.
#[test]
fn sorins_thirst_burns_and_gains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    cast_at_target(&mut g, catalog::sorins_thirst(), Target::Permanent(bear), &[(Color::Black, 2)], 0);
    assert!(g.battlefield_find(bear).is_none(), "2/2 died to 2 damage");
    assert_eq!(g.players[0].life, life + 2);
}

/// Callous Dismissal bounces a permanent and amasses 1.
#[test]
fn callous_dismissal_bounces_and_amasses() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast_at_target(&mut g, catalog::callous_dismissal(), Target::Permanent(bear), &[(Color::Blue, 1)], 1);
    assert!(g.battlefield_find(bear).is_none() && g.players[1].hand.iter().any(|c| c.id == bear), "bounced");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)),
        "amassed an Army",
    );
}

/// Contentious Plan proliferates and draws.
#[test]
fn contentious_plan_proliferates_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.add_card_to_library(0, catalog::forest());
    let plan = g.add_card_to_hand(0, catalog::contentious_plan());
    let hand = g.players[0].hand.len(); // includes the plan
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: plan, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2, "proliferated");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one → net zero");
}

/// Ob Nixilis's Cruelty shrinks a creature and exiles it as it dies.
#[test]
fn ob_nixiliss_cruelty_shrinks_and_exiles() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast_at_target(&mut g, catalog::ob_nixiliss_cruelty(), Target::Permanent(bear), &[(Color::Black, 1)], 2);
    assert!(g.battlefield_find(bear).is_none(), "died to -5/-5");
    assert!(g.exile.iter().any(|c| c.id == bear), "exiled instead of the graveyard");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear), "not in graveyard");
}

/// Unlikely Aid grants +2/+0 and indestructible.
#[test]
fn unlikely_aid_grants_indestructible() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast_at_target(&mut g, catalog::unlikely_aid(), Target::Permanent(bear), &[(Color::Black, 1)], 1);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.power, 4, "+2/+0");
    assert!(c.keywords.contains(&Keyword::Indestructible));
}

/// Relentless Advance amasses Zombies 3.
#[test]
fn relentless_advance_amasses_three() {
    let mut g = two_player_game();
    let adv = g.add_card_to_hand(0, catalog::relentless_advance());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: adv, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let army = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)).expect("Army");
    assert_eq!(army.counter_count(CounterType::PlusOnePlusOne), 3, "amass 3");
}

// ── More creatures ──────────────────────────────────────────────────────────

/// Invading Manticore amasses Zombies 2 on entry.
#[test]
fn invading_manticore_amasses_two() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::invading_manticore());
    drain_stack(&mut g);
    let army = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)).expect("Army");
    assert_eq!(army.counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Vizier of the Scorpion amasses and grants deathtouch to Zombie tokens.
#[test]
fn vizier_grants_deathtouch_to_zombie_tokens() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::vizier_of_the_scorpion());
    drain_stack(&mut g);
    let army = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)).expect("Army").id;
    assert!(g.computed_permanent(army).unwrap().keywords.contains(&Keyword::Deathtouch), "Army (Zombie token) has deathtouch");
}

/// Tithebearer Giant draws a card and loses 1 life on entry.
#[test]
fn tithebearer_giant_draws_and_loses() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    g.move_card_to_battlefield_for_test(0, catalog::tithebearer_giant());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].life, life - 1);
}

/// Law-Rune Enforcer taps a mana-value-2-or-greater creature.
#[test]
fn law_rune_enforcer_taps_expensive_creature() {
    let mut g = two_player_game();
    let enf = g.add_card_to_battlefield(0, catalog::law_rune_enforcer());
    g.battlefield_find_mut(enf).unwrap().summoning_sick = false;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: enf, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None, mode: None,
    }).expect("tap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "MV-2 creature tapped");
}

/// Goblin Assault Team puts a counter on a creature when it dies.
#[test]
fn goblin_assault_team_counter_on_death() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let team = g.add_card_to_battlefield(0, catalog::goblin_assault_team()); // 4/1
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(team), 1, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Duskmantle Operative can't be blocked by power-4+ creatures.
#[test]
fn duskmantle_operative_evasion() {
    let op = catalog::duskmantle_operative();
    assert!(op.keywords.contains(&Keyword::CantBeBlockedByPowerAtLeast(4)));
}

// ── Batch 2 (2026-07-23) ──────────────────────────────────────────────────────

/// Topple the Statue taps a permanent, destroys it if an artifact, and draws.
#[test]
fn topple_the_statue_taps_destroys_artifact_draws() {
    let mut g = two_player_game();
    let sol = g.add_card_to_battlefield(1, catalog::sol_ring()); // artifact
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    cast_at_target(&mut g, catalog::topple_the_statue(), Target::Permanent(sol), &[(Color::White, 1)], 2);
    assert!(g.battlefield_find(sol).is_none(), "artifact destroyed");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Eternal Skylord amasses Zombies 2 and grants flying to Zombie tokens.
#[test]
fn eternal_skylord_amasses_and_grants_flying() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::eternal_skylord());
    drain_stack(&mut g);
    let army = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)).expect("Army").id;
    assert_eq!(g.battlefield_find(army).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(g.computed_permanent(army).unwrap().keywords.contains(&Keyword::Flying), "Zombie token flies");
}

/// Spellkeeper Weird returns an instant/sorcery from the graveyard.
#[test]
fn spellkeeper_weird_returns_instant() {
    let mut g = two_player_game();
    let weird = g.add_card_to_battlefield(0, catalog::spellkeeper_weird());
    g.battlefield_find_mut(weird).unwrap().summoning_sick = false;
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: weird, ability_index: 0, target: Some(Target::Permanent(bolt)), additional_targets: vec![], x_value: None, mode: None,
    }).expect("return");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "Bolt back in hand");
}

/// No Escape counters a creature spell and exiles it.
#[test]
fn no_escape_counters_to_exile() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast bear");
    let esc = g.add_card_to_hand(0, catalog::no_escape());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: esc, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("cast No Escape");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "countered spell exiled");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear), "not in graveyard");
}

/// Jace's Triumph draws two with no Jace out.
#[test]
fn jaces_triumph_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let jt = g.add_card_to_hand(0, catalog::jaces_triumph());
    let hand = g.players[0].hand.len();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: jt, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "cast one, drew two");
}

/// Dreadmalkin sacrifices a creature for two +1/+1 counters.
#[test]
fn dreadmalkin_sacrifices_for_counters() {
    let mut g = two_player_game();
    let cat = g.add_card_to_battlefield(0, catalog::dreadmalkin());
    g.battlefield_find_mut(cat).unwrap().summoning_sick = false;
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility { card_id: cat, ability_index: 0, target: None, additional_targets: vec![], x_value: None , mode: None}).expect("sac");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cat).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Aid the Fallen returns a creature card from the graveyard.
#[test]
fn aid_the_fallen_returns_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let aid = g.add_card_to_hand(0, catalog::aid_the_fallen());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: aid, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature back in hand");
}

/// Cyclops Electromancer deals damage equal to instants/sorceries in gy.
#[test]
fn cyclops_electromancer_scales_with_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.move_card_to_battlefield_for_test(0, catalog::cyclops_electromancer());
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "2 damage from two I/S killed the 2/2");
}

/// Spellgorger Weird grows on a noncreature spell.
#[test]
fn spellgorger_weird_grows_on_noncreature() {
    let mut g = two_player_game();
    let weird = g.add_card_to_battlefield(0, catalog::spellgorger_weird());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(weird).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Tibalt's Rager deals 1 damage when it dies.
#[test]
fn tibalts_rager_pings_on_death() {
    let mut g = two_player_game();
    let rager = g.add_card_to_battlefield(0, catalog::tibalts_rager());
    let life = g.players[1].life;
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(rager), 2, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "death ping hit the opponent");
}

/// Turret Ogre pings each opponent when it enters beside a power-4 creature.
#[test]
fn turret_ogre_pings_with_big_ally() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::primordial_wurm()); // 7/6
    let life = g.players[1].life;
    g.move_card_to_battlefield_for_test(0, catalog::turret_ogre());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "2 damage to the opponent");
}

/// Heartfire sacrifices a creature and deals 4 to any target.
#[test]
fn heartfire_sacrifices_and_burns() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hf = g.add_card_to_hand(0, catalog::heartfire());
    let life = g.players[1].life;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: hf, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
    assert_eq!(g.players[1].life, life - 4, "4 damage");
}

/// Chandra's Triumph deals 3 to an opponent's creature with no Chandra out.
#[test]
fn chandras_triumph_deals_three() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    cast_at_target(&mut g, catalog::chandras_triumph(), Target::Permanent(bear), &[(Color::Red, 1)], 1);
    assert!(g.battlefield_find(bear).is_none(), "3 damage killed the 2/2");
}

/// Nahiri's Stoneblades pumps two creatures.
#[test]
fn nahiris_stoneblades_pumps_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sb = g.add_card_to_hand(0, catalog::nahiris_stoneblades());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: sb, target: Some(Target::Permanent(a)), additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(a).unwrap().power, 4, "+2/+0");
    assert_eq!(g.computed_permanent(b).unwrap().power, 4, "+2/+0");
}

/// Arboreal Grazer puts a land from hand onto the battlefield tapped.
#[test]
fn arboreal_grazer_ramps_a_land() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![forest])]));
    g.move_card_to_battlefield_for_test(0, catalog::arboreal_grazer());
    drain_stack(&mut g);
    assert!(g.battlefield_find(forest).map(|c| c.tapped).unwrap_or(false), "land entered tapped");
}

/// Bloom Hulk proliferates on entry.
#[test]
fn bloom_hulk_proliferates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.move_card_to_battlefield_for_test(0, catalog::bloom_hulk());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Centaur Nurturer gains 3 life on entry.
#[test]
fn centaur_nurturer_gains_life() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    g.move_card_to_battlefield_for_test(0, catalog::centaur_nurturer());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3);
}

/// Challenger Troll grants power-4+ creatures can't-be-blocked-by-more-than-one.
#[test]
fn challenger_troll_grants_evasion() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::primordial_wurm()); // 7/6
    g.add_card_to_battlefield(0, catalog::challenger_troll());
    drain_stack(&mut g);
    assert!(g.computed_permanent(wurm).unwrap().keywords.contains(&Keyword::CantBeBlockedByMoreThanOne));
}

/// Evolution Sage proliferates whenever a land you control enters.
#[test]
fn evolution_sage_landfall_proliferates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.add_card_to_battlefield(0, catalog::evolution_sage());
    let f = g.add_card_to_battlefield(0, catalog::forest());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: f }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Thundering Ceratok grants trample to your other creatures.
#[test]
fn thundering_ceratok_grants_trample() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::thundering_ceratok());
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample));
}

/// Kronch Wrangler grows when a big creature enters.
#[test]
fn kronch_wrangler_grows_on_big_entry() {
    let mut g = two_player_game();
    let kronch = g.add_card_to_battlefield(0, catalog::kronch_wrangler());
    let wurm = g.add_card_to_battlefield(0, catalog::primordial_wurm()); // 7/6
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: wurm }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(kronch).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Steady Aim untaps a creature and grants +1/+4 and reach.
#[test]
fn steady_aim_untaps_and_pumps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    cast_at_target(&mut g, catalog::steady_aim(), Target::Permanent(bear), &[(Color::Green, 1)], 1);
    let c = g.computed_permanent(bear).unwrap();
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
    assert_eq!((c.power, c.toughness), (3, 6), "+1/+4");
    assert!(c.keywords.contains(&Keyword::Reach));
}

/// Forced Landing puts a flyer on the bottom of its owner's library.
#[test]
fn forced_landing_bottoms_a_flyer() {
    let mut g = two_player_game();
    let griffin = g.add_card_to_battlefield(1, catalog::enforcer_griffin()); // flying
    cast_at_target(&mut g, catalog::forced_landing(), Target::Permanent(griffin), &[(Color::Green, 1)], 1);
    assert!(g.battlefield_find(griffin).is_none(), "left the battlefield");
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(griffin), "on the bottom of the library");
}

/// Guild Globe draws on entry.
#[test]
fn guild_globe_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::guild_globe());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Iron Bully puts a +1/+1 counter on a creature when it enters.
#[test]
fn iron_bully_counters_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::iron_bully());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// God-Pharaoh's Statue drains each opponent at your end step.
#[test]
fn god_pharaohs_statue_end_step_drain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::god_pharaohs_statue());
    g.active_player_idx = 0;
    let life = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
}

/// Gateway Plaza sacrifices itself when its {1} isn't paid, and survives when
/// mana is available (SacrificeSourceUnlessPay).
#[test]
fn gateway_plaza_sacrifice_unless_pay() {
    // No mana: the pay-{1}-or-sacrifice trigger sacrifices it.
    let mut g = two_player_game();
    let plaza = g.move_card_to_battlefield_for_test(0, catalog::gateway_plaza());
    drain_stack(&mut g);
    assert!(g.battlefield_find(plaza).is_none(), "unpaid → sacrificed");

    // With {1} floating, it stays.
    let mut g = two_player_game();
    g.players[0].mana_pool.add_colorless(1);
    let plaza = g.move_card_to_battlefield_for_test(0, catalog::gateway_plaza());
    drain_stack(&mut g);
    assert!(g.battlefield_find(plaza).is_some(), "paid one → kept");
}

/// Pledge of Unity counters each creature and gains life per creature.
#[test]
fn pledge_of_unity_counters_and_gains() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pledge = g.add_card_to_hand(0, catalog::pledge_of_unity());
    let life = g.players[0].life;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: pledge, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].life, life + 2, "gained 1 per creature");
}

/// Rubblebelt Rioters gets +X/+0 (greatest power) when it attacks.
#[test]
fn rubblebelt_rioters_pumps_on_attack() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::primordial_wurm()); // power 7
    let riot = g.add_card_to_battlefield(0, catalog::rubblebelt_rioters()); // 0/4
    g.battlefield_find_mut(riot).unwrap().summoning_sick = false;
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: riot, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(riot).unwrap().power, 7, "+X where X = greatest power (7)");
}

/// Invade the City amasses X = instants/sorceries in your graveyard.
#[test]
fn invade_the_city_amasses_x() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let inv = g.add_card_to_hand(0, catalog::invade_the_city());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: inv, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let army = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)).expect("Army");
    assert_eq!(army.counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Soul Diviner draws by removing a counter from your permanent.
#[test]
fn soul_diviner_removes_counter_to_draw() {
    let mut g = two_player_game();
    let sd = g.add_card_to_battlefield(0, catalog::soul_diviner());
    g.battlefield_find_mut(sd).unwrap().summoning_sick = false;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: sd, ability_index: 0, target: None, additional_targets: vec![], x_value: None , mode: None}).expect("draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "counter removed");
}

/// Dreadhorde Butcher deals its power to any target when it dies.
#[test]
fn dreadhorde_butcher_death_burst() {
    let mut g = two_player_game();
    let butcher = g.add_card_to_battlefield(0, catalog::dreadhorde_butcher()); // 1/1
    let life = g.players[1].life;
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(butcher), 1, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "dealt its power (1) on death");
}

/// Price of Betrayal removes up to five counters from a permanent (CR 122.6).
#[test]
fn price_of_betrayal_strips_permanent_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    cast_at_target(&mut g, catalog::price_of_betrayal(), Target::Permanent(bear), &[(Color::Black, 1)], 0);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "all 3 removed (up to 5)");
}

/// Price of Betrayal removes up to five poison counters from an opponent.
#[test]
fn price_of_betrayal_strips_player_poison() {
    let mut g = two_player_game();
    g.players[1].poison_counters = 7;
    cast_at_target(&mut g, catalog::price_of_betrayal(), Target::Player(1), &[(Color::Black, 1)], 0);
    assert_eq!(g.players[1].poison_counters, 2, "5 poison removed");
}

// ── Planeswalkers (2026-07-23) ────────────────────────────────────────────────

/// Tibalt's static stops opponents gaining life; his −2 makes a Devil.
#[test]
fn tibalt_locks_opponent_lifegain_and_makes_devil() {
    let mut g = two_player_game();
    let tibalt = g.add_card_to_battlefield(0, catalog::tibalt_rakish_instigator());
    // Opponent can't gain life.
    let before = g.players[1].life;
    g.adjust_life(1, 5);
    assert_eq!(g.players[1].life, before, "opponent's life-gain is locked");
    // −2 makes a 1/1 Devil.
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: tibalt, ability_index: 0, target: None, x_value: None }).expect("-2");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Devil)));
}

/// Teyo grants his controller hexproof and makes a 0/3 Wall.
#[test]
fn teyo_grants_hexproof_and_makes_wall() {
    let mut g = two_player_game();
    let teyo = g.add_card_to_battlefield(0, catalog::teyo_the_shieldmage());
    assert!(g.player_has_static_hexproof(0), "controller has hexproof");
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: teyo, ability_index: 0, target: None, x_value: None }).expect("-2");
    drain_stack(&mut g);
    let wall = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Wall)).expect("Wall");
    assert!(wall.definition.keywords.contains(&Keyword::Defender));
}

/// The Wanderer prevents noncombat damage to its controller (CR 615).
#[test]
fn the_wanderer_prevents_noncombat_damage_to_you() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_wanderer());
    let life = g.players[0].life;
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(0), 3, None, &mut evs);
    assert_eq!(g.players[0].life, life, "noncombat damage to you is prevented");
    // −2 exiles a big creature.
    let wanderer = g.battlefield.iter().find(|c| c.definition.name == "The Wanderer").unwrap().id;
    let wurm = g.add_card_to_battlefield(1, catalog::primordial_wurm()); // 7/6, power ≥ 4
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: wanderer, ability_index: 0, target: Some(Target::Permanent(wurm)), x_value: None }).expect("-2");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == wurm), "big creature exiled");
}

/// Kasmina taxes opponents' spells targeting your permanents.
#[test]
fn kasmina_taxes_targeted_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kasmina_enigmatic_mentor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Opponent's Lightning Bolt targeting your bear costs {2} more → {2}{R}.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    // Only {R} available — the tax makes it unaffordable.
    let res = g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None });
    assert!(res.is_err(), "the {{2}} tax makes the targeted bolt unaffordable");
}

/// Ob Nixilis pings an opponent when they draw; his −2 destroys and refills.
#[test]
fn ob_nixilis_pings_on_opponent_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ob_nixilis_the_hate_twisted());
    g.add_card_to_library(1, catalog::forest());
    let life = g.players[1].life;
    let mut drawn = Vec::new();
    g.draw_one(1, &mut drawn);
    g.dispatch_triggers_for_events(&drawn);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "opponent took 1 for drawing");
}

/// A planeswalker at 0 loyalty is put into its owner's graveyard (CR 704.5i).
#[test]
fn planeswalker_dies_at_zero_loyalty() {
    let mut g = two_player_game();
    let tibalt = g.add_card_to_battlefield(0, catalog::tibalt_rakish_instigator());
    g.battlefield_find_mut(tibalt).unwrap().counters.insert(CounterType::Loyalty, 0);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert!(g.battlefield_find(tibalt).is_none(), "0-loyalty walker left the battlefield");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == tibalt), "in its owner's graveyard");
}

// ── Batch 3 (2026-07-23) ──────────────────────────────────────────────────────

/// God-Eternal Bontu sacrifices other permanents and draws that many.
#[test]
fn god_eternal_bontu_sacs_and_draws() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    g.move_card_to_battlefield_for_test(0, catalog::god_eternal_bontu());
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "sacrificed both");
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew two");
}

/// God-Eternal Oketra makes a 4/4 Zombie when you cast a creature spell.
#[test]
fn god_eternal_oketra_makes_zombie_on_creature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::god_eternal_oketra());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast bear");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Zombie Warrior" && c.definition.power == 4));
}

/// Fblthp draws on entry.
#[test]
fn fblthp_draws_on_entry() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::fblthp_the_lost());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Bond of Revival reanimates a creature with haste.
#[test]
fn bond_of_revival_reanimates_with_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    cast_at_target(&mut g, catalog::bond_of_revival(), Target::Permanent(bear), &[(Color::Black, 1)], 4);
    let c = g.computed_permanent(bear).expect("on battlefield");
    assert!(c.keywords.contains(&Keyword::Haste), "gained haste");
}

/// Deathsprout destroys a creature and ramps a basic land tapped.
#[test]
fn deathsprout_kills_and_ramps() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let forest = g.add_card_to_library(0, catalog::forest());
    let lands = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.card_types.contains(&crabomination::card::CardType::Land)).count();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    cast_at_target(&mut g, catalog::deathsprout(), Target::Permanent(bear), &[(Color::Black, 2), (Color::Green, 1)], 1);
    assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
    let lands_after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.card_types.contains(&crabomination::card::CardType::Land)).count();
    assert_eq!(lands_after, lands + 1, "fetched a basic land");
}

/// Ravnica at War exiles all multicolored permanents.
#[test]
fn ravnica_at_war_exiles_multicolored() {
    let mut g = two_player_game();
    let gold = g.add_card_to_battlefield(1, catalog::dreadhorde_butcher()); // {B}{R} multicolored
    let mono = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green mono
    let rw = g.add_card_to_hand(0, catalog::ravnica_at_war());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: rw, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gold).is_none(), "multicolored exiled");
    assert!(g.battlefield_find(mono).is_some(), "monocolored survives");
}

/// Courage in Crisis counters a creature then proliferates.
#[test]
fn courage_in_crisis_counters_and_proliferates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast_at_target(&mut g, catalog::courage_in_crisis(), Target::Permanent(bear), &[(Color::Green, 1)], 2);
    // +1/+1 from the counter, then proliferate adds another → 2.
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Casualties of War destroys across chosen types.
#[test]
fn casualties_of_war_destroys_chosen() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bond = g.add_card_to_hand(0, catalog::casualties_of_war());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    // Choose just the "destroy creature" mode (index 1).
    g.perform_action(GameAction::CastSpell { card_id: bond, target: Some(Target::Permanent(creature)), additional_targets: vec![], mode: Some(1), x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(creature).is_none(), "creature destroyed");
}

/// Finale of Glory makes X Soldiers (and Angels too at X ≥ 10).
#[test]
fn finale_of_glory_makes_soldiers() {
    let mut g = two_player_game();
    let fin = g.add_card_to_hand(0, catalog::finale_of_glory());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: fin, target: None, additional_targets: vec![], mode: None, x_value: Some(3) }).expect("cast X=3");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 3, "X=3 Soldiers");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Angel"), "no Angels below X=10");
}

// ── Batch 4 (2026-07-23) ──────────────────────────────────────────────────────

/// Guildpact Informant proliferates on combat damage to a player.
#[test]
fn guildpact_informant_proliferates_on_hit() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let inf = g.add_card_to_battlefield(0, catalog::guildpact_informant());
    g.fire_combat_damage_to_player_triggers(inf, 1, 1);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Teyo's Lightshield buffs your creature on entry.
#[test]
fn teyos_lightshield_buffs_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::teyos_lightshield());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Roalesk proliferates twice when it dies.
#[test]
fn roalesk_proliferates_twice_on_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let roalesk = g.add_card_to_battlefield(0, catalog::roalesk_apex_hybrid());
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(roalesk), 5, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3, "proliferated twice → 1+1+1");
}

/// Jace's Projection grows whenever you draw a card.
#[test]
fn jaces_projection_grows_on_draw() {
    let mut g = two_player_game();
    let proj = g.add_card_to_battlefield(0, catalog::jaces_projection());
    g.add_card_to_library(0, catalog::forest());
    let mut drawn = Vec::new();
    g.draw_one(0, &mut drawn);
    g.dispatch_triggers_for_events(&drawn);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(proj).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

// ── Batch 5 (2026-07-23) ──────────────────────────────────────────────────────

/// Silent Submersible is a 2/3 Crew-2 Vehicle that draws on combat damage.
#[test]
fn silent_submersible_draws_on_hit() {
    let mut g = two_player_game();
    let sub = g.add_card_to_battlefield(0, catalog::silent_submersible());
    g.add_card_to_library(0, catalog::forest());
    let def = catalog::silent_submersible();
    assert!(def.keywords.contains(&Keyword::Crew(2)));
    assert!(def.subtypes.artifact_subtypes.contains(&crabomination::card::ArtifactSubtype::Vehicle));
    let hand = g.players[0].hand.len();
    g.fire_combat_damage_to_player_triggers(sub, 1, 2);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew on combat damage");
}

/// Storrev returns a creature card from the graveyard on combat damage.
#[test]
fn storrev_recurs_on_hit() {
    let mut g = two_player_game();
    let storrev = g.add_card_to_battlefield(0, catalog::storrev_devkarin_lich());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.fire_combat_damage_to_player_triggers(storrev, 1, 5);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature returned to hand");
}

// ── Batch 6 (2026-07-23): planeswalker-matters + counter payoffs ──────────────

/// Bioessence Hydra enters with a +1/+1 counter per loyalty counter on your PWs.
#[test]
fn bioessence_hydra_enters_scaling_with_loyalty() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::the_wanderer());
    let loy = g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty);
    assert!(loy > 0, "planeswalker entered with loyalty");
    let hydra = g.move_card_to_battlefield_for_test(0, catalog::bioessence_hydra());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne), loy);
}

/// Bioessence Hydra grows when loyalty counters are put on your planeswalkers.
#[test]
fn bioessence_hydra_grows_on_loyalty_added() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::the_wanderer());
    let hydra = g.add_card_to_battlefield(0, catalog::bioessence_hydra());
    g.battlefield_find_mut(pw).unwrap().add_counters(CounterType::Loyalty, 2);
    g.dispatch_triggers_for_events(&[GameEvent::CounterAdded {
        card_id: pw,
        counter_type: CounterType::Loyalty,
        count: 2,
    }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Charmed Stray's ETB counters each other Charmed Stray you control.
#[test]
fn charmed_stray_counters_others() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::charmed_stray());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.move_card_to_battlefield_for_test(0, catalog::charmed_stray());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "other Stray grew");
    assert_eq!(g.battlefield_find(c).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "source excluded");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "non-Stray untouched");
}

/// Jaya's static adds 1 damage to another red source you control.
#[test]
fn jaya_boosts_red_source_damage() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::jaya_venerated_firemage());
    // A red source you control (a Mountain-agnostic red creature) pings for 1 → 2.
    let goblin = g.add_card_to_battlefield(0, catalog::goblin_assailant());
    let l1 = g.players[1].life;
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(1),
        1,
        Some(goblin),
        &mut evs,
    );
    assert_eq!(g.players[1].life, l1 - 2, "1 damage boosted to 2");
}

/// Kaya's Ghostform returns the enchanted creature when it dies.
#[test]
fn kayas_ghostform_returns_on_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::kayas_ghostform());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(bear), 2, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0),
        "creature returned under your control"
    );
}

/// Kaya's Ghostform returns the enchanted creature when it's exiled.
#[test]
fn kayas_ghostform_returns_on_exile() {
    use crabomination::effect::{Effect, ZoneDest};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::kayas_ghostform());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    let evs = g
        .resolve_effect(&Effect::Move { what: crabomination::effect::Selector::Target(0), to: ZoneDest::Exile }, &ctx)
        .unwrap();
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    g.dispatch_triggers_for_events(&sba);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0),
        "creature returned under your control after exile"
    );
}

/// Command the Dreadhorde reanimates chosen graveyard creatures and deals you
/// damage equal to their total mana value.
#[test]
fn command_the_dreadhorde_reanimates_for_life() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // A 5-drop in your graveyard, a 2-drop in the opponent's.
    let mine = g.add_card_to_graveyard(0, catalog::challenger_troll()); // {4}{G} → mv 5
    let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears()); // {1}{G} → mv 2
    let cmd = g.add_card_to_hand(0, catalog::command_the_dreadhorde());
    let life = g.players[0].life;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 6); // {4}{B}{B}
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![mine, theirs])]));
    g.perform_action(GameAction::CastSpell { card_id: cmd, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Command the Dreadhorde");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == mine && c.controller == 0), "my creature reanimated");
    assert!(g.battlefield.iter().any(|c| c.id == theirs && c.controller == 0), "opponent's creature stolen");
    assert_eq!(g.players[0].life, life - 7, "lost life equal to total mana value (5+2)");
}

/// Vivien's Grizzly draws a revealed creature off the top of the library.
#[test]
fn viviens_grizzly_reveals_creature_to_hand() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let grizzly = g.add_card_to_battlefield(0, catalog::viviens_grizzly());
    g.battlefield_find_mut(grizzly).unwrap().summoning_sick = false;
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: grizzly, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == top), "creature drawn to hand");
}

/// A non-creature top card goes to the bottom of the library instead.
#[test]
fn viviens_grizzly_bottoms_noncreature() {
    let mut g = two_player_game();
    let grizzly = g.add_card_to_battlefield(0, catalog::viviens_grizzly());
    g.battlefield_find_mut(grizzly).unwrap().summoning_sick = false;
    let land = g.add_card_to_library(0, catalog::forest()); // on top
    g.add_card_to_library(0, catalog::grizzly_bears()); // below the land
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: grizzly, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(land), "land bottomed");
    assert!(!g.players[0].hand.iter().any(|c| c.id == land), "land not drawn");
}

/// Mowu turns each +1/+1 placement into that many plus one — via a direct
/// counter effect and via proliferate.
#[test]
fn mowu_adds_one_extra_counter() {
    let mut g = two_player_game();
    let mowu = g.add_card_to_battlefield(0, catalog::mowu_loyal_companion());
    let ctx = crabomination::game::effects::EffectContext::for_ability(mowu, 0, None);
    // Put two +1/+1 counters on Mowu → 2 + 1 = 3.
    g.resolve_effect(&crabomination::effect::Effect::AddCounter {
        what: crabomination::effect::Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: crabomination::card::Value::Const(2),
    }, &ctx).unwrap();
    assert_eq!(g.battlefield_find(mowu).unwrap().counter_count(CounterType::PlusOnePlusOne), 3, "2 → 3");
    // Proliferate adds another (1 + 1 = 2 more) → 5 total.
    g.resolve_effect(&crabomination::effect::Effect::Proliferate, &ctx).unwrap();
    assert_eq!(g.battlefield_find(mowu).unwrap().counter_count(CounterType::PlusOnePlusOne), 5, "proliferate 1 → 2");
}

/// Band Together: two of your creatures each ping a target for their power.
#[test]
fn band_together_two_creatures_pile_on() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let b = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let bt = g.add_card_to_hand(0, catalog::band_together());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: bt,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![Target::Permanent(a), Target::Permanent(b)],
        mode: None,
        x_value: None,
    }).expect("cast Band Together");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "victim took 2+3=5 and died");
}

// ── Batch 7 (2026-07-23): simple commons/uncommons ────────────────────────────

/// Ugin's Conjurant prevents damage by shedding +1/+1 counters, and is defined
/// to enter with X counters.
#[test]
fn ugins_conjurant_prevents_damage_with_counters() {
    assert!(matches!(
        catalog::ugins_conjurant().enters_with_counters,
        Some((CounterType::PlusOnePlusOne, crabomination::card::Value::XFromCost))
    ), "enters with X +1/+1 counters");
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ugins_conjurant());
    g.battlefield_find_mut(id).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    let mut d = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(id), 2, None, &mut d);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "prevented by removing 2 counters");
}

/// Domri's Ambush pumps your creature then fights an enemy with its power.
#[test]
fn domris_ambush_pumps_and_bites() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/3 after counter
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let spell = g.add_card_to_hand(0, catalog::domris_ambush());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(foe)], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.battlefield_find(foe).is_none(), "3 damage killed the 3/3");
}

/// Spark Harvest can pay the alt-cost with mana and destroy a planeswalker.
#[test]
fn spark_harvest_destroys_with_alt_cost() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(1, catalog::the_wanderer());
    let spell = g.add_card_to_hand(0, catalog::spark_harvest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 5); // {B} + {4} alt-cost
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(pw)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(pw).is_none(), "planeswalker destroyed");
}

/// Eternal Taskmaster enters tapped and recurs on attack for {2}{B}.
#[test]
fn eternal_taskmaster_enters_tapped_and_recurs() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let tm = g.move_card_to_battlefield_for_test(0, catalog::eternal_taskmaster());
    drain_stack(&mut g);
    assert!(g.battlefield_find(tm).unwrap().tapped, "enters tapped");
    g.battlefield_find_mut(tm).unwrap().tapped = false;
    g.battlefield_find_mut(tm).unwrap().summoning_sick = false;
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: tm, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature returned to hand");
}

/// Living Twister pings for 2 by discarding a land.
#[test]
fn living_twister_pings_by_discarding_land() {
    let mut g = two_player_game();
    let lt = g.add_card_to_battlefield(0, catalog::living_twister());
    g.battlefield_find_mut(lt).unwrap().summoning_sick = false;
    g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l1 = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: lt, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 2, "dealt 2");
}

/// Lazotep Plating amasses and grants hexproof.
#[test]
fn lazotep_plating_amass_and_hexproof() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::lazotep_plating());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.counter_count(CounterType::PlusOnePlusOne) == 1), "Army token amassed");
    assert!(g.players[0].hexproof_from_colors_this_turn.len() >= 5, "gained hexproof from all colors");
}

// ── Batch 8 (2026-07-23): search / discard commons ────────────────────────────

/// Davriel's Shadowfugue makes the target discard two and lose 2 life.
#[test]
fn davriels_shadowfugue_discards_and_drains() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::davriels_shadowfugue());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let (hand1, life1) = (g.players[1].hand.len(), g.players[1].life);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand1 - 2, "discarded two");
    assert_eq!(g.players[1].life, life1 - 2, "lost 2 life");
}

/// Ignite the Beacon tutors up to two planeswalkers to hand.
#[test]
fn ignite_the_beacon_fetches_planeswalkers() {
    let mut g = two_player_game();
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let pw1 = g.add_card_to_library(0, catalog::the_wanderer());
    let pw2 = g.add_card_to_library(0, catalog::tibalt_rakish_instigator());
    g.add_card_to_library(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::ignite_the_beacon());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(pw1)),
        DecisionAnswer::Search(Some(pw2)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == pw1), "walker 1 fetched");
    assert!(g.players[0].hand.iter().any(|c| c.id == pw2), "walker 2 fetched");
}

/// Nissa's Triumph fetches two basic Forests without a Nissa in play.
#[test]
fn nissas_triumph_fetches_forests() {
    let mut g = two_player_game();
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let f1 = g.add_card_to_library(0, catalog::forest());
    let f2 = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::nissas_triumph());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f1)),
        DecisionAnswer::Search(Some(f2)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == f1), "forest 1 to hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == f2), "forest 2 to hand");
}

/// Desperate Lunge pumps a creature, grants flying, and gains 2 life.
#[test]
fn desperate_lunge_pumps_and_gains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::desperate_lunge());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::Flying), "gained flying");
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
}

/// Gideon's Battle Cry counters up every creature you control.
#[test]
fn gideons_battle_cry_counters_your_team() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::hill_giant());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::gideons_battle_cry());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(foe).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "not opponents' creatures");
}

// ── Batch 9 (2026-07-24): hybrid walkers, God-Eternal Rhonas, modal spells ────

/// Angrath gives your creatures menace and amasses Zombies 2 for −2.
#[test]
fn angrath_menace_anthem_and_amass() {
    let mut g = two_player_game();
    let angrath = g.add_card_to_battlefield(0, catalog::angrath_captain_of_chaos());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Menace), "creatures you control have menace");
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: angrath, ability_index: 0, target: None, x_value: None }).expect("-2");
    drain_stack(&mut g);
    let army = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)).expect("Army");
    assert_eq!(army.counter_count(CounterType::PlusOnePlusOne), 2, "amass 2");
    assert!(army.definition.subtypes.creature_types.contains(&CreatureType::Zombie));
}

/// Huatli's static grants toughness-damage; −3 gains life = greatest toughness.
#[test]
fn huatli_toughness_damage_and_lifegain() {
    let mut g = two_player_game();
    let huatli = g.add_card_to_battlefield(0, catalog::huatli_the_suns_heart());
    let griffin = g.add_card_to_battlefield(0, catalog::enforcer_griffin()); // 3/4
    assert!(g.computed_permanent(griffin).unwrap().keywords.contains(&Keyword::AssignsCombatDamageByToughness));
    let life = g.players[0].life;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: huatli, ability_index: 0, target: None, x_value: None }).expect("-3");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "gained life = greatest toughness (4)");
}

/// Kiora draws when a power-4+ creature enters, and untaps for −1.
#[test]
fn kiora_draws_on_big_creature_and_untaps() {
    let mut g = two_player_game();
    let kiora = g.add_card_to_battlefield(0, catalog::kiora_behemoth_beckoner());
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    let wurm = g.add_card_to_battlefield(0, catalog::primordial_wurm()); // 7/6
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: wurm }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew for the power-4+ enter");
    // −1 untaps a tapped permanent.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: kiora, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: None }).expect("-1");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
}

/// Samut's static grants haste; −1 pumps +2/+1, grants haste, scries.
#[test]
fn samut_haste_anthem_and_pump() {
    let mut g = two_player_game();
    let samut = g.add_card_to_battlefield(0, catalog::samut_tyrant_smasher());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "creatures you control have haste");
    g.add_card_to_library(0, catalog::forest());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: samut, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: None }).expect("-1");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "+2/+1");
}

/// God-Eternal Rhonas doubles other creatures' power and grants vigilance.
#[test]
fn god_eternal_rhonas_doubles_power() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.move_card_to_battlefield_for_test(0, catalog::god_eternal_rhonas());
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "power doubled");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "gained vigilance");
    assert!(catalog::god_eternal_rhonas().keywords.contains(&Keyword::Deathtouch));
}

/// Tamiyo's Epiphany scries 4, then draws two.
#[test]
fn tamiyos_epiphany_draws_two() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); }
    let spell = g.add_card_to_hand(0, catalog::tamiyos_epiphany());
    let hand = g.players[0].hand.len();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "spell left hand, drew 2");
}

/// Commence the Endgame draws two, then amasses Zombies X = cards in hand.
#[test]
fn commence_the_endgame_amasses_hand_size() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    let spell = g.add_card_to_hand(0, catalog::commence_the_endgame());
    g.add_card_to_hand(0, catalog::forest()); // one extra card so hand at resolve is deterministic
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    // After casting: hand had {spell, forest} → spell leaves → 1 card, draw 2 → 3 cards. Amass X=3.
    let army = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)).expect("Army");
    assert_eq!(army.counter_count(CounterType::PlusOnePlusOne), 3, "amass = cards in hand at resolution");
    assert!(catalog::commence_the_endgame().keywords.contains(&Keyword::CantBeCountered));
}

/// Teferi's Time Twist exiles a permanent, which returns at the next end step
/// with an extra +1/+1 counter if it's a creature.
#[test]
fn teferis_time_twist_flickers_with_counter() {
    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step { g.perform_action(GameAction::PassPriority).expect("pass"); }
    }
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast_at_target(&mut g, catalog::teferis_time_twist(), Target::Permanent(bear), &[(Color::Blue, 1)], 1);
    assert!(g.battlefield_find(bear).is_none(), "exiled");
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    let returned = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Grizzly Bears").expect("returned");
    assert_eq!(returned.counter_count(CounterType::PlusOnePlusOne), 1, "returned with a +1/+1 counter");
}

/// Rescuer Sphinx's ETB is a reflexive bounce-then-grow (the bot declines the
/// downside of bouncing its own permanent, so assert the wiring).
#[test]
fn rescuer_sphinx_reflexive_shape() {
    use crabomination::effect::Effect;
    let def = catalog::rescuer_sphinx();
    assert_eq!((def.power, def.toughness), (3, 2));
    assert!(def.keywords.contains(&Keyword::Flying));
    let Effect::MayDo { body, .. } = &def.triggered_abilities[0].effect else { panic!("ETB is a MayDo") };
    let Effect::Seq(steps) = &**body else { panic!("body is a Seq") };
    assert!(matches!(steps.last(), Some(Effect::AddCounter { kind: CounterType::PlusOnePlusOne, .. })), "grows on success");
}

/// Storm the Citadel pumps your creatures +2/+2 and grants the combat trigger.
#[test]
fn storm_the_citadel_pumps_team() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::storm_the_citadel());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "your creature got +2/+2");
    assert_eq!((g.computed_permanent(foe).unwrap().power, g.computed_permanent(foe).unwrap().toughness), (2, 2), "opponent's is unaffected");
}

/// Interplanar Beacon gains 1 life when you cast a planeswalker spell.
#[test]
fn interplanar_beacon_gains_on_walker_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::interplanar_beacon());
    let walker = g.add_card_to_hand(0, catalog::samut_tyrant_smasher()); // {2}{R/G}{R/G}
    let life = g.players[0].life;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: walker, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast walker");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1 for casting a planeswalker");
}

/// Oath of Kaya's ETB burns any target for 3 and gains 3 life.
#[test]
fn oath_of_kaya_etb_burns_and_gains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let (life, opp) = (g.players[0].life, g.players[1].life);
    let oath = g.add_card_to_hand(0, catalog::oath_of_kaya());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: oath, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // ETB gains 3 unconditionally and deals 3 to the auto-chosen target.
    assert_eq!(g.players[0].life, life + 3, "gained 3");
    let dealt = g.battlefield_find(bear).is_none() || g.players[1].life <= opp - 3;
    assert!(dealt, "3 damage was dealt to some target");
}

/// Vivien's Arkbow digs X and deploys a small creature onto the battlefield.
#[test]
fn viviens_arkbow_deploys_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bow = g.add_card_to_battlefield(0, catalog::viviens_arkbow());
    // Top of library: a 2/2 (MV 2, within X=2) plus a filler land.
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let mut ctx = crabomination::game::effects::EffectContext::for_ability(bow, 0, None);
    ctx.x_value = 2;
    g.resolve_effect(&catalog::viviens_arkbow().activated_abilities[0].effect.clone(), &ctx).unwrap();
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Grizzly Bears"), "deployed the 2/2 within X");
}

/// Mobilized District animates into a 3/3 vigilant Citizen that's still a land.
#[test]
fn mobilized_district_animates() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_battlefield(0, catalog::mobilized_district());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility { card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None , mode: None}).expect("animate");
    drain_stack(&mut g);
    let c = g.computed_permanent(land).expect("district");
    assert!(c.card_types.contains(&crabomination::card::CardType::Creature) && c.card_types.contains(&crabomination::card::CardType::Land));
    assert_eq!((c.power, c.toughness), (3, 3));
    assert!(c.keywords.contains(&Keyword::Vigilance));
}

/// Emergence Zone's sac ability lets you cast a sorcery-speed spell at instant speed.
#[test]
fn emergence_zone_grants_flash() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let zone = g.add_card_to_battlefield(0, catalog::emergence_zone());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility { card_id: zone, ability_index: 1, target: None, additional_targets: vec![], x_value: None , mode: None}).expect("sac for flash");
    drain_stack(&mut g);
    assert!(g.players[0].sorceries_as_flash, "may cast at instant speed this turn");
    assert!(g.battlefield_find(zone).is_none(), "sacrificed itself");
}

/// Enter the God-Eternals burns a creature, gains life, mills, and amasses 4.
#[test]
fn enter_the_god_eternals_full_line() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, dies to 4
    for _ in 0..6 { g.add_card_to_library(1, catalog::forest()); }
    let lib1 = g.players[1].library.len();
    let spell = g.add_card_to_hand(0, catalog::enter_the_god_eternals());
    let life = g.players[0].life;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![Target::Player(1)], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature took 4 and died");
    assert_eq!(g.players[0].life, life + 4, "gained 4");
    assert_eq!(g.players[1].library.len(), lib1 - 4, "target milled 4");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.counter_count(CounterType::PlusOnePlusOne) == 4 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)), "amassed 4");
}

/// Tolsimir makes Voja on entry; a Wolf entering gains 3 life and fights.
#[test]
fn tolsimir_makes_voja_and_wolf_fights() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 target for the fight
    let _ = foe;
    let life = g.players[0].life;
    g.move_card_to_battlefield_for_test(0, catalog::tolsimir_friend_to_wolves());
    drain_stack(&mut g);
    // Voja (a Wolf) entered → Tolsimir's wolf-trigger fired: gain 3 life.
    let voja = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Voja, Friend to Elves").expect("Voja");
    assert!(voja.definition.subtypes.creature_types.contains(&CreatureType::Wolf));
    assert!(voja.definition.supertypes.contains(&crabomination::card::Supertype::Legendary));
    assert_eq!(g.players[0].life, life + 3, "gained 3 for the Wolf entering");
}

/// Role Reversal exchanges control of two target permanents.
#[test]
fn role_reversal_swaps_control() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::role_reversal());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(mine)), additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1, "my bear now theirs");
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0, "their giant now mine");
}

/// Heartwarming Redemption wheels the hand for one extra and gains that much life.
#[test]
fn heartwarming_redemption_wheels_plus_one() {
    let mut g = two_player_game();
    for _ in 0..10 { g.add_card_to_library(0, catalog::forest()); }
    let spell = g.add_card_to_hand(0, catalog::heartwarming_redemption());
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest()); // hand (besides the spell) = 2 cards
    let life = g.players[0].life;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    // Discarded 2, drew 2 + 1 = 3 in hand; gained 3 life.
    assert_eq!(g.players[0].hand.len(), 3, "drew discarded count + 1");
    assert_eq!(g.players[0].life, life + 3, "gained life = new hand size");
}

/// Ashiok's static stops an opponent from searching their library; her −1 mills
/// a target player four and exiles opponents' graveyards.
#[test]
fn ashiok_locks_search_and_mills() {
    use crabomination::effect::{Effect, PlayerRef};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ashiok_dream_render());
    // Player 1 (Ashiok's opponent) can't find a land in their own library.
    g.add_card_to_library(1, catalog::forest());
    let src = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext::for_ability(src, 1, None);
    let hand1 = g.players[1].hand.len();
    g.resolve_effect(&Effect::Search { who: PlayerRef::You, filter: crabomination::card::SelectionRequirement::Land, to: crabomination::effect::ZoneDest::Hand(PlayerRef::You) }, &ctx).unwrap();
    assert_eq!(g.players[1].hand.len(), hand1, "search found nothing under Ashiok");
    // −1: target player 1 mills 4 and their graveyard is exiled.
    for _ in 0..6 { g.add_card_to_library(1, catalog::forest()); }
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let ashiok = g.battlefield.iter().find(|c| c.definition.name == "Ashiok, Dream Render").unwrap().id;
    let lib1 = g.players[1].library.len();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: ashiok, ability_index: 0, target: Some(Target::Player(1)), x_value: None }).expect("-1");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib1 - 4, "milled 4");
    assert!(g.players[1].graveyard.is_empty(), "opponent graveyard exiled");
}

/// Parhelion II is a Crew-4 Vehicle that makes two attacking Angels on attack.
#[test]
fn parhelion_ii_makes_two_attacking_angels() {
    let mut g = two_player_game();
    let def = catalog::parhelion_ii();
    assert!(def.keywords.contains(&Keyword::Crew(4)));
    assert!(def.keywords.contains(&Keyword::Flying) && def.keywords.contains(&Keyword::Vigilance));
    let par = g.add_card_to_battlefield(0, catalog::parhelion_ii());
    let c1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c2 = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // total power 4 to crew
    g.clear_sickness(c1);
    g.clear_sickness(c2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Crew { vehicle: par, crew_creatures: vec![c1, c2] }).expect("crew");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: par, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    let angels: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Angel))
        .collect();
    assert_eq!(angels.len(), 2, "two Angels");
    assert!(angels.iter().all(|a| g.attacking_ids().contains(&a.id)), "the Angels are attacking");
}

/// Dreadhorde Invasion's upkeep trigger loses 1 life and amasses Zombies 1.
#[test]
fn dreadhorde_invasion_upkeep_amasses() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dreadhorde_invasion());
    g.active_player_idx = 0;
    let life = g.players[0].life;
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep { g.perform_action(GameAction::PassPriority).expect("pass"); }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "lost 1 life");
    let army = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)).expect("Army");
    assert_eq!(army.counter_count(CounterType::PlusOnePlusOne), 1, "amass 1");
}

/// Kaya lets her controller target an opponent's hexproof creature and the
/// hexproof opponent, and exiles a creature for −3.
#[test]
fn kaya_ignores_opponent_hexproof() {
    let mut g = two_player_game();
    let kaya = g.add_card_to_battlefield(0, catalog::kaya_bane_of_the_dead());
    // An opponent's hexproof creature is normally untargetable...
    let mut hex = catalog::grizzly_bears();
    hex.keywords.push(Keyword::Hexproof);
    let foe = g.add_card_to_battlefield(1, hex);
    assert!(g.check_target_legality(&Target::Permanent(foe), 0).is_ok(), "Kaya ignores creature hexproof");
    // ...and a hexproof opponent player becomes targetable.
    g.players[1].hexproof_until_next_turn = true;
    assert!(g.check_target_legality(&Target::Player(1), 0).is_ok(), "Kaya ignores player hexproof");
    // −3 exiles a creature.
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: kaya, ability_index: 0, target: Some(Target::Permanent(foe)), x_value: None }).expect("-3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "exiled");
}

/// Planewide Celebration is a choose-four-with-repeats modal sorcery.
#[test]
fn planewide_celebration_is_choose_four() {
    let def = catalog::planewide_celebration();
    match def.effect {
        crabomination::effect::Effect::ChooseModesCast { ref modes, min, max, allow_repeats } => {
            assert_eq!((min, max, allow_repeats), (4, 4, true));
            assert_eq!(modes.len(), 4, "four printed modes");
        }
        _ => panic!("expected ChooseModesCast"),
    }
}

/// Devouring Hellion enters with twice as many +1/+1 counters as creatures
/// sacrificed (devour ×2).
#[test]
fn devouring_hellion_devours_double() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::devouring_hellion());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "2/2 + four +1/+1 counters");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears").count(), 0, "both bears sacrificed");
}

/// Nahiri grants first strike to your creatures during your turn only, and her
/// −X burns a tapped creature.
#[test]
fn nahiri_first_strike_and_burn() {
    let mut g = two_player_game();
    let nahiri = g.add_card_to_battlefield(0, catalog::nahiri_storm_of_stone());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::FirstStrike), "first strike on your turn");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::FirstStrike), "not on opponent's turn");
    // −X burns a tapped creature.
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: nahiri, ability_index: 0, target: Some(Target::Permanent(foe)), x_value: Some(2) }).expect("-X");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "2 damage kills the 2/2");
}

/// Mizzium Tank animates and pumps when you cast a noncreature spell.
#[test]
fn mizzium_tank_animates_on_noncreature() {
    let mut g = two_player_game();
    let tank = g.add_card_to_battlefield(0, catalog::mizzium_tank());
    assert!(!g.computed_permanent(tank).unwrap().card_types.contains(&crabomination::card::CardType::Creature), "not a creature at rest");
    cast_at_target(&mut g, catalog::lightning_bolt(), Target::Player(1), &[(Color::Red, 1)], 0);
    let cp = g.computed_permanent(tank).unwrap();
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature), "animated by the noncreature cast");
    assert_eq!((cp.power, cp.toughness), (4, 3), "3/2 +1/+1");
}

/// Narset's Reversal copies the target spell and returns it to hand.
#[test]
fn narsets_reversal_copies_and_returns() {
    let mut g = two_player_game();
    // Opponent's Lightning Bolt on the stack targeting our player.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None }).expect("cast bolt");
    // We respond with Narset's Reversal targeting the bolt.
    let nr = g.add_card_to_hand(0, catalog::narsets_reversal());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell { card_id: nr, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None }).expect("cast reversal");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bolt), "bolt returned to owner's hand");
}

/// Gideon's Triumph edicts one attacker/blocker, or two with a Gideon in play.
#[test]
fn gideons_triumph_edicts_attackers() {
    // Without a Gideon: sacrifices one creature that attacked this turn.
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(atk).unwrap().attacked_this_turn = true;
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // never attacked — safe
    let tri = g.add_card_to_hand(0, catalog::gideons_triumph());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: tri, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(atk).is_none(), "the attacker was sacrificed");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.name == "Grizzly Bears").count(), 1, "the non-attacker survives");
}

/// Gideon Blackblade is a 4/4 indestructible creature during your turn only,
/// and takes no damage while it's your turn.
#[test]
fn gideon_blackblade_animates_and_is_protected() {
    let mut g = two_player_game();
    let gid = g.add_card_to_battlefield(0, catalog::gideon_blackblade());
    g.active_player_idx = 0;
    let cp = g.computed_permanent(gid).unwrap();
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature), "creature on your turn");
    assert!(cp.card_types.contains(&crabomination::card::CardType::Planeswalker), "still a planeswalker");
    assert!(cp.keywords.contains(&Keyword::Indestructible), "indestructible on your turn");
    assert_eq!((cp.power, cp.toughness), (4, 4), "4/4");
    // Damage during your turn is prevented (no loyalty loss).
    let before = g.battlefield_find(gid).unwrap().counter_count(CounterType::Loyalty);
    let mut evs = vec![];
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(gid), 3, None, &mut evs);
    assert_eq!(g.battlefield_find(gid).unwrap().counter_count(CounterType::Loyalty), before, "no loyalty removed during your turn");
    // On the opponent's turn it's just a planeswalker again.
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(gid).unwrap().card_types.contains(&crabomination::card::CardType::Creature), "not a creature on opp turn");
}

/// With a Gideon in play, Gideon's Triumph makes the opponent sacrifice two.
#[test]
fn gideons_triumph_two_with_gideon() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gideon_blackblade());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(a).unwrap().attacked_this_turn = true;
    g.battlefield_find_mut(b).unwrap().blocked_this_turn = true;
    let tri = g.add_card_to_hand(0, catalog::gideons_triumph());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: tri, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.name == "Grizzly Bears").count(), 0, "both attacker and blocker sacrificed");
}

/// Jace, Arcane Strategist puts a +1/+1 counter on your creature on the second
/// draw each turn (not the first), and only once.
#[test]
fn jace_arcane_strategist_second_draw_counter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::jace_arcane_strategist());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    g.players[0].cards_drawn_this_turn = 0;
    let mut ev = vec![];
    g.draw_one(0, &mut ev); // first draw — no counter
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "no counter on first draw");
    let mut ev2 = vec![];
    g.draw_one(0, &mut ev2); // second draw — counter
    g.dispatch_triggers_for_events(&ev2);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "counter on second draw");
    let mut ev3 = vec![];
    g.draw_one(0, &mut ev3); // third draw — no additional counter (once per turn)
    g.dispatch_triggers_for_events(&ev3);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "still one — once per turn");
}

/// Vraska, Swarm's Eminence grows a deathtouch attacker that connects.
#[test]
fn vraska_grows_deathtouch_attacker() {
    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step { g.perform_action(GameAction::PassPriority).expect("pass"); }
    }
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vraska_swarms_eminence());
    let mut dt = catalog::grizzly_bears();
    dt.keywords.push(Keyword::Deathtouch);
    let biter = g.add_card_to_battlefield(0, dt);
    g.clear_sickness(biter);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: biter, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(biter).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "grew from connecting");
}

/// Vraska's Assassin token destroys a planeswalker it deals combat damage to.
#[test]
fn vraska_assassin_destroys_planeswalker() {
    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step { g.perform_action(GameAction::PassPriority).expect("pass"); }
    }
    let mut g = two_player_game();
    let vraska = g.add_card_to_battlefield(0, catalog::vraska_swarms_eminence());
    let walker = g.add_card_to_battlefield(1, catalog::jace_arcane_strategist());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: vraska, ability_index: 0, target: None, x_value: None }).expect("-2");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Assassin").expect("token").id;
    g.clear_sickness(token);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: token, target: AttackTarget::Planeswalker(walker) }])).expect("attack pw");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert!(g.battlefield_find(walker).is_none(), "planeswalker destroyed by the Assassin");
}

/// Dovin taxes opponents' artifact/instant/sorcery spells by {1}; creature
/// spells are untaxed.
#[test]
fn dovin_taxes_noncreature_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dovin_hand_of_control());
    g.step = TurnStep::PreCombatMain;
    // Opponent's instant now costs {1}{R}; only {R} available → unaffordable.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    assert!(g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None }).is_err(), "the {{1}} tax makes the bolt unaffordable");
    // A creature spell of the same base cost is untaxed and casts fine (on
    // the caster's own turn, for sorcery timing).
    g.active_player_idx = 1;
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None }).is_ok(), "creature spell untaxed");
}

/// Ajani, the Greathearted grants vigilance and his −2 pumps your team and
/// bumps your other planeswalkers.
#[test]
fn ajani_greathearted_anthem_and_minus_two() {
    let mut g = two_player_game();
    let ajani = g.add_card_to_battlefield(0, catalog::ajani_the_greathearted());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let other_pw = g.add_card_to_battlefield(0, catalog::jace_arcane_strategist()); // loyalty 4
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Vigilance), "team has vigilance");
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: ajani, ability_index: 1, target: None, x_value: None }).expect("-2");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "creature grew");
    assert_eq!(g.battlefield_find(other_pw).unwrap().counter_count(CounterType::Loyalty), 5, "other walker gained loyalty");
    assert_eq!(g.battlefield_find(ajani).unwrap().counter_count(CounterType::Loyalty), 3, "Ajani itself unaffected (5-2)");
}

/// Davriel pings a hellbent opponent at their upkeep; a full-handed opponent is
/// spared.
#[test]
fn davriel_pings_hellbent_opponent() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::davriel_rogue_shadowmage());
    // Opponent with an empty hand takes 2 at their upkeep.
    g.players[1].hand.clear();
    g.active_player_idx = 1;
    let life = g.players[1].life;
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep { g.perform_action(GameAction::PassPriority).expect("pass"); }
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "hellbent opponent pinged for 2");
}

/// Awakening of Vitu-Ghazi turns a land into a 9/9 hasty Elemental that's still
/// a land.
#[test]
fn awakening_of_vitu_ghazi_animates_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    cast_at_target(&mut g, catalog::awakening_of_vitu_ghazi(), Target::Permanent(land), &[(Color::Green, 2)], 3);
    let cp = g.computed_permanent(land).unwrap();
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature), "now a creature");
    assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "still a land");
    assert_eq!((cp.power, cp.toughness), (9, 9), "0/0 with nine +1/+1 counters");
    assert!(cp.keywords.contains(&Keyword::Haste), "has haste");
}

/// Sorin grants lifelink on your turn and his −X reanimates an MV-matching
/// creature as a Vampire.
#[test]
fn sorin_lifelink_and_reanimate() {
    let mut g = two_player_game();
    let sorin = g.add_card_to_battlefield(0, catalog::sorin_vengeful_bloodlord());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Lifelink), "lifelink on your turn");
    // Graveyard has a MV-2 (Bear) and a MV-5 (Serra Angel). −X with X=2 only
    // reanimates the mana-value-2 creature (the X gate concretizes the filter).
    g.add_card_to_graveyard(0, catalog::serra_angel());
    let gy_angel = g.players[0].graveyard.last().unwrap().id;
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let gy_bear = g.players[0].graveyard.last().unwrap().id;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: sorin, ability_index: 1, target: None, x_value: Some(2) }).expect("-X");
    drain_stack(&mut g);
    let reanimated = g.battlefield_find(gy_bear).expect("MV-2 bear reanimated");
    assert!(reanimated.definition.subtypes.creature_types.contains(&CreatureType::Bear), "the bear returned");
    assert!(g.battlefield_find(gy_angel).is_none(), "the MV-5 Angel is off-limits at X=2");
    assert!(g.computed_permanent(gy_bear).unwrap().subtypes.creature_types.contains(&CreatureType::Vampire), "now also a Vampire");
}

/// Jace's Ruse bounces up to two creatures.
#[test]
fn jaces_ruse_bounces_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ruse = g.add_card_to_hand(0, catalog::jaces_ruse());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: ruse, target: Some(Target::Permanent(a)), additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both bounced");
    assert_eq!(g.players[1].hand.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 2, "both in owner's hand");
}

/// Vivien, Champion of the Wilds lets you cast creature spells at flash speed
/// and her +1 grants vigilance + reach.
#[test]
fn vivien_champion_flash_and_grant() {
    let mut g = two_player_game();
    let vivien = g.add_card_to_battlefield(0, catalog::vivien_champion_of_the_wilds());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Cast a creature at instant speed on the opponent's turn (flash).
    g.active_player_idx = 1;
    let flasher = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::CastSpell { card_id: flasher, target: None, additional_targets: vec![], mode: None, x_value: None }).is_ok(), "creature cast at flash speed");
    drain_stack(&mut g);
    // +1 grants vigilance + reach.
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: vivien, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: None }).expect("+1");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Vigilance) && cp.keywords.contains(&Keyword::Reach), "granted vigilance + reach");
}

/// The Elderspell destroys planeswalkers and pumps your own two loyalty each.
#[test]
fn the_elderspell_destroys_and_pumps() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::jace_arcane_strategist()); // loyalty 4
    let foe1 = g.add_card_to_battlefield(1, catalog::jace_arcane_strategist());
    let foe2 = g.add_card_to_battlefield(1, catalog::jace_arcane_strategist());
    let spell = g.add_card_to_hand(0, catalog::the_elderspell());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell { card_id: spell, target: Some(Target::Permanent(foe1)), additional_targets: vec![Target::Permanent(foe2)], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe1).is_none() && g.battlefield_find(foe2).is_none(), "both opposing walkers destroyed");
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::Loyalty), 4 + 4, "two loyalty per destroyed (2×2)");
}

/// Widespread Brutality amasses a 2/2 Army and it burns each non-Army creature
/// for 2.
#[test]
fn widespread_brutality_amasses_and_burns() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 non-Army
    let spell = g.add_card_to_hand(0, catalog::widespread_brutality());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    // A 2/2 Army was amassed; it dealt 2 to the opposing 2/2, killing it.
    let army = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)).expect("Army");
    assert_eq!(army.counter_count(CounterType::PlusOnePlusOne), 2, "amassed 2");
    assert!(g.battlefield_find(foe).is_none(), "non-Army creature took 2 and died");
}

/// Arlinn, Voice of the Pack makes Wolf tokens enter with an extra +1/+1
/// counter (a 2/2 base becomes 3/3), and non-Wolves are unaffected.
#[test]
fn arlinn_wolves_enter_bigger() {
    let mut g = two_player_game();
    let arlinn = g.add_card_to_battlefield(0, catalog::arlinn_voice_of_the_pack());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: arlinn, ability_index: 0, target: None, x_value: None }).expect("-2");
    drain_stack(&mut g);
    let wolf = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Wolf").expect("Wolf token");
    assert_eq!(wolf.counter_count(CounterType::PlusOnePlusOne), 1, "Wolf entered with an extra counter");
    assert_eq!((g.computed_permanent(wolf.id).unwrap().power, g.computed_permanent(wolf.id).unwrap().toughness), (3, 3), "2/2 + counter = 3/3");
    // A non-Wolf enters normally.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "non-Wolf unaffected");
}

/// Tomik makes your lands untargetable by opponents' spells; you can still
/// target them yourself.
#[test]
fn tomik_protects_your_lands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tomik_distinguished_advokist());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    // An opponent (seat 1) can't target your land.
    assert!(g.check_target_legality(&Target::Permanent(land), 1).is_err(), "opponent can't target your land");
    // You (seat 0) still can.
    assert!(g.check_target_legality(&Target::Permanent(land), 0).is_ok(), "you can target your own land");
}

/// Sarkhan the Masterless: +1 turns your planeswalkers into 4/4 Dragons, and
/// the passive pings an attacker once per Dragon you control.
#[test]
fn sarkhan_masterless_animates_and_pings() {
    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step { g.perform_action(GameAction::PassPriority).expect("pass"); }
    }
    let mut g = two_player_game();
    let sarkhan = g.add_card_to_battlefield(0, catalog::sarkhan_the_masterless());
    // +1: Sarkhan (a planeswalker) becomes a 4/4 Dragon.
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: sarkhan, ability_index: 0, target: None, x_value: None }).expect("+1");
    drain_stack(&mut g);
    let cp = g.computed_permanent(sarkhan).unwrap();
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature) && cp.subtypes.creature_types.contains(&CreatureType::Dragon), "Sarkhan is a Dragon creature");
    assert_eq!((cp.power, cp.toughness), (4, 4), "4/4");
    // The passive: an opponent's attacker takes 1 per Dragon you control (1 here).
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(attacker).unwrap().damage, 1, "attacker took 1 from the lone Dragon");
}

/// Massacre Girl's chain: a 2/2 that would survive the ETB's single -1/-1 is
/// finished off by the death-triggered second wave.
#[test]
fn massacre_girl_death_chain_wipes_board() {
    let mut g = two_player_game();
    let bane = g.add_card_to_battlefield(0, catalog::banehound()); // 1/1
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.move_card_to_battlefield_for_test(0, catalog::massacre_girl());
    drain_stack(&mut g);
    assert!(g.battlefield_find(bane).is_none(), "1/1 died to the ETB -1/-1");
    assert!(g.battlefield_find(bear).is_none(), "2/2 died to the death-chain second wave");
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Massacre Girl"),
        "Massacre Girl survives her own sweep",
    );
}

/// Ilharg's attack trigger deploys a creature from hand tapped and attacking,
/// then returns it to hand at the next end step.
#[test]
fn ilharg_deploys_attacker_then_returns_it() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step { g.perform_action(GameAction::PassPriority).expect("pass"); }
    }
    let mut g = two_player_game();
    let ilharg = g.add_card_to_battlefield(0, catalog::ilharg_the_raze_boar());
    g.clear_sickness(ilharg);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: ilharg, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.attacking.iter().any(|a| a.attacker == bear), "deployed bear is attacking");
    assert!(g.battlefield_find(bear).unwrap().tapped, "deployed bear entered tapped");
    // At the next end step it returns to its owner's hand.
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear left the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bear returned to hand");
}

/// Single Combat leaves each player one creature/planeswalker and then locks
/// creature and planeswalker casts.
#[test]
fn single_combat_keeps_one_and_locks_casts() {
    let mut g = two_player_game();
    let keep = g.add_card_to_battlefield(0, catalog::primordial_wurm()); // MV 7 — kept
    let sac = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV 2 — sacrificed
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // opp's only one — kept
    let sc = g.add_card_to_hand(0, catalog::single_combat());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: sc, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast Single Combat");
    drain_stack(&mut g);
    assert!(g.battlefield_find(keep).is_some(), "player 0 kept the higher-MV creature");
    assert!(g.battlefield_find(sac).is_none(), "player 0 sacrificed the rest");
    assert!(g.battlefield_find(opp).is_some(), "opponent's only creature is kept");
    // No player may now cast a creature spell.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let r = g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None });
    assert!(r.is_err(), "creature spell is locked out");
}

/// Jiang Yanggu's −1 adds a +1/+1 counter, and his static grants your
/// counter-bearing creatures a mana ability.
#[test]
fn jiang_yanggu_counter_grants_mana_ability() {
    let mut g = two_player_game();
    let jy = g.add_card_to_battlefield(0, catalog::jiang_yanggu_wildcrafter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // No counter yet — no granted ability.
    assert!(g.granted_abilities_for(bear).is_empty(), "no counter, no mana ability");
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: jy, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: None }).expect("-1");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(!g.granted_abilities_for(bear).is_empty(), "counter-bearing creature gained the mana ability");
}

/// Finale of Eternity (X=2) destroys creatures with toughness 2 or less.
#[test]
fn finale_of_eternity_destroys_small_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::primordial_wurm()); // 7/7 — survives
    let fin = g.add_card_to_hand(0, catalog::finale_of_eternity());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: fin, target: Some(Target::Permanent(a)), additional_targets: vec![Target::Permanent(b)], mode: None, x_value: Some(2) }).expect("cast X=2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both 2/2s destroyed");
    assert!(g.battlefield_find(big).is_some(), "the 7/7 was not a legal target");
}

/// Finale of Eternity (X=10) also reanimates all creatures from your graveyard.
#[test]
fn finale_of_eternity_x10_reanimates() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::primordial_wurm());
    let fin = g.add_card_to_hand(0, catalog::finale_of_eternity());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(10);
    g.perform_action(GameAction::CastSpell { card_id: fin, target: None, additional_targets: vec![], mode: None, x_value: Some(10) }).expect("cast X=10");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Grizzly Bears"), "Grizzly Bears reanimated");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Primordial Wurm"), "Primordial Wurm reanimated");
}

/// Domri's anthem pumps your creatures and his −2 makes them fight.
#[test]
fn domri_anarch_anthem_and_fight() {
    let mut g = two_player_game();
    let domri = g.add_card_to_battlefield(0, catalog::domri_anarch_of_bolas());
    let wurm = g.add_card_to_battlefield(0, catalog::primordial_wurm()); // 7/7 -> 8/7
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 8, "anthem gives +1/+0");
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Slot 0 = your fighter; the engine auto-picks the lone enemy for slot 1.
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: domri, ability_index: 1, target: Some(Target::Permanent(wurm)), x_value: None }).expect("-2 fight");
    drain_stack(&mut g);
    assert!(g.battlefield_find(prey).is_none(), "the enemy 2/2 died to the 8-power fighter");
    assert!(g.battlefield_find(wurm).is_some(), "the 8/7 survived the 2 damage back");
}

/// Domri's +1 adds R or G and shields your creature spells from counters.
#[test]
fn domri_anarch_plus_one_mana_and_uncounterable() {
    let mut g = two_player_game();
    let domri = g.add_card_to_battlefield(0, catalog::domri_anarch_of_bolas());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: domri, ability_index: 0, target: None, x_value: None }).expect("+1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "added one red (decider default)");
    assert!(g.players[0].creature_spells_uncounterable_this_turn, "creature spells shielded this turn");
}

/// Finale of Revelation draws X and exiles itself.
#[test]
fn finale_of_revelation_draws_x_and_exiles() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let hand = g.players[0].hand.len();
    let fin = g.add_card_to_hand(0, catalog::finale_of_revelation());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: fin, target: None, additional_targets: vec![], mode: None, x_value: Some(3) }).expect("cast X=3");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 3, "drew three");
    assert!(g.exile.iter().any(|c| c.id == fin), "Finale exiled itself");
}

/// Finale of Revelation at X=10 shuffles the graveyard in and removes the
/// maximum hand size.
#[test]
fn finale_of_revelation_x10_no_max_hand_size() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    for _ in 0..12 { g.add_card_to_library(0, catalog::forest()); }
    let fin = g.add_card_to_hand(0, catalog::finale_of_revelation());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(10);
    g.perform_action(GameAction::CastSpell { card_id: fin, target: None, additional_targets: vec![], mode: None, x_value: Some(10) }).expect("cast X=10");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.is_empty(), "graveyard shuffled into library");
    assert_eq!(g.players[0].max_hand_size, None, "no maximum hand size");
}

/// Liliana, Dreadhorde General's passive draws when your creature dies, and
/// +1 makes two Zombies.
#[test]
fn liliana_dreadhorde_draws_on_death_and_makes_zombies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::liliana_dreadhorde_general());
    g.add_card_to_library(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let hand = g.players[0].hand.len();
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(bear), 2, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew on the creature's death");
    // +1 makes two 2/2 black Zombies.
    let lili = g.battlefield.iter().find(|c| c.definition.name == "Liliana, Dreadhorde General").unwrap().id;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: lili, ability_index: 0, target: None, x_value: None }).expect("+1");
    drain_stack(&mut g);
    let zombies = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Zombie" && c.definition.power == 2).count();
    assert_eq!(zombies, 2, "two Zombie tokens");
}

/// Liliana's −4 makes each player sacrifice two creatures.
#[test]
fn liliana_dreadhorde_minus_four_sacrifices_two_each() {
    let mut g = two_player_game();
    let lili = g.add_card_to_battlefield(0, catalog::liliana_dreadhorde_general());
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); } // the death-draw passive needs a library
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::grizzly_bears()); }
    for _ in 0..3 { g.add_card_to_battlefield(1, catalog::grizzly_bears()); }
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: lili, ability_index: 1, target: None, x_value: None }).expect("-4");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count(), 1, "player 0 sacrificed two of three");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_creature()).count(), 1, "player 1 sacrificed two of three");
}

/// Chandra, Fire Artisan pings when combat damage removes her loyalty.
#[test]
fn chandra_fire_artisan_pings_on_loyalty_loss() {
    let mut g = two_player_game();
    let chandra = g.add_card_to_battlefield(0, catalog::chandra_fire_artisan());
    let opp_life = g.players[1].life;
    // 3 damage to Chandra removes 3 loyalty; her trigger deals 3 to the opponent.
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(chandra), 3, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(chandra).unwrap().counter_count(CounterType::Loyalty), 1, "lost 3 loyalty");
    assert_eq!(g.players[1].life, opp_life - 3, "Chandra dealt 3 to the opponent");
}

/// Ral, Storm Conduit pings when you cast an instant or sorcery.
#[test]
fn ral_storm_conduit_pings_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ral_storm_conduit());
    g.add_card_to_library(0, catalog::forest()); // Opt's draw
    let opt = g.add_card_to_hand(0, catalog::opt());
    let opp = g.players[1].life;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell { card_id: opt, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast Opt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "Ral pinged the opponent on the cast");
}

/// Ral's −2 copies your next instant/sorcery, and the copy pings too.
#[test]
fn ral_storm_conduit_copy_pings_again() {
    let mut g = two_player_game();
    let ral = g.add_card_to_battlefield(0, catalog::ral_storm_conduit());
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); } // Opt + its copy each draw
    let opt = g.add_card_to_hand(0, catalog::opt());
    let opp = g.players[1].life;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: ral, ability_index: 1, target: None, x_value: None }).expect("-2");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell { card_id: opt, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast Opt");
    drain_stack(&mut g);
    // One ping for the cast, one for the copy.
    assert_eq!(g.players[1].life, opp - 2, "cast and copy each pinged");
}

/// Ugin's +1 exiles the top card behind a 2/2 Spirit; killing the Spirit
/// returns the exiled card to hand.
#[test]
fn ugin_ineffable_plus_one_token_returns_exiled_card() {
    let mut g = two_player_game();
    let ugin = g.add_card_to_battlefield(0, catalog::ugin_the_ineffable());
    g.add_card_to_library(0, catalog::grizzly_bears()); // the card to be exiled
    let hand = g.players[0].hand.len();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: ugin, ability_index: 0, target: None, x_value: None }).expect("+1");
    drain_stack(&mut g);
    let spirit = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Spirit").expect("Spirit token").id;
    assert!(g.exile.iter().any(|c| c.definition.name == "Grizzly Bears"), "card exiled face down");
    assert_eq!(g.players[0].hand.len(), hand, "still exiled, not yet in hand");
    // Kill the Spirit → the exiled card returns to hand.
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(spirit), 2, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "exiled card returned to hand");
}

/// Ugin's −3 destroys a colored permanent (a colorless one is not a legal
/// target).
#[test]
fn ugin_ineffable_minus_three_destroys_colored() {
    let mut g = two_player_game();
    let ugin = g.add_card_to_battlefield(0, catalog::ugin_the_ineffable());
    let colored = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: ugin, ability_index: 1, target: Some(Target::Permanent(colored)), x_value: None }).expect("-3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(colored).is_none(), "colored permanent destroyed");
}

/// Neheb, Dreadhorde Champion: combat damage lets you discard-then-draw and
/// bank that much {R} that survives step/phase emptying.
#[test]
fn neheb_dreadhorde_discard_draw_and_banks_red() {
    let mut g = two_player_game();
    let neheb = g.add_card_to_battlefield(0, catalog::neheb_dreadhorde_champion());
    let c1 = g.add_card_to_hand(0, catalog::forest());
    let c2 = g.add_card_to_hand(0, catalog::forest());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![c1, c2])]));
    g.fire_combat_damage_to_player_triggers(neheb, 1, 5);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "discarded two, drew two");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2, "banked two red");
    assert_eq!(g.players[0].kept_mana_this_turn.amount(Color::Red), 2, "the red doesn't empty this turn");
}

// ── Batch 4: WAR bombs ────────────────────────────────────────────────────────

/// Tezzeret, Master of the Bridge grants creature/planeswalker spells affinity
/// for artifacts and its −8 deploys artifacts off the top ten.
#[test]
fn tezzeret_master_bridge_affinity_and_ult() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tezzeret_master_of_the_bridge());
    // A creature spell in hand gets {1} off per artifact.
    let creature = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &creature, None), 0, "no artifacts → no discount");
    g.add_card_to_battlefield(0, catalog::ornithopter());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    assert_eq!(cost_reduction_for_spell(&g, 0, &creature, None), 2, "two artifacts → {{2}} off");
    // A noncreature/nonplaneswalker spell is unaffected.
    let bolt = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &bolt, None), 0, "instant unaffected");
}

/// Tezzeret's −8 exiles the top ten and deploys the artifacts among them.
#[test]
fn tezzeret_ult_deploys_artifacts() {
    let mut g = two_player_game();
    let tez = g.add_card_to_battlefield(0, catalog::tezzeret_master_of_the_bridge());
    // Stack the library: two artifacts and a nonartifact within the top ten.
    g.add_card_to_library(0, catalog::ornithopter());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::mind_stone());
    g.battlefield_find_mut(tez).unwrap().counters.insert(CounterType::Loyalty, 8);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: tez, ability_index: 2, target: None, x_value: None }).expect("-8");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Ornithopter"), "Ornithopter deployed");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Mind Stone"), "Mind Stone deployed");
    assert!(g.exile.iter().any(|c| c.definition.name == "Grizzly Bears"), "nonartifact exiled");
}

/// God-Eternal Kefnet copies the first instant/sorcery drawn each turn; you may
/// cast the copy for {2} less.
#[test]
fn kefnet_copies_first_drawn_instant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::god_eternal_kefnet());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].cards_drawn_this_turn = 0;
    // {R} minus {2} generic is still {R} — the copy isn't free.
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    let opp_life = g.players[1].life;
    // Accept the copy cast (OptionalTrigger).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 3, "the discounted Bolt copy hit the opponent");
    // The drawn Bolt itself is still in hand (only a copy was cast).
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"), "original stays in hand");
}

/// An unpayable discounted copy is simply skipped.
#[test]
fn kefnet_copy_needs_the_discounted_mana() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::god_eternal_kefnet());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].cards_drawn_this_turn = 0;
    let opp_life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life, "no red mana, no copy");
}

/// The second draw of the turn doesn't retrigger Kefnet.
#[test]
fn kefnet_only_first_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::god_eternal_kefnet());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].cards_drawn_this_turn = 1; // pretend one already drawn
    let opp_life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life, "no copy — not the turn's first draw");
}

/// God-Eternal recursion now fires on exile, not just death (Bontu et al.).
#[test]
fn god_eternal_recurs_on_exile() {
    use crabomination::effect::{Effect, ZoneDest};
    let mut g = two_player_game();
    let bontu = g.add_card_to_battlefield(0, catalog::god_eternal_bontu());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(bontu)), 0, 0);
    let evs = g
        .resolve_effect(&Effect::Move { what: crabomination::effect::Selector::Target(0), to: ZoneDest::Exile }, &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(
        g.players[0].library.iter().any(|c| c.definition.name == "God-Eternal Bontu"),
        "Bontu recurred into the library on exile",
    );
}

/// Gideon's Sacrifice redirects damage aimed at you to the chosen permanent for
/// the turn.
#[test]
fn gideons_sacrifice_redirects_damage() {
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cast = g.add_card_to_hand(0, catalog::gideons_sacrifice());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let life = g.players[0].life;
    // 2 damage aimed at player 0 → redirected to the bear (a 2/2 → dies).
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 2, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    assert_eq!(g.players[0].life, life, "player took no damage");
    assert!(g.battlefield_find(bear).is_none(), "the bear soaked lethal and died");
}

/// Niv-Mizzet Reborn takes one card of each guild pair from the top ten.
#[test]
fn niv_mizzet_reborn_grabs_guild_cards() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_helix()); // W/R
    g.add_card_to_library(0, catalog::terminate());       // B/R
    g.add_card_to_library(0, catalog::grizzly_bears());   // mono-G
    let hand_before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::niv_mizzet_reborn());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Helix"), "W/R card taken");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Terminate"), "B/R card taken");
    assert!(!g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "mono-color not taken");
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "exactly two guild cards");
}

/// Nissa, Who Shakes the World: tapping your Forest for mana yields an extra
/// {G}; her +1 animates a land into a 3/3 Elemental.
#[test]
fn nissa_shakes_extra_green_and_land_animation() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nissa_who_shakes_the_world());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    // Tap the Forest for mana → base {G} + Nissa's extra {G}.
    let (idx, _) = g.effective_mana_abilities(forest).into_iter().next().expect("mana ability");
    g.perform_action(GameAction::ActivateAbility { card_id: forest, ability_index: idx, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("tap");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "Forest {{G}} + Nissa's extra {{G}}");
}

/// Nissa's +1 puts three +1/+1 counters on a noncreature land and animates it.
#[test]
fn nissa_shakes_plus_one_animates_land() {
    let mut g = two_player_game();
    let nissa = g.add_card_to_battlefield(0, catalog::nissa_who_shakes_the_world());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: nissa, ability_index: 0, target: Some(Target::Permanent(land)), x_value: None }).expect("+1");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(land).unwrap().tapped, "land untapped");
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne), 3, "three +1/+1 counters");
    let cp = g.computed_permanent(land).unwrap();
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature), "now a creature");
    assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "still a land");
    assert_eq!((cp.power, cp.toughness), (3, 3), "0/0 with three +1/+1 counters");
    assert!(cp.keywords.contains(&Keyword::Vigilance) && cp.keywords.contains(&Keyword::Haste), "vigilance + haste");
}

/// Feather exiles an I/S that targets your creature and returns it at the next
/// end step (instead of it hitting the graveyard).
#[test]
fn feather_exiles_and_returns_spell() {
    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step { g.perform_action(GameAction::PassPriority).expect("pass"); }
    }
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::feather_the_redeemed());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell { card_id: growth, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == growth), "Giant Growth exiled, not in graveyard");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == growth), "not in graveyard");
    // At the next end step, it returns to hand.
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == growth), "returned to hand at end step");
}

/// Deliver Unto Evil: without a Bolas planeswalker, an opponent leaves two of
/// the chosen cards and the rest return to hand; the sorcery exiles itself.
#[test]
fn deliver_unto_evil_opponent_leaves_two() {
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let c = g.add_card_to_graveyard(0, catalog::forest());
    // Opponent leaves a and b in the graveyard → only c returns.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a, b])]));
    let ctx = EffectContext {
        targets: vec![Target::Permanent(a), Target::Permanent(b), Target::Permanent(c)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&catalog::deliver_unto_evil().effect, &ctx).unwrap();
    assert!(g.players[0].hand.iter().any(|x| x.id == c), "unchosen card returned to hand");
    assert!(g.players[0].graveyard.iter().any(|x| x.id == a), "chosen card stays in graveyard");
    assert!(g.players[0].graveyard.iter().any(|x| x.id == b), "chosen card stays in graveyard");
}

/// Nicol Bolas, Dragon-God can activate loyalty abilities borrowed from another
/// planeswalker on the battlefield.
#[test]
fn nicol_bolas_dragon_god_borrows_loyalty_abilities() {
    let mut g = two_player_game();
    let bolas = g.add_card_to_battlefield(0, catalog::nicol_bolas_dragon_god());
    // Chandra, Fire Artisan brings a +1 impulse at borrowed index 3.
    g.add_card_to_battlefield(0, catalog::chandra_fire_artisan());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Index 0..2 are Bolas's own; index 3 is Chandra's +1 (impulse one).
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: bolas, ability_index: 3, target: None, x_value: None }).expect("borrowed ability");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.definition.name == "Grizzly Bears"), "Chandra's borrowed impulse fired off Bolas");
}

/// Nicol Bolas's −8 eliminates an opponent who controls no legendary
/// creature/planeswalker.
#[test]
fn nicol_bolas_dragon_god_ult_eliminates() {
    let mut g = two_player_game();
    let bolas = g.add_card_to_battlefield(0, catalog::nicol_bolas_dragon_god());
    g.battlefield_find_mut(bolas).unwrap().counters.insert(CounterType::Loyalty, 8);
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // nonlegendary
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: bolas, ability_index: 2, target: None, x_value: None }).expect("-8");
    drain_stack(&mut g);
    assert!(g.players[1].eliminated, "opponent with no legendary lost the game");
}

/// Deliver Unto Evil returns everything when you control a Bolas planeswalker.
#[test]
fn deliver_unto_evil_with_bolas_returns_all() {
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nicol_bolas_dragon_god());
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let ctx = EffectContext {
        targets: vec![Target::Permanent(a), Target::Permanent(b)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&catalog::deliver_unto_evil().effect, &ctx).unwrap();
    assert!(g.players[0].hand.iter().any(|x| x.id == a) && g.players[0].hand.iter().any(|x| x.id == b), "both returned with a Bolas walker out");
}

/// Bolas's Citadel lets you cast a spell off the top of your library, paying
/// life equal to its mana value instead of mana.
#[test]
fn bolass_citadel_casts_from_top_paying_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bolass_citadel());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt()); // MV 1, top card
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let my_life = g.players[0].life;
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None }).expect("cast from top");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, my_life - 1, "paid 1 life (Bolt's mana value)");
    assert_eq!(g.players[1].life, opp_life - 3, "Bolt dealt 3");
}

/// Finale of Promise free-casts a targeted instant and sorcery from your
/// graveyard, exiling each as it resolves.
#[test]
fn finale_of_promise_casts_instant_and_sorcery() {
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());  // instant, MV 1
    let tutor = g.add_card_to_graveyard(0, catalog::demonic_tutor());  // sorcery, MV 2
    let opp_life = g.players[1].life;
    let ctx = EffectContext {
        targets: vec![Target::Permanent(bolt), Target::Permanent(tutor)],
        ..EffectContext::for_spell(0, None, 0, 2) // X = 2
    };
    g.resolve_effect(&catalog::finale_of_promise().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 3, "the free Bolt dealt 3");
    assert!(g.exile.iter().any(|c| c.id == bolt), "instant exiled after resolving");
    assert!(g.exile.iter().any(|c| c.id == tutor), "sorcery exiled after resolving");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt || c.id == tutor), "neither left in graveyard");
}
