//! Weatherlight (WTH) — `catalog::sets::wth`.

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

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

fn activate(
    g: &mut GameState,
    id: CardId,
    index: usize,
    target: Option<Target>,
) -> Result<(), crabomination::game::GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
}

fn cast(
    g: &mut GameState,
    id: CardId,
    target: Option<Target>,
) -> Result<(), crabomination::game::GameError> {
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
}

/// Abyssal Gatekeeper's death is a symmetric edict.
#[test]
fn abyssal_gatekeeper_edicts_the_table_on_death() {
    let mut g = two_player_game();
    let keeper = g.add_card_to_battlefield(0, catalog::abyssal_gatekeeper());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut events = Vec::new();
    g.destroy_permanent(keeper, false, &mut events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(mine).is_none());
}

/// Cinder Giant scorches its own team but never itself.
#[test]
fn cinder_giant_spares_only_itself() {
    let mut g = two_player_game();
    let giant = ready(&mut g, 0, catalog::cinder_giant());
    let friend = ready(&mut g, 0, catalog::grizzly_bears());
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    let upkeep = catalog::cinder_giant().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(giant, 0, None);
    g.resolve_effect(&upkeep, &ctx).expect("upkeep");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_some());
    assert!(g.battlefield_find(friend).is_none());
    assert!(g.battlefield_find(foe).is_some(), "only your own board burns");
}

/// Cinder Wall burns out at end of combat once it blocks.
#[test]
fn cinder_wall_dies_after_one_block() {
    let mut g = two_player_game();
    let wall = ready(&mut g, 0, catalog::cinder_wall());
    let attacker = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, attacker)])).expect("block");
    advance_to(&mut g, TurnStep::End);
    assert!(g.battlefield_find(wall).is_none());
}

/// Bubble Matrix shuts damage off for every creature, not just yours.
#[test]
fn bubble_matrix_protects_both_boards() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bubble_matrix());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(theirs))).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(theirs).unwrap().damage, 0);
    assert!(g.battlefield_find(mine).is_some());
}

/// Dingus Staff bills the dying creature's controller.
#[test]
fn dingus_staff_bills_the_owner_of_the_dead() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dingus_staff());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("bolt it");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 20);
}

/// Empyrial Armor scales with the cards in your hand.
#[test]
fn empyrial_armor_scales_with_your_hand() {
    let mut g = two_player_game();
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let armor = g.add_card_to_battlefield(0, catalog::empyrial_armor());
    g.battlefield_find_mut(armor).unwrap().attached_to = Some(bear);
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Familiar Ground stops gang blocks on your side only.
#[test]
fn familiar_ground_only_helps_your_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::familiar_ground());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    assert!(
        g.computed_permanent(mine)
            .unwrap()
            .keywords
            .contains(&Keyword::CantBeBlockedByMoreThanOne)
    );
    assert!(
        !g.computed_permanent(theirs)
            .unwrap()
            .keywords
            .contains(&Keyword::CantBeBlockedByMoreThanOne)
    );
}

/// Fatal Blow only finishes a creature that's already been hit.
#[test]
fn fatal_blow_needs_prior_damage() {
    let mut g = two_player_game();
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let blow = g.add_card_to_hand(0, catalog::fatal_blow());
    g.players[0].mana_pool.add(Color::Black, 1);
    assert!(cast(&mut g, blow, Some(Target::Permanent(bear))).is_err(), "undamaged");

    g.battlefield_find_mut(bear).unwrap().damage = 1;
    g.battlefield_find_mut(bear).unwrap().dealt_damage_this_turn = true;
    cast(&mut g, blow, Some(Target::Permanent(bear))).expect("finish it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// Abjure needs a blue permanent to feed it.
#[test]
fn abjure_eats_a_blue_permanent() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let abjure = g.add_card_to_hand(0, catalog::abjure());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    g.players[0].mana_pool.add(Color::Blue, 1);
    assert!(
        cast(&mut g, abjure, Some(Target::Permanent(bolt))).is_err(),
        "no blue permanent to sacrifice"
    );

    // A land is colorless — it has to be an actually blue permanent.
    let djinn = g.add_card_to_battlefield(0, catalog::cloud_djinn());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, abjure, Some(Target::Permanent(bolt))).expect("counter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20);
    assert!(g.battlefield_find(djinn).is_none(), "the Djinn was the cost");
}

/// Jangling Automaton unlocks the defender's whole board when it attacks.
#[test]
fn jangling_automaton_untaps_the_defenders() {
    let mut g = two_player_game();
    let automaton = ready(&mut g, 0, catalog::jangling_automaton());
    let blocker = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(blocker).unwrap().tapped = true;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: automaton,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(blocker).unwrap().tapped);
}

/// Downdraft grounds a flier, then sweeps the rest when cashed in.
#[test]
fn downdraft_grounds_then_sweeps() {
    let mut g = two_player_game();
    let downdraft = ready(&mut g, 0, catalog::downdraft());
    let flier = ready(&mut g, 1, catalog::cloud_djinn());
    let grounded = ready(&mut g, 1, catalog::fledgling_djinn());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 1);
    activate(&mut g, downdraft, 0, Some(Target::Permanent(flier))).expect("ground it");
    drain_stack(&mut g);
    assert!(!g.computed_permanent(flier).unwrap().keywords.contains(&Keyword::Flying));

    activate(&mut g, downdraft, 1, None).expect("sweep");
    drain_stack(&mut g);
    assert!(g.battlefield_find(grounded).is_none(), "the 2/2 flier died");
    assert!(g.battlefield_find(flier).is_some(), "the grounded one was spared");
}
