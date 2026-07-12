//! Functionality tests for `catalog::sets::decks::recent172`.

use crate::card::Keyword;
use crate::catalog;
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;

/// Starting Column's max-speed sac draws two and discards one.
#[test]
fn starting_column_max_speed_sac_draws() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let col = g.add_card_to_battlefield(0, catalog::starting_column());
    g.clear_sickness(col);
    let a = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![a])]));
    g.players[0].speed = 4;
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: col, ability_index: 1, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("max-speed sac");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew two, discarded one");
    assert!(g.battlefield_find(col).is_none(), "sacrificed");
}

/// Haunted Hellride buffs and untaps a creature when you attack.
#[test]
fn haunted_hellride_attack_trigger() {
    let mut g = two_player_game();
    let hellride = g.add_card_to_battlefield(0, catalog::haunted_hellride());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Animate the Vehicle so it can be the attacker.
    let ts = g.next_timestamp();
    g.add_continuous_effect(crate::game::layers::ContinuousEffect {
        timestamp: ts,
        source: hellride,
        affected: crate::game::layers::AffectedPermanents::Specific(vec![hellride]),
        layer: crate::game::layers::Layer::L4Type,
        sublayer: None,
        duration: crate::game::layers::EffectDuration::UntilEndOfTurn,
        modification: crate::game::layers::Modification::AddCardType(crate::card::CardType::Creature),
    });
    g.clear_sickness(hellride);
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hellride, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let s = g.battlefield_find(bear).unwrap();
    assert!(!s.tapped, "the bear was untapped");
    assert_eq!(s.power(), 3, "+1/+0");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch));
}

/// Unswerving Sloth untaps your team and gains indestructible on a saddled
/// attack.
#[test]
fn unswerving_sloth_saddled_attack() {
    let mut g = two_player_game();
    let sloth = g.add_card_to_battlefield(0, catalog::unswerving_sloth());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(sloth);
    g.battlefield_find_mut(sloth).unwrap().saddled = true;
    g.battlefield_find_mut(other).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sloth, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(other).unwrap().tapped, "team untapped");
    assert!(g.computed_permanent(sloth).unwrap().keywords.contains(&Keyword::Indestructible));
}

/// Thundering Broodwagon destroys a low-MV opposing permanent on ETB.
#[test]
fn thundering_broodwagon_etb_destroys_low_mv() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let big = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // MV 6
    g.move_card_to_battlefield_for_test(0, catalog::thundering_broodwagon());
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_none(), "MV-2 creature destroyed");
    assert!(g.battlefield_find(big).is_some(), "high-MV permanent untouched");
}

/// Tune Up returns an artifact from the graveyard to the battlefield.
#[test]
fn tune_up_reanimates_artifact() {
    let mut g = two_player_game();
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let spell = g.add_card_to_hand(0, catalog::tune_up());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(spell, Some(Target::Permanent(ring)), vec![], None, None).expect("cast Tune Up");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == ring), "Sol Ring back on the battlefield");
}
