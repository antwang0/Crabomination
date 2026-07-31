#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── Talisman cycle (RW / UR / GU) ────────────────────────────────────────────

/// Talisman of Conviction: {T}: Add {C} (index 0); index 1 = {R}, index 2 = {W}.
#[test]
fn talisman_of_conviction_taps_for_red_costing_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::talisman_of_conviction());
    g.clear_sickness(id);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("red tap succeeds");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    assert_eq!(g.players[0].life, life_before - 1,
        "Talisman costs 1 life when tapped for a color");
}

/// Talisman of Creativity: index 1 = {U}, index 2 = {R}.
#[test]
fn talisman_of_creativity_taps_for_blue_or_red_costing_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::talisman_of_creativity());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("red tap succeeds");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    assert_eq!(g.players[0].life, 19,
        "Talisman costs 1 life when tapped for a color");
}

/// Talisman of Curiosity: index 1 = {G}, index 2 = {U}.
#[test]
fn talisman_of_curiosity_taps_for_green_costing_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::talisman_of_curiosity());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("green tap succeeds");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    assert_eq!(g.players[0].life, 19);
}

// ── Edict / forced-sacrifice removal ─────────────────────────────────────────

/// Edict-flavour sacrifice picks the lowest-CMC creature first.
/// Validates the new auto-decider sacrifice prioritization (tokens
/// first, then by lowest CMC, then by lowest power).
#[test]
fn forced_sacrifice_picks_lowest_cmc_creature_first() {
    let mut g = two_player_game();
    // Two creatures: a 4/5 (CMC 5) and a 2/2 (CMC 2). The bot should
    // sacrifice the 2/2 first.
    let big = g.add_card_to_battlefield(0, catalog::serra_angel());
    let small = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ib = g.add_card_to_hand(0, catalog::innocent_blood());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ib, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Innocent Blood castable for {B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == big),
        "Higher-CMC creature should survive Innocent Blood when a smaller one exists");
    assert!(!g.battlefield.iter().any(|c| c.id == small),
        "Lower-CMC creature should be sacrificed first");
}

/// Innocent Blood: each player sacrifices a creature.
#[test]
fn innocent_blood_each_player_sacrifices_a_creature() {
    let mut g = two_player_game();
    let p0_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p1_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ib = g.add_card_to_hand(0, catalog::innocent_blood());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ib, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Innocent Blood castable for {B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == p0_bear),
        "P0's bear should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == p1_bear),
        "P1's bear should be sacrificed");
}

/// Diabolic Edict: target opponent sacrifices a creature.
#[test]
fn diabolic_edict_targets_opponent_to_sacrifice_a_creature() {
    let mut g = two_player_game();
    let p1_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // P0 also has a creature — to verify Edict picks from the *target*'s
    // pool, not the caster's.
    let p0_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let de = g.add_card_to_hand(0, catalog::diabolic_edict());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: de,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Diabolic Edict castable for {1}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == p1_bear),
        "P1's bear should be sacrificed");
    assert!(g.battlefield.iter().any(|c| c.id == p0_bear),
        "P0's bear should not be touched");
}

/// Geth's Verdict: target sacs + loses 1 life.
#[test]
fn geths_verdict_sacs_target_and_drains_one_life() {
    let mut g = two_player_game();
    let p1_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gv = g.add_card_to_hand(0, catalog::geths_verdict());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: gv,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Geth's Verdict castable for {B}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == p1_bear),
        "P1's bear should be sacrificed");
    assert_eq!(g.players[1].life, 19, "P1 should lose 1 life");
}

// ── Burn / interaction ───────────────────────────────────────────────────────

/// Magma Jet: 2 damage to any target + Scry 2.
#[test]
fn magma_jet_deals_two_and_scries_two() {
    let mut g = two_player_game();
    // Stock the library so Scry has visible inputs.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let lib_before = g.players[0].library.len();
    let mj = g.add_card_to_hand(0, catalog::magma_jet());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: mj,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Magma Jet castable for {1}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "Target should take 2 damage");
    // Library size unchanged after Scry — cards stay on top by default
    // (AutoDecider keeps the top of the library).
    assert_eq!(g.players[0].library.len(), lib_before,
        "Scry shouldn't draw or mill cards");
}

/// Remand: counters a target spell, returns it to its owner's hand,
/// caster draws a card.
#[test]
fn remand_counters_returns_to_hand_and_draws() {
    let mut g = two_player_game();
    // Seed P0's library so the cantrip has an input.
    g.add_card_to_library(0, catalog::island());
    let hand_before_p0 = g.players[0].hand.len();
    // P1 casts a Lightning Bolt at P0.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable for {R}");
    // P0 Remands the bolt.
    g.priority.player_with_priority = 0;
    let rem = g.add_card_to_hand(0, catalog::remand());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: rem,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Remand castable for {1}{U}");
    drain_stack(&mut g);
    // Bolt didn't resolve.
    assert_eq!(g.players[0].life, 20, "Bolt was countered");
    // Bolt landed back in P1's hand (Move target → owner's hand).
    assert!(g.players[1].hand.iter().any(|c| c.id == bolt),
        "Bolt should be back in P1's hand");
    // Cantrip: P0 drew a card. Hand started at `hand_before_p0`, then we
    // added the Remand (+1), cast it (-1), drew 1 (+1) → end at +1.
    assert_eq!(g.players[0].hand.len(), hand_before_p0 + 1,
        "Cantrip should net P0 one card");
}

/// Read the Bones: scry 2, draw 2, lose 2.
#[test]
fn read_the_bones_scry_two_draw_two_lose_two() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;
    let rb = g.add_card_to_hand(0, catalog::read_the_bones());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: rb, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Read the Bones castable for {2}{B}");
    drain_stack(&mut g);
    // hand_before captured before we added Read the Bones; the spell's
    // own +1/-1 round-trip cancels, so the +2 draw is the only delta.
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "+2 draw");
    assert_eq!(g.players[0].life, life_before - 2, "lose 2 life");
}

/// Storm Crow: 1U 1/2 flying Bird body.
#[test]
fn storm_crow_is_a_one_two_flying_bird() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::storm_crow());
    let card = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 2);
    assert!(card.definition.keywords.contains(&crabomination::card::Keyword::Flying));
    assert!(card.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Bird));
}

/// Ancient Grudge: destroys a target artifact, lands in graveyard with
/// flashback available.
#[test]
fn ancient_grudge_destroys_artifact_with_flashback_available() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let ag = g.add_card_to_hand(0, catalog::ancient_grudge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ag,
        target: Some(Target::Permanent(stone)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Ancient Grudge castable for {1}{R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "Mind Stone should be destroyed");
    // Spell ended up in graveyard, available for flashback.
    let in_yard = g.players[0].graveyard.iter().any(|c| c.id == ag);
    assert!(in_yard, "Ancient Grudge in graveyard");
    let card = g.players[0].graveyard.iter().find(|c| c.id == ag).unwrap();
    assert!(card.definition.has_flashback().is_some(),
        "Flashback cost should still be on the card");
}

/// Ancient Grudge: cast from graveyard via Flashback {G} — destroys a
/// second artifact and exiles the spell on resolution.
#[test]
fn ancient_grudge_flashback_destroys_second_artifact_and_exiles() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    // Ancient Grudge starts in P0's graveyard.
    let ag = g.add_card_to_graveyard(0, catalog::ancient_grudge());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastFlashback {
        card_id: ag,
        target: Some(Target::Permanent(stone)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Flashback castable for {G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "Mind Stone should be destroyed by flashback");
    assert!(g.exile.iter().any(|c| c.id == ag),
        "Flashback resolves into exile");
}

/// Tragic Slip: target creature gets -13/-13 EOT (effectively lethal).
#[test]
fn tragic_slip_without_morbid_only_shrinks_minus_one() {
    // No creature has died this turn → Morbid is off → only -1/-1.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ts = g.add_card_to_hand(0, catalog::tragic_slip());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ts,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Tragic Slip castable for {B}");
    drain_stack(&mut g);
    let c = g.battlefield.iter().find(|c| c.id == bear)
        .expect("2/2 bear survives a -1/-1 (becomes 1/1)");
    assert_eq!(c.power(), 1, "Morbid off: -1/-1 only");
    assert_eq!(c.toughness(), 1);
}

#[test]
fn tragic_slip_with_morbid_kills_via_minus_thirteen() {
    // A creature died this turn → Morbid is on → full -13/-13.
    let mut g = two_player_game();
    // A creature died this turn so Morbid is satisfied.
    g.players[0].creatures_died_this_turn = 1;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ts = g.add_card_to_hand(0, catalog::tragic_slip());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ts,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Tragic Slip castable for {B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Morbid on: -13/-13 kills the bear");
}

// ── New cards: rummagers, burn, counters, removal, white tokens, ETB destroy ──

/// Tormenting Voice: discard a card, then draw two — net +1 hand minus the
/// spell itself, so the hand stays the same size while filtering.
#[test]
fn tormenting_voice_discards_one_and_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::lightning_bolt()); // chuck-able
    let id = g.add_card_to_hand(0, catalog::tormenting_voice());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    let grave_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tormenting Voice castable for {1}{R}");
    drain_stack(&mut g);

    // Net: -1 cast, -1 discard, +2 draw = 0 change.
    assert_eq!(g.players[0].hand.len(), hand_before, "Voice nets 0 hand size");
    assert_eq!(g.players[0].graveyard.len(), grave_before + 2,
        "Spell + discarded card both go to graveyard");
}

/// Wild Guess and Tormenting Voice mirror — same effect, different cost.
#[test]
fn wild_guess_discards_one_and_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::wild_guess());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wild Guess castable for {R}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "Wild Guess nets 0 hand size");
}

/// Thrill of Possibility is the instant-speed version. Same loot pattern,
/// but the spell is castable as an Instant.
#[test]
fn thrill_of_possibility_is_an_instant_loot_2() {
    use crabomination::card::CardType;
    let card = catalog::thrill_of_possibility();
    assert!(card.card_types.contains(&CardType::Instant),
        "Thrill of Possibility should be an Instant");

    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, card);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Thrill castable for {1}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "Thrill nets 0 hand size");
}

/// Volcanic Hammer is a 3-damage sorcery — straight Lightning Strike at
/// sorcery timing.
#[test]
fn volcanic_hammer_deals_three_to_creature() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::volcanic_hammer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Volcanic Hammer castable for {1}{R}");
    drain_stack(&mut g);

    // Serra is 4/4 — 3 damage doesn't kill it but does mark it.
    let serra = g.battlefield.iter().find(|c| c.id == big).expect("Serra survives");
    assert_eq!(serra.damage, 3, "Volcanic Hammer should mark 3 damage");
}

/// Slagstorm mode 0: sweeps creatures (3 to each), leaves players alone.
#[test]
fn slagstorm_mode_zero_sweeps_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let lion = g.add_card_to_battlefield(0, catalog::savannah_lions()); // 2/1
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;

    let id = g.add_card_to_hand(0, catalog::slagstorm());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Slagstorm castable for {1}{R}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear (2/2) should die to 3 damage");
    assert!(!g.battlefield.iter().any(|c| c.id == lion),
        "Savannah Lions (2/1) should die to 3 damage");
    assert_eq!(g.players[0].life, p0_life_before, "mode 0 doesn't burn players");
    assert_eq!(g.players[1].life, p1_life_before, "mode 0 doesn't burn players");
}

/// Slagstorm mode 1: 3 damage to each player, creatures survive.
#[test]
fn slagstorm_mode_one_burns_each_player() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;

    let id = g.add_card_to_hand(0, catalog::slagstorm());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Slagstorm castable for {1}{R}{R}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, p0_before - 3,
        "mode 1 burns the caster too (Slagstorm is symmetric)");
    assert_eq!(g.players[1].life, p1_before - 3, "mode 1 burns each player");
    assert!(g.battlefield.iter().any(|c| c.id == serra),
        "mode 1 doesn't touch creatures");
}

/// Cancel: counter target spell.
#[test]
fn cancel_counters_a_spell() {
    let mut g = two_player_game();
    // P1 casts a Lightning Bolt at P0; P0 cancels.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();

    let cancel = g.add_card_to_hand(0, catalog::cancel());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: cancel,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cancel castable for {1}{U}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, 20, "Bolt should never resolve");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Countered spell still goes to its owner's graveyard");
}

/// Annul rejects a noncreature, non-artifact, non-enchantment spell at
/// cast time (e.g. Lightning Bolt is an instant, not in scope).
#[test]
fn annul_rejects_instant_target_at_cast_time() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();

    let annul = g.add_card_to_hand(0, catalog::annul());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: annul,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Annul shouldn't accept an instant target");
}

/// Hero's Downfall destroys a target creature.
#[test]
fn heros_downfall_destroys_target_creature() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // legendary? no
    let id = g.add_card_to_hand(0, catalog::heros_downfall());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Hero's Downfall castable for {1}{B}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == serra),
        "Serra Angel should be destroyed");
}

/// Cast Down rejects a Legendary creature target at cast time.
#[test]
fn cast_down_rejects_legendary_creature() {
    let mut g = two_player_game();
    // Griselbrand is legendary.
    let gris = g.add_card_to_battlefield(1, catalog::griselbrand());
    let id = g.add_card_to_hand(0, catalog::cast_down());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(gris)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Cast Down shouldn't accept a legendary target");
}

/// Cast Down destroys a nonlegendary creature.
#[test]
fn cast_down_destroys_nonlegendary_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cast_down());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cast Down castable for {1}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be destroyed");
}

/// Mind Rot: target player discards two cards.
#[test]
fn mind_rot_discards_two_from_target() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::lightning_bolt());
    }
    let id = g.add_card_to_hand(0, catalog::mind_rot());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mind Rot castable for {2}{B}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before - 2,
        "Mind Rot should remove two cards from the target's hand");
}

/// Raise Dead returns a creature card from the graveyard to the hand.
#[test]
fn raise_dead_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::raise_dead());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Raise Dead castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Bear should return to hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear),
        "Bear should leave graveyard");
}

/// Healing Salve: gain 3 life on target.
#[test]
fn healing_salve_gives_three_life() {
    let mut g = two_player_game();
    g.players[0].life = 10;
    let id = g.add_card_to_hand(0, catalog::healing_salve());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Healing Salve castable for {W}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 13, "+3 life");
}

/// Healing Salve mode 1: prevent the next 3 damage to a creature (CR 615.7).
/// A 2/2 bear with the shield survives a Lightning Bolt.
#[test]
fn healing_salve_mode_one_prevents_next_three_damage() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let salve = g.add_card_to_hand(0, catalog::healing_salve());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: salve, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Healing Salve castable for {W}");
    drain_stack(&mut g);
    // Bolt the bear: 3 prevented by the shield → bear lives, shield spent.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "bear survives a prevented bolt");
    assert!(g.prevention_shields.is_empty(), "next-3 shield consumed");
}

/// Raise the Alarm creates two 1/1 Soldier tokens.
#[test]
fn raise_the_alarm_creates_two_soldier_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::raise_the_alarm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Raise the Alarm castable for {1}{W}");
    drain_stack(&mut g);

    let soldiers: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Soldier")
        .collect();
    assert_eq!(soldiers.len(), 2, "Two Soldier tokens should enter");
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Two new permanents on the battlefield");
}

/// Reclamation Sage's ETB destroys an artifact.
#[test]
fn reclamation_sage_etb_destroys_artifact() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::reclamation_sage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reclamation Sage castable for {2}{G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "Mind Stone should be destroyed by Sage's ETB");
}

/// Acidic Slime is a 2/2 Deathtouch and its ETB hits a land.
#[test]
fn acidic_slime_etb_destroys_land() {
    use crabomination::card::Keyword;
    let card = catalog::acidic_slime();
    assert!(card.keywords.contains(&Keyword::Deathtouch),
        "Acidic Slime has Deathtouch");
    assert_eq!(card.power, 2);
    assert_eq!(card.toughness, 2);

    let mut g = two_player_game();
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    let id = g.add_card_to_hand(0, card);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Acidic Slime castable for {3}{G}{G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mountain),
        "Mountain should be destroyed by Slime's ETB");
}

/// Stoke the Flames: convoke 4-damage instant. Casting at full {4}{R} is
/// fine; the convoke half is exercised by the existing convoke tests.
#[test]
fn stoke_the_flames_deals_four_at_full_cost() {
    use crabomination::card::Keyword;
    let card = catalog::stoke_the_flames();
    assert!(card.keywords.contains(&Keyword::Convoke),
        "Stoke the Flames has Convoke");
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, card);
    // Real Oracle: `{2}{R}{R}` Instant.
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Stoke the Flames castable for {2}{R}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 4);
}

// ── Bounce ───────────────────────────────────────────────────────────────────

/// Unsummon: target creature returns to its owner's hand.
#[test]
fn unsummon_returns_target_creature_to_owners_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::unsummon());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Unsummon castable for {U}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should leave the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "Bear should return to its owner's (Bob's) hand, not the caster's");
}

/// Boomerang: bounces non-creature permanents (Sol Ring), proving the wider
/// `Permanent` filter compared to Unsummon.
#[test]
fn boomerang_bounces_a_non_creature_permanent() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::boomerang());
    g.players[0].mana_pool.add(Color::Blue, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Boomerang castable for {U}{U}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == ring),
        "Sol Ring should leave the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == ring),
        "Sol Ring should return to its owner's hand");
}

/// Cyclonic Rift rejects targeting your own permanents at cast time.
#[test]
fn cyclonic_rift_rejects_your_own_permanent() {
    let mut g = two_player_game();
    let your_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cyclonic_rift());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(your_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Cyclonic Rift should reject your own creature: {:?}", err);
}

/// Cyclonic Rift bounces an opp permanent.
#[test]
fn cyclonic_rift_bounces_opponent_nonland_permanent() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cyclonic_rift());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cyclonic Rift castable for {1}{U}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear));
    assert!(g.players[1].hand.iter().any(|c| c.id == opp_bear));
}

#[test]
fn cyclonic_rift_overload_bounces_all_opponent_nonland_permanents() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let own_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cyclonic_rift());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(6);

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Cyclonic Rift Overload for {6}{U}");
    drain_stack(&mut g);

    // Both opponent creatures should be bounced.
    assert!(!g.battlefield.iter().any(|c| c.id == bear1));
    assert!(!g.battlefield.iter().any(|c| c.id == bear2));
    // Own creature should remain.
    assert!(g.battlefield.iter().any(|c| c.id == own_bear));
}

/// Repeal: pays X = 2, bounces a CMC-2 creature, draws a card.
#[test]
fn repeal_with_x_two_bounces_two_drop_and_cantrips() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // {1}{G}
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::repeal());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    }).expect("Repeal castable for {2}{U} (X=2)");
    drain_stack(&mut g);

    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "Bear should bounce to opp's hand");
    // Repeal goes to caster's graveyard; draw replaces it from library.
    // Net hand change: -1 (cast) + 1 (draw) = 0.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Cast (-1) + cantrip (+1) = net 0");
}

/// Repeal: when X is too small the cmc gate fails — only the cantrip fires,
/// the target stays on the battlefield.
#[test]
fn repeal_x_zero_against_two_drop_does_not_bounce() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::repeal());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(0),
    }).expect("Repeal castable for {U} (X=0)");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "Bear should stay on battlefield: 2 > X=0");
}

// ── Removal ──────────────────────────────────────────────────────────────────

/// Murder destroys any creature, including a black one (vs Doom Blade).
#[test]
fn murder_destroys_any_creature_including_black() {
    let mut g = two_player_game();
    let specter = g.add_card_to_battlefield(1, catalog::hypnotic_specter());
    let id = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(specter)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Murder castable for {1}{B}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == specter),
        "Hypnotic Specter (black) should die to Murder");
}

/// Go for the Throat destroys non-artifact creatures, rejects artifact creatures.
#[test]
fn go_for_the_throat_destroys_nonartifact_but_rejects_artifact() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let memnite = g.add_card_to_battlefield(1, catalog::memnite()); // 1/1 artifact creature
    let id_ok = g.add_card_to_hand(0, catalog::go_for_the_throat());
    let id_bad = g.add_card_to_hand(0, catalog::go_for_the_throat());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id_ok,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Go for the Throat castable for {1}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Bear should die");

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id_bad,
        target: Some(Target::Permanent(memnite)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Go for the Throat should reject Memnite (artifact): {:?}", err);
}

/// Disfigure: -2/-2 EOT — kills a 2/2.
#[test]
fn disfigure_kills_a_two_two_via_minus_two_minus_two() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::disfigure());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Disfigure castable for {B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear (2/2) should die to -2/-2");
}

#[test]
fn borderland_marauder_pumps_while_attacking() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::borderland_marauder());
    g.clear_sickness(id);
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((1, 2)));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }]).expect("attack");
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((3, 2)),
        "+2/+0 while attacking");
}

#[test]
fn pia_nalaar_etb_makes_a_thopter() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pia_nalaar());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Thopter" && c.controller == 0),
        "ETB mints a 1/1 Thopter");
}

#[test]
fn spikeshot_goblin_pings_for_its_power() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::spikeshot_goblin());
    g.clear_sickness(id);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().damage, 1, "deals damage equal to its power (1)");
}

#[test]
fn zealous_conscripts_steals_and_untaps_a_permanent() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(enemy).unwrap().tapped = true;
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Target(Target::Permanent(enemy))]));
    let id = g.add_card_to_battlefield(0, catalog::zealous_conscripts());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(enemy).unwrap().controller, 0, "gained control");
    assert!(!g.battlefield_find(enemy).unwrap().tapped, "untapped the stolen permanent");
}

#[test]
fn palace_sentinels_makes_you_monarch() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::palace_sentinels());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0), "ETB makes you the monarch");
}

#[test]
fn knight_of_the_white_orchid_ramps_when_behind_on_lands() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Opponent controls more lands than you (you: 0, opp: 1).
    g.add_card_to_battlefield(1, catalog::island());
    let plains = g.add_card_to_library(0, catalog::plains());
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(plains)),
    ]));
    let id = g.add_card_to_battlefield(0, catalog::knight_of_the_white_orchid());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == plains), "tutored a Plains to the battlefield");
}

#[test]
fn adanto_vanguard_gets_plus_two_zero_while_attacking() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::adanto_vanguard());
    g.clear_sickness(id);
    // Not attacking: base 1/1.
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((1, 1)));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }])
        .expect("attack");
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((3, 1)),
        "+2/+0 while attacking");
}

#[test]
fn cloud_of_faeries_untaps_two_lands_on_etb() {
    let mut g = two_player_game();
    let l1 = g.add_card_to_battlefield(0, catalog::island());
    let l2 = g.add_card_to_battlefield(0, catalog::island());
    let l3 = g.add_card_to_battlefield(0, catalog::island());
    for l in [l1, l2, l3] { g.battlefield_find_mut(l).unwrap().tapped = true; }
    let id = g.add_card_to_battlefield(0, catalog::cloud_of_faeries());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    let untapped = [l1, l2, l3].iter().filter(|l| !g.battlefield_find(**l).unwrap().tapped).count();
    assert_eq!(untapped, 2, "untaps up to two lands");
}

#[test]
fn gnarled_scarhide_bestows_plus_two_one_and_cant_block() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::gnarled_scarhide());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastBestow {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bestow onto the bear");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 3), "enchanted creature gets +2/+1");
    assert!(b.keywords.contains(&Keyword::CantBlock), "and can't block");
}

#[test]
fn cordial_vampire_grows_vampires_on_death() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let cordial = g.add_card_to_battlefield(0, catalog::cordial_vampire());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Bolt the bear so SBA dispatches CreatureDied to Cordial Vampire.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the bear");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cordial).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "a creature death puts a +1/+1 on each Vampire you control");
}

#[test]
fn vampire_hexmage_sacrifices_to_strip_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let hexmage = g.add_card_to_battlefield(0, catalog::vampire_hexmage());
    // A target carrying counters of two kinds.
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(target).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.battlefield_find_mut(target).unwrap().add_counters(CounterType::Charge, 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hexmage, ability_index: 0, target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sacrifice to remove counters");
    drain_stack(&mut g);
    let t = g.battlefield_find(target).unwrap();
    assert_eq!(t.counters.values().sum::<u32>(), 0, "all counters removed");
    assert!(g.battlefield_find(hexmage).is_none(), "Hexmage sacrificed");
}

#[test]
fn plagued_rusalka_sacrifices_a_creature_to_shrink() {
    let mut g = two_player_game();
    let rusalka = g.add_card_to_battlefield(0, catalog::plagued_rusalka());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::elite_vanguard()); // 2/1
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rusalka, ability_index: 0, target: Some(Target::Permanent(victim)),
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    }).expect("sac a creature for -1/-1");
    drain_stack(&mut g);
    // Fodder (or the Rusalka) sacrificed; the 2/1 victim shrinks to 1/0 and dies.
    assert!(g.battlefield_find(fodder).is_none() || g.players[0].graveyard.iter().any(|c| c.id == fodder));
    assert!(g.battlefield_find(victim).is_none(), "2/1 dies to -1/-1");
}

#[test]
fn dragonmaster_outcast_makes_dragon_with_six_lands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dragonmaster_outcast());
    // Five lands → no Dragon.
    for _ in 0..5 { g.add_card_to_battlefield(0, catalog::forest()); }
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Dragon"), "no Dragon below 6 lands");
    // Sixth land → Dragon.
    g.add_card_to_battlefield(0, catalog::forest());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Dragon" && c.controller == 0),
        "makes a 5/5 Dragon at six lands");
}

/// Languish: every creature gets -2/-2 EOT — sweeps 2/2s, leaves 4/4s alive.
#[test]
fn languish_sweeps_small_but_leaves_big_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let lions = g.add_card_to_battlefield(0, catalog::savannah_lions()); // 2/1
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::languish());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Languish castable for {2}{B}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear (2/2) should die to -2/-2");
    assert!(!g.battlefield.iter().any(|c| c.id == lions),
        "Savannah Lions (2/1) should die to -2/-2");
    assert!(g.battlefield.iter().any(|c| c.id == serra),
        "Serra (4/4) should survive — 4-2 = 2 toughness left");
}

/// Lay Down Arms exiles a creature whose MV ≤ Plains you control, the
/// exiled creature's controller gains 3 life, and it rejects higher-MV
/// targets.
#[test]
fn lay_down_arms_exiles_by_plains_count_and_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::plains());
    g.add_card_to_battlefield(0, catalog::plains()); // 2 Plains → MV cap 2
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // CMC 2
    let craw = g.add_card_to_battlefield(1, catalog::craw_wurm()); // CMC 6
    let id_ok = g.add_card_to_hand(0, catalog::lay_down_arms());
    let id_bad = g.add_card_to_hand(0, catalog::lay_down_arms());
    g.players[0].mana_pool.add(Color::White, 2);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id_ok,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lay Down Arms castable for {W}");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "Bear (CMC 2 ≤ 2 Plains) should be exiled");
    assert_eq!(g.players[1].life, life_before + 3, "bear's controller gains 3 life");

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id_bad,
        target: Some(Target::Permanent(craw)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Lay Down Arms should reject CMC-6 Craw Wurm with only 2 Plains: {:?}", err);
}

/// Smelt destroys an artifact.
#[test]
fn smelt_destroys_an_artifact() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::smelt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Smelt castable for {R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == ring));
}

// ── X-cost burn ──────────────────────────────────────────────────────────────

/// Banefire: X damage to a creature scales with X paid.
#[test]
fn banefire_deals_x_damage_to_creature() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::banefire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(5),
    }).expect("Banefire castable for {5}{R} (X=5)");
    drain_stack(&mut g);

    // Banefire is sorcery — damage marks the creature; lethal moves it to graveyard via SBA.
    assert!(!g.battlefield.iter().any(|c| c.id == serra),
        "Serra (4 toughness) should die to 5 damage");
}

/// Banefire to a player face — pure burn.
#[test]
fn banefire_burns_a_player_face_for_x() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::banefire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(7);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(7),
    }).expect("Banefire castable for {7}{R} (X=7)");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 7,
        "Banefire X=7 should burn for 7");
}

#[test]
fn banefire_uncounterable_at_x_five() {
    // Push (modern_decks): "If X is 5 or more, this spell can't be
    // countered" rider wired via `caster_grants_uncounterable_with_x`.
    // X=5 → the cast pushes `StackItem::Spell { uncounterable: true }`
    // and counterspells targeting it fizzle.
    use crabomination::game::types::StackItem;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::banefire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(5),
    }).expect("Banefire castable for {5}{R} (X=5)");

    // Inspect the stack item to confirm uncounterable is set.
    let uncounterable = g.stack.iter().find_map(|si| match si {
        StackItem::Spell { uncounterable, .. } => Some(*uncounterable),
        _ => None,
    });
    assert_eq!(uncounterable, Some(true),
        "Banefire at X=5 should land on the stack as uncounterable");
}

#[test]
fn banefire_counterable_below_x_five() {
    // X=4: stays counterable (rider doesn't kick in until X ≥ 5).
    use crabomination::game::types::StackItem;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::banefire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(4),
    }).expect("Banefire castable for {4}{R} (X=4)");

    let uncounterable = g.stack.iter().find_map(|si| match si {
        StackItem::Spell { uncounterable, .. } => Some(*uncounterable),
        _ => None,
    });
    assert_eq!(uncounterable, Some(false),
        "Banefire at X=4 should remain counterable");
}

// ── Tokens ───────────────────────────────────────────────────────────────────

/// Spectral Procession creates three 1/1 white flying spirits.
#[test]
fn spectral_procession_creates_three_flying_spirits() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spectral_procession());
    // Cheapest cast: pay each {2/W} pip with white → {W}{W}{W}.
    g.players[0].mana_pool.add(Color::White, 3);
    let bf_count_before = g.battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .count();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spectral Procession castable for {W}{W}{W} via mono-hybrid pips");
    drain_stack(&mut g);

    let new_tokens: Vec<_> = g.battlefield
        .iter()
        .filter(|c| c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(new_tokens.len(), 3,
        "Spectral Procession should create three Spirit tokens");
    for tok in &new_tokens {
        assert!(tok.definition.keywords.contains(&Keyword::Flying),
            "Spirit tokens should have flying");
        assert_eq!(tok.definition.power, 1);
        assert_eq!(tok.definition.toughness, 1);
    }
    let bf_count_after = g.battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .count();
    assert_eq!(bf_count_after, bf_count_before + 3,
        "+3 permanents on caster's side of board");
}

#[test]
fn spectral_procession_castable_with_six_generic() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spectral_procession());
    // Pay every {2/W} pip with the generic side → {6}.
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spectral Procession castable for {6} via the generic side");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    assert_eq!(spirits, 3);
}

// ── Recursion ────────────────────────────────────────────────────────────────

/// Regrowth: returns any card type from your graveyard to your hand.
#[test]
fn regrowth_returns_a_land_card_from_graveyard() {
    let mut g = two_player_game();
    let mountain = g.add_card_to_graveyard(0, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::regrowth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mountain)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Regrowth castable for {1}{G}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mountain),
        "Mountain card should return to caster's hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == mountain),
        "Mountain card should leave graveyard");
}

/// Beast Within: destroy any permanent, the controller gets a 3/3 Beast.
#[test]
fn beast_within_destroys_and_creates_beast_for_controller() {
    use crabomination::card::{CreatureType, CardType};
    let mut g = two_player_game();
    let opp_ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::beast_within());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_ring)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Beast Within castable for {2}{G}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_ring),
        "Sol Ring should be destroyed");
    let beasts: Vec<_> = g.battlefield
        .iter()
        .filter(|c| c.controller == 1
            && c.definition.card_types.contains(&CardType::Creature)
            && c.definition.subtypes.creature_types.contains(&CreatureType::Beast))
        .collect();
    assert_eq!(beasts.len(), 1,
        "Opp (Sol Ring's controller) should get one 3/3 Beast token");
    assert_eq!(beasts[0].definition.power, 3);
    assert_eq!(beasts[0].definition.toughness, 3);
}

/// Grasp of Darkness: -4/-4 EOT — kills a 4/4.
#[test]
fn grasp_of_darkness_kills_a_four_four() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::grasp_of_darkness());
    g.players[0].mana_pool.add(Color::Black, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Grasp of Darkness castable for {B}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == serra),
        "Serra Angel (4/4) should die to -4/-4");
}

/// Shatter destroys an artifact.
#[test]
fn shatter_destroys_an_artifact() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::shatter());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Shatter castable for {1}{R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == ring));
}

