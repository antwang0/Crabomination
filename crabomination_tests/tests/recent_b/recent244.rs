//! Functionality tests for `catalog::sets::decks::recent244` (MKM batch).

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game, GameEvent};
use crabomination::mana::Color;

fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
}

/// Vitu-Ghazi Inspector rewards a collected evidence: +1/+1 on a creature and 2
/// life. Without evidence, its ETB does nothing.
#[test]
fn vitu_ghazi_inspector_rewards_collected_evidence() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 6 to collect
    }
    let spell = g.add_card_to_hand(0, catalog::vitu_ghazi_inspector());
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Vitu-Ghazi Inspector collecting evidence");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained 2 life with evidence");
    let counters: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0))
        .sum();
    assert_eq!(counters, 1, "a +1/+1 counter was placed");
}

/// Novice Inspector investigates on ETB.
#[test]
fn novice_inspector_investigates() {
    let mut g = two_player_game();
    let ni = g.add_card_to_battlefield(0, catalog::novice_inspector());
    g.fire_self_etb_triggers(ni, 0);
    drain_stack(&mut g);
    assert_eq!(clues(&g, 0), 1, "ETB investigate");
}

/// Curious Cadaver returns from the graveyard when you sacrifice a Clue.
#[test]
fn curious_cadaver_returns_on_clue_sacrifice() {
    let mut g = two_player_game();
    // A Clue on the battlefield, and the Cadaver waiting in the graveyard.
    let inspector = g.add_card_to_battlefield(0, catalog::novice_inspector());
    g.fire_self_etb_triggers(inspector, 0);
    drain_stack(&mut g);
    let clue = g.battlefield.iter().find(|c| c.definition.name == "Clue").unwrap().id;
    let cadaver = g.add_card_to_graveyard(0, catalog::curious_cadaver());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentSacrificed { card_id: clue, who: 0 }]);
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.id == cadaver),
        "Cadaver returned to hand on Clue sacrifice"
    );
}

/// They Went This Way ramps a tapped basic and investigates.
#[test]
fn they_went_this_way_ramps_and_investigates() {
    let mut g = two_player_game();
    let src = g.add_card_to_hand(0, catalog::they_went_this_way());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let effect = catalog::they_went_this_way().effect.clone();
    let ctx = EffectContext::for_ability(src, 0, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    let land = g.battlefield.iter().find(|c| c.id == forest).unwrap();
    assert!(land.tapped, "basic entered tapped");
    assert_eq!(clues(&g, 0), 1, "investigated");
}

/// Undercover Crocodelf has Disguise and investigates on combat connect.
#[test]
fn undercover_crocodelf_disguise_and_connect_investigate() {
    let mut g = two_player_game();
    let croc = g.add_card_to_battlefield(0, catalog::undercover_crocodelf());
    assert!(
        catalog::undercover_crocodelf()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::Disguise(_))),
        "has Disguise"
    );
    let effect = catalog::undercover_crocodelf().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(croc, 0, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(clues(&g, 0), 1, "investigated on combat damage");
}

/// Sharp-Eyed Rookie grows and investigates when a bigger creature enters, but
/// ignores a smaller one.
#[test]
fn sharp_eyed_rookie_grows_on_bigger_creature() {
    let mut g = two_player_game();
    let rookie = g.add_card_to_battlefield(0, catalog::sharp_eyed_rookie());
    // A 1/1 is not bigger — no trigger.
    let small = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: small }]);
    drain_stack(&mut g);
    assert_eq!(clues(&g, 0), 0, "small creature does not trigger");
    // A 3/3 is bigger in both stats — +1/+1 and investigate.
    let big = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: big }]);
    drain_stack(&mut g);
    let c = g.computed_permanent(rookie).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "grew from the counter");
    assert_eq!(clues(&g, 0), 1, "investigated");
}
