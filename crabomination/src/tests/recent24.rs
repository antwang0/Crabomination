//! Functionality tests for `catalog::sets::decks::recent24` — Aetherdrift
//! staples (Vehicles, cycling, discard-count triggers, Mount/Vehicle anthems).

use crate::catalog;
use crate::card::{CardType, CounterType, CreatureType, Keyword, Subtypes, TokenDefinition};
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::mana::Color;
use crate::TurnStep;

fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == controller && c.definition.name == name)
        .count()
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
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

/// Cult Healer's Eerie trigger fires when an enchantment (a Room) enters,
/// granting it lifelink until end of turn.
#[test]
fn cult_healer_eerie_enchantment_enters() {
    let mut g = two_player_game();
    let healer = g.add_card_to_battlefield(0, catalog::cult_healer());
    let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
    ready(&mut g);
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
        .expect("cast Room (enchantment enters)");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(healer).unwrap().keywords.contains(&Keyword::Lifelink),
        "Cult Healer gained lifelink from Eerie",
    );
}

/// Balemurk Leech's Eerie trigger fires on fully unlocking a Room (both
/// doors): the opponent loses 1 life.
#[test]
fn balemurk_leech_eerie_room_fully_unlocked() {
    let mut g = two_player_game();
    let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
    ready(&mut g);
    // Open the right door first (no Leech yet → enchantment-enters is a no-op).
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: true })
        .expect("cast right door");
    drain_stack(&mut g);
    g.add_card_to_battlefield(0, catalog::balemurk_leech());
    let foe_life = g.players[1].life;
    // Unlock the left door → room fully unlocked → only the Eerie trigger.
    g.perform_action(GameAction::UnlockRoomDoor { card_id: room, right: false })
        .expect("unlock left door");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(room).unwrap().unlocked_doors, 0b11, "fully unlocked");
    assert_eq!(g.players[1].life, foe_life - 1, "opponent lost 1 from Eerie");
}

/// Unwilling Vessel mints an X/X flying Spirit on death where X is its
/// possession counters (LKI counter read, CR 603.10).
#[test]
fn unwilling_vessel_dies_mints_spirit() {
    let mut g = two_player_game();
    let uv = g.add_card_to_battlefield(0, catalog::unwilling_vessel());
    {
        let c = g.battlefield_find_mut(uv).unwrap();
        c.add_counters(CounterType::Possession, 2);
        c.damage = 2; // lethal vs 2 toughness
    }
    g.priority.player_with_priority = 0;
    g.check_state_based_actions();
    drain_stack(&mut g);
    let spirit = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.definition.name == "Spirit")
        .expect("Spirit token minted");
    assert_eq!((spirit.power(), spirit.toughness()), (2, 2), "X/X from 2 possession counters");
    assert!(spirit.definition.keywords.contains(&Keyword::Flying), "Spirit flies");
}

/// Gremlin Tamer's Eerie trigger mints a Gremlin when an enchantment enters.
#[test]
fn gremlin_tamer_eerie_makes_gremlin() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gremlin_tamer());
    let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
    ready(&mut g);
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
        .expect("cast Room");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Gremlin"), 1, "Eerie minted a Gremlin");
}

/// Erratic Apparition's Eerie trigger pumps it +1/+1.
#[test]
fn erratic_apparition_eerie_pumps() {
    let mut g = two_player_game();
    let ea = g.add_card_to_battlefield(0, catalog::erratic_apparition());
    let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
    ready(&mut g);
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
        .expect("cast Room");
    drain_stack(&mut g);
    let cp = g.computed_permanent(ea).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4), "+1/+1 from Eerie");
}

/// Commune with Evil puts one of the top four into hand, the rest into the
/// graveyard, and gains 3 life.
#[test]
fn commune_with_evil_digs_and_gains() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let cwe = g.add_card_to_hand(0, catalog::commune_with_evil());
    ready(&mut g);
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    g.perform_action(GameAction::CastSpell {
        card_id: cwe, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Commune with Evil");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "one card to hand (spell left hand)");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 3, "rest to graveyard");
    assert_eq!(g.players[0].life, life + 3, "gained 3 life");
}

/// Acrobatic Cheerleader's Survival gives it a flying counter once it's tapped.
#[test]
fn acrobatic_cheerleader_survival_flies() {
    let mut g = two_player_game();
    let ac = g.add_card_to_battlefield(0, catalog::acrobatic_cheerleader());
    g.battlefield_find_mut(ac).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(ac).unwrap().keywords.contains(&Keyword::Flying),
        "gained flying from Survival",
    );
}

/// Clockwork Percussionist's death exiles the top card and grants a may-play.
#[test]
fn clockwork_percussionist_dies_impulse() {
    let mut g = two_player_game();
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    let cp = g.add_card_to_battlefield(0, catalog::clockwork_percussionist());
    g.battlefield_find_mut(cp).unwrap().damage = 1; // lethal vs 1 toughness
    g.priority.player_with_priority = 0;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == top), "top card exiled with may-play");
}

/// Diversion Specialist sacrifices a creature to impulse the top card.
#[test]
fn diversion_specialist_sac_impulses() {
    let mut g = two_player_game();
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::diversion_specialist());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    ready(&mut g);
    let ds = g.battlefield.iter().find(|c| c.definition.name == "Diversion Specialist").unwrap().id;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ds, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate Diversion Specialist");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert!(g.exile.iter().any(|c| c.id == top), "top card exiled to play this turn");
}

/// Sumala Sentry counters itself and the turned card when a face-down
/// permanent you control is turned face up.
#[test]
fn sumala_sentry_turn_face_up_counters() {
    let mut g = two_player_game();
    let ss = g.add_card_to_battlefield(0, catalog::sumala_sentry());
    let top = g.next_id();
    g.players[0].library.insert(0, crate::card::CardInstance::new(top, catalog::elder_gargaroth(), 0));
    let ctx = crate::game::effects::EffectContext::for_ability(top, 0, None);
    let mut events = vec![];
    g.manifest_card(top, 0, &ctx, &mut events);
    ready(&mut g);
    g.perform_action(GameAction::TurnFaceUp { card_id: top }).expect("turn face up");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ss).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Sumala got a +1/+1 counter",
    );
    assert_eq!(
        g.battlefield_find(top).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "turned card got a +1/+1 counter",
    );
}

/// Cryptid Inspector counters itself when a face-down permanent enters and
/// when one is turned face up.
#[test]
fn cryptid_inspector_face_down_matters() {
    let mut g = two_player_game();
    let ci = g.add_card_to_battlefield(0, catalog::cryptid_inspector());
    let top = g.next_id();
    g.players[0].library.insert(0, crate::card::CardInstance::new(top, catalog::elder_gargaroth(), 0));
    let ctx = crate::game::effects::EffectContext::for_ability(top, 0, None);
    let mut events = vec![];
    g.manifest_card(top, 0, &ctx, &mut events);
    g.dispatch_triggers_for_events(&events); // face-down permanent entered
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ci).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "+1/+1 when a face-down permanent entered",
    );
    ready(&mut g);
    g.perform_action(GameAction::TurnFaceUp { card_id: top }).expect("turn face up");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ci).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "+1/+1 again when turned face up",
    );
}

/// Fanatic of the Harrowing makes each player discard, then its controller draws.
#[test]
fn fanatic_of_the_harrowing_discards_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let foh = g.add_card_to_hand(0, catalog::fanatic_of_the_harrowing());
    ready(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: foh, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Fanatic");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 1, "opponent discarded one");
    // P0 discarded the fodder and drew the Forest: net hand back to one card.
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"), "you discarded");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "you drew");
}

/// Spectral Snatcher carries Ward—Discard and Swampcycling.
#[test]
fn spectral_snatcher_keywords() {
    let def = catalog::spectral_snatcher();
    assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Ward(crate::card::WardCost::Discard(1)))));
    assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Landcycling(_, crate::card::LandType::Swamp))));
}

/// Ghostly Keybearer carries a combat-damage trigger; the UnlockRoomDoor
/// effect it fires opens a still-locked door of the targeted Room.
#[test]
fn ghostly_keybearer_unlocks_a_door() {
    use crate::card::Effect;
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let gk = g.add_card_to_battlefield(0, catalog::ghostly_keybearer());
    let def = catalog::ghostly_keybearer();
    assert_eq!(
        def.triggered_abilities[0].event.kind,
        crate::card::EventKind::DealsCombatDamageToPlayer,
        "fires on dealing combat damage to a player",
    );
    // A Room with its left door unlocked; the right is still locked.
    let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
    ready(&mut g);
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
        .expect("cast left door");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(room).unwrap().unlocked_doors, 0b01, "only left open");
    // Resolve the unlock effect directly (the trigger body) against the Room.
    let ctx = EffectContext::for_ability(gk, 0, Some(Target::Permanent(room)));
    let evs = g
        .resolve_effect(
            &Effect::UnlockRoomDoor { what: crate::card::Selector::Target(0) },
            &ctx,
        )
        .expect("unlock effect");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(room).unwrap().unlocked_doors, 0b11, "right door unlocked");
}

/// Enduring Tenacity drains an opponent when you gain life.
#[test]
fn enduring_tenacity_drains_on_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::enduring_tenacity());
    let foe = g.players[1].life;
    let ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(999), 0, None);
    let events = g
        .resolve_effect(
            &crate::card::Effect::GainLife {
                who: crate::card::Selector::You,
                amount: crate::card::Value::Const(3),
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe - 3, "opponent lost the life gained");
}

/// Threats Around Every Corner manifests dread on ETB.
#[test]
fn threats_around_every_corner_manifests() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let tac = g.add_card_to_hand(0, catalog::threats_around_every_corner());
    ready(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: tac, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Threats");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.face_down), "manifested a face-down 2/2");
}

/// Insidious Fungus sacrifices itself to destroy an artifact.
#[test]
fn insidious_fungus_sacs_to_destroy_artifact() {
    let mut g = two_player_game();
    let fungus = g.add_card_to_battlefield(0, catalog::insidious_fungus());
    let art = g.add_card_to_battlefield(1, catalog::ornithopter());
    g.clear_sickness(fungus);
    ready(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fungus, ability_index: 0, target: Some(Target::Permanent(art)),
        additional_targets: vec![], x_value: None,
    })
    .expect("activate Insidious Fungus (mode 0)");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fungus).is_none(), "Fungus sacrificed");
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
}

/// Winter's Intervention deals 2 to a creature and gains 2 life.
#[test]
fn winters_intervention_burns_and_gains() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let wi = g.add_card_to_hand(0, catalog::winters_intervention());
    ready(&mut g);
    let life = g.players[0].life;
    cast_at(&mut g, wi, Target::Permanent(foe));
    assert!(g.battlefield_find(foe).is_none(), "2 kills the 2/2");
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
}

/// Shroudstomper drains and draws on enter.
#[test]
fn shroudstomper_etb_drains_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let ss = g.add_card_to_hand(0, catalog::shroudstomper());
    ready(&mut g);
    let (foe, life, hand) = (g.players[1].life, g.players[0].life, g.players[0].hand.len());
    g.perform_action(GameAction::CastSpell {
        card_id: ss, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Shroudstomper");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe - 2, "opponent lost 2");
    assert_eq!(g.players[0].life, life + 2, "gained 2");
    // Hand: -1 (cast Shroudstomper) +1 (drew) = net same.
    assert_eq!(g.players[0].hand.len(), hand, "drew a card");
}

/// Patched Plaything enters with two -1/-1 counters only when cast from hand.
#[test]
fn patched_plaything_cast_zone_counters() {
    let mut g = two_player_game();
    // Cast from hand → enters as a 2/1 with two -1/-1 counters.
    let pp = g.add_card_to_hand(0, catalog::patched_plaything());
    ready(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: pp, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Patched Plaything");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(pp).unwrap().counter_count(CounterType::MinusOneMinusOne),
        2,
        "hand-cast enters with two -1/-1 counters",
    );

    // Entering any other way (here: straight onto the battlefield) skips them.
    let direct = g.add_card_to_battlefield(0, catalog::patched_plaything());
    assert_eq!(
        g.battlefield_find(direct).unwrap().counter_count(CounterType::MinusOneMinusOne),
        0,
        "non-hand entry has no -1/-1 counters",
    );
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

/// Defend the Rider can make a 1/1 Pilot token.
#[test]
fn defend_the_rider_makes_pilot() {
    let mut g = two_player_game();
    let dr = g.add_card_to_hand(0, catalog::defend_the_rider());
    ready(&mut g);
    // Mode 2 (token) is chosen by the auto-decider when no controlled
    // permanent exists to target for mode 1.
    g.perform_action(GameAction::CastSpell {
        card_id: dr, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("cast Defend the Rider (token mode)");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Pilot"), 1);
}

/// Full Throttle adds two combat phases after the main phase and untaps
/// attackers at the beginning of each combat this turn.
#[test]
fn full_throttle_adds_two_combats() {
    let mut g = two_player_game();
    let ft = g.add_card_to_hand(0, catalog::full_throttle());
    let atk = g.add_card_to_battlefield(0, catalog::canyon_vaulter());
    ready(&mut g);
    let combats_before = g.additional_post_main_combats;
    g.perform_action(GameAction::CastSpell {
        card_id: ft, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Full Throttle");
    drain_stack(&mut g);
    assert_eq!(g.additional_post_main_combats, combats_before + 2, "two additional combats queued");

    // Mark the creature as a tapped attacker, then enter a fresh Begin
    // Combat: the delayed rider untaps it so it can attack again.
    {
        let c = g.battlefield_find_mut(atk).unwrap();
        c.tapped = true;
        c.attacked_this_turn = true;
    }
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(atk).unwrap().tapped, "attacker untapped for next combat");
}

/// Canyon Vaulter's crew trigger (CR 702.122) gives the crewed Vehicle flying.
#[test]
fn canyon_vaulter_crew_grants_flying() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(0, catalog::debris_beetle()); // Crew 2, no flying
    let cv = g.add_card_to_battlefield(0, catalog::canyon_vaulter()); // power 3
    g.clear_sickness(cv);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![cv] })
        .expect("crew");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(veh).unwrap().keywords.contains(&Keyword::Flying),
        "crewed Vehicle gained flying from Canyon Vaulter's trigger",
    );
}

/// Reckless Velocitaur's crew trigger pumps the crewed Vehicle +2/+0 trample.
#[test]
fn reckless_velocitaur_crew_pumps() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(0, catalog::air_response_unit()); // 3/3 Crew 1
    let rv = g.add_card_to_battlefield(0, catalog::reckless_velocitaur());
    g.clear_sickness(rv);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![rv] })
        .expect("crew");
    drain_stack(&mut g);
    let cp = g.computed_permanent(veh).unwrap();
    assert_eq!(cp.power, 5, "+2/+0");
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// The crew trigger is gated to the controller's main phase: crewing during
/// combat (instant speed) doesn't fire it.
#[test]
fn crew_trigger_silent_outside_main_phase() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(0, catalog::debris_beetle()); // no flying
    let cv = g.add_card_to_battlefield(0, catalog::canyon_vaulter());
    g.clear_sickness(cv);
    g.active_player_idx = 0;
    g.step = TurnStep::BeginCombat; // not a main phase
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![cv] })
        .expect("crew");
    drain_stack(&mut g);
    assert!(
        !g.computed_permanent(veh).unwrap().keywords.contains(&Keyword::Flying),
        "no flying — crew happened outside the main phase",
    );
}

// ── Duskmourn (DSK) tail ─────────────────────────────────────────────────────

/// Emerge from the Cocoon reanimates a creature from the graveyard.
#[test]
fn emerge_from_the_cocoon_reanimates() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let e = g.add_card_to_hand(0, catalog::emerge_from_the_cocoon());
    ready(&mut g);
    let life = g.players[0].life;
    cast_at(&mut g, e, Target::Permanent(dead));
    assert!(g.battlefield_find(dead).is_some(), "Bears reanimated to the battlefield");
    assert_eq!(g.players[0].life, life + 3, "gained 3");
}

/// Enter the Enigma makes a creature unblockable and draws.
#[test]
fn enter_the_enigma_unblockable_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let e = g.add_card_to_hand(0, catalog::enter_the_enigma());
    ready(&mut g);
    let hand = g.players[0].hand.len();
    cast_at(&mut g, e, Target::Permanent(bear));
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
    assert_eq!(g.players[0].hand.len(), hand, "spell left hand (-1) and drew (+1)");
}

/// Exorcise exiles a power-4+ creature.
#[test]
fn exorcise_exiles_big_creature() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let ex = g.add_card_to_hand(0, catalog::exorcise());
    ready(&mut g);
    cast_at(&mut g, ex, Target::Permanent(big));
    assert!(g.battlefield_find(big).is_none(), "4/4 exiled");
}

/// Fear of Lost Teeth pings on death and gains a life.
#[test]
fn fear_of_lost_teeth_dies_pings() {
    let mut g = two_player_game();
    let f = g.add_card_to_battlefield(0, catalog::fear_of_lost_teeth());
    g.battlefield_find_mut(f).unwrap().damage = 1; // lethal vs 1 toughness
    let life = g.players[0].life;
    let foe_life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1");
    assert_eq!(g.players[1].life, foe_life - 1, "pinged the opponent");
}

/// Friendly Teddy draws for each player on death.
#[test]
fn friendly_teddy_dies_each_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(1, catalog::forest());
    let t = g.add_card_to_battlefield(0, catalog::friendly_teddy());
    g.battlefield_find_mut(t).unwrap().damage = 2;
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 + 1);
    assert_eq!(g.players[1].hand.len(), h1 + 1);
}

/// Give In to Violence pumps +2/+2 and grants lifelink.
#[test]
fn give_in_to_violence_pumps_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let giv = g.add_card_to_hand(0, catalog::give_in_to_violence());
    ready(&mut g);
    cast_at(&mut g, giv, Target::Permanent(bear));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords.contains(&Keyword::Lifelink));
}

/// Grasping Longneck gains 2 life when it dies; it has reach.
#[test]
fn grasping_longneck_reach_and_dies_gain() {
    let mut g = two_player_game();
    let gl = g.add_card_to_battlefield(0, catalog::grasping_longneck());
    assert!(g.computed_permanent(gl).unwrap().keywords.contains(&Keyword::Reach));
    g.battlefield_find_mut(gl).unwrap().damage = 2; // lethal vs 2 toughness
    let life = g.players[0].life;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2);
}

/// Horrid Vigor grants deathtouch and indestructible.
#[test]
fn horrid_vigor_grants_deathtouch_indestructible() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hv = g.add_card_to_hand(0, catalog::horrid_vigor());
    ready(&mut g);
    cast_at(&mut g, hv, Target::Permanent(bear));
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Deathtouch));
    assert!(cp.keywords.contains(&Keyword::Indestructible));
}

/// Glimmerburst draws two and makes a Glimmer token.
#[test]
fn glimmerburst_draws_and_makes_glimmer() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let gb = g.add_card_to_hand(0, catalog::glimmerburst());
    ready(&mut g);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: gb, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "-1 cast, +2 drawn");
    assert_eq!(count_named(&g, 0, "Glimmer"), 1);
}

/// Friendly Ghost flies and pumps a creature +2/+4 on ETB.
#[test]
fn friendly_ghost_etb_pumps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let fg = g.add_card_to_battlefield(0, catalog::friendly_ghost());
    assert!(g.computed_permanent(fg).unwrap().keywords.contains(&Keyword::Flying));
    g.fire_self_etb_triggers(fg, 0);
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 6), "+2/+4");
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

/// Sawblade Skinripper grows on its sac ability and, at end step, deals damage
/// equal to the number of permanents sacrificed this turn to any target.
#[test]
fn sawblade_skinripper_sac_payoff() {
    let mut g = two_player_game();
    let saw = g.add_card_to_battlefield(0, catalog::sawblade_skinripper());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(saw);
    ready(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: saw, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate Sawblade sac ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(
        g.battlefield_find(saw).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Sawblade got a +1/+1 counter",
    );
    assert_eq!(g.players[0].permanents_sacrificed_this_turn, 1);
    // End step: 1 permanent sacrificed → 1 damage to the opponent.
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "1 damage from the end-step trigger");
}

/// A minimal 1/1 white creature token (test fixture).
fn soldier_1_1() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    }
}

/// Toby enters and mints a 4/4 Beast that can't attack or block alone.
#[test]
fn toby_makes_lonely_beast() {
    let mut g = two_player_game();
    let toby = g.add_card_to_battlefield(0, catalog::toby_beastie_befriender());
    g.fire_self_etb_triggers(toby, 0);
    drain_stack(&mut g);
    let beast = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.name == "Beast")
        .expect("Beast token created");
    assert_eq!((beast.definition.power, beast.definition.toughness), (4, 4));
    assert!(beast.definition.keywords.contains(&Keyword::CantAttackOrBlockAlone));
}

/// A creature with CantAttackOrBlockAlone can't be the only attacker.
#[test]
fn cant_attack_or_block_alone_blocks_lone_attack() {
    let mut g = two_player_game();
    let beast = g.add_token_to_battlefield(0, &soldier_with_alone());
    g.clear_sickness(beast);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    let res = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: beast,
        target: AttackTarget::Player(1),
    }]));
    assert!(res.is_err(), "lone attack with can't-attack-alone is illegal");
}

/// Toby's anthem grants flying to your creature tokens once you control four.
#[test]
fn toby_anthem_grants_flying_at_four_tokens() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::toby_beastie_befriender());
    let t1 = g.add_token_to_battlefield(0, &soldier_1_1());
    // Three tokens: below the threshold, no flying yet.
    g.add_token_to_battlefield(0, &soldier_1_1());
    g.add_token_to_battlefield(0, &soldier_1_1());
    assert!(!g.computed_permanent(t1).unwrap().keywords.contains(&Keyword::Flying));
    // Fourth token trips the anthem.
    g.add_token_to_battlefield(0, &soldier_1_1());
    assert!(g.computed_permanent(t1).unwrap().keywords.contains(&Keyword::Flying));
}

/// A token with CantAttackOrBlockAlone — a soldier fixture for the combat test.
fn soldier_with_alone() -> TokenDefinition {
    let mut t = soldier_1_1();
    t.keywords = vec![Keyword::CantAttackOrBlockAlone];
    t
}

/// A creature with CantAttackOrBlockAlone can't be the only blocker (CR 509.1c).
#[test]
fn cant_block_alone_rejects_lone_block() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let lone = g.add_token_to_battlefield(1, &soldier_with_alone());
    g.clear_sickness(atk);
    g.clear_sickness(lone);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    let err = g.perform_action(GameAction::DeclareBlockers(vec![(lone, atk)]));
    assert!(err.is_err(), "can't block alone");
}

/// Twitching Doll's mana ability adds a nest counter; sacrificing it makes a
/// Spider per counter (CR 605.1a incidental rider + LKI counter read).
#[test]
fn twitching_doll_nests_then_sacs_for_spiders() {
    let mut g = two_player_game();
    let doll = g.add_card_to_battlefield(0, catalog::twitching_doll());
    g.clear_sickness(doll);
    ready(&mut g);
    // Mana ability twice → two nest counters; pool gains two mana total.
    for _ in 0..2 {
        g.battlefield_find_mut(doll).unwrap().tapped = false;
        g.perform_action(GameAction::ActivateAbility {
            card_id: doll, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("mana + nest counter");
    }
    assert_eq!(g.battlefield_find(doll).unwrap().counter_count(CounterType::Nest), 2);
    // Sacrifice ability: make a Spider per counter (2).
    g.battlefield_find_mut(doll).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: doll, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("sacrifice for Spiders");
    drain_stack(&mut g);
    assert!(g.battlefield_find(doll).is_none(), "Doll sacrificed");
    assert_eq!(count_named(&g, 0, "Spider"), 2, "two Spiders from two counters");
}

/// Fanatic of the Harrowing only draws if you discarded a card this way: with
/// an empty hand you discard nothing and don't draw.
#[test]
fn fanatic_of_the_harrowing_conditional_draw() {
    // You have a card to discard → discard it, then draw.
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let fan = g.add_card_to_battlefield(0, catalog::fanatic_of_the_harrowing());
    g.fire_self_etb_triggers(fan, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "discarded one, drew one");

    // Empty hand → nothing discarded by you → no draw.
    let mut g = two_player_game();
    g.players[0].hand.clear();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let fan = g.add_card_to_battlefield(0, catalog::fanatic_of_the_harrowing());
    g.fire_self_etb_triggers(fan, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 0, "no discard this way → no draw");
}

/// Fear of Isolation costs an extra "return a permanent you control"; cast it
/// and the bounce happens while it enters.
#[test]
fn fear_of_isolation_bounces_a_permanent() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::island());
    let foi = g.add_card_to_hand(0, catalog::fear_of_isolation());
    ready(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: foi, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Fear of Isolation");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "permanent returned to hand");
    assert_eq!(count_named(&g, 0, "Fear of Isolation"), 1, "enchantment creature entered");
}

/// Trapped in the Screen exiles an opponent's permanent on ETB and gives it
/// back when the enchantment leaves (linked exile, CR 603.6e).
#[test]
fn trapped_in_the_screen_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let trap = g.add_card_to_battlefield(0, catalog::trapped_in_the_screen());
    g.fire_self_etb_triggers(trap, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "opponent's creature exiled");
    // Destroy the enchantment → linked exile returns the creature.
    g.remove_to_graveyard_with_triggers(trap);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == foe), "creature returns when Trapped leaves");
}

/// Sheltered by Ghosts enchants your creature (+1/+0, lifelink, ward) and
/// exiles an opponent's nonland permanent on ETB.
#[test]
fn sheltered_by_ghosts_buffs_and_exiles() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::sheltered_by_ghosts());
    ready(&mut g);
    cast_at(&mut g, aura, Target::Permanent(mine));
    drain_stack(&mut g);
    let cp = g.computed_permanent(mine).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2), "+1/+0");
    assert!(cp.keywords.contains(&Keyword::Lifelink), "gains lifelink");
    assert!(g.battlefield_find(foe).is_none(), "opponent permanent exiled on ETB");
}

/// Ragged Playmate makes a small creature unblockable for the turn.
#[test]
fn ragged_playmate_grants_unblockable() {
    let mut g = two_player_game();
    let rp = g.add_card_to_battlefield(0, catalog::ragged_playmate());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 (power 2)
    g.clear_sickness(rp);
    ready(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rp, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    })
    .expect("activate Ragged Playmate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
}

/// Hand That Feeds gets +2/+0 and menace on attack only with delirium active.
#[test]
fn hand_that_feeds_delirium_attack_buff() {
    let mut g = two_player_game();
    let hand = g.add_card_to_battlefield(0, catalog::hand_that_feeds());
    g.clear_sickness(hand);
    // Stock the graveyard with four card types for delirium.
    for c in [
        catalog::grizzly_bears(),       // creature
        catalog::lightning_bolt(),      // instant
        catalog::island(),             // land
        catalog::ornithopter(),        // artifact
    ] {
        let id = g.next_id();
        g.players[0].graveyard.push(crate::card::CardInstance::new(id, c, 0));
    }
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hand, target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(hand).unwrap();
    assert_eq!(cp.power, 4, "+2/+0 from delirium");
    assert!(cp.keywords.contains(&Keyword::Menace), "gains menace");
}

/// Marauding Dreadship incubates 2 on ETB.
#[test]
fn marauding_dreadship_etb_incubates() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::marauding_dreadship());
    g.fire_self_etb_triggers(ship, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Incubator"), 1, "Incubator token created");
}
