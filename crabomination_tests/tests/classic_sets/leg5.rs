//! Legends (LEG) wave 6 — the block-and-kill creatures, the prevention
//! bodies and the set's remaining legends (`catalog::sets::leg5`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
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

/// Seat 0's `attacker` swings at seat 1 and `blocker` blocks.
fn block(g: &mut GameState, attacker: CardId, blocker: CardId) {
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    drain_stack(g);
}

fn to_end_of_combat(g: &mut GameState) {
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

fn to_upkeep(g: &mut GameState) {
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

/// Abomination kills a green blocker at end of combat, not on impact.
#[test]
fn abomination_kills_its_green_blocker_at_end_of_combat() {
    let mut g = main_phase();
    let abom = g.add_card_to_battlefield(0, catalog::abomination());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    block(&mut g, abom, bear);
    assert!(g.battlefield_find(bear).is_some(), "not yet");
    to_end_of_combat(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// A blue blocker walks away — the trigger is colour-gated.
#[test]
fn abomination_ignores_an_off_colour_blocker() {
    let mut g = main_phase();
    let abom = g.add_card_to_battlefield(0, catalog::abomination());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_vapor());
    block(&mut g, abom, wall);
    to_end_of_combat(&mut g);
    assert!(g.battlefield_find(wall).is_some(), "blue survives");
}

/// Infernal Medusa spares a Wall that blocks it, but not a real creature.
#[test]
fn infernal_medusa_spares_walls_that_block_it() {
    let mut g = main_phase();
    let medusa = g.add_card_to_battlefield(0, catalog::infernal_medusa());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_heat());
    block(&mut g, medusa, wall);
    to_end_of_combat(&mut g);
    assert!(g.battlefield_find(wall).is_some());
}

/// Aisling Leprechaun repaints its blocker for good.
#[test]
fn aisling_leprechaun_turns_its_blocker_green() {
    let mut g = main_phase();
    let lions = g.add_card_to_battlefield(0, catalog::savannah_lions());
    let fae = g.add_card_to_battlefield(1, catalog::aisling_leprechaun());
    block(&mut g, lions, fae);
    let colors = g.computed_permanent(lions).unwrap().colors.clone();
    assert_eq!(colors, vec![Color::Green]);
}

/// CR 615 — Enchanted Being takes nothing from an enchanted creature.
#[test]
fn enchanted_being_shrugs_off_enchanted_creatures() {
    let mut g = main_phase();
    let being = g.add_card_to_battlefield(1, catalog::enchanted_being());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let aura = g.add_card_to_hand(0, catalog::spirit_link());
    cast(&mut g, 0, aura, Some(Target::Permanent(giant)));
    block(&mut g, giant, being);
    to_end_of_combat(&mut g);
    assert_eq!(g.battlefield_find(being).unwrap().damage, 0);
}

/// The same body still takes damage from an unenchanted attacker.
#[test]
fn enchanted_being_still_takes_ordinary_damage() {
    let mut g = main_phase();
    let being = g.add_card_to_battlefield(1, catalog::enchanted_being());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    block(&mut g, giant, being);
    to_end_of_combat(&mut g);
    assert!(g.battlefield_find(being).is_none(), "3 damage kills the 2/2");
}

/// Clergy of the Holy Nimbus regenerates for free — until an opponent pays.
#[test]
fn clergy_regenerates_until_an_opponent_pays() {
    let mut g = main_phase();
    let clergy = g.add_card_to_battlefield(0, catalog::clergy_of_the_holy_nimbus());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: clergy,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free regenerate");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(clergy)));
    assert!(g.battlefield_find(clergy).is_some(), "regenerated");

    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: clergy,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("opponents only — seat 1 may activate");
    drain_stack(&mut g);
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt2, Some(Target::Permanent(clergy)));
    assert!(g.battlefield_find(clergy).is_none());
}

/// Firestorm Phoenix bounces instead of dying.
#[test]
fn firestorm_phoenix_returns_to_hand_instead_of_dying() {
    let mut g = main_phase();
    let bird = g.add_card_to_battlefield(0, catalog::firestorm_phoenix());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(bird)));
    assert!(g.battlefield_find(bird).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Firestorm Phoenix"));
    assert!(g.players[0].graveyard.iter().all(|c| c.definition.name != "Firestorm Phoenix"));
}

/// The Wretched keeps everything that blocked it.
#[test]
fn the_wretched_steals_its_blockers() {
    let mut g = main_phase();
    let wretched = g.add_card_to_battlefield(0, catalog::the_wretched());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_heat());
    block(&mut g, wretched, wall);
    to_end_of_combat(&mut g);
    assert_eq!(g.battlefield_find(wall).unwrap().controller, 0);
}

/// Elder Spawn eats an Island each upkeep, or takes six off you.
#[test]
fn elder_spawn_burns_you_when_it_goes_unfed() {
    let mut g = main_phase();
    let spawn = g.add_card_to_battlefield(0, catalog::elder_spawn());
    to_upkeep(&mut g);
    assert!(g.battlefield_find(spawn).is_none());
    assert_eq!(g.players[0].life, 14);
}

/// Wood Elemental arrives as big as the Forests it ate.
#[test]
fn wood_elemental_is_as_big_as_its_forests() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    let forests: Vec<CardId> =
        (0..3).map(|_| g.add_card_to_battlefield(0, catalog::forest())).collect();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(3)]));
    let elemental = g.add_card_to_hand(0, catalog::wood_elemental());
    cast(&mut g, 0, elemental, None);
    let id = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Wood Elemental")
        .map(|c| c.id)
        .expect("resolved");
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(forests.iter().all(|f| g.battlefield_find(*f).is_none()));
}

/// Divine Intervention counts down two upkeeps and then draws the game.
#[test]
fn divine_intervention_draws_the_game() {
    let mut g = main_phase();
    let di = g.add_card_to_battlefield_with_counters(0, catalog::divine_intervention());
    assert_eq!(g.battlefield_find(di).unwrap().counter_count(CounterType::Intervention), 2);
    to_upkeep(&mut g);
    assert!(!g.is_game_over(), "one counter left");
    g.step = TurnStep::PreCombatMain;
    to_upkeep(&mut g);
    assert_eq!(g.game_over, Some(None), "the game is a draw");
}

/// Urborg's second mode strips swampwalk.
#[test]
fn urborg_strips_swampwalk() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::urborg());
    let walker = g.add_card_to_battlefield(0, catalog::solkanar_the_swamp_king());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 1,
        target: Some(Target::Permanent(walker)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(
        !g.computed_permanent(walker)
            .unwrap()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::Landwalk(_)))
    );
}

/// Alchor's Tomb repaints one of your permanents for good.
#[test]
fn alchors_tomb_repaints_a_permanent() {
    let mut g = main_phase();
    let tomb = g.add_card_to_battlefield(0, catalog::alchors_tomb());
    g.clear_sickness(tomb);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tomb,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_ne!(g.computed_permanent(bear).unwrap().colors, vec![Color::Green]);
}

/// Rohgahh pumps the Kobolds he keeps.
#[test]
fn rohgahh_pumps_his_kobolds() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::rohgahh_of_kher_keep());
    let kobold = g.add_card_to_battlefield(0, catalog::kobolds_of_kher_keep());
    let cp = g.computed_permanent(kobold).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3));
}

/// Stangg brings a twin along.
#[test]
fn stangg_arrives_with_his_twin() {
    let mut g = main_phase();
    let stangg = g.add_card_to_hand(0, catalog::stangg());
    cast(&mut g, 0, stangg, None);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Stangg Twin"));
}
