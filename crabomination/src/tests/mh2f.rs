//! Functionality tests for `catalog::sets::decks::mh2f` — MH2 sweep batch 7.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

/// Ruin (the aftermath half) hits for your land count, from the graveyard.
#[test]
fn road_ruin_aftermath() {
    let mut g = two_player_game();
    let d = catalog::road_ruin();
    assert!(d.split.as_ref().unwrap().aftermath, "right half is aftermath");
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let right = d.split.as_ref().unwrap().right.effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(victim)];
    let events = g.resolve_effect(&right, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "3 lands = 3 damage kills the 2/2");
}

/// Ethersworn Sphinx discounts per artifact and cascades on cast.
#[test]
fn ethersworn_sphinx() {
    let mut g = two_player_game();
    for _ in 0..7 {
        g.add_card_to_battlefield(0, catalog::parcel_myr());
    }
    g.add_card_to_library(0, catalog::grizzly_bears()); // cascade hit (MV 2 < 9)
    let sphinx = g.add_card_to_hand(0, catalog::ethersworn_sphinx());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    // Affinity 7 → {W}{U} alone pays it.
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, sphinx);
    assert!(g.battlefield_find(sphinx).is_some(), "affinity-discounted cast");
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "cascade free-cast the bear"
    );
}

/// Blossoming Calm shields you until your next turn.
#[test]
fn blossoming_calm_player_hexproof() {
    let mut g = two_player_game();
    assert!(catalog::blossoming_calm().keywords.contains(&Keyword::Rebound));
    let life = g.players[0].life;
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&catalog::blossoming_calm().effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    assert_eq!(g.players[0].life, life + 2);
    assert!(g.players[0].hexproof_until_next_turn);
    // An opponent's targeted burn can't aim at the shielded player.
    let bolt = g.add_card_to_hand(1, catalog::slaying_fire());
    g.players[1].mana_pool.add(Color::Red, 3);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "hexproof player can't be targeted"
    );
    // The shield expires at player 0's untap.
    g.active_player_idx = 0;
    g.do_untap();
    assert!(!g.players[0].hexproof_until_next_turn, "expires at your untap");
}

/// Foundry Helix gains 4 only off an artifact sacrifice.
#[test]
fn foundry_helix_artifact_rider() {
    let mut g = two_player_game();
    let relic = g.add_card_to_battlefield(0, catalog::parcel_myr());
    let helix = g.add_card_to_hand(0, catalog::foundry_helix());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let my_life = g.players[0].life;
    let opp_life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.pending_cast_sacrifices = Some(vec![relic]);
    g.perform_action(GameAction::CastSpell {
        card_id: helix,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast with artifact fodder");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 4, "4 damage");
    assert_eq!(g.players[0].life, my_life + 4, "artifact fodder: gain 4");

    // Nonartifact fodder: no lifegain.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let helix2 = g.add_card_to_hand(0, catalog::foundry_helix());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let my_life = g.players[0].life;
    g.pending_cast_sacrifices = Some(vec![bear]);
    g.perform_action(GameAction::CastSpell {
        card_id: helix2,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast with creature fodder");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, my_life, "creature fodder: no gain");
}

/// Diamond Lion converts hand + body into three mana of one color.
#[test]
fn diamond_lion_mana_burst() {
    let mut g = two_player_game();
    let lion = g.add_card_to_battlefield(0, catalog::diamond_lion());
    g.clear_sickness(lion);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::island());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: lion, ability_index: 0, target: None, additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3, "three mana added");
    assert!(g.players[0].hand.is_empty(), "hand discarded");
    assert!(g.battlefield_find(lion).is_none(), "lion sacrificed");
}

/// Deepwood Denizen's draw discounts by team counters.
#[test]
fn deepwood_denizen_counter_discount() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(0, catalog::deepwood_denizen());
    g.clear_sickness(elf);
    g.add_card_to_library(0, catalog::island());
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(buddy).unwrap().add_counters(CounterType::PlusOnePlusOne, 5);
    // {5}{G} − 5 = {G}: one green mana suffices.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: elf, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("discounted draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "drew");
}

/// Mount Velus Manticore lobs the discarded card's type count.
#[test]
fn mount_velus_manticore_lob() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mount_velus_manticore());
    // Artifact creature = two card types.
    g.add_card_to_hand(0, catalog::parcel_myr());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let opp_life = g.players[1].life;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2, "two card types = 2 damage");
}

/// Breathless Knight grows off graveyard arrivals but not hand casts.
#[test]
fn breathless_knight_gy_watch() {
    let mut g = two_player_game();
    let knight = g.add_card_to_battlefield(0, catalog::breathless_knight());
    // A plain hand cast: no counter.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.priority.player_with_priority = 0;
    cast(&mut g, bear);
    assert_eq!(
        g.battlefield_find(knight).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0,
        "hand cast: no growth"
    );
    // A reanimation: counter.
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let mut ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(dead)];
    let events = g
        .resolve_effect(
            &crate::effect::Effect::Move {
                what: crate::effect::Selector::Target(0),
                to: crate::effect::ZoneDest::Battlefield {
                    controller: crate::effect::PlayerRef::You,
                    tapped: false,
                },
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(knight).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "graveyard arrival: +1/+1"
    );
}
