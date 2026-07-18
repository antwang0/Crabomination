//! Functionality tests for `catalog::sets::decks::recent243` (MKM batch).

use crabomination::card::{CardType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
}

/// The Chase Is On pumps +3/+0, grants first strike, and investigates.
#[test]
fn the_chase_is_on_pumps_and_investigates() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::the_chase_is_on());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast The Chase Is On");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 2), "+3/+0");
    assert!(c.keywords.contains(&Keyword::FirstStrike), "gains first strike");
    assert_eq!(clues(&g, 0), 1, "investigated");
}

/// Galvanize deals 3, or 5 once you've drawn two cards this turn.
#[test]
fn galvanize_scales_with_draws() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    // A 0/4 dies only to the 5-damage mode.
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_omens());
    let spell = g.add_card_to_hand(0, catalog::galvanize());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::forest());
    }
    let mut evs = vec![];
    g.draw_one(0, &mut evs);
    g.draw_one(0, &mut evs);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(wall)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Galvanize");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wall).is_none(), "5 damage after two draws kills the 0/4");
}

/// Red Herring has haste + must-attack, and sacs itself to draw.
#[test]
fn red_herring_keywords_and_sac_draw() {
    let mut g = two_player_game();
    let rh = g.add_card_to_battlefield(0, catalog::red_herring());
    g.add_card_to_library(0, catalog::forest());
    let c = g.computed_permanent(rh).unwrap();
    assert!(c.keywords.contains(&Keyword::Haste) && c.keywords.contains(&Keyword::MustAttack));
    let before = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rh,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("sacrifice to draw");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rh).is_none(), "sacrificed");
    assert_eq!(g.players[0].hand.len(), before + 1, "drew a card");
}

/// Vengeful Creeper, when turned face up, destroys an opponent's artifact.
#[test]
fn vengeful_creeper_face_up_destroys() {
    let mut g = two_player_game();
    let creeper = g.add_card_to_battlefield(0, catalog::vengeful_creeper());
    let orn = g.add_card_to_battlefield(1, catalog::ornithopter());
    let effect = catalog::vengeful_creeper().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(creeper, 0, Some(Target::Permanent(orn)));
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(orn).is_none(), "opponent artifact destroyed");
}

/// Rubblebelt Maverick surveils 2 on ETB.
#[test]
fn rubblebelt_maverick_surveils_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::forest());
    let b = g.add_card_to_library(0, catalog::forest());
    let mav = g.add_card_to_battlefield(0, catalog::rubblebelt_maverick());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![a, b],
    }]));
    g.fire_self_etb_triggers(mav, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 2, "surveilled two cards to graveyard");
}

/// Leering Onlooker's graveyard ability mints two tapped flying Bats.
#[test]
fn leering_onlooker_makes_two_tapped_bats() {
    let mut g = two_player_game();
    let src = g.add_card_to_graveyard(0, catalog::leering_onlooker());
    let effect = catalog::leering_onlooker().activated_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(src, 0, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    let bats: Vec<_> =
        g.battlefield.iter().filter(|c| c.definition.name == "Bat" && c.controller == 0).collect();
    assert_eq!(bats.len(), 2, "two Bats");
    assert!(bats.iter().all(|c| c.tapped && c.definition.keywords.contains(&Keyword::Flying)));
}

/// Tunnel Tipster grows at end step after a face-down creature entered.
#[test]
fn tunnel_tipster_grows_after_facedown() {
    let mut g = two_player_game();
    let tip = g.add_card_to_battlefield(0, catalog::tunnel_tipster());
    g.players[0].face_down_activity_this_turn = true;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let c = g.computed_permanent(tip).unwrap();
    assert_eq!((c.power, c.toughness), (2, 2), "+1/+1 counter after a face-down entered");
}

/// Gravestone Strider exiles a card from a graveyard.
#[test]
fn gravestone_strider_exiles_from_graveyard() {
    let mut g = two_player_game();
    let strider = g.add_card_to_battlefield(0, catalog::gravestone_strider());
    let victim = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let effect = catalog::gravestone_strider().activated_abilities[1].effect.clone();
    let ctx = EffectContext::for_ability(strider, 0, Some(Target::Permanent(victim)));
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.exile.iter().any(|c| c.id == victim), "graveyard card exiled");
    assert!(g.players[1].graveyard.iter().all(|c| c.id != victim));
}

/// The once-per-turn mana ability is a Golem's ramp; sanity-check the type line.
#[test]
fn gravestone_strider_is_artifact_creature() {
    let d = catalog::gravestone_strider();
    assert!(d.card_types.contains(&CardType::Artifact) && d.card_types.contains(&CardType::Creature));
}
