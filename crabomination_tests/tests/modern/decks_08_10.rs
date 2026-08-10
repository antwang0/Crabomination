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

// ── modern_decks-8 tests ─────────────────────────────────────────────────────

/// Incinerate deals 3 damage to a creature, killing a 2/2.
#[test]
fn incinerate_deals_three_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::incinerate());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Incinerate castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Incinerate (3 damage) should kill the Grizzly Bears");
}

/// Incinerate burns a player face for 3.
#[test]
fn incinerate_burns_a_player_for_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::incinerate());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Incinerate can hit a player");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 3);
}

/// Searing Spear: 3 damage to any target.
#[test]
fn searing_spear_deals_three_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::searing_spear());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Searing Spear castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

/// Flame Slash: 4 damage destroys a 4-toughness creature.
#[test]
fn flame_slash_kills_a_four_toughness_creature() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::flame_slash());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Flame Slash castable for {R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == serra),
        "Flame Slash (4 damage) should kill the 4/4 Serra Angel");
}

/// Flame Slash rejects a player target at cast time (creature-only).
#[test]
fn flame_slash_rejects_player_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::flame_slash());
    g.players[0].mana_pool.add(Color::Red, 1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Flame Slash should reject a player target: {:?}", err);
}

/// Roast: 5 damage kills a non-flier (Grizzly Bears).
#[test]
fn roast_kills_a_non_flying_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::roast());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Roast castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

/// Roast rejects a flier at cast time.
#[test]
fn roast_rejects_a_flier() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
    let id = g.add_card_to_hand(0, catalog::roast());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Roast should reject a flying creature: {:?}", err);
}

/// Smother destroys a 2-CMC creature.
#[test]
fn smother_destroys_low_cmc_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2 CMC
    let id = g.add_card_to_hand(0, catalog::smother());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Smother castable for {1}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

/// Smother rejects high-CMC creature targets at cast time.
#[test]
fn smother_rejects_high_cmc_target() {
    let mut g = two_player_game();
    let craw = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6 CMC
    let id = g.add_card_to_hand(0, catalog::smother());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(craw)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Smother should reject a 6-CMC Craw Wurm: {:?}", err);
}

/// Smother's "can't be regenerated" clause: a shielded creature still dies.
#[test]
fn smother_ignores_regeneration_shield() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().regeneration_shields = 1;
    let id = g.add_card_to_hand(0, catalog::smother());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Smother castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Smother destroys through a regeneration shield");
}

/// Final Reward: exiles a creature.
#[test]
fn final_reward_exiles_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::final_reward());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Final Reward castable for {4}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should leave the battlefield");
    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be exiled, not graveyarded");
}

/// Holy Light sweeps -1/-1 across all creatures, killing 1-toughness.
#[test]
fn holy_light_sweeps_minus_one_minus_one() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves()); // 1/1
    g.clear_sickness(elf); // auto-tap may only tap a non-sick elf (CR 602.5g)
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::holy_light());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Holy Light castable for {1}{W}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == elf),
        "1/1 Llanowar Elves should die to -1/-1");
    let bear_view = g.battlefield.iter().find(|c| c.id == bear);
    assert!(bear_view.is_some(),
        "Grizzly Bears (2/2) survives -1/-1 sweep");
}

/// Mana Tithe counters a spell when controller can't pay {1}.
#[test]
fn mana_tithe_counters_when_controller_cannot_pay_one() {
    let mut g = two_player_game();
    // P1 casts a Lightning Bolt on their turn with red mana only — no
    // leftover generic to pay the tax.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lightning Bolt castable for {R}");

    // P0 responds with Mana Tithe.
    g.priority.player_with_priority = 0;
    let tithe = g.add_card_to_hand(0, catalog::mana_tithe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: tithe,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mana Tithe castable for {W}");
    drain_stack(&mut g);

    // Bolt should be countered (lands in graveyard, no damage to P0).
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Lightning Bolt should be countered to graveyard");
    assert_eq!(g.players[0].life, 20,
        "P0 should not have taken Bolt damage");
}

/// Rampant Growth tutors a basic into play tapped.
#[test]
fn rampant_growth_searches_a_basic_into_play_tapped() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt()); // padding non-basic

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
    ]));

    let id = g.add_card_to_hand(0, catalog::rampant_growth());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rampant Growth castable for {1}{G}");
    drain_stack(&mut g);

    let view = g.battlefield.iter().find(|c| c.id == forest);
    assert!(view.is_some(), "Forest should be on battlefield");
    assert!(view.unwrap().tapped, "Forest should enter tapped");
}

/// Cultivate fetches two basics: one tapped to play, one to hand.
#[test]
fn cultivate_searches_two_basics() {
    let mut g = two_player_game();
    let bf_target = g.add_card_to_library(0, catalog::forest());
    let hand_target = g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::lightning_bolt());

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bf_target)),
        DecisionAnswer::Search(Some(hand_target)),
    ]));

    let id = g.add_card_to_hand(0, catalog::cultivate());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cultivate castable for {2}{G}");
    drain_stack(&mut g);

    let bf = g.battlefield.iter().find(|c| c.id == bf_target);
    assert!(bf.is_some(), "First basic on battlefield");
    assert!(bf.unwrap().tapped, "Battlefield basic enters tapped");
    assert!(g.players[0].hand.iter().any(|c| c.id == hand_target),
        "Second basic into hand");
}

/// Farseek tutors a basic into play tapped.
#[test]
fn farseek_searches_a_basic_into_play_tapped() {
    let mut g = two_player_game();
    let plains = g.add_card_to_library(0, catalog::plains());

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(plains)),
    ]));

    let id = g.add_card_to_hand(0, catalog::farseek());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Farseek castable for {1}{G}");
    drain_stack(&mut g);

    let view = g.battlefield.iter().find(|c| c.id == plains);
    assert!(view.is_some(), "Plains should be on battlefield");
    assert!(view.unwrap().tapped, "Plains should enter tapped");
}

/// Sakura-Tribe Elder: tap-and-sac search for a basic.
#[test]
fn sakura_tribe_elder_sacrifices_for_a_basic() {
    let mut g = two_player_game();
    let elder = g.add_card_to_battlefield(0, catalog::sakura_tribe_elder());
    g.clear_sickness(elder);
    let forest = g.add_card_to_library(0, catalog::forest());

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
    ]));

    g.perform_action(GameAction::ActivateAbility {
        card_id: elder, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Sakura-Tribe Elder activates");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == elder),
        "Elder should be sacrificed");
    let view = g.battlefield.iter().find(|c| c.id == forest);
    assert!(view.is_some(), "Forest tutored to battlefield");
    assert!(view.unwrap().tapped, "Forest enters tapped");
}

#[test]
fn thornweald_archer_has_reach_and_deathtouch() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::thornweald_archer());
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == id).unwrap();
    assert!(c.keywords.contains(&crabomination::card::Keyword::Reach));
    assert!(c.keywords.contains(&crabomination::card::Keyword::Deathtouch));
}

#[test]
fn wild_nacatl_grows_with_mountain_and_plains() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::wild_nacatl());
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((1, 1)));
    g.add_card_to_battlefield(0, catalog::mountain());
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((2, 2)),
        "+1/+1 with a Mountain");
    g.add_card_to_battlefield(0, catalog::plains());
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((3, 3)),
        "+1/+1 more with a Plains");
}

#[test]
fn skyshroud_elite_grows_against_nonbasic_lands() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::skyshroud_elite());
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((1, 1)));
    // Opponent's nonbasic land → +1/+2.
    g.add_card_to_battlefield(1, catalog::wasteland());
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((2, 3)));
}

#[test]
fn werebear_threshold_pumps_at_seven_graveyard_cards() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::werebear());
    // Below threshold: base 1/1.
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((1, 1)));
    // Fill graveyard to seven → +3/+3.
    for _ in 0..7 { g.add_card_to_graveyard(0, catalog::forest()); }
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((4, 4)),
        "Threshold grants +3/+3");
}

#[test]
fn viridian_emissary_dies_ramps_a_basic_tapped() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::viridian_emissary());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(forest)),
    ]));
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    let f = g.battlefield.iter().find(|c| c.id == forest).expect("ramped a basic");
    assert!(f.tapped, "basic enters tapped");
}

/// Wood Elves: ETB search a Forest into play untapped.
#[test]
fn wood_elves_etb_searches_forest_untapped() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt()); // padding

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
    ]));

    let id = g.add_card_to_hand(0, catalog::wood_elves());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wood Elves castable for {2}{G}");
    drain_stack(&mut g);

    let view = g.battlefield.iter().find(|c| c.id == forest);
    assert!(view.is_some(), "Forest tutored to battlefield");
    assert!(!view.unwrap().tapped, "Forest should ENTER UNTAPPED for Wood Elves");
}

/// Elvish Mystic: tap for {G}.
#[test]
fn elvish_mystic_taps_for_green() {
    let mut g = two_player_game();
    let mystic = g.add_card_to_battlefield(0, catalog::elvish_mystic());
    g.clear_sickness(mystic);
    let pool_before = g.players[0].mana_pool.total();

    g.perform_action(GameAction::ActivateAbility {
        card_id: mystic, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Elvish Mystic activates");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == mystic && c.tapped),
        "Mystic should be tapped");
    assert_eq!(g.players[0].mana_pool.total(), pool_before + 1,
        "Mystic adds 1 green mana");
    assert!(g.players[0].mana_pool.amount(Color::Green) >= 1,
        "Pool should have at least 1 green");
}

/// Harmonize: draws three cards.
#[test]
fn harmonize_draws_three_cards() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::harmonize());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Harmonize castable for {2}{G}{G}");
    drain_stack(&mut g);

    // -1 (cast) + 3 (draw) = +2 hand.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3,
        "Harmonize nets +2 cards");
}

/// Concentrate: draws three cards.
#[test]
fn concentrate_draws_three_cards() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::concentrate());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Concentrate castable for {2}{U}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
}

/// Severed Strands: sac a creature, gain life = its toughness, destroy an
/// opponent's creature. Sacrificing a 3-toughness Hill Giant gains 3.
#[test]
fn severed_strands_sacs_and_destroys_for_life() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    g.clear_sickness(fodder);
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::severed_strands());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Severed Strands castable for {1}{B}");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder),
        "Fodder should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == target),
        "Target should be destroyed");
    assert_eq!(g.players[0].life, life_before + 3,
        "P0 gains life equal to the sacrificed creature's toughness (3)");
}

/// Anticipate: scry 2 + draw 1 net (-1 cast +1 draw = net 0 hand).
#[test]
fn anticipate_resolves_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::anticipate());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Anticipate castable for {1}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before,
        "Anticipate (cast -1, draw +1) should net 0 hand");
}

/// Divination: -1 cast +2 draw = net +1 hand.
#[test]
fn divination_draws_two_cards() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Divination castable for {2}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2);
}

/// Ambition's Cost: draws 3 and lose 3 life.
#[test]
fn ambitions_cost_draws_three_loses_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::swamp()); }
    let id = g.add_card_to_hand(0, catalog::ambitions_cost());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ambition's Cost castable for {3}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
    assert_eq!(g.players[0].life, life_before - 3);
}

/// Path of Peace: kill an opp creature; their controller gains 4 life.
#[test]
fn path_of_peace_destroys_and_gives_opp_four_life() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::path_of_peace());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    let opp_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Path of Peace castable for {3}{W}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == serra),
        "Serra Angel destroyed");
    assert_eq!(g.players[1].life, opp_life_before + 4,
        "Opponent (target's controller) gains 4 life");
}

// ── modern_decks-9 tests ─────────────────────────────────────────────────────

/// Despise: target opp discards a chosen creature.
#[test]
fn despise_takes_a_creature_from_opp_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::lightning_bolt()); // non-creature padding
    let id = g.add_card_to_hand(0, catalog::despise());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Despise castable for {B}");
    drain_stack(&mut g);

    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "Bear (creature) should be the discard pick");
}

/// Distress: takes a non-creature non-land from opp hand.
#[test]
fn distress_takes_a_nonland_card_from_opp_hand() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::forest()); // land, can't be picked
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears()); // nonland (creature) — valid
    let id = g.add_card_to_hand(0, catalog::distress());
    g.players[0].mana_pool.add(Color::Black, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Distress castable for {B}{B}");
    drain_stack(&mut g);

    // CR — Distress hits any nonland card, creatures included.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears),
        "the nonland creature should be a legal discard pick");
}

/// Vryn Wingmare: 2/1 flying body with the noncreature-spell tax.
#[test]
fn vryn_wingmare_is_a_flying_two_one() {
    let g = two_player_game();
    let def = catalog::vryn_wingmare();
    assert_eq!(def.power, 2);
    assert_eq!(def.toughness, 1);
    assert!(def.keywords.contains(&crabomination::card::Keyword::Flying));
    assert_eq!(def.static_abilities.len(), 1,
        "Vryn Wingmare should ship its noncreature-tax static");
    let _ = g; // suppress unused
}

/// Vryn Wingmare's tax is observable: opp's second-spell-this-turn
/// gets a +{1} surcharge filtered to noncreature.
#[test]
fn vryn_wingmare_taxes_noncreature_spells_after_first_cast() {
    let mut g = two_player_game();
    let wingmare = g.add_card_to_battlefield(0, catalog::vryn_wingmare());
    g.clear_sickness(wingmare);
    // P1 has cast one spell already this turn — the next noncreature
    // spell should be taxed +{1}.
    g.players[1].spells_cast_this_turn = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    // {R} only — printed cost; with Vryn Wingmare's +{1} should fail.
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Bolt with only {{R}} should be rejected under Vryn Wingmare's tax: {:?}", err);
}

/// Lava Coil: 4 damage kills a 4-toughness creature.
#[test]
fn lava_coil_kills_a_four_toughness() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::lava_coil());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lava Coil castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == serra),
        "Lava Coil (4 damage) should kill Serra Angel (4 toughness)");
    // Push (modern_decks): Lava Coil now exiles creatures it would kill
    // instead of graveyarding them, approximating the printed "if that
    // creature would die this turn, exile it instead" rider.
    assert!(g.exile.iter().any(|c| c.id == serra),
        "Lava Coil should exile (not graveyard) creatures it would kill");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == serra),
        "Lava Coil should not put the dead creature in graveyard");
}

#[test]
fn lava_coil_deals_damage_without_killing_a_five_toughness() {
    // 4 damage doesn't kill a 5-toughness creature; the else branch
    // resolves with `DealDamage` only (no exile).
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());  // 5/5
    let id = g.add_card_to_hand(0, catalog::lava_coil());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(dragon)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lava Coil castable for {1}{R}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == dragon),
        "5-toughness dragon survives the 4 damage");
    let damage = g.battlefield_find(dragon).unwrap().damage;
    assert_eq!(damage, 4, "Dragon should have 4 damage marked");
}

/// Jaya's Greeting: 3 damage + scry 2.
#[test]
fn jayas_greeting_deals_three_and_scries() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::jayas_greeting());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Jaya's Greeting castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Jaya's Greeting (3 dmg) should kill Grizzly Bears");
}

/// Telling Time: scry 2 + draw 1 net 0 hand.
#[test]
fn telling_time_resolves_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::telling_time());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Telling Time castable for {1}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before,
        "Telling Time net 0 hand (cast -1, draw +1)");
}

/// Read the Tides mode 0: -1 cast + 3 draw = +2 hand.
#[test]
fn read_the_tides_draws_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::read_the_tides());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Read the Tides castable for {5}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
}

/// Read the Tides mode 1: return up to two target creatures to hand.
#[test]
fn read_the_tides_bounces_two_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::read_the_tides());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: Some(1),
        x_value: None,
    }).expect("Read the Tides castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == a || c.id == b),
        "both targeted creatures are returned to hand");
    assert!(g.players[1].hand.iter().any(|c| c.id == a),
        "Grizzly Bears returned to owner's hand");
    assert!(g.players[1].hand.iter().any(|c| c.id == b),
        "Serra Angel returned to owner's hand");
}

/// Last Gasp: -3/-3 kills a 3-toughness creature.
#[test]
fn last_gasp_kills_a_three_toughness() {
    let mut g = two_player_game();
    // Hypnotic Specter is 2/2 — let's use an explicit 3-toughness.
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::last_gasp());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Last Gasp castable for {1}{B}");
    drain_stack(&mut g);

    // 4 - 3 = 1 toughness left, still alive (no damage marked).
    let view = g.battlefield.iter().find(|c| c.id == serra);
    assert!(view.is_some(),
        "Serra (4/4) survives -3/-3 with 1 toughness left");
    // But a 3-toughness creature would die — verify with bear (2/2 → -1/-1 → dies).
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id2 = g.add_card_to_hand(0, catalog::last_gasp());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id2,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Last Gasp castable second time");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear (2/2) dies to -3/-3");
}

/// Wild Mongrel: discard ability gives +1/+1 EOT and sets a chosen color.
#[test]
fn wild_mongrel_pumps_via_discard() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let mongrel = g.add_card_to_battlefield(0, catalog::wild_mongrel());
    g.clear_sickness(mongrel);
    let fodder = g.add_card_to_hand(0, catalog::lightning_bolt());

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Discard(vec![fodder]),
        DecisionAnswer::Color(Color::Blue),
    ]));

    g.perform_action(GameAction::ActivateAbility {
        card_id: mongrel, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Wild Mongrel activates");
    drain_stack(&mut g);

    let view = g.battlefield.iter().find(|c| c.id == mongrel)
        .expect("Wild Mongrel still on battlefield");
    // Wild Mongrel is 2/2 + 1/1 EOT = 3/3.
    assert_eq!(view.power(), 3, "power should be base 2 + bonus 1 = 3");
    assert_eq!(view.toughness(), 3, "toughness should be base 2 + bonus 1 = 3");
    // Color of choice (Blue) replaces its printed green via layer 5.
    let computed = g.computed_permanent(mongrel).unwrap();
    assert_eq!(computed.colors.to_vec(), vec![Color::Blue], "becomes the chosen color");
    // Fodder discarded.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder),
        "Discarded card lands in graveyard");
}

// ── Modern utility lands and artifacts (modern_decks-10 batch) ──────────────

#[test]
fn glimmerpost_etbs_tapped_and_grants_one_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::glimmerpost());
    g.perform_action(GameAction::PlayLand(id))
        .expect("Glimmerpost playable as a land");
    drain_stack(&mut g);

    let card = g.battlefield_find(id).expect("Glimmerpost on the battlefield");
    assert!(card.tapped, "Glimmerpost has the etb-tap trigger");
    assert_eq!(g.players[0].life, life_before + 1,
        "ETB grants 1 life per Locus — just itself here");
}

#[test]
fn glimmerpost_etb_lifegain_scales_with_locus_count() {
    let mut g = two_player_game();
    // Already control a Cloudpost (a Locus); Glimmerpost makes two.
    g.add_card_to_battlefield(0, catalog::cloudpost());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::glimmerpost());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2,
        "two Loci → gain 2 life");
}

#[test]
fn cloudpost_mana_scales_with_locus_count() {
    let mut g = two_player_game();
    // Two Loci (a Cloudpost + a Glimmerpost) → Cloudpost taps for {C}{C}.
    g.add_card_to_battlefield(0, catalog::glimmerpost());
    let id = g.add_card_to_battlefield(0, catalog::cloudpost());
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).unwrap();
    assert_eq!(g.players[0].mana_pool.total(), 2,
        "Cloudpost adds {{C}} per Locus you control");
}

#[test]
fn glimmerpost_taps_for_colorless_after_untap() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::glimmerpost());
    // Drop the post-ETB tapped state before activating.
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    let total_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Glimmerpost mana ability");
    assert_eq!(g.players[0].mana_pool.total(), total_before + 1,
        "Glimmerpost taps for {{C}}");
}

#[test]
fn cloudpost_etbs_tapped_and_taps_for_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::cloudpost());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);

    assert!(g.battlefield_find(id).unwrap().tapped, "Cloudpost ETB-tapped");
    // Untap and verify mana ability.
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).unwrap();
    assert_eq!(g.players[0].mana_pool.total(), 1,
        "Cloudpost taps for one colorless");
}

#[test]
fn lotus_field_etb_sacrifices_two_lands() {
    let mut g = two_player_game();
    // Stock the battlefield with three Forests so the sac doesn't kill
    // the Field itself by triggering before it has friends to sacrifice.
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    let f2 = g.add_card_to_battlefield(0, catalog::forest());
    let f3 = g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::lotus_field());

    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);

    // The Field is on the battlefield (tapped via the ETB-tap step).
    assert!(g.battlefield_find(id).is_some(), "Lotus Field stays in play");
    assert!(g.battlefield_find(id).unwrap().tapped);
    // Two of the three forests sacrificed; one remains.
    let remaining_forests = [f1, f2, f3].iter()
        .filter(|fid| g.battlefield_find(**fid).is_some())
        .count();
    assert_eq!(remaining_forests, 1,
        "Lotus Field's ETB should sacrifice two of your lands");
}

/// CR 701.16 — a `wants_ui` player choosing a *multi* sacrifice (Lotus Field's
/// "sacrifice two lands") gets a `ChooseCards` modal to pick exactly two,
/// rather than the engine auto-dumping the weakest.
#[test]
fn lotus_field_ui_player_chooses_which_lands_to_sacrifice() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    let f2 = g.add_card_to_battlefield(0, catalog::forest());
    let f3 = g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::lotus_field());

    g.perform_action(GameAction::PlayLand(id)).unwrap();
    // Resolve the ETB trigger: both players pass priority.
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();

    let pd = g.pending_decision.as_ref().expect("a sacrifice choice is pending");
    assert_eq!(pd.acting_player(), 0);
    match &pd.decision {
        crabomination::decision::Decision::ChooseCards { candidates, min, max, .. } => {
            assert_eq!((*min, *max), (2, 2), "must choose exactly two");
            assert!(candidates.len() >= 4, "all of your lands are offered");
        }
        other => panic!("expected ChooseCards, got {other:?}"),
    }

    // Choose to sacrifice f1 and f2 — keep f3 and the Field.
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Cards(vec![f1, f2])))
        .expect("submit the multi-sacrifice choice");

    assert!(g.battlefield_find(f1).is_none(), "first chosen land sacrificed");
    assert!(g.battlefield_find(f2).is_none(), "second chosen land sacrificed");
    assert!(g.battlefield_find(f3).is_some(), "unchosen land kept");
    assert!(g.battlefield_find(id).is_some(), "Lotus Field kept");
}

#[test]
fn lotus_field_taps_for_three_of_one_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lotus_field());
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Lotus Field mana ability");
    // ManaPayload::AnyOneColor with Const(3) deposits 3 mana in a single color.
    assert_eq!(g.players[0].mana_pool.total(), 3,
        "Lotus Field should add 3 mana of one color");
}

#[test]
fn evolving_wilds_sacrifices_to_search_basic() {
    let mut g = two_player_game();
    // Stock a basic in the library to fetch.
    let plains_id = g.add_card_to_library(0, catalog::plains());
    let wilds_id = g.add_card_to_battlefield(0, catalog::evolving_wilds());
    g.battlefield.iter_mut().find(|c| c.id == wilds_id).unwrap().tapped = false;

    // Scripted decider picks the basic to fetch.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains_id))]));

    g.perform_action(GameAction::ActivateAbility {
        card_id: wilds_id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Evolving Wilds search ability");
    drain_stack(&mut g);

    // Wilds was sacrificed to its own cost; Plains is on the battlefield tapped.
    assert!(g.battlefield_find(wilds_id).is_none(),
        "Evolving Wilds sacrificed itself to its activation cost");
    let plains_inplay = g.battlefield_find(plains_id)
        .expect("Plains landed on the battlefield");
    assert!(plains_inplay.tapped, "Wilds searches put the basic onto BF tapped");
}

#[test]
fn mistvault_bridge_etbs_tapped_indestructible_artifact_land() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mistvault_bridge());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);

    let card = g.battlefield_find(id).unwrap();
    assert!(card.tapped, "Bridge ETB-tapped");
    assert!(card.definition.card_types.contains(&CardType::Artifact));
    assert!(card.definition.card_types.contains(&CardType::Land));
    assert!(card.definition.keywords.contains(&Keyword::Indestructible));
    // No basic land types (the printed bridges have none).
    assert!(card.definition.subtypes.land_types.is_empty());
}

#[test]
fn drossforge_bridge_taps_for_black_or_red() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::drossforge_bridge());
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    // Ability 0 = {T}: Add {B}.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1, "first ability adds black");
}

#[test]
fn coalition_relic_taps_for_one_mana_of_any_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::coalition_relic());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Coalition Relic's mana ability");
    // AnyOneColor — pool gains 1 mana of *some* color.
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

#[test]
fn coalition_relic_taps_to_add_charge_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::coalition_relic());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Coalition Relic's charge-counter ability");
    drain_stack(&mut g);
    let relic = g.battlefield_find(id).expect("relic still on battlefield");
    assert_eq!(relic.counter_count(CounterType::Charge), 1,
        "Activating ability #1 deposits one charge counter");
    assert!(relic.tapped, "tap-cost activated abilities tap the source");
}

#[test]
fn coalition_relic_precombat_burst_removes_all_charges_for_mana() {
    use crabomination::card::CounterType;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::coalition_relic());
    g.battlefield_find_mut(id).unwrap().add_counters(CounterType::Charge, 4);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    // Accept the optional "remove all charges, add a mana each" trigger.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Charge), 0,
        "all charge counters removed");
    assert_eq!(g.players[0].mana_pool.total(), 4,
        "one mana of any color per charge removed");
}

#[test]
fn coalition_relic_precombat_burst_can_be_declined() {
    use crabomination::card::CounterType;
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::coalition_relic());
    g.battlefield_find_mut(id).unwrap().add_counters(CounterType::Charge, 2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    // AutoDecider declines the MayDo by default.
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Charge), 2,
        "declined — charges kept");
    assert_eq!(g.players[0].mana_pool.total(), 0, "no mana when declined");
}

#[test]
fn ghost_vacuum_auto_target_picks_graveyard_card_when_present() {
    // Without the `prefers_graveyard_target` heuristic, the bot walks the
    // battlefield first and Ghost Vacuum (filter `Any`) would auto-target
    // a battlefield permanent — then exile it. The fix routes Move-to-
    // Exile spells through the graveyard walk first.
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let _vac = g.add_card_to_battlefield(0, catalog::ghost_vacuum());

    let target = g.auto_target_for_effect(
        &catalog::ghost_vacuum().activated_abilities[0].effect, 0
    );
    assert_eq!(target, Some(Target::Permanent(dead)),
        "Auto-target should pick a graveyard card, not a battlefield permanent");
}

#[test]
fn ghost_vacuum_exiles_target_card_from_graveyard() {
    let mut g = two_player_game();
    // Seed P1's graveyard with a Bear directly.
    let bear_id = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let vac = g.add_card_to_battlefield(0, catalog::ghost_vacuum());
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::ActivateAbility {
        card_id: vac, ability_index: 0, target: Some(Target::Permanent(bear_id)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Ghost Vacuum activated for {{2}}, {{T}}");
    drain_stack(&mut g);

    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear_id),
        "Bear left the graveyard");
    assert!(g.exile.iter().any(|c| c.id == bear_id),
        "Bear is now in the exile zone");
}

#[test]
fn all_bridges_are_indestructible_artifact_lands_with_two_color_taps() {
    use crabomination::card::CardDefinition;
    let factories: &[fn() -> CardDefinition] = &[
        catalog::mistvault_bridge, catalog::drossforge_bridge, catalog::razortide_bridge,
        catalog::goldmire_bridge, catalog::silverbluff_bridge, catalog::tanglepool_bridge,
        catalog::slagwoods_bridge, catalog::thornglint_bridge, catalog::darkmoss_bridge,
        catalog::rustvale_bridge,
    ];
    for &factory in factories {
        let def = factory();
        assert!(def.card_types.contains(&CardType::Artifact), "{}: artifact", def.name);
        assert!(def.card_types.contains(&CardType::Land), "{}: land", def.name);
        assert!(def.keywords.contains(&Keyword::Indestructible), "{}: indestructible", def.name);
        assert!(def.subtypes.land_types.is_empty(), "{}: no basic types", def.name);
        // Two mana abilities (one per colour) + the etb-tap trigger.
        assert_eq!(def.activated_abilities.len(), 2, "{}: two mana abilities", def.name);
        assert!(!def.triggered_abilities.is_empty(), "{}: etb-tap trigger", def.name);
    }
}

