//! Functionality tests for War of the Spark (WAR) — `catalog::sets::war`.

use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
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
        card_id: enf, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None,
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
        card_id: weird, ability_index: 0, target: Some(Target::Permanent(bolt)), additional_targets: vec![], x_value: None,
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
    g.perform_action(GameAction::ActivateAbility { card_id: cat, ability_index: 0, target: None, additional_targets: vec![], x_value: None }).expect("sac");
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
    g.perform_action(GameAction::ActivateAbility { card_id: sd, ability_index: 0, target: None, additional_targets: vec![], x_value: None }).expect("draw");
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
        card_id: grizzly, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
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
        card_id: grizzly, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(land), "land bottomed");
    assert!(!g.players[0].hand.iter().any(|c| c.id == land), "land not drawn");
}
