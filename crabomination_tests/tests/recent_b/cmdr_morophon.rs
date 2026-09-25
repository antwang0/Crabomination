//! Commander: the Everyone's Invited! Secret Lair deck (SLD, Morophon, the
//! Boundless, `decks::cmdr_morophon`).

use crabomination::card::{ArtifactSubtype, CardId, CounterType, CreatureType, Keyword, Supertype};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn act_as(g: &mut GameState, seat: usize, action: GameAction) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    act_as(g, 0, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    act_as(g, 0, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn kw(g: &GameState, id: CardId, k: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&k))
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn bolt(g: &mut GameState, at: CardId) {
    let b = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(g, b, &[Target::Permanent(at)]).expect("bolt");
}

/// CR 700.8 — Stick Together: each player keeps a largest party (a Cleric
/// Wizard fills whichever slot leaves room) and sacrifices the rest.
#[test]
fn cr_700_8_stick_together_keeps_a_party() {
    let mut g = main_phase(2);
    let warrior = g.add_card_to_battlefield(0, catalog::harper_recruiter());
    let changeling = g.add_card_to_battlefield(0, catalog::skeletal_changeling());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let st = g.add_card_to_hand(0, catalog::stick_together());
    cast_at(&mut g, st, &[]).expect("cast");
    assert!(g.battlefield_find(warrior).is_some() && g.battlefield_find(changeling).is_some());
    assert!(g.battlefield_find(bear).is_none(), "no party role");
    assert!(g.battlefield_find(theirs).is_none());
}

/// Harper Recruiter: on attack, a party from the top four comes to hand —
/// one per role, the rest to the bottom.
#[test]
fn harper_recruiter_recruits_a_party() {
    let mut g = main_phase(2);
    let a = g.add_card_to_library(0, catalog::harper_recruiter());
    let b = g.add_card_to_library(0, catalog::harper_recruiter());
    let c = g.add_card_to_library(0, catalog::skeletal_changeling());
    let d = g.add_card_to_library(0, catalog::grizzly_bears());
    let hr = g.add_card_to_battlefield(0, catalog::harper_recruiter());
    g.clear_sickness(hr);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: hr,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let in_hand = |id| g.players[0].hand.iter().any(|x| x.id == id);
    assert!(in_hand(c), "the changeling fills a role");
    assert_eq!([a, b].iter().filter(|&&x| in_hand(x)).count(), 1, "one Warrior, not two");
    assert!(!in_hand(d));
    assert_eq!(g.players[0].library.len(), 2);
}

/// CR 707.2 — Moritte copies a permanent you control, legendary and snow,
/// with two extra +1/+1 counters and changeling.
#[test]
fn cr_707_2_moritte_copies_as_a_snow_legend() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::moritte_of_the_frost());
    cast_at(&mut g, m, &[]).expect("cast");
    let c = g.battlefield_find(m).expect("on the battlefield");
    assert_eq!(c.definition.name, "Grizzly Bears");
    assert!(c.definition.supertypes.contains(&Supertype::Legendary));
    assert!(c.definition.supertypes.contains(&Supertype::Snow));
    assert_eq!(pt(&g, m), (4, 4));
    assert!(kw(&g, m, Keyword::Changeling));
}

/// CR 611.2b — Shapesharer's copy lasts until your next turn: through the
/// opponent's turn, gone as yours begins.
#[test]
fn cr_611_2b_shapesharer_copy_lasts_until_your_next_turn() {
    let mut g = main_phase(2);
    let ss = g.add_card_to_battlefield(0, catalog::shapesharer());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    activate(&mut g, ss, 0, &[Target::Permanent(ss), Target::Permanent(giant)]).expect("copy");
    assert_eq!(g.battlefield_find(ss).unwrap().definition.name, "Hill Giant");
    // Through this turn's cleanup and into the opponent's turn.
    for _ in 0..40 {
        if g.active_player_idx == 1 && g.step == TurnStep::PreCombatMain {
            break;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.active_player_idx, 1);
    assert_eq!(g.battlefield_find(ss).unwrap().definition.name, "Hill Giant", "still a copy on their turn");
    for _ in 0..40 {
        if g.active_player_idx == 0 && g.step == TurnStep::Upkeep {
            break;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(ss).unwrap().definition.name, "Shapesharer", "reverted as your turn began");
}

/// Brenard: a dying nontoken creature comes back as a 1/1 Food Golem token
/// copy (+2/+2 and trample from Brenard) that can be sacrificed for 3 life.
#[test]
fn brenard_bakes_a_food_golem() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::brenard_ginger_sculptor());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let doom = g.add_card_to_hand(0, catalog::doom_blade());
    cast_at(&mut g, doom, &[Target::Permanent(giant)]).expect("kill");
    let token = named(&g, 0, "Hill Giant");
    assert_eq!(token.len(), 1);
    let t = token[0];
    let c = g.battlefield_find(t).unwrap();
    assert!(c.is_token && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Food));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Golem));
    assert_eq!(pt(&g, t), (3, 3), "1/1 plus Brenard's +2/+2");
    assert!(kw(&g, t, Keyword::Trample));
    let idx = g.battlefield_find(t).unwrap().definition.activated_abilities.len() - 1;
    g.clear_sickness(t);
    let life = g.players[0].life;
    activate(&mut g, t, idx, &[]).expect("eat it");
    assert_eq!(g.players[0].life, life + 3);
}

/// Nameless Inversion: +3/-3 and no creature types, even on a changeling.
#[test]
fn nameless_inversion_strips_every_type() {
    let mut g = main_phase(2);
    let target = g.add_card_to_battlefield(1, catalog::guardian_gladewalker());
    g.battlefield_find_mut(target).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    let ni = g.add_card_to_hand(0, catalog::nameless_inversion());
    cast_at(&mut g, ni, &[Target::Permanent(target)]).expect("cast");
    assert_eq!(pt(&g, target), (7, 1));
    let cp = g.computed_permanent(target).unwrap();
    assert!(cp.subtypes().creature_types.is_empty() && !cp.keywords().contains(&Keyword::Changeling));
}

/// Raise the Palisade bounces every creature not of the chosen type (a
/// changeling is every type).
#[test]
fn raise_the_palisade_keeps_one_tribe() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let ch = g.add_card_to_battlefield(0, catalog::skeletal_changeling());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Bear)]));
    let rp = g.add_card_to_hand(0, catalog::raise_the_palisade());
    cast_at(&mut g, rp, &[]).expect("cast");
    assert!(g.battlefield_find(bear).is_some() && g.battlefield_find(ch).is_some());
    assert!(g.battlefield_find(giant).is_none());
}

/// Realmbreaker mills an opponent three and takes a land of theirs.
#[test]
fn realmbreaker_takes_their_land() {
    let mut g = main_phase(2);
    for _ in 0..2 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let forest = g.add_card_to_library(1, catalog::forest());
    let rb = g.add_card_to_battlefield(0, catalog::realmbreaker_the_invasion_tree());
    activate(&mut g, rb, 0, &[Target::Player(1)]).expect("activate");
    let f = g.battlefield_find(forest).expect("on the battlefield");
    assert!(f.controller == 0 && f.tapped);
}

/// Sophia makes Tiny; a Dog hitting a player makes a Food and a Clue.
#[test]
fn sophia_and_tiny_investigate() {
    let mut g = main_phase(2);
    let s = g.add_card_to_hand(0, catalog::sophia_dogged_detective());
    cast_at(&mut g, s, &[]).expect("cast");
    let tiny = named(&g, 0, "Tiny");
    assert_eq!(tiny.len(), 1);
    g.clear_sickness(tiny[0]);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: tiny[0],
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    for _ in 0..6 {
        if g.step == TurnStep::EndCombat {
            break;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(named(&g, 0, "Food").len(), 1);
    assert_eq!(named(&g, 0, "Clue").len(), 1);
}

/// Spoils of Adventure and Tazri cost {1} less per party creature.
#[test]
fn party_discounts() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::harper_recruiter());
    g.add_card_to_battlefield(0, catalog::skeletal_changeling());
    let spoils = g.add_card_to_hand(0, catalog::spoils_of_adventure());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: spoils, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{4}{W}{U} less two");
}

/// The Bears of Littjara: I makes a changeling Shapeshifter.
#[test]
fn the_bears_of_littjara_opens_with_a_shapeshifter() {
    let mut g = main_phase(2);
    let bl = g.add_card_to_hand(0, catalog::the_bears_of_littjara());
    cast_at(&mut g, bl, &[]).expect("cast");
    let s = named(&g, 0, "Shapeshifter");
    assert_eq!(s.len(), 1);
    assert!(kw(&g, s[0], Keyword::Changeling));
}

/// Unsettled Mariner taxes an opponent's targeting of your permanents.
#[test]
fn unsettled_mariner_taxes_targeting() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::unsettled_mariner());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: b, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "countered: no {{1}} left to pay");
}

/// The 1/1 changelings: Fire-Belly pumps at most twice a turn, Mothdust
/// flies by tapping a creature, Skeletal regenerates, Gladewalker counters,
/// Amoeboid grants every type.
#[test]
fn the_changeling_squad() {
    let mut g = main_phase(2);
    let fb = g.add_card_to_battlefield(0, catalog::fire_belly_changeling());
    for _ in 0..2 {
        activate(&mut g, fb, 0, &[]).expect("pump");
    }
    assert!(activate(&mut g, fb, 0, &[]).is_err(), "a third is over the limit");
    assert_eq!(pt(&g, fb), (3, 1));
    let md = g.add_card_to_battlefield(0, catalog::mothdust_changeling());
    activate(&mut g, md, 0, &[]).expect("tap a creature");
    assert!(kw(&g, md, Keyword::Flying));
    let sk = g.add_card_to_battlefield(0, catalog::skeletal_changeling());
    activate(&mut g, sk, 0, &[]).expect("shield");
    bolt(&mut g, sk);
    assert!(g.battlefield_find(sk).is_some(), "regenerated");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gg = g.add_card_to_hand(0, catalog::guardian_gladewalker());
    cast_at(&mut g, gg, &[]).expect("cast");
    let counters: u32 = [bear, gg, fb, md, sk]
        .iter()
        .filter_map(|&id| g.battlefield_find(id))
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(counters, 1);
    let am = g.add_card_to_battlefield(0, catalog::amoeboid_changeling());
    g.clear_sickness(am);
    activate(&mut g, am, 0, &[Target::Permanent(bear)]).expect("gain types");
    assert!(kw(&g, bear, Keyword::Changeling));
}
