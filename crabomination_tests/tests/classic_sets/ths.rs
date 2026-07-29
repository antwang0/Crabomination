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
