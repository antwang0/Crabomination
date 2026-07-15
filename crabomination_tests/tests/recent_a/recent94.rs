//! Functionality tests for the `catalog::sets::decks::recent94` Equipment /
//! Voltron batch.

use crabomination::card::{CardInstance, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::*;

/// Attach `eq` to `creature` directly (test shortcut, bypassing the equip action).
fn attach(g: &mut GameState, eq: crabomination::card::CardId, creature: crabomination::card::CardId) {
    g.battlefield.iter_mut().find(|c| c.id == eq).unwrap().attached_to = Some(creature);
}

/// Akiri's power tracks the number of artifacts you control; toughness stays 3.
#[test]
fn akiri_power_scales_with_artifacts() {
    let mut g = two_player_game();
    let akiri = g.add_card_to_battlefield(0, catalog::akiri_line_slinger());
    let cp = g.computed_permanent(akiri).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 3), "0/3 with no artifacts");
    g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.add_card_to_battlefield(0, catalog::grafted_wargear());
    let cp = g.computed_permanent(akiri).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3), "+1/+0 per artifact, toughness fixed");
}

/// Goreclaw discounts creature spells with power 4 or greater by {2}.
#[test]
fn goreclaw_discounts_big_creatures() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::goreclaw_terror_of_qal_sisma());
    let big = CardInstance::new(g.next_id(), catalog::shivan_dragon(), 0); // 5/5
    let small = CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0); // 2/2
    assert_eq!(cost_reduction_for_spell(&g, 0, &big, None), 2, "power 5 → {{2}} off");
    assert_eq!(cost_reduction_for_spell(&g, 0, &small, None), 0, "power 2 → no discount");
}

/// Reyav grants double strike to an equipped creature when it attacks.
#[test]
fn reyav_grants_double_strike_to_equipped_attacker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::reyav_master_smith());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let axe = g.add_card_to_battlefield(0, catalog::bonesplitter());
    attach(&mut g, axe, bear);
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    // Via perform_action so the YourControl attack trigger dispatches.
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("bear attacks");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "equipped attacker gained double strike");
}

/// Wyleth draws a card for each Aura/Equipment attached to it when it attacks.
#[test]
fn wyleth_draws_per_attached() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let wyleth = g.add_card_to_battlefield(0, catalog::wyleth_soul_of_steel());
    let axe = g.add_card_to_battlefield(0, catalog::bonesplitter());
    let boots = g.add_card_to_battlefield(0, catalog::grafted_wargear());
    attach(&mut g, axe, wyleth);
    attach(&mut g, boots, wyleth);
    g.battlefield.iter_mut().find(|c| c.id == wyleth).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let before = g.players[0].hand.len();
    g.declare_attackers(vec![Attack { attacker: wyleth, target: AttackTarget::Player(1) }])
        .expect("Wyleth attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 2, "drew one per attached (2)");
}

/// Kazuul's Toll Collector attaches a target Equipment you control to itself.
#[test]
fn kazuul_attaches_equipment_to_self() {
    let mut g = two_player_game();
    let kazuul = g.add_card_to_battlefield(0, catalog::kazuuls_toll_collector());
    let axe = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: kazuul, ability_index: 0, target: Some(Target::Permanent(axe)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("attach the Equipment");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(axe).unwrap().attached_to, Some(kazuul), "axe now on Kazuul");
    assert_eq!(g.computed_permanent(kazuul).unwrap().power, 5, "3/2 + 2/0 = 5/2");
}

/// Hammer of Nazahn grants +2/+0 and indestructible to the creature it equips.
#[test]
fn hammer_of_nazahn_equip_bonus() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hammer = g.add_card_to_battlefield(0, catalog::hammer_of_nazahn());
    attach(&mut g, hammer, bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "2/2 + 2/0");
    assert!(cp.keywords.contains(&Keyword::Indestructible), "gains indestructible");
}

/// Argentum Armor is a +6/+6 anvil.
#[test]
fn argentum_armor_equip_bonus() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let armor = g.add_card_to_battlefield(0, catalog::argentum_armor());
    attach(&mut g, armor, bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8), "2/2 + 6/6");
}

/// Vorpal Sword grants deathtouch; Prowler's Helm grants the evasion keyword.
#[test]
fn equipment_grant_keywords() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::vorpal_sword());
    attach(&mut g, sword, bear);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch));

    let cat = g.add_card_to_battlefield(0, catalog::kembas_skyguard());
    let helm = g.add_card_to_battlefield(0, catalog::prowlers_helm());
    attach(&mut g, helm, cat);
    assert!(g.computed_permanent(cat).unwrap().keywords.iter()
        .any(|k| matches!(k, Keyword::CantBeBlockedExceptBy(_))), "gains can't-be-blocked-except-by");
}

/// Sylvia gives Dragons you control double strike.
#[test]
fn sylvia_grants_dragons_double_strike() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon());
    assert!(!g.computed_permanent(dragon).unwrap().keywords.contains(&Keyword::DoubleStrike));
    g.add_card_to_battlefield(0, catalog::sylvia_brightspear());
    assert!(g.computed_permanent(dragon).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "Dragon gained double strike from Sylvia");
}

/// Kwende upgrades first strike to double strike for your creatures.
#[test]
fn kwende_upgrades_first_strike() {
    let mut g = two_player_game();
    // Akiri has first strike printed.
    let akiri = g.add_card_to_battlefield(0, catalog::akiri_line_slinger());
    assert!(!g.computed_permanent(akiri).unwrap().keywords.contains(&Keyword::DoubleStrike));
    g.add_card_to_battlefield(0, catalog::kwende_pride_of_femeref());
    assert!(g.computed_permanent(akiri).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "first-striker upgraded to double strike");
}

/// Kemba's Skyguard gains 2 life on entry.
#[test]
fn kembas_skyguard_gains_life() {
    let mut g = two_player_game();
    let before = g.players[0].life;
    let cat = g.add_card_to_battlefield(0, catalog::kembas_skyguard());
    g.fire_self_etb_triggers(cat, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before + 2, "gained 2 life");
}

/// Niv-Mizzet, Parun can't be countered (partial completion).
#[test]
fn niv_mizzet_parun_cant_be_countered() {
    assert!(catalog::nivmizzet_parun().keywords.contains(&Keyword::CantBeCountered));
}

