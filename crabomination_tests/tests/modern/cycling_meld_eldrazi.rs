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

// ── CR 709.5 — Rooms (Unholy Annex // Ritual Chamber) ───────────────────────

/// Casting the left door unlocks only it: the end-step trigger is live
/// (no Demon → lose 2 + draw), the Chamber's unlock trigger is not.
#[test]
fn room_left_door_cast_unlocks_annex_only() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
        .expect("cast Unholy Annex");
    drain_stack(&mut g);
    let c = g.battlefield_find(room).unwrap();
    assert_eq!(c.unlocked_doors, 1, "left unlocked");
    assert!(!g.battlefield.iter().any(|c| c.is_token), "no Demon minted");
    // End-step trigger fires for the controller; no Demon → draw + lose 2.
    g.active_player_idx = 0;
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew");
    assert_eq!(g.players[0].life, 18, "no Demon: lost 2");
}

/// Casting the right door mints the 6/6 Demon; unlocking the left later at
/// sorcery speed turns the end-step trigger into a drain.
#[test]
fn room_right_door_then_unlock_left() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: true })
        .expect("cast Ritual Chamber");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(room).unwrap().unlocked_doors, 2, "right unlocked");
    let demon = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Demon")
        .expect("6/6 Demon minted");
    assert_eq!((demon.power(), demon.toughness()), (6, 6));
    // Unlock the left door at sorcery speed (CR 709.5e).
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::UnlockRoomDoor { card_id: room, right: false })
        .expect("unlock Unholy Annex");
    assert_eq!(g.battlefield_find(room).unwrap().unlocked_doors, 3, "fully unlocked");
    // With a Demon, the end-step trigger drains.
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "opponent drained 2");
    assert_eq!(g.players[0].life, 22, "gained 2");
}

/// Glassworks' unlock deals 4 to an opponent's creature; Shattered Yard pings
/// each opponent at the end step.
#[test]
fn glassworks_room_unlock_and_endstep_ping() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let room = g.add_card_to_hand(0, catalog::glassworks_shattered_yard());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
        .expect("cast Glassworks");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "4 damage kills the 2/2");
    // Unlock Shattered Yard, then the end step pings the opponent for 1.
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::UnlockRoomDoor { card_id: room, right: true })
        .expect("unlock Shattered Yard");
    drain_stack(&mut g);
    let foe_life = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 1, "Shattered Yard pinged the opponent");
}

/// Unlocked designations are battlefield-only: a bounced Room comes back
/// locked, and a Room round-trips through a snapshot.
#[test]
fn room_designations_reset_on_leave_and_roundtrip_serde() {
    let mut g = two_player_game();
    let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false }).expect("cast");
    drain_stack(&mut g);
    // Snapshot round-trip preserves the unlocked designation.
    let json = serde_json::to_string(&g.battlefield_find(room).unwrap()).unwrap();
    let restored: crabomination::card::CardInstance = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.unlocked_doors, 1, "serde keeps the designation");
    assert_eq!(restored.definition.triggered_abilities.len(), 1, "left door live after load");
    // Bounce: designation clears (CR 709.5c).
    let mut evs = Vec::new();
    g.move_card_to(room, &crabomination::effect::ZoneDest::Hand(crabomination::effect::PlayerRef::Seat(0)),
        &crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0), &mut evs);
    let back = g.players[0].hand.iter().find(|c| c.id == room).expect("bounced");
    assert_eq!(back.unlocked_doors, 0, "locked again");
    assert!(back.definition.triggered_abilities.is_empty(), "no live abilities off-battlefield");
}

/// Bottomless Pool's unlock bounces a creature; Drowned Diner's unlock
/// loots three-for-one; Meat Locker stuns.
#[test]
fn rooms_pool_and_diner_unlock_triggers() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pool = g.add_card_to_hand(0, catalog::bottomless_pool_locker_room());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastRoomDoor { card_id: pool, right: false })
        .expect("cast Bottomless Pool");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));

    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let diner = g.add_card_to_hand(0, catalog::meat_locker_drowned_diner());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    let hand = g.players[0].hand.len() - 1;
    g.perform_action(GameAction::CastRoomDoor { card_id: diner, right: true })
        .expect("cast Drowned Diner");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew 3, discarded 1");
}

/// Meat Locker's unlock taps and double-stuns a creature.
#[test]
fn meat_locker_taps_and_stuns() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let room = g.add_card_to_hand(0, catalog::meat_locker_drowned_diner());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
        .expect("cast Meat Locker");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.tapped, "tapped");
    assert_eq!(c.counter_count(CounterType::Stun), 2, "two stun counters");
}

/// Room affordances surface castable doors from hand and unlockable doors
/// on the battlefield.
#[test]
fn room_affordances_surface_doors() {
    let mut g = two_player_game();
    let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let aff = g.compute_hand_affordances(0);
    assert!(aff.room_castable.contains(&(room, 0)), "left door castable");
    assert!(aff.room_castable.contains(&(room, 1)), "right door castable");
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false }).expect("cast");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    let aff = g.compute_hand_affordances(0);
    assert!(aff.room_unlockable.contains(&(room, 1)), "locked right door unlockable");
    assert!(!aff.room_unlockable.contains(&(room, 0)), "left already unlocked");
}

// ── Amonkhet cycling batch ──────────────────────────────────────────────────

/// Cycling duals enter tapped, tap for both colors, and cycle for {2}.
#[test]
fn cycling_dual_enters_tapped_and_cycles() {
    let mut g = two_player_game();
    let land = g.add_card_to_hand(0, catalog::sheltered_thicket());
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped, "enters tapped");
    g.add_card_to_library(0, catalog::island());
    let second = g.add_card_to_hand(0, catalog::fetid_pools());
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len() - 1;
    g.perform_action(GameAction::Cycle { card_id: second, x_value: None }).expect("cycle");
    assert_eq!(g.players[0].hand.len(), hand + 1, "cycled into a card");
}

/// Gempalm Incinerator's cycle trigger burns for the Goblin count.
#[test]
fn gempalm_incinerator_cycle_burn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::goblin_guide());
    g.add_card_to_battlefield(0, catalog::goblin_guide());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let gp = g.add_card_to_hand(0, catalog::gempalm_incinerator());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::Cycle { card_id: gp, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "2 Goblins = 2 damage killed the bear");
}

/// Curator of Mysteries scries when you cycle another card.
#[test]
fn curator_of_mysteries_scries_on_cycle() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::curator_of_mysteries());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let cy = g.add_card_to_hand(0, catalog::fetid_pools());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Cycle { card_id: cy, x_value: None }).expect("cycle");
    // The scry trigger is on the stack; resolving it consults the decider.
    drain_stack(&mut g);
    // No assert on ordering — reaching here without panicking means the
    // trigger resolved; check the cycled card drew first.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cy));
}

/// Omen of the Sea: flash ETB scry-2 + draw, then sac for value later.
#[test]
fn omen_of_the_sea_etb_and_sac() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let omen = g.add_card_to_hand(0, catalog::omen_of_the_sea());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len() - 1;
    cast(&mut g, omen);
    assert_eq!(g.players[0].hand.len(), hand + 1, "ETB drew");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: omen, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac to scry");
    drain_stack(&mut g);
    assert!(g.battlefield_find(omen).is_none(), "sacrificed");
}

/// Memory Deluge digs X = mana spent (4 normally) and takes two.
#[test]
fn memory_deluge_digs_mana_spent() {
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let md = g.add_card_to_hand(0, catalog::memory_deluge());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len() - 1;
    cast(&mut g, md);
    assert_eq!(g.players[0].hand.len(), hand + 2, "took two of the top four");
    assert_eq!(g.players[0].library.len(), 4, "rest bottomed");
}

/// Horror of the Broken Lands grows on cycling another card.
#[test]
fn horror_grows_on_cycle() {
    let mut g = two_player_game();
    let horror = g.add_card_to_battlefield(0, catalog::horror_of_the_broken_lands());
    g.add_card_to_library(0, catalog::island());
    let cy = g.add_card_to_hand(0, catalog::desert_cerodon());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::Cycle { card_id: cy, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    let cp = g.computed_permanent(horror).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 5), "+2/+1");
}

/// Shefet Monitor's cycle fetches a basic land onto the battlefield.
#[test]
fn shefet_monitor_cycle_fetches_land() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let forest = g.add_card_to_library(0, catalog::forest());
    let monitor = g.add_card_to_hand(0, catalog::shefet_monitor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(forest)),
    ]));
    g.perform_action(GameAction::Cycle { card_id: monitor, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert!(g.battlefield_find(forest).is_some(), "Forest fetched onto battlefield");
}

/// Architects of Will rearranges the top three of a targeted library and
/// cycles for a hybrid pip.
#[test]
fn architects_of_will_etb_and_hybrid_cycle() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    let aw = g.add_card_to_hand(0, catalog::architects_of_will());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aw, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aw).is_some());
    // Hybrid {U/B} cycling pays with black.
    g.add_card_to_library(0, catalog::island());
    let second = g.add_card_to_hand(0, catalog::architects_of_will());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::Cycle { card_id: second, x_value: None }).expect("hybrid cycle");
}

// ── Ikoria cycling payoffs ──────────────────────────────────────────────────

/// Cycling one card pays out across the whole cycle-matters board, and
/// Valiant Rescuer's token is once-per-turn.
#[test]
fn ikoria_cycle_payoffs_fire_on_one_cycle() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let fox = g.add_card_to_battlefield(0, catalog::flourishing_fox());
    g.add_card_to_battlefield(0, catalog::drannith_healer());
    g.add_card_to_battlefield(0, catalog::drannith_stinger());
    g.add_card_to_battlefield(0, catalog::valiant_rescuer());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let c1 = g.add_card_to_hand(0, catalog::imposing_vantasaur());
    let c2 = g.add_card_to_hand(0, catalog::desert_cerodon());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Cycle { card_id: c1, x_value: None }).expect("cycle 1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21, "Healer gained 1");
    assert_eq!(g.players[1].life, 19, "Stinger pinged");
    assert_eq!(g.battlefield_find(fox).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 1, "Rescuer token");
    // Second cycle: Rescuer stays quiet (once per turn), the rest fire again.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::Cycle { card_id: c2, x_value: None }).expect("cycle 2");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 1, "still one token");
    assert_eq!(g.players[1].life, 18, "Stinger pinged again");
}

/// Zenith Flare scales with cycling cards in the graveyard.
#[test]
fn zenith_flare_counts_cycling_cards() {
    let mut g = two_player_game();
    // Three cycling cards + one non-cycling card in the graveyard.
    for f in [catalog::desert_cerodon, catalog::imposing_vantasaur, catalog::street_wraith] {
        let id = g.add_card_to_hand(0, f());
        let mut evs = Vec::new();
        g.discard_card(0, id, &mut evs);
    }
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.discard_card(0, bear, &mut evs);
    let zf = g.add_card_to_hand(0, catalog::zenith_flare());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: zf, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Zenith Flare");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "3 cycling cards = 3 damage");
    assert_eq!(g.players[0].life, 23, "gained 3");
}

/// Savai Thundermane converts {2} into a 2-damage drain on any cycle.
#[test]
fn savai_thundermane_pays_on_cycle() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::savai_thundermane());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let cy = g.add_card_to_hand(0, catalog::desert_cerodon());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::Cycle { card_id: cy, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2 damage killed the bear");
    assert_eq!(g.players[0].life, 22, "gained 2");
}

/// Gisela doubles damage to the opponent's side and halves damage to her
/// controller's side (CR 614.5, side-scoped).
#[test]
fn gisela_scoped_damage_scaling() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gisela_blade_of_goldnight());
    // Bolt at the opponent: 3 → 6.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt opponent");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "3 doubled to 6");
    // Bolt at Gisela's controller: 3 → 1 (prevent half rounded up).
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt controller");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19, "3 halved to 1");
}

// ── Final batch: Siege Rhino, Morbid Opportunist, Aftermath Analyst ─────────

/// Siege Rhino's ETB drains each opponent for 3.
#[test]
fn siege_rhino_etb_drain() {
    let mut g = two_player_game();
    let rhino = g.add_card_to_hand(0, catalog::siege_rhino());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, rhino);
    assert_eq!(g.players[1].life, 17);
    assert_eq!(g.players[0].life, 23);
}

/// Morbid Opportunist draws once per turn no matter how many others die.
#[test]
fn morbid_opportunist_draws_once_per_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::morbid_opportunist());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    for _ in 0..2 {
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().damage = 9;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1, "second death this turn doesn't draw");
}

/// Aftermath Analyst mills three, then its sac returns all graveyard lands
/// tapped.
#[test]
fn aftermath_analyst_mills_then_returns_lands() {
    let mut g = two_player_game();
    for f in [catalog::forest, catalog::island, catalog::grizzly_bears] {
        g.add_card_to_library(0, f());
    }
    let aa = g.add_card_to_hand(0, catalog::aftermath_analyst());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, aa);
    assert_eq!(g.players[0].graveyard.len(), 3, "milled three");
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: aa, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac");
    drain_stack(&mut g);
    let lands: Vec<_> = g.battlefield.iter().filter(|c| c.definition.is_land()).collect();
    assert_eq!(lands.len(), 2, "both lands returned");
    assert!(lands.iter().all(|c| c.tapped), "returned tapped");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "nonland stays in graveyard");
}

// ── Meld (CR 701.37): Urza, Lord Protector + The Mightstone and Weakstone ───

/// Urza's {7} melds with the Mightstone into Urza, Planeswalker (loyalty 7);
/// both components are gone from the battlefield.
#[test]
fn meld_urza_creates_planeswalker() {
    let mut g = two_player_game();
    let urza = g.add_card_to_battlefield(0, catalog::urza_lord_protector());
    g.add_card_to_battlefield(0, catalog::the_mightstone_and_weakstone());
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::ActivateAbility {
        card_id: urza, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("meld");
    drain_stack(&mut g);
    let pw = g.battlefield.iter().find(|c| c.definition.name == "Urza, Planeswalker")
        .expect("melded planeswalker");
    assert_eq!(pw.counter_count(CounterType::Loyalty), 7);
    assert_eq!(pw.meld_parts.len(), 2);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Urza, Lord Protector"));
}

/// CR 701.37b — without the partner, the {7} ability does nothing.
#[test]
fn meld_without_partner_is_noop() {
    let mut g = two_player_game();
    let urza = g.add_card_to_battlefield(0, catalog::urza_lord_protector());
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::ActivateAbility {
        card_id: urza, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == urza), "Urza stays");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Urza, Planeswalker"));
}

/// CR 701.37b — an opponent-owned partner can't meld ("you both own and
/// control").
#[test]
fn meld_requires_owning_both() {
    let mut g = two_player_game();
    let urza = g.add_card_to_battlefield(0, catalog::urza_lord_protector());
    let stone = g.add_card_to_battlefield(1, catalog::the_mightstone_and_weakstone());
    // steal it: controller 0, owner stays 1
    g.battlefield_find_mut(stone).unwrap().controller = 0;
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::ActivateAbility {
        card_id: urza, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Urza, Planeswalker"));
}

/// CR 712.16 — a dying melded permanent goes to the graveyard as both
/// component cards.
#[test]
fn melded_permanent_dies_as_both_cards() {
    let mut g = two_player_game();
    let urza = g.add_card_to_battlefield(0, catalog::urza_lord_protector());
    g.add_card_to_battlefield(0, catalog::the_mightstone_and_weakstone());
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::ActivateAbility {
        card_id: urza, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("meld");
    drain_stack(&mut g);
    let pw = g.battlefield.iter().find(|c| c.definition.name == "Urza, Planeswalker")
        .unwrap().id;
    g.remove_from_battlefield_to_graveyard_raw(pw);
    let names: Vec<&str> = g.players[0].graveyard.iter()
        .map(|c| c.definition.name).collect();
    assert!(names.contains(&"Urza, Lord Protector"));
    assert!(names.contains(&"The Mightstone and Weakstone"));
    assert!(!names.contains(&"Urza, Planeswalker"));
}

/// CR 606.3 override — Urza, Planeswalker activates twice per turn, not
/// three times; +2 grants the artifact/instant/sorcery discount.
#[test]
fn urza_planeswalker_twice_per_turn_and_discount() {
    let mut g = two_player_game();
    let urza = g.add_card_to_battlefield(0, catalog::urza_lord_protector());
    g.add_card_to_battlefield(0, catalog::the_mightstone_and_weakstone());
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::ActivateAbility {
        card_id: urza, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("meld");
    drain_stack(&mut g);
    let pw = g.battlefield.iter().find(|c| c.definition.name == "Urza, Planeswalker")
        .unwrap().id;
    g.step = TurnStep::PreCombatMain;
    for i in 0..2 {
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: pw, ability_index: 0, target: None, x_value: None,
        }).unwrap_or_else(|e| panic!("activation {i}: {e:?}"));
        drain_stack(&mut g);
    }
    let third = g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw, ability_index: 0, target: None, x_value: None,
    });
    assert!(third.is_err(), "third activation rejected");
    assert_eq!(g.players[0].life, 24, "+2 gained twice");
    assert_eq!(g.players[0].turn_spell_discounts.len(), 2);
    // {4} artifact now costs {0}
    let stone = g.add_card_to_hand(0, catalog::mind_stone());
    cast(&mut g, stone);
    assert!(g.battlefield.iter().any(|c| c.id == stone), "free artifact cast");
}

// ── CR 615.7: chosen-source one-event shields (Circle of Protection) ────────

/// CoP: Red's shield soaks one whole damage event from the chosen red
/// source, then expires — a second bolt from the same source connects.
#[test]
fn circle_of_protection_red_soaks_one_event() {
    let mut g = two_player_game();
    let cop = g.add_card_to_battlefield(0, catalog::circle_of_protection_red());
    let goblin = g.add_card_to_battlefield(1, catalog::raging_goblin());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cop, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate CoP");
    drain_stack(&mut g);
    // chosen source = the only red candidate (the Goblin)
    let mut events = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0), 3, Some(goblin), &mut events);
    assert_eq!(g.players[0].life, 20, "first event fully prevented");
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0), 3, Some(goblin), &mut events);
    assert_eq!(g.players[0].life, 17, "shield was one-event only");
}

/// The CoP shield doesn't soak damage from a different source.
#[test]
fn circle_of_protection_shield_is_source_restricted() {
    let mut g = two_player_game();
    let cop = g.add_card_to_battlefield(0, catalog::circle_of_protection_red());
    let goblin = g.add_card_to_battlefield(1, catalog::raging_goblin());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _ = goblin;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cop, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate CoP");
    drain_stack(&mut g);
    let mut events = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0), 2, Some(bears), &mut events);
    assert_eq!(g.players[0].life, 18, "green source ignores the red shield");
    assert_eq!(g.prevention_shields.len(), 1, "shield still up");
}


// ── Tribute (CR 702.104) ─────────────────────────────────────────────────────

/// Tribute paid: the opponent puts the counter on; the trigger half is
/// skipped (no haste / no extra pump).
#[test]
fn tribute_paid_adds_counters_and_skips_trigger() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let id = g.add_card_to_hand(0, catalog::fanatic_of_xenagos());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "tribute paid");
    assert_eq!((c.power(), c.toughness()), (4, 4));
    assert!(!c.granted_keywords_eot.contains(&crabomination::card::Keyword::Haste), "no haste half");
}

/// Tribute declined (AutoDecider): Fanatic gets +1/+1 and haste for the turn.
#[test]
fn tribute_declined_fires_the_trigger_half() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fanatic_of_xenagos());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!((c.power(), c.toughness()), (4, 4), "+1/+1 until EOT");
    let computed = g.compute_battlefield();
    assert!(computed.iter().find(|c| c.id == id).unwrap()
        .keywords.contains(&crabomination::card::Keyword::Haste));
}

/// Oracle of Bones (tribute declined) free-casts an instant from hand; a
/// creature card is not offered.
#[test]
fn oracle_of_bones_free_casts_instant_only() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(false),           // opponent declines tribute
        DecisionAnswer::Cards(vec![bolt]),     // controller picks the bolt
    ]));
    let oracle = g.add_card_to_hand(0, catalog::oracle_of_bones());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, oracle);
    assert_eq!(g.players[1].life, 17, "free Bolt resolved at the opponent");
}

// ── Disturb (CR 702.146) ─────────────────────────────────────────────────────

/// Disturb-casting Baithook Angler from the graveyard resolves the back
/// face: a 1/2 flying Hook-Haunt Drifter.
#[test]
fn disturb_casts_back_face_from_graveyard() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::baithook_angler());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastDisturb { card_id: id, target: None, additional_targets: vec![] }).expect("disturb");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(c.definition.name, "Hook-Haunt Drifter");
    assert!(c.transformed);
    assert_eq!((c.power(), c.toughness()), (1, 2));
    assert!(c.definition.keywords.contains(&Keyword::Flying));
}

/// CR 702.146e — a dying Disturb back face is exiled instead of going to
/// the graveyard.
#[test]
fn disturb_back_face_exiles_instead_of_dying() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::baithook_angler());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastDisturb { card_id: id, target: None, additional_targets: vec![] }).expect("disturb");
    drain_stack(&mut g);
    g.remove_from_battlefield_to_graveyard_raw(id);
    assert!(g.exile.iter().any(|c| c.id == id), "exiled instead");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == id));
}

/// Disturb is graveyard-only and demands the disturb cost.
#[test]
fn disturb_requires_graveyard_and_cost() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::beloved_beggar());
    assert!(g.perform_action(GameAction::CastDisturb { card_id: id, target: None, additional_targets: vec![] }).is_err(),
        "hand card can't be disturb-cast");
    let gy = g.add_card_to_graveyard(0, catalog::beloved_beggar());
    // no mana
    assert!(g.perform_action(GameAction::CastDisturb { card_id: gy, target: None, additional_targets: vec![] }).is_err());
}

/// Lunarch Veteran's front face gains 1 on each other creature ETB.
#[test]
fn lunarch_veteran_front_gains_on_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lunarch_veteran());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bear);
    assert_eq!(g.players[0].life, 21);
}

// ── Modern staples batch: CoCo / Twin / draw-win / discard-matters ──────────

/// Collected Company puts up to two MV≤3 creatures from the top six onto
/// the battlefield; non-eligible cards stay out.
#[test]
fn collected_company_puts_two_cheap_creatures_onto_battlefield() {
    let mut g = two_player_game();
    for f in [
        catalog::grizzly_bears, catalog::island, catalog::raging_goblin,
        catalog::vulpine_goliath, catalog::forest, catalog::island,
    ] {
        g.add_card_to_library(0, f());
    }
    let coco = g.add_card_to_hand(0, catalog::collected_company());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf = g.battlefield.len();
    cast(&mut g, coco);
    assert_eq!(g.battlefield.len(), bf + 2, "two creatures entered");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Raging Goblin"));
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Vulpine Goliath"),
        "MV 5 creature not eligible");
}

/// Splinter Twin grants the enchanted creature a tap ability minting a
/// hasty token copy that's exiled at the next end step.
#[test]
fn splinter_twin_grants_copy_ability() {
    let mut g = two_player_game();
    let exarch = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(exarch);
    let twin = g.add_card_to_hand(0, catalog::splinter_twin());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, twin, Target::Permanent(exarch));
    // the granted ability surfaces past the printed list (index 0 — Bears
    // print none)
    g.perform_action(GameAction::ActivateAbility {
        card_id: exarch, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("granted twin ability");
    drain_stack(&mut g);
    let copies: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Grizzly Bears" && c.is_token).collect();
    assert_eq!(copies.len(), 1, "token copy minted");
}

/// Laboratory Maniac flips an empty-library draw into a win.
#[test]
fn laboratory_maniac_wins_on_empty_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::laboratory_maniac());
    g.players[0].library.clear();
    let opt = g.add_card_to_hand(0, catalog::ponder());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, opt);
    assert!(g.players[1].eliminated, "opponent eliminated — P0 wins");
    assert!(!g.players[0].eliminated);
}

/// Without the Maniac the empty draw still eliminates the drawer.
#[test]
fn empty_draw_without_override_still_loses() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    let opt = g.add_card_to_hand(0, catalog::ponder());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, opt);
    assert!(g.players[0].eliminated);
}

/// Thassa's Oracle wins when devotion to blue covers the (empty) library.
#[test]
fn thassas_oracle_wins_with_empty_library() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    let oracle = g.add_card_to_hand(0, catalog::thassas_oracle());
    g.players[0].mana_pool.add(Color::Blue, 2);
    cast(&mut g, oracle);
    assert!(g.players[1].eliminated, "devotion 2 >= library 0 wins");
}

/// Bedlam Reveler's graveyard-affinity reduction + ETB hand-flush draw 3.
#[test]
fn bedlam_reveler_discount_and_etb() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
    }
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let rev = g.add_card_to_hand(0, catalog::bedlam_reveler());
    g.add_card_to_hand(0, catalog::forest());
    // {6}{R}{R} - 4 IS cards = {2}{R}{R}
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, rev);
    assert!(g.battlefield.iter().any(|c| c.id == rev));
    assert_eq!(g.players[0].hand.len(), 3, "hand flushed, drew 3");
}

/// Tolarian Winds draws as many as it discarded.
#[test]
fn tolarian_winds_swaps_hand() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest());
    let winds = g.add_card_to_hand(0, catalog::tolarian_winds());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, winds);
    assert_eq!(g.players[0].hand.len(), 2, "discarded two, drew two");
}

/// Flameblade Adept grows +1/+0 per discard this turn.
#[test]
fn flameblade_adept_grows_per_discard() {
    let mut g = two_player_game();
    let adept = g.add_card_to_battlefield(0, catalog::flameblade_adept());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest());
    let winds = g.add_card_to_hand(0, catalog::tolarian_winds());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, winds); // discards the two Forests
    let c = g.battlefield_find(adept).unwrap();
    assert_eq!(c.power(), 3, "+1/+0 per discard");
}

/// Hollow One can be cast for free after three discards.
#[test]
fn hollow_one_free_after_three_discards() {
    let mut g = two_player_game();
    g.players[0].cards_discarded_this_turn = 3;
    let hollow = g.add_card_to_hand(0, catalog::hollow_one());
    cast(&mut g, hollow); // {5} - {6} clamps to 0
    assert!(g.battlefield.iter().any(|c| c.id == hollow));
}

/// Jori En draws on the second spell each turn — and only the second.
#[test]
fn jori_en_draws_on_second_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::jori_en_ruin_diver());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand0 = g.players[0].hand.len();
    for i in 0..3 {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast_at(&mut g, bolt, Target::Player(1));
        let expected = hand0 + usize::from(i >= 1);
        assert_eq!(g.players[0].hand.len(), expected, "after spell {}", i + 1);
    }
}

/// Surged Crush of Tentacles bounces the board and leaves an 8/8 Octopus.
#[test]
fn crush_of_tentacles_surged_makes_octopus() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // a spell this turn satisfies surge
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    let crush = g.add_card_to_hand(0, catalog::crush_of_tentacles());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: crush, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("surge cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "board bounced");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Octopus"),
        "surged rider minted the Octopus");
}

/// Lightning Skelemental forces two discards and dies at end of turn.
#[test]
fn lightning_skelemental_discards_and_sacrifices() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::forest());
    let skel = g.add_card_to_hand(0, catalog::lightning_skelemental());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 2);
    cast_at(&mut g, skel, Target::Player(1));
    assert_eq!(g.players[1].hand.len(), 0, "discarded two");
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(skel).is_none(), "sacrificed at end step");
}

/// The bot recasts a graveyard Flashback card when it's the only play.
#[test]
fn bot_offers_flashback_recast() {
    use crabomination::server::bot::{Bot, RandomBot};
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::hellspark_elemental());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let action = RandomBot::new().next_action(&g, 0);
    assert!(matches!(action, Some(GameAction::CastFlashback { card_id, .. }) if card_id == id),
        "bot flashbacks Hellspark Elemental: {action:?}");
}

/// The bot disturb-casts a graveyard DFC when it's the only play.
#[test]
fn bot_offers_disturb_recast() {
    use crabomination::server::bot::{Bot, RandomBot};
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::baithook_angler());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let action = RandomBot::new().next_action(&g, 0);
    assert!(matches!(action, Some(GameAction::CastDisturb { card_id, .. }) if card_id == id),
        "bot disturb-casts Baithook Angler: {action:?}");
}

// ── Tron + Eldrazi lands + Spirits batch ─────────────────────────────────────

/// Urza's Tower taps for {C} alone and {C}{C}{C} with full Tron assembled.
#[test]
fn urza_tron_assembles() {
    let mut g = two_player_game();
    let tower = g.add_card_to_battlefield(0, catalog::urzas_tower());
    g.perform_action(GameAction::ActivateAbility {
        card_id: tower, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap tower");
    assert_eq!(g.players[0].mana_pool.total(), 1, "lone tower: {{C}}");
    g.battlefield_find_mut(tower).unwrap().tapped = false;
    g.players[0].mana_pool = Default::default();
    g.add_card_to_battlefield(0, catalog::urzas_mine());
    g.add_card_to_battlefield(0, catalog::urzas_power_plant());
    g.perform_action(GameAction::ActivateAbility {
        card_id: tower, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap assembled tower");
    assert_eq!(g.players[0].mana_pool.total(), 3, "full tron: {{C}}{{C}}{{C}}");
}

/// Eldrazi Temple's second ability is Eldrazi-creature-spell-only mana.
#[test]
fn eldrazi_temple_restricted_mana() {
    let mut g = two_player_game();
    let temple = g.add_card_to_battlefield(0, catalog::eldrazi_temple());
    g.perform_action(GameAction::ActivateAbility {
        card_id: temple, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for restricted {C}{C}");
    // Restricted {C} sits apart from the free pool (it only becomes
    // spendable through pay_for_spell when the restriction permits).
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2);
    // a non-Eldrazi spell can't spend it
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "restricted mana refuses a Bear");
}

/// Eye of Ugin discounts colorless Eldrazi spells by {2}.
#[test]
fn eye_of_ugin_discounts_eldrazi() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::eye_of_ugin());
    // Reality Smasher {5} -> {3}
    let smasher = g.add_card_to_hand(0, catalog::reality_smasher());
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, smasher);
    assert!(g.battlefield.iter().any(|c| c.id == smasher));
}

/// Kor Firewalker gains 1 when ANY player casts a red spell.
#[test]
fn kor_firewalker_gains_on_red_casts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kor_firewalker());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("opponent bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18, "bolt for 3, gained 1 from the cast");
}

/// Mausoleum Wanderer's sac ability taxes the spell by its power.
#[test]
fn mausoleum_wanderer_taxes_by_power() {
    let mut g = two_player_game();
    let wanderer = g.add_card_to_battlefield(0, catalog::mausoleum_wanderer());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: wanderer, ability_index: 0, target: Some(Target::Permanent(bolt)),
        additional_targets: Vec::new(),
        x_value: None,
    }).expect("sac wanderer");
    drain_stack(&mut g);
    // bolt's controller had no mana left to pay {1} -> countered
    assert_eq!(g.players[0].life, 20, "bolt countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

/// Paradise Mantle grants the equipped creature a tap-for-any-color mana
/// ability.
#[test]
fn paradise_mantle_grants_mana_ability() {
    use crabomination::decision::{DecisionAnswer};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let mantle = g.add_card_to_battlefield(0, catalog::paradise_mantle());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: mantle, target: bear }).expect("equip");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("granted mana ability");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Thought Monitor's affinity discounts it; ETB draws two.
#[test]
fn thought_monitor_affinity_and_draw() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::ornithopter());
    }
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let tm = g.add_card_to_hand(0, catalog::thought_monitor());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1); // {6}{U} - 5 artifacts
    cast(&mut g, tm);
    assert!(g.battlefield.iter().any(|c| c.id == tm));
    assert_eq!(g.players[0].hand.len(), 2, "drew two");
}

/// Rattlechains grants flash to Spirit spells.
#[test]
fn rattlechains_spirit_flash() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rattlechains());
    // not our turn — a Spirit can still be cast
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    let spirit = g.add_card_to_hand(0, catalog::mausoleum_wanderer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spirit, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("flash spirit");
}

// ── Milled triggers (EventKind::CardMilled) ──────────────────────────────────

/// Narcomoeba jumps to the battlefield when milled.
#[test]
fn narcomoeba_returns_when_milled() {
    use crabomination::decision::DecisionAnswer;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::narcomoeba());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(
            &Effect::Mill {
                who: crabomination::effect::Selector::You,
                amount: crabomination::effect::Value::Const(1),
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Narcomoeba"));
}

/// Creeping Chill drains 3 when milled and exiles itself.
#[test]
fn creeping_chill_drains_when_milled() {
    use crabomination::decision::DecisionAnswer;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::creeping_chill());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(
            &Effect::Mill {
                who: crabomination::effect::Selector::You,
                amount: crabomination::effect::Value::Const(1),
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
    assert_eq!(g.players[0].life, 23);
    assert!(g.exile.iter().any(|c| c.definition.name == "Creeping Chill"));
}

// ── One-spell-per-turn locks (Rule of Law family) ────────────────────────────

/// Rule of Law rejects a second cast (any Cast* variant) by either player.
#[test]
fn rule_of_law_blocks_second_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rule_of_law());
    let b1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let b2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast_at(&mut g, b1, Target::Player(1));
    let second = g.perform_action(GameAction::CastSpell {
        card_id: b2, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(matches!(second, Err(GameError::SpellLimitReached)));
    // graveyard recasts are spells too
    let hellspark = g.add_card_to_graveyard(0, catalog::hellspark_elemental());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::CastFlashback {
        card_id: hellspark, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).is_err());
}

/// Archon of Emeria makes an opponent's nonbasic land enter tapped; the
/// controller's own lands are unaffected.
#[test]
fn archon_of_emeria_taps_opponent_nonbasics() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::archon_of_emeria());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_hand(1, catalog::sunbaked_canyon());
    g.perform_action(GameAction::PlayLand(land)).expect("play nonbasic");
    assert!(g.battlefield_find(land).unwrap().tapped, "entered tapped");
    let basic = g.add_card_to_hand(1, catalog::mountain());
    g.players[1].lands_played_this_turn = 0;
    g.perform_action(GameAction::PlayLand(basic)).expect("play basic");
    assert!(!g.battlefield_find(basic).unwrap().tapped, "basics unaffected");
}

// ── Eldrazi / graveyard-matters batch ────────────────────────────────────────

#[test]
fn prized_amalgam_returns_tapped_at_next_end_step_after_gy_reanimation() {
    use crabomination::effect::{PlayerRef, ZoneDest};
    let mut g = two_player_game();
    let amalgam = g.add_card_to_graveyard(0, catalog::prized_amalgam());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    // Reanimate the bear (gy → battlefield) and dispatch its ETB events.
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let mut events = Vec::new();
    g.move_card_to(
        bear,
        &ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        &ctx,
        &mut events,
    );
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.delayed_triggers.iter().any(|t|
        t.kind == crabomination::game::types::DelayedKind::NextEndStep),
        "Amalgam registers an end-step return");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let a = g.battlefield_find(amalgam).expect("Amalgam returned");
    assert!(a.tapped, "returns tapped");
}

#[test]
fn prized_amalgam_ignores_creatures_entering_from_hand() {
    let mut g = two_player_game();
    let _amalgam = g.add_card_to_graveyard(0, catalog::prized_amalgam());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bear }]);
    drain_stack(&mut g);
    assert!(!g.delayed_triggers.iter().any(|t|
        t.kind == crabomination::game::types::DelayedKind::NextEndStep),
        "a non-graveyard entry must not queue the return");
}

#[test]
fn chord_of_calling_fetches_creature_with_mana_value_at_most_x() {
    use crabomination::effect::{Effect, PlayerRef, ZoneDest};
    use crabomination::card::SelectionRequirement;
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
    let wurm = g.add_card_to_library(0, catalog::pelakka_wurm()); // MV 7
    let effect = Effect::Search {
        who: PlayerRef::You,
        filter: SelectionRequirement::Creature
            .and(SelectionRequirement::ManaValueAtMostXFromCost),
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
    };
    // The wurm is not in the eligible set, so a scripted pick of it is
    // rejected; pick the bear (the only X≤2 candidate).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 2);
    let _ = g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.battlefield_find(bear).is_some(), "MV-2 creature fetched with X=2");
    assert!(g.battlefield_find(wurm).is_none(), "MV-7 creature is not eligible");
}

#[test]
fn shadowspear_activation_strips_hexproof_and_indestructible_until_eot() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let spear = g.add_card_to_battlefield(0, catalog::shadowspear());
    let troll = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(troll).unwrap().granted_keywords_eot.push(Keyword::Hexproof);
    g.battlefield_find_mut(troll).unwrap().granted_keywords_eot.push(Keyword::Indestructible);
    let ability = catalog::shadowspear().activated_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(spear, 0, None);
    let _ = g.resolve_effect(&ability, &ctx).unwrap();
    let c = g.battlefield_find(troll).unwrap();
    assert!(!c.has_keyword(&Keyword::Hexproof), "hexproof stripped");
    assert!(!c.has_keyword(&Keyword::Indestructible), "indestructible stripped");
    assert!(!c.is_indestructible(), "destroyable for the turn");
}

#[test]
fn all_is_dust_sweeps_colored_permanents_only() {
    use crabomination::effect::Effect;
    let mut g = two_player_game();
    let my_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let stone = g.add_card_to_battlefield(0, catalog::oblivion_stone()); // colorless
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let effect = match catalog::all_is_dust().effect {
        e @ Effect::SacrificeAllMatching { .. } => e,
        other => panic!("unexpected effect shape: {other:?}"),
    };
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let _ = g.resolve_effect(&effect, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(my_bear).is_none(), "your colored permanent is sacrificed");
    assert!(g.battlefield_find(opp_bear).is_none(), "opp colored permanent is sacrificed");
    assert!(g.battlefield_find(stone).is_some(), "colorless artifact survives");
    assert!(g.battlefield_find(land).is_some(), "lands survive");
}

#[test]
fn oblivion_stone_spares_fate_countered_permanents_then_clears_fate() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(0, catalog::oblivion_stone());
    let saved = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let doomed = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(saved).unwrap().add_counters(CounterType::Fate, 1);
    let nuke = catalog::oblivion_stone().activated_abilities[1].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(stone, 0, None);
    let _ = g.resolve_effect(&nuke, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(saved).is_some(), "fate-countered creature survives");
    assert!(g.battlefield_find(doomed).is_none(), "unprotected creature destroyed");
    assert_eq!(
        g.battlefield_find(saved).unwrap().counter_count(CounterType::Fate),
        0,
        "fate counters removed afterwards"
    );
}

#[test]
fn emrakul_cannot_be_targeted_by_colored_spells() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let emrakul = g.add_card_to_battlefield(0, catalog::emrakul_the_aeons_torn());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(emrakul)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(err.is_err(), "a colored spell can't target Emrakul");
}

#[test]
fn emrakul_in_graveyard_shuffles_it_back_into_library() {
    let mut g = two_player_game();
    let emrakul = g.add_card_to_battlefield(0, catalog::emrakul_the_aeons_torn());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let lib_before = g.players[0].library.len();
    let mut events = Vec::new();
    g.sacrifice_one(emrakul, 0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.is_empty(), "graveyard shuffled away");
    assert_eq!(g.players[0].library.len(), lib_before + 2, "Emrakul + bear shuffled in");
}

#[test]
fn chord_of_calling_rejects_pick_above_x() {
    use crabomination::effect::{Effect, PlayerRef, ZoneDest};
    use crabomination::card::SelectionRequirement;
    let mut g = two_player_game();
    let wurm = g.add_card_to_library(0, catalog::pelakka_wurm()); // MV 7
    let effect = Effect::Search {
        who: PlayerRef::You,
        filter: SelectionRequirement::Creature
            .and(SelectionRequirement::ManaValueAtMostXFromCost),
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
    };
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(wurm))]));
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 2);
    let _ = g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.battlefield_find(wurm).is_none(), "an over-X pick must be rejected");
}

#[test]
fn relentless_assault_untaps_attackers_and_adds_post_main_combat() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().attacked_this_turn = true;
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let effect = catalog::relentless_assault().effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let _ = g.resolve_effect(&effect, &ctx).unwrap();
    assert!(!g.battlefield_find(bear).unwrap().tapped, "attacker untapped");
    assert_eq!(g.additional_post_main_combats, 1);

    // Leaving the postcombat main loops back to Begin Combat once.
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PostCombatMain;
    let mut steps = Vec::new();
    for _ in 0..24 {
        g.perform_action(GameAction::PassPriority).unwrap();
        if steps.last() != Some(&g.step) {
            steps.push(g.step);
        }
        if g.step == TurnStep::End { break; }
    }
    assert!(steps.contains(&TurnStep::BeginCombat),
        "an extra combat phase begins after the main phase: {steps:?}");
    let after_combat = steps.iter().skip_while(|s| **s != TurnStep::EndCombat).nth(1);
    assert_eq!(after_combat, Some(&TurnStep::PostCombatMain),
        "the extra combat is followed by an additional main phase");
}

/// CR 113.9 targeting precision: Stifle's filter rejects a permanent with
/// no ability on the stack.
#[test]
fn stifle_rejects_target_without_ability_on_stack() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let stifle = g.add_card_to_hand(1, catalog::stifle());
    g.players[1].mana_pool.add(Color::Blue, 1);
    let r = g.perform_action(GameAction::CastSpell {
        card_id: stifle,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(r.is_err(), "no ability on the stack from that source → illegal target");
}

/// CR 702.113 Awaken — Part the Waterveil's awaken cast animates the
/// targeted land into a 6/6 (0/0 + six counters) Elemental with haste and
/// still banks the extra turn.
#[test]
fn part_the_waterveil_awaken_animates_a_land_and_banks_extra_turn() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::part_the_waterveil());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: spell,
        pitch_card: None,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("awaken cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].extra_turns, 1, "extra turn banked");
    let cp = g.compute_battlefield();
    let v = cp.iter().find(|c| c.id == land).unwrap();
    assert!(v.card_types.contains(&CardType::Creature), "land animated");
    assert!(v.card_types.contains(&CardType::Land), "still a land");
    assert_eq!((v.power, v.toughness), (6, 6), "0/0 + six +1/+1 counters");
    assert!(v.keywords.contains(&crabomination::card::Keyword::Haste));
}

/// CR 702.113a — the regular cast must not animate anything.
#[test]
fn part_the_waterveil_regular_cast_skips_awaken() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::part_the_waterveil());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("regular cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].extra_turns, 1);
    let cp = g.compute_battlefield();
    let v = cp.iter().find(|c| c.id == land).unwrap();
    assert!(!v.card_types.contains(&CardType::Creature), "land untouched");
}

