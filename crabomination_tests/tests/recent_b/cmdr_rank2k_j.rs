//! Commander: two-card gaps of the top-2000 commanders (`decks::cmdr_rank2k_j`).

use crabomination::card::{CardId, CardType, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
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

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None });
}

/// Cryptic Annelid scries three times without touching the library's size.
#[test]
fn cryptic_annelid_scries_thrice() {
    let mut g = pod(2);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let worm = g.add_card_to_hand(0, catalog::cryptic_annelid());
    cast(&mut g, worm, None);
    assert!(g.battlefield_find(worm).is_some());
    assert_eq!(g.players[0].library.len(), 6);
}

/// Devouring Swarm eats a creature to grow.
#[test]
fn devouring_swarm_eats_for_power() {
    let mut g = pod(2);
    let swarm = ready(&mut g, 0, catalog::devouring_swarm());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    act(&mut g, GameAction::ActivateAbility { card_id: swarm, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None });
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.computed_permanent(swarm).unwrap().power, 3);
}

/// Emerge Unscathed protects and rebounds.
#[test]
fn emerge_unscathed_protects_and_rebounds() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::emerge_unscathed());
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords().iter().any(|k| matches!(k, Keyword::Protection(_))));
    assert!(g.exile.iter().any(|c| c.id == spell), "rebound exiles it");
}

/// Giant's Grasp, on a Giant of yours, steals a nonland permanent.
#[test]
fn giants_grasp_steals_while_it_stays() {
    let mut g = pod(2);
    let giant = ready(&mut g, 0, catalog::outland_colossus());
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let grasp = g.add_card_to_hand(0, catalog::giants_grasp());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: grasp,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ring))]));
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(grasp).unwrap().attached_to, Some(giant));
    assert_eq!(g.battlefield_find(ring).unwrap().controller, 0);
}

/// Kraul Harpooner grows by its graveyard and fights a flier.
#[test]
fn kraul_harpooner_shoots_down_a_flier() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let harpooner = ready(&mut g, 0, catalog::kraul_harpooner());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let undergrowth = catalog::kraul_harpooner().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(harpooner, 0, Some(Target::Permanent(angel)));
    g.resolve_effect(&undergrowth, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(angel).is_none(), "5 damage kills the 4/4");
}

/// Liege of the Tangle's awakening counter makes a land an 8/8 green
/// Elemental.
#[test]
fn liege_of_the_tangle_awakens_lands() {
    let mut g = pod(2);
    let liege = ready(&mut g, 0, catalog::liege_of_the_tangle());
    let forest = ready(&mut g, 0, catalog::forest());
    let ability = catalog::liege_of_the_tangle().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(liege, 0, Some(Target::Permanent(forest)));
    g.resolve_effect(&ability, &ctx).unwrap();
    let land = g.battlefield_find(forest).unwrap();
    assert_eq!(land.counter_count(CounterType::Awakening), 1);
    let cp = g.computed_permanent(forest).unwrap();
    assert!(cp.card_types().contains(&CardType::Creature) && cp.card_types().contains(&CardType::Land));
    assert_eq!((cp.power, cp.toughness), (8, 8));
    assert!(cp.colors.contains(Color::Green));
}

/// Mystic Speculation bought back returns to hand.
#[test]
fn mystic_speculation_buys_itself_back() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let spec = g.add_card_to_hand(0, catalog::mystic_speculation());
    act(&mut g, GameAction::CastSpellBuyback { card_id: spec, target: None, additional_targets: vec![], mode: None, x_value: None });
    assert!(g.players[0].hand.iter().any(|c| c.id == spec));
}

/// Outland Colossus can't be double-blocked.
#[test]
fn outland_colossus_takes_one_blocker() {
    let mut g = pod(2);
    let colossus = ready(&mut g, 0, catalog::outland_colossus());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: colossus, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(a, colossus), (b, colossus)])).is_err());
}

/// Pol Jamaar draws one per creature of the named type.
#[test]
fn pol_jamaar_counts_the_named_type() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    ready(&mut g, 0, catalog::silvergill_adept());
    ready(&mut g, 0, catalog::silvergill_adept());
    ready(&mut g, 0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Merfolk)]));
    let pol = g.add_card_to_hand(0, catalog::pol_jamaar_illusionist());
    cast(&mut g, pol, None);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Tatyova's seventh land animates a land, which flies.
#[test]
fn tatyova_animates_at_seven_lands() {
    let mut g = pod(2);
    let tat = ready(&mut g, 0, catalog::tatyova_steward_of_tides());
    let lands: Vec<CardId> = (0..6).map(|_| ready(&mut g, 0, catalog::forest())).collect();
    let trigger = catalog::tatyova_steward_of_tides().triggered_abilities[0].clone();
    let ctx = EffectContext::for_ability(tat, 0, Some(Target::Permanent(lands[0])));
    g.resolve_effect(&trigger.effect, &ctx).unwrap();
    assert!(!g.computed_permanent(lands[0]).unwrap().card_types().contains(&CardType::Creature), "six lands is not enough");
    ready(&mut g, 0, catalog::forest());
    g.resolve_effect(&trigger.effect, &ctx).unwrap();
    let cp = g.computed_permanent(lands[0]).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords().contains(&Keyword::Flying) && cp.keywords().contains(&Keyword::Haste));
}

/// Wings of the Cosmos pumps, gives flying and untaps.
#[test]
fn wings_of_the_cosmos_lifts_and_untaps() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let wings = g.add_card_to_hand(0, catalog::wings_of_the_cosmos());
    cast(&mut g, wings, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 5));
    assert!(cp.keywords().contains(&Keyword::Flying));
    assert!(!g.battlefield_find(bear).unwrap().tapped);
}

/// Winter Eladrin bounces another creature.
#[test]
fn winter_eladrin_gusts_a_creature_home() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let eladrin = g.add_card_to_hand(0, catalog::winter_eladrin());
    cast(&mut g, eladrin, Some(Target::Permanent(bear)));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}
