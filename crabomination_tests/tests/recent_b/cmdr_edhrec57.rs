//! Commander: EDHREC ranks 2001-2500, one-card gaps, third batch
//! (`decks::cmdr_edhrec57`).

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn fill_pool(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
    g.priority.player_with_priority = 0;
}

fn cast_at(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    fill_pool(g);
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    fill_pool(g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let atks = attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect();
    g.perform_action(GameAction::DeclareAttackers(atks)).expect("attack");
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// Archangel Elspeth's −2 makes an Angel with flying and two counters; her −6
/// returns every cheap nonland permanent card (CR 606).
#[test]
fn archangel_elspeth_angels_and_returns() {
    let mut g = pod(2);
    let elspeth = ready(&mut g, 0, catalog::archangel_elspeth());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: elspeth,
        ability_index: 1,
        target: Some(Target::Permanent(bears)),
        x_value: None,
    })
    .expect("-2");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!(cp.power, 4);
    assert!(cp.keywords().contains(&Keyword::Flying));
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Angel));

    let mut g = pod(2);
    let elspeth = ready(&mut g, 0, catalog::archangel_elspeth());
    g.battlefield_find_mut(elspeth).unwrap().add_counters(CounterType::Loyalty, 4);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let maw = g.add_card_to_graveyard(0, catalog::colossal_dreadmaw());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: elspeth,
        ability_index: 2,
        target: None,
        x_value: None,
    })
    .expect("-6");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_some(), "mana value 1 comes back");
    assert!(g.battlefield_find(maw).is_none(), "mana value 6 stays");
}

/// Magus of the Order trades itself and another green creature for a green
/// creature from the library.
#[test]
fn magus_of_the_order_tutors_a_green_creature() {
    let mut g = pod(2);
    g.players[0].library.clear();
    let maw = g.add_card_to_library(0, catalog::colossal_dreadmaw());
    let magus = ready(&mut g, 0, catalog::magus_of_the_order());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, magus, 0, None).expect("activate");
    assert!(g.battlefield_find(magus).is_none() && g.battlefield_find(bears).is_none());
    assert!(g.battlefield_find(maw).is_some());
}

/// Manaforge Cinder filters at most three times a turn.
#[test]
fn manaforge_cinder_three_times_a_turn() {
    let mut g = pod(2);
    let cinder = ready(&mut g, 0, catalog::manaforge_cinder());
    for _ in 0..3 {
        activate(&mut g, cinder, 0, None).expect("filter");
    }
    assert!(activate(&mut g, cinder, 0, None).is_err(), "a fourth is refused");
}

/// Boundless Realms fetches as many basics as you have lands.
#[test]
fn boundless_realms_doubles_lands() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.players[0].library.clear();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::boundless_realms());
    cast_at(&mut g, spell, None).expect("cast");
    assert_eq!(named(&g, 0, "Forest"), 6);
}

/// Goblin Warrens turns two Goblins into three.
#[test]
fn goblin_warrens_two_for_three() {
    let mut g = pod(2);
    let warrens = ready(&mut g, 0, catalog::goblin_warrens());
    for _ in 0..2 {
        ready(&mut g, 0, catalog::goblin_guide());
    }
    activate(&mut g, warrens, 0, None).expect("activate");
    assert_eq!(named(&g, 0, "Goblin Guide"), 0);
    assert_eq!(named(&g, 0, "Goblin"), 3);
}

/// Cathedral of War: exalted pumps a lone attacker (CR 702.83).
#[test]
fn cathedral_of_war_exalts() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::cathedral_of_war());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    attack(&mut g, &[bears]);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 3);
}

/// Depart the Realm bounces to the owner's hand.
#[test]
fn depart_the_realm_bounces() {
    let mut g = pod(2);
    let maw = ready(&mut g, 1, catalog::colossal_dreadmaw());
    let spell = g.add_card_to_hand(0, catalog::depart_the_realm());
    cast_at(&mut g, spell, Some(Target::Permanent(maw))).expect("cast");
    assert!(g.players[1].hand.iter().any(|c| c.id == maw));
    assert!(catalog::depart_the_realm().foretell_cost.is_some());
}

/// Honored Crop-Captain and Márton Stromgald pump the other attackers.
#[test]
fn crop_captain_and_marton_rally_attackers() {
    let mut g = pod(2);
    let captain = ready(&mut g, 0, catalog::honored_crop_captain());
    let marton = ready(&mut g, 0, catalog::marton_stromgald());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    attack(&mut g, &[captain, marton, bears]);
    // Bears: 2 + 1 (Captain) + 2 (Márton: two other attackers) = 5.
    assert_eq!(g.computed_permanent(bears).unwrap().power, 5);
    // Captain: 3 + 2 (Márton) = 5; Márton: 1 + 1 (Captain) = 2.
    assert_eq!(g.computed_permanent(captain).unwrap().power, 5);
    assert_eq!(g.computed_permanent(marton).unwrap().power, 2);
}

/// Ironroot Warlord counts your creatures and makes Soldiers.
#[test]
fn ironroot_warlord_counts_creatures() {
    let mut g = pod(2);
    let lord = ready(&mut g, 0, catalog::ironroot_warlord());
    ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(lord).unwrap().power, 2);
    activate(&mut g, lord, 0, None).expect("soldier");
    assert_eq!(g.computed_permanent(lord).unwrap().power, 3);
}

/// Corpse Connoisseur bins a creature from the library and has unearth.
#[test]
fn corpse_connoisseur_entombs() {
    let mut g = pod(2);
    g.players[0].library.clear();
    let maw = g.add_card_to_library(0, catalog::colossal_dreadmaw());
    let cc = g.add_card_to_hand(0, catalog::corpse_connoisseur());
    cast_at(&mut g, cc, None).expect("cast");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == maw));
}

/// Braulios draws on attack and drops a land from hand; Imaryll grows with
/// your other Elves.
#[test]
fn braulios_and_imaryll_attack() {
    let mut g = pod(2);
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::island());
    let braulios = ready(&mut g, 0, catalog::braulios_of_pheres_band());
    g.add_card_to_battlefield(0, catalog::forest());
    let imaryll = ready(&mut g, 0, catalog::imaryll_elfhame_elite());
    ready(&mut g, 0, catalog::llanowar_elves());
    attack(&mut g, &[braulios, imaryll]);
    assert_eq!(named(&g, 0, "Island"), 1, "the drawn land went in");
    assert_eq!(g.computed_permanent(braulios).unwrap().power, 2);
    assert_eq!(g.computed_permanent(imaryll).unwrap().power, 4);
}

/// Frodo picks up a two-drop Equipment as he enters and takes no damage on
/// your turn.
#[test]
fn frodo_suits_up_and_is_shielded() {
    let mut g = pod(2);
    let greaves = ready(&mut g, 0, catalog::vulshok_morningstar());
    let frodo = g.add_card_to_hand(0, catalog::frodo_determined_hero());
    cast_at(&mut g, frodo, Some(Target::Permanent(greaves))).expect("cast");
    assert_eq!(g.battlefield_find(greaves).unwrap().attached_to, Some(frodo));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, Some(Target::Permanent(frodo))).expect("bolt");
    assert!(g.battlefield_find(frodo).is_some(), "prevented on your turn");
}
