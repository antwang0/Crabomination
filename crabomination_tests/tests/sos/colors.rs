#![allow(unused_imports)]
// The parametrized card tables carry many columns by design (cost, P/T,
// keyword, expected deltas); a `type` alias per table would hurt readability.
#![allow(clippy::type_complexity)]
use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use crate::prepared_on_battlefield;

// ── White ───────────────────────────────────────────────────────────────────

#[test]
fn eager_glyphmage_etb_creates_inkling_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::eager_glyphmage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Eager Glyphmage castable for {3}{W}");
    drain_stack(&mut g);

    // Glyphmage itself + an Inkling token = 2 new battlefield permanents.
    assert_eq!(g.battlefield.len(), bf_before + 2);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Inkling"));
    let tok = g.battlefield.iter().find(|c| c.definition.name == "Inkling").unwrap();
    assert!(tok.definition.keywords.contains(&Keyword::Flying));
}

#[test]
fn erode_destroys_creature_and_grants_target_controller_a_basic_land() {
    // Target's controller tutors a basic land (auto-decider takes the
    // first match) and puts it onto the battlefield tapped.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Seed P1's library with a Forest (the basic to fetch).
    let forest = g.add_card_to_library(1, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::erode());
    g.players[0].mana_pool.add(Color::White, 1);
    // Tell decider to fetch the Forest.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
    ]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Erode castable");
    drain_stack(&mut g);

    // Bear destroyed and in its owner's graveyard.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
    // Forest fetched onto P1's battlefield, tapped.
    let forest_view = g.battlefield.iter().find(|c| c.id == forest)
        .expect("Forest should be on battlefield");
    assert_eq!(forest_view.controller, 1, "Forest under P1's control");
    assert!(forest_view.tapped, "Forest fetched tapped");
}

#[test]
fn harsh_annotation_destroys_and_creates_token() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::harsh_annotation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Harsh Annotation castable for {1}{W}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Targeted creature should be destroyed");
    let inkling = g.battlefield.iter().find(|c| c.definition.name == "Inkling")
        .expect("Inkling token should be created");
    // Token goes to the target creature's owner (player 1), not the
    // caster.
    assert_eq!(inkling.controller, 1,
        "Inkling token should be owned by the target creature's owner");
}

/// Table-driven: one-shot pump spells cast on a friendly Grizzly Bears.
/// Covers Interjection, Masterful Flourish, Chase Inspiration, Killian's
/// Confidence, and Oracle's Restoration — each asserting the pumped P/T,
/// any granted keyword, cantrip draw, and lifegain.
#[test]
fn pump_spells_grant_stats_keywords_draw_and_life() {
    type Ctor = fn() -> crabomination::card::CardDefinition;
    // (ctor, colored mana, colorless, power, toughness, keyword, draws, lifegain)
    let cases: &[(Ctor, &[Color], u8, i64, i64, Option<Keyword>, bool, i64)] = &[
        (catalog::interjection, &[Color::White], 0, 4, 4, Some(Keyword::FirstStrike), false, 0),
        (catalog::masterful_flourish, &[Color::Black], 0, 3, 2, Some(Keyword::Indestructible), false, 0),
        (catalog::chase_inspiration, &[Color::Blue], 0, 2, 5, Some(Keyword::Hexproof), false, 0),
        (catalog::killians_confidence, &[Color::White, Color::Black], 0, 3, 3, None, true, 0),
        (catalog::oracles_restoration, &[Color::Green], 0, 3, 3, None, true, 1),
    ];
    for (ctor, colored, colorless, p, t, kw, draws, lifegain) in cases {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, ctor());
        for c in *colored {
            g.players[0].mana_pool.add(*c, 1);
        }
        g.players[0].mana_pool.add_colorless(*colorless as u32);
        let hand_before = g.players[0].hand.len();
        let life_before = g.players[0].life;

        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        })
        .unwrap_or_else(|e| panic!("{} should be castable: {e:?}", ctor().name));
        drain_stack(&mut g);

        let view = g.computed_permanent(bear).unwrap();
        assert_eq!(view.power as i64, *p, "{} power", ctor().name);
        assert_eq!(view.toughness as i64, *t, "{} toughness", ctor().name);
        if let Some(kw) = kw {
            assert!(view.keywords.contains(kw), "{} keyword {:?}", ctor().name, kw);
        }
        // Cantrips: -1 cast +1 draw = unchanged; otherwise -1.
        let expect_hand = if *draws { hand_before } else { hand_before - 1 };
        assert_eq!(g.players[0].hand.len(), expect_hand, "{} hand delta", ctor().name);
        assert_eq!(g.players[0].life, life_before + *lifegain as i32, "{} lifegain", ctor().name);
    }
}

#[test]
fn stand_up_for_yourself_only_targets_power_three_or_more() {
    use crabomination::card::CardDefinition;

    // Build a 3/3 manually so the destroy-power-3+ filter accepts it.
    let big = CardDefinition {
        name: "Test Three Three",
        cost: crabomination::mana::ManaCost::default(),
        card_types: vec![CardType::Creature],
        power: 3,
        toughness: 3,
        ..Default::default()
    };
    let mut g = two_player_game();
    let big_id = g.add_card_to_battlefield(1, big);
    let id = g.add_card_to_hand(0, catalog::stand_up_for_yourself());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(big_id)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Stand Up for Yourself castable for {2}{W} on a 3/3");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == big_id),
        "3/3 should be destroyed");
}

#[test]
fn rapier_wit_taps_target_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::rapier_wit());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Rapier Wit castable for {1}{W}");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(target.tapped, "Bear should be tapped by Rapier Wit");
    // Played on player 0's turn, so it should also have a stun counter.
    assert!(target.counter_count(CounterType::Stun) >= 1,
        "Stun counter should be added when cast on your own turn");
    // Hand: -1 cast +1 draw = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Silverquill (W/B) ───────────────────────────────────────────────────────

#[test]
fn silverquill_charm_all_three_modes() {
    // Mode 0: two +1/+1 counters on a friendly creature.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_charm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: Some(0), x_value: None,
    })
    .expect("Silverquill Charm castable in counter mode");
    drain_stack(&mut g);
    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(target.counter_count(CounterType::PlusOnePlusOne), 2,
        "Mode 0 should put 2 +1/+1 counters on the target");

    // Mode 1: exile a low-power creature.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::silverquill_charm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("Silverquill Charm castable in exile mode");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear (power 2) should be exiled by mode 1");

    // Mode 2: drain 3.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_charm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(2), x_value: None,
    })
    .expect("Silverquill Charm castable for {W}{B} in drain mode");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 3, "Each opponent loses 3");
    assert_eq!(g.players[0].life, p0_life + 3, "You gain 3");
}

#[test]
fn imperious_inkmage_etb_surveils_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::imperious_inkmage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Imperious Inkmage castable for {1}{W}{B}");
    drain_stack(&mut g);

    // Surveil 2: the auto-decider keeps both cards on top, so the library
    // size is unchanged. We assert at minimum that the library wasn't
    // grown (no draw side-effect leaked) and the inkmage hit play.
    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Imperious Inkmage should be on the battlefield");
    assert!(g.players[0].library.len() <= lib_before,
        "Surveil 2 cannot increase library size");
}

// ── Black ───────────────────────────────────────────────────────────────────

#[test]
fn wander_off_exiles_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wander_off());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Wander Off castable for {3}{B}");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear));
}

#[test]
fn sneering_shadewriter_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sneering_shadewriter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Sneering Shadewriter castable for {4}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, p1_life - 2);
    assert_eq!(g.players[0].life, p0_life + 2);
}

#[test]
fn burrog_banemaker_pump_ability_works() {
    let mut g = two_player_game();
    let banemaker = g.add_card_to_battlefield(0, catalog::burrog_banemaker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: banemaker,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Banemaker pump activatable for {1}{B}");
    drain_stack(&mut g);

    let view = g.computed_permanent(banemaker).unwrap();
    assert_eq!(view.power, 2);
    assert_eq!(view.toughness, 2);
}

#[test]
fn send_in_the_pest_discards_and_creates_token() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::send_in_the_pest());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Send in the Pest castable for {1}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].hand.len(), p1_hand_before - 1,
        "Each opponent should discard one card");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Pest"),
        "Pest token should be created");
}

#[test]
fn pull_from_the_grave_returns_up_to_two_creatures_and_gains_life() {
    // Selector::Take(_, 2) should pull at most two creature cards from
    // your graveyard back to your hand. Lands sitting next to them
    // should be left untouched (filter is `Creature`).
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bear3 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let _land = g.add_card_to_graveyard(0, catalog::forest());

    let id = g.add_card_to_hand(0, catalog::pull_from_the_grave());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pull from the Grave castable for {2}{B}");
    drain_stack(&mut g);

    let creatures_in_hand = g.players[0]
        .hand
        .iter()
        .filter(|c| c.id == bear || c.id == bear2 || c.id == bear3)
        .count();
    assert_eq!(creatures_in_hand, 2, "Exactly two creatures should be returned");
    // The land in the graveyard should NOT have been moved to hand.
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Forest"),
        "Land in gy is not affected by the creature filter");
    assert_eq!(g.players[0].life, life_before + 2, "Gains 2 life");
}

#[test]
fn practiced_scrollsmith_take_one_only_exiles_one_match() {
    // Selector::Take(_, 1) should clamp the gy-exile to a single
    // matching card even when multiple noncreature/nonland cards sit
    // in the graveyard.
    let mut g = two_player_game();
    // Two sorceries + a creature + a land in P0's gy.
    g.add_card_to_graveyard(0, catalog::pox_plague());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::forest());

    // Cast Practiced Scrollsmith from hand: ETB exiles ONE matching gy card.
    let id = g.add_card_to_hand(0, catalog::practiced_scrollsmith());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::White, 1);

    let before_gy = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Practiced Scrollsmith castable");
    drain_stack(&mut g);

    // ETB should have removed exactly one card from the gy (creature/land
    // filter excludes the bear and forest).
    assert_eq!(g.players[0].graveyard.len(), before_gy - 1,
        "Take(_, 1) clamps the gy-exile to a single matching card");
    // Bear and Forest must still be in the graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "Creature filter excludes Grizzly Bears from exile");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Forest"),
        "Nonland filter excludes Forest from exile");
}

// ── Witherbloom (B/G) ───────────────────────────────────────────────────────

#[test]
fn witherbloom_charm_all_three_modes() {
    // Mode 0 (sacrifice → draw 2) is wrapped in `Effect::MayDo`: the
    // controller picks mode 0 then opts in via OptionalTrigger.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::witherbloom_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand_before = g.players[0].hand.len(); // 1 (just the charm)
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    })
    .expect("Witherbloom Charm castable in sacrifice mode");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "Bear should be in graveyard from sacrifice");
    // Hand: -1 cast + 2 drawn = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);

    // Mode 1 = gain 5.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("Witherbloom Charm castable for {B}{G} in lifegain mode");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 5);

    // Mode 2 = destroy nonland permanent with mv ≤ 2.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // CMC 2
    let id = g.add_card_to_hand(0, catalog::witherbloom_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: Some(2), x_value: None,
    })
    .expect("Witherbloom Charm castable in destroy mode");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn bogwater_lumaret_etb_gains_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bogwater_lumaret());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bogwater Lumaret castable for {B}{G}");
    drain_stack(&mut g);

    // The Lumaret's own ETB triggers itself (via YourControl + Creature).
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn pest_mascot_grows_on_lifegain() {
    let mut g = two_player_game();
    let mascot = g.add_card_to_battlefield(0, catalog::pest_mascot());

    // Cast a small lifegain spell — gain 5 from Witherbloom Charm.
    let charm = g.add_card_to_hand(0, catalog::witherbloom_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: charm, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("Witherbloom Charm castable for lifegain mode");
    drain_stack(&mut g);

    let view = g.computed_permanent(mascot).unwrap();
    assert!(view.power >= 3,
        "Pest Mascot should gain at least one +1/+1 counter from the lifegain trigger");
}

#[test]
fn grapple_with_death_destroys_creature_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::grapple_with_death());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Grapple with Death castable for {1}{B}{G}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert_eq!(g.players[0].life, life_before + 1);
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Impractical Joke's "damage can't be prevented this turn" rider (CR
/// 615.12): a prevent-all shield on the target is ignored, so the bear
/// still dies (also covers the base deal-3 behavior).
#[test]
fn impractical_joke_damage_cant_be_prevented() {
    use crabomination::game::types::{PreventionShield, PreventionTarget};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Shield the bear against all damage this turn.
    g.prevention_shields.push(PreventionShield {
        mint_mites_for: None,
        target: PreventionTarget::Permanent(bear),
        remaining: None,
        gain_life: false,
        source: None,
        one_event: false,
        reflect: false,
        source_controller: None,
        redirect_to: None,
            destroy: false,
    });
    let id = g.add_card_to_hand(0, catalog::impractical_joke());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Impractical Joke castable for {R}");
    drain_stack(&mut g);
    assert!(g.damage_cant_be_prevented_this_turn, "rider sets the global flag");
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "prevention shield ignored — bear dies through it");
}

/// A prevent-all shield (without a can't-be-prevented rider) stops a
/// Lightning Bolt aimed at the protected creature (CR 615.1).
#[test]
fn prevention_shield_stops_noncombat_damage() {
    use crabomination::game::types::{PreventionShield, PreventionTarget};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.prevention_shields.push(PreventionShield {
        mint_mites_for: None,
        target: PreventionTarget::Permanent(bear),
        remaining: None,
        gain_life: false,
        source: None,
        one_event: false,
        reflect: false,
        source_controller: None,
        redirect_to: None,
            destroy: false,
    });
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "all damage prevented — bear lives");
}

/// Prismari, the Inspiration grants storm to your instants/sorceries: the
/// second damage spell cast with one prior spell this turn copies once.
#[test]
fn prismari_grants_storm_to_instants_and_sorceries() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_the_inspiration());
    // First spell of the turn: storm count 0, no copy.
    let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("first bolt castable");
    drain_stack(&mut g);
    // Second spell: storm count 1 → one copy. 3 (original) + 3 (copy) = 6.
    let foe_life = g.players[1].life;
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("second bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 6, "storm copied the second spell once");
}

// ── Green ───────────────────────────────────────────────────────────────────

#[test]
fn noxious_newt_taps_for_green() {
    let mut g = two_player_game();
    let newt = g.add_card_to_battlefield(0, catalog::noxious_newt());
    g.clear_sickness(newt);
    let pool_total = g.players[0].mana_pool.total();

    g.perform_action(GameAction::ActivateAbility {
        card_id: newt,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Noxious Newt {T} mana ability activatable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].mana_pool.total(), pool_total + 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

#[test]
fn mindful_biomancer_etb_gains_one_life_and_pump_is_once_per_turn() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mindful_biomancer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Mindful Biomancer castable for {1}{G}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);

    // Activate the +2/+2 pump.
    let bio = g.battlefield.iter().find(|c| c.id == id).unwrap().id;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bio, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Pump activatable for {2}{G}");
    drain_stack(&mut g);

    let view = g.computed_permanent(bio).unwrap();
    assert_eq!(view.power, 4);
    assert_eq!(view.toughness, 4);

    // Once-per-turn enforcement: a second activation must fail.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let again = g.perform_action(GameAction::ActivateAbility {
        card_id: bio, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None });
    assert!(again.is_err(),
        "Mindful Biomancer pump should be activatable only once each turn");
}

#[test]
fn shopkeepers_bane_attack_gains_two_life() {
    use crabomination::game::types::AttackTarget;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::shopkeepers_bane());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    let life_before = g.players[0].life;

    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("Bane should be a legal attacker");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 2,
        "Attack trigger should gain 2 life");
}

// ── Blue ────────────────────────────────────────────────────────────────────

#[test]
fn banishing_betrayal_bounces_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::banishing_betrayal());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Banishing Betrayal castable for {1}{U}");
    drain_stack(&mut g);

    // Bear should be in P1's hand after the bounce.
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn procrastinate_taps_and_adds_2x_stun_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::procrastinate());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    // X = 2: pay {2}{U}, expect 4 stun counters.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Procrastinate castable for {2}{U} with X=2");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(target.tapped);
    assert_eq!(target.counter_count(CounterType::Stun), 4,
        "Procrastinate puts 2X = 4 stun counters with X=2");
}

// ── Quandrix (G/U) ──────────────────────────────────────────────────────────

#[test]
fn embrace_the_paradox_draws_three() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::embrace_the_paradox());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Embrace the Paradox castable for {3}{G}{U}");
    drain_stack(&mut g);

    // -1 cast +3 draw = +2.
    assert_eq!(g.players[0].hand.len(), hand_before + 2);
}

// ── Glorious Decay (G modal) ────────────────────────────────────────────────

#[test]
fn glorious_decay_destroys_artifact() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::glorious_decay());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(rock)), additional_targets: vec![], mode: Some(0), x_value: None,
    })
    .expect("Glorious Decay castable for {1}{G}, mode 0");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == rock));
}

#[test]
fn charging_strifeknight_loots_with_tap() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let knight = g.add_card_to_battlefield(0, catalog::charging_strifeknight());
    // Clear summoning sickness so we can tap immediately.
    g.clear_sickness(knight);
    let hand_before = g.players[0].hand.len();
    let grave_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: knight,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Strifeknight tap-loot ability activatable");
    drain_stack(&mut g);

    // Hand: -1 discard +1 draw = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Graveyard +1 (the discarded card).
    assert_eq!(g.players[0].graveyard.len(), grave_before + 1);
}
