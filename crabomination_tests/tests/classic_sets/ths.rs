//! Functionality tests for Theros (THS) — `catalog::sets::ths`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// Main-phase game with seat 0 holding priority.
fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// Cast `card` (already in seat 0's hand) at `target` with `mana` white.
fn cast_at(g: &mut GameState, card: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: card,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// Stat / keyword lines for the THS heroic + bestow batch.
#[test]
fn ths_heroes_stat_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::akroan_crusader, 1, 1, &[]),
        (catalog::battlewise_hoplite, 2, 2, &[]),
        (catalog::favored_hoplite, 1, 2, &[]),
        (catalog::leafcrown_dryad, 2, 2, &[Keyword::Reach]),
        (catalog::nimbus_naiad, 2, 2, &[Keyword::Flying]),
        (catalog::observant_alseid, 2, 2, &[Keyword::Vigilance]),
        (catalog::cavern_lampad, 2, 2, &[Keyword::Intimidate]),
        (catalog::nyleas_emissary, 3, 3, &[Keyword::Trample]),
        (catalog::heliods_emissary, 3, 3, &[]),
    ];
    for (f, p, t, kws) in table {
        let d = f();
        assert_eq!((d.power, d.toughness), (*p, *t), "{}", d.name);
        for kw in *kws {
            assert!(d.keywords.contains(kw), "{} lacks {:?}", d.name, kw);
        }
    }
    // Every bestow card carries a bestow cost and an Aura bonus.
    for f in [
        catalog::leafcrown_dryad as fn() -> crabomination::card::CardDefinition,
        catalog::nimbus_naiad,
        catalog::observant_alseid,
        catalog::cavern_lampad,
        catalog::nyleas_emissary,
        catalog::heliods_emissary,
    ] {
        let d = f();
        assert!(d.bestow.is_some() && d.equipped_bonus.is_some(), "{}", d.name);
    }
}

/// Heroic fires off a spell that targets the creature — Akroan Crusader makes
/// a hasty Soldier.
#[test]
fn akroan_crusader_heroic_makes_a_soldier() {
    let mut g = main_phase();
    let crusader = g.add_card_to_battlefield(0, catalog::akroan_crusader());
    let pump = g.add_card_to_hand(0, catalog::battlewise_valor());
    g.players[0].mana_pool.add(Color::White, 2);
    cast_at(&mut g, pump, Some(Target::Permanent(crusader)));
    let soldier = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Soldier")
        .expect("Soldier token");
    assert!(soldier.definition.keywords.contains(&Keyword::Haste));
    // The pump itself resolved too.
    let cp = g.computed_permanent(crusader).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+2/+2 from Battlewise Valor");
}

/// Battlewise Hoplite's heroic adds a counter and scries.
#[test]
fn battlewise_hoplite_heroic_counter() {
    let mut g = main_phase();
    let hoplite = g.add_card_to_battlefield(0, catalog::battlewise_hoplite());
    let pump = g.add_card_to_hand(0, catalog::battlewise_valor());
    g.players[0].mana_pool.add(Color::White, 2);
    cast_at(&mut g, pump, Some(Target::Permanent(hoplite)));
    assert_eq!(
        g.battlefield_find(hoplite).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Favored Hoplite's heroic shields it for the turn.
#[test]
fn favored_hoplite_heroic_prevents_damage() {
    let mut g = main_phase();
    let hoplite = g.add_card_to_battlefield(0, catalog::favored_hoplite());
    let pump = g.add_card_to_hand(0, catalog::battlewise_valor());
    g.players[0].mana_pool.add(Color::White, 2);
    cast_at(&mut g, pump, Some(Target::Permanent(hoplite)));
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(hoplite),
        5,
        None,
        &mut evs,
    );
    assert_eq!(g.battlefield_find(hoplite).unwrap().damage, 0, "all damage prevented");
}

/// Dauntless Onslaught pumps two creatures; Coordinated Assault grants first
/// strike to two.
#[test]
fn two_target_pump_spells_hit_both() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let onslaught = g.add_card_to_hand(0, catalog::dauntless_onslaught());
    g.players[0].mana_pool.add(Color::White, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: onslaught,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    for id in [a, b] {
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4));
    }

    let assault = g.add_card_to_hand(0, catalog::coordinated_assault());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: assault,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    for id in [a, b] {
        assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::FirstStrike));
    }
}

/// Chosen by Heliod draws on ETB and gives +0/+2; Messenger's Speed grants
/// trample and haste.
#[test]
fn theros_auras_attach_and_grant() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let chosen = g.add_card_to_hand(0, catalog::chosen_by_heliod());
    g.players[0].mana_pool.add(Color::White, 2);
    let hand = g.players[0].hand.len();
    cast_at(&mut g, chosen, Some(Target::Permanent(bear)));
    // -1 for the Aura leaving hand, +1 for the ETB draw.
    assert_eq!(g.players[0].hand.len(), hand);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4));

    let speed = g.add_card_to_hand(0, catalog::messengers_speed());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, speed, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Trample) && cp.keywords.contains(&Keyword::Haste));
}

/// Fate Foretold replaces itself and draws again when the host dies.
#[test]
fn fate_foretold_draws_on_host_death() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let aura = g.add_card_to_hand(0, catalog::fate_foretold());
    g.players[0].mana_pool.add(Color::Blue, 2);
    cast_at(&mut g, aura, Some(Target::Permanent(bear)));
    let hand = g.players[0].hand.len();
    g.battlefield_find_mut(bear).unwrap().damage = 99;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the host's controller drew");
}

/// Dragon Mantle grants the host a firebreathing ability.
#[test]
fn dragon_mantle_grants_firebreathing() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let mantle = g.add_card_to_hand(0, catalog::dragon_mantle());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, mantle, Some(Target::Permanent(bear)));
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("granted firebreathing");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// Bestowing Nylea's Emissary makes it an Aura granting +3/+3 and trample.
#[test]
fn nyleas_emissary_bestows_as_an_aura() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cat = g.add_card_to_hand(0, catalog::nyleas_emissary());
    g.players[0].mana_pool.add(Color::Green, 6);
    g.perform_action(GameAction::CastBestow {
        card_id: cat,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bestow");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(cp.keywords.contains(&Keyword::Trample));
    assert!(
        !g.computed_permanent(cat).unwrap().card_types.contains(&crabomination::card::CardType::Creature),
        "a bestowed permanent isn't a creature",
    );
}

/// The Ordeal cycle: three attacks add three counters, then the Aura
/// sacrifices itself and pays off.
#[test]
fn ordeal_of_heliod_pays_off_on_the_third_attack() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let ordeal = g.add_card_to_hand(0, catalog::ordeal_of_heliod());
    g.players[0].mana_pool.add(Color::White, 2);
    cast_at(&mut g, ordeal, Some(Target::Permanent(bear)));
    let life = g.players[0].life;
    for _ in 0..3 {
        g.attacking.clear();
        g.battlefield_find_mut(bear).unwrap().tapped = false;
        g.step = TurnStep::DeclareAttackers;
        let evs = g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]).expect("attack");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3
    );
    assert!(g.battlefield_find(ordeal).is_none(), "the Aura sacrificed itself");
    assert_eq!(g.players[0].life, life + 10);
}

/// Ordeal of Purphoros burns when it goes; Ordeal of Thassa draws two.
#[test]
fn ordeal_payoffs_burn_and_draw() {
    for (factory, check) in [
        (
            catalog::ordeal_of_purphoros as fn() -> crabomination::card::CardDefinition,
            0usize,
        ),
        (catalog::ordeal_of_thassa, 1),
    ] {
        let mut g = main_phase();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::forest());
        }
        let ordeal = g.add_card_to_hand(0, factory());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add(Color::Red, 1);
        cast_at(&mut g, ordeal, Some(Target::Permanent(bear)));
        let (life, hand) = (g.players[1].life, g.players[0].hand.len());
        for _ in 0..3 {
            g.attacking.clear();
            g.battlefield_find_mut(bear).unwrap().tapped = false;
            g.step = TurnStep::DeclareAttackers;
            let evs = g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]).expect("attack");
            g.dispatch_triggers_for_events(&evs);
            drain_stack(&mut g);
        }
        if check == 0 {
            assert_eq!(g.players[1].life, life - 3, "3 damage");
        } else {
            assert_eq!(g.players[0].hand.len(), hand + 2, "drew two");
        }
    }
}

/// Heliod's Emissary taps a blocker whether it attacks itself or its host does.
#[test]
fn heliods_emissary_taps_on_attack() {
    let mut g = main_phase();
    let elk = g.add_card_to_battlefield(0, catalog::heliods_emissary());
    g.clear_sickness(elk);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    let evs = g.declare_attackers(vec![Attack { attacker: elk, target: AttackTarget::Player(1) }]).expect("attack");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped);
}

