//! `decks::recent327` — the cards that were each blocked on one primitive.

use crabomination::card::{CardDefinition, CounterType, CreatureType, SelectionRequirement as R};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
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
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn etb(g: &mut GameState, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(0, def);
    g.fire_self_etb_triggers(id, 0);
    drain_stack(g);
    id
}

fn cast(g: &mut GameState, card_id: CardId, target: Option<Target>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// Cursed Recording banks a time counter per instant/sorcery and blows up at
/// the seventh.
#[test]
fn cursed_recording_counts_to_seven_then_burns_you() {
    let mut g = main_phase();
    g.players[0].life = 40;
    g.players[1].life = 200;
    let rec = etb(&mut g, catalog::cursed_recording());
    for _ in 0..6 {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, bolt, Some(Target::Player(1)));
    }
    assert_eq!(g.battlefield_find(rec).unwrap().counter_count(CounterType::Time), 6);
    assert_eq!(g.players[0].life, 40, "no damage yet");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.battlefield_find(rec).unwrap().counter_count(CounterType::Time), 0, "reset");
    assert_eq!(g.players[0].life, 20, "20 to the face");
}

/// Its {T} copies the next instant or sorcery you cast this turn.
#[test]
fn cursed_recording_taps_to_copy_your_next_spell() {
    let mut g = main_phase();
    let rec = etb(&mut g, catalog::cursed_recording());
    g.clear_sickness(rec);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rec,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("arm the copy");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 14, "the bolt resolved twice");
}

/// Leyline of Resonance copies a pump that targets one of your creatures, and
/// leaves a removal spell aimed at an opponent's creature alone.
#[test]
fn leyline_of_resonance_copies_only_single_friendly_creature_spells() {
    let mut g = main_phase();
    etb(&mut g, catalog::leyline_of_resonance());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    cast(&mut g, growth, Some(Target::Permanent(mine)));
    assert_eq!(g.computed_permanent(mine).unwrap().power, 8, "2 + 3 + 3 — copied");

    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(theirs)));
    g.check_state_based_actions();
    assert_eq!(g.players[1].life, 20, "an opponent's creature isn't a copy trigger");
}

/// Leyline of Transformation's chosen type reaches creature cards in hand and
/// graveyard, not just the battlefield.
#[test]
fn leyline_of_transformation_types_cards_outside_the_battlefield() {
    let mut g = main_phase();
    let leyline = g.add_card_to_battlefield(0, catalog::leyline_of_transformation());
    g.battlefield_find_mut(leyline).unwrap().chosen_creature_type = Some(CreatureType::Zombie);
    let in_hand = g.add_card_to_hand(0, catalog::grizzly_bears());
    let in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let zombie = R::HasCreatureType(CreatureType::Zombie);
    for (id, where_) in [(in_hand, "hand"), (in_gy, "graveyard")] {
        let card = g.find_card_anywhere(id).unwrap().clone();
        assert!(
            g.evaluate_requirement_on_card(&zombie, &card, 0),
            "the {where_} card picked up the chosen type"
        );
    }
    let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let card = g.find_card_anywhere(theirs).unwrap().clone();
    assert!(!g.evaluate_requirement_on_card(&zombie, &card, 0), "only cards you own");
}

/// Hedge Shredder's mill drops any land it hits onto the battlefield tapped.
#[test]
fn hedge_shredder_deploys_the_lands_it_mills() {
    let mut g = main_phase();
    let shredder = g.add_card_to_battlefield(0, catalog::hedge_shredder());
    let land = g.add_card_to_library(0, catalog::mountain());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let crew = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::Crew { vehicle: shredder, crew_creatures: vec![crew] })
        .expect("crew 1");
    g.clear_sickness(shredder);
    g.step = TurnStep::DeclareAttackers;
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.declare_attackers(vec![Attack { attacker: shredder, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    let deployed = g.battlefield_find(land).expect("the milled land entered play");
    assert!(deployed.tapped, "tapped");
}

/// Undead Sprinter can only be recast from the graveyard once a non-Zombie
/// creature has died, and enters with a counter when it is.
#[test]
fn undead_sprinter_needs_a_non_zombie_death() {
    let mut g = main_phase();
    let sprinter = g.add_card_to_graveyard(0, catalog::undead_sprinter());
    flood(&mut g, 0);
    let recast = |g: &mut GameState| {
        g.perform_action(GameAction::CastFlashback {
            card_id: sprinter,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(recast(&mut g).is_err(), "nothing has died");

    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.destroy_permanent(bear, false, &mut Vec::new());
    drain_stack(&mut g);
    flood(&mut g, 0);
    recast(&mut g).expect("a non-Zombie died");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(sprinter).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "cast this way, it enters with a +1/+1 counter"
    );
}
