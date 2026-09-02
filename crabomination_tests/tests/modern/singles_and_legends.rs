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
    g.perform_action(GameAction::ActivateAbility { card_id: port, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
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
        target: Some(Target::Permanent(opp_land)), additional_targets: Vec::new(), x_value: None, mode: None,
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
    g.perform_action(GameAction::ActivateAbility { card_id: hc, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
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
    g.perform_action(GameAction::ActivateAbility { card_id: hc, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
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
    g.perform_action(GameAction::ActivateAbility { card_id: sc, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
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
    g.perform_action(GameAction::ActivateAbility { card_id: wg, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
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
    g.perform_action(GameAction::ActivateAbility { card_id: fi, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
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
    g.perform_action(GameAction::ActivateAbility { card_id: sc, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
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
    g.perform_action(GameAction::ActivateAbility { card_id: v, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
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
        .perform_action(GameAction::ActivateAbility { card_id: v, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
        .is_err());
    // Controlling a Mountain unlocks it.
    g.add_card_to_battlefield(0, catalog::mountain());
    g.perform_action(GameAction::ActivateAbility { card_id: v, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
        .expect("red now allowed");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::Red) > 0);
}

// ── Koma, Cosmos Serpent ────────────────────────────────────────────────────

// ── Mesmeric Orb ────────────────────────────────────────────────────────────

// ── Chalice of the Void ─────────────────────────────────────────────────────

// ── Candelabra of Tawnos ────────────────────────────────────────────────────

// ── Archdruid's Charm ───────────────────────────────────────────────────────

// ── Awaken the Honored Dead ─────────────────────────────────────────────────

// ── Growing Ranks ───────────────────────────────────────────────────────────

// ── Monument to Endurance ───────────────────────────────────────────────────

/// "Whenever you discard a card, choose one that hasn't been chosen this turn."
/// Shipped as an unrelated `{2}, {T}: +2/+2` pump until the `token`
/// oracle-verb class; the "this turn" window is still an approximation (the
/// pick is recorded for the game, see the card's doc comment).
#[test]
fn monument_to_endurance_fires_a_fresh_mode_on_each_discard() {
    let mut g = two_player_game();
    let mon = g.add_card_to_battlefield(0, catalog::monument_to_endurance());
    // The Draw mode is one of the three, and `two_player_game()` deals an
    // empty library — drawing off it loses the game before the third discard.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let pitch: Vec<_> = (0..3).map(|_| g.add_card_to_hand(0, catalog::grizzly_bears())).collect();
    let life_before = g.players[1].life;
    for card in pitch {
        let mut evs = Vec::new();
        g.discard_card(0, card, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    let chosen = &g.battlefield_find(mon).expect("Monument still out").modes_chosen;
    assert_eq!(chosen.len(), 3, "one fresh mode per discard, never a repeat");
    let mut sorted = chosen.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, vec![0, 1, 2], "all three modes get used exactly once");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"), "the Treasure mode ran");
    assert_eq!(g.players[1].life, life_before - 3, "the drain mode ran");
}

// ── Exotic Orchard ──────────────────────────────────────────────────────────

#[test]
fn exotic_orchard_taps_for_any_color() {
    let mut g = two_player_game();
    let eo = g.add_card_to_battlefield(0, catalog::exotic_orchard());
    g.perform_action(GameAction::ActivateAbility {
        card_id: eo, ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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

// ── Moonshadow ──────────────────────────────────────────────────────────────

// ── Golos, Tireless Pilgrim ─────────────────────────────────────────────────

// ── Maelstrom Archangel ─────────────────────────────────────────────────────

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
        card_id: tp, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Trenchpost tap should work");
    assert_eq!(g.players[0].mana_pool.total(), 1, "Should add 1 colorless mana");
}


// ─────────────────────────────────────────────────────────────────────────
// One definition audit for the printed shapes this file used to check with a
// six-line test each. Every row is exactly what the seven deleted tests
// asserted — printed name, mana value, P/T, keywords, supertypes,
// enters-with-counters, and the triggered / activated ability counts — so the
// coverage is unchanged and a failure names the card. Same shape as
// `stx/part_23.rs`'s table; see CLAUDE.md's "one table-driven definition
// audit per set".
// ─────────────────────────────────────────────────────────────────────────

struct PrintedShape {
    def: fn() -> crabomination::card::CardDefinition,
    name: &'static str,
    cmc: Option<u32>,
    pt: Option<(i32, i32)>,
    kws: &'static [crabomination::card::Keyword],
    supers: &'static [crabomination::card::Supertype],
    enters_with_counters: bool,
    trigs: Option<usize>,
    acts: Option<usize>,
}

#[test]
fn singles_and_legends_printed_shapes() {
    use crabomination::card::{Keyword, Supertype};
    const ROWS: &[PrintedShape] = &[
        PrintedShape { def: catalog::vengevine, name: "Vengevine",
            cmc: None, pt: Some((4, 3)), kws: &[Keyword::Haste], supers: &[],
            enters_with_counters: false, trigs: Some(1), acts: None },
        PrintedShape { def: catalog::koma_cosmos_serpent, name: "Koma, Cosmos Serpent",
            cmc: None, pt: Some((6, 6)), kws: &[Keyword::CantBeCountered], supers: &[],
            enters_with_counters: false, trigs: Some(1), acts: None },
        // Printed 3/3 with a begin-combat trigger, corrected 2026-09-01: the
        // body was a 0/0 entering with five +1/+1 counters and drawing on ETB,
        // which is not this card at all.
        PrintedShape { def: catalog::ursine_monstrosity, name: "Ursine Monstrosity",
            cmc: None, pt: Some((3, 3)), kws: &[Keyword::Trample], supers: &[],
            enters_with_counters: false, trigs: Some(1), acts: None },
        // 7/7, not the 2/1 the deleted test's *name* claimed; its asserts said 7/7.
        // Menace, not flying — the printed keyword, corrected 2026-08-30; the
        // six -1/-1 counters and the graveyard trigger, the printed body.
        PrintedShape { def: catalog::moonshadow, name: "Moonshadow",
            cmc: None, pt: Some((7, 7)), kws: &[Keyword::Menace], supers: &[],
            enters_with_counters: true, trigs: Some(1), acts: None },
        PrintedShape { def: catalog::golos_tireless_pilgrim, name: "Golos, Tireless Pilgrim",
            cmc: None, pt: Some((3, 5)), kws: &[], supers: &[Supertype::Legendary],
            enters_with_counters: false, trigs: Some(1), acts: None },
        PrintedShape { def: catalog::maelstrom_archangel, name: "Maelstrom Archangel",
            cmc: Some(5), pt: Some((5, 5)), kws: &[Keyword::Flying], supers: &[],
            enters_with_counters: false, trigs: None, acts: None },
        PrintedShape { def: catalog::ramos_dragon_engine, name: "Ramos, Dragon Engine",
            cmc: None, pt: Some((4, 4)), kws: &[Keyword::Flying], supers: &[],
            enters_with_counters: false, trigs: Some(1), acts: Some(1) },
    ];
    for row in ROWS {
        let def = (row.def)();
        assert_eq!(def.name, row.name, "printed name");
        if let Some(cmc) = row.cmc {
            assert_eq!(def.cost.cmc(), cmc, "{} mana value", row.name);
        }
        if let Some((p, t)) = row.pt {
            assert_eq!((def.power, def.toughness), (p, t), "{} printed P/T", row.name);
        }
        for kw in row.kws {
            assert!(def.keywords.contains(kw), "{} has {:?}", row.name, kw);
        }
        for st in row.supers {
            assert!(def.supertypes.contains(st), "{} is {:?}", row.name, st);
        }
        if row.enters_with_counters {
            assert!(def.enters_with_counters.is_some(), "{} enters with counters", row.name);
        }
        if let Some(n) = row.trigs {
            assert_eq!(def.triggered_abilities.len(), n, "{} triggered abilities", row.name);
        }
        if let Some(n) = row.acts {
            assert_eq!(def.activated_abilities.len(), n, "{} activated abilities", row.name);
        }
    }
}

/// Moonshadow enters as a 1/1 (7/7 under six -1/-1 counters) and sheds a
/// counter whenever a permanent card is put into your graveyard.
#[test]
fn moonshadow_enters_shrunk_and_grows_as_permanents_hit_your_graveyard() {
    let mut g = two_player_game();
    let moon = g.move_card_to_battlefield_for_test(0, catalog::moonshadow());
    let cp = g.computed_permanent(moon).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "six -1/-1 counters on a 7/7");
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let evs = g.remove_to_graveyard_with_triggers(bears);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(moon).unwrap().counter_count(CounterType::MinusOneMinusOne), 5,
        "a permanent card in your graveyard removed one counter");
    // An instant card going to the graveyard is not a permanent card.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.discard_card(0, bolt, &mut Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(moon).unwrap().counter_count(CounterType::MinusOneMinusOne), 5);
}

/// Vault Plunderer: target player draws a card AND loses 1 life — the two
/// land on the same targeted player (a bot seat auto-picks itself for a
/// draw; the body used to draw for you and lose nothing).
#[test]
fn vault_plunderer_targets_a_player_for_the_draw_and_the_life() {
    let mut g = two_player_game();
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let vp = g.add_card_to_battlefield(0, catalog::vault_plunderer());
    let (hand, life) = (g.players[1].hand.len(), g.players[1].life);
    let (my_hand, my_life) = (g.players[0].hand.len(), g.players[0].life);
    g.fire_self_etb_triggers(vp, 0);
    drain_stack(&mut g);
    let drew = [g.players[0].hand.len() - my_hand, g.players[1].hand.len() - hand];
    let lost = [my_life - g.players[0].life, life - g.players[1].life];
    assert_eq!(drew.iter().sum::<usize>(), 1, "exactly one player drew");
    assert_eq!(drew.map(|d| d as i32), lost, "the player who drew is the one who lost 1 life");
}
