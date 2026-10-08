//! Commander: most-built commanders seated from their EDHREC average decks
//! (`decks::cmdr_most_built`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn activate(g: &mut GameState, id: CardId, index: usize) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

fn plus_ones(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// CR 603.2c — Ob Nixilis: three opponents each losing 1 life at once (Mount
/// Doom's ping) is ONE batch, so one trigger: one counter, one card exiled.
#[test]
fn ob_nixilis_triggers_once_for_a_batch_of_single_losses() {
    let mut g = pod(4);
    let ob = ready(&mut g, 0, catalog::ob_nixilis_captive_kingpin());
    let doom = ready(&mut g, 0, catalog::mount_doom());
    let library = g.players[0].library.len();
    activate(&mut g, doom, 1);
    assert_eq!([g.players[1].life, g.players[2].life, g.players[3].life], [19, 19, 19]);
    assert_eq!(plus_ones(&g, ob), 1);
    assert_eq!(g.players[0].library.len(), library - 1, "one impulse-exiled card");
}

/// Ruling 2023-05-12 — combat damage to a player is read in total: two 1/1s
/// connecting with one opponent is a loss of 2, so no trigger; one 1/1 on
/// each of two opponents is a loss of exactly 1 each, one trigger.
#[test]
fn ob_nixilis_reads_a_players_total_combat_loss() {
    let mut g = pod(3);
    let ob = ready(&mut g, 0, catalog::ob_nixilis_captive_kingpin());
    let a = ready(&mut g, 0, catalog::memnite());
    let b = ready(&mut g, 0, catalog::memnite());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    advance_to(&mut g, TurnStep::EndCombat);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(plus_ones(&g, ob), 0, "a loss of 2 is not exactly 1");

    let mut g = pod(3);
    let ob = ready(&mut g, 0, catalog::ob_nixilis_captive_kingpin());
    let a = ready(&mut g, 0, catalog::memnite());
    let b = ready(&mut g, 0, catalog::memnite());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    advance_to(&mut g, TurnStep::EndCombat);
    drain_stack(&mut g);
    assert_eq!([g.players[1].life, g.players[2].life], [19, 19]);
    assert_eq!(plus_ones(&g, ob), 1, "both opponents lost exactly 1 in one batch");
}

/// Mount Doom's last ability: the chosen two survive (Ob Nixilis and the
/// costliest by the headless pick), every other creature is destroyed, and
/// the land and a legendary artifact are the cost.
#[test]
fn mount_doom_keeps_up_to_two_creatures_and_destroys_the_rest() {
    let mut g = pod(3);
    let doom = ready(&mut g, 0, catalog::mount_doom());
    let mox = ready(&mut g, 0, catalog::mox_amber());
    for seat in 0..3 {
        ready(&mut g, seat, catalog::grizzly_bears());
        ready(&mut g, seat, catalog::grizzly_bears());
    }
    activate(&mut g, doom, 2);
    let creatures = g.battlefield.iter().filter(|c| c.definition.is_creature()).count();
    assert_eq!(creatures, 2);
    assert!(g.battlefield_find(doom).is_none() && g.battlefield_find(mox).is_none());
}

/// Manabarbs — each land tapped for mana is its own 1 damage to the player
/// who tapped it (2009-10-01 rulings), whoever controls the enchantment.
#[test]
fn manabarbs_burns_whoever_taps_a_land() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::manabarbs());
    let m1 = ready(&mut g, 1, catalog::mountain());
    let m2 = ready(&mut g, 1, catalog::mountain());
    for land in [m1, m2] {
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("tap for mana");
    }
    g.priority.player_with_priority = 1;
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 20);
}

/// Shadow of the Goblin — a land PLAYED from the graveyard (Crucible of
/// Worlds) pings each opponent; one played from hand does not.
#[test]
fn shadow_of_the_goblin_pings_for_a_land_played_from_elsewhere() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::shadow_of_the_goblin());
    ready(&mut g, 0, catalog::crucible_of_worlds());
    let from_hand = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(from_hand)).expect("play from hand");
    drain_stack(&mut g);
    assert_eq!([g.players[1].life, g.players[2].life], [20, 20]);
    g.players[0].lands_played_this_turn = 0;
    let from_yard = g.add_card_to_graveyard(0, catalog::mountain());
    g.perform_action(GameAction::PlayLandFromGraveyard(from_yard)).expect("play from graveyard");
    drain_stack(&mut g);
    assert_eq!([g.players[1].life, g.players[2].life], [19, 19]);
}

fn cast_at(g: &mut GameState, id: CardId, target: Option<crabomination::game::types::Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

/// CR 702.40 — Storm's combat damage gives the NEXT instant or sorcery this
/// turn storm: a Bolt after one earlier spell is copied once; the one after
/// it is not (the grant is spent).
#[test]
fn storm_gives_the_next_instant_or_sorcery_storm() {
    use crabomination::game::types::Target;
    let mut g = pod(3);
    let storm = ready(&mut g, 0, catalog::storm_force_of_nature());
    flood(&mut g);
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast_at(&mut g, opt, None);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: storm, target: AttackTarget::Player(2) }]))
        .expect("attack");
    advance_to(&mut g, TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[2].life, 17);
    flood(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 14, "the Bolt and one storm copy");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 11, "the grant was spent");
}

/// Ashling's Magecraft: the second resolution this turn deals 2 to each
/// opponent and each creature they control; the third adds {R}{R}{R}{R}.
#[test]
fn ashling_escalates_on_the_second_and_third_spell() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::ashling_flame_dancer());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    for i in 0..3 {
        for _ in 0..2 {
            g.add_card_to_hand(0, catalog::island());
        }
        let opt = g.add_card_to_hand(0, catalog::opt());
        flood(&mut g);
        let red = g.players[0].mana_pool.amount(Color::Red);
        cast_at(&mut g, opt, None);
        match i {
            0 => assert_eq!(g.players[1].life, 20),
            1 => {
                assert_eq!([g.players[1].life, g.players[2].life], [18, 18]);
                assert!(g.battlefield_find(bear).is_none(), "their creature took 2");
                assert!(g.battlefield_find(mine).is_some(), "yours did not");
            }
            _ => assert_eq!(g.players[0].mana_pool.amount(Color::Red), red + 4),
        }
    }
}

/// Electro: an instant or sorcery cast adds {R}; leaving the battlefield it
/// may pay {X} for a reflexive X damage to target player (2025-09-19 ruling).
#[test]
fn electro_adds_red_and_burns_as_it_leaves() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let electro = ready(&mut g, 0, catalog::electro_assaulting_battery());
    let opt = g.add_card_to_hand(0, catalog::opt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast_at(&mut g, opt, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "Electro's red mana");
    g.players[0].mana_pool.add_colorless(3);
    // The bots' profile: a hostile player slot names an opponent first.
    g.players[0].hostile_player_targets = true;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(3)]));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, Some(Target::Permanent(electro)));
    assert!(g.battlefield_find(electro).is_none());
    assert_eq!(g.players[1].life, 17, "paid {{X}} = 3 for 3 damage");
}

fn shock_it(g: &mut GameState, id: CardId) {
    flood(g);
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast_at(g, shock, Some(crabomination::game::types::Target::Permanent(id)));
}

/// CR 702.54 — Indoraptor's bloodthirst X counts all damage dealt to your
/// opponents this turn (2023-11-10 ruling), summed across opponents.
#[test]
fn indoraptor_enters_with_the_turns_damage_to_opponents() {
    let mut g = pod(3);
    g.players[1].damage_taken_this_turn = 2;
    g.players[2].damage_taken_this_turn = 3;
    let raptor = g.add_card_to_hand(0, catalog::indoraptor_the_perfect_hybrid());
    flood(&mut g);
    cast_at(&mut g, raptor, None);
    assert_eq!(plus_ones(&g, raptor), 5);
}

/// Indoraptor's enrage: the random opponent with no nontoken creature to
/// sacrifice is dealt damage equal to its power.
#[test]
fn indoraptor_enrage_punishes_a_random_opponent() {
    let mut g = pod(2);
    let raptor = ready(&mut g, 0, catalog::indoraptor_the_perfect_hybrid());
    g.battlefield_find_mut(raptor).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    shock_it(&mut g, raptor);
    assert!(g.battlefield_find(raptor).is_some(), "a 6/4 survives the Shock");
    assert_eq!(g.players[1].life, 14, "6 damage to the only opponent");
}

/// Polyraptor's enrage copies it; Silverclad Ferocidons' makes each opponent
/// sacrifice a permanent.
#[test]
fn polyraptor_copies_itself_and_ferocidons_eats_a_permanent() {
    let mut g = pod(3);
    let poly = ready(&mut g, 0, catalog::polyraptor());
    shock_it(&mut g, poly);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Polyraptor").count(), 2);

    let mut g = pod(3);
    let fero = ready(&mut g, 0, catalog::silverclad_ferocidons());
    ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 2, catalog::mountain());
    shock_it(&mut g, fero);
    assert!(g.battlefield.iter().all(|c| c.controller == 0), "each opponent lost its one permanent");
}

/// Forerunner of the Empire: a Dinosaur of yours entering may have it deal 1
/// damage to each creature.
#[test]
fn forerunner_of_the_empire_pings_each_creature_when_a_dinosaur_enters() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::forerunner_of_the_empire());
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let dino = g.add_card_to_hand(0, catalog::polyraptor());
    flood(&mut g);
    cast_at(&mut g, dino, None);
    assert!(g.battlefield_find(elf).is_none(), "the 1/1 took 1");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Polyraptor").count(), 2, "and Polyraptor copied itself");
}
