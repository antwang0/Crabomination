//! Commander: the Calling All Angels precon (FDC, Giada, `decks::cmdr_giada`).

use crabomination::card::{ArtifactSubtype, CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_action(id: CardId, target: Option<Target>, mode: Option<usize>) -> GameAction {
    GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: None }
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g, 0);
    g.perform_action(cast_action(id, target, None)).expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, index: usize) -> Result<(), String> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))
}

fn attack(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    for (a, _) in &attacks {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn clues(g: &GameState, seat: usize) -> usize {
    count_named(g, seat, "Clue")
}

// ── Creatures ──────────────────────────────────────────────────────────────

/// Up to two artifacts and/or enchantments exiled on entry; plainscycling.
#[test]
fn angel_of_the_ruins_exiles_two_on_entry() {
    let mut g = main_phase();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let prison = g.add_card_to_battlefield(1, catalog::ghostly_prison());
    let angel = g.add_card_to_hand(0, catalog::angel_of_the_ruins());
    cast(&mut g, angel, None);
    assert!(g.exile.iter().any(|c| c.id == ring) && g.exile.iter().any(|c| c.id == prison));
    assert!(catalog::angel_of_the_ruins().keywords.iter().any(|k| matches!(k, Keyword::Landcycling(..))));
}

/// CR 207.2c — lieutenant turns on only while you control your commander.
#[test]
fn angelic_field_marshal_is_a_lieutenant() {
    let mut g = main_phase();
    let marshal = g.add_card_to_battlefield(0, catalog::angelic_field_marshal());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, marshal), (3, 3));
    let giada = g.add_card_to_battlefield(0, catalog::giada_font_of_hope());
    g.players[0].commanders.push(giada);
    assert_eq!(pt(&g, marshal), (5, 5));
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Vigilance));
}

/// CR 603.10a — the counters are read from the departed permanent.
#[test]
fn angelic_sleuth_investigates_off_a_permanent_that_had_counters() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::angelic_sleuth());
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let grown = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(grown).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    for id in [plain, grown] {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, bolt, Some(Target::Permanent(id)));
    }
    assert_eq!(clues(&g, 0), 1, "only the one with a counter");
}

/// Two or more attackers draw you a card; another player's attack with two
/// draws THEM one only if none attacked you.
#[test]
fn firemane_commando_rewards_wide_attacks() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.add_card_to_battlefield(0, catalog::firemane_commando());
    for s in 0..3 {
        for _ in 0..3 {
            g.add_card_to_library(s, catalog::plains());
        }
    }
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack(&mut g, vec![(a, 1), (b, 2)]);
    assert_eq!(g.players[0].hand.len(), 1, "you attacked with two");

    let run = |targets: [usize; 2]| {
        let mut g = multi_player_game(3);
        g.add_card_to_battlefield(0, catalog::firemane_commando());
        g.add_card_to_library(1, catalog::plains());
        let ids: Vec<_> = (0..2).map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears())).collect();
        g.active_player_idx = 1;
        attack(&mut g, ids.into_iter().zip(targets).collect());
        g.players[1].hand.len()
    };
    assert_eq!(run([2, 2]), 1, "none attacked Firemane's controller: they draw");
    assert_eq!(run([0, 2]), 0, "one attacked you: no card");
}

/// Grows as it attacks; each counter shaves {1} off Angel and Human spells.
#[test]
fn herald_of_war_discounts_angels_per_counter() {
    let mut g = main_phase();
    let herald = g.add_card_to_battlefield(0, catalog::herald_of_war());
    attack(&mut g, vec![(herald, 1)]);
    g.battlefield_find_mut(herald).unwrap().tapped = false;
    attack(&mut g, vec![(herald, 1)]);
    assert_eq!(g.battlefield_find(herald).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    let card = catalog::seraph_of_the_sword();
    let id = g.add_card_to_hand(0, card);
    let inst = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
    assert_eq!(crabomination::game::actions::cost_reduction_for_spell(&g, 0, &inst, None), 2);
}

/// A nontoken creature dying investigates, and Clues have exalted (CR
/// 702.83a — one instance per Clue).
#[test]
fn merchant_of_truth_investigates_and_clues_exalt() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::merchant_of_truth());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bears)));
    assert_eq!(clues(&g, 0), 1);
    let lone = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack(&mut g, vec![(lone, 1)]);
    assert_eq!(pt(&g, lone), (3, 3), "one Clue: exalted once");
}

/// You have hexproof (CR 702.11d); damage to it is life gained.
#[test]
fn metropolis_reformer_hexproofs_you_and_banks_damage() {
    let mut g = main_phase();
    let reformer = g.add_card_to_battlefield(0, catalog::metropolis_reformer());
    let spike = g.add_card_to_hand(1, catalog::lava_spike());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    assert!(g.perform_action(cast_action(spike, Some(Target::Player(0)), None)).is_err());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Permanent(reformer)));
    assert_eq!(g.players[0].life, 22);
}

/// The alternative cost: {W} and four untapped fliers tapped. Other fliers
/// are indestructible.
#[test]
fn sephara_casts_off_four_fliers_and_protects_them() {
    let mut g = main_phase();
    let fliers: Vec<_> = (0..4).map(|_| g.add_card_to_battlefield(0, catalog::serra_angel())).collect();
    let sephara = g.add_card_to_hand(0, catalog::sephara_skys_blade());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: sephara,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{W} + four fliers");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sephara).is_some());
    assert!(fliers.iter().all(|f| g.battlefield_find(*f).unwrap().tapped));
    let wrath = g.add_card_to_hand(0, catalog::day_of_judgment());
    cast(&mut g, wrath, None);
    assert!(fliers.iter().all(|f| g.battlefield_find(*f).is_some()), "indestructible");
    assert!(g.battlefield_find(sephara).is_none(), "Sephara itself isn't");
}

/// CR 615 — combat damage to it is prevented; the Hill Giant it blocks
/// bounces off and dies.
#[test]
fn seraph_of_the_sword_takes_no_combat_damage() {
    let mut g = main_phase();
    let seraph = g.add_card_to_battlefield(0, catalog::seraph_of_the_sword());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.active_player_idx = 1;
    attack(&mut g, vec![(giant, 0)]);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![(seraph, giant)])).expect("block");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(seraph).is_some_and(|c| c.damage == 0));
    assert!(g.battlefield_find(giant).is_none(), "the Seraph's own damage still lands");
}

/// Angel spells cost {2} less.
#[test]
fn starnheim_aspirant_discounts_angels() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::starnheim_aspirant());
    let seraph = g.add_card_to_hand(0, catalog::seraph_of_the_sword());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(cast_action(seraph, None, None)).expect("{1}{W}");
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// An Angel comes back with two +1/+1 counters; anything else without.
#[test]
fn defy_death_counters_only_an_angel() {
    let mut g = main_phase();
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    for id in [angel, bears] {
        let spell = g.add_card_to_hand(0, catalog::defy_death());
        cast(&mut g, spell, Some(Target::Permanent(id)));
    }
    assert_eq!(g.battlefield_find(angel).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Mode one needs toughness 4 or greater; mode two takes an enchantment.
#[test]
fn destroy_evil_modes() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::serra_angel());
    let prison = g.add_card_to_battlefield(1, catalog::ghostly_prison());
    let evil = g.add_card_to_hand(0, catalog::destroy_evil());
    flood(&mut g, 0);
    assert!(g.perform_action(cast_action(evil, Some(Target::Permanent(bears)), Some(0))).is_err());
    cast_mode(&mut g, evil, giant, 0);
    let evil = g.add_card_to_hand(0, catalog::destroy_evil());
    cast_mode(&mut g, evil, prison, 1);
    assert!(g.battlefield_find(giant).is_none() && g.battlefield_find(prison).is_none());
}

fn cast_mode(g: &mut GameState, id: CardId, target: CardId, mode: usize) {
    flood(g, 0);
    g.perform_action(cast_action(id, Some(Target::Permanent(target)), Some(mode))).expect("cast");
    drain_stack(g);
}

/// Destroy an artifact or enchantment and gain 4.
#[test]
fn invoke_the_divine_destroys_and_gains() {
    let mut g = main_phase();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let spell = g.add_card_to_hand(0, catalog::invoke_the_divine());
    cast(&mut g, spell, Some(Target::Permanent(ring)));
    assert!(g.battlefield_find(ring).is_none());
    assert_eq!(g.players[0].life, 24);
}

// ── Noncreature permanents ─────────────────────────────────────────────────

/// Monarch on entry (CR 724); the upkeep token is an Angel while you're the
/// monarch and a Spirit otherwise.
#[test]
fn court_of_grace_makes_an_angel_for_the_monarch() {
    let mut g = main_phase();
    let court = g.add_card_to_hand(0, catalog::court_of_grace());
    cast(&mut g, court, None);
    assert_eq!(g.monarch, Some(0));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Angel"), 1);
    g.monarch = Some(1);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Spirit"), 1);
}

/// Draws only with three lands of ONE name.
#[test]
fn endless_atlas_needs_three_lands_with_the_same_name() {
    let mut g = main_phase();
    let atlas = g.add_card_to_battlefield(0, catalog::endless_atlas());
    g.add_card_to_library(0, catalog::plains());
    for land in [catalog::plains(), catalog::plains(), catalog::island(), catalog::island()] {
        g.add_card_to_battlefield(0, land);
    }
    flood(&mut g, 0);
    assert!(activate(&mut g, atlas, 0).is_err(), "two and two is not three");
    g.add_card_to_battlefield(0, catalog::plains());
    activate(&mut g, atlas, 0).expect("three Plains");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// 1 life on entry and per Angel entering under your control.
#[test]
fn seraph_sanctuary_gains_per_angel() {
    let mut g = main_phase();
    let land = g.add_card_to_hand(0, catalog::seraph_sanctuary());
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21);
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    cast(&mut g, angel, None);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bears, None);
    assert_eq!(g.players[0].life, 22, "the Angel, not the bears");
}

/// A page on entry and whenever your commander enters; spend one to draw.
#[test]
fn tome_of_legends_turns_pages_off_your_commander() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::plains());
    let tome = g.add_card_to_hand(0, catalog::tome_of_legends());
    cast(&mut g, tome, None);
    assert_eq!(g.battlefield_find(tome).unwrap().counter_count(CounterType::Page), 1);
    let giada = g.add_card_to_hand(0, catalog::giada_font_of_hope());
    g.players[0].commanders.push(giada);
    cast(&mut g, giada, None);
    assert_eq!(g.battlefield_find(tome).unwrap().counter_count(CounterType::Page), 2);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bears, None);
    assert_eq!(g.battlefield_find(tome).unwrap().counter_count(CounterType::Page), 2, "not a commander");
    let hand = g.players[0].hand.len();
    activate(&mut g, tome, 0).expect("draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert!(catalog::tome_of_legends().subtypes.artifact_subtypes.contains(&ArtifactSubtype::Book));
}
