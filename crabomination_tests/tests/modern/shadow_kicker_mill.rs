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

// ── Shadow / artifact-staples batch ──────────────────────────────────────────

#[test]
fn shadow_creatures_only_block_each_other() {
    use crabomination::card::Keyword;
    let slayer = catalog::dauthi_slayer();
    assert!(slayer.keywords.contains(&Keyword::Shadow));
    assert!(slayer.keywords.contains(&Keyword::MustAttack));
    let mut g = two_player_game();
    let dauthi = g.add_card_to_battlefield(0, catalog::dauthi_slayer());
    g.clear_sickness(dauthi);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::Attack {
        attacker: dauthi,
        target: crabomination::game::AttackTarget::Player(1),
    }])).unwrap();
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    // CR 702.28b — a non-shadow creature can't block the Dauthi.
    let r = g.declare_blockers(vec![(bear, dauthi)]);
    assert!(r.is_err(), "non-shadow creature must not block a shadow attacker");
}

#[test]
fn soltari_champion_pumps_other_attackers() {
    let mut g = two_player_game();
    let champ = g.add_card_to_battlefield(0, catalog::soltari_champion());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let trig = catalog::soltari_champion().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(champ, 0, None, 0);
    let _ = g.resolve_effect(&trig, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 3, "other creature +1/+1");
    assert_eq!(g.battlefield_find(champ).unwrap().power(), 2, "champion itself untouched");
}

#[test]
fn simic_ascendancy_accrues_growth_and_wins_at_twenty() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let asc = g.add_card_to_battlefield(0, catalog::simic_ascendancy());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Three +1/+1 counters land on the bear → three growth counters.
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    g.dispatch_triggers_for_events(&[GameEvent::CounterAdded {
        card_id: bear,
        counter_type: CounterType::PlusOnePlusOne,
        count: 3,
    }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(asc).unwrap().counter_count(CounterType::Growth),
        3,
        "growth counters track the batch size"
    );
    // At twenty growth counters the upkeep check wins the game.
    g.battlefield_find_mut(asc).unwrap().add_counters(CounterType::Growth, 17);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.is_game_over(), "twenty growth counters win the game");
}

#[test]
fn myr_retriever_dies_returns_an_artifact_from_graveyard() {
    let mut g = two_player_game();
    let myr = g.add_card_to_battlefield(0, catalog::myr_retriever());
    let key = g.add_card_to_graveyard(0, catalog::voltaic_key());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(key))]));
    let mut events = Vec::new();
    g.sacrifice_one(myr, 0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == key), "Voltaic Key returned to hand");
}

#[test]
fn time_sieve_sacrifices_five_artifacts_for_extra_turn() {
    let mut g = two_player_game();
    let sieve = g.add_card_to_battlefield(0, catalog::time_sieve());
    let fodder: Vec<_> = (0..5).map(|_| g.add_card_to_battlefield(0, catalog::voltaic_key())).collect();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sieve, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activation");
    drain_stack(&mut g);
    assert_eq!(g.players[0].extra_turns, 1);
    for f in fodder {
        assert!(g.battlefield_find(f).is_none(), "fodder sacrificed");
    }
}

#[test]
fn ichor_wellspring_draws_on_etb_and_on_death() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let well = g.add_card_to_battlefield(0, catalog::ichor_wellspring());
    let hand_before = g.players[0].hand.len();
    let mut events = Vec::new();
    g.sacrifice_one(well, 0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "death draw fired");
}

#[test]
fn jace_beleren_plus_two_draws_for_each_player() {
    let mut g = two_player_game();
    for p in 0..2 {
        for _ in 0..3 { g.add_card_to_library(p, catalog::forest()); }
    }
    let jace = g.add_card_to_battlefield(0, catalog::jace_beleren());
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: jace, ability_index: 0, target: None, x_value: None,
    }).expect("loyalty +2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 + 1);
    assert_eq!(g.players[1].hand.len(), h1 + 1);
    assert_eq!(
        g.battlefield_find(jace).unwrap().counter_count(crabomination::card::CounterType::Loyalty),
        5
    );
}

#[test]
fn darksteel_colossus_shuffles_into_library_instead_of_graveyard() {
    let mut g = two_player_game();
    let colossus = g.add_card_to_battlefield(0, catalog::darksteel_colossus());
    let lib_before = g.players[0].library.len();
    let mut events = Vec::new();
    g.sacrifice_one(colossus, 0, &mut events);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != colossus), "never hits the graveyard");
    assert_eq!(g.players[0].library.len(), lib_before + 1, "shuffled into the library");
}

#[test]
fn open_the_vaults_returns_artifacts_and_enchantments_from_all_graveyards() {
    let mut g = two_player_game();
    let my_art = g.add_card_to_graveyard(0, catalog::voltaic_key());
    let opp_art = g.add_card_to_graveyard(1, catalog::ichor_wellspring());
    let creature = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let effect = catalog::open_the_vaults().effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    let mine = g.battlefield_find(my_art).expect("my artifact returned");
    assert_eq!(mine.controller, 0);
    let theirs = g.battlefield_find(opp_art).expect("opp artifact returned");
    assert_eq!(theirs.controller, 1, "returns under its owner's control");
    assert!(g.battlefield_find(creature).is_none(), "creatures stay dead");
}

#[test]
fn spreading_seas_turns_enchanted_land_into_an_island() {
    use crabomination::card::LandType;
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let seas = g.add_card_to_hand(0, catalog::spreading_seas());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len() - 1;
    g.perform_action(GameAction::CastSpell {
        card_id: seas,
        target: Some(Target::Permanent(mountain)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "ETB draw");
    let cp = g.compute_battlefield();
    let v = cp.iter().find(|c| c.id == mountain).unwrap();
    assert!(v.subtypes.land_types.contains(&LandType::Island), "now an Island");
    assert!(!v.subtypes.land_types.contains(&LandType::Mountain), "Mountain type replaced");
}

// ── Praetor cycle + utility lands ────────────────────────────────────────────

#[test]
fn elesh_norn_anthems_both_sides() {
    let mut g = two_player_game();
    let _norn = g.add_card_to_battlefield(0, catalog::elesh_norn_grand_cenobite());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cp = g.compute_battlefield();
    let m = cp.iter().find(|c| c.id == mine).unwrap();
    assert_eq!((m.power, m.toughness), (4, 4), "your bear +2/+2");
    let t = cp.iter().find(|c| c.id == theirs).unwrap();
    assert_eq!((t.power, t.toughness), (0, 0), "opp bear -2/-2");
    g.check_state_based_actions();
    assert!(g.battlefield_find(theirs).is_none(), "0-toughness bear dies to SBA");
}

#[test]
fn urabrask_grants_haste_and_taps_opponent_creatures() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let _ura = g.add_card_to_battlefield(0, catalog::urabrask_the_hidden());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.compute_battlefield();
    assert!(cp.iter().find(|c| c.id == mine).unwrap().keywords.contains(&Keyword::Haste));
    // An opponent's creature enters tapped (CR 614.13 replacement).
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    let opp_bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: opp_bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp_bear).unwrap().tapped, "enters tapped under Urabrask");
}

#[test]
fn vorinclex_locks_an_opponent_land_after_it_taps_for_mana() {
    let mut g = two_player_game();
    let _vor = g.add_card_to_battlefield(0, catalog::vorinclex_voice_of_hunger());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().skip_next_untap,
        "the tapped land is flagged to skip its next untap");
}

#[test]
fn jin_gitaxias_reduces_opponent_max_hand_size() {
    let mut g = two_player_game();
    let _jin = g.add_card_to_battlefield(0, catalog::jin_gitaxias_core_augur());
    assert_eq!(g.effective_max_hand_size(1), Some(0), "7 - 7 = 0 for the opponent");
    assert_eq!(g.effective_max_hand_size(0), Some(7), "controller unaffected");
}

#[test]
fn aven_riftwatcher_gains_life_on_entry_and_exit() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let aven = g.add_card_to_hand(0, catalog::aven_riftwatcher());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aven, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "ETB gain");
    assert_eq!(
        g.battlefield_find(aven).unwrap().counter_count(crabomination::card::CounterType::Time),
        2,
        "vanishing 2"
    );
    let mut events = Vec::new();
    g.sacrifice_one(aven, 0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "leave-the-battlefield gain");
}

#[test]
fn den_of_the_bugbear_attacks_and_mints_an_attacking_goblin() {
    let mut g = two_player_game();
    let den = g.add_card_to_battlefield(0, catalog::den_of_the_bugbear());
    g.clear_sickness(den);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: den, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).unwrap();
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::Attack {
        attacker: den,
        target: crabomination::game::AttackTarget::Player(1),
    }])).expect("animated Den attacks");
    drain_stack(&mut g);
    let goblin = g.battlefield.iter().find(|c| c.is_token && c.controller == 0)
        .expect("a Goblin token was minted");
    assert!(g.attacking().iter().any(|a| a.attacker == goblin.id), "token enters attacking");
}

#[test]
fn sunscorched_desert_pings_on_entry() {
    let mut g = two_player_game();
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    let desert = g.add_card_to_hand(0, catalog::sunscorched_desert());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(desert)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "ETB ping");
}

// ── Mill + landfall batch ────────────────────────────────────────────────────

#[test]
fn ruin_crab_landfall_mills_each_opponent() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(1, catalog::forest()); }
    let _crab = g.add_card_to_battlefield(0, catalog::ruin_crab());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 3, "opponent milled three");
}

#[test]
fn fractured_sanity_cycle_trigger_mills_four() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(1, catalog::forest()); }
    g.add_card_to_library(0, catalog::forest());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let fs = g.add_card_to_hand(0, catalog::fractured_sanity());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Cycle { card_id: fs, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 4, "cycle trigger milled four");
}

#[test]
fn tashas_hideous_laughter_exiles_to_twenty_mana_value() {
    let mut g = two_player_game();
    // 7 + 7 + 7 = 21 ≥ 20 after three wurms; the fourth stays.
    for _ in 0..4 { g.add_card_to_library(1, catalog::pelakka_wurm()); }
    let effect = catalog::tashas_hideous_laughter().effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let _ = g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[1].library.len(), 1, "stops once the MV pile reaches 20");
    assert_eq!(g.exile.len(), 3, "exiled, not milled");
}

#[test]
fn court_of_cunning_mills_more_while_monarch() {
    let mut g = two_player_game();
    for _ in 0..15 { g.add_card_to_library(1, catalog::forest()); }
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let court = g.add_card_to_hand(0, catalog::court_of_cunning());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: court, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0), "ETB crowns the controller");
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 10, "monarch mills ten");
}

#[test]
fn scute_swarm_copies_itself_with_six_lands() {
    let mut g = two_player_game();
    let swarm = g.add_card_to_battlefield(0, catalog::scute_swarm());
    for _ in 0..5 { g.add_card_to_battlefield(0, catalog::forest()); }
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).unwrap(); // sixth land
    drain_stack(&mut g);
    let copies = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Scute Swarm")
        .count();
    assert_eq!(copies, 1, "six lands → token copy of Scute Swarm");
    let _ = swarm;
}

#[test]
fn rampaging_baloths_landfall_mints_a_beast() {
    let mut g = two_player_game();
    let _b = g.add_card_to_battlefield(0, catalog::rampaging_baloths());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.power() == 4), "4/4 Beast minted");
}

#[test]
fn burgeoning_drops_a_land_when_the_opponent_plays_one() {
    let mut g = two_player_game();
    let _b = g.add_card_to_battlefield(0, catalog::burgeoning());
    let mine = g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    let theirs = g.add_card_to_hand(1, catalog::forest());
    g.perform_action(GameAction::PlayLand(theirs)).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_some(), "free land drop off Burgeoning");
}

#[test]
fn admonition_angel_landfall_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::admonition_angel());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none() && g.exile.iter().any(|c| c.id == bear),
        "bear exiled by the landfall trigger");
    let mut events = Vec::new();
    g.sacrifice_one(angel, 0, &mut events);
    assert!(g.battlefield_find(bear).is_some(), "bear returns when the Angel leaves");
}

#[test]
fn pack_rat_scales_with_rats_and_copies_itself() {
    let mut g = two_player_game();
    let rat = g.add_card_to_battlefield(0, catalog::pack_rat());
    let cp = g.compute_battlefield();
    let v = cp.iter().find(|c| c.id == rat).unwrap();
    assert_eq!((v.power, v.toughness), (1, 1), "lone rat is 1/1");
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_hand(0, catalog::forest()); // discard fodder
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rat, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("copy activation");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let v = cp.iter().find(|c| c.id == rat).unwrap();
    assert_eq!((v.power, v.toughness), (2, 2), "two rats → 2/2 each");
}

// ── Multikicker (CR 702.33c) ─────────────────────────────────────────────────

/// Everflowing Chalice kicked twice enters with two charge counters and taps
/// for {C}{C}.
#[test]
fn everflowing_chalice_multikicker_charges_and_taps_for_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::everflowing_chalice());
    g.players[0].mana_pool.add_colorless(4); // {0} base + 2 × {2}
    g.perform_action(GameAction::CastSpellMultikicked {
        card_id: id, times: 2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable kicked twice");
    drain_stack(&mut g);
    let chalice = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(chalice.counter_count(CounterType::Charge), 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.total(), 2, "adds {{C}} per charge counter");
}

/// Unkicked, the Chalice enters with no counters; kicking requires the mana.
#[test]
fn everflowing_chalice_unkicked_enters_empty() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::everflowing_chalice());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("free cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().find(|c| c.id == id).unwrap()
        .counter_count(CounterType::Charge), 0);
}

// ── Archive Trap (search-this-turn alt cost) ─────────────────────────────────

/// After the opponent searches their library, Archive Trap casts for {0} and
/// mills them thirteen.
#[test]
fn archive_trap_free_after_opponent_searches() {
    let mut g = two_player_game();
    // Opponent fetches: resolve an Effect::Search for them via a scripted
    // search (their evolving_wilds works as a real searcher).
    let fetch = g.add_card_to_battlefield(1, catalog::evolving_wilds());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: fetch, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("fetch activation");
    drain_stack(&mut g);
    assert!(g.players[1].searched_library_this_turn, "search stamped");

    g.priority.player_with_priority = 0;
    let trap = g.add_card_to_hand(0, catalog::archive_trap());
    for _ in 0..15 {
        g.add_card_to_library(1, catalog::forest());
    }
    let lib_before = g.players[1].library.len();
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: trap, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("free via trap condition");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib_before - 13, "milled 13");
}

/// Without a search this turn the {0} alternative cost is rejected.
#[test]
fn archive_trap_alt_cost_rejected_without_search() {
    let mut g = two_player_game();
    let trap = g.add_card_to_hand(0, catalog::archive_trap());
    assert!(g.perform_action(GameAction::CastSpellAlternative {
        card_id: trap, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no search this turn → no free cast");
}

// ── Torbran (additive source-scoped damage replacement) ──────────────────────

/// With Torbran out, a red spell you control deals +2 to the opponent; a
/// non-red source is unchanged.
#[test]
fn torbran_adds_two_to_red_source_damage_to_opponents() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::torbran_thane_of_red_fell());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 15, "3 + 2 = 5 damage");
}

/// Torbran's bonus also applies to red combat damage, and not to the
/// controller's own side.
#[test]
fn torbran_combat_damage_bonus_red_attacker_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::torbran_thane_of_red_fell());
    // Goblin Guide: red 2/2 haste.
    let goblin = g.add_card_to_battlefield(0, catalog::goblin_guide());
    g.clear_sickness(goblin);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: goblin, target: AttackTarget::Player(1) },
    ])).expect("attack");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat resolves");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16, "2 + 2 = 4 combat damage");
}

// ── Scrap Trawler (lesser-MV-than-dying-artifact return) ─────────────────────

/// An artifact creature dying (lethal damage → SBA) returns a
/// strictly-cheaper artifact card from the graveyard to hand; equal-MV
/// cards are not legal targets.
#[test]
fn scrap_trawler_returns_lesser_mv_artifact_on_artifact_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::scrap_trawler());
    // MV-2 artifact creature to die; MV-0 + MV-2 artifacts in graveyard.
    let dying = g.add_card_to_battlefield(0, catalog::hedron_crawler());
    let cheap = g.add_card_to_graveyard(0, catalog::ornithopter()); // MV 0
    let equal = g.add_card_to_graveyard(0, catalog::hedron_crawler()); // MV 2
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(dying)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the crawler");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == cheap), "MV 0 < 2 returned");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == equal), "equal MV stays");
}

/// A nonartifact creature dying doesn't fire the Trawler.
#[test]
fn scrap_trawler_ignores_nonartifact_deaths() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::scrap_trawler());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cheap = g.add_card_to_graveyard(0, catalog::ornithopter());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the bear");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cheap), "no trigger");
}

// ── Conflagrate (flashback with discard-X) ───────────────────────────────────

/// Hard-cast: X damage divided; single target takes all of it.
#[test]
fn conflagrate_hard_cast_deals_x() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::conflagrate());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: Some(3),
    }).expect("X=3 cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Flashback charges {R}{R} plus discarding X cards; X drives the damage.
#[test]
fn conflagrate_flashback_discards_x_cards() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::conflagrate());
    for _ in 0..2 { g.add_card_to_hand(0, catalog::forest()); }
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastFlashback {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: Some(2),
    }).expect("flashback X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 2, "discarded X=2");
    assert_eq!(g.players[1].life, 18, "2 damage");
    assert!(g.exile.iter().any(|c| c.id == id), "flashback exiles");
}

/// Flashback with X greater than hand size is rejected.
#[test]
fn conflagrate_flashback_rejected_without_enough_cards() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::conflagrate());
    g.players[0].hand.clear();
    g.players[0].mana_pool.add(Color::Red, 2);
    assert!(g.perform_action(GameAction::CastFlashback {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: Some(2),
    }).is_err());
}

// ── Chandra, Torch of Defiance (impulse with fallback) ───────────────────────

/// +1 exiles the top card with a may-cast grant; if it's still in exile at
/// the end step (uncast), each opponent takes 2.
#[test]
fn chandra_torch_plus_one_burns_when_impulse_uncast() {
    let mut g = two_player_game();
    let chandra = g.add_card_to_battlefield(0, catalog::chandra_torch_of_defiance());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: chandra, ability_index: 0, target: None, x_value: None,
    }).expect("+1");
    drain_stack(&mut g);
    let exiled = g.exile.last().expect("top card exiled");
    assert!(exiled.may_play_until.is_some(), "may-cast grant stamped");
    let life = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "uncast impulse → 2 damage");
}

/// Casting the impulsed card consumes it; no end-step damage.
#[test]
fn chandra_torch_plus_one_no_burn_when_cast() {
    let mut g = two_player_game();
    let chandra = g.add_card_to_battlefield(0, catalog::chandra_torch_of_defiance());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: chandra, ability_index: 0, target: None, x_value: None,
    }).expect("+1");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast the impulsed card");
    drain_stack(&mut g);
    let life = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life, "cast → no fallback damage");
}

// ── Dauthi Voidwalker (void-counter exile + free play) ───────────────────────

/// With the Voidwalker out, an opponent's dying creature is exiled with a
/// void counter instead of hitting their graveyard.
#[test]
fn dauthi_voidwalker_exiles_opponent_cards_with_void_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dauthi_voidwalker());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().all(|c| c.id != bear), "not in graveyard");
    let exiled = g.exile.iter().find(|c| c.id == bear).expect("exiled instead");
    assert_eq!(exiled.counter_count(CounterType::Void), 1, "void counter stamped");
    // The caster's own Bolt still hits their graveyard (opponents_only).
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt));
}

/// The sac ability grants a free play of a void-countered exile card.
#[test]
fn dauthi_voidwalker_sac_frees_void_card() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let walker = g.add_card_to_battlefield(0, catalog::dauthi_voidwalker());
    g.clear_sickness(walker);
    // Opponent-owned card already in exile with a void counter.
    let stolen = g.add_card_to_exile(1, catalog::serra_angel());
    g.find_card_anywhere_mut(stolen).unwrap().add_counters(CounterType::Void, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: walker, ability_index: 0, target: Some(Target::Permanent(stolen)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac activation");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != walker), "walker sacrificed");
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: stolen, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("free cast of the void card");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == stolen), "Angel enters under the activator");
}

// ── Urza's Saga (saga-granted activated abilities) ───────────────────────────

/// Chapter I grants "{T}: Add {C}"; chapter II grants the Construct mint;
/// chapter III tutors a cheap artifact and the Saga sacrifices itself.
#[test]
fn urzas_saga_chapters_grant_abilities_then_tutor() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let saga = g.add_card_to_hand(0, catalog::urzas_saga());
    let orni = g.add_card_to_library(0, catalog::ornithopter()); // chapter III target
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(orni))]));
    g.perform_action(GameAction::PlayLand(saga)).expect("land drop");
    drain_stack(&mut g);
    // Chapter I — mana ability granted.
    g.perform_action(GameAction::ActivateAbility {
        card_id: saga, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("granted {T}: Add {C}");
    assert_eq!(g.players[0].mana_pool.total(), 1);
    g.battlefield_find_mut(saga).unwrap().tapped = false;
    // Chapter II — construct mint granted.
    g.saga_advance(saga);
    drain_stack(&mut g);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: saga, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("granted construct mint");
    drain_stack(&mut g);
    let construct = g.battlefield.iter().find(|c| c.definition.name == "Construct")
        .expect("token minted");
    let cid = construct.id;
    // Construct scales with artifacts you control (itself + the Saga is not
    // an artifact — just the token = 1/1).
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == cid).map(|c| (c.power, c.toughness)), Some((1, 1)));
    // Chapter III — tutor onto the battlefield; Saga sacrifices via SBA.
    g.saga_advance(saga);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Ornithopter"),
        "chapter III fetched the 0-cost artifact");
    assert!(g.battlefield.iter().all(|c| c.id != saga), "Saga sacrificed");
}

/// CR 700.5 — Heliod's god gate: not a creature below five white devotion,
/// a creature at five or more.
#[test]
fn heliod_devotion_gate_toggles_creatureness() {
    let mut g = two_player_game();
    let heliod = g.add_card_to_battlefield(0, catalog::heliod_sun_crowned()); // {2}{W} = 1 devotion
    let cp = g.compute_battlefield();
    assert!(!cp.iter().find(|c| c.id == heliod).unwrap().card_types.contains(&CardType::Creature),
        "1 white pip < 5 → not a creature");
    // Add four more white pips of devotion (two Serra Angels = {3}{W}{W} each).
    g.add_card_to_battlefield(0, catalog::serra_angel());
    g.add_card_to_battlefield(0, catalog::serra_angel());
    let cp = g.compute_battlefield();
    assert!(cp.iter().find(|c| c.id == heliod).unwrap().card_types.contains(&CardType::Creature),
        "5 white pips → creature");
}

// ── CR 700.9 modified (Kodama of the West Tree) ──────────────────────────────

/// A countered creature is modified → gains trample from Kodama; a bare one
/// doesn't.
#[test]
fn kodama_grants_trample_to_modified_creatures_only() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kodama_of_the_west_tree());
    let modded = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bare = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(modded).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let cp = g.compute_battlefield();
    assert!(cp.iter().find(|c| c.id == modded).unwrap().keywords.contains(&Keyword::Trample));
    assert!(!cp.iter().find(|c| c.id == bare).unwrap().keywords.contains(&Keyword::Trample));
}

/// A modified creature connecting fetches a tapped basic.
#[test]
fn kodama_fetches_basic_on_modified_combat_damage() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kodama_of_the_west_tree());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.clear_sickness(bear);
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("attack");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat resolves");
    drain_stack(&mut g);
    let fetched = g.battlefield.iter().find(|c| c.id == forest).expect("Forest fetched");
    assert!(fetched.tapped, "enters tapped");
}

/// CR 702.26 — layer-granted Phasing (a static "X has phasing") phases the
/// permanent out at its controller's untap, not just the printed keyword.
#[test]
fn granted_phasing_phases_out_at_untap() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Grant Phasing via a continuous effect (the layer system path).
    g.add_continuous_effect(crabomination::game::layers::ContinuousEffect {
        timestamp: 9999,
        source: bear,
        affected: crabomination::game::layers::AffectedPermanents::Specific(vec![bear]),
        layer: crabomination::game::layers::Layer::L6Ability,
        sublayer: None,
        duration: crabomination::game::layers::EffectDuration::WhileSourceOnBattlefield,
        modification: crabomination::game::layers::Modification::AddKeyword(Keyword::Phasing),
    });
    g.active_player_idx = 0;
    g.do_phasing();
    assert!(g.battlefield.iter().all(|c| c.id != bear), "phased out");
    assert!(g.phased_out.iter().any(|c| c.id == bear));
}

// ── CR 608.2b multi-target fizzle ────────────────────────────────────────────

/// A multi-target spell with every target gone fizzles into the graveyard.
#[test]
fn cr_608_2b_multi_target_spell_fizzles_when_all_targets_illegal() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let trail = g.add_card_to_hand(0, catalog::arc_trail());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: trail, target: Some(Target::Permanent(b1)),
        additional_targets: vec![Target::Permanent(b2)], mode: None, x_value: None,
    }).expect("cast");
    // Both targets leave before resolution.
    g.remove_to_graveyard_with_triggers(b1);
    g.remove_to_graveyard_with_triggers(b2);
    let life = g.players[1].life;
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life, "no stray damage");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == trail), "fizzled to graveyard");
}

/// With only one of two targets gone, the spell still resolves against the
/// surviving one.
#[test]
fn cr_608_2b_multi_target_spell_resolves_with_one_legal_target() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::serra_angel());
    let trail = g.add_card_to_hand(0, catalog::arc_trail());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: trail, target: Some(Target::Permanent(b1)),
        additional_targets: vec![Target::Permanent(b2)], mode: None, x_value: None,
    }).expect("cast");
    g.remove_to_graveyard_with_triggers(b1); // slot-0 target gone
    drain_stack(&mut g);
    let angel = g.battlefield.iter().find(|c| c.id == b2).expect("Angel survives 1 damage");
    assert_eq!(angel.damage, 1, "slot-1 damage still resolved");
}

// ── Wolfbriar Elemental + Collective Restraint ───────────────────────────────

/// Multikicked three times → three Wolves alongside the 4/4.
#[test]
fn wolfbriar_elemental_mints_wolves_per_kick() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::wolfbriar_elemental());
    g.players[0].mana_pool.add(Color::Green, 7); // {2}{G}{G} + 3×{G}
    g.perform_action(GameAction::CastSpellMultikicked {
        card_id: id, times: 3, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("kicked thrice");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Wolf").count(), 3);
}

/// Collective Restraint taxes attackers by the defender's domain count.
#[test]
fn collective_restraint_taxes_by_domain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::collective_restraint());
    g.add_card_to_battlefield(1, catalog::forest());
    g.add_card_to_battlefield(1, catalog::island());
    let goblin = g.add_card_to_battlefield(0, catalog::goblin_guide());
    g.clear_sickness(goblin);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    // Two basic types → tax {2}; no mana → attack rejected.
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: goblin, target: AttackTarget::Player(1) },
    ])).is_err(), "unpaid domain tax blocks the attack");
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: goblin, target: AttackTarget::Player(1) },
    ])).expect("paid tax → attack legal");
}

// ── Kicker / multikicker batch ───────────────────────────────────────────────

/// Gnarlid Pack kicked twice enters as a 4/4.
#[test]
fn gnarlid_pack_enters_with_kick_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::gnarlid_pack());
    g.players[0].mana_pool.add(Color::Green, 6);
    g.perform_action(GameAction::CastSpellMultikicked {
        card_id: id, times: 2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("kicked twice");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((4, 4)));
}

/// Rite of Replication unkicked makes one copy; kicked makes five.
#[test]
fn rite_of_replication_kicked_makes_five_copies() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let rite = g.add_card_to_hand(0, catalog::rite_of_replication());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: rite, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("kicked");
    drain_stack(&mut g);
    let copies = g.battlefield.iter()
        .filter(|c| c.definition.name == "Serra Angel" && c.controller == 0).count();
    assert_eq!(copies, 5, "five token copies under the caster");
}

/// Marsh Casualties unkicked gives the targeted player's creatures -1/-1.
#[test]
fn marsh_casualties_shrinks_target_players_creatures() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::marsh_casualties());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == theirs).map(|c| (c.power, c.toughness)), Some((1, 1)));
    assert_eq!(cp.iter().find(|c| c.id == mine).map(|c| (c.power, c.toughness)), Some((2, 2)),
        "caster's own creatures untouched");
}

// ── Phyrexian Scriptures ─────────────────────────────────────────────────────

/// Chapter I counters + artifact-izes a creature so chapter II spares it.
#[test]
fn phyrexian_scriptures_chapters() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let saga = g.add_card_to_hand(0, catalog::phyrexian_scriptures());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    // Accept chapter I's "may" (AutoDecider declines optional triggers).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, saga);
    drain_stack(&mut g);
    // Chapter I — counter + artifact type on my bear.
    let cp = g.compute_battlefield();
    let bear = cp.iter().find(|c| c.id == mine).unwrap();
    assert_eq!((bear.power, bear.toughness), (3, 3), "+1/+1 counter applied");
    assert!(bear.card_types.contains(&CardType::Artifact), "became an artifact");
    // Chapter II — destroy all nonartifact creatures (Angel dies, bear lives).
    g.saga_advance(saga);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == mine), "artifact creature survives");
    assert!(g.battlefield.iter().all(|c| c.id != theirs), "nonartifact creature destroyed");
    // Chapter III — opponents' graveyards exiled; mine untouched.
    let opp_gy = g.players[1].graveyard.len();
    assert!(opp_gy > 0);
    g.saga_advance(saga);
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 0, "opponent graveyard exiled");
}

// ── Multikicker ETB payoffs ──────────────────────────────────────────────────

/// Lightkeeper kicked twice gains 4 life.
#[test]
fn lightkeeper_of_emeria_gains_two_per_kick() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lightkeeper_of_emeria());
    g.players[0].mana_pool.add(Color::White, 6);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpellMultikicked {
        card_id: id, times: 2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("kicked twice");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4);
}

/// Bloodhusk Ritualist kicked twice makes the opponent discard two.
#[test]
fn bloodhusk_ritualist_discards_per_kick() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bloodhusk_ritualist());
    for _ in 0..3 { g.add_card_to_hand(1, catalog::forest()); }
    let hand = g.players[1].hand.len();
    g.players[0].mana_pool.add(Color::Black, 5);
    g.perform_action(GameAction::CastSpellMultikicked {
        card_id: id, times: 2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("kicked twice");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand - 2);
}

/// Joraga Warcaller's counter-scaled anthem pumps other Elves only.
#[test]
fn joraga_warcaller_anthem_scales_with_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let joraga = g.add_card_to_battlefield(0, catalog::joraga_warcaller());
    let elf = g.add_card_to_battlefield(0, catalog::elves_of_deep_shadow());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(joraga).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let cp = g.compute_battlefield();
    let elf_v = cp.iter().find(|c| c.id == elf).unwrap();
    assert_eq!((elf_v.power, elf_v.toughness), (3, 3), "1/1 elf +2/+2");
    let joraga_v = cp.iter().find(|c| c.id == joraga).unwrap();
    assert_eq!((joraga_v.power, joraga_v.toughness), (3, 3), "own counters only (no self-anthem)");
    let bear_v = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((bear_v.power, bear_v.toughness), (2, 2), "non-Elf untouched");
}

// ── Mill batch ───────────────────────────────────────────────────────────────

/// Traumatize halves the opponent's library (rounded down).
#[test]
fn traumatize_mills_half_library() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::traumatize());
    while g.players[1].library.len() < 11 { g.add_card_to_library(1, catalog::forest()); }
    let lib = g.players[1].library.len();
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib - lib / 2);
}

/// Bruvac doubles an opponent's mill.
#[test]
fn bruvac_doubles_opponent_mill() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bruvac_the_grandiloquent());
    for _ in 0..20 { g.add_card_to_library(1, catalog::forest()); }
    let lib = g.players[1].library.len();
    let ms = g.add_card_to_hand(0, catalog::mind_sculpt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, ms);
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib - 14, "7 doubled to 14");
}

/// Chasm Skulker grows on draws and leaves Squids behind.
#[test]
fn chasm_skulker_grows_and_spawns_squids() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let skulker = g.add_card_to_battlefield(0, catalog::chasm_skulker());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let opt = g.add_card_to_hand(0, catalog::opt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, opt);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(skulker).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    g.remove_to_graveyard_with_triggers(skulker);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Squid").count(), 1);
}

/// Wight of Precinct Six scales with opponents' graveyard creatures only.
#[test]
fn wight_of_precinct_six_scales_with_opp_graveyard_creatures() {
    let mut g = two_player_game();
    let wight = g.add_card_to_battlefield(0, catalog::wight_of_precinct_six());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::lightning_bolt()); // noncreature — ignored
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // own gy — ignored
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == wight).map(|c| (c.power, c.toughness)), Some((2, 2)));
}

/// Maddening Cacophony kicked mills half the library rounded up.
#[test]
fn maddening_cacophony_kicked_mills_half_up() {
    let mut g = two_player_game();
    while g.players[1].library.len() < 9 { g.add_card_to_library(1, catalog::forest()); }
    let lib = g.players[1].library.len();
    let id = g.add_card_to_hand(0, catalog::maddening_cacophony());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("kicked");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib - lib.div_ceil(2));
}

/// Consuming Aberration's CDA tracks opponents' graveyards; its cast
/// trigger mills them further.
#[test]
fn consuming_aberration_scales_and_mills_on_cast() {
    let mut g = two_player_game();
    let ab = g.add_card_to_battlefield(0, catalog::consuming_aberration());
    for _ in 0..5 { g.add_card_to_library(1, catalog::forest()); }
    g.add_card_to_graveyard(1, catalog::forest());
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == ab).map(|c| c.power), Some(1));
    let opt = g.add_card_to_hand(0, catalog::opt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.add_card_to_library(0, catalog::forest());
    cast(&mut g, opt);
    drain_stack(&mut g);
    // The cast trigger reveals down to the first land: the opponent's
    // library is all forests, so exactly one card is milled (1 + 1 + 1).
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == ab).map(|c| c.power), Some(2), "1 in gy + 1 milled");
}

/// Luminarch Ascension quests up on the opponent's end step (no life lost)
/// and mints Angels once it has four counters.
#[test]
fn luminarch_ascension_quests_and_mints_angels() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let asc = g.add_card_to_battlefield(0, catalog::luminarch_ascension());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 1; // opponent's turn
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(asc).unwrap().counter_count(CounterType::Quest), 1);
    // Gated activation: rejected below four counters.
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: asc, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "needs four quest counters");
    g.battlefield_find_mut(asc).unwrap().add_counters(CounterType::Quest, 3);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: asc, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("four counters → activate");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Angel"));
}

/// No quest counter on an end step where you lost life.
#[test]
fn luminarch_ascension_blocked_by_life_loss() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let asc = g.add_card_to_battlefield(0, catalog::luminarch_ascension());
    g.players[0].lost_life_this_turn = true;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(asc).unwrap().counter_count(CounterType::Quest), 0);
}

// ── ZEN vampires + mill enchantments ─────────────────────────────────────────

/// Kicked Gatekeeper edicts the opponent; unkicked it's just a body.
#[test]
fn gatekeeper_of_malakir_kicked_edicts() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gk = g.add_card_to_hand(0, catalog::gatekeeper_of_malakir());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: gk, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("kicked");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != bear), "opponent sacrificed");
}

/// Bloodwitch drains per Vampire you control.
#[test]
fn malakir_bloodwitch_drains_per_vampire() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vampire_nighthawk());
    let bw = g.add_card_to_hand(0, catalog::malakir_bloodwitch());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.players[0].mana_pool.add(Color::Black, 5);
    cast(&mut g, bw);
    drain_stack(&mut g);
    // Nighthawk + the Bloodwitch itself = 2 Vampires.
    assert_eq!(g.players[1].life, l1 - 2);
    assert_eq!(g.players[0].life, l0 + 2);
}

/// Psychic Corrosion mills opponents on each of your draws.
#[test]
fn psychic_corrosion_mills_on_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::psychic_corrosion());
    for _ in 0..4 { g.add_card_to_library(1, catalog::forest()); }
    g.add_card_to_library(0, catalog::forest());
    let lib = g.players[1].library.len();
    let opt = g.add_card_to_hand(0, catalog::opt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, opt);
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib - 2);
}

/// Drowned Secrets only fires on blue spells.
#[test]
fn drowned_secrets_mills_on_blue_casts_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::drowned_secrets());
    for _ in 0..6 { g.add_card_to_library(1, catalog::forest()); }
    let lib = g.players[1].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib, "red spell → no mill");
    let opt = g.add_card_to_hand(0, catalog::opt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.add_card_to_library(0, catalog::forest());
    cast(&mut g, opt);
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib - 2, "blue spell → mill 2");
}

// ── Tribal one-drops ─────────────────────────────────────────────────────────

/// Indulgent Aristocrat's sac ability counters every Vampire you control.
#[test]
fn indulgent_aristocrat_sac_counters_vampires() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let aristo = g.add_card_to_battlefield(0, catalog::indulgent_aristocrat());
    let hawk = g.add_card_to_battlefield(0, catalog::vampire_nighthawk());
    let fodder = g.add_card_to_battlefield(0, catalog::ornithopter()); // lowest power → auto-pick
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: aristo, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac activation");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != fodder), "fodder sacrificed");
    assert_eq!(g.battlefield_find(aristo).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(hawk).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Stromkirk Noble can't be blocked by a Human and grows on connection.
#[test]
fn stromkirk_noble_unblockable_by_humans_and_grows() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let noble = g.add_card_to_battlefield(0, catalog::stromkirk_noble());
    g.clear_sickness(noble);
    let human = g.add_card_to_battlefield(1, catalog::champion_of_the_parish());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: noble, target: AttackTarget::Player(1) },
    ])).expect("attack");
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(human, noble)])).is_err(),
        "Human can't block");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(noble).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Drana counters every attacking creature you control when she connects.
#[test]
fn drana_counters_attackers_on_combat_damage() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let drana = g.add_card_to_battlefield(0, catalog::drana_liberator_of_malakir());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(drana);
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: drana, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("attack");
    g.step = TurnStep::CombatDamage;
    g.resolve_first_strike_damage().expect("first-strike damage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne) >= 1,
        "Drana's first-strike connection counters the attacking bear");
    let _ = drana;
}

/// Cryptbreaker mints Zombies for a discard and taps three Zombies to draw.
#[test]
fn cryptbreaker_mints_and_draws() {
    let mut g = two_player_game();
    let cb = g.add_card_to_battlefield(0, catalog::cryptbreaker());
    g.clear_sickness(cb);
    g.add_card_to_hand(0, catalog::forest()); // discard fodder
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cb, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("mint");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Zombie").count(), 1);
    // Two more Zombies + Cryptbreaker untapped = three to tap for the draw.
    g.battlefield_find_mut(cb).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cb, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).err(); // out of mana — ignore
    let z2 = g.add_card_to_battlefield(0, catalog::diregraf_ghoul());
    g.battlefield_find_mut(z2).unwrap().tapped = false;
    let hand = g.players[0].hand.len();
    let life = g.players[0].life;
    g.add_card_to_library(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: cb, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap-three draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].life, life - 1);
}

/// Mistcutter Hydra enters with X counters and can't be countered.
#[test]
fn mistcutter_hydra_x_counters_and_uncounterable() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mistcutter_hydra());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("X=3");
    // Counterspell bounces off.
    let cs = g.add_card_to_hand(1, catalog::counterspell());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: cs, target: Some(Target::Permanent(id)), additional_targets: vec![],
        mode: None, x_value: None,
    }).ok();
    drain_stack(&mut g);
    let hydra = g.battlefield.iter().find(|c| c.id == id).expect("resolved despite Counterspell");
    assert_eq!(hydra.counter_count(CounterType::PlusOnePlusOne), 3);
    assert!(hydra.has_keyword(&Keyword::Haste));
}

/// Underworld Connections grants the enchanted land a pay-1-draw tap ability.
#[test]
fn underworld_connections_grants_land_draw() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::swamp());
    let aura = g.add_card_to_hand(0, catalog::underworld_connections());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(land)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("enchant land");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(aura).unwrap().attached_to, Some(land));
    g.add_card_to_library(0, catalog::forest());
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    // The granted ability surfaces after the land's printed mana ability.
    let printed = g.battlefield_find(land).unwrap().definition.activated_abilities.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: printed, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("granted draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].life, life - 1);
}

// ── modern_decks staples expansion ──────────────────────────────────────────

/// Wilderness Reclamation untaps all your lands at your end step.
#[test]
fn wilderness_reclamation_untaps_lands_at_end_step() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::wilderness_reclamation());
    let land_a = g.add_card_to_battlefield(0, catalog::forest());
    let land_b = g.add_card_to_battlefield(0, catalog::island());
    g.battlefield_find_mut(land_a).unwrap().tapped = true;
    g.battlefield_find_mut(land_b).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(land_a).unwrap().tapped, "Forest untapped");
    assert!(!g.battlefield_find(land_b).unwrap().tapped, "Island untapped");
}

/// Silvergill Adept costs {1}{U} with a Merfolk to reveal, {4}{U} without.
#[test]
fn silvergill_adept_reveal_or_pay_three() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(1, catalog::forest());
    }
    // With another Merfolk in hand: castable for {1}{U}.
    let _merfolk = g.add_card_to_hand(0, catalog::tideshaper_mystic());
    let adept = g.add_card_to_hand(0, catalog::silvergill_adept());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: adept, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {1}{U} revealing a Merfolk");
    drain_stack(&mut g);
    assert!(g.battlefield_find(adept).is_some());

    // Without a Merfolk in hand: {1}{U} is not enough.
    let adept2 = g.add_card_to_hand(1, catalog::silvergill_adept());
    g.players[1].hand.retain(|c| c.id == adept2);
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: adept2, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "no Merfolk to reveal → needs {{3}} more"
    );
    g.players[1].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: adept2, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {4}{U} without a reveal");
}

/// Vodalian Hexcatcher pumps other Merfolk and sacs one to tax-counter.
#[test]
fn vodalian_hexcatcher_lord_and_sac_counter() {
    let mut g = two_player_game();
    let hex = g.add_card_to_battlefield(0, catalog::vodalian_hexcatcher());
    let folk = g.add_card_to_battlefield(0, catalog::tideshaper_mystic());
    let cp = g.computed_permanent(folk).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "lord pumps other Merfolk");
    // Opponent casts a sorcery; sac the Merfolk to counter unless {1}.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("opp bolt");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: hex, ability_index: 0,
        target: Some(Target::Permanent(bolt)), additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("sac a Merfolk to counter");
    assert!(!g.battlefield.iter().any(|c| c.id == folk), "Merfolk sacrificed");
    drain_stack(&mut g);
    // Opponent had no mana left → bolt countered.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "bolt countered");
    assert_eq!(g.players[0].life, 20, "no damage");
}

/// Svyelun is indestructible only with two other Merfolk and grants ward.
#[test]
fn svyelun_indestructible_gate_and_ward() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let svy = g.add_card_to_battlefield(0, catalog::svyelun_of_sea_and_sky());
    let m1 = g.add_card_to_battlefield(0, catalog::tideshaper_mystic());
    assert!(
        !g.computed_permanent(svy).unwrap().keywords.contains(&Keyword::Indestructible),
        "one other Merfolk is not enough"
    );
    let _m2 = g.add_card_to_battlefield(0, catalog::silvergill_adept());
    assert!(
        g.computed_permanent(svy).unwrap().keywords.contains(&Keyword::Indestructible),
        "two other Merfolk → indestructible"
    );
    assert!(
        g.computed_permanent(m1).unwrap().keywords.iter().any(|k| matches!(k, Keyword::Ward(_))),
        "other Merfolk have ward {{1}}"
    );
}

/// Tideshaper Mystic turns a land into an Island during your turn only.
#[test]
fn tideshaper_mystic_changes_land_type_your_turn_only() {
    use crabomination::card::LandType;
    let mut g = two_player_game();
    let tide = g.add_card_to_battlefield(0, catalog::tideshaper_mystic());
    g.clear_sickness(tide);
    let land = g.add_card_to_battlefield(1, catalog::forest());
    // Not your turn → rejected.
    g.active_player_idx = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: tide, ability_index: 0,
            target: Some(Target::Permanent(land)), additional_targets: Vec::new(), x_value: None, mode: None,
        })
        .is_err(),
        "activate only during your turn"
    );
    g.active_player_idx = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)])); // Island
    g.perform_action(GameAction::ActivateAbility {
        card_id: tide, ability_index: 0,
        target: Some(Target::Permanent(land)), additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("activates on your turn");
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).unwrap();
    assert!(cp.subtypes.land_types.contains(&LandType::Island), "now an Island");
}

/// Devoted Druid untaps itself for a -1/-1 counter; Vizier of Remedies
/// shaves the counter for the infinite-mana combo.
#[test]
fn devoted_druid_vizier_combo_untaps_without_counters() {
    let mut g = two_player_game();
    let druid = g.add_card_to_battlefield(0, catalog::devoted_druid());
    g.clear_sickness(druid);
    // Without Vizier: untap costs a real -1/-1 counter.
    g.perform_action(GameAction::ActivateAbility {
        card_id: druid, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("tap for G");
    g.perform_action(GameAction::ActivateAbility {
        card_id: druid, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("untap with a counter");
    drain_stack(&mut g);
    let d = g.battlefield_find(druid).unwrap();
    assert!(!d.tapped, "untapped");
    assert_eq!(d.counter_count(CounterType::MinusOneMinusOne), 1, "counter placed");

    // With Vizier: the counter never lands.
    g.add_card_to_battlefield(0, catalog::vizier_of_remedies());
    g.perform_action(GameAction::ActivateAbility {
        card_id: druid, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("tap again");
    g.perform_action(GameAction::ActivateAbility {
        card_id: druid, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("untap");
    drain_stack(&mut g);
    let d = g.battlefield_find(druid).unwrap();
    assert!(!d.tapped, "untapped again");
    assert_eq!(
        d.counter_count(CounterType::MinusOneMinusOne),
        1,
        "Vizier shaved the second counter (still just one)"
    );
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "two G floated");
}

/// Voice of Victory locks opponents out of casting during your turn.
#[test]
fn voice_of_victory_locks_opponent_casts_on_your_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::voice_of_victory());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "opponent can't cast during your turn"
    );
    // On their own turn it resolves fine.
    g.active_player_idx = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable on the opponent's own turn");
}

/// Ocelot Pride makes a Cat at end step after lifegain; with the city's
/// blessing it copies each token that entered this turn.
#[test]
fn ocelot_pride_end_step_cat_and_blessing_copies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ocelot_pride());
    g.players[0].life_gained_this_turn = 2;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let cats = g.battlefield.iter().filter(|c| c.definition.name == "Cat").count();
    assert_eq!(cats, 1, "one Cat token without the blessing");

    // With the blessing: the end-step trigger doubles this turn's tokens.
    g.players[0].city_blessing = true;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let cats_after = g.battlefield.iter().filter(|c| c.definition.name == "Cat").count();
    assert!(cats_after >= 3, "blessing copies this turn's tokens (got {cats_after})");
}

/// Aether Vial charges at upkeep and vials in a creature with MV == charges.
#[test]
fn aether_vial_charges_and_deploys_matching_mv() {
    let mut g = two_player_game();
    let vial = g.add_card_to_battlefield(0, catalog::aether_vial());
    // Two upkeep ticks (MayDo auto-accept via scripted decider).
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(vial).unwrap().counter_count(CounterType::Charge),
        2,
        "two charge counters"
    );
    // Hand: an MV-2 bear (deployable) and an MV-1 elf (not).
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let _elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: vial, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("tap the vial");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "MV-2 creature deployed");
}

/// Mox Amber taps for a color among your legendaries — and for nothing
/// without one.
#[test]
fn mox_amber_needs_a_legendary() {
    let mut g = two_player_game();
    let mox = g.add_card_to_battlefield(0, catalog::mox_amber());
    g.perform_action(GameAction::ActivateAbility {
        card_id: mox, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0, "no legendary → no mana");
    g.battlefield_find_mut(mox).unwrap().tapped = false;
    // A blue legendary creature on the battlefield → produces {U}.
    g.add_card_to_battlefield(0, catalog::svyelun_of_sea_and_sky());
    g.perform_action(GameAction::ActivateAbility {
        card_id: mox, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("activates with a legendary");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "adds {{U}}");
}

/// Ulamog's attack trigger exiles the defender's top twenty.
#[test]
fn ulamog_attack_exiles_top_twenty() {
    let mut g = two_player_game();
    let ula = g.add_card_to_battlefield(0, catalog::ulamog_the_ceaseless_hunger());
    g.clear_sickness(ula);
    for _ in 0..25 {
        g.add_card_to_library(1, catalog::forest());
    }
    let lib_before = g.players[1].library.len();
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ula, target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(
        g.players[1].library.len(),
        lib_before - 20,
        "defender exiled twenty"
    );
}

/// Crypt Incursion exiles a graveyard's creatures and gains 3 per card.
#[test]
fn crypt_incursion_exiles_and_gains() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::llanowar_elves());
    g.add_card_to_graveyard(1, catalog::lightning_bolt()); // not a creature
    let id = g.add_card_to_hand(0, catalog::crypt_incursion());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 6, "3 life per creature exiled");
    assert_eq!(
        g.players[1].graveyard.iter().filter(|c| c.definition.is_creature()).count(),
        0,
        "creatures exiled"
    );
}

/// Stern Scolding counters a small creature spell but not a big one.
#[test]
fn stern_scolding_counters_small_creatures_only() {
    let mut g = two_player_game();
    // Opponent casts Grizzly Bears (2/2 — counterable).
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("opp bear");
    let scold = g.add_card_to_hand(0, catalog::stern_scolding());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: scold, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("counter the 2/2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear countered");

    // A 10/10 Ulamog is not a legal target.
    let ula = g.add_card_to_hand(1, catalog::ulamog_the_ceaseless_hunger());
    g.players[1].mana_pool.add_colorless(10);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: ula, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("opp Ulamog");
    let scold2 = g.add_card_to_hand(0, catalog::stern_scolding());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: scold2, target: Some(Target::Permanent(ula)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "10/10 fails the P/T ≤ 2 filter"
    );
}

/// Peer Through Depths digs five for an instant/sorcery.
#[test]
fn peer_through_depths_takes_instant_to_hand() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt()); // top area
    let id = g.add_card_to_hand(0, catalog::peer_through_depths());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "Bolt to hand");
}

/// Malevolent Rumble digs four for a permanent and makes a Spawn that
/// sacrifices for {C}.
#[test]
fn malevolent_rumble_digs_and_spawns() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears()); // top
    let id = g.add_card_to_hand(0, catalog::malevolent_rumble());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "permanent to hand");
    let spawn = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Eldrazi Spawn")
        .map(|c| c.id)
        .expect("spawn created");
    g.perform_action(GameAction::ActivateAbility {
        card_id: spawn, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("sac for {C}");
    assert!(g.players[0].mana_pool.colorless_amount() >= 1, "spawn made {{C}}");
}

/// Eldrazi Repurposer spawns on cast and on death.
#[test]
fn eldrazi_repurposer_spawns_twice() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::eldrazi_repurposer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let spawns = |g: &GameState| {
        g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count()
    };
    assert_eq!(spawns(&g), 1, "cast trigger spawn");
    let events = g.remove_to_graveyard_with_triggers(id);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(spawns(&g), 2, "dies trigger spawn");
}

/// Writhing Chrysalis grows when you sacrifice another Eldrazi.
#[test]
fn writhing_chrysalis_grows_on_spawn_sacrifice() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::writhing_chrysalis());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let spawn = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Eldrazi Spawn")
        .map(|c| c.id)
        .expect("two spawns minted");
    g.perform_action(GameAction::ActivateAbility {
        card_id: spawn, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("sac a spawn");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "+1/+1 from the sacrifice"
    );
}

/// It That Heralds the End discounts big colorless spells and pumps other
/// colorless creatures.
#[test]
fn it_that_heralds_the_end_discount_and_anthem() {
    let mut g = two_player_game();
    let herald = g.add_card_to_battlefield(0, catalog::it_that_heralds_the_end());
    // Anthem: a Devoid creature gets +1/+1; the herald itself doesn't.
    let drone = g.add_card_to_battlefield(0, catalog::eldrazi_repurposer());
    let cp = g.computed_permanent(drone).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "other colorless creature pumped");
    let self_cp = g.computed_permanent(herald).unwrap();
    assert_eq!((self_cp.power, self_cp.toughness), (2, 2), "not itself");
    // Ulamog ({10}, colorless) castable with 9 mana.
    let ula = g.add_card_to_hand(0, catalog::ulamog_the_ceaseless_hunger());
    g.players[0].mana_pool.add_colorless(9);
    g.perform_action(GameAction::CastSpell {
        card_id: ula, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("{1} discount applies");
}

/// Primal Command runs two chosen modes (default: gain 7 + search).
#[test]
fn primal_command_gains_and_searches() {
    let mut g = two_player_game();
    let bear_in_lib = g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::primal_command());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Modes(vec![0, 3]),
        DecisionAnswer::Search(Some(bear_in_lib)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 7, "mode 1: gained 7");
    assert!(
        g.players[0].hand.iter().any(|c| c.id == bear_in_lib),
        "mode 4: searched a creature to hand"
    );
}

/// Priest of Fell Rites reanimates via tap+life+sac, and unearths itself.
#[test]
fn priest_of_fell_rites_reanimates_and_unearths() {
    let mut g = two_player_game();
    let priest = g.add_card_to_battlefield(0, catalog::priest_of_fell_rites());
    g.clear_sickness(priest);
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: priest, ability_index: 0,
        target: Some(Target::Permanent(dead)), additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("reanimate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "creature reanimated");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == priest), "priest sacrificed");
    assert_eq!(g.players[0].life, 17, "paid 3 life");

    // Unearth: return from graveyard with haste; exiled at next end step.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: priest, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("unearth");
    drain_stack(&mut g);
    assert!(g.battlefield_find(priest).is_some(), "priest back");
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(
        g.exile.iter().any(|c| c.id == priest),
        "unearthed creature exiled at end step"
    );
}

/// Platinum Emperion freezes its controller's life total.
#[test]
fn platinum_emperion_locks_life_total() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::platinum_emperion());
    let life = g.players[0].life;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt the face");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "life total can't change");
}

/// Madcap Experiment digs to an artifact and bills the reveal count — unless
/// the artifact it finds is Platinum Emperion, which (being on the
/// battlefield before the bill) freezes the life total. Both halves tested.
#[test]
fn madcap_experiment_finds_artifact() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    // add_to_library_bottom: first added = top.
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let vial = g.add_card_to_library(0, catalog::aether_vial());
    let id = g.add_card_to_hand(0, catalog::madcap_experiment());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(vial).is_some(), "artifact onto battlefield");
    assert_eq!(g.players[0].life, life - 3, "3 cards revealed → 3 life");

    // The Emperion line: it's on the battlefield before the bill arrives.
    g.players[0].library.clear();
    let emperion = g.add_card_to_library(0, catalog::platinum_emperion());
    let id2 = g.add_card_to_hand(0, catalog::madcap_experiment());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life2 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id2, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(emperion).is_some(), "Emperion onto battlefield");
    assert_eq!(g.players[0].life, life2, "Emperion freezes the bill");
}

/// Jace, the Mind Sculptor: 0 draws three and stacks two back; -1 bounces.
#[test]
fn jace_brainstorms_and_bounces() {
    let mut g = two_player_game();
    let jace = g.add_card_to_battlefield(0, catalog::jace_the_mind_sculptor());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let hand_before = g.players[0].hand.len();
    g.activate_loyalty_ability(jace, 1, None, None).expect("0: Brainstorm");
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].hand.len(),
        hand_before + 1,
        "draw three, put two back"
    );
    // Next turn: -1 bounces a creature.
    g.battlefield_find_mut(jace).unwrap().loyalty_uses_this_turn = 0;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.activate_loyalty_ability(jace, 2, Some(Target::Permanent(bear)), None)
        .expect("-1: bounce");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "to owner's hand");
}

