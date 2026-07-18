//! Functionality tests for `catalog::sets::decks::recent241` (MKM batch).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::{EffectContext, EntityRef};
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game, GameEvent};

fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
}

/// Sanitation Automaton's ETB surveil bins the top card to the graveyard.
#[test]
fn sanitation_automaton_surveils() {
    let mut g = two_player_game();
    let top = g.add_card_to_library(0, catalog::forest());
    let auto = g.add_card_to_battlefield(0, catalog::sanitation_automaton());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![top],
    }]));
    g.fire_self_etb_triggers(auto, 0);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "top card surveilled to graveyard");
}

/// Loxodon Eavesdropper investigates on ETB and grows on the second draw.
#[test]
fn loxodon_eavesdropper_investigates_and_grows() {
    let mut g = two_player_game();
    let lox = g.add_card_to_battlefield(0, catalog::loxodon_eavesdropper());
    g.fire_self_etb_triggers(lox, 0);
    drain_stack(&mut g);
    assert_eq!(clues(&g, 0), 1, "ETB investigate made a Clue");
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::forest());
    }
    let mut evs = vec![];
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let mut evs = vec![];
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let c = g.computed_permanent(lox).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "+1/+1 on the second draw");
    assert!(c.keywords.contains(&Keyword::Vigilance), "gains vigilance on the second draw");
}

/// Jaded Analyst sheds defender and gains vigilance on the second draw.
#[test]
fn jaded_analyst_loses_defender_on_second_draw() {
    let mut g = two_player_game();
    let jaded = g.add_card_to_battlefield(0, catalog::jaded_analyst());
    assert!(g.computed_permanent(jaded).unwrap().keywords.contains(&Keyword::Defender));
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    for _ in 0..2 {
        let mut evs = vec![];
        g.draw_one(0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
    }
    drain_stack(&mut g);
    let c = g.computed_permanent(jaded).unwrap();
    assert!(!c.keywords.contains(&Keyword::Defender), "defender removed");
    assert!(c.keywords.contains(&Keyword::Vigilance), "vigilance gained");
}

/// Innocent Bystander investigates when dealt three or more damage.
#[test]
fn innocent_bystander_investigates_on_big_hit() {
    let mut g = two_player_game();
    let bystander = g.add_card_to_battlefield(0, catalog::innocent_bystander());
    let mut evs = vec![];
    g.deal_damage_to_from(EntityRef::Permanent(bystander), 3, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(clues(&g, 0), 1, "3 damage triggered investigate");
}

/// Innocent Bystander does not investigate on a small hit.
#[test]
fn innocent_bystander_ignores_small_hit() {
    let mut g = two_player_game();
    // 2/1 body — 2 damage would be lethal, so keep it alive with a toughness bump.
    let bystander = g.add_card_to_battlefield(0, catalog::innocent_bystander());
    let mut evs = vec![];
    g.deal_damage_to_from(EntityRef::Permanent(bystander), 2, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(clues(&g, 0), 0, "2 damage does not trigger investigate");
}

/// Rot Farm Mortipede pumps when a creature card leaves the graveyard.
#[test]
fn rot_farm_mortipede_pumps_on_graveyard_departure() {
    let mut g = two_player_game();
    let mort = g.add_card_to_battlefield(0, catalog::rot_farm_mortipede());
    let gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.dispatch_triggers_for_events(&[GameEvent::CardLeftGraveyard { player: 0, card_id: gy }]);
    drain_stack(&mut g);
    let c = g.computed_permanent(mort).unwrap();
    assert_eq!(c.power, 4, "+1/+0 until end of turn");
    assert!(c.keywords.contains(&Keyword::Menace) && c.keywords.contains(&Keyword::Lifelink));
}

/// Dog Walker mints two tapped Dog tokens when turned face up.
#[test]
fn dog_walker_makes_dogs_face_up() {
    let mut g = two_player_game();
    let walker = g.add_card_to_battlefield(0, catalog::dog_walker());
    let effect = catalog::dog_walker().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(walker, 0, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    let dogs: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Dog").collect();
    assert_eq!(dogs.len(), 2, "two Dog tokens");
    assert!(dogs.iter().all(|d| d.tapped), "tokens enter tapped");
}

/// Forum Familiar bounces another permanent you control and grows when turned
/// face up.
#[test]
fn forum_familiar_bounces_and_grows() {
    let mut g = two_player_game();
    let fam = g.add_card_to_battlefield(0, catalog::forum_familiar());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::forum_familiar().triggered_abilities[0].effect.clone();
    let ctx = EffectContext {
        targets: vec![Target::Permanent(other)],
        ..EffectContext::for_ability(fam, 0, None)
    };
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == other), "other permanent returned to hand");
    // Forum Familiar is 1/1; the +1/+1 counter makes it 2/2.
    let c = g.computed_permanent(fam).unwrap();
    assert_eq!((c.power, c.toughness), (2, 2), "gained a +1/+1 counter");
}

/// Sanguine Savior grants lifelink to another creature when turned face up.
#[test]
fn sanguine_savior_grants_lifelink() {
    let mut g = two_player_game();
    let savior = g.add_card_to_battlefield(0, catalog::sanguine_savior());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::sanguine_savior().triggered_abilities[0].effect.clone();
    let ctx = EffectContext {
        targets: vec![Target::Permanent(ally)],
        ..EffectContext::for_ability(savior, 0, None)
    };
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Lifelink));
}

/// Snarling Gorehound surveils when a small creature you control enters.
#[test]
fn snarling_gorehound_surveils_on_small_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::snarling_gorehound());
    let top = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![top],
    }]));
    // A 2/2 (power ≤ 2) entering triggers the surveil.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bear }]);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "surveilled to graveyard");
}
