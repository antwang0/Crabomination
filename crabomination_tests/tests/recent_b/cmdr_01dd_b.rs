//! Commander: the 2026-10-20 (`01DDeUtc`) one-gap pair closers
//! (`decks::cmdr_01dd_k`).

use crabomination::card::{CardId, CardType, CounterType, CreatureType, Subtypes, TokenDefinition};
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
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, more: Vec<Target>, x: Option<u32>, mode: Option<usize>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: more, mode, x_value: x })
        .expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    cast_with(g, id, target, vec![], None, None);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), GameError> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn body(name: &str, power: i32, kind: CreatureType) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power,
        toughness: power,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![kind], ..Default::default() },
        ..Default::default()
    }
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn to_end_step(g: &mut GameState) {
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(g);
}

/// Lightning Serpent enters with X +1/+0 counters and is sacrificed at the
/// beginning of the end step.
#[test]
fn lightning_serpent_hits_for_two_plus_x_then_goes() {
    let mut g = pod(3);
    let serpent = g.add_card_to_hand(0, catalog::lightning_serpent());
    cast_with(&mut g, serpent, None, vec![], Some(4), None);
    assert_eq!(pt(&g, serpent), (6, 1), "2/1 plus four +1/+0 counters");
    to_end_step(&mut g);
    assert!(g.battlefield_find(serpent).is_none(), "sacrificed at the end step");
}

/// Enthralling Victor takes only a creature with power 2 or less, untapped
/// and hasty, until end of turn.
#[test]
fn enthralling_victor_borrows_a_small_creature() {
    let mut g = pod(3);
    let small = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let big = g.add_token_to_battlefield(2, &body("Beast", 3, CreatureType::Beast));
    g.battlefield_find_mut(small).unwrap().tapped = true;
    let victor = g.add_card_to_hand(0, catalog::enthralling_victor());
    cast(&mut g, victor, None);
    let bear = g.battlefield_find(small).unwrap();
    assert_eq!(bear.controller, 0, "power 2: taken");
    assert!(!bear.tapped, "and untapped");
    assert_eq!(g.battlefield_find(big).unwrap().controller, 2, "power 3 is no target");
}

/// Hall: {4}, {T}: investigate.
#[test]
fn hall_investigates() {
    let mut g = pod(2);
    let hall = g.add_card_to_battlefield(0, catalog::hall());
    activate(&mut g, hall, 2, None).expect("investigate");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Clue").count(), 1);
}

/// Sculpted Perfection incubates 2 and pumps Phyrexians only.
#[test]
fn sculpted_perfection_incubates_and_pumps_phyrexians() {
    let mut g = pod(2);
    let phyrexian = g.add_token_to_battlefield(0, &body("Phyrexian", 2, CreatureType::Phyrexian));
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let perfection = g.add_card_to_hand(0, catalog::sculpted_perfection());
    cast(&mut g, perfection, None);
    assert_eq!(pt(&g, phyrexian), (3, 3));
    assert_eq!(pt(&g, bear), (2, 2));
    let incubator = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Incubator").expect("incubated");
    assert_eq!(incubator.counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Grim Captain's Call returns one card of each of the four types, chosen on
/// resolution, and leaves the rest.
#[test]
fn grim_captains_call_returns_one_of_each_type() {
    let mut g = pod(2);
    let mut ids = Vec::new();
    for def in [
        catalog::dinosaur_hunter(),          // Pirate
        catalog::cleaving_reaper(),          // neither — stays
        catalog::wrens_run_vanquisher(),     // Elf — stays
    ] {
        ids.push(g.add_card_to_graveyard(0, def));
    }
    let call = g.add_card_to_hand(0, catalog::grim_captains_call());
    cast(&mut g, call, None);
    let in_hand = |id: CardId| g.players[0].hand.iter().any(|c| c.id == id);
    assert!(in_hand(ids[0]), "the Pirate came back");
    assert!(!in_hand(ids[1]) && !in_hand(ids[2]));
}

/// Hunter of Eyeblights marks an opposing creature and then destroys it.
#[test]
fn hunter_of_eyeblights_marks_then_kills() {
    let mut g = pod(3);
    let theirs = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let hunter = g.add_card_to_hand(0, catalog::hunter_of_eyeblights());
    cast(&mut g, hunter, Some(Target::Permanent(theirs)));
    assert_eq!(g.battlefield_find(theirs).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    g.battlefield_find_mut(hunter).unwrap().summoning_sick = false;
    activate(&mut g, hunter, 0, Some(Target::Permanent(theirs))).expect("destroy");
    assert!(g.battlefield_find(theirs).is_none());
}

/// CR 702.112 — Honored Hierarch taps for mana only once renowned.
#[test]
fn cr_702_112_honored_hierarch_needs_renown_to_tap_for_mana() {
    let mut g = pod(2);
    let hierarch = g.add_card_to_battlefield(0, catalog::honored_hierarch());
    g.battlefield_find_mut(hierarch).unwrap().summoning_sick = false;
    assert!(activate(&mut g, hierarch, 0, None).is_err(), "not renowned yet");
    g.battlefield_find_mut(hierarch).unwrap().renowned = true;
    activate(&mut g, hierarch, 0, None).expect("renowned: taps for mana");
    assert!(g.computed_permanent(hierarch).unwrap().keywords().contains(&crabomination::card::Keyword::Vigilance));
}

/// Cleaving Reaper returns from the graveyard only on a turn an Angel or
/// Berserker entered under your control.
#[test]
fn cleaving_reaper_recurs_after_an_angel_entered() {
    let mut g = pod(2);
    let reaper = g.add_card_to_graveyard(0, catalog::cleaving_reaper());
    assert!(activate(&mut g, reaper, 0, None).is_err(), "no Angel or Berserker entered");
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    cast(&mut g, angel, None);
    let life = g.players[0].life;
    activate(&mut g, reaper, 0, None).expect("an Angel entered");
    assert!(g.players[0].hand.iter().any(|c| c.id == reaper));
    assert_eq!(g.players[0].life, life - 3);
}

/// Apocalypse Demon is as big as your graveyard and taps at upkeep unless you
/// sacrifice another creature.
#[test]
fn apocalypse_demon_counts_the_graveyard_and_taps_without_a_sacrifice() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    let demon = g.add_card_to_battlefield(0, catalog::apocalypse_demon());
    assert_eq!(pt(&g, demon), (4, 4));
    g.step = TurnStep::Untap;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    while g.step != TurnStep::Draw {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(demon).unwrap().tapped, "no sacrifice: tapped");
}

/// Return from Extinction's second mode needs two cards that share a type.
#[test]
fn return_from_extinction_returns_two_that_share_a_type() {
    let mut g = pod(2);
    let a = g.add_card_to_graveyard(0, catalog::dinosaur_hunter()); // Human Pirate
    let b = g.add_card_to_graveyard(0, catalog::enthralling_victor()); // Human Warrior
    let c = g.add_card_to_graveyard(0, catalog::nectar_faerie()); // Faerie Wizard
    let spell = g.add_card_to_hand(0, catalog::return_from_extinction());
    flood(&mut g, 0);
    let bad = g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(c)],
        mode: Some(1),
        x_value: None,
    });
    assert!(bad.is_err(), "a Human and a Faerie share no type");
    cast_with(&mut g, spell, Some(Target::Permanent(a)), vec![Target::Permanent(b)], None, Some(1));
    let in_hand = |id: CardId| g.players[0].hand.iter().any(|c| c.id == id);
    assert!(in_hand(a) && in_hand(b));
}

/// Dinosaur Hunter destroys a Dinosaur it damages, and nothing else.
#[test]
fn dinosaur_hunter_kills_the_dinosaur_it_fights() {
    let mut g = pod(2);
    let hunter = g.add_card_to_battlefield(0, catalog::dinosaur_hunter());
    let dino = g.add_token_to_battlefield(1, &body("Dinosaur", 8, CreatureType::Dinosaur));
    let bear = g.add_token_to_battlefield(1, &body("Bear", 8, CreatureType::Bear));
    let fight = |g: &mut GameState, other: CardId| {
        let bite = g.add_card_to_hand(0, catalog::prey_upon());
        cast_with(g, bite, Some(Target::Permanent(hunter)), vec![Target::Permanent(other)], None, None);
    };
    fight(&mut g, bear);
    assert!(g.battlefield_find(bear).is_some(), "not a Dinosaur");
    let hunter2 = g.add_card_to_battlefield(0, catalog::dinosaur_hunter());
    let bite = g.add_card_to_hand(0, catalog::prey_upon());
    cast_with(&mut g, bite, Some(Target::Permanent(hunter2)), vec![Target::Permanent(dino)], None, None);
    assert!(g.battlefield_find(dino).is_none(), "the Dinosaur is destroyed");
}

/// Pack Attack: +X/+0 where X is the number of players being attacked.
#[test]
fn pack_attack_counts_the_attacked_players() {
    let mut g = pod(4);
    let a = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let b = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    for id in [a, b] {
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    let attack = g.add_card_to_hand(0, catalog::pack_attack());
    cast(&mut g, attack, None);
    assert_eq!(pt(&g, a), (4, 2), "two players attacked");
}

/// Savage Alliance's third mode pings each creature ONE target opponent
/// controls (CR 115.1: a target player, not each opponent).
#[test]
fn savage_alliance_pings_one_opponents_creatures() {
    let mut g = pod(3);
    let theirs = g.add_token_to_battlefield(1, &body("Squirrel", 1, CreatureType::Squirrel));
    let other = g.add_token_to_battlefield(2, &body("Squirrel", 1, CreatureType::Squirrel));
    let spell = g.add_card_to_hand(0, catalog::savage_alliance());
    cast_with(&mut g, spell, Some(Target::Player(2)), vec![], None, Some(2));
    assert!(g.battlefield_find(other).is_none(), "seat 2's creature took 1");
    assert!(g.battlefield_find(theirs).is_some(), "seat 1 was not targeted");
}

/// CR 711.2 — Caravan Escort at level 5 is a 5/5 first striker.
#[test]
fn cr_711_2_caravan_escort_levels_into_a_first_striker() {
    let mut g = pod(2);
    let escort = g.add_card_to_battlefield(0, catalog::caravan_escort());
    for _ in 0..5 {
        activate(&mut g, escort, 0, None).expect("level up");
    }
    assert_eq!(pt(&g, escort), (5, 5));
    assert!(g.computed_permanent(escort).unwrap().keywords().contains(&crabomination::card::Keyword::FirstStrike));
}

/// Elegant Entourage pumps another creature when an ally enters.
#[test]
fn elegant_entourage_pumps_on_alliance() {
    let mut g = pod(2);
    let entourage = g.add_card_to_battlefield(0, catalog::elegant_entourage());
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let wolves = g.add_card_to_hand(0, catalog::timber_wolves());
    cast(&mut g, wolves, None);
    assert_eq!(pt(&g, entourage), (4, 4), "not itself");
    assert_eq!(pt(&g, bear).0 + pt(&g, wolves).0, 2 + 1 + 1, "another creature got +1/+1");
}

/// Jodah's Codex costs {1} less per basic land type you control.
#[test]
fn jodahs_codex_is_cheaper_with_domain() {
    let mut g = pod(2);
    let codex = g.add_card_to_battlefield(0, catalog::jodahs_codex());
    for land in [catalog::plains(), catalog::island(), catalog::swamp(), catalog::mountain()] {
        g.add_card_to_battlefield(0, land);
    }
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: codex,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("four basic types: {1}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Timely Interference kicked makes the creature block this turn.
#[test]
fn timely_interference_kicked_forces_a_block() {
    let mut g = pod(2);
    let theirs = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let spell = g.add_card_to_hand(0, catalog::timely_interference());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: spell,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked");
    drain_stack(&mut g);
    let c = g.computed_permanent(theirs).unwrap();
    assert_eq!(c.power, 1);
    assert!(c.keywords().contains(&crabomination::card::Keyword::MustBlock));
}

/// Magus of the Jar: everyone draws seven off a stashed hand.
#[test]
fn magus_of_the_jar_draws_everyone_seven() {
    let mut g = pod(3);
    let magus = g.add_card_to_battlefield(0, catalog::magus_of_the_jar());
    g.battlefield_find_mut(magus).unwrap().summoning_sick = false;
    g.add_card_to_hand(1, catalog::island());
    activate(&mut g, magus, 0, None).expect("activate");
    assert!(g.battlefield_find(magus).is_none(), "sacrificed");
    for seat in 0..3 {
        assert_eq!(g.players[seat].hand.len(), 7);
    }
}

/// Grim Giganotosaurus' monstrosity is cheaper per big opposing creature and
/// wipes every other artifact and creature.
#[test]
fn grim_giganotosaurus_goes_monstrous_and_wipes() {
    let mut g = pod(2);
    let dino = g.add_card_to_battlefield(0, catalog::grim_giganotosaurus());
    let big = g.add_token_to_battlefield(1, &body("Beast", 4, CreatureType::Beast));
    let mine = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(9);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dino,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{9}{B}{G} with one power-4 opposing creature");
    drain_stack(&mut g);
    assert_eq!(pt(&g, dino), (20, 20));
    assert!(g.battlefield_find(big).is_none() && g.battlefield_find(mine).is_none());
}

/// Emptiness evoked for {W}{W} returns a small creature and is sacrificed;
/// the {B}{B} half does not fire.
#[test]
fn emptiness_evoked_with_white_reanimates() {
    let mut g = pod(2);
    let victim = g.add_card_to_graveyard(0, catalog::dinosaur_hunter());
    let theirs = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let emptiness = g.add_card_to_hand(0, catalog::emptiness());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: emptiness,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("evoke");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_some(), "{{W}}{{W}} spent: reanimated");
    assert!(g.battlefield_find(emptiness).is_none(), "evoked: sacrificed");
    assert_eq!(g.battlefield_find(theirs).unwrap().counter_count(CounterType::MinusOneMinusOne), 0);
}

/// Flitting Guerrilla's death mills everyone two, then stacks a creature card.
#[test]
fn flitting_guerrilla_mills_and_stacks_a_creature() {
    let mut g = pod(2);
    let flitter = g.add_card_to_battlefield(0, catalog::flitting_guerrilla());
    let keeper = g.add_card_to_graveyard(0, catalog::dinosaur_hunter());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let libs: Vec<usize> = g.players.iter().map(|p| p.library.len()).collect();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(flitter)));
    assert_eq!(g.players[1].library.len(), libs[1] - 2);
    assert!(g.exile.iter().any(|c| c.id == flitter), "exiled itself");
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(keeper), "the creature card is on top");
}

/// Jace's Mindseeker casts an instant milled from the opponent for free.
#[test]
fn jaces_mindseeker_casts_a_milled_instant() {
    let mut g = multi_player_game(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_library(1, catalog::lightning_bolt());
    for _ in 0..6 {
        g.add_card_to_library(1, catalog::island());
    }
    let victim = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let seeker = g.add_card_to_hand(0, catalog::jaces_mindseeker());
    cast(&mut g, seeker, Some(Target::Player(1)));
    assert_eq!(g.players[1].library.len(), 2, "milled five");
    assert!(g.battlefield_find(victim).is_none() || g.players[1].life < life, "the Bolt was cast");
}

/// CR 701.49 — under an opponent's Keen-Eared Sentry a player ventures once a
/// turn; the Sentry's own controller is not limited, and the limit resets.
#[test]
fn cr_701_49_keen_eared_sentry_limits_opponents_to_one_venture_a_turn() {
    let mut g = pod(3);
    g.add_card_to_battlefield(1, catalog::keen_eared_sentry());
    let room = |g: &GameState, seat: usize| g.players[seat].dungeon.as_ref().map(|(_, at)| *at);
    for _ in 0..2 {
        let apprentice = g.add_card_to_hand(0, catalog::eccentric_apprentice());
        cast(&mut g, apprentice, None);
    }
    assert_eq!(room(&g, 0), Some(0), "the second venture this turn was refused");
    g.players[0].ventured_this_turn = false; // a new turn
    let apprentice = g.add_card_to_hand(0, catalog::eccentric_apprentice());
    cast(&mut g, apprentice, None);
    assert_eq!(room(&g, 0), Some(1), "next turn: ventures again");
}

/// Keen-Eared Sentry gives its controller hexproof against opponents.
#[test]
fn keen_eared_sentry_gives_you_hexproof() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::keen_eared_sentry());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    let r = g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(r.is_err(), "seat 1 has hexproof");
}

/// Once Upon a Time is free only as the first spell you cast this game.
#[test]
fn once_upon_a_time_is_free_only_as_the_first_spell() {
    let mut g = pod(2);
    let first = g.add_card_to_hand(0, catalog::once_upon_a_time());
    let second = g.add_card_to_hand(0, catalog::once_upon_a_time());
    g.priority.player_with_priority = 0;
    let free = |g: &mut GameState, id: CardId| {
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: id,
            pitch_card: None,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    free(&mut g, first).expect("first spell this game: free");
    drain_stack(&mut g);
    assert!(free(&mut g, second).is_err(), "no longer the first spell");
}

/// Veinfire Borderpost's alternative cost: {1} and a basic land back to hand.
#[test]
fn veinfire_borderpost_returns_a_basic_for_its_alternative_cost() {
    let mut g = pod(2);
    let swamp = g.add_card_to_battlefield(0, catalog::swamp());
    let post = g.add_card_to_hand(0, catalog::veinfire_borderpost());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: post,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{1} and a basic land");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == swamp), "the Swamp went home");
    assert!(g.battlefield_find(post).unwrap().tapped, "enters tapped");
}

/// Scout's Warning: the next creature spell has flash; a noncreature spell
/// in between does not spend it, and the creature after the first does not
/// get it.
#[test]
fn scouts_warning_flashes_only_the_next_creature() {
    let mut g = pod(2);
    let warning = g.add_card_to_hand(0, catalog::scouts_warning());
    cast(&mut g, warning, None);
    g.step = TurnStep::End;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    let first = g.add_card_to_hand(0, catalog::timber_wolves());
    cast(&mut g, first, None);
    assert!(g.battlefield_find(first).is_some(), "the next creature had flash");
    let second = g.add_card_to_hand(0, catalog::timber_wolves());
    flood(&mut g, 0);
    let r = g.perform_action(GameAction::CastSpell {
        card_id: second,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(r.is_err(), "spent by the first creature");
}

/// Wolf of Devil's Breach pitches a card for damage equal to its mana value.
#[test]
fn wolf_of_devils_breach_discards_for_damage() {
    let mut g = pod(2);
    let wolf = g.add_card_to_battlefield(0, catalog::wolf_of_devils_breach());
    g.battlefield_find_mut(wolf).unwrap().summoning_sick = false;
    let target = g.add_token_to_battlefield(1, &body("Beast", 5, CreatureType::Beast));
    g.add_card_to_hand(0, catalog::apocalypse_demon()); // mana value 6
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: wolf, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "6 damage to the 5/5");
    assert!(g.players[0].hand.is_empty(), "the card was discarded");
}

/// Realm Razer exiles every land and returns them tapped when it leaves.
#[test]
fn realm_razer_exiles_all_lands_until_it_leaves() {
    let mut g = pod(3);
    let mine = g.add_card_to_battlefield(0, catalog::forest());
    let theirs = g.add_card_to_battlefield(1, catalog::island());
    let razer = g.add_card_to_hand(0, catalog::realm_razer());
    cast(&mut g, razer, None);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(razer)));
    let back = g.battlefield_find(theirs).expect("returned");
    assert!(back.tapped && back.controller == 1, "tapped, under its owner");
}

/// Out of Bounds counters a spell.
#[test]
fn out_of_bounds_counters_a_spell() {
    let mut g = pod(2);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    let life = g.players[0].life;
    let counter = g.add_card_to_hand(0, catalog::out_of_bounds());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "countered");
}

/// Consecrate Land makes its land indestructible.
#[test]
fn consecrate_land_makes_the_land_indestructible() {
    let mut g = pod(2);
    let land = g.add_card_to_battlefield(0, catalog::plains());
    let aura = g.add_card_to_hand(0, catalog::consecrate_land());
    cast(&mut g, aura, Some(Target::Permanent(land)));
    assert!(g.computed_permanent(land).unwrap().keywords().contains(&crabomination::card::Keyword::Indestructible));
}

/// CR 702.167a — Throne of the Grim Captain crafts with one each of four
/// types (a missing one refuses the craft); The Grim Captain's attack makes
/// each opponent sacrifice and returns a crafted creature card attacking.
#[test]
fn cr_702_167a_throne_crafts_with_four_types_and_the_captain_attacks() {
    let mut g = pod(3);
    let throne = g.add_card_to_battlefield(0, catalog::throne_of_the_grim_captain());
    g.add_token_to_battlefield(0, &body("Dinosaur", 3, CreatureType::Dinosaur));
    g.add_token_to_battlefield(0, &body("Merfolk", 1, CreatureType::Merfolk));
    let vampire = g.add_card_to_graveyard(0, catalog::vampire_nighthawk());
    assert!(activate(&mut g, throne, 1, None).is_err(), "no Pirate: no craft");
    g.add_card_to_graveyard(0, catalog::dinosaur_hunter()); // Human Pirate
    activate(&mut g, throne, 1, None).expect("craft");
    let captain = g.battlefield_find(throne).expect("returned transformed");
    assert_eq!(captain.definition.name, "The Grim Captain");
    let doomed = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let doomed2 = g.add_token_to_battlefield(2, &body("Bear", 2, CreatureType::Bear));
    g.battlefield_find_mut(throne).unwrap().summoning_sick = false;
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 4]));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: throne, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(doomed).is_none() && g.battlefield_find(doomed2).is_none(), "each opponent sacrificed");
    let back = g.battlefield_find(vampire).expect("the Vampire came back");
    assert!(back.tapped && g.attacking.iter().any(|a| a.attacker == vampire), "tapped and attacking");
}

fn loyalty(g: &mut GameState, id: CardId, index: usize) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target: None, x_value: None })
        .expect("loyalty");
    drain_stack(g);
}

/// Narset Transcendent's +1 takes a noncreature, nonland top card and leaves a
/// land on top.
#[test]
fn narset_transcendent_plus_one_takes_a_spell_not_a_land() {
    let mut g = multi_player_game(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    let narset = g.add_card_to_battlefield(0, catalog::narset_transcendent());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 2]));
    let hand = g.players[0].hand.len();
    loyalty(&mut g, narset, 0);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the Bolt");
    g.battlefield_find_mut(narset).unwrap().loyalty_uses_this_turn = 0; // a new turn
    loyalty(&mut g, narset, 0);
    assert_eq!(g.players[0].hand.len(), hand + 1, "a land stays on top");
    assert_eq!(g.players[0].library.len(), 1);
}

/// CR 114.4 — Narset's emblem stops opponents' noncreature spells, not
/// creature spells, and not its owner's.
#[test]
fn cr_114_4_narset_emblem_locks_opponents_noncreature_spells() {
    let mut g = pod(3);
    let narset = g.add_card_to_battlefield(0, catalog::narset_transcendent());
    g.battlefield_find_mut(narset).unwrap().add_counters(CounterType::Loyalty, 3);
    loyalty(&mut g, narset, 2);
    let try_cast = |g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition, target: Option<Target>| {
        let id = g.add_card_to_hand(seat, def);
        flood(g, seat);
        g.priority.player_with_priority = seat;
        g.active_player_idx = seat;
        let r = g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None });
        drain_stack(g);
        r.is_ok()
    };
    assert!(!try_cast(&mut g, 1, catalog::lightning_bolt(), Some(Target::Player(2))), "a noncreature spell");
    assert!(try_cast(&mut g, 1, catalog::timber_wolves(), None), "a creature spell");
    assert!(try_cast(&mut g, 0, catalog::lightning_bolt(), Some(Target::Player(2))), "its owner");
}

/// Elite Arcanist stores an instant and casts copies of it for X = its mana
/// value.
#[test]
fn elite_arcanist_casts_copies_of_its_instant() {
    let mut g = pod(2);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let arcanist = g.add_card_to_hand(0, catalog::elite_arcanist());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 4]));
    cast(&mut g, arcanist, None);
    assert!(g.exile.iter().any(|c| c.id == bolt), "the Bolt is exiled");
    g.battlefield_find_mut(arcanist).unwrap().summoning_sick = false;
    let life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: arcanist,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{1}: X is the Bolt's mana value");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "a copy of the Bolt");
    assert!(g.exile.iter().any(|c| c.id == bolt), "the card stays exiled");
}

/// Minion Reflector copies an entering nontoken creature for {2}.
#[test]
fn minion_reflector_copies_an_entering_creature() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::minion_reflector());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 2]));
    let wolves = g.add_card_to_hand(0, catalog::timber_wolves());
    cast(&mut g, wolves, None);
    let copies = g.battlefield.iter().filter(|c| c.definition.name == "Timber Wolves").count();
    assert_eq!(copies, 2, "the card and its token copy");
    to_end_step(&mut g);
    let copies = g.battlefield.iter().filter(|c| c.definition.name == "Timber Wolves").count();
    assert_eq!(copies, 1, "the copy is sacrificed at the end step");
}

/// CR 603.8 — Phyrexian Devourer grows by the exiled card's mana value and is
/// sacrificed once its power reaches 7.
#[test]
fn cr_603_8_phyrexian_devourer_grows_then_sacrifices_itself() {
    let mut g = multi_player_game(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_library(0, catalog::lightning_bolt()); // mana value 1
    g.add_card_to_library(0, catalog::apocalypse_demon()); // mana value 6
    let devourer = g.add_card_to_battlefield(0, catalog::phyrexian_devourer());
    activate(&mut g, devourer, 0, None).expect("exile the Bolt");
    assert_eq!(pt(&g, devourer), (2, 2));
    activate(&mut g, devourer, 0, None).expect("exile the Demon");
    assert!(g.battlefield_find(devourer).is_none(), "power 8: sacrificed");
}

/// CR 119.4 — Dash Hopes: the targeted spell's caster pays 5 life to counter
/// it when it can spare the life; at 7 life it cannot keep its margin, and
/// Dash Hopes resolves.
#[test]
fn cr_119_4_dash_hopes_is_countered_by_a_life_payment() {
    for (life, countered) in [(20, true), (7, false)] {
        let mut g = pod(3);
        g.players[1].life = life;
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(2)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        let hopes = g.add_card_to_hand(0, catalog::dash_hopes());
        flood(&mut g, 0);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: hopes,
            target: Some(Target::Permanent(bolt)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("dash hopes");
        let seat2 = g.players[2].life;
        drain_stack(&mut g);
        if countered {
            assert_eq!(g.players[1].life, life - 5, "paid 5 life");
            assert_eq!(g.players[2].life, seat2 - 3, "Dash Hopes was countered; the Bolt resolved");
        } else {
            assert_eq!(g.players[1].life, life);
            assert_eq!(g.players[2].life, seat2, "the Bolt was countered");
        }
    }
}
