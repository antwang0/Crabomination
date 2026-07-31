//! Urza's Saga (USG) gap closure, first wave.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

/// Every USG factory is registered under its printed name.
#[test]
fn usg_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::albino_troll as fn() -> crabomination::card::CardDefinition,
        catalog::humble,
        catalog::rewind,
        catalog::raze,
        catalog::sanctum_guardian,
        catalog::goblin_offensive,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// The echo cycle all carry their printed echo cost.
#[test]
fn usg_echo_bodies_carry_echo() {
    for (f, cmc) in [
        (catalog::acridian as fn() -> crabomination::card::CardDefinition, 2),
        (catalog::albino_troll, 2),
        (catalog::cradle_guard, 3),
        (catalog::goblin_patrol, 1),
        (catalog::goblin_war_buggy, 2),
        (catalog::pouncing_jaguar, 1),
    ] {
        let def = f();
        assert!(
            def.keywords.iter().any(|k| matches!(k, Keyword::Echo(c) if c.cmc() == cmc)),
            "{} is missing Echo {{{cmc}}}",
            def.name
        );
    }
}

/// The cycling cards all carry Cycling {2}.
#[test]
fn usg_cycling_cards_carry_cycling_two() {
    for f in [
        catalog::disciple_of_grace as fn() -> crabomination::card::CardDefinition,
        catalog::disciple_of_law,
        catalog::pendrell_drake,
        catalog::sandbar_merfolk,
        catalog::sandbar_serpent,
        catalog::clear,
        catalog::expunge,
        catalog::lay_waste,
        catalog::scrap,
        catalog::rescind,
    ] {
        let def = f();
        assert!(
            def.keywords.iter().any(|k| matches!(k, Keyword::Cycling(c) if c.cmc() == 2)),
            "{} is missing Cycling {{2}}",
            def.name
        );
    }
}

/// Fire Ants sweeps the ground and leaves fliers (and itself) alone.
#[test]
fn fire_ants_sweeps_only_the_ground() {
    let mut g = two_player_game();
    let ants = g.add_card_to_battlefield(0, catalog::fire_ants());
    g.battlefield_find_mut(ants).unwrap().summoning_sick = false;
    let ground = g.add_card_to_battlefield(1, catalog::serra_zealot()); // 1/1
    let flier = g.add_card_to_battlefield(1, catalog::pegasus_charger()); // 2/1 flying
    activate(&mut g, ants, 0, None);
    assert!(g.battlefield_find(ground).is_none());
    assert!(g.battlefield_find(flier).is_some());
    assert_eq!(g.battlefield_find(ants).unwrap().damage, 0, "it spares itself");
}

/// Humble blanks a creature's abilities and shrinks it to 0/1.
#[test]
fn humble_leaves_a_vanilla_zero_one() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::pegasus_charger());
    let spell = g.add_card_to_hand(0, catalog::humble());
    mana(&mut g, 0);
    cast(&mut g, spell, Some(Target::Permanent(victim)));
    let cp = g.computed_permanent(victim).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 1));
    assert!(!cp.keywords.contains(&Keyword::Flying));
}

/// Rewind counters and refunds four lands.
#[test]
fn rewind_counters_and_untaps_four_lands() {
    let mut g = two_player_game();
    for _ in 0..5 {
        let land = g.add_card_to_battlefield(0, catalog::island());
        g.battlefield_find_mut(land).unwrap().tapped = true;
    }
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let rewind = g.add_card_to_hand(0, catalog::rewind());
    mana(&mut g, 0);
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Bolt");
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    cast(&mut g, rewind, Some(Target::Permanent(bolt)));
    assert_eq!(g.players[0].life, life, "the Bolt was countered");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && !c.tapped).count(),
        4,
        "four lands came back"
    );
}

/// Raze eats a land of yours on the way to one of theirs.
#[test]
fn raze_costs_a_land_of_your_own() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::mountain());
    let theirs = g.add_card_to_battlefield(1, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::raze());
    mana(&mut g, 0);
    cast(&mut g, spell, Some(Target::Permanent(theirs)));
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(mine).is_none(), "the additional cost was paid");
}

/// Goblin Offensive mints X Goblins.
#[test]
fn goblin_offensive_mints_x_goblins() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::goblin_offensive());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("cast Goblin Offensive");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count(), 3);
}

/// Priest of Gix pays for itself on arrival.
#[test]
fn priest_of_gix_refunds_three_black() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::priest_of_gix());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

/// Sanctum Guardian trades itself for one whole damage event.
#[test]
fn sanctum_guardian_blanks_a_damage_event() {
    let mut g = two_player_game();
    let guardian = g.add_card_to_battlefield(0, catalog::sanctum_guardian());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, guardian, 0, None);
    let life = g.players[0].life;
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        4,
        Some(attacker),
        &mut ev,
    );
    assert_eq!(g.players[0].life, life, "the whole event was prevented");
}

/// Hibernation sends every green permanent home, whoever owns it.
#[test]
fn hibernation_bounces_all_green_permanents() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::pouncing_jaguar());
    let theirs = g.add_card_to_battlefield(1, catalog::argothian_swine());
    let neutral = g.add_card_to_battlefield(1, catalog::crazed_skirge());
    let spell = g.add_card_to_hand(0, catalog::hibernation());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    assert!(g.battlefield_find(mine).is_none());
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(neutral).is_some(), "black stays put");
    assert_eq!(g.players[1].hand.len(), 1, "it went to its own owner's hand");
}

/// Mobile Fort charges once a turn.
#[test]
fn mobile_fort_can_charge_once_a_turn() {
    let mut g = two_player_game();
    let fort = g.add_card_to_battlefield(0, catalog::mobile_fort());
    mana(&mut g, 0);
    activate(&mut g, fort, 0, None);
    let cp = g.computed_permanent(fort).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 5));
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: fort,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "once each turn"
    );
}

/// Phyrexian Ghoul eats a friend for +2/+2.
#[test]
fn phyrexian_ghoul_eats_a_friend() {
    let mut g = two_player_game();
    let ghoul = g.add_card_to_battlefield(0, catalog::phyrexian_ghoul());
    let fodder = g.add_card_to_battlefield(0, catalog::serra_zealot());
    activate(&mut g, ghoul, 0, None);
    assert!(g.battlefield_find(fodder).is_none());
    let cp = g.computed_permanent(ghoul).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}
