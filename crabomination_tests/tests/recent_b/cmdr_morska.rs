//! Commander: the Deep Clue Sea precon (MKC, Morska, `decks::cmdr_morska`),
//! and the primitives it brought: permanent ascend (CR 702.131b), ending goad
//! (CR 701.15a), a silence until your next turn, Tezzeret's first-artifact
//! discount.

use crabomination::card::{ArtifactSubtype, CardId, CounterType, Keyword};
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
    for seat in 0..n {
        for _ in 0..12 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
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

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn clues(g: &GameState, seat: usize) -> Vec<CardId> {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Clue))
        .map(|c| c.id)
        .collect()
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
    g.step = TurnStep::PreCombatMain;
}

/// Seat 0 draws two through Divination (the second card is the turn's second).
fn divine(g: &mut GameState) {
    let d = g.add_card_to_hand(0, catalog::divination());
    cast(g, 0, d, None).expect("divination");
}

/// CR 702.131b — ascend on a permanent grants the blessing the moment the
/// tenth permanent enters, with no upkeep or enter trigger in between; the
/// Detective then can't be blocked.
#[test]
fn cr_702_131b_permanent_ascend_is_continuous() {
    let mut g = pod(2);
    let det = g.add_card_to_hand(0, catalog::detective_of_the_month());
    cast(&mut g, 0, det, None).expect("cast");
    for _ in 0..8 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    assert!(!g.players[0].city_blessing, "nine permanents");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, 0, bear, None).expect("tenth permanent");
    assert!(g.players[0].city_blessing);
    let det = named(&g, 0, "Detective of the Month")[0];
    assert!(g.computed_permanent(det).unwrap().keywords().contains(&Keyword::Unblockable));
    let bear = named(&g, 0, "Grizzly Bears")[0];
    assert!(!g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Unblockable), "not a Detective");
}

/// CR 702.131b — no ascend permanent, no blessing, however many permanents.
#[test]
fn cr_702_131b_ten_permanents_without_ascend_grant_nothing() {
    let mut g = pod(2);
    for _ in 0..11 {
        let f = g.add_card_to_hand(0, catalog::forest());
        g.priority.player_with_priority = 0;
        g.players[0].lands_played_this_turn = 0;
        g.perform_action(GameAction::PlayLand(f)).expect("land");
    }
    assert!(!g.players[0].city_blessing);
}

/// Detective of the Month — your second draw each turn makes a 2/2 Detective.
#[test]
fn detective_of_the_month_second_draw_makes_a_detective() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::detective_of_the_month());
    divine(&mut g);
    assert_eq!(named(&g, 0, "Detective").len(), 1);
    divine(&mut g);
    assert_eq!(named(&g, 0, "Detective").len(), 1, "third and fourth draws don't trigger");
}

/// CR 701.15a — Serene Sleuth investigates per goaded creature you control,
/// then ends their goad.
#[test]
fn cr_701_15a_serene_sleuth_ungoads() {
    let mut g = pod(3);
    let s = g.add_card_to_hand(0, catalog::serene_sleuth());
    cast(&mut g, 0, s, None).expect("cast");
    assert_eq!(clues(&g, 0).len(), 1, "enters and investigates");
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.battlefield.find_by_id_mut(a).unwrap().goaded_by.push(1);
    g.battlefield.find_by_id_mut(b).unwrap().goaded_by.push(2);
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(clues(&g, 0).len(), 3);
    for id in [a, b] {
        assert!(!g.is_goaded(g.battlefield_find(id).unwrap()));
    }
}

/// Innocuous Researcher — untapping your lands at your end step locks you out
/// of casting through the opponents' turns, until your own turn begins.
#[test]
fn innocuous_researcher_silence_lasts_until_your_turn() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::innocuous_researcher());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(land).unwrap().tapped, "lands untapped");
    assert!(g.players[0].silenced_this_turn);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    assert!(cast(&mut g, 0, bolt, Some(Target::Player(1))).is_err(), "can't cast this turn");
    // Seat 1's turn: still silenced.
    g.active_player_idx = 1;
    g.step = TurnStep::Untap;
    g.do_untap();
    assert!(g.players[0].silenced_this_turn, "still locked on the opponent's turn");
    // Seat 0's own turn: free again.
    g.active_player_idx = 0;
    g.do_untap();
    assert!(!g.players[0].silenced_this_turn);
}

/// Tezzeret, Betrayer of Flesh — only the first artifact ability each turn is
/// {2} cheaper: a Clue cracks for free once, then costs {2} again.
#[test]
fn tezzeret_discounts_the_first_artifact_ability_each_turn() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::tezzeret_betrayer_of_flesh());
    let a = g.add_card_to_hand(0, catalog::armed_with_proof());
    cast(&mut g, 0, a, None).expect("two Clues");
    g.players[0].mana_pool = Default::default();
    let [c1, c2] = clues(&g, 0)[..] else { panic!("two clues") };
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, c1, 0, None).expect("first Clue is free");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert!(activate(&mut g, 0, c2, 0, None).is_err(), "the second costs {{2}} and the pool is empty");
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, 0, c2, 0, None).expect("paid");
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Tezzeret's +1 keeps both cards when an artifact is discarded instead.
#[test]
fn tezzeret_plus_one_discards_an_artifact_instead() {
    let mut g = pod(2);
    let t = g.add_card_to_battlefield(0, catalog::tezzeret_betrayer_of_flesh());
    g.add_card_to_hand(0, catalog::sol_ring());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: t, ability_index: 0, target: None, x_value: None })
        .expect("+1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 3, "2 + 2 drawn - the Sol Ring");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Sol Ring"));
}

/// Morska — investigates each upkeep; the second draw puts two counters on it.
#[test]
fn morska_investigates_and_grows() {
    let mut g = pod(2);
    let m = g.add_card_to_battlefield(0, catalog::morska_undersea_sleuth());
    step(&mut g, TurnStep::Upkeep);
    assert_eq!(clues(&g, 0).len(), 1);
    divine(&mut g);
    assert_eq!(g.battlefield_find(m).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Erdwal Illuminator — the first investigation each turn investigates again;
/// the second doesn't.
#[test]
fn erdwal_illuminator_doubles_the_first_investigation() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::erdwal_illuminator());
    let s = g.add_card_to_hand(0, catalog::serene_sleuth());
    cast(&mut g, 0, s, None).expect("cast");
    assert_eq!(clues(&g, 0).len(), 2);
    let s = g.add_card_to_hand(0, catalog::serene_sleuth());
    cast(&mut g, 0, s, None).expect("cast");
    assert_eq!(clues(&g, 0).len(), 3);
}

/// Graf Mole and Ulvenwald Mysteries — sacrificing a Clue gains 3 and makes a
/// Human Soldier; a nontoken creature dying investigates.
#[test]
fn clue_sacrifice_payoffs() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::graf_mole());
    g.add_card_to_battlefield(0, catalog::ulvenwald_mysteries());
    let s = g.add_card_to_hand(0, catalog::serene_sleuth());
    cast(&mut g, 0, s, None).expect("cast");
    let clue = clues(&g, 0)[0];
    let life = g.players[0].life;
    flood(&mut g, 0);
    activate(&mut g, 0, clue, 0, None).expect("crack");
    assert_eq!(g.players[0].life, life + 3);
    assert_eq!(named(&g, 0, "Human Soldier").len(), 1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let sleuth = named(&g, 0, "Serene Sleuth")[0];
    cast(&mut g, 1, bolt, Some(Target::Permanent(sleuth))).expect("bolt");
    assert_eq!(clues(&g, 0).len(), 1, "the Sleuth died and investigated");
    let soldier = named(&g, 0, "Human Soldier")[0];
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(soldier))).expect("bolt");
    assert_eq!(clues(&g, 0).len(), 1, "a token dying doesn't");
}

/// Knowledge Is Power — +X/+X for each card drawn this turn.
#[test]
fn knowledge_is_power_counts_draws() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::knowledge_is_power());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, bear), (2, 2));
    divine(&mut g);
    assert_eq!(pt(&g, bear), (4, 4));
}

/// Alandra — a Drake on the second draw; the fifth pumps Alandra and Drakes by
/// hand size.
#[test]
fn alandra_drakes_and_the_fifth_draw() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::alandra_sky_dreamer());
    divine(&mut g);
    let drake = named(&g, 0, "Drake");
    assert_eq!(drake.len(), 1);
    divine(&mut g);
    assert_eq!(pt(&g, a), (2, 4));
    divine(&mut g);
    let x = g.players[0].hand.len() as i32;
    assert_eq!(pt(&g, a), (2 + x, 4 + x));
    assert_eq!(pt(&g, drake[0]), (2 + x, 2 + x));
}

/// Armed with Proof — two Clues that equip for {2} and give +2/+0.
#[test]
fn armed_with_proof_clues_are_equipment() {
    let mut g = pod(2);
    let a = g.add_card_to_hand(0, catalog::armed_with_proof());
    cast(&mut g, 0, a, None).expect("cast");
    let cs = clues(&g, 0);
    assert_eq!(cs.len(), 2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: cs[0], target: bear }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (4, 2));
}

/// Tangletrove Kelp — at each combat your other Clues become 6/6 Plants.
#[test]
fn tangletrove_kelp_animates_clues() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::tangletrove_kelp());
    let s = g.add_card_to_hand(0, catalog::serene_sleuth());
    cast(&mut g, 0, s, None).expect("cast");
    let clue = clues(&g, 0).into_iter().find(|&c| g.battlefield_find(c).unwrap().definition.name != "Tangletrove Kelp");
    let clue = clue.expect("a Clue token");
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(pt(&g, clue), (6, 6));
}

/// Aerial Extortionist — the exiled permanent's owner may cast it again.
#[test]
fn aerial_extortionist_exiles_castably() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ae = g.add_card_to_hand(0, catalog::aerial_extortionist());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: ae, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "exiled");
    assert!(g.exile.iter().any(|c| c.definition.name == "Grizzly Bears"));
    let exiled = g.exile.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap();
    assert!(exiled.may_play_until.is_some(), "its owner may cast it");
}

/// Bennie Bracks — any end step after you made a token draws a card.
#[test]
fn bennie_bracks_draws_after_a_token() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::bennie_bracks_zoologist());
    let hand = g.players[0].hand.len();
    step(&mut g, TurnStep::End);
    assert_eq!(g.players[0].hand.len(), hand, "no token, no card");
    let s = g.add_card_to_hand(0, catalog::serene_sleuth());
    cast(&mut g, 0, s, None).expect("cast");
    step(&mut g, TurnStep::End);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Sophia — Tiny arrives; sacrificing an artifact token counters every Dog.
#[test]
fn sophia_makes_tiny_and_feeds_dogs() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::sophia_dogged_detective());
    cast(&mut g, 0, s, None).expect("cast");
    let tiny = named(&g, 0, "Tiny");
    assert_eq!(tiny.len(), 1);
    assert!(g.computed_permanent(tiny[0]).unwrap().keywords().contains(&Keyword::Trample));
    let ss = g.add_card_to_hand(0, catalog::serene_sleuth());
    cast(&mut g, 0, ss, None).expect("a Clue to sacrifice");
    let sophia = named(&g, 0, "Sophia, Dogged Detective")[0];
    flood(&mut g, 0);
    activate(&mut g, 0, sophia, 0, None).expect("sac a Clue");
    assert_eq!(pt(&g, tiny[0]), (3, 3));
    assert!(clues(&g, 0).is_empty());
}

/// Sophia — a Dog connecting makes a Food and a Clue.
#[test]
fn sophia_dog_connects_for_food_and_clue() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::sophia_dogged_detective());
    cast(&mut g, 0, s, None).expect("cast");
    let tiny = named(&g, 0, "Tiny")[0];
    g.clear_sickness(tiny);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: tiny, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(named(&g, 0, "Food").len(), 1);
    assert_eq!(clues(&g, 0).len(), 1);
}

/// Chulane — a creature spell draws and drops a land from hand.
#[test]
fn chulane_draws_and_ramps_on_creature_spells() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::chulane_teller_of_tales());
    let lands = named(&g, 0, "Forest").len();
    g.add_card_to_hand(0, catalog::forest());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, 0, bear, None).expect("cast");
    assert_eq!(named(&g, 0, "Forest").len(), lands + 1);
}

/// Organic Extinction — nonartifact creatures die; artifact creatures stay.
#[test]
fn organic_extinction_spares_artifact_creatures() {
    let mut g = pod(2);
    let kelp = g.add_card_to_battlefield(0, catalog::tangletrove_kelp());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let oe = g.add_card_to_hand(0, catalog::organic_extinction());
    cast(&mut g, 0, oe, None).expect("cast");
    assert!(g.battlefield_find(kelp).is_some());
    assert!(named(&g, 1, "Grizzly Bears").is_empty());
}

/// Confirm Suspicions — counters and investigates three times.
#[test]
fn confirm_suspicions_counters_and_investigates() {
    let mut g = pod(2);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    let cs = g.add_card_to_hand(0, catalog::confirm_suspicions());
    let life = g.players[0].life;
    cast(&mut g, 0, cs, Some(Target::Permanent(bolt))).expect("counter");
    assert_eq!(g.players[0].life, life);
    assert_eq!(clues(&g, 0).len(), 3);
}
