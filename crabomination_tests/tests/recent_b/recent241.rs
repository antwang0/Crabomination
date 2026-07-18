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

/// CR 701.60 — a suspected creature has menace and can't block.
#[test]
fn cr_701_60_suspected_creature_has_menace_and_cant_block() {
    use crabomination::effect::{Effect, Selector};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext {
        targets: vec![Target::Permanent(bear)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&Effect::Suspect { what: Selector::Target(0) }, &ctx).unwrap();
    let c = g.computed_permanent(bear).unwrap();
    assert!(c.keywords.contains(&Keyword::Menace), "suspected -> menace");
    assert!(c.keywords.contains(&Keyword::CantBlock), "suspected -> can't block");
}

/// CR 701.13 — an investigated Clue sacrifices for a card.
#[test]
fn cr_701_13_clue_sacrifices_to_draw() {
    use crabomination::game::GameAction;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let lox = g.add_card_to_battlefield(0, catalog::loxodon_eavesdropper());
    g.add_card_to_library(0, catalog::forest());
    g.fire_self_etb_triggers(lox, 0);
    drain_stack(&mut g);
    let clue = g.battlefield.iter().find(|c| c.definition.name == "Clue").unwrap().id;
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: clue,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("sacrifice the Clue to draw");
    drain_stack(&mut g);
    assert!(g.battlefield_find(clue).is_none(), "Clue sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

/// Mistway Spy, once turned face up, investigates whenever a creature you
/// control deals combat damage to a player this turn.
#[test]
fn mistway_spy_investigates_on_combat_damage() {
    let mut g = two_player_game();
    let spy = g.add_card_to_battlefield(0, catalog::mistway_spy());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::mistway_spy().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(spy, 0, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    // A creature you control deals combat damage to player 1.
    g.fire_combat_damage_to_player_triggers(attacker, 1, 2);
    drain_stack(&mut g);
    assert_eq!(clues(&g, 0), 1, "investigated on the combat damage");
}

/// Glint Weaver distributes three +1/+1 counters and gains life for greatest
/// toughness.
#[test]
fn glint_weaver_counters_and_lifegain() {
    let mut g = two_player_game();
    let weaver = g.add_card_to_battlefield(0, catalog::glint_weaver()); // 3/3
    let big = g.add_card_to_battlefield(0, catalog::avenger_of_zendikar()); // 5/5
    let effect = catalog::glint_weaver().triggered_abilities[0].effect.clone();
    let ctx = EffectContext {
        targets: vec![Target::Permanent(big)],
        ..EffectContext::for_ability(weaver, 0, None)
    };
    let life = g.players[0].life;
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    // All three counters land on the sole target (5/5 -> 8/8), greatest toughness
    // among the OTHER creatures (Avenger) is then 8.
    assert_eq!(g.computed_permanent(big).unwrap().toughness, 8, "three counters distributed");
    assert_eq!(g.players[0].life, life + 8, "gained life = greatest toughness");
}

/// Exit Specialist can't be blocked by big creatures and bounces one when
/// turned face up.
#[test]
fn exit_specialist_evasion_and_bounce() {
    let mut g = two_player_game();
    let exit = g.add_card_to_battlefield(0, catalog::exit_specialist());
    assert!(g
        .computed_permanent(exit)
        .unwrap()
        .keywords
        .contains(&Keyword::CantBeBlockedByPowerAtLeast(3)));
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let effect = catalog::exit_specialist().triggered_abilities[0].effect.clone();
    let ctx = EffectContext {
        targets: vec![Target::Permanent(victim)],
        ..EffectContext::for_ability(exit, 0, None)
    };
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "creature returned to hand");
}

/// Projektor Inspector loots when a Detective you control enters.
#[test]
fn projektor_inspector_loots_on_detective() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::projektor_inspector());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let to_pitch = g.add_card_to_hand(0, catalog::forest());
    // AutoDecider declines "may" — script the yes + the discard choice.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Discard(vec![to_pitch]),
    ]));
    let other = g.add_card_to_battlefield(0, catalog::loxodon_eavesdropper()); // a Detective
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: other }]);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == to_pitch), "looted (drew then discarded)");
}

/// Hotshot Investigators bounces a creature and investigates when it was yours.
#[test]
fn hotshot_investigators_bounces_and_investigates_own() {
    let mut g = two_player_game();
    let hot = g.add_card_to_battlefield(0, catalog::hotshot_investigators());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::hotshot_investigators().triggered_abilities[0].effect.clone();
    let ctx = EffectContext {
        targets: vec![Target::Permanent(mine)],
        ..EffectContext::for_ability(hot, 0, None)
    };
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "own creature returned to hand");
    assert_eq!(clues(&g, 0), 1, "controlled it -> investigate");
}

/// Frantic Scapegoat suspects itself on entry.
#[test]
fn frantic_scapegoat_suspects_itself() {
    let mut g = two_player_game();
    let goat = g.add_card_to_battlefield(0, catalog::frantic_scapegoat());
    g.fire_self_etb_triggers(goat, 0);
    drain_stack(&mut g);
    let c = g.computed_permanent(goat).unwrap();
    // Suspected creatures have menace and can't block.
    assert!(c.keywords.contains(&Keyword::Menace), "suspected -> menace");
}

/// Slice from the Shadows gives target creature -X/-X and can't be countered.
#[test]
fn slice_from_the_shadows_shrinks() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::avenger_of_zendikar()); // 5/5
    let def = catalog::slice_from_the_shadows();
    assert!(def.keywords.contains(&Keyword::CantBeCountered));
    let ctx = EffectContext {
        targets: vec![Target::Permanent(victim)],
        ..EffectContext::for_spell(0, None, 0, 3)
    };
    g.resolve_effect(&def.effect, &ctx).unwrap();
    drain_stack(&mut g);
    let c = g.computed_permanent(victim).unwrap();
    assert_eq!((c.power, c.toughness), (2, 2), "-3/-3 applied");
}

/// Cerebral Confiscation's first mode makes the opponent discard two cards.
#[test]
fn cerebral_confiscation_discards_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let modes = match &catalog::cerebral_confiscation().effect {
        crabomination::effect::Effect::ChooseMode(m) => m.clone(),
        _ => panic!("not modal"),
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let before = g.players[1].hand.len();
    g.resolve_effect(&modes[0], &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), before - 2, "opponent discarded two");
}

/// Caught Red-Handed steals a creature for the turn and suspects it.
#[test]
fn caught_red_handed_steals_and_suspects() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext {
        targets: vec![Target::Permanent(creature)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&catalog::caught_red_handed().effect, &ctx).unwrap();
    drain_stack(&mut g);
    let c = g.computed_permanent(creature).unwrap();
    assert_eq!(c.controller, 0, "control gained");
    assert!(c.keywords.contains(&Keyword::Haste), "gains haste");
    assert!(c.keywords.contains(&Keyword::Menace), "suspected -> menace");
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
