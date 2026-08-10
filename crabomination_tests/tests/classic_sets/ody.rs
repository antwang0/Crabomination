//! Odyssey (ODY) gap wave 1 — Threshold, flashback and the graveyard shell.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::game::effects::EntityRef;
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

fn cast_multi(
    g: &mut GameState,
    seat: usize,
    id: CardId,
    target: Option<Target>,
    additional_targets: Vec<Target>,
) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets,
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

// ── Wave 3 ──────────────────────────────────────────────────────────────────

/// Hallowed Healer's bigger shield is Threshold-gated.
#[test]
fn hallowed_healer_upgrades_past_threshold() {
    let mut g = main_phase();
    let healer = g.add_card_to_battlefield(0, catalog::hallowed_healer());
    g.battlefield_find_mut(healer).unwrap().summoning_sick = false;
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: healer,
            ability_index: 1,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the 4-point shield needs Threshold"
    );
    fill_graveyard(&mut g, 0);
    g.battlefield_find_mut(healer).unwrap().tapped = false;
    activate(&mut g, 0, healer, 1, Some(Target::Player(0)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let before = g.players[0].life;
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, before, "4 soaks a Bolt");
}

/// Master Apothecary taps a fellow Cleric rather than itself.
#[test]
fn master_apothecary_taps_a_cleric() {
    let mut g = main_phase();
    let apothecary = g.add_card_to_battlefield(0, catalog::master_apothecary());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: apothecary,
            ability_index: 0,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "no other untapped Cleric to tap"
    );
    let helper = g.add_card_to_battlefield(0, catalog::dedicated_martyr());
    g.battlefield_find_mut(helper).unwrap().summoning_sick = false;
    activate(&mut g, 0, apothecary, 0, Some(Target::Player(0)));
    assert!(g.battlefield_find(helper).unwrap().tapped, "the helper paid the cost");
}

/// Thought Devourer stacks its hand-size tax with Thought Eater's.
#[test]
fn hand_size_reductions_stack() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::thought_eater());
    assert_eq!(g.effective_max_hand_size(0), Some(4));
    g.add_card_to_battlefield(0, catalog::thought_devourer());
    assert_eq!(g.effective_max_hand_size(0), Some(0));
}

/// Painbringer's -X/-X scales with the graveyard cards it exiles.
#[test]
fn painbringer_scales_with_exiled_cards() {
    let mut g = main_phase();
    let bringer = g.add_card_to_battlefield(0, catalog::painbringer());
    g.battlefield_find_mut(bringer).unwrap().summoning_sick = false;
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bringer,
        ability_index: 0,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("activate for X=3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "a 3/3 dies to -3/-3");
    assert!(g.players[0].graveyard.is_empty(), "all three were exiled");
}

/// Whipkeeper doubles the damage already marked on a creature.
#[test]
fn whipkeeper_doubles_marked_damage() {
    let mut g = main_phase();
    let keeper = g.add_card_to_battlefield(0, catalog::whipkeeper());
    g.battlefield_find_mut(keeper).unwrap().summoning_sick = false;
    let victim = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    g.battlefield_find_mut(victim).unwrap().damage = 2;
    activate(&mut g, 0, keeper, 0, Some(Target::Permanent(victim)));
    assert_eq!(g.battlefield_find(victim).unwrap().damage, 4);
}

/// Zombie Assassin pays with two graveyard cards and itself.
#[test]
fn zombie_assassin_eats_a_nonblack_creature() {
    let mut g = main_phase();
    let assassin = g.add_card_to_battlefield(0, catalog::zombie_assassin());
    g.battlefield_find_mut(assassin).unwrap().summoning_sick = false;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: assassin,
            ability_index: 0,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "an empty graveyard can't pay"
    );
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    activate(&mut g, 0, assassin, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none(), "the nonblack creature died");
    assert!(g.battlefield_find(assassin).is_none(), "the Assassin sacrificed itself");
}

/// Aboshan taps the whole ground for {U}{U}{U}.
#[test]
fn aboshan_taps_the_ground() {
    let mut g = main_phase();
    let aboshan = g.add_card_to_battlefield(0, catalog::aboshan_cephalid_emperor());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flier = g.add_card_to_battlefield(1, catalog::storm_crow());
    activate(&mut g, 0, aboshan, 1, None);
    assert!(g.battlefield_find(ground).unwrap().tapped);
    assert!(!g.battlefield_find(flier).unwrap().tapped, "fliers are spared");
}

// ── Wave 4 ──────────────────────────────────────────────────────────────────

/// Epicenter takes one land, or every land past Threshold.
#[test]
fn epicenter_scales_with_threshold() {
    let mut g = main_phase();
    for seat in [0, 1] {
        for _ in 0..2 {
            g.add_card_to_battlefield(seat, catalog::forest());
        }
    }
    let quake = g.add_card_to_hand(0, catalog::epicenter());
    cast(&mut g, 0, quake, Some(Target::Player(1)));
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_land()).count(), 1);

    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    for seat in [0, 1] {
        for _ in 0..2 {
            g.add_card_to_battlefield(seat, catalog::forest());
        }
    }
    let quake = g.add_card_to_hand(0, catalog::epicenter());
    cast(&mut g, 0, quake, Some(Target::Player(1)));
    assert!(g.battlefield.iter().all(|c| !c.definition.is_land()), "Threshold wipes every land");
}

/// Burning Sands taxes the dead creature's controller a land.
#[test]
fn burning_sands_taxes_a_land_per_death() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::burning_sands());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(land).is_none(), "its controller lost a land");
}

/// Laquatus's Creativity swaps a hand of the same size.
#[test]
fn laquatuss_creativity_swaps_the_hand() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::forest());
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::laquatuss_creativity());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 3, "drew three, discarded three");
    assert_eq!(g.players[1].graveyard.len(), 3);
}

/// Need for Speed converts a spare land into haste.
#[test]
fn need_for_speed_sacrifices_a_land_for_haste() {
    let mut g = main_phase();
    let engine = g.add_card_to_battlefield(0, catalog::need_for_speed());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let body = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, engine, 0, Some(Target::Permanent(body)));
    assert!(g.battlefield_find(land).is_none(), "the land paid the cost");
    assert!(g.computed_permanent(body).unwrap().keywords.contains(&Keyword::Haste));
}

// ── Wave 5 ──────────────────────────────────────────────────────────────────

/// Terravore is the size of every graveyard's lands.
#[test]
fn terravore_counts_every_graveyards_lands() {
    let mut g = main_phase();
    let vore = g.add_card_to_battlefield(0, catalog::terravore());
    let cp = g.computed_permanent(vore).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 0));
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(1, catalog::forest());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let cp = g.computed_permanent(vore).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "only the lands count");
}

/// Squirrel Mob grows with each other Squirrel, and Nut Collector lords them
/// past Threshold.
#[test]
fn squirrel_mob_and_nut_collector_scale_together() {
    let mut g = main_phase();
    let mob = g.add_card_to_battlefield(0, catalog::squirrel_mob());
    assert_eq!(g.computed_permanent(mob).unwrap().power, 2);
    let nest = g.add_card_to_battlefield(0, catalog::squirrel_nest());
    let _ = nest;
    g.add_card_to_battlefield(0, catalog::squirrel_mob());
    assert_eq!(g.computed_permanent(mob).unwrap().power, 3, "one other Squirrel");
    g.add_card_to_battlefield(0, catalog::nut_collector());
    fill_graveyard(&mut g, 0);
    assert_eq!(g.computed_permanent(mob).unwrap().power, 5, "+2/+2 from Threshold");
}

/// Screams of the Damned trades graveyard cards for a board-wide ping.
#[test]
fn screams_of_the_damned_pings_everything() {
    let mut g = main_phase();
    let screams = g.add_card_to_battlefield(0, catalog::screams_of_the_damned());
    g.add_card_to_graveyard(0, catalog::forest());
    let victim = g.add_card_to_battlefield(1, catalog::raging_goblin());
    let before = g.players[1].life;
    activate(&mut g, 0, screams, 0, None);
    assert!(g.battlefield_find(victim).is_none(), "the 1/1 died");
    assert_eq!(g.players[1].life, before - 1);
    assert!(g.players[0].graveyard.is_empty(), "the cost exiled the card");
}

/// Skeletal Scrying pays X out of the graveyard, then draws and drains X.
#[test]
fn skeletal_scrying_pays_with_the_graveyard() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::forest());
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let scry = g.add_card_to_hand(0, catalog::skeletal_scrying());
    mana(&mut g, 0);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: scry,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast for X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(g.players[0].life, life - 2);
}

/// Cabal Patriarch shrinks a creature off a body or off the graveyard.
#[test]
fn cabal_patriarch_shrinks_two_ways() {
    let mut g = main_phase();
    let patriarch = g.add_card_to_battlefield(0, catalog::cabal_patriarch());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, patriarch, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none(), "-2/-2 killed the 2/2");
    assert!(g.battlefield_find(fodder).is_none(), "the body paid the cost");

    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    activate(&mut g, 0, patriarch, 1, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none(), "the graveyard half works too");
}

// ── Wave 6 ──────────────────────────────────────────────────────────────────

/// The Lhurgoyf cycle each counts its own card type across every graveyard.
#[test]
fn lhurgoyf_cycle_counts_its_card_type() {
    let mut g = main_phase();
    let cogni = g.add_card_to_battlefield(0, catalog::cognivore());
    let magni = g.add_card_to_battlefield(0, catalog::magnivore());
    assert_eq!(g.computed_permanent(cogni).unwrap().power, 0);
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::coffin_purge());
    g.add_card_to_graveyard(0, catalog::skull_fracture());
    assert_eq!(g.computed_permanent(cogni).unwrap().power, 2, "two instants");
    assert_eq!(g.computed_permanent(magni).unwrap().power, 1, "one sorcery");
}

/// Dwarven Shrine reads the cast spell's name, not its own.
#[test]
fn dwarven_shrine_counts_the_cast_spells_copies() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::dwarven_shrine());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let before = g.players[1].life;
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[1].life, before - 4, "twice two copies");
}

/// Acceptable Losses pays a random card and burns for five.
#[test]
fn acceptable_losses_pitches_at_random() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    g.add_card_to_hand(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::acceptable_losses());
    cast(&mut g, 0, spell, Some(Target::Permanent(victim)));
    assert_eq!(g.battlefield_find(victim).unwrap().damage, 5);
    assert!(g.players[0].hand.is_empty(), "the random discard was paid");
}

/// Kirtar's Wrath leaves two Spirits behind past Threshold.
#[test]
fn kirtars_wrath_leaves_spirits_past_threshold() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wrath = g.add_card_to_hand(0, catalog::kirtars_wrath());
    cast(&mut g, 0, wrath, None);
    assert!(g.battlefield.iter().all(|c| !c.definition.is_creature()));

    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wrath = g.add_card_to_hand(0, catalog::kirtars_wrath());
    cast(&mut g, 0, wrath, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count(), 2);
}

/// Junk Golem eats a counter each upkeep and dies once it runs out.
#[test]
fn junk_golem_counts_down() {
    let mut g = main_phase();
    let golem = g.add_card_to_hand(0, catalog::junk_golem());
    cast(&mut g, 0, golem, None);
    assert_eq!(g.battlefield_find(golem).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    for _ in 0..2 {
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(golem).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    // The third upkeep takes the last counter, leaving a 0/0 that dies to SBA.
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(golem).is_none(), "out of counters, out of Golem");
}

/// Decimate needs one legal target of each of its four types.
#[test]
fn decimate_takes_one_of_each() {
    let mut g = main_phase();
    let art = g.add_card_to_battlefield(1, catalog::millikin());
    let cre = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ench = g.add_card_to_battlefield(1, catalog::battle_strain());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::decimate());
    cast_multi(
        &mut g,
        0,
        spell,
        Some(Target::Permanent(art)),
        vec![Target::Permanent(cre), Target::Permanent(ench), Target::Permanent(land)],
    );
    for id in [art, cre, ench, land] {
        assert!(g.battlefield_find(id).is_none(), "everything was destroyed");
    }
}

// ── Wave 7 ──────────────────────────────────────────────────────────────────

/// The Egg cycle cracks for two coloured mana and a card.
#[test]
fn egg_cracks_for_two_colors_and_a_card() {
    let mut g = main_phase();
    let egg = g.add_card_to_battlefield(0, catalog::skycloud_egg());
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: egg,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("crack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Cabal Pit pings you for mana and only shrinks past Threshold.
#[test]
fn cabal_pit_pings_and_gates_on_threshold() {
    let mut g = main_phase();
    let pit = g.add_card_to_battlefield(0, catalog::cabal_pit());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    activate(&mut g, 0, pit, 0, None);
    assert_eq!(g.players[0].life, life - 1, "the tap cost a point");
    g.battlefield_find_mut(pit).unwrap().tapped = false;
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: pit,
            ability_index: 1,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the sac ability needs Threshold"
    );
    fill_graveyard(&mut g, 0);
    activate(&mut g, 0, pit, 1, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none(), "-2/-2 killed the 2/2");
}

/// Braids taxes every player's upkeep, including her controller's.
#[test]
fn braids_taxes_each_upkeep() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::braids_cabal_minion());
    let mine = g.add_card_to_battlefield(0, catalog::forest());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "the active player paid too");
}

/// Pianna pumps the whole attack, not just herself.
#[test]
fn pianna_pumps_the_swing() {
    let mut g = main_phase();
    let pianna = g.add_card_to_battlefield(0, catalog::pianna_nomad_captain());
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [pianna, friend] {
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: pianna, target: AttackTarget::Player(1) },
        Attack { attacker: friend, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(friend).unwrap().power, 3);
    assert_eq!(g.computed_permanent(pianna).unwrap().power, 3);
}

/// Divert repoints a spell when its caster declines to pay {2}.
#[test]
fn divert_repoints_an_unpaid_spell() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::hill_giant());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt on the stack");
    let divert = g.add_card_to_hand(0, catalog::divert());
    cast(&mut g, 0, divert, Some(Target::Permanent(bolt)));
    assert!(g.battlefield_find(mine).is_some(), "the Bolt was pointed elsewhere");
}

/// Otarian Juggernaut walks past Walls (CR 509.1b).
#[test]
fn otarian_juggernaut_ignores_walls() {
    let mut g = main_phase();
    let jug = g.add_card_to_battlefield(0, catalog::otarian_juggernaut());
    g.battlefield_find_mut(jug).unwrap().summoning_sick = false;
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_omens());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: jug,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(wall, jug)])).is_err(),
        "a Wall can't block it"
    );
}

// ── Wave 8 ──────────────────────────────────────────────────────────────────

/// Psychatog feeds on the hand and on the graveyard.
#[test]
fn psychatog_eats_hand_and_graveyard() {
    let mut g = main_phase();
    let tog = g.add_card_to_battlefield(0, catalog::psychatog());
    g.add_card_to_hand(0, catalog::forest());
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    activate(&mut g, 0, tog, 0, None);
    assert_eq!(g.computed_permanent(tog).unwrap().power, 2, "the hand fed it");
    activate(&mut g, 0, tog, 1, None);
    assert_eq!(g.computed_permanent(tog).unwrap().power, 3, "the graveyard fed it");
    // Two of the three (the two originals plus the discarded card) were exiled.
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Nomad Decoy's double tap is Threshold-gated.
#[test]
fn nomad_decoy_double_taps_past_threshold() {
    let mut g = main_phase();
    let decoy = g.add_card_to_battlefield(0, catalog::nomad_decoy());
    g.battlefield_find_mut(decoy).unwrap().summoning_sick = false;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: decoy,
            ability_index: 1,
            target: Some(Target::Permanent(a)),
            additional_targets: vec![Target::Permanent(b)],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the double tap needs Threshold"
    );
    fill_graveyard(&mut g, 0);
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: decoy,
        ability_index: 1,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("double tap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
}

/// Persuasion steals the creature for as long as it stays attached.
#[test]
fn persuasion_steals_until_it_leaves() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::persuasion());
    cast(&mut g, 0, aura, Some(Target::Permanent(victim)));
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 0);
    let _ = g.remove_to_graveyard_with_triggers(aura);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 1, "control reverts");
}

/// Demoralize hands out menace, or shuts blocking off past Threshold.
#[test]
fn demoralize_scales_with_threshold() {
    let mut g = main_phase();
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::demoralize());
    cast(&mut g, 0, spell, None);
    assert!(g.computed_permanent(blocker).unwrap().keywords.contains(&Keyword::Menace));

    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::demoralize());
    cast(&mut g, 0, spell, None);
    assert!(g.computed_permanent(blocker).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Testament of Faith stands up as an X/X Wall.
#[test]
fn testament_of_faith_animates_for_x() {
    let mut g = main_phase();
    let test = g.add_card_to_battlefield(0, catalog::testament_of_faith());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: test,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("animate for X=3");
    drain_stack(&mut g);
    let cp = g.computed_permanent(test).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::Defender));
}

// ── Wave 9 ──────────────────────────────────────────────────────────────────

/// Mindslicer's death empties every hand.
#[test]
fn mindslicer_death_empties_hands() {
    let mut g = main_phase();
    let slicer = g.add_card_to_battlefield(0, catalog::mindslicer());
    for seat in [0, 1] {
        for _ in 0..3 {
            g.add_card_to_hand(seat, catalog::forest());
        }
    }
    let mut events = Vec::new();
    g.destroy_permanent(slicer, false, &mut events);
    g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: slicer }]);
    drain_stack(&mut g);
    assert!(g.players[0].hand.is_empty() && g.players[1].hand.is_empty());
}

/// Rotting Giant eats a graveyard card to attack, and dies without one.
#[test]
fn rotting_giant_pays_with_its_graveyard() {
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(0, catalog::rotting_giant());
    g.battlefield_find_mut(giant).unwrap().summoning_sick = false;
    g.add_card_to_graveyard(0, catalog::forest());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: giant,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_some(), "the graveyard card kept it alive");
    assert!(g.players[0].graveyard.is_empty(), "the card was exiled to pay");

    // A second attack with an empty graveyard sacrifices it.
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(0, catalog::rotting_giant());
    g.battlefield_find_mut(giant).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: giant,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none(), "nothing to pay with");
}

/// Tombfire only takes the flashback cards.
#[test]
fn tombfire_exiles_only_flashback_cards() {
    let mut g = main_phase();
    let fb = g.add_card_to_graveyard(1, catalog::seize_the_day());
    let plain = g.add_card_to_graveyard(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::tombfire());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert!(g.exile.iter().any(|c| c.id == fb), "the flashback card left");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == plain), "the land stayed");
}

/// Haunting Echoes strips the graveyard and the matching library copies.
#[test]
fn haunting_echoes_strips_library_copies() {
    let mut g = main_phase();
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::forest()); // basic — stays
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::wall_of_omens());
    let spell = g.add_card_to_hand(0, catalog::haunting_echoes());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].graveyard.len(), 1, "only the basic land is left");
    assert!(
        !g.players[1].library.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the library copy went too"
    );
    assert!(g.players[1].library.iter().any(|c| c.definition.name == "Wall of Omens"));
}

/// Unifying Theory offers each caster a {2} cantrip.
#[test]
fn unifying_theory_offers_a_cantrip() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::unifying_theory());
    let before = g.players[0].hand.len();
    let spell = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, 0, spell, None);
    // AutoDecider declines the tax, so no card is drawn.
    assert_eq!(g.players[0].hand.len(), before);
}

/// Aether Burst bounces one more creature per copy already in a graveyard.
#[test]
fn aether_burst_scales_with_its_own_copies() {
    let mut g = main_phase();
    g.add_card_to_graveyard(0, catalog::aether_burst());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::aether_burst());
    cast_multi(&mut g, 0, spell, Some(Target::Permanent(a)), vec![Target::Permanent(b)]);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Seize the Day untaps an attacker and buys another combat.
#[test]
fn seize_the_day_grants_an_extra_combat() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::seize_the_day());
    cast(&mut g, 0, spell, Some(Target::Permanent(bear)));
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(g.additional_post_main_combats, 1);
}

/// Repentant Vampire turns white and gains a tapper past Threshold.
#[test]
fn repentant_vampire_turns_white_past_threshold() {
    let mut g = main_phase();
    let vamp = g.add_card_to_battlefield(0, catalog::repentant_vampire());
    assert!(!g.computed_permanent(vamp).unwrap().colors.contains(Color::White));
    assert!(g.granted_abilities_for(vamp).is_empty());
    fill_graveyard(&mut g, 0);
    assert!(g.computed_permanent(vamp).unwrap().colors.contains(Color::White));
    assert_eq!(g.granted_abilities_for(vamp).len(), 1, "the Threshold tapper is live");
}

/// Wayward Angel goes black, bigger and hungrier past Threshold.
#[test]
fn wayward_angel_falls_past_threshold() {
    let mut g = main_phase();
    let angel = g.add_card_to_battlefield(0, catalog::wayward_angel());
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    fill_graveyard(&mut g, 0);
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7));
    assert!(cp.keywords.contains(&Keyword::Trample));
    assert!(cp.colors.contains(Color::Black));
}

/// Stone-Tongue Basilisk lures every blocker past Threshold.
#[test]
fn stone_tongue_basilisk_lures_past_threshold() {
    let mut g = main_phase();
    let basilisk = g.add_card_to_battlefield(0, catalog::stone_tongue_basilisk());
    assert!(!g.computed_permanent(basilisk).unwrap().keywords.contains(&Keyword::AllMustBlock));
    fill_graveyard(&mut g, 0);
    assert!(g.computed_permanent(basilisk).unwrap().keywords.contains(&Keyword::AllMustBlock));
}

/// Seton's Desire pumps, and lures past Threshold.
#[test]
fn setons_desire_lures_past_threshold() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::setons_desire());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(!cp.keywords.contains(&Keyword::AllMustBlock));
    fill_graveyard(&mut g, 0);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::AllMustBlock));
}

/// Verdant Succession refills from the library when a green creature dies.
#[test]
fn verdant_succession_refetches_the_dead() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::verdant_succession());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let mut events = Vec::new();
    g.destroy_permanent(bear, false, &mut events);
    g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: bear }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count(),
        1,
        "the library copy replaced it"
    );
}

/// Balancing Act trims every board down to the smallest one.
#[test]
fn balancing_act_levels_the_boards() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::balancing_act());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 1);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1).count(), 1);
}

/// Obstinate Familiar lets its controller decline a draw.
#[test]
fn obstinate_familiar_skips_a_draw() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::obstinate_familiar());
    g.add_card_to_library(0, catalog::forest());
    let before = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut events = Vec::new();
    assert!(!g.draw_one(0, &mut events), "the draw was skipped");
    assert_eq!(g.players[0].hand.len(), before);
    // Declining the skip draws normally.
    let mut events = Vec::new();
    assert!(g.draw_one(0, &mut events));
    assert_eq!(g.players[0].hand.len(), before + 1);
}

/// Decaying Soil eats a graveyard card each upkeep.
#[test]
fn decaying_soil_eats_the_graveyard_each_upkeep() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::decaying_soil());
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(0, catalog::forest());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Pulsating Illusion pumps once a turn off a discard.
#[test]
fn pulsating_illusion_pumps_once_a_turn() {
    let mut g = main_phase();
    let illusion = g.add_card_to_battlefield(0, catalog::pulsating_illusion());
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
    }
    activate(&mut g, 0, illusion, 0, None);
    let cp = g.computed_permanent(illusion).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 5));
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: illusion,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "only once each turn"
    );
}

// ── Wave 10 ─────────────────────────────────────────────────────────────────

/// Blazing Salvo burns the creature when its controller declines the 5.
#[test]
fn blazing_salvo_burns_when_the_offer_is_declined() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::blazing_salvo());
    cast(&mut g, 0, spell, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "3 damage killed the 2/2");
    assert_eq!(g.players[1].life, 20, "the face was spared");
}

/// Lava Blister eats a nonbasic land on the same decline.
#[test]
fn lava_blister_destroys_the_nonbasic() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::abandoned_outpost());
    let spell = g.add_card_to_hand(0, catalog::lava_blister());
    cast(&mut g, 0, spell, Some(Target::Permanent(land)));
    assert!(g.battlefield_find(land).is_none());
}

/// Bamboozle bins two of the four it reveals.
#[test]
fn bamboozle_bins_two_of_four() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::bamboozle());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].graveyard.len(), 2);
    assert_eq!(g.players[1].library.len(), 2);
}

/// Predict draws two on a hit and one on a miss.
#[test]
fn predict_pays_off_on_a_hit() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.add_card_to_library(1, catalog::grizzly_bears());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::predict());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard("Grizzly Bears".into())]));
    let before = g.players[0].hand.len();
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[0].hand.len(), before - 1 + 2, "named it — two cards");
}

/// Cephalid Shrine taxes a spell by its copies already in graveyards.
#[test]
fn cephalid_shrine_taxes_by_graveyard_copies() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::cephalid_shrine());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    // The trigger auto-pays the {1} tax off the caster's floating mana.
    cast(&mut g, 0, bear, None);
    assert!(g.battlefield_find(bear).is_some(), "the tax was paid, so it resolved");
}

/// Charmed Pendant banks one mana per coloured pip on the milled card.
#[test]
fn charmed_pendant_banks_the_milled_pips() {
    let mut g = main_phase();
    let pendant = g.add_card_to_battlefield(0, catalog::charmed_pendant());
    g.battlefield_find_mut(pendant).unwrap().summoning_sick = false;
    g.add_card_to_library(0, catalog::grizzly_bears()); // {1}{G}
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pendant,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "one green mana from the single green pip");
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Earnest Fellowship makes every creature dodge same-coloured removal.
#[test]
fn earnest_fellowship_grants_protection_from_own_colors() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::earnest_fellowship());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    assert!(
        g.computed_permanent(bear)
            .unwrap()
            .keywords
            .contains(&Keyword::ProtectionFromOwnColors)
    );
}

/// Savage Firecat sheds a counter every time you tap a land.
#[test]
fn savage_firecat_sheds_counters_on_land_taps() {
    let mut g = main_phase();
    let cat = g.add_card_to_battlefield(0, catalog::savage_firecat());
    g.battlefield_find_mut(cat).unwrap().add_counters(CounterType::PlusOnePlusOne, 7);
    assert_eq!(g.computed_permanent(cat).unwrap().power, 7);
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(cat).unwrap().power, 6);
}

/// Catalyst Stone discounts your flashbacks and taxes theirs.
#[test]
fn catalyst_stone_shifts_flashback_costs() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::catalyst_stone());
    let seize = g.add_card_to_graveyard(0, catalog::seize_the_day());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    // Flashback {2}{R} - {2} = {R}: one red mana is enough.
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFlashback {
        card_id: seize,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("discounted flashback");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped);
}

/// Pardic Firecat counts as a Flame Burst from the graveyard.
#[test]
fn pardic_firecat_counts_as_flame_burst() {
    let mut g = main_phase();
    g.add_card_to_graveyard(0, catalog::pardic_firecat());
    let burst = g.add_card_to_hand(0, catalog::flame_burst());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast(&mut g, 0, burst, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "2 + 1 for the Firecat killed the 2/2");
}

/// Aura Graft steals an Aura and moves it off its old host.
#[test]
fn aura_graft_moves_the_aura() {
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura_card = g.add_card_to_battlefield(1, catalog::setons_desire());
    g.battlefield_find_mut(aura_card).unwrap().attached_to = Some(theirs);
    let graft = g.add_card_to_hand(0, catalog::aura_graft());
    cast(&mut g, 0, graft, Some(Target::Permanent(aura_card)));
    assert_eq!(g.battlefield_find(aura_card).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(aura_card).unwrap().attached_to, Some(mine));
}

/// Holistic Wisdom trades a hand card for a same-type graveyard card.
#[test]
fn holistic_wisdom_buys_back_a_shared_type() {
    let mut g = main_phase();
    let wisdom = g.add_card_to_battlefield(0, catalog::holistic_wisdom());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // creature — the exile cost
    let target = g.add_card_to_graveyard(0, catalog::wall_of_omens()); // creature
    activate(&mut g, 0, wisdom, 0, Some(Target::Permanent(target)));
    assert!(g.players[0].hand.iter().any(|c| c.id == target), "shared type — it came back");
}

/// Immobilizing Ink locks the creature down until a card is paid.
#[test]
fn immobilizing_ink_locks_the_creature() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ink = g.add_card_to_hand(0, catalog::immobilizing_ink());
    cast(&mut g, 0, ink, Some(Target::Permanent(bear)));
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    assert!(g.untap_prevented_by_static(bear), "the Ink holds it down");
}

/// Spiritualize turns a creature's damage into life and cantrips.
#[test]
fn spiritualize_grants_lifelink_and_draws() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::spiritualize());
    let before = g.players[0].hand.len();
    cast(&mut g, 0, spell, Some(Target::Permanent(bear)));
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Lifelink));
    assert_eq!(g.players[0].hand.len(), before - 1 + 1);
}

/// Graceful Antelope walks over Plains.
#[test]
fn graceful_antelope_has_plainswalk() {
    let mut g = main_phase();
    let antelope = g.add_card_to_battlefield(0, catalog::graceful_antelope());
    assert!(
        g.computed_permanent(antelope)
            .unwrap()
            .keywords
            .contains(&Keyword::Landwalk(crabomination::card::LandType::Plains))
    );
}

// ── Wave 11 ─────────────────────────────────────────────────────────────────

/// Delaying Shield banks the damage as delay counters.
#[test]
fn delaying_shield_banks_damage_as_counters() {
    let mut g = main_phase();
    let shield = g.add_card_to_battlefield(0, catalog::delaying_shield());
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 3, None, &mut events);
    assert_eq!(g.players[0].life, 20, "no life was lost");
    assert_eq!(g.battlefield_find(shield).unwrap().counter_count(CounterType::Delay), 3);
}

/// Nefarious Lich pays damage out of the graveyard and cashes life gain for
/// cards.
#[test]
fn nefarious_lich_swaps_graveyard_for_life() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::nefarious_lich());
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 3, None, &mut events);
    assert_eq!(g.players[0].life, 20, "the graveyard soaked it");
    assert_eq!(g.players[0].graveyard.len(), 1);

    g.add_card_to_library(0, catalog::forest());
    let before = g.players[0].hand.len();
    g.adjust_life(0, 1);
    assert_eq!(g.players[0].life, 20, "the gain became a draw");
    assert_eq!(g.players[0].hand.len(), before + 1);
}

/// Nefarious Lich kills you when the graveyard runs dry.
#[test]
fn nefarious_lich_kills_you_when_the_graveyard_is_short() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::nefarious_lich());
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 1, None, &mut events);
    assert!(g.players[0].eliminated);
}

/// Mine Layer's counters blow up the land the moment it taps.
#[test]
fn mine_layer_destroys_a_mined_land_on_tap() {
    let mut g = main_phase();
    let layer = g.add_card_to_battlefield(0, catalog::mine_layer());
    g.battlefield_find_mut(layer).unwrap().summoning_sick = false;
    let land = g.add_card_to_battlefield(1, catalog::forest());
    activate(&mut g, 0, layer, 0, Some(Target::Permanent(land)));
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(CounterType::Mine), 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "the mine went off");
}

/// Traveling Plague shrinks its host by one more each upkeep.
#[test]
fn traveling_plague_grows_each_upkeep() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let plague = g.add_card_to_hand(0, catalog::traveling_plague());
    cast(&mut g, 0, plague, Some(Target::Permanent(bear)));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "one plague counter");
}

/// Steam Vines blows up the land it sits on when it taps.
#[test]
fn steam_vines_destroys_the_land_it_taps() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let vines = g.add_card_to_hand(0, catalog::steam_vines());
    cast(&mut g, 0, vines, Some(Target::Permanent(land)));
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "the land burned");
    assert_eq!(g.players[1].life, 19);
}

// ── Wave 12 ─────────────────────────────────────────────────────────────────

/// Karmic Justice answers an opponent's removal on your artifacts.
#[test]
fn karmic_justice_answers_noncreature_removal() {
    use crabomination::card::SelectionRequirement;
    use crabomination::effect::{Effect, Selector};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::karmic_justice());
    let mine = g.add_card_to_battlefield(0, catalog::catalyst_stone());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(mine, 1, None);
    let events = g
        .resolve_effect(
            &Effect::Destroy {
                what: Selector::EachPermanent(SelectionRequirement::Artifact),
            },
            &ctx,
        )
        .expect("opponent destroys it");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "Karmic Justice took one back");
}

/// Liquid Fire splits its five points.
#[test]
fn liquid_fire_splits_five_damage() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::liquid_fire());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    cast(&mut g, 0, spell, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "2 damage killed the 2/2");
    assert_eq!(g.players[1].life, 17, "the other 3 went to the face");
}

/// Bomb Squad detonates a creature at four fuse counters.
#[test]
fn bomb_squad_detonates_at_four_counters() {
    let mut g = main_phase();
    let squad = g.add_card_to_battlefield(0, catalog::bomb_squad());
    g.battlefield_find_mut(squad).unwrap().summoning_sick = false;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::Fuse, 3);
    activate(&mut g, 0, squad, 0, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "the fourth counter set it off");
    assert_eq!(g.players[1].life, 16);
}

/// Impulsive Maneuvers doubles an attacker's damage on a winning flip.
#[test]
fn impulsive_maneuvers_doubles_on_heads() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::impulsive_maneuvers());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    // The winning flip doubles the attacker's damage for the turn.
    g.step = TurnStep::DeclareBlockers;
    let _ = g.resolve_combat();
    assert_eq!(g.players[1].life, 16, "2 power doubled");
}

/// Shifty Doppelganger swaps itself out for something from hand.
#[test]
fn shifty_doppelganger_cheats_a_creature_in() {
    let mut g = main_phase();
    let dop = g.add_card_to_battlefield(0, catalog::shifty_doppelganger());
    g.battlefield_find_mut(dop).unwrap().summoning_sick = false;
    let big = g.add_card_to_hand(0, catalog::grizzly_bears());
    activate(&mut g, 0, dop, 0, None);
    assert!(g.battlefield_find(big).is_some(), "it came down");
    assert!(g.computed_permanent(big).unwrap().keywords.contains(&Keyword::Haste));
    assert!(g.battlefield_find(dop).is_none(), "the Doppelganger exiled itself");
}
