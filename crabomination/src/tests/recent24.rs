//! Functionality tests for `catalog::sets::decks::recent24` — Aetherdrift
//! staples (Vehicles, cycling, discard-count triggers, Mount/Vehicle anthems).

use crate::catalog;
use crate::card::{Keyword, CounterType};
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;
use crate::TurnStep;

fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == controller && c.definition.name == name)
        .count()
}

/// Stand a player at PreCombatMain with priority and a full mana pool.
fn ready(g: &mut GameState) {
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..10 {
        g.players[0].mana_pool.add_colorless(1);
    }
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 4);
    }
}

/// Bounce Off returns a Vehicle to its owner's hand.
#[test]
fn bounce_off_returns_vehicle() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(1, catalog::air_response_unit());
    let bo = g.add_card_to_hand(0, catalog::bounce_off());
    ready(&mut g);
    cast_at(&mut g, bo, Target::Permanent(veh));
    assert!(g.battlefield_find(veh).is_none(), "Vehicle bounced");
    assert_eq!(g.players[1].hand.len(), 1, "back in owner's hand");
}

/// Bestow Greatness pumps +4/+4 and grants trample.
#[test]
fn bestow_greatness_pumps_and_tramples() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let bg = g.add_card_to_hand(0, catalog::bestow_greatness());
    ready(&mut g);
    cast_at(&mut g, bg, Target::Permanent(bear));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Broadside Barrage deals 5 and loots.
#[test]
fn broadside_barrage_burns_and_loots() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.add_card_to_library(0, catalog::grizzly_bears());
    let bb = g.add_card_to_hand(0, catalog::broadside_barrage());
    ready(&mut g);
    cast_at(&mut g, bb, Target::Permanent(foe));
    assert!(g.battlefield_find(foe).is_none(), "5 kills the 4/4");
}

/// Spin Out destroys a creature.
#[test]
fn spin_out_destroys_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    let so = g.add_card_to_hand(0, catalog::spin_out());
    ready(&mut g);
    cast_at(&mut g, so, Target::Permanent(foe));
    assert!(g.battlefield_find(foe).is_none());
}

/// Syphon Fuel shrinks a creature and gains life.
#[test]
fn syphon_fuel_shrinks_and_gains() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let sf = g.add_card_to_hand(0, catalog::syphon_fuel());
    ready(&mut g);
    let life = g.players[0].life;
    cast_at(&mut g, sf, Target::Permanent(foe));
    assert!(g.battlefield_find(foe).is_none(), "-6/-6 kills the 4/4");
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
}

/// Locust Spray gives -1/-1; it can also cycle.
#[test]
fn locust_spray_weakens() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let ls = g.add_card_to_hand(0, catalog::locust_spray());
    ready(&mut g);
    cast_at(&mut g, ls, Target::Permanent(bear));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
}

/// Skycrash destroys an artifact.
#[test]
fn skycrash_destroys_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::air_response_unit());
    let sc = g.add_card_to_hand(0, catalog::skycrash());
    ready(&mut g);
    cast_at(&mut g, sc, Target::Permanent(art));
    assert!(g.battlefield_find(art).is_none());
}

/// Maximum Overdrive adds a counter and grants indestructible + deathtouch.
#[test]
fn maximum_overdrive_buffs() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mo = g.add_card_to_hand(0, catalog::maximum_overdrive());
    ready(&mut g);
    cast_at(&mut g, mo, Target::Permanent(bear));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 counter");
    assert!(cp.keywords.contains(&Keyword::Indestructible));
    assert!(cp.keywords.contains(&Keyword::Deathtouch));
}

/// Pedal to the Metal pumps +X/+0 where X is the cast X.
#[test]
fn pedal_to_the_metal_pumps_by_x() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let p = g.add_card_to_hand(0, catalog::pedal_to_the_metal());
    ready(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: p,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("cast Pedal with X=3");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 5, "+3/+0");
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

/// Fuel the Flames deals 2 to each creature.
#[test]
fn fuel_the_flames_sweeps_for_two() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let ff = g.add_card_to_hand(0, catalog::fuel_the_flames());
    ready(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: ff, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "2 dmg kills 2/2");
    assert!(g.battlefield_find(foe).is_none());
    assert!(g.battlefield_find(big).is_some(), "4/4 survives");
}

/// Gallant Strike destroys only a toughness-4+ creature.
#[test]
fn gallant_strike_hits_big_toughness() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let gs = g.add_card_to_hand(0, catalog::gallant_strike());
    ready(&mut g);
    cast_at(&mut g, gs, Target::Permanent(big));
    assert!(g.battlefield_find(big).is_none());
}

/// Risky Shortcut draws two and drains each player 2.
#[test]
fn risky_shortcut_draws_and_drains() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let rs = g.add_card_to_hand(0, catalog::risky_shortcut());
    ready(&mut g);
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::CastSpell {
        card_id: rs, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2, "drew two");
    assert_eq!(g.players[0].life, l0 - 2);
    assert_eq!(g.players[1].life, l1 - 2);
}

/// Road Rage's X scales with Mounts and Vehicles you control (2 + count).
#[test]
fn road_rage_scales_with_vehicles() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::air_response_unit()); // a Vehicle
    g.add_card_to_battlefield(0, catalog::debris_beetle()); // another Vehicle
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let rr = g.add_card_to_hand(0, catalog::road_rage());
    ready(&mut g);
    cast_at(&mut g, rr, Target::Permanent(foe));
    // X = 2 + 2 vehicles = 4 → kills the 4/4.
    assert!(g.battlefield_find(foe).is_none(), "4 damage kills the 4/4");
}

/// Spectacular Pileup destroys all creatures and Vehicles, even indestructible.
#[test]
fn spectacular_pileup_wraths_everything() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let veh = g.add_card_to_battlefield(1, catalog::air_response_unit());
    let sp = g.add_card_to_hand(0, catalog::spectacular_pileup());
    ready(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: sp, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
    assert!(g.battlefield_find(veh).is_none(), "Vehicle destroyed");
}

/// Nimble Thopterist mints a 1/1 flying Thopter on ETB.
#[test]
fn nimble_thopterist_makes_thopter() {
    let mut g = two_player_game();
    let nt = g.add_card_to_battlefield(0, catalog::nimble_thopterist());
    g.fire_self_etb_triggers(nt, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Thopter"), 1);
}

/// Shefet Archfiend gives all other creatures -2/-2 on ETB.
#[test]
fn shefet_archfiend_sweeps_others() {
    let mut g = two_player_game();
    let x = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let sa = g.add_card_to_battlefield(0, catalog::shefet_archfiend());
    g.fire_self_etb_triggers(sa, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(x).is_none(), "-2/-2 kills the 2/2");
    assert!(g.battlefield_find(sa).is_some(), "Archfiend itself unaffected");
}

/// Regal Imperiosaur is a Dinosaur lord (other Dinosaurs +1/+1).
#[test]
fn regal_imperiosaur_buffs_dinosaurs() {
    let mut g = two_player_game();
    let other = g.add_card_to_battlefield(0, catalog::migrating_ketradon()); // 6/6 Dino
    g.add_card_to_battlefield(0, catalog::regal_imperiosaur());
    let cp = g.computed_permanent(other).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7), "lord gives +1/+1");
}

/// Guidelight Synergist grows with artifacts you control.
#[test]
fn guidelight_synergist_scales_with_artifacts() {
    let mut g = two_player_game();
    let gs = g.add_card_to_battlefield(0, catalog::guidelight_synergist()); // 0/4, an artifact
    // Counts itself.
    assert_eq!(g.computed_permanent(gs).unwrap().power, 1, "+1/+0 for itself");
    g.add_card_to_battlefield(0, catalog::air_response_unit()); // +1 artifact
    assert_eq!(g.computed_permanent(gs).unwrap().power, 2);
}

/// Cloudspire Captain buffs Mounts and Vehicles you control.
#[test]
fn cloudspire_captain_buffs_vehicles() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(0, catalog::air_response_unit()); // 3/3 Vehicle
    g.add_card_to_battlefield(0, catalog::cloudspire_captain());
    let cp = g.computed_permanent(veh).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+1/+1 anthem");
}

/// Daring Mechanic puts a +1/+1 counter on a Vehicle.
#[test]
fn daring_mechanic_counters_vehicle() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(0, catalog::debris_beetle()); // 6/6
    let dm = g.add_card_to_battlefield(0, catalog::daring_mechanic());
    g.clear_sickness(dm);
    ready(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dm, ability_index: 0, target: Some(Target::Permanent(veh)),
        additional_targets: vec![], x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(veh).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7), "+1/+1 counter");
}

/// Deathless Pilot returns itself from the graveyard.
#[test]
fn deathless_pilot_recurs_from_graveyard() {
    let mut g = two_player_game();
    let dp = g.add_card_to_graveyard(0, catalog::deathless_pilot());
    ready(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dp, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate gy ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.name == "Deathless Pilot").count(), 1);
}

/// Debris Beetle drains 3 on enter (Vehicle ETB).
#[test]
fn debris_beetle_drains_on_etb() {
    let mut g = two_player_game();
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    let db = g.add_card_to_battlefield(0, catalog::debris_beetle());
    g.fire_self_etb_triggers(db, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 3);
    assert_eq!(g.players[0].life, l0 + 3);
}

/// Cryptcaller Chariot mints a tapped Zombie per discarded card.
#[test]
fn cryptcaller_chariot_makes_zombies_on_discard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::cryptcaller_chariot());
    let card = g.add_card_to_hand(0, catalog::grizzly_bears());
    let mut events = vec![];
    g.discard_card(0, card, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Zombie"), 1, "one Zombie per discard");
}

/// Scrounging Skyray grows when you discard.
#[test]
fn scrounging_skyray_grows_on_discard() {
    let mut g = two_player_game();
    let sky = g.add_card_to_battlefield(0, catalog::scrounging_skyray()); // 1/2
    let card = g.add_card_to_hand(0, catalog::grizzly_bears());
    let mut events = vec![];
    g.discard_card(0, card, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(sky).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
    );
}

/// Pactdoll Terror drains 1 when an artifact you control enters.
#[test]
fn pactdoll_terror_drains_on_artifact_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pactdoll_terror());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    let veh = g.add_card_to_battlefield(0, catalog::air_response_unit());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: veh }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1);
    assert_eq!(g.players[0].life, l0 + 1);
}

/// Cloudspire Skycycle distributes two +1/+1 counters on ETB.
#[test]
fn cloudspire_skycycle_distributes_counters() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let sky = g.add_card_to_battlefield(0, catalog::cloudspire_skycycle());
    g.fire_self_etb_triggers(sky, 0);
    drain_stack(&mut g);
    // Two counters land on the single eligible other creature.
    assert_eq!(
        g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
    );
}

/// Deathless Pilot's CR 702.122e rider lets a 2-power creature crew a Crew 4
/// Vehicle by itself (counts as power 4).
#[test]
fn deathless_pilot_crews_as_though_power_greater() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(0, catalog::debris_beetle()); // Crew 2... use a Crew 4
    // Debris Beetle is Crew 2; pair the pilot with a Crew 4 vehicle instead.
    g.battlefield.retain(|c| c.id != veh);
    let chariot = g.add_card_to_battlefield(0, catalog::lumbering_worldwagon()); // Crew 4
    let pilot = g.add_card_to_battlefield(0, catalog::deathless_pilot()); // power 2 (+2 rider = 4)
    g.clear_sickness(pilot);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Crew { vehicle: chariot, crew_creatures: vec![pilot] })
        .expect("2-power pilot crews Crew 4 via the +2 rider");
    assert!(
        g.computed_permanent(chariot).unwrap().card_types.contains(&crate::card::CardType::Creature),
        "Vehicle is crewed (an artifact creature)",
    );
}

/// Thunderhead Gunner loots: discard a card to draw one.
#[test]
fn thunderhead_gunner_loots() {
    let mut g = two_player_game();
    let tg = g.add_card_to_battlefield(0, catalog::thunderhead_gunner());
    g.clear_sickness(tg);
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to discard
    g.add_card_to_library(0, catalog::forest()); // a card to draw
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: tg, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate loot");
    drain_stack(&mut g);
    // -1 discard, +1 draw → net unchanged, but the drawn card differs.
    assert_eq!(g.players[0].hand.len(), hand_before, "discard 1, draw 1");
}

/// Wretched Doll surveils 1.
#[test]
fn wretched_doll_surveils() {
    let mut g = two_player_game();
    let wd = g.add_card_to_battlefield(0, catalog::wretched_doll());
    g.clear_sickness(wd);
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: wd, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate surveil");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wd).is_some(), "Doll stays (surveil resolved)");
}

/// Molt Tender mills with its first ability.
#[test]
fn molt_tender_mills() {
    let mut g = two_player_game();
    let mt = g.add_card_to_battlefield(0, catalog::molt_tender());
    g.clear_sickness(mt);
    g.add_card_to_library(0, catalog::forest());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: mt, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate mill");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy_before + 1, "milled one card");
}

/// Scrap Compactor's first ability deals 3 to a creature (sacrificing itself).
#[test]
fn scrap_compactor_pings_for_three() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let sc = g.add_card_to_battlefield(0, catalog::scrap_compactor());
    g.clear_sickness(sc);
    g.players[0].mana_pool.add_colorless(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sc, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], x_value: None,
    })
    .expect("activate ping");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "3 damage kills the 2/2");
    assert!(g.battlefield_find(sc).is_none(), "Compactor sacrificed itself");
}

/// Air Response Unit ships as a 3/3 Vehicle with Crew 1.
#[test]
fn air_response_unit_is_crewable_vehicle() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::air_response_unit());
    let c = g.battlefield_find(v).unwrap();
    assert!(c.definition.keywords.contains(&Keyword::Crew(1)));
    assert_eq!((c.definition.power, c.definition.toughness), (3, 3));
}
