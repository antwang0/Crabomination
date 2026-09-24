//! Commander: the Arcane Wizardry precon (C17, Inalla, `decks::cmdr_inalla`).
//! The primitives it needed are tested in `core_rules/commander_cards.rs`.

use crabomination::card::{CardId, CardType, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
}

/// CR 113.6b eminence — from the command zone, a nontoken Wizard entering
/// may be copied for {1}; the hasty token is exiled at the next end step.
/// On the battlefield, tapping five Wizards costs a player 7 life.
#[test]
fn inalla_copies_wizards_from_the_command_zone_and_taps_five_for_seven() {
    let mut g = pod(2);
    g.seat_commanders(0, vec![catalog::inalla_archmage_ritualist()]);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let mage = g.add_card_to_hand(0, catalog::serendib_sorcerer());
    cast(&mut g, 0, mage, None).expect("cast");
    let copies = named(&g, 0, "Serendib Sorcerer");
    assert_eq!(copies.len(), 2, "the card and its token copy");
    let token = *copies.iter().find(|&&c| c != mage).unwrap();
    assert!(g.computed_permanent(token).unwrap().keywords().contains(&Keyword::Haste));
    step(&mut g, TurnStep::End);
    assert!(g.battlefield_find(token).is_none(), "exiled at the end step");

    let mut g = pod(2);
    let inalla = g.add_card_to_battlefield(0, catalog::inalla_archmage_ritualist());
    for _ in 0..4 {
        let w = g.add_card_to_battlefield(0, catalog::serendib_sorcerer());
        g.clear_sickness(w);
    }
    g.clear_sickness(inalla);
    let life = g.players[1].life;
    activate(&mut g, 0, inalla, 0, Some(Target::Player(1)), None).expect("five Wizards, Inalla one of them");
    assert_eq!(g.players[1].life, life - 7);
}

/// Body Double copies a creature card in an opponent's graveyard.
#[test]
fn body_double_copies_a_graveyard_creature() {
    let mut g = pod(2);
    g.add_card_to_graveyard(1, catalog::serra_angel());
    let bd = g.add_card_to_hand(0, catalog::body_double());
    cast(&mut g, 0, bd, None).expect("cast");
    assert_eq!(g.battlefield_find(bd).unwrap().definition.name, "Serra Angel");
    assert_eq!(pt(&g, bd), (4, 4));
}

/// Clone Legion copies every creature the target player controls.
#[test]
fn clone_legion_copies_a_players_board() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    let cl = g.add_card_to_hand(0, catalog::clone_legion());
    cast(&mut g, 0, cl, Some(Target::Player(1))).expect("cast");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1);
    assert_eq!(named(&g, 0, "Serra Angel").len(), 1);
}

/// CR 506.2 — the curser draws when the cursed player is attacked, and so
/// does the opponent attacking them.
#[test]
fn curse_of_verbosity_draws_for_the_curser_and_the_attacker() {
    let mut g = pod(3);
    for s in 0..3 {
        g.add_card_to_library(s, catalog::island());
    }
    let curse = g.add_card_to_hand(0, catalog::curse_of_verbosity());
    cast(&mut g, 0, curse, Some(Target::Player(2))).expect("cast");
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    g.active_player_idx = 1;
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bears);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(2) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 + 1);
    assert_eq!(g.players[1].hand.len(), h1 + 1);
}

/// Grixis Panorama fetches a tapped basic Swamp.
#[test]
fn grixis_panorama_fetches_a_grixis_basic() {
    let mut g = pod(2);
    let pan = g.add_card_to_battlefield(0, catalog::grixis_panorama());
    g.add_card_to_library(0, catalog::forest());
    let swamp = g.add_card_to_library(0, catalog::swamp());
    activate(&mut g, 0, pan, 1, None, None).expect("fetch");
    assert!(g.battlefield_find(swamp).is_some_and(|c| c.tapped));
    assert!(g.battlefield_find(pan).is_none());
}

/// Izzet Chemister banks a Bolt under itself, then sacrifices to cast it free.
#[test]
fn izzet_chemister_banks_and_recasts() {
    let mut g = pod(2);
    let chem = g.add_card_to_battlefield(0, catalog::izzet_chemister());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    activate(&mut g, 0, chem, 0, Some(Target::Permanent(bolt)), None).expect("bank");
    assert!(g.exile.iter().any(|c| c.id == bolt && c.exiled_with == Some(chem)));
    g.battlefield_find_mut(chem).unwrap().tapped = false;
    let life = g.players[1].life;
    activate(&mut g, 0, chem, 1, None, None).expect("recast");
    assert_eq!(g.players[1].life, life - 3, "the Bolt was cast free at the opponent");
    assert!(g.battlefield_find(chem).is_none());
}

/// Kess lets an instant be cast from the graveyard once a turn, then exiles it.
#[test]
fn kess_recasts_an_instant_and_exiles_it() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::kess_dissident_mage());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let life = g.players[1].life;
    cast(&mut g, 0, bolt, Some(Target::Player(1))).expect("from the graveyard");
    assert_eq!(g.players[1].life, life - 3);
    assert!(g.exile.iter().any(|c| c.id == bolt));
}

/// Magus of the Abyss — at each upkeep the active player loses one
/// nonartifact creature of their choice; an artifact creature is safe.
#[test]
fn magus_of_the_abyss_takes_one_nonartifact_creature() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::magus_of_the_abyss());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let thopter = g.add_card_to_battlefield(1, catalog::ornithopter());
    g.active_player_idx = 1;
    step(&mut g, TurnStep::Upkeep);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(angel).is_some() && g.battlefield_find(thopter).is_some());
}

/// Magus of the Mind exiles one plus the turn's spell count and grants free
/// plays from among them.
#[test]
fn magus_of_the_mind_scales_with_the_storm_count() {
    let mut g = pod(2);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Player(1))).expect("one spell");
    let magus = g.add_card_to_battlefield(0, catalog::magus_of_the_mind());
    g.clear_sickness(magus);
    let exiled = g.exile.len();
    activate(&mut g, 0, magus, 0, None, None).expect("activate");
    assert_eq!(g.exile.len(), exiled + 2, "one plus one spell this turn");
    let free = g.exile.iter().rev().take(2).map(|c| c.id).collect::<Vec<_>>();
    g.players[0].mana_pool.empty();
    g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: free[0], target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast free from exile, no mana");
    drain_stack(&mut g);
    assert!(g.battlefield_find(free[0]).is_some());
}

/// Mairsil cages a creature card from the graveyard and uses its ability.
#[test]
fn mairsil_borrows_a_caged_cards_ability() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let sorcerer = g.add_card_to_graveyard(0, catalog::prodigal_sorcerer());
    let m = g.add_card_to_hand(0, catalog::mairsil_the_pretender());
    cast(&mut g, 0, m, None).expect("cast");
    let caged = g.exile.iter().find(|c| c.id == sorcerer).expect("caged");
    assert_eq!(caged.counter_count(CounterType::Cage), 1);
    g.clear_sickness(m);
    let life = g.players[1].life;
    activate(&mut g, 0, m, 0, Some(Target::Player(1)), None).expect("the Sorcerer's ping");
    assert_eq!(g.players[1].life, life - 1);
}

/// Mirror of the Forebears names Bear and becomes a copy of your Bear until
/// end of turn, still an artifact.
#[test]
fn mirror_of_the_forebears_copies_your_named_type() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mirror = g.add_card_to_hand(0, catalog::mirror_of_the_forebears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(CreatureType::Bear)]));
    cast(&mut g, 0, mirror, None).expect("cast");
    activate(&mut g, 0, mirror, 0, Some(Target::Permanent(bear)), None).expect("copy");
    let c = g.computed_permanent(mirror).expect("still here");
    assert_eq!(c.power, 2);
    assert!(c.card_types().contains(&CardType::Creature) && c.card_types().contains(&CardType::Artifact));
}

/// Rulings — flashed in during declare attackers, Portal Mage points an
/// attacker at another player.
#[test]
fn portal_mage_redirects_an_attacker() {
    let mut g = pod(3);
    g.active_player_idx = 1;
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.clear_sickness(giant);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: giant, target: AttackTarget::Player(0) }]).expect("attack");
    drain_stack(&mut g);
    let mage = g.add_card_to_hand(0, catalog::portal_mage());
    cast(&mut g, 0, mage, None).expect("flash");
    assert_eq!(g.attacking[0].target, AttackTarget::Player(2));
}

/// Serendib Sorcerer makes another creature a 0/2.
#[test]
fn serendib_sorcerer_shrinks_a_creature() {
    let mut g = pod(2);
    let s = g.add_card_to_battlefield(0, catalog::serendib_sorcerer());
    g.clear_sickness(s);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    activate(&mut g, 0, s, 0, Some(Target::Permanent(angel)), None).expect("activate");
    assert_eq!(pt(&g, angel), (0, 2));
}

/// Shifting Shadow — the enchanted creature has haste; at its controller's
/// upkeep it's destroyed and the Aura jumps to the next creature revealed.
#[test]
fn shifting_shadow_polymorphs_each_upkeep() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    g.add_card_to_library(0, catalog::island());
    let shadow = g.add_card_to_hand(0, catalog::shifting_shadow());
    cast(&mut g, 0, shadow, Some(Target::Permanent(bear))).expect("cast");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Haste));
    step(&mut g, TurnStep::Upkeep);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.battlefield_find(shadow).and_then(|c| c.attached_to), Some(angel));
    assert!(g.computed_permanent(angel).unwrap().keywords().contains(&Keyword::Haste));
}

/// Taigam keeps one of three at upkeep, bins the rest, and turns graveyard
/// cards into -X/-X.
#[test]
fn taigam_digs_and_shrinks() {
    let mut g = pod(2);
    let t = g.add_card_to_battlefield(0, catalog::taigam_sidisis_hand());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let (hand, gy) = (g.players[0].hand.len(), g.players[0].graveyard.len());
    step(&mut g, TurnStep::Upkeep);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].graveyard.len(), gy + 2);
    g.clear_sickness(t);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    activate(&mut g, 0, t, 0, Some(Target::Permanent(angel)), Some(2)).expect("exile two");
    assert_eq!(pt(&g, angel), (2, 2));
    assert_eq!(g.players[0].graveyard.len(), gy);
}

/// Vindictive Lich — each mode at a different opponent: at four seats all
/// three land on three players; in a duel only one mode can.
#[test]
fn vindictive_lich_splits_its_modes_across_opponents() {
    let mut g = pod(4);
    for s in 1..4 {
        g.add_card_to_battlefield(s, catalog::grizzly_bears());
        g.add_card_to_hand(s, catalog::island());
        g.add_card_to_hand(s, catalog::island());
    }
    let lich = g.add_card_to_battlefield(0, catalog::vindictive_lich());
    let before: Vec<(i32, usize, usize)> = (1..4)
        .map(|s| (g.players[s].life, g.players[s].hand.len(), named(&g, s, "Grizzly Bears").len()))
        .collect();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(lich))).expect("kill it");
    let after: Vec<(i32, usize, usize)> = (1..4)
        .map(|s| (g.players[s].life, g.players[s].hand.len(), named(&g, s, "Grizzly Bears").len()))
        .collect();
    let lost5 = before.iter().zip(&after).filter(|(b, a)| b.0 - a.0 == 5).count();
    let discarded = before.iter().zip(&after).filter(|(b, a)| b.1 - a.1 == 2).count();
    let sacked = before.iter().zip(&after).filter(|(b, a)| b.2 - a.2 == 1).count();
    assert_eq!((lost5, discarded, sacked), (1, 1, 1));
    for (b, a) in before.iter().zip(&after) {
        let hits = (b.0 != a.0) as u8 + (b.1 != a.1) as u8 + (b.2 != a.2) as u8;
        assert!(hits <= 1, "each mode at a different player");
    }

    // A duel: one opponent, so one mode — the five life.
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::island());
    let lich = g.add_card_to_battlefield(0, catalog::vindictive_lich());
    let (life, hand) = (g.players[1].life, g.players[1].hand.len());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(lich))).expect("kill it");
    assert_eq!(g.players[1].life, life - 5);
    assert_eq!(g.players[1].hand.len(), hand, "no second mode at the same player");
    assert_eq!(named(&g, 1, "Grizzly Bears").len(), 1);
}

