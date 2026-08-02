//! Odyssey (ODY) gap wave 1 — Threshold, flashback and the graveyard shell.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Stock seven cards into `seat`'s graveyard so Threshold turns on.
fn fill_graveyard(g: &mut GameState, seat: usize) {
    for _ in 0..7 {
        g.add_card_to_graveyard(seat, catalog::forest());
    }
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// The sac-land cycle enters tapped and cracks for any colour.
#[test]
fn sac_land_enters_tapped_and_cracks_for_any_color() {
    let mut g = main_phase();
    let land = g.add_card_to_hand(0, catalog::abandoned_outpost());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    assert!(g.battlefield_find(land).unwrap().tapped, "it enters tapped");
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("crack");
    assert!(g.battlefield_find(land).is_none(), "the land was sacrificed");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

// ── Threshold ───────────────────────────────────────────────────────────────

/// Mystic Zealot is a ground 2/4 until Threshold turns it into a 3/5 flier.
#[test]
fn mystic_zealot_grows_and_flies_past_threshold() {
    let mut g = main_phase();
    let zealot = g.add_card_to_battlefield(0, catalog::mystic_zealot());
    let cp = g.computed_permanent(zealot).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4));
    assert!(!cp.keywords.contains(&Keyword::Flying));
    fill_graveyard(&mut g, 0);
    let cp = g.computed_permanent(zealot).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 5));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Frightcrawler swells but loses the ability to block past Threshold.
#[test]
fn frightcrawler_cant_block_past_threshold() {
    let mut g = main_phase();
    let crawler = g.add_card_to_battlefield(0, catalog::frightcrawler());
    assert!(!g.computed_permanent(crawler).unwrap().keywords.contains(&Keyword::CantBlock));
    fill_graveyard(&mut g, 0);
    let cp = g.computed_permanent(crawler).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::CantBlock));
}

/// Krosan Avenger's regeneration is Threshold-gated.
#[test]
fn krosan_avenger_regenerates_only_past_threshold() {
    let mut g = main_phase();
    let avenger = g.add_card_to_battlefield(0, catalog::krosan_avenger());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: avenger,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "no Threshold, no regeneration"
    );
    fill_graveyard(&mut g, 0);
    activate(&mut g, 0, avenger, 0, None);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(avenger)));
    assert!(g.battlefield_find(avenger).is_some(), "the shield saved it");
}

// ── Flashback ───────────────────────────────────────────────────────────────

/// Chatter of the Squirrel makes a token now and a second one from the yard.
#[test]
fn chatter_of_the_squirrel_flashes_back() {
    let mut g = main_phase();
    let chatter = g.add_card_to_hand(0, catalog::chatter_of_the_squirrel());
    cast(&mut g, 0, chatter, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").count(), 1);
    mana(&mut g, 0);
    g.perform_action(GameAction::CastFlashback {
        card_id: chatter,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flashback");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").count(), 2);
    assert!(g.exile.iter().any(|c| c.id == chatter), "flashback exiles it");
}

// ── Graveyard-count payoffs ─────────────────────────────────────────────────

/// Muscle Burst counts its own copies in every graveyard.
#[test]
fn muscle_burst_scales_with_its_copies() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::muscle_burst());
    let burst = g.add_card_to_hand(0, catalog::muscle_burst());
    cast(&mut g, 0, burst, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (2 + 4, 2 + 4), "3 base + 1 copy");
}

/// Ghastly Demise only kills what your graveyard can cover.
#[test]
fn ghastly_demise_needs_a_full_graveyard() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let demise = g.add_card_to_hand(0, catalog::ghastly_demise());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: demise,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "an empty graveyard covers nothing"
    );
    fill_graveyard(&mut g, 0);
    cast(&mut g, 0, demise, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "toughness 2 ≤ 7 cards");
}

// ── The rest of the shell ───────────────────────────────────────────────────

/// Thought Nibbler shaves two off its controller's maximum hand size.
#[test]
fn thought_nibbler_shrinks_your_hand_size() {
    let mut g = main_phase();
    assert_eq!(g.effective_max_hand_size(0), Some(7));
    g.add_card_to_battlefield(0, catalog::thought_nibbler());
    assert_eq!(g.effective_max_hand_size(0), Some(5));
    assert_eq!(g.effective_max_hand_size(1), Some(7), "only its controller");
}

/// Sphere of Law shaves two off every red source's damage to you.
#[test]
fn sphere_of_law_softens_red_damage() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::sphere_of_law());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let before = g.players[0].life;
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, before - 1, "3 damage minus 2");
}

/// Cease-Fire locks the target player out of creature spells for the turn.
#[test]
fn cease_fire_locks_out_creature_spells() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::forest());
    let fire = g.add_card_to_hand(0, catalog::cease_fire());
    cast(&mut g, 0, fire, Some(Target::Player(1)));
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "creature spells are locked"
    );
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("noncreature spells are fine");
}

/// Soulcatcher grows on every dying flier, but not on ground creatures.
#[test]
fn soulcatcher_counts_dying_fliers() {
    let mut g = main_phase();
    let catcher = g.add_card_to_battlefield(0, catalog::soulcatcher());
    let flier = g.add_card_to_battlefield(1, catalog::storm_crow());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(ground)));
    assert_eq!(g.battlefield_find(catcher).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(flier)));
    assert_eq!(g.battlefield_find(catcher).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Rabid Elephant scales with the size of the gang blocking it.
#[test]
fn rabid_elephant_punishes_gang_blocks() {
    let mut g = main_phase();
    let ele = g.add_card_to_battlefield(0, catalog::rabid_elephant());
    g.battlefield_find_mut(ele).unwrap().summoning_sick = false;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ele,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(a, ele), (b, ele)]))
        .expect("double block");
    drain_stack(&mut g);
    let cp = g.computed_permanent(ele).unwrap();
    assert_eq!((cp.power, cp.toughness), (3 + 4, 4 + 4));
}

/// Standstill cashes itself in when anyone casts a spell.
#[test]
fn standstill_refills_the_casters_opponents() {
    let mut g = main_phase();
    let still = g.add_card_to_battlefield(0, catalog::standstill());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let before = g.players[0].hand.len();
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert!(g.battlefield_find(still).is_none(), "Standstill sacrificed itself");
    assert_eq!(g.players[0].hand.len(), before + 3, "the caster's opponent drew three");
}

/// Aven Windreader turns a library top face up until it moves.
#[test]
fn aven_windreader_reveals_a_library_top() {
    let mut g = main_phase();
    let reader = g.add_card_to_battlefield(0, catalog::aven_windreader());
    g.battlefield_find_mut(reader).unwrap().summoning_sick = false;
    g.add_card_to_library(1, catalog::forest());
    activate(&mut g, 0, reader, 0, Some(Target::Player(1)));
    assert!(g.library_top_revealed_by_effect_for_test(1));
    let mut events = vec![];
    g.draw_one(1, &mut events);
    assert!(
        !g.library_top_revealed_by_effect_for_test(1),
        "the reveal ends when the top moves"
    );
}

// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// Kamahl's Desire grants first strike now and +3/+0 past Threshold.
#[test]
fn kamahls_desire_scales_with_threshold() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::kamahls_desire());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    let cp = g.computed_permanent(host).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
    fill_graveyard(&mut g, 0);
    assert_eq!(g.computed_permanent(host).unwrap().power, 5);
}

/// Gorilla Titan is an 8/8 only while your graveyard is empty.
#[test]
fn gorilla_titan_shrinks_once_you_have_a_graveyard() {
    let mut g = main_phase();
    let titan = g.add_card_to_battlefield(0, catalog::gorilla_titan());
    assert_eq!(g.computed_permanent(titan).unwrap().power, 8);
    g.add_card_to_graveyard(0, catalog::forest());
    assert_eq!(g.computed_permanent(titan).unwrap().power, 4);
}

/// Thermal Blast upgrades from 3 to 5 damage past Threshold.
#[test]
fn thermal_blast_hits_harder_past_threshold() {
    let mut g = main_phase();
    let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    let blast = g.add_card_to_hand(0, catalog::thermal_blast());
    cast(&mut g, 0, blast, Some(Target::Permanent(big)));
    assert_eq!(g.battlefield_find(big).unwrap().damage, 3, "3 without Threshold");

    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    let blast = g.add_card_to_hand(0, catalog::thermal_blast());
    cast(&mut g, 0, blast, Some(Target::Permanent(big)));
    assert_eq!(g.battlefield_find(big).unwrap().damage, 5, "5 past Threshold");
}

/// Squirrel Nest turns the enchanted land into a token engine.
#[test]
fn squirrel_nest_makes_a_squirrel_per_tap() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let nest = g.add_card_to_hand(0, catalog::squirrel_nest());
    cast(&mut g, 0, nest, Some(Target::Permanent(land)));
    activate(&mut g, 0, land, 1, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").count(), 1);
}

/// Bearscape eats two graveyard cards per Bear.
#[test]
fn bearscape_trades_your_graveyard_for_bears() {
    let mut g = main_phase();
    let scape = g.add_card_to_battlefield(0, catalog::bearscape());
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    activate(&mut g, 0, scape, 0, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Bear").count(), 1);
    assert!(g.players[0].graveyard.is_empty(), "both cards were exiled");
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: scape,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "an empty graveyard can't pay the cost"
    );
}

/// Price of Glory destroys a land tapped for mana off-turn only.
#[test]
fn price_of_glory_punishes_off_turn_mana() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::price_of_glory());
    let mine = g.add_card_to_battlefield(0, catalog::forest());
    let theirs = g.add_card_to_battlefield(1, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: mine,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("active player taps freely");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_some(), "it's the active player's turn");
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: theirs,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "an off-turn tap loses the land");
}

/// Nantuko Mentor doubles the target's power.
#[test]
fn nantuko_mentor_doubles_power() {
    let mut g = main_phase();
    let mentor = g.add_card_to_battlefield(0, catalog::nantuko_mentor());
    g.battlefield_find_mut(mentor).unwrap().summoning_sick = false;
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    activate(&mut g, 0, mentor, 0, Some(Target::Permanent(giant)));
    let cp = g.computed_permanent(giant).unwrap();
    assert_eq!((cp.power, cp.toughness), (3 + 3, 3 + 3));
}

/// Ground Seal locks graveyard cards out of being targeted.
#[test]
fn ground_seal_protects_graveyards() {
    let mut g = main_phase();
    let before = g.players[0].hand.len();
    g.add_card_to_library(0, catalog::forest());
    let seal = g.add_card_to_hand(0, catalog::ground_seal());
    cast(&mut g, 0, seal, None);
    assert_eq!(g.players[0].hand.len(), before + 1, "the Seal cantripped");
    let victim = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let purge = g.add_card_to_hand(0, catalog::coffin_purge());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: purge,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "graveyard cards can't be targeted"
    );
}
