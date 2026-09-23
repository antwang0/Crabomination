//! Commander: the Heads I Win, Tails You Lose deck's missing cards
//! (`decks::cmdr_zndrsplt`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::Blue, Color::Red] {
        g.players[0].mana_pool.add(c, 10);
    }
    g.players[0].mana_pool.add_colorless(10);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    flood(g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .unwrap_or_else(|e| panic!("activate: {e:?}"));
    drain_stack(g);
}

fn script(g: &mut GameState, answers: Vec<DecisionAnswer>) {
    g.decider = Box::new(ScriptedDecider::new(answers));
}

fn keywords(g: &GameState, id: CardId) -> Vec<Keyword> {
    g.computed_permanent(id).unwrap().keywords().to_vec()
}

/// Bloodsworn Steward pumps and hastes your commander creatures only.
#[test]
fn bloodsworn_steward_boosts_commanders() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::bloodsworn_steward());
    let okaun = g.add_card_to_battlefield(0, catalog::okaun_eye_of_chaos());
    g.players[0].commanders.push(okaun);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(okaun).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(keywords(&g, okaun).contains(&Keyword::Haste));
    assert_eq!(g.computed_permanent(bears).unwrap().power, 2, "not a commander");
}

/// Boompile wipes nonland permanents on a won flip and does nothing on a
/// lost one.
#[test]
fn boompile_destroys_nonland_permanents_on_a_win() {
    let mut g = main_phase();
    let pile = g.add_card_to_battlefield(0, catalog::boompile());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(1, catalog::island());
    script(&mut g, vec![DecisionAnswer::Bool(false)]);
    activate(&mut g, pile, 0, None);
    assert!(g.battlefield_find(bears).is_some(), "lost the flip");
    g.battlefield_find_mut(pile).unwrap().tapped = false;
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    activate(&mut g, pile, 0, None);
    assert!(g.battlefield_find(bears).is_none() && g.battlefield_find(pile).is_none());
    assert!(g.battlefield_find(land).is_some(), "lands stay");
}

/// Desolate Lighthouse loots.
#[test]
fn desolate_lighthouse_loots() {
    let mut g = main_phase();
    let house = g.add_card_to_battlefield(0, catalog::desolate_lighthouse());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, house, 1, None);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Flamekin Village enters tapped unless an Elemental is revealed, and hastes a
/// creature for {R}.
#[test]
fn flamekin_village_reveal_and_haste() {
    let mut g = main_phase();
    let land = g.add_card_to_hand(0, catalog::flamekin_village());
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped, "no Elemental to reveal");
    g.battlefield_find_mut(land).unwrap().tapped = false;
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, land, 1, Some(Target::Permanent(bears)));
    assert!(keywords(&g, bears).contains(&Keyword::Haste));
}

/// Footfall Crater gives its land "{T}: target creature gains trample and haste".
#[test]
fn footfall_crater_grants_its_land_an_ability() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    let crater = g.add_card_to_hand(0, catalog::footfall_crater());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: crater,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let granted = g.granted_abilities_for(land);
    assert_eq!(granted.len(), 1);
    let index = catalog::mountain().activated_abilities.len();
    activate(&mut g, land, index, Some(Target::Permanent(bears)));
    let kws = keywords(&g, bears);
    assert!(kws.contains(&Keyword::Trample) && kws.contains(&Keyword::Haste));
}

/// Frenetic Sliver gives every Sliver its {0} flip: a win exiles it until the
/// next end step, a loss sacrifices it.
#[test]
fn frenetic_sliver_flickers_or_sacrifices() {
    let mut g = main_phase();
    let frenetic = g.add_card_to_battlefield(0, catalog::frenetic_sliver());
    let other = g.add_card_to_battlefield(1, catalog::frenetic_sliver());
    // "All Slivers", either side — and one instance per Frenetic Sliver.
    assert_eq!(g.granted_abilities_for(other).len(), 2);
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    activate(&mut g, frenetic, 0, None);
    assert!(g.exile.iter().any(|c| c.id == frenetic), "won: exiled");
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(frenetic).is_some(), "back at the end step");

    let mut g = main_phase();
    let frenetic = g.add_card_to_battlefield(0, catalog::frenetic_sliver());
    script(&mut g, vec![DecisionAnswer::Bool(false)]);
    activate(&mut g, frenetic, 0, None);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == frenetic), "lost: sacrificed");
}

/// Daretti: +2 loots up to two, −2 trades an artifact for one in the yard, and
/// the −10 emblem brings back an artifact that went to the graveyard at the
/// next end step.
#[test]
fn daretti_scrap_savant_abilities() {
    let loyalty = |g: &mut GameState, id, index, target| {
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: id,
            ability_index: index,
            target,
            x_value: None,
        })
        .expect("loyalty");
        drain_stack(g);
    };
    let mut g = main_phase();
    let daretti = g.add_card_to_battlefield(0, catalog::daretti_scrap_savant());
    let fodder = g.add_card_to_battlefield(0, catalog::boompile());
    let back = g.add_card_to_graveyard(0, catalog::sol_ring());
    loyalty(&mut g, daretti, 1, Some(Target::Permanent(back)));
    assert!(g.battlefield_find(back).is_some(), "Sol Ring returned");
    assert!(g.battlefield_find(fodder).is_none(), "Boompile was the cost");

    let mut g = main_phase();
    let daretti = g.add_card_to_battlefield(0, catalog::daretti_scrap_savant());
    g.battlefield_find_mut(daretti)
        .unwrap()
        .add_counters(crabomination::card::CounterType::Loyalty, 10);
    loyalty(&mut g, daretti, 2, None);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let murder_like = crabomination::effect::Effect::Destroy {
        what: crabomination::effect::Selector::EachPermanent(
            crabomination::card::SelectionRequirement::Artifact,
        ),
    };
    let ctx = crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&murder_like, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == ring));
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_some(), "the emblem returned it");
}
