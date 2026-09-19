//! CR 614.12 — "As this permanent enters" is a replacement effect, and its
//! choice is made before the permanent enters (CR 614.12a). The engine has
//! four battlefield-entry paths; these assert the funnel
//! (`game::as_enters::apply_as_enters_replacements`) runs on all of them, not
//! on the cast path alone.

use crabomination::card::{CardDefinition, CardType, EntersChoiceMode, Keyword};
use crabomination::catalog;
use crabomination::effect::Effect;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{cast, drain_stack, two_player_game};

/// A permanent whose whole body is an as-enters "choose a color" replacement
/// (Coldsteel Heart's shape), typed by the caller so the same body can be a
/// land, an artifact or a creature.
fn choose_color_body(name: &'static str, types: Vec<CardType>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: types,
        // A body, so the creature-typed form is not a 0/0 the first SBA sweep
        // eats before anything can copy it.
        power: 2,
        toughness: 2,
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        ..Default::default()
    }
}

// ── The land drop (CR 614.12a) ───────────────────────────────────────────────

/// A land is only ever *played*, so the land drop is the one entry path that
/// has to apply its as-enters replacement or the choice never happens at all.
/// Cavern of Souls, Unclaimed Territory and Three Tree City all live here.
#[test]
fn cr_614_12a_a_played_land_applies_its_as_enters_replacement() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, choose_color_body("As-Enters Land", vec![CardType::Land]));
    g.perform_action(GameAction::PlayLand(id)).expect("land drop is legal");
    let land = g.battlefield_find(id).expect("the land entered");
    assert!(
        land.chosen_color.is_some(),
        "CR 614.12a — the colour is chosen as the land enters, off the land drop"
    );
}

/// And the choice is made *before* the permanent is on the battlefield as far
/// as anything that reads it is concerned: no ETB trigger is on the stack
/// carrying it, so the stack is empty the moment the land drop returns.
#[test]
fn cr_614_12_the_as_enters_choice_leaves_nothing_on_the_stack() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, choose_color_body("As-Enters Land", vec![CardType::Land]));
    g.perform_action(GameAction::PlayLand(id)).expect("land drop is legal");
    assert!(
        g.stack.is_empty(),
        "CR 614.12 — a replacement modifies the entry; it never uses the stack"
    );
    assert!(g.battlefield_find(id).expect("entered").chosen_color.is_some());
}

// ── The universal move: reanimation, blink, put-onto-the-battlefield ─────────

/// CR 614.12a — a reanimated Corrupted Shapeshifter chooses its body as it
/// enters. Its printed line is `*/*` (0/0), so without the replacement it is
/// a 0/0 that the first SBA sweep kills.
#[test]
fn cr_614_12a_a_reanimated_permanent_makes_its_as_enters_choice() {
    let mut g = two_player_game();
    let id = g.move_card_to_battlefield_for_test(0, catalog::corrupted_shapeshifter());
    let cp = g.computed_permanent(id).expect("the Shapeshifter entered");
    assert_eq!(
        (cp.power, cp.toughness),
        (3, 3),
        "CR 614.12a — the default decider takes mode 0, a 3/3 flyer"
    );
    assert!(cp.keywords().contains(&Keyword::Flying), "the chosen mode's keyword too");
    g.check_state_based_actions();
    assert!(
        g.battlefield_find(id).is_some(),
        "CR 704.5f — the chosen body is in place before the first SBA sweep, so it survives"
    );
}

/// The same permanent reanimated without the replacement would be a 0/0: the
/// assertion above is about the *entry*, not about the card, so pin the
/// negative too — a body with no `enters_as_choice` really does die.
#[test]
fn cr_704_5f_a_zero_toughness_reanimation_with_no_as_enters_choice_dies() {
    let mut g = two_player_game();
    let id = g.move_card_to_battlefield_for_test(
        0,
        CardDefinition {
            name: "Bare Zero Over Zero",
            card_types: vec![CardType::Creature],
            ..Default::default()
        },
    );
    g.check_state_based_actions();
    assert!(g.battlefield_find(id).is_none(), "a 0/0 with no replacement dies to SBA");
}

// ── The token mint: CR 614.12's own worked example ──────────────────────────

/// CR 614.12 — "a token that enters as a copy of Voice of All has its colour
/// chosen as the token is created". A token has no cast and no spell, so the
/// mint is the only place the replacement can run.
#[test]
fn cr_614_12_a_token_copy_makes_the_copied_cards_as_enters_choice() {
    let mut g = two_player_game();
    // The copy source: a creature whose body is the as-enters colour choice.
    let src = g.add_card_to_battlefield(
        0,
        choose_color_body("As-Enters Creature", vec![CardType::Creature]),
    );
    let maker = g.add_card_to_hand(0, catalog::cackling_counterpart());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: maker,
        target: Some(Target::Permanent(src)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("the copier is castable");
    drain_stack(&mut g);
    let token = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.id != src)
        .expect("the token copy was minted");
    assert!(
        token.chosen_color.is_some(),
        "CR 614.12 — the copy's as-enters choice is made as the token is created"
    );
}

// ── The cast path keeps its answer (the funnel must not regress it) ─────────

/// The path that already applied every replacement still does, and the two
/// mode pickers moved ahead of the enters-with-counters step without moving
/// the answer: Corrupted Shapeshifter cast still enters as the chosen mode.
#[test]
fn cr_614_12a_the_cast_path_still_applies_the_mode_choice() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::corrupted_shapeshifter());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    cast(&mut g, id);
    let cp = g.computed_permanent(id).expect("the Shapeshifter resolved");
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// The funnel is one call in four places, so a card carrying *both* an
/// `as_enters_effect` and an `enters_as_choice` gets both on every path.
#[test]
fn cr_614_12_both_as_enters_replacements_apply_on_one_entry() {
    let mut g = two_player_game();
    let id = g.move_card_to_battlefield_for_test(
        0,
        CardDefinition {
            name: "Two Replacements",
            card_types: vec![CardType::Creature],
            as_enters_effect: Some(Effect::ChooseColorForSelf),
            enters_as_choice: Some(vec![EntersChoiceMode {
                power: 4,
                toughness: 4,
                keywords: vec![Keyword::Trample],
            }]),
            ..Default::default()
        },
    );
    let inst = g.battlefield_find(id).expect("entered");
    assert!(inst.chosen_color.is_some(), "the free-form replacement ran");
    let cp = g.computed_permanent(id).expect("entered");
    assert_eq!((cp.power, cp.toughness), (4, 4), "and so did the mode picker");
}

// ── The choose-a-name class (CR 201.3 + CR 614.12) ──────────────────────────

/// The nine "As this ~ enters, choose a … card name" cards, as one table.
/// Each was an `EntersBattlefield` trigger until 2026-09-19: the permanent was
/// on the battlefield with its name-keyed static live and *nothing chosen*
/// until its controller let the trigger resolve. What is asserted is the
/// window, not the field — the name is stamped by the time the entry returns,
/// with nothing left on the stack to resolve.
#[test]
fn cr_614_12a_the_choose_a_name_class_names_as_it_enters() {
    let namers: [(&str, fn() -> CardDefinition); 9] = [
        ("Pithing Needle", catalog::pithing_needle),
        ("Phyrexian Revoker", catalog::phyrexian_revoker),
        ("Disruptor Flute", catalog::disruptor_flute),
        ("Meddling Mage", catalog::meddling_mage),
        ("Alpine Moon", catalog::alpine_moon),
        ("Nevermore", catalog::nevermore),
        ("Council of the Absolute", catalog::council_of_the_absolute),
        ("Sorcerous Spyglass", catalog::sorcerous_spyglass),
        ("Silverquill Silencer", catalog::silverquill_silencer),
    ];
    for (name, factory) in namers {
        let mut g = two_player_game();
        // The namer's heuristic wants an opponent permanent with an activated
        // ability to point at; give it one.
        g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
        let id = g.move_card_to_battlefield_for_test(0, factory());
        let inst = g.battlefield_find(id).unwrap_or_else(|| panic!("{name} entered"));
        assert!(
            inst.named_card.is_some(),
            "CR 614.12a — {name} names as it enters, not off a trigger"
        );
        assert!(
            g.stack.is_empty(),
            "CR 614.12 — {name} leaves no trigger on the stack to carry the choice"
        );
    }
}

/// And the consequence the window had: Pithing Needle's name-keyed static is
/// live the moment the Needle is on the battlefield. CR 614.12a leaves no
/// priority in between for the named source's controller to use the ability
/// the Needle is about to lock.
#[test]
fn cr_614_12a_a_named_sources_ability_is_locked_with_no_window() {
    let mut g = two_player_game();
    let tim = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
    let needle = g.move_card_to_battlefield_for_test(0, catalog::pithing_needle());
    assert_eq!(
        g.battlefield_find(needle).and_then(|c| c.named_card.clone()).as_deref(),
        Some("Prodigal Sorcerer"),
        "the heuristic names the opponent's activated-ability permanent"
    );
    assert!(g.stack.is_empty(), "nothing is waiting to resolve");
    g.active_player_idx = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: tim,
            ability_index: 0,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
        "CR 614.12a — the lock is on before the named source's controller ever \
         holds priority"
    );
}
