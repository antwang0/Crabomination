//! Commander: the 2026-10-20 (`01E4rMUL`) two-gap pair closers
//! (`decks::cmdr_01e4_a`).

use crabomination::card::{CardId, CardType, CounterType, CreatureType, Keyword, Subtypes, TokenDefinition};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast_x(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    cast_x(g, id, target, None);
}

fn activate(g: &mut GameState, id: CardId, index: usize) -> Result<(), GameError> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn body(name: &str, power: i32, kind: CreatureType) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power,
        toughness: power,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![kind], ..Default::default() },
        ..Default::default()
    }
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn attack_with(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

/// Dragonloft Idol is a 3/3 until you control a Dragon, then a 4/4 flier
/// with trample.
#[test]
fn dragonloft_idol_grows_wings_beside_a_dragon() {
    let mut g = pod(3);
    let idol = g.add_card_to_battlefield(0, catalog::dragonloft_idol());
    assert_eq!(pt(&g, idol), (3, 3));
    g.add_card_to_battlefield(1, catalog::scion_of_ugin());
    assert_eq!(pt(&g, idol), (3, 3), "an opponent's Dragon does nothing");
    g.add_card_to_battlefield(0, catalog::scion_of_ugin());
    assert_eq!(pt(&g, idol), (4, 4));
    let idol = g.computed_permanent(idol).unwrap();
    assert!(idol.keywords().contains(&Keyword::Flying) && idol.keywords().contains(&Keyword::Trample));
}

/// CR 603.4 — Webweaver Changeling's lifegain is an intervening if: two
/// creature cards in the graveyard gain nothing, three gain 5.
#[test]
fn cr_603_4_webweaver_changeling_counts_creature_cards() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.add_card_to_graveyard(0, catalog::island());
    let first = g.add_card_to_hand(0, catalog::webweaver_changeling());
    let life = g.players[0].life;
    cast(&mut g, first, None);
    assert_eq!(g.players[0].life, life, "two creature cards and a land: no trigger");
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let second = g.add_card_to_hand(0, catalog::webweaver_changeling());
    cast(&mut g, second, None);
    assert_eq!(g.players[0].life, life + 5);
}

/// Fathom Fleet Boarder costs 2 life alone, nothing beside another Pirate.
#[test]
fn fathom_fleet_boarder_wants_another_pirate() {
    let mut g = pod(2);
    let life = g.players[0].life;
    let first = g.add_card_to_hand(0, catalog::fathom_fleet_boarder());
    cast(&mut g, first, None);
    assert_eq!(g.players[0].life, life - 2, "no other Pirate");
    let second = g.add_card_to_hand(0, catalog::fathom_fleet_boarder());
    cast(&mut g, second, None);
    assert_eq!(g.players[0].life, life - 2, "the first Boarder is another Pirate");
}

/// Deadeye Tormentor's raid discard fires only after you attacked.
#[test]
fn deadeye_tormentor_raids_for_a_discard() {
    let mut g = pod(3);
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::island());
    let quiet = g.add_card_to_hand(0, catalog::deadeye_tormentor());
    cast(&mut g, quiet, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 2, "no attack this turn");
    g.players[0].attacked_this_turn = true;
    let raider = g.add_card_to_hand(0, catalog::deadeye_tormentor());
    cast(&mut g, raider, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Scarred Vinebreeder's pump costs an Elf card from your graveyard; a
/// non-Elf creature card does not pay it.
#[test]
fn scarred_vinebreeder_exiles_an_elf_card() {
    let mut g = pod(2);
    let breeder = g.add_card_to_battlefield(0, catalog::scarred_vinebreeder());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    assert!(activate(&mut g, breeder, 0).is_err(), "a Bear is no Elf");
    let elf = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    activate(&mut g, breeder, 0).expect("exile the Elf");
    assert_eq!(pt(&g, breeder), (4, 4));
    assert!(g.exile.iter().any(|c| c.id == elf));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
}

/// Consuming Fervor: +3/+3, and the enchanted creature takes a -1/-1 counter
/// at its controller's upkeep.
#[test]
fn consuming_fervor_shrinks_its_host_each_upkeep() {
    let mut g = pod(2);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let fervor = g.add_card_to_hand(0, catalog::consuming_fervor());
    cast(&mut g, fervor, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (5, 5));
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::End;
    while !(g.active_player_idx == 0 && g.step == TurnStep::Upkeep) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
    assert_eq!(pt(&g, bear), (4, 4));
}

/// Hydrolash shrinks attacking creatures only, and draws.
#[test]
fn hydrolash_blunts_only_attackers() {
    let mut g = pod(2);
    let attacker = g.add_token_to_battlefield(0, &body("Bear", 3, CreatureType::Bear));
    let home = g.add_token_to_battlefield(0, &body("Wolf", 3, CreatureType::Wolf));
    attack_with(&mut g, &[attacker]);
    let hand = g.players[0].hand.len();
    let lash = g.add_card_to_hand(0, catalog::hydrolash());
    cast(&mut g, lash, None);
    assert_eq!(pt(&g, attacker), (1, 3));
    assert_eq!(pt(&g, home), (3, 3));
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Archangel's Light gains 2 a graveyard card, then the graveyard joins the
/// library.
#[test]
fn archangels_light_counts_then_shuffles_the_graveyard() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    let library = g.players[0].library.len();
    let life = g.players[0].life;
    let light = g.add_card_to_hand(0, catalog::archangels_light());
    cast(&mut g, light, None);
    assert_eq!(g.players[0].life, life + 6);
    assert!(g.players[0].graveyard.iter().all(|c| c.id == light), "only the resolved Light is left");
    assert_eq!(g.players[0].library.len(), library + 3);
}

/// Abnormal Endurance: +2/+0, and the creature comes back tapped when it
/// dies this turn.
#[test]
fn abnormal_endurance_returns_the_creature_tapped() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let endurance = g.add_card_to_hand(0, catalog::abnormal_endurance());
    cast(&mut g, endurance, Some(Target::Permanent(bears)));
    assert_eq!(pt(&g, bears), (4, 2));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bears)));
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("returned");
    assert!(back.tapped);
}

/// Luxurious Libation pumps by X and leaves a green-white Citizen.
#[test]
fn luxurious_libation_pumps_by_x_and_makes_a_citizen() {
    let mut g = pod(2);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let libation = g.add_card_to_hand(0, catalog::luxurious_libation());
    cast_x(&mut g, libation, Some(Target::Permanent(bear)), Some(3));
    assert_eq!(pt(&g, bear), (5, 5));
    let citizen = g.battlefield.iter().find(|c| c.definition.name == "Citizen").expect("a Citizen");
    assert_eq!(citizen.controller, 0);
}

/// Otherworldly Outburst leaves a 3/2 Eldrazi Horror when its target dies
/// this turn.
#[test]
fn otherworldly_outburst_leaves_a_horror_behind() {
    let mut g = pod(2);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let outburst = g.add_card_to_hand(0, catalog::otherworldly_outburst());
    cast(&mut g, outburst, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (3, 2));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear)));
    let horror = g.battlefield.iter().find(|c| c.definition.name == "Eldrazi Horror").expect("a Horror");
    assert_eq!(pt(&g, horror.id), (3, 2));
}

/// Burden of Guilt taps the creature it enchants for {1}.
#[test]
fn burden_of_guilt_taps_its_host() {
    let mut g = pod(2);
    let theirs = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let burden = g.add_card_to_hand(0, catalog::burden_of_guilt());
    cast(&mut g, burden, Some(Target::Permanent(theirs)));
    activate(&mut g, burden, 0).expect("tap it");
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}
