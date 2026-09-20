//! CR 614.12 — "As this permanent enters" is a replacement effect, and its
//! choice is made before the permanent enters (CR 614.12a). The engine has
//! four battlefield-entry paths; these assert the funnel
//! (`game::as_enters::apply_as_enters_replacements`) runs on all of them, not
//! on the cast path alone.

use crabomination::card::{CardDefinition, CardType, EntersChoiceMode, Keyword};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{cast, drain_stack, two_player_game};

/// The shape both class tables below are written in. Aliased because a bare
/// `[(&str, fn() -> CardDefinition); N]` trips `clippy::type_complexity`; the
/// same alias is what `counters.rs` and `structural_audit.rs` use.
type Factory = fn() -> CardDefinition;
/// One row of the stamp table: name, factory, and what the entry must have
/// written on the instance by the time it returns.
type Stamped = (&'static str, Factory, fn(&crabomination::card::CardInstance) -> bool);

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
    let namers: [(&str, Factory); 9] = [
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

// ── The choose-a-colour class (CR 614.12) ──────────────────────────────────

/// The twenty "As this ~ enters, choose a color" cards, as one table. Same
/// window argument as the choose-a-name class: until 2026-09-19 each shipped
/// an `EntersBattlefield` trigger, so a colour-keyed static (Caged Sun's
/// "+1/+1 to creatures of the chosen color", Story Circle's prevention) was
/// live against an *unchosen* colour for at least one SBA check.
///
/// Auras are excluded: they need an attach target to enter at all, and their
/// own entry is covered by `cho_mannos_blessing_chooses_as_it_enters` below.
#[test]
fn cr_614_12a_the_choose_a_colour_class_chooses_as_it_enters() {
    let namers: [(&str, Factory); 14] = [
        ("Caged Sun", catalog::caged_sun),
        ("Chameleon Spirit", catalog::chameleon_spirit),
        ("Coldsteel Heart", catalog::coldsteel_heart),
        ("Diamond Mare", catalog::diamond_mare),
        ("Hall of Triumph", catalog::hall_of_triumph),
        ("Heraldic Banner", catalog::heraldic_banner),
        ("Iona, Shield of Emeria", catalog::iona_shield_of_emeria),
        ("Jeweled Torque", catalog::jeweled_torque),
        ("Order of the Stars", catalog::order_of_the_stars),
        ("Quirion Elves", catalog::quirion_elves),
        ("Silhana Starfletcher", catalog::silhana_starfletcher),
        ("Story Circle", catalog::story_circle),
        ("Volrath's Laboratory", catalog::volraths_laboratory),
        ("Ward Sliver", catalog::ward_sliver),
    ];
    for (name, factory) in namers {
        let mut g = two_player_game();
        let id = g.move_card_to_battlefield_for_test(0, factory());
        let inst = g.battlefield_find(id).unwrap_or_else(|| panic!("{name} entered"));
        assert!(
            inst.chosen_color.is_some(),
            "CR 614.12a — {name} chooses its colour as it enters"
        );
        assert!(
            g.stack.is_empty(),
            "CR 614.12 — {name} leaves no trigger on the stack to carry the choice"
        );
    }
}

/// Caged Sun's colour-keyed anthem is the consequence: before the conversion
/// the Sun was on the battlefield with the anthem live and no colour chosen,
/// so a creature it should have been pumping read its printed body across at
/// least one state-based-action check.
#[test]
fn cr_614_12a_a_colour_keyed_anthem_is_live_with_the_colour_already_chosen() {
    use crabomination::decision::OneColorDecider;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(OneColorDecider::new(Color::Green));
    let sun = g.move_card_to_battlefield_for_test(0, catalog::caged_sun());
    assert_eq!(
        g.battlefield_find(sun).and_then(|c| c.chosen_color),
        Some(Color::Green),
        "the colour is chosen as the Sun enters"
    );
    assert!(g.stack.is_empty(), "nothing is waiting to resolve");
    let cp = g.computed_permanent(bears).expect("the Bears are still there");
    assert_eq!(
        (cp.power, cp.toughness),
        (3, 3),
        "CR 614.12a — the anthem reads the chosen colour with no window before it"
    );
}

/// An Aura's as-enters colour choice happens on the *cast* path, where it
/// always did — the conversion must not have cost it.
#[test]
fn cho_mannos_blessing_chooses_as_it_enters() {
    use crabomination::decision::OneColorDecider;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::cho_mannos_blessing());
    g.players[0].mana_pool.add(Color::White, 2);
    g.decider = Box::new(OneColorDecider::new(Color::Red));
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("the Aura is castable");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(aura).and_then(|c| c.chosen_color),
        Some(Color::Red),
        "CR 614.12a — the Aura's colour is chosen as it enters"
    );
}


// ── The land drop's suspension (CR 614.12a on a `wants_ui` seat) ────────────

/// A land drop is the one entry path with nowhere to park a continuation: no
/// stack item above it, and nothing replayable behind it (the land is pushed
/// and the drop is spent). `ResumeContext::LandEntry` is that place — a
/// `wants_ui` controller is asked, and the entry finishes on the answer.
#[test]
fn cr_614_12a_a_ui_seat_is_asked_for_a_land_drops_as_enters_choice() {
    use crabomination::card::CreatureType;
    use crabomination::decision::DecisionAnswer;
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let id = g.add_card_to_hand(
        0,
        CardDefinition {
            name: "Asking Land",
            card_types: vec![CardType::Land],
            as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
            ..Default::default()
        },
    );
    g.perform_action(GameAction::PlayLand(id)).expect("land drop is legal");
    assert!(g.stack.is_empty(), "CR 614.12 — a replacement never uses the stack");
    assert!(g.pending_decision.is_some(), "the UI seat is asked, not answered for");
    g.submit_decision(DecisionAnswer::CreatureType(CreatureType::Bear)).expect("name Bear");
    assert_eq!(
        g.battlefield_find(id).expect("still on the battlefield").chosen_creature_type,
        Some(CreatureType::Bear),
        "CR 614.12a — the named type is the one that stuck"
    );
    assert!(g.pending_decision.is_none(), "the entry finished on the answer");
}

/// The rest of the entry is owed after the answer, not before it: a land that
/// both asks and enters with counters gets its counters on the resume, once,
/// and its ETB triggers fire there too.
#[test]
fn cr_614_12a_a_suspended_land_drop_finishes_its_entry_on_the_answer() {
    use crabomination::card::{CounterType, CreatureType};
    use crabomination::decision::DecisionAnswer;
    use crabomination::effect::Value;
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let id = g.add_card_to_hand(
        0,
        CardDefinition {
            name: "Asking Depletion Land",
            card_types: vec![CardType::Land],
            as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
            enters_with_counters: Some((CounterType::Depletion, Value::Const(2))),
            ..Default::default()
        },
    );
    g.perform_action(GameAction::PlayLand(id)).expect("land drop is legal");
    assert_eq!(
        g.battlefield_find(id).map(|c| c.counter_count(CounterType::Depletion)),
        Some(0),
        "CR 614.12a — the entry is paused at the ask, so nothing after it has run"
    );
    g.submit_decision(DecisionAnswer::CreatureType(CreatureType::Bear)).expect("name Bear");
    let land = g.battlefield_find(id).expect("still on the battlefield");
    assert_eq!(land.chosen_creature_type, Some(CreatureType::Bear));
    assert_eq!(
        land.counter_count(CounterType::Depletion),
        2,
        "the printed entering counters land on the resume, exactly once"
    );
}

/// A bot seat does not suspend, so the land drop answers through the decider
/// in one call and never parks anything — the path the simulator takes.
#[test]
fn cr_614_12a_a_headless_seats_land_drop_never_parks() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, choose_color_body("As-Enters Land", vec![CardType::Land]));
    g.perform_action(GameAction::PlayLand(id)).expect("land drop is legal");
    assert!(g.pending_decision.is_none(), "a headless seat is answered for, not asked");
    assert!(g.battlefield_find(id).expect("entered").chosen_color.is_some());
}
// ── The choose-a-type class (CR 614.12) ────────────────────────────────────

/// The twenty-four "As this ~ enters, choose a creature type" cards, as one
/// table. Engineered Plague is the sharp one — "creatures of the chosen type
/// get -1/-1" was live against an *unchosen* type for at least one SBA check
/// while its trigger sat on the stack.
#[test]
fn cr_614_12a_the_choose_a_type_class_chooses_as_it_enters() {
    let namers: [(&str, Factory); 19] = [
        ("Adaptive Automaton", catalog::adaptive_automaton),
        ("An-Zerrin Ruins", catalog::an_zerrin_ruins),
        ("Ashes of the Fallen", catalog::ashes_of_the_fallen),
        ("Conspiracy", catalog::conspiracy),
        ("Door of Destinies", catalog::door_of_destinies),
        ("Engineered Plague", catalog::engineered_plague),
        ("Icon of Ancestry", catalog::icon_of_ancestry),
        ("Kindred Discovery", catalog::kindred_discovery),
        ("Metallic Mimic", catalog::metallic_mimic),
        ("Obelisk of Urd", catalog::obelisk_of_urd),
        ("Patchwork Banner", catalog::patchwork_banner),
        ("Plague Engineer", catalog::plague_engineer),
        ("Radiant Destiny", catalog::radiant_destiny),
        ("Rally the Ranks", catalog::rally_the_ranks),
        ("Reflections of Littjara", catalog::reflections_of_littjara),
        ("Shared Triumph", catalog::shared_triumph),
        ("Vanquisher's Banner", catalog::vanquishers_banner),
        ("Volrath's Laboratory", catalog::volraths_laboratory),
        ("Xenograft", catalog::xenograft),
    ];
    for (name, factory) in namers {
        let mut g = two_player_game();
        // Something on the board for the type heuristic to name.
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.move_card_to_battlefield_for_test(0, factory());
        let inst = g.battlefield_find(id).unwrap_or_else(|| panic!("{name} entered"));
        assert!(
            inst.chosen_creature_type.is_some(),
            "CR 614.12a — {name} names its creature type as it enters"
        );
        assert!(
            g.stack.is_empty(),
            "CR 614.12 — {name} leaves no trigger on the stack to carry the choice"
        );
    }
}

/// Shimmer's clause names a **land** type (CR 205.3i), stamped on
/// `chosen_land_type`; the class is one clause with three namespaces.
#[test]
fn cr_614_12a_shimmer_chooses_a_land_type_as_it_enters() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::forest());
    let id = g.move_card_to_battlefield_for_test(0, catalog::shimmer());
    let inst = g.battlefield_find(id).expect("Shimmer entered");
    assert!(
        inst.chosen_land_type.is_some(),
        "CR 614.12a — the land type is chosen as Shimmer enters"
    );
    assert!(g.stack.is_empty(), "no trigger carries it");
}

/// Serra's Emissary is the class's other odd member: it chooses a **card**
/// type,
/// not a creature type (CR 205.2a), and stamps `chosen_card_type`.
#[test]
fn cr_614_12a_serras_emissary_chooses_a_card_type_as_it_enters() {
    let mut g = two_player_game();
    let id = g.move_card_to_battlefield_for_test(0, catalog::serras_emissary());
    let inst = g.battlefield_find(id).expect("the Emissary entered");
    assert!(
        inst.chosen_card_type.is_some(),
        "CR 614.12a — the card type is chosen as the Emissary enters"
    );
    assert!(g.stack.is_empty(), "no trigger carries it");
}

/// The three type-namers that are *lands*: their only entry is a land drop,
/// which applied no as-enters replacement at all until the funnel landed.
/// Cavern of Souls' restricted mana is unusable without the chosen type, so
/// the window it left was a land that tapped for nothing.
#[test]
fn cr_614_12a_a_type_naming_land_names_off_the_land_drop() {
    let lands: [(&str, Factory); 3] = [
        ("Cavern of Souls", catalog::cavern_of_souls),
        ("Secluded Courtyard", catalog::secluded_courtyard),
        ("Three Tree City", catalog::three_tree_city),
    ];
    for (name, factory) in lands {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, factory());
        g.perform_action(GameAction::PlayLand(id)).expect("land drop is legal");
        let land = g.battlefield_find(id).unwrap_or_else(|| panic!("{name} entered"));
        assert!(
            land.chosen_creature_type.is_some(),
            "CR 614.12a — {name} names its type as it is played"
        );
        assert!(g.stack.is_empty(), "{name} leaves no trigger behind");
    }
}

/// And the consequence, on the card the class exists for: Engineered Plague's
/// type-keyed penalty is applied against a type that is already chosen, with
/// no state-based-action check in between.
#[test]
fn cr_614_12a_a_type_keyed_penalty_never_reads_an_unchosen_type() {
    use crabomination::card::CreatureType;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 Bear
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(
        CreatureType::Bear,
    )]));
    let plague = g.move_card_to_battlefield_for_test(0, catalog::engineered_plague());
    assert_eq!(
        g.battlefield_find(plague).and_then(|c| c.chosen_creature_type),
        Some(CreatureType::Bear),
        "the type is chosen as the Plague enters"
    );
    assert!(g.stack.is_empty(), "nothing is waiting to resolve");
    let cp = g.computed_permanent(bears).expect("the Bears are still there");
    assert_eq!(
        (cp.power, cp.toughness),
        (1, 1),
        "CR 614.12a — -1/-1 is on before any SBA check could read an unchosen type"
    );
}

// ── The pay-life-or-tapped class (CR 614.12 + CR 119.4) ────────────────────

/// The ten shocklands and their nine {3}-life descendants. This was the last
/// bucket of the 614.12 census and the one with the highest EDHREC ranks
/// (Watery Grave 52, Godless Shrine 61, Hallowed Fountain 65, Overgrown Tomb
/// 73), and the consequence the class is named for is here: as a trigger the
/// land sat on the battlefield **untapped** with the choice still on the
/// stack, so its controller could hold priority and tap it for mana it should
/// never have made.
#[test]
fn cr_614_12a_a_shockland_resolves_its_choice_inside_the_land_drop() {
    let lands: [(&str, Factory); 6] = [
        ("Watery Grave", catalog::watery_grave),
        ("Godless Shrine", catalog::godless_shrine),
        ("Hallowed Fountain", catalog::hallowed_fountain),
        ("Overgrown Tomb", catalog::overgrown_tomb),
        ("Blood Crypt", catalog::blood_crypt),
        ("Steam Vents", catalog::steam_vents),
    ];
    for (name, factory) in lands {
        let mut g = two_player_game();
        let before = g.players[0].life;
        let id = g.add_card_to_hand(0, factory());
        g.perform_action(GameAction::PlayLand(id)).expect("land drop is legal");
        assert!(
            g.stack.is_empty(),
            "CR 614.12 — {name} leaves no trigger on the stack, so there is no \
             window to tap it in"
        );
        let land = g.battlefield_find(id).unwrap_or_else(|| panic!("{name} entered"));
        // Whichever branch the decider took, the land and the life total agree
        // by the time the drop returns: paid and untapped, or unpaid and tapped.
        assert_eq!(
            land.tapped,
            g.players[0].life == before,
            "{name}: tapped iff the 2 life was not paid"
        );
    }
}

/// CR 119.4 — a player who cannot pay the life is not asked, and the land
/// enters tapped. Shocklands are the one land that can kill you, so the floor
/// matters: at 2 life the payment is legal (life may reach 0 as a cost is
/// paid, and CR 704.5a kills you afterwards); at 1 it is not.
#[test]
fn cr_119_4_a_seat_that_cannot_pay_the_life_enters_the_land_tapped() {
    let mut g = two_player_game();
    g.players[0].life = 1;
    let id = g.add_card_to_hand(0, catalog::watery_grave());
    g.perform_action(GameAction::PlayLand(id)).expect("land drop is legal");
    assert_eq!(g.players[0].life, 1, "CR 119.4 — the life was never payable");
    assert!(
        g.battlefield_find(id).expect("entered").tapped,
        "so the land takes the printed downside and enters tapped"
    );
}

/// A permanent that *enters* tapped never *becomes* tapped, so the decline
/// branch must not emit `PermanentTapped` — `Effect::Tap` did, which is what
/// `Effect::SourceEntersTapped` replaces.
#[test]
fn cr_614_12_entering_tapped_is_not_becoming_tapped() {
    let mut g = two_player_game();
    g.players[0].life = 1; // forces the decline branch
    let id = g.add_card_to_hand(0, catalog::watery_grave());
    let events = g.perform_action(GameAction::PlayLand(id)).expect("land drop is legal");
    assert!(
        g.battlefield_find(id).expect("entered").tapped,
        "it entered tapped"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentTapped { card_id, .. } if *card_id == id)),
        "CR 614.12 — entering tapped is not a tap event"
    );
}

/// CR 614 — an "enters untapped" replacement (Spelunking) outranks the
/// decline branch, exactly as it outranks `StaticEffect::EntersTapped`. Both
/// read the same helper now, so they cannot disagree.
#[test]
fn cr_614_spelunking_outranks_the_decline_branch() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spelunking());
    g.players[0].life = 1; // forces the decline branch
    let id = g.add_card_to_hand(0, catalog::watery_grave());
    g.perform_action(GameAction::PlayLand(id)).expect("land drop is legal");
    assert!(
        !g.battlefield_find(id).expect("entered").tapped,
        "CR 614 — lands you control enter untapped, decline branch or not"
    );
}

/// And a `wants_ui` seat is asked rather than answered for: the land drop
/// parks the two-mode question on `ResumeContext::LandEntry` and finishes the
/// entry on the answer. `AutoDecider` would have answered mode 0 (pay), so
/// taking mode 1 here proves the prompt was real.
#[test]
fn cr_614_12a_a_ui_seat_is_asked_whether_to_pay_the_shockland() {
    use crabomination::decision::DecisionAnswer;
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let id = g.add_card_to_hand(0, catalog::steam_vents());
    let before = g.players[0].life;
    g.perform_action(GameAction::PlayLand(id)).expect("land drop is legal");
    assert!(g.stack.is_empty(), "nothing reaches the stack");
    assert!(g.pending_decision.is_some(), "the UI seat is asked");
    g.submit_decision(DecisionAnswer::Mode(1)).expect("decline the payment");
    let land = g.battlefield_find(id).expect("still on the battlefield");
    assert!(land.tapped, "declined, so it entered tapped");
    assert_eq!(g.players[0].life, before, "and no life was paid");
    assert!(g.pending_decision.is_none(), "the entry finished on the answer");
}

// ── The "other" bucket's convertible members (CR 614.12) ───────────────────

/// Eight more as-enters clauses that were ETB triggers, each stamping a value
/// the permanent's own static ability then reads: the chosen opponent (both
/// Racks), a basic land type (Realmwright, Roots of Life), two colours
/// (Tablet of the Guilds), a number (Sanctum Prelate), a creature to protect
/// (Dauntless Bodyguard) and a parity (Lavabrink Venturer). Asserted the same
/// way as the other classes — stamped by the time the entry returns, with
/// nothing on the stack to carry it.
#[test]
fn cr_614_12a_the_remaining_as_enters_stamps_happen_inside_the_entry() {
    let mut g = two_player_game();
    // Dauntless Bodyguard needs another creature to name.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let stamped: [Stamped; 7] = [
        ("Cursed Rack", catalog::cursed_rack, |c| c.chosen_player.is_some()),
        ("The Rack", catalog::the_rack, |c| c.chosen_player.is_some()),
        ("Realmwright", catalog::realmwright, |c| c.chosen_land_type.is_some()),
        ("Roots of Life", catalog::roots_of_life, |c| c.chosen_land_type.is_some()),
        ("Tablet of the Guilds", catalog::tablet_of_the_guilds, |c| {
            c.chosen_colors.len() == 2
        }),
        ("Sanctum Prelate", catalog::sanctum_prelate, |c| c.chosen_number.is_some()),
        ("Dauntless Bodyguard", catalog::dauntless_bodyguard, |c| {
            c.chosen_permanent.is_some()
        }),
    ];
    for (name, factory, stamped_ok) in stamped {
        let id = g.move_card_to_battlefield_for_test(0, factory());
        let inst = g.battlefield_find(id).unwrap_or_else(|| panic!("{name} entered"));
        assert!(stamped_ok(inst), "CR 614.12a — {name} stamps its choice as it enters");
        assert!(g.stack.is_empty(), "CR 614.12 — {name} leaves no trigger behind");
    }
}

/// Lavabrink Venturer's "choose odd or even" is the one that needed the new
/// primitive. `Effect::ChooseMode` reads `ctx.mode` — the pick made when the
/// spell went on the stack — and an as-enters replacement has no stack item
/// of its own, so it silently took mode 0 (even) and the decider was never
/// consulted. `AsEntersChooseMode` asks, so a scripted mode 1 sticks.
///
/// ⚠ The ask goes through the decider here rather than suspending: only the
/// LAND DROP has a resume context for an as-enters question
/// (`ResumeContext::LandEntry`). The cast, move and mint paths still drive
/// the ask through `resolve_effect_driven`, which is the standing behaviour
/// for all forty-odd `as_enters_effect` cards and is recorded as such.
#[test]
fn cr_614_12a_lavabrink_venturer_asks_for_its_parity() {
    use crabomination::card::Keyword;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lavabrink_venturer());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    cast(&mut g, id);
    let cp = g.computed_permanent(id).expect("the Venturer entered");
    assert!(
        cp.keywords()
            .iter()
            .any(|k| matches!(k, Keyword::ProtectionFromManaValueParity { odd: true })),
        "CR 614.12a — the chosen parity is the one that stuck, not a silent mode 0"
    );
}

/// And the default is unchanged: no script, mode 0, which is "even".
#[test]
fn cr_614_12a_lavabrink_venturers_default_parity_is_mode_zero() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lavabrink_venturer());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    cast(&mut g, id);
    let cp = g.computed_permanent(id).expect("the Venturer entered");
    assert!(
        cp.keywords()
            .iter()
            .any(|k| matches!(k, Keyword::ProtectionFromManaValueParity { odd: false })),
    );
}

/// Crowd-Control Warden is the census's last straightforward row: "As this
/// creature enters **or is turned face up**, put X +1/+1 counters on it."
/// The enters half is the replacement; the counters are on it before the
/// first SBA sweep, with nothing on the stack. (The turned-face-up half
/// stays a trigger — the engine has no as-turns-face-up applier.)
#[test]
fn cr_614_12_crowd_control_warden_counts_as_it_enters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    let id = g.move_card_to_battlefield_for_test(0, catalog::crowd_control_warden());
    let inst = g.battlefield_find(id).expect("the Warden entered");
    assert_eq!(
        inst.counter_count(CounterType::PlusOnePlusOne),
        3,
        "CR 614.12 — one counter per other creature, counted as it enters"
    );
    assert!(g.stack.is_empty(), "no trigger carries it");
}
