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

// ── Vengevine ───────────────────────────────────────────────────────────────

#[test]
fn vengevine_is_4_3_haste_elemental() {
    use crabomination::card::Keyword;
    let card = catalog::vengevine();
    assert_eq!(card.name, "Vengevine");
    assert_eq!(card.power, 4);
    assert_eq!(card.toughness, 3);
    assert!(card.keywords.contains(&Keyword::Haste));
    assert_eq!(card.triggered_abilities.len(), 1, "graveyard return trigger");
}

// ── Portal to Phyrexia ──────────────────────────────────────────────────────

#[test]
fn portal_to_phyrexia_etb_forces_opponent_sacrifice() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear3 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _ = (bear1, bear2, bear3);

    let portal = g.add_card_to_hand(0, catalog::portal_to_phyrexia());
    g.players[0].mana_pool.add_colorless(9);
    let opp_creatures_before = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.card_types.contains(&CardType::Creature))
        .count();
    assert_eq!(opp_creatures_before, 3);

    g.perform_action(GameAction::CastSpell {
        card_id: portal, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Portal castable");
    drain_stack(&mut g);

    let opp_creatures_after = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.card_types.contains(&CardType::Creature))
        .count();
    assert_eq!(opp_creatures_after, 0, "Portal ETB should sac 3 creatures");
}

// ── Finale of Devastation ───────────────────────────────────────────────────

// ── Rishadan Port ───────────────────────────────────────────────────────────

#[test]
fn rishadan_port_taps_for_colorless() {
    let mut g = two_player_game();
    let port = g.add_card_to_battlefield(0, catalog::rishadan_port());
    g.perform_action(GameAction::ActivateAbility { card_id: port, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("tap for {C}");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.colorless_amount() > 0, "Port should produce colorless mana");
}

#[test]
fn rishadan_port_taps_target_land() {
    let mut g = two_player_game();
    let port = g.add_card_to_battlefield(0, catalog::rishadan_port());
    let opp_land = g.add_card_to_battlefield(1, catalog::island());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: port, ability_index: 1,
        target: Some(Target::Permanent(opp_land)), additional_targets: Vec::new(), x_value: None,
    }).expect("tap opp land");
    drain_stack(&mut g);
    let opp_land_card = g.battlefield.iter().find(|c| c.id == opp_land).unwrap();
    assert!(opp_land_card.tapped, "Opponent's land should be tapped");
}

// ── Horizon Canopy ──────────────────────────────────────────────────────────

#[test]
fn horizon_canopy_taps_for_green_costing_one_life() {
    let mut g = two_player_game();
    let hc = g.add_card_to_battlefield(0, catalog::horizon_canopy());
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility { card_id: hc, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("tap for {G}");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::Green) > 0, "Should produce green mana");
    assert_eq!(g.players[0].life, life_before - 1, "Should cost 1 life");
}

#[test]
fn horizon_canopy_sac_draws_a_card() {
    let mut g = two_player_game();
    let hc = g.add_card_to_battlefield(0, catalog::horizon_canopy());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility { card_id: hc, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("sac for draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "Should draw 1");
    assert!(g.battlefield.iter().all(|c| c.id != hc), "HC should be sacrificed");
}

// ── Sunbaked Canyon ─────────────────────────────────────────────────────────

#[test]
fn sunbaked_canyon_taps_for_red_costing_one_life() {
    let mut g = two_player_game();
    let sc = g.add_card_to_battlefield(0, catalog::sunbaked_canyon());
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility { card_id: sc, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("tap for {R}");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::Red) > 0, "Should produce red mana");
    assert_eq!(g.players[0].life, life_before - 1, "Should cost 1 life");
}

// ── Waterlogged Grove ───────────────────────────────────────────────────────

#[test]
fn waterlogged_grove_taps_for_green_costing_one_life() {
    let mut g = two_player_game();
    let wg = g.add_card_to_battlefield(0, catalog::waterlogged_grove());
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility { card_id: wg, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("tap for {G}");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::Green) > 0, "Should produce green mana");
    assert_eq!(g.players[0].life, life_before - 1, "Should cost 1 life");
}

// ── Horizon-cycle completion (Fiery Islet / Nurturing Peatland / Silent Clearing) ──

#[test]
fn fiery_islet_taps_for_blue_costing_one_life() {
    let mut g = two_player_game();
    let fi = g.add_card_to_battlefield(0, catalog::fiery_islet());
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility { card_id: fi, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("tap for {U}");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::Blue) > 0);
    assert_eq!(g.players[0].life, life_before - 1);
}

#[test]
fn silent_clearing_sac_draws_a_card() {
    let mut g = two_player_game();
    let sc = g.add_card_to_battlefield(0, catalog::silent_clearing());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility { card_id: sc, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("sac for draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert!(g.battlefield.iter().all(|c| c.id != sc));
}

// ── Verge lands (conditional second-color mana ability) ─────────────────────

#[test]
fn blazemire_verge_taps_for_black_unconditionally() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::blazemire_verge());
    g.perform_action(GameAction::ActivateAbility { card_id: v, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("tap for {B}");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::Black) > 0);
}

#[test]
fn blazemire_verge_red_gated_on_swamp_or_mountain() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::blazemire_verge());
    // No Swamp/Mountain controlled → the red ability is illegal.
    assert!(g
        .perform_action(GameAction::ActivateAbility { card_id: v, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None })
        .is_err());
    // Controlling a Mountain unlocks it.
    g.add_card_to_battlefield(0, catalog::mountain());
    g.perform_action(GameAction::ActivateAbility { card_id: v, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("red now allowed");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::Red) > 0);
}

// ── Koma, Cosmos Serpent ────────────────────────────────────────────────────

#[test]
fn koma_cosmos_serpent_is_6_6_uncounterable_serpent() {
    use crabomination::card::Keyword;
    let card = catalog::koma_cosmos_serpent();
    assert_eq!(card.name, "Koma, Cosmos Serpent");
    assert_eq!(card.power, 6);
    assert_eq!(card.toughness, 6);
    assert!(card.keywords.contains(&Keyword::CantBeCountered));
    assert_eq!(card.triggered_abilities.len(), 1, "upkeep token trigger");
}

// ── Mesmeric Orb ────────────────────────────────────────────────────────────

// ── Chalice of the Void ─────────────────────────────────────────────────────

// ── Candelabra of Tawnos ────────────────────────────────────────────────────

// ── Archdruid's Charm ───────────────────────────────────────────────────────

// ── Awaken the Honored Dead ─────────────────────────────────────────────────

// ── Growing Ranks ───────────────────────────────────────────────────────────

// ── Monument to Endurance ───────────────────────────────────────────────────

#[test]
fn monument_to_endurance_pumps_target_creature() {
    let mut g = two_player_game();
    let mon = g.add_card_to_battlefield(0, catalog::monument_to_endurance());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    let power_before = g.battlefield.iter().find(|c| c.id == bear).unwrap().definition.power;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mon, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate pump");
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let cp = computed.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(cp.power, power_before + 2, "Should pump +2/+2");
}

// ── Exotic Orchard ──────────────────────────────────────────────────────────

#[test]
fn exotic_orchard_taps_for_any_color() {
    let mut g = two_player_game();
    let eo = g.add_card_to_battlefield(0, catalog::exotic_orchard());
    g.perform_action(GameAction::ActivateAbility {
        card_id: eo, ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for mana");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.total() > 0, "Should produce mana");
}

// ── Master of Death ─────────────────────────────────────────────────────────

#[test]
fn growing_ranks_populates_a_token_on_upkeep() {
    // Growing Ranks populates (CR 701.32): copies a creature token you control.
    use crabomination::card::{CardDefinition, CardType, CreatureType, Subtypes};
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::growing_ranks());
    let tok = g.add_card_to_battlefield(0, CardDefinition {
        name: "Centaur",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Centaur], ..Default::default() },
        power: 3,
        toughness: 3,
        ..Default::default()
    });
    g.battlefield_find_mut(tok).unwrap().is_token = true;
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let centaurs = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Centaur").count();
    assert_eq!(centaurs, 2, "populate copied the Centaur token");
}

#[test]
fn master_of_death_returns_from_graveyard_on_upkeep() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    // Put Master of Death directly into the graveyard.
    let _mod_id = g.add_card_to_graveyard(0, catalog::master_of_death());
    let hand_before = g.players[0].hand.len();
    // ScriptedDecider answers MayDo(yes) to pay 1 life.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
    ]));
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    // Master of Death should be in hand now.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Master of Death should return to hand");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Master of Death"),
        "Master of Death should be in hand");
    // Player should have lost 1 life.
    assert_eq!(g.players[0].life, 19, "Should have paid 1 life");
}

// ── Basking Broodscale ──────────────────────────────────────────────────────

// ── Sowing Mycospawn ────────────────────────────────────────────────────────

// ── Ursine Monstrosity ──────────────────────────────────────────────────────

#[test]
fn ursine_monstrosity_enters_with_five_counters_and_draws() {
    use crabomination::card::Keyword;
    let card = catalog::ursine_monstrosity();
    assert_eq!(card.name, "Ursine Monstrosity");
    assert!(card.keywords.contains(&Keyword::Trample));
    assert!(card.enters_with_counters.is_some());
    assert_eq!(card.triggered_abilities.len(), 1, "ETB draw");
}

// ── Moonshadow ──────────────────────────────────────────────────────────────

#[test]
fn moonshadow_is_2_1_flying_faerie_with_discard_trigger() {
    use crabomination::card::Keyword;
    let card = catalog::moonshadow();
    assert_eq!(card.name, "Moonshadow");
    assert_eq!(card.power, 7);
    assert_eq!(card.toughness, 7);
    assert!(card.keywords.contains(&Keyword::Flying));
    assert_eq!(card.triggered_abilities.len(), 1, "combat damage discard");
}

// ── Golos, Tireless Pilgrim ─────────────────────────────────────────────────

#[test]
fn golos_tireless_pilgrim_is_legendary_3_5_with_etb() {
    use crabomination::card::Supertype;
    let card = catalog::golos_tireless_pilgrim();
    assert_eq!(card.name, "Golos, Tireless Pilgrim");
    assert!(card.supertypes.contains(&Supertype::Legendary));
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 5);
    assert_eq!(card.triggered_abilities.len(), 1, "ETB land search");
}

// ── Maelstrom Archangel ─────────────────────────────────────────────────────

#[test]
fn maelstrom_archangel_is_5_5_flying_five_color() {
    use crabomination::card::Keyword;
    let card = catalog::maelstrom_archangel();
    assert_eq!(card.name, "Maelstrom Archangel");
    assert_eq!(card.power, 5);
    assert_eq!(card.toughness, 5);
    assert!(card.keywords.contains(&Keyword::Flying));
    assert_eq!(card.cost.cmc(), 5, "WUBRG = 5 CMC");
}

/// Combat damage to a player → free-cast a spell from hand.
#[test]
fn maelstrom_archangel_free_casts_from_hand_on_combat_damage() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::maelstrom_archangel());
    g.clear_sickness(angel);
    let fatty = g.add_card_to_hand(0, catalog::serra_angel()); // no mana available
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![fatty])]));
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: angel, target: AttackTarget::Player(1) },
    ])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat resolves");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fatty).is_some(), "hand spell cast for free");
    assert_eq!(g.players[1].life, 15, "5 combat damage dealt");
}

// ── Duplicant ───────────────────────────────────────────────────────────────

/// Imprint: ETB exiles a creature; Duplicant's CDA takes its P/T.
#[test]
fn duplicant_imprints_and_copies_pt() {
    let mut g = two_player_game();
    let fatty = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let dup = g.add_card_to_hand(0, catalog::duplicant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: dup, target: Some(Target::Permanent(fatty)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Duplicant castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == fatty), "target exiled on ETB");
    let cp = g.computed_permanent(dup).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "CDA copies the exiled card's P/T");
}

// ── Ramos, Dragon Engine ────────────────────────────────────────────────────

#[test]
fn ramos_dragon_engine_is_4_4_flying_dragon_with_counter_trigger() {
    use crabomination::card::Keyword;
    let card = catalog::ramos_dragon_engine();
    assert_eq!(card.name, "Ramos, Dragon Engine");
    assert_eq!(card.power, 4);
    assert_eq!(card.toughness, 4);
    assert!(card.keywords.contains(&Keyword::Flying));
    assert_eq!(card.triggered_abilities.len(), 1, "spell-cast counter trigger");
    assert_eq!(card.activated_abilities.len(), 1, "mana burst activation");
}

// ── Omnath, Locus of Creation ───────────────────────────────────────────────

#[test]
fn omnath_locus_of_creation_etb_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let omnath = g.add_card_to_hand(0, catalog::omnath_locus_of_creation());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: omnath, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Omnath castable");
    drain_stack(&mut g);
    // ETB draws 1 (net 0 after casting Omnath); the life-gain is the first
    // landfall, not the ETB.
    assert_eq!(g.players[0].hand.len(), hand_before, "ETB draws 1 (net 0)");
    assert_eq!(g.players[0].life, life_before, "no life gain on ETB");
}

// ── Omnath, Locus of Rage ───────────────────────────────────────────────────

// ── Torsten ─────────────────────────────────────────────────────────────────

#[test]
fn torsten_etb_takes_creatures_and_lands_and_dies_to_seven_soldiers() {
    let mut g = two_player_game();
    // Top of library: Bear (creature), Forest (land), Bolt (instant). Only the
    // first two go to hand; the instant is bottomed.
    g.players[0].library.clear();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::torsten_founder_of_benalia());
    // Fire the ETB trigger.
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2, "creature + land taken to hand");
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bolt), "instant bottomed");
    // Now kill it: seven Soldiers.
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.definition.name == "Soldier" && c.controller == 0).count();
    assert_eq!(soldiers, 7, "dies → seven 1/1 Soldiers");
}

// ── Coveted Jewel ───────────────────────────────────────────────────────────

#[test]
fn coveted_jewel_etb_draws_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let jewel = g.add_card_to_hand(0, catalog::coveted_jewel());
    g.players[0].mana_pool.add_colorless(6);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: jewel, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Jewel castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "ETB should draw 3 (net +2 after casting)");
}

// ── The Mightstone and Weakstone ────────────────────────────────────────────

// ── Doomsday Excruciator ────────────────────────────────────────────────────

#[test]
fn doomsday_excruciator_etb_leaves_each_player_six_cards() {
    let mut g = two_player_game();
    for _ in 0..15 { g.add_card_to_library(0, catalog::island()); }
    for _ in 0..15 { g.add_card_to_library(1, catalog::forest()); }
    let id = g.add_card_to_battlefield(0, catalog::doomsday_excruciator());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), 6, "controller keeps bottom six");
    assert_eq!(g.players[1].library.len(), 6, "opponent keeps bottom six");
    assert!(g.exile.len() >= 18, "the rest are exiled");
}

// ── Planar Nexus ────────────────────────────────────────────────────────────

// ── Kozilek's Command ───────────────────────────────────────────────────────

// ── Eldrazi Confluence ──────────────────────────────────────────────────────

// ── Aluren ──────────────────────────────────────────────────────────────────

// ── New cube cards ─────────────────────────────────────────────────────────

#[test]
fn messenger_falcons_etb_draws_a_card() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::messenger_falcons());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Messenger Falcons castable");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.definition.name == "Messenger Falcons"));
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1, "ETB draws 1");
}

#[test]
fn messenger_falcons_hybrid_pip_payable_with_blue() {
    // {2}{G/U}{W}: pay the hybrid pip with blue instead of green.
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::messenger_falcons());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Messenger Falcons castable for {2}{U}{W} via the hybrid pip");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Messenger Falcons"));
}

#[test]
fn trenchpost_taps_for_one_colorless() {
    let mut g = two_player_game();
    let tp = g.add_card_to_battlefield(0, catalog::trenchpost());
    g.perform_action(GameAction::ActivateAbility {
        card_id: tp, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Trenchpost tap should work");
    assert_eq!(g.players[0].mana_pool.total(), 1, "Should add 1 colorless mana");
}

