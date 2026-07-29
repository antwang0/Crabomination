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


// ── Batch 2 ─────────────────────────────────────────────────────────────────

/// Stat / keyword lines for the second THS batch.
#[test]
fn ths_batch2_stat_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::epharas_warden, 1, 2, &[]),
        (catalog::fleshmad_steed, 2, 2, &[]),
        (catalog::blood_toll_harpy, 2, 1, &[Keyword::Flying]),
        (catalog::benthic_giant, 4, 5, &[Keyword::Hexproof]),
        (catalog::crackling_triton, 2, 3, &[]),
        (catalog::lagonna_band_elder, 3, 2, &[]),
        (catalog::minotaur_skullcleaver, 2, 2, &[Keyword::Haste]),
        (catalog::decorated_griffin, 2, 3, &[Keyword::Flying]),
        (catalog::coastline_chimera, 1, 5, &[Keyword::Flying]),
        (catalog::breaching_hippocamp, 3, 2, &[Keyword::Flash]),
        (catalog::agent_of_horizons, 3, 2, &[]),
    ];
    for (f, p, t, kws) in table {
        let d = f();
        assert_eq!((d.power, d.toughness), (*p, *t), "{}", d.name);
        for kw in *kws {
            assert!(d.keywords.contains(kw), "{} lacks {:?}", d.name, kw);
        }
    }
}

/// Coastline Chimera buys an extra block for the turn (CR 509.1b).
#[test]
fn coastline_chimera_buys_an_extra_block() {
    let mut g = main_phase();
    let chimera = g.add_card_to_battlefield(1, catalog::coastline_chimera());
    let a1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let a2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a1);
    g.clear_sickness(a2);
    g.attacking = vec![
        Attack { attacker: a1, target: AttackTarget::Player(1) },
        Attack { attacker: a2, target: AttackTarget::Player(1) },
    ];
    g.step = TurnStep::DeclareBlockers;
    assert!(g.declare_blockers(vec![(chimera, a1), (chimera, a2)]).is_err(), "one block by default");
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::White, 1);
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: chimera,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("buy an extra block");
    drain_stack(&mut g);
    g.declare_blockers(vec![(chimera, a1), (chimera, a2)]).expect("now legal");
    assert_eq!(g.attackers_blocked_by(chimera).len(), 2);
}

/// Blood-Toll Harpy drains both players; Lagonna-Band Elder only pays off with
/// an enchantment out.
#[test]
fn etb_riders_fire_conditionally() {
    let mut g = main_phase();
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.move_card_to_battlefield_for_test(0, catalog::blood_toll_harpy());
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life), (l0 - 1, l1 - 1));

    // No enchantment: no life.
    let life = g.players[0].life;
    g.move_card_to_battlefield_for_test(0, catalog::lagonna_band_elder());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life);
    // With one: +3.
    g.add_card_to_battlefield(0, catalog::messengers_speed());
    g.move_card_to_battlefield_for_test(0, catalog::lagonna_band_elder());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3);
}

/// Dark Betrayal only answers black creatures; Glare of Heresy only white
/// permanents.
#[test]
fn color_restricted_removal_checks_the_color() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    let zombie = g.add_card_to_battlefield(1, catalog::gravedigger()); // black
    let betrayal = g.add_card_to_hand(0, catalog::dark_betrayal());
    g.players[0].mana_pool.add(Color::Black, 1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: betrayal, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "a green creature isn't a legal target");
    cast_at(&mut g, betrayal, Some(Target::Permanent(zombie)));
    assert!(g.battlefield_find(zombie).is_none());
}

/// Hunt the Hunter pumps yours, then fights theirs.
#[test]
fn hunt_the_hunter_pumps_then_fights() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 green
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hunt = g.add_card_to_hand(0, catalog::hunt_the_hunter());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: hunt,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(theirs).is_none(), "4/4 kills the 2/2");
    assert!(g.battlefield_find(mine).is_some(), "the 4/4 survives 2 damage");
}

/// March of the Returned rebuys two creature cards.
#[test]
fn march_of_the_returned_rebuys_two() {
    let mut g = main_phase();
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::hill_giant());
    let march = g.add_card_to_hand(0, catalog::march_of_the_returned());
    g.players[0].mana_pool.add(Color::Black, 4);
    g.perform_action(GameAction::CastSpell {
        card_id: march,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.iter().filter(|c| c.id == a || c.id == b).count(), 2);
}

/// Defend the Hearth fogs only the players — creatures still trade.
#[test]
fn defend_the_hearth_fogs_players_only() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let fog = g.add_card_to_hand(0, catalog::defend_the_hearth());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast_at(&mut g, fog, None);
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(1) }];
    g.step = TurnStep::CombatDamage;
    let life = g.players[1].life;
    g.resolve_combat().expect("damage");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life, "no combat damage to players");
}

/// Crackling Triton's sac ability and Flamecast Wheel's both fire off their
/// own removal.
#[test]
fn sacrifice_abilities_deal_their_damage() {
    let mut g = main_phase();
    let triton = g.add_card_to_battlefield(0, catalog::crackling_triton());
    g.players[0].mana_pool.add(Color::Red, 3);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: triton, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    })
    .expect("sac for 2");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
    assert!(g.battlefield_find(triton).is_none(), "sacrificed as a cost");

    let wheel = g.add_card_to_battlefield(0, catalog::flamecast_wheel());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wheel, ability_index: 0, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], x_value: None,
    })
    .expect("wheel");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "3 damage kills the 2/2");
}

// ── batch 3 ─────────────────────────────────────────────────────────────────

/// Stat / keyword lines and bestow wiring for the batch-3 bodies.
#[test]
fn ths_batch3_stat_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::silent_artisan, 3, 5, &[]),
        (catalog::vaporkin, 2, 1, &[Keyword::Flying, Keyword::CanBlockOnlyFlying]),
        (catalog::insatiable_harpy, 2, 2, &[Keyword::Flying, Keyword::Lifelink]),
        (catalog::satyr_rambler, 2, 1, &[Keyword::Trample]),
        (catalog::triton_shorethief, 1, 2, &[]),
        (catalog::fabled_hero, 2, 2, &[Keyword::DoubleStrike]),
        (catalog::soldier_of_the_pantheon, 2, 1, &[Keyword::ProtectionFromMulticolored]),
        (catalog::horizon_chimera, 3, 2, &[Keyword::Flash, Keyword::Flying, Keyword::Trample]),
        (catalog::anax_and_cymede, 3, 2, &[Keyword::FirstStrike, Keyword::Vigilance]),
        (catalog::returned_phalanx, 3, 3, &[Keyword::Defender]),
    ];
    for (f, p, t, kws) in table {
        let d = f();
        assert_eq!((d.power, d.toughness), (*p, *t), "{}", d.name);
        for kw in *kws {
            assert!(d.keywords.contains(kw), "{} lacks {:?}", d.name, kw);
        }
    }
    for f in [
        catalog::celestial_archon as fn() -> crabomination::card::CardDefinition,
        catalog::thassas_emissary,
        catalog::erebos_s_emissary,
        catalog::nighthowler,
        catalog::purphoross_emissary,
        catalog::spearpoint_oread,
        catalog::boon_satyr,
    ] {
        let d = f();
        assert!(d.bestow.is_some() && d.equipped_bonus.is_some(), "{}", d.name);
    }
}

/// Heroic payoffs across the batch: counters, life, damage and a team pump.
#[test]
fn ths_batch3_heroic_payoffs() {
    // Fabled Hero: one +1/+1 counter; Centaur Battlemaster: three.
    for (f, want) in [
        (catalog::fabled_hero as fn() -> crabomination::card::CardDefinition, 1),
        (catalog::centaur_battlemaster, 3),
        (catalog::staunch_hearted_warrior, 2),
    ] {
        let mut g = main_phase();
        let hero = g.add_card_to_battlefield(0, f());
        let pump = g.add_card_to_hand(0, catalog::battlewise_valor());
        g.players[0].mana_pool.add(Color::White, 2);
        cast_at(&mut g, pump, Some(Target::Permanent(hero)));
        assert_eq!(
            g.battlefield_find(hero).unwrap().counter_count(CounterType::PlusOnePlusOne),
            want
        );
    }
    // Setessan Battle Priest gains 2 life.
    let mut g = main_phase();
    let priest = g.add_card_to_battlefield(0, catalog::setessan_battle_priest());
    let pump = g.add_card_to_hand(0, catalog::battlewise_valor());
    g.players[0].mana_pool.add(Color::White, 2);
    let life = g.players[0].life;
    cast_at(&mut g, pump, Some(Target::Permanent(priest)));
    assert_eq!(g.players[0].life, life + 2);
}

/// Tormented Hero enters tapped and its heroic drains for 1.
#[test]
fn tormented_hero_enters_tapped_and_drains() {
    let mut g = main_phase();
    let hero = g.add_card_to_hand(0, catalog::tormented_hero());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast_at(&mut g, hero, None);
    assert!(g.battlefield_find(hero).unwrap().tapped, "enters tapped");
    let pump = g.add_card_to_hand(0, catalog::battlewise_valor());
    g.players[0].mana_pool.add(Color::White, 2);
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    cast_at(&mut g, pump, Some(Target::Permanent(hero)));
    assert_eq!((g.players[0].life, g.players[1].life), (mine + 1, theirs - 1));
}

/// Agent of the Fates' heroic edicts each opponent.
#[test]
fn agent_of_the_fates_heroic_edicts() {
    let mut g = main_phase();
    let agent = g.add_card_to_battlefield(0, catalog::agent_of_the_fates());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pump = g.add_card_to_hand(0, catalog::battlewise_valor());
    g.players[0].mana_pool.add(Color::White, 2);
    cast_at(&mut g, pump, Some(Target::Permanent(agent)));
    assert!(g.battlefield_find(victim).is_none(), "opponent sacrificed their only creature");
}

/// Devotion scales the batch's ETBs: Evangel of Heliod's token count and
/// Reverent Hunter's counters read the controller's coloured pips.
#[test]
fn ths_batch3_devotion_etbs() {
    let mut g = main_phase();
    // Two white pips already on board, plus Evangel's own {W}{W} = 4.
    g.add_card_to_battlefield(0, catalog::wingsteed_rider());
    let evangel = g.add_card_to_hand(0, catalog::evangel_of_heliod());
    g.players[0].mana_pool.add(Color::White, 6);
    cast_at(&mut g, evangel, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Soldier").count(), 4);

    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::satyr_rambler()); // no green pips
    let hunter = g.add_card_to_hand(0, catalog::reverent_hunter());
    g.players[0].mana_pool.add(Color::Green, 3);
    cast_at(&mut g, hunter, None);
    assert_eq!(
        g.battlefield_find(hunter).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Reverent Hunter's own green pip"
    );
}

/// Karametra's Acolyte taps for green equal to devotion.
#[test]
fn karametras_acolyte_taps_for_devotion() {
    let mut g = main_phase();
    let acolyte = g.add_card_to_battlefield(0, catalog::karametras_acolyte());
    g.clear_sickness(acolyte);
    g.add_card_to_battlefield(0, catalog::boon_satyr()); // {1}{G}{G} = 2 more pips
    g.perform_action(GameAction::ActivateAbility {
        card_id: acolyte,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3);
}

/// Monstrosity riders: Keepsake Gorgon kills a non-Gorgon, and Hundred-Handed
/// One picks up reach + the extra-block allowance (CR 509.1b).
#[test]
fn ths_batch3_monstrosity_riders() {
    let mut g = main_phase();
    let gorgon = g.add_card_to_battlefield(0, catalog::keepsake_gorgon());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gorgon,
        ability_index: 0,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("monstrosity");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none());

    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(0, catalog::hundred_handed_one());
    assert!(!g.computed_permanent(giant).unwrap().keywords.contains(&Keyword::Reach));
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: giant,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("monstrosity");
    drain_stack(&mut g);
    let kws = g.computed_permanent(giant).unwrap().keywords;
    assert!(kws.contains(&Keyword::Reach) && kws.contains(&Keyword::CanBlockAdditional(99)));
}

/// Sealock Monster's monstrosity turns a land into an Island, which also frees
/// its own can't-attack restriction.
#[test]
fn sealock_monster_makes_an_island() {
    let mut g = main_phase();
    let monster = g.add_card_to_battlefield(0, catalog::sealock_monster());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: monster,
        ability_index: 0,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("monstrosity");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(land)
            .unwrap()
            .subtypes
            .land_types
            .contains(&crabomination::card::LandType::Island)
    );
}

/// Erebos's Emissary pumps itself as a creature; as a bestowed Aura the pump
/// goes to the enchanted creature instead.
#[test]
fn erebos_s_emissary_pump_follows_the_aura() {
    let mut g = main_phase();
    let snake = g.add_card_to_battlefield(0, catalog::erebos_s_emissary());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: snake,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("discard a creature card");
    drain_stack(&mut g);
    let cp = g.computed_permanent(snake).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "+2/+2 on itself");

    // Bestowed: the host gets the pump.
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let snake = g.add_card_to_hand(0, catalog::erebos_s_emissary());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastBestow {
        card_id: snake,
        target: Some(Target::Permanent(host)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bestow");
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: snake,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("discard a creature card");
    drain_stack(&mut g);
    let cp = g.computed_permanent(host).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7), "2/2 + bestow 3/3 + the 2/2 pump");
}

/// Nighthowler's P/T is the creature-card count in all graveyards, on both
/// sides of the bestow switch.
#[test]
fn nighthowler_counts_all_graveyards() {
    let mut g = main_phase();
    let howler = g.add_card_to_battlefield(0, catalog::nighthowler());
    for seat in [0, 1] {
        let id = g.add_card_to_battlefield(seat, catalog::grizzly_bears());
        g.remove_from_battlefield_to_graveyard_raw(id);
    }
    let cp = g.computed_permanent(howler).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// Firedrinker Satyr mirrors damage onto its controller.
#[test]
fn firedrinker_satyr_mirrors_damage() {
    let mut g = main_phase();
    let satyr = g.add_card_to_battlefield(0, catalog::firedrinker_satyr());
    let life = g.players[0].life;
    let mut evs = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(satyr),
        1,
        None,
        &mut evs,
    );
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1);
}

/// Flamespeaker Adept grows whenever its controller scries.
#[test]
fn flamespeaker_adept_grows_on_scry() {
    let mut g = main_phase();
    let adept = g.add_card_to_battlefield(0, catalog::flamespeaker_adept());
    g.add_card_to_library(0, catalog::mountain());
    let jolt = g.add_card_to_hand(0, catalog::spark_jolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, jolt, Some(Target::Player(1)));
    let cp = g.computed_permanent(adept).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3));
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
    assert_eq!(g.players[1].life, 19, "the Jolt still burned");
}

/// Akroan Hoplite counts the whole attacking team.
#[test]
fn akroan_hoplite_scales_with_the_team() {
    let mut g = main_phase();
    let hoplite = g.add_card_to_battlefield(0, catalog::akroan_hoplite());
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(hoplite);
    g.clear_sickness(friend);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![
        Attack { attacker: hoplite, target: AttackTarget::Player(1) },
        Attack { attacker: friend, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(hoplite).unwrap().power, 3, "1 + two attackers");
}

/// Kragma Warcaller hastes Minotaurs and pumps the attacker.
#[test]
fn kragma_warcaller_hastes_and_pumps_minotaurs() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::kragma_warcaller());
    let minotaur = g.add_card_to_battlefield(0, catalog::minotaur_skullcleaver());
    assert!(g.computed_permanent(minotaur).unwrap().keywords.contains(&Keyword::Haste));
    g.step = TurnStep::DeclareAttackers;
    let evs = g
        .declare_attackers(vec![Attack { attacker: minotaur, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let base = catalog::minotaur_skullcleaver().power;
    assert_eq!(g.computed_permanent(minotaur).unwrap().power, base + 2);
}

/// Rage of Purphoros burns through a regeneration shield (CR 701.15g).
#[test]
fn rage_of_purphoros_blanks_regeneration() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().regeneration_shields = 1;
    let rage = g.add_card_to_hand(0, catalog::rage_of_purphoros());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, rage, Some(Target::Permanent(victim)));
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "the shield doesn't save it");
}

/// Peak Eruption destroys a Mountain and burns its controller.
#[test]
fn peak_eruption_hits_mountains_only() {
    let mut g = main_phase();
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    let peak = g.add_card_to_hand(0, catalog::peak_eruption());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, peak, Some(Target::Permanent(mountain)));
    g.check_state_based_actions();
    assert!(g.battlefield_find(mountain).is_none());
    assert_eq!(g.players[1].life, 17);
}

/// Stoneshock Giant's monstrosity blanks the opponent's ground blockers.
#[test]
fn stoneshock_giant_stops_ground_blockers() {
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(0, catalog::stoneshock_giant());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flyer = g.add_card_to_battlefield(1, catalog::insatiable_harpy());
    g.players[0].mana_pool.add_colorless(6);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: giant,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("monstrosity");
    drain_stack(&mut g);
    assert!(g.computed_permanent(ground).unwrap().keywords.contains(&Keyword::CantBlock));
    assert!(!g.computed_permanent(flyer).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Titan of Eternal Fire arms every Human you control.
#[test]
fn titan_of_eternal_fire_arms_humans() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::titan_of_eternal_fire());
    let human = g.add_card_to_battlefield(0, catalog::fabled_hero());
    g.clear_sickness(human);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: human,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("granted pinger");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
}

/// Returned Phalanx buys its way past defender for the turn.
#[test]
fn returned_phalanx_can_buy_an_attack() {
    let mut g = main_phase();
    let zombie = g.add_card_to_battlefield(0, catalog::returned_phalanx());
    g.clear_sickness(zombie);
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.declare_attackers(vec![Attack { attacker: zombie, target: AttackTarget::Player(1) }])
            .is_err(),
        "defender blocks the attack"
    );
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: zombie,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("pay {1}{U}");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: zombie, target: AttackTarget::Player(1) }])
        .expect("attacks now");
}

/// Viper's Kiss shrinks the host and locks its activated abilities.
#[test]
fn vipers_kiss_locks_abilities() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(1, catalog::scholar_of_athreos());
    g.clear_sickness(host);
    let kiss = g.add_card_to_hand(0, catalog::vipers_kiss());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast_at(&mut g, kiss, Some(Target::Permanent(host)));
    let cp = g.computed_permanent(host).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 3));
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: host,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .is_err(),
        "activated abilities are locked"
    );
}

/// Scourgemark cantrips and pumps.
#[test]
fn scourgemark_draws_and_pumps() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::scourgemark());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand = g.players[0].hand.len();
    cast_at(&mut g, aura, Some(Target::Permanent(host)));
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    assert_eq!(g.computed_permanent(host).unwrap().power, 3);
}

/// Witches' Eye grants its scry ability to the equipped creature.
#[test]
fn witches_eye_grants_a_scry_ability() {
    let mut g = main_phase();
    let eye = g.add_card_to_battlefield(0, catalog::witches_eye());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.add_card_to_library(0, catalog::mountain());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: eye, target: bear }).expect("equip");
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("granted scry");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Nemesis of Mortals is cheaper for each creature card in your graveyard.
#[test]
fn nemesis_of_mortals_costs_less_per_graveyard_creature() {
    let mut g = main_phase();
    for _ in 0..3 {
        let id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.remove_from_battlefield_to_graveyard_raw(id);
    }
    let nemesis = g.add_card_to_hand(0, catalog::nemesis_of_mortals());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 2);
    cast_at(&mut g, nemesis, None);
    assert!(g.battlefield_find(nemesis).is_some(), "{{4}}{{G}}{{G}} minus three");
}

/// Disciple of Phenax only sees devotion-many cards.
#[test]
fn disciple_of_phenax_reveals_devotion_many() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let disciple = g.add_card_to_hand(0, catalog::disciple_of_phenax());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: disciple,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 3, "one of the revealed cards is discarded");
}

/// Mogis's Marauder's target cap is devotion to black.
#[test]
fn mogiss_marauder_caps_targets_at_devotion() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let marauder = g.add_card_to_hand(0, catalog::mogiss_marauder());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: marauder,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    // Devotion is 1 ({B} on the Marauder itself), so only the first slot lands.
    assert!(g.computed_permanent(a).unwrap().keywords.contains(&Keyword::Haste));
    assert!(!g.computed_permanent(b).unwrap().keywords.contains(&Keyword::Haste));
}

/// Spells that ride a scry: Artisan's Sorrow and Sea God's Revenge.
#[test]
fn ths_batch3_scry_spells() {
    let mut g = main_phase();
    let enchantment = g.add_card_to_battlefield(1, catalog::scourgemark());
    let sorrow = g.add_card_to_hand(0, catalog::artisans_sorrow());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_at(&mut g, sorrow, Some(Target::Permanent(enchantment)));
    g.check_state_based_actions();
    assert!(g.battlefield_find(enchantment).is_none());

    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let revenge = g.add_card_to_hand(0, catalog::sea_gods_revenge());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: revenge,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 2);
}
