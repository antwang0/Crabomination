//! Commander: the Spirit Squadron precon (VOC, Millicent,
//! `decks::cmdr_millicent`).

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 {
        two_player_game()
    } else {
        multi_player_game(seats)
    };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target: targets.first().cloned(),
            additional_targets: targets.iter().skip(1).cloned().collect(),
            mode: None,
            x_value: x,
        },
    )
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, id, targets, None)
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target: targets.first().cloned(),
            additional_targets: targets.iter().skip(1).cloned().collect(),
            x_value: None,
            mode: None,
        },
    )
}

fn tokens(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.is_token && c.definition.name == name)
        .map(|c| c.id)
        .collect()
}

fn to_step(g: &mut GameState, step: TurnStep) {
    for _ in 0..20 {
        if g.step == step {
            return;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    panic!("never reached {step:?}");
}

fn bolt(g: &mut GameState, at: Target) {
    let b = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(g, b, &[at]).expect("bolt");
}

/// Angel of Flight Alabaster returns a Spirit card at upkeep.
#[test]
fn angel_of_flight_alabaster_recurs_a_spirit() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::angel_of_flight_alabaster());
    let s = g.add_card_to_graveyard(0, catalog::spectral_shepherd());
    g.step = TurnStep::Untap;
    to_step(&mut g, TurnStep::Upkeep);
    assert!(g.players[0].hand.iter().any(|c| c.id == s));
}

/// Breath of the Sleepless: a Spirit at instant speed on an opponent's turn
/// taps a creature.
#[test]
fn breath_of_the_sleepless_flashes_spirits_and_taps() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::breath_of_the_sleepless());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    let s = g.add_card_to_hand(0, catalog::spectral_shepherd());
    cast_at(&mut g, s, &[]).expect("a Spirit with flash");
    assert!(g.battlefield_find(s).is_some());
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Disorder in the Court exiles X creatures and investigates X times.
#[test]
fn disorder_in_the_court_blinks_and_investigates() {
    let mut g = main_phase(2);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let d = g.add_card_to_hand(0, catalog::disorder_in_the_court());
    cast_x(
        &mut g,
        d,
        &[Target::Permanent(a), Target::Permanent(b)],
        Some(2),
    )
    .expect("cast");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert_eq!(tokens(&g, 0, "Clue").len(), 2);
    to_step(&mut g, TurnStep::End);
    assert!(
        g.battlefield_find(a)
            .is_some_and(|c| c.tapped && c.controller == 1)
    );
}

/// Donal copies a flying creature spell as a 1/1 Spirit (CR 707.9b).
#[test]
fn donal_copies_a_flier_as_a_one_one_spirit() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::donal_herald_of_wings());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let angel = g.add_card_to_hand(0, catalog::angel_of_flight_alabaster());
    cast_at(&mut g, angel, &[]).expect("cast");
    let copy = tokens(&g, 0, "Angel of Flight Alabaster");
    assert_eq!(copy.len(), 1, "the copy resolved into a token");
    let c = g.computed_permanent(copy[0]).unwrap();
    assert_eq!((c.power, c.toughness), (1, 1));
    assert!(
        g.battlefield_find(copy[0])
            .unwrap()
            .definition
            .has_creature_type(CreatureType::Spirit)
    );
    assert!(g.battlefield_find(angel).is_some());
}

/// Drogskol Reinforcements shields Spirits from noncombat damage and gives
/// the others melee.
#[test]
fn drogskol_reinforcements_shields_spirits() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::drogskol_reinforcements());
    let s = g.add_card_to_battlefield(0, catalog::ghostly_pilferer());
    bolt(&mut g, Target::Permanent(s));
    assert!(g.battlefield_find(s).is_some(), "the Bolt is prevented");
    assert!(
        g.computed_permanent(s)
            .unwrap()
            .keywords()
            .contains(&Keyword::Melee)
    );
}

/// Ethereal Investigator investigates once per opponent.
#[test]
fn ethereal_investigator_investigates_per_opponent() {
    let mut g = main_phase(4);
    let e = g.add_card_to_hand(0, catalog::ethereal_investigator());
    cast_at(&mut g, e, &[]).expect("cast");
    assert_eq!(tokens(&g, 0, "Clue").len(), 3);
}

/// Flood of Tears: four of your nontoken permanents bounced, you may put a
/// permanent card back.
#[test]
fn flood_of_tears_resets_and_redeploys() {
    let mut g = main_phase(2);
    let mine: Vec<CardId> = (0..4)
        .map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears()))
        .collect();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![mine[0]])]));
    let f = g.add_card_to_hand(0, catalog::flood_of_tears());
    cast_at(&mut g, f, &[]).expect("cast");
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs));
    assert!(g.battlefield_find(mine[0]).is_some(), "put back");
    assert!(g.battlefield_find(mine[1]).is_none());
}

/// Ghostly Pilferer: discard a card to go unblockable.
#[test]
fn ghostly_pilferer_discards_to_slip_through() {
    let mut g = main_phase(2);
    let gp = g.add_card_to_battlefield(0, catalog::ghostly_pilferer());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    activate(&mut g, gp, 0, &[]).expect("discard");
    assert!(g.players[0].hand.is_empty());
    assert!(
        g.computed_permanent(gp)
            .unwrap()
            .keywords()
            .contains(&Keyword::Unblockable)
    );
}

/// Haunted Library pays {1} for a Spirit when an opponent's creature dies.
#[test]
fn haunted_library_haunts() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::haunted_library());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    bolt(&mut g, Target::Permanent(bear));
    assert_eq!(tokens(&g, 0, "Spirit").len(), 1);
}

/// Haunting Imitation copies each creature on top as a 1/1 flying Spirit;
/// with none it comes back to hand.
#[test]
fn haunting_imitation_copies_the_tops() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let h = g.add_card_to_hand(0, catalog::haunting_imitation());
    cast_at(&mut g, h, &[]).expect("cast");
    let t = tokens(&g, 0, "Grizzly Bears");
    assert_eq!(t.len(), 2);
    let c = g.computed_permanent(t[0]).unwrap();
    assert_eq!((c.power, c.toughness), (1, 1));
    assert!(c.keywords().contains(&Keyword::Flying));

    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(1, catalog::island());
    let h = g.add_card_to_hand(0, catalog::haunting_imitation());
    cast_at(&mut g, h, &[]).expect("cast");
    assert!(
        g.players[0].hand.iter().any(|c| c.id == h),
        "no creature: back to hand"
    );
}

/// Nebelgast Herald taps an opposing creature as a Spirit enters; Rhoda
/// grows off that tap (not an attack).
#[test]
fn nebelgast_taps_and_rhoda_grows() {
    let mut g = main_phase(2);
    let rhoda = g.add_card_to_battlefield(0, catalog::rhoda_geist_avenger());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let n = g.add_card_to_hand(0, catalog::nebelgast_herald());
    cast_at(&mut g, n, &[Target::Permanent(bear)]).expect("cast");
    assert!(g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(
        g.battlefield_find(rhoda)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Occult Epiphany: a Spirit per card type among the discards.
#[test]
fn occult_epiphany_counts_discarded_types() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let o = g.add_card_to_hand(0, catalog::occult_epiphany());
    cast_x(&mut g, o, &[], Some(2)).expect("cast");
    assert_eq!(tokens(&g, 0, "Spirit").len(), 2, "creature + instant");
}

/// Priest of the Blessed Graf: a Spirit per opponent with more lands.
#[test]
fn priest_of_the_blessed_graf_counts_land_leaders() {
    let mut g = main_phase(4);
    g.add_card_to_battlefield(0, catalog::priest_of_the_blessed_graf());
    g.add_card_to_battlefield(1, catalog::plains());
    g.add_card_to_battlefield(2, catalog::plains());
    to_step(&mut g, TurnStep::End);
    assert_eq!(tokens(&g, 0, "Spirit").len(), 2);
}

/// Spectral Arcanist casts a cheap spell from a graveyard, then exiles it.
#[test]
fn spectral_arcanist_recasts_from_a_graveyard() {
    let mut g = main_phase(2);
    let b = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let life = g.players[1].life;
    let a = g.add_card_to_hand(0, catalog::spectral_arcanist());
    cast_at(&mut g, a, &[]).expect("cast");
    assert_eq!(g.players[1].life, life - 3);
    assert!(g.exile.iter().any(|c| c.id == b));
}

/// Spectral Shepherd bounces a Spirit you control.
#[test]
fn spectral_shepherd_rescues_a_spirit() {
    let mut g = main_phase(2);
    let s = g.add_card_to_battlefield(0, catalog::spectral_shepherd());
    activate(&mut g, s, 0, &[Target::Permanent(s)]).expect("bounce");
    assert!(g.players[0].hand.iter().any(|c| c.id == s));
}

/// Sudden Salvation returns a creature that died this turn to its owner,
/// tapped, and draws a card for that opponent.
#[test]
fn sudden_salvation_undoes_the_turn() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    bolt(&mut g, Target::Permanent(bear));
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    let s = g.add_card_to_hand(0, catalog::sudden_salvation());
    cast_at(&mut g, s, &[Target::Permanent(bear)]).expect("cast");
    assert!(
        g.battlefield_find(bear)
            .is_some_and(|c| c.tapped && c.controller == 1)
    );
    assert_eq!(
        g.players[0].hand.len(),
        hand + 1,
        "one opponent controls one"
    );
}

/// Timin taps a creature at the beginning of each combat.
#[test]
fn timin_taps_at_each_combat() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::timin_youthful_geist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    to_step(&mut g, TurnStep::BeginCombat);
    assert!(g.battlefield_find(bear).unwrap().tapped);
}
