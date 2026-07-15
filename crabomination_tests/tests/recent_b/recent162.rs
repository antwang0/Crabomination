//! Functionality tests for `catalog::sets::decks::recent162` (Foundations).

use crabomination::catalog;
use crabomination::card::{CounterType, Keyword, WardCost};
use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

/// Felling Blow grows your creature, then that creature swings for its power.
#[test]
fn felling_blow_pumps_and_fights() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → 3/3? no, 2/2
    let id = g.add_card_to_hand(0, catalog::felling_blow());
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(foe)],
        mode: None,
        x_value: None,
    })
    .expect("cast Felling Blow");
    drain_stack(&mut g);
    // Counter made it a 3/3; 3 damage kills the opposing 2/2.
    assert_eq!(g.computed_permanent(mine).map(|c| c.power), Some(3), "+1/+1 counter");
    assert!(g.battlefield_find(foe).is_none(), "took lethal damage equal to power");
}

/// Inspiration from Beyond mills and returns an instant/sorcery.
#[test]
fn inspiration_from_beyond_returns_spell() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::inspiration_from_beyond());
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Inspiration from Beyond");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "returned the instant to hand");
}

/// Sower of Chaos grants can't-block to a target creature.
#[test]
fn sower_of_chaos_grants_cant_block() {
    let mut g = two_player_game();
    let sower = g.add_card_to_battlefield(0, catalog::sower_of_chaos());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sower, ability_index: 0,
        target: Some(Target::Permanent(foe)), additional_targets: vec![], x_value: None,
    })
    .expect("activate can't-block");
    drain_stack(&mut g);
    assert!(g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Searslicer Goblin's Raid mints a Goblin at end step after you attacked.
#[test]
fn searslicer_goblin_raid_token() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::searslicer_goblin());
    g.players[0].attacked_this_turn = true;
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Goblin" && c.controller == 0),
        "Raid created a Goblin token"
    );
}

/// Sire of Seven Deaths is a 7/7 with the full keyword pile and Ward—pay 7 life.
#[test]
fn sire_of_seven_deaths_keywords() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::sire_of_seven_deaths());
    let cp = g.computed_permanent(c).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7));
    for k in [Keyword::FirstStrike, Keyword::Vigilance, Keyword::Menace, Keyword::Trample, Keyword::Reach, Keyword::Lifelink] {
        assert!(cp.keywords.contains(&k), "missing {k:?}");
    }
    assert!(cp.keywords.contains(&Keyword::Ward(WardCost::Life(7))), "Ward—pay 7 life");
}

/// Preposterous Proportions gives your team +10/+10 and vigilance.
#[test]
fn preposterous_proportions_pumps_team() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::preposterous_proportions());
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Preposterous Proportions");
    drain_stack(&mut g);
    for id in [a, b] {
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (12, 12), "+10/+10");
        assert!(cp.keywords.contains(&Keyword::Vigilance), "gained vigilance");
    }
}

/// Slumbering Cerberus won't untap normally, but Morbid untaps it after a death.
#[test]
fn slumbering_cerberus_morbid_untaps() {
    let mut g = two_player_game();
    let dog = g.add_card_to_battlefield(0, catalog::slumbering_cerberus());
    g.battlefield_find_mut(dog).unwrap().tapped = true;
    // A creature died this turn.
    let chump = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_to_graveyard_with_triggers(chump);
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(dog).unwrap().tapped, "Morbid untapped the Cerberus");
}

/// Squad Rallier digs a small creature into hand.
#[test]
fn squad_rallier_digs_creature() {
    let mut g = two_player_game();
    let rallier = g.add_card_to_battlefield(0, catalog::squad_rallier());
    g.add_card_to_library(0, catalog::grizzly_bears()); // power 2 → eligible
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.players[0].mana_pool.add(Color::White, 3);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: rallier, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate dig");
    drain_stack(&mut g);
    assert!(g.players[0].hand.len() > hand, "dug a creature into hand");
}

/// Sphinx of Forgotten Lore grants flashback to a graveyard spell on attack.
#[test]
fn sphinx_grants_flashback_on_attack() {
    let mut g = two_player_game();
    let sphinx = g.add_card_to_battlefield(0, catalog::sphinx_of_forgotten_lore());
    g.clear_sickness(sphinx);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sphinx, target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(
        g.players[0].graveyard.iter().find(|c| c.id == bolt).unwrap().granted_flashback_eot.is_some(),
        "the graveyard spell gained flashback"
    );
}

/// Claws Out pumps your whole team.
#[test]
fn claws_out_pumps_team() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::claws_out());
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Claws Out");
    drain_stack(&mut g);
    let cp = g.computed_permanent(a).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
}

/// Skyknight Squire gains flying once it reaches three +1/+1 counters.
#[test]
fn skyknight_squire_flies_at_three() {
    let mut g = two_player_game();
    let squire = g.add_card_to_battlefield(0, catalog::skyknight_squire());
    g.step = TurnStep::PreCombatMain;
    for _ in 0..3 {
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast a creature to trigger the Squire");
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(squire).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    assert!(g.computed_permanent(squire).unwrap().keywords.contains(&Keyword::Flying), "3 counters → flying");
}

/// Luminous Rebuke destroys a creature, and is cheaper against a tapped one.
#[test]
fn luminous_rebuke_destroys_tapped_cheaply() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::luminous_rebuke());
    // {4}{W} - {3} = {1}{W} against a tapped creature.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Luminous Rebuke for its reduced cost");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "destroyed the tapped creature");
}
