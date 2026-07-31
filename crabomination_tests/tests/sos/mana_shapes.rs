#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use crate::prepared_on_battlefield;

// ── Hybrid mana: auto-tap produces a payable color ──────────────────────────

#[test]
fn auto_tap_pays_hybrid_pair_from_one_of_each_color_land() {
    use crabomination::mana::{cost, hybrid};
    // {W/B}{W/B} with a Plains + a Swamp: auto-tap must split the lands
    // (one W, one B) rather than hunting for two of the same color.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::plains());
    g.add_card_to_battlefield(0, catalog::swamp());
    let hcost = cost(&[
        hybrid(Color::White, Color::Black),
        hybrid(Color::White, Color::Black),
    ]);
    g.auto_tap_for_cost(0, &hcost);
    assert!(
        g.players[0].mana_pool.clone().pay(&hcost).is_ok(),
        "auto-tap should produce mana that pays {{W/B}}{{W/B}} from a Plains + Swamp"
    );
}

#[test]
fn auto_tap_pays_hybrid_from_only_off_color_land() {
    use crabomination::mana::{cost, hybrid};
    // {W/B} with only a Swamp: the engine must tap the Swamp for black,
    // not always reach for the first color (white) and strand the cast.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::swamp());
    let hcost = cost(&[hybrid(Color::White, Color::Black)]);
    g.auto_tap_for_cost(0, &hcost);
    assert_eq!(
        g.players[0].mana_pool.amount(Color::Black), 1,
        "auto-tap should have tapped the Swamp for black"
    );
    assert!(
        g.players[0].mana_pool.clone().pay(&hcost).is_ok(),
        "the tapped black mana pays the {{W/B}} pip"
    );
}

#[test]
fn auto_tap_hybrid_card_casts_with_off_color_lands() {
    use crabomination::mana::ManaSymbol;
    // End-to-end: Manamorphose is {1}{R/G}. With only two Forests, the
    // {R/G} pip must be paid by the *green* (second) half — exactly the
    // case the old "always try the first color" auto-tap stranded. The
    // cast should succeed and the spell resolve to the graveyard.
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    // Library padding so the spell's "draw a card" can't deck-out.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::manamorphose());
    let def = catalog::manamorphose();
    assert!(
        def.cost.symbols.iter().any(|s| matches!(s, ManaSymbol::Hybrid(_, _))),
        "test fixture must have a two-color hybrid pip"
    );
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("{1}{R/G} should be castable off two Forests (green pays the hybrid pip)");
    drain_stack(&mut g);
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == id),
        "Manamorphose should resolve to the graveyard"
    );
}

// ── Manual mana tapping (UI players) ────────────────────────────────────────
//
// For a UI player (`wants_ui = true`), the engine auto-taps the *forced*
// parts of a cost (a colour only one kind of source can make, or
// interchangeable basics) and only requires a manual tap when there's a
// genuine choice of which sources to tap (leaving different mana behind).
// Bots (`wants_ui = false`) keep full auto-tap.

#[test]
fn ui_player_auto_taps_forced_green_with_spare_mountains() {
    // The reported case: a {4}{G}{G} card (Craw Wurm) where the only green
    // sources are Forests, plus spare Mountains for the generic. The two
    // green pips are forced onto the Forests, and the leftover Mountains
    // are interchangeable, so the whole cost auto-taps and the cast goes
    // through — no manual prompt.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    let f2 = g.add_card_to_battlefield(0, catalog::forest());
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let id = g.add_card_to_hand(0, catalog::craw_wurm());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("{4}{G}{G} should auto-tap the only green sources (Forests) + Mountains");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id), "Craw Wurm resolves");
    assert!(
        g.battlefield.iter().find(|c| c.id == f1).unwrap().tapped
            && g.battlefield.iter().find(|c| c.id == f2).unwrap().tapped,
        "both Forests auto-tapped for the forced green pips"
    );
}

#[test]
fn ui_player_must_tap_when_generic_has_a_color_choice() {
    // {4}{G}{G} with 2 Forests (green) + 4 Mountains + 1 Island. Green is
    // forced onto the Forests (auto-tapped, {G}{G} floats), but the generic
    // {4} can hold back either a Mountain or the Island — a real choice — so
    // the cast stops for a manual tap of the generic part.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    // CR 601.2g proper tapping is `manual_mana`, not `wants_ui`: this test
    // models a human choosing their own sources. A bot seat wants its
    // decisions surfaced but still auto-taps.
    g.players[0].manual_mana = true;
    let mut lands = Vec::new();
    lands.push(g.add_card_to_battlefield(0, catalog::forest()));
    lands.push(g.add_card_to_battlefield(0, catalog::forest()));
    for _ in 0..4 {
        lands.push(g.add_card_to_battlefield(0, catalog::mountain()));
    }
    lands.push(g.add_card_to_battlefield(0, catalog::island()));
    let id = g.add_card_to_hand(0, catalog::craw_wurm());

    let result = g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(
        matches!(result, Err(GameError::ManualTapRequired { .. })),
        "a real generic color-choice must require a manual tap, got {result:?}"
    );
    // Partial: spell returns to hand, but the forced green auto-tapped —
    // the two Forests are tapped with {G}{G} floating; the ambiguous
    // generic lands stay untapped for the player to choose.
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "spell returns to hand");
    let forests = [lands[0], lands[1]];
    assert!(
        forests.iter().all(|fid| g.battlefield.iter().find(|c| c.id == *fid).unwrap().tapped),
        "the two Forests (only green sources) auto-tapped for the forced {{G}}{{G}}"
    );
    assert!(
        lands[2..].iter().all(|lid| !g.battlefield.iter().find(|c| c.id == *lid).unwrap().tapped),
        "the ambiguous generic lands (Mountains + Island) stay untapped"
    );
    assert_eq!(
        g.players[0].mana_pool.amount(Color::Green), 2,
        "{{G}}{{G}} floats in the pool, ready once the generic is tapped"
    );
}

#[test]
fn ui_player_partial_tap_then_manual_generic_completes_cast() {
    // Full flow: forced green auto-taps the Forests, then the player taps
    // the Mountains they choose for the generic and the re-submitted cast
    // (what the client's pending-cast driver does) goes through.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    // CR 601.2g proper tapping is `manual_mana`, not `wants_ui`: this test
    // models a human choosing their own sources. A bot seat wants its
    // decisions surfaced but still auto-taps.
    g.players[0].manual_mana = true;
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    let f2 = g.add_card_to_battlefield(0, catalog::forest());
    let mut mountains = Vec::new();
    for _ in 0..4 {
        mountains.push(g.add_card_to_battlefield(0, catalog::mountain()));
    }
    let island = g.add_card_to_battlefield(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::craw_wurm());

    let r = g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(r, Err(GameError::ManualTapRequired { .. })));
    assert!(g.battlefield.iter().find(|c| c.id == f1).unwrap().tapped);
    assert!(g.battlefield.iter().find(|c| c.id == f2).unwrap().tapped);

    // Player taps the 4 Mountains for the generic (holding the Island).
    for m in &mountains {
        g.perform_action(GameAction::ActivateAbility {
            card_id: *m, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        })
        .expect("tap a Mountain for mana");
    }
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("green floating + 4 Mountains tapped pays {4}{G}{G}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id), "Craw Wurm resolves");
    assert!(
        !g.battlefield.iter().find(|c| c.id == island).unwrap().tapped,
        "the Island the player chose to hold stays untapped"
    );
}

#[test]
fn ui_player_auto_taps_when_no_choice_remains() {
    // Exactly 2 Forests for a {1}{G} cost: the player must tap both, so
    // there's no choice — the engine auto-taps and the cast succeeds.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let mut forests = Vec::new();
    for _ in 0..2 {
        forests.push(g.add_card_to_battlefield(0, catalog::forest()));
    }
    let id = g.add_card_to_hand(0, catalog::grizzly_bears());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("no spare lands → engine auto-taps and the cast succeeds");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id), "Grizzly Bears resolves");
    assert!(
        forests.iter().all(|fid| g.battlefield.iter().find(|c| c.id == *fid).unwrap().tapped),
        "both Forests tapped"
    );
}

#[test]
fn ui_player_casts_from_prefilled_pool_ignoring_spare_lands() {
    // The player has already arranged their mana (pool covers the cost),
    // so the cast goes through the fast path even with spare untapped
    // lands on the board — those stay untapped.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let mut forests = Vec::new();
    for _ in 0..3 {
        forests.push(g.add_card_to_battlefield(0, catalog::forest()));
    }
    g.players[0].mana_pool.add(Color::Green, 2); // covers {1}{G}
    let id = g.add_card_to_hand(0, catalog::grizzly_bears());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("pool already covers the cost → no manual tap needed");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id), "Grizzly Bears resolves");
    assert!(
        forests.iter().all(|fid| !g.battlefield.iter().find(|c| c.id == *fid).unwrap().tapped),
        "lands stay untapped — the spell was paid from the pool"
    );
}

#[test]
fn ui_player_irrelevant_spare_source_does_not_force_manual_tap() {
    // {G} (Llanowar Elves) with a Forest + an Island. Only the Forest can
    // pay {G}; the leftover Island is irrelevant to this cost, so there's
    // no real choice — the engine auto-taps the Forest and casts.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let island = g.add_card_to_battlefield(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::llanowar_elves());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("an off-color spare land is not a real choice for a mono-color cost");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id), "Llanowar Elves resolves");
    assert!(g.battlefield.iter().find(|c| c.id == forest).unwrap().tapped, "Forest tapped for {{G}}");
    assert!(!g.battlefield.iter().find(|c| c.id == island).unwrap().tapped, "Island left untapped");
}

#[test]
fn bot_still_auto_taps_with_spare_lands() {
    // Regression: a non-UI payer (bot / default) keeps full auto-tap even
    // when spare lands exist — the manual-tap gate is UI-only.
    let mut g = two_player_game();
    assert!(!g.players[0].wants_ui, "default player is non-UI");
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("non-UI payer auto-taps regardless of spare lands");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id), "Grizzly Bears resolves for the bot");
}

// ── Mica, Reader of Ruins ─────────────────────────────────────────────────

// ── The Dawning Archaic ───────────────────────────────────────────────────

// ── Strixhaven Skycoach ───────────────────────────────────────────────────

// ── Biblioplex Tomekeeper ─────────────────────────────────────────────────

// ── Skycoach Waypoint ─────────────────────────────────────────────────────

// ── Prismari, the Inspiration ─────────────────────────────────────────────

// ── Social Snub ───────────────────────────────────────────────────────────

#[test]
fn social_snub_each_player_sacrifices_and_drains() {
    let mut g = two_player_game();
    // Give both players a creature.
    let p0_creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p1_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::social_snub());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Social Snub castable for {1}{W}{B}");
    drain_stack(&mut g);

    // Both creatures should have been sacrificed.
    assert!(!g.battlefield.iter().any(|c| c.id == p0_creature),
        "P0's creature should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == p1_creature),
        "P1's creature should be sacrificed");
    // Opponent loses 1 life, you gain 1 life.
    assert_eq!(g.players[1].life, p1_life - 1,
        "Opponent should lose 1 life");
    assert_eq!(g.players[0].life, p0_life + 1,
        "Caster should gain 1 life");
}

// ── Strife Scholar preparation card ─────────────────────────────────────

#[test]
fn awaken_the_ages_spirits_are_two_two() {
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::strife_scholar());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Awaken the Ages castable for {5}{R}");
    drain_stack(&mut g);

    let spirit = g.battlefield.iter().find(|c| c.definition.name == "Spirit")
        .expect("Spirit token minted");
    assert_eq!(spirit.definition.power, 2, "Spirits are 2/2");
    assert_eq!(spirit.definition.toughness, 2);
}

// ── Colorstorm Stallion ─────────────────────────────────────────────────

#[test]
fn colorstorm_stallion_magecraft_pump() {
    let mut g = two_player_game();
    let stallion = g.add_card_to_battlefield(0, catalog::colorstorm_stallion());
    g.clear_sickness(stallion);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);

    let s = g.computed_permanent(stallion).unwrap();
    assert_eq!(s.power, 4, "Stallion should be 4/4 after magecraft +1/+1");
    assert_eq!(s.toughness, 4);
}

// ── Elemental Mascot ─────────────────────────────────────────────────

#[test]
fn elemental_mascot_magecraft_pump() {
    let mut g = two_player_game();
    let mascot = g.add_card_to_battlefield(0, catalog::elemental_mascot());
    g.clear_sickness(mascot);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);

    let m = g.computed_permanent(mascot).unwrap();
    assert_eq!(m.power, 2, "Mascot should be 2/4 after magecraft +1/+0");
    assert_eq!(m.toughness, 4);
}

// ── Molten Note ─────────────────────────────────────────────────────────

#[test]
fn molten_note_deals_x_plus_two_damage_and_untaps() {
    let mut g = two_player_game();
    // 5/5 creature on opp side.
    let big = g.add_card_to_battlefield(1, catalog::beledros_witherbloom());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    // Tap the attacker.
    g.battlefield.iter_mut().find(|c| c.id == attacker).unwrap().tapped = true;

    let id = g.add_card_to_hand(0, catalog::molten_note());
    // X=4 → total damage = 4+2 = 6.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(4),
    }).expect("Molten Note castable for {X=4}{R}{W}");
    drain_stack(&mut g);

    // 6 damage to a 6/6 kills it.
    assert!(!g.battlefield.iter().any(|c| c.id == big),
        "6/6 should be killed by 6 damage");
    // Our creature should be untapped.
    assert!(!g.battlefield.iter().find(|c| c.id == attacker).unwrap().tapped,
        "Our creature should be untapped");
}

// ── Social Snub ─────────────────────────────────────────────────────────

#[test]
fn social_snub_each_player_sacs_and_drains() {
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::social_snub());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Social Snub castable for {1}{W}{B}");
    drain_stack(&mut g);

    // Each player should have sacrificed one creature.
    let p0_creatures = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count();
    let p1_creatures = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_creature()).count();
    assert_eq!(p0_creatures, 0, "P0 should have sacrificed their creature");
    assert_eq!(p1_creatures, 0, "P1 should have sacrificed their creature");
    // Drain 1: opp loses 1, you gain 1.
    assert_eq!(g.players[1].life, life1_before - 1);
    assert_eq!(g.players[0].life, life0_before + 1);
}

// ── Fix What's Broken ─────────────────────────────────────────────────

#[test]
fn fix_whats_broken_returns_creatures_from_gy() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let id = g.add_card_to_hand(0, catalog::fix_whats_broken());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4); // 2 generic + 2 for X
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("Fix What's Broken castable (X=2)");
    drain_stack(&mut g);

    // Bear (MV 2) should be on battlefield (matches X=2).
    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be returned from graveyard to battlefield");
    // X (=2) life lost.
    assert_eq!(g.players[0].life, life_before - 2);
}

// ── Biblioplex Tomekeeper ─────────────────────────────────────────────

#[test]
fn biblioplex_tomekeeper_enters_as_3_4_construct() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::biblioplex_tomekeeper());
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Biblioplex Tomekeeper castable for {4}");
    drain_stack(&mut g);

    let perm = g.battlefield.iter().find(|c| c.id == id)
        .expect("Tomekeeper should be on battlefield");
    assert!(perm.definition.card_types.contains(&CardType::Artifact));
    assert!(perm.definition.card_types.contains(&CardType::Creature));
    let view = g.computed_permanent(id).unwrap();
    assert_eq!(view.power, 3);
    assert_eq!(view.toughness, 4);
}

// ── Strixhaven Skycoach ───────────────────────────────────────────────

#[test]
fn strixhaven_skycoach_etb_searches_for_basic_land() {
    let mut g = two_player_game();
    // Seed library with a basic land to find.
    let forest = g.add_card_to_library(0, catalog::forest());

    let id = g.add_card_to_hand(0, catalog::strixhaven_skycoach());
    g.players[0].mana_pool.add_colorless(3);

    // Script the decider to pick the Forest from the search.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
    ]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Strixhaven Skycoach castable for {3}");
    drain_stack(&mut g);

    // Skycoach on battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Skycoach should be on battlefield");
    let view = g.computed_permanent(id).unwrap();
    assert!(view.keywords.contains(&Keyword::Flying),
        "Skycoach should have flying");

    // Forest should be in hand (searched from library).
    assert!(g.players[0].hand.iter().any(|c| c.id == forest),
        "Forest should have been searched into hand");
}

// ── The Dawning Archaic ───────────────────────────────────────────────

#[test]
fn the_dawning_archaic_enters_as_7_7_reach() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::the_dawning_archaic());
    g.players[0].mana_pool.add_colorless(10);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("The Dawning Archaic castable for {10}");
    drain_stack(&mut g);

    let perm = g.battlefield.iter().find(|c| c.id == id)
        .expect("The Dawning Archaic should be on battlefield");
    assert!(perm.definition.card_types.contains(&CardType::Creature));
    let view = g.computed_permanent(id).unwrap();
    assert_eq!(view.power, 7);
    assert_eq!(view.toughness, 7);
    assert!(view.keywords.contains(&Keyword::Reach),
        "The Dawning Archaic should have reach");
}

// ── Prismari, the Inspiration ────────────────────────────────────────

#[test]
fn prismari_the_inspiration_enters_as_7_7_flying() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_the_inspiration());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Prismari castable for {5}{U}{R}");
    drain_stack(&mut g);

    let perm = g.battlefield.iter().find(|c| c.id == id)
        .expect("Prismari should be on battlefield");
    assert!(perm.definition.card_types.contains(&CardType::Creature));
    assert!(perm.definition.has_creature_type(crabomination::card::CreatureType::Elder));
    assert!(perm.definition.has_creature_type(crabomination::card::CreatureType::Dragon));
    let view = g.computed_permanent(id).unwrap();
    assert_eq!(view.power, 7);
    assert_eq!(view.toughness, 7);
    assert!(view.keywords.contains(&Keyword::Flying),
        "Prismari should have flying");
}

// ── Nita, Forum Conciliator ──────────────────────────────────────────

#[test]
fn nita_forum_conciliator_enters_as_2_3() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::nita_forum_conciliator());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Nita castable for {1}{W}{B}");
    drain_stack(&mut g);

    let perm = g.battlefield.iter().find(|c| c.id == id)
        .expect("Nita should be on battlefield");
    assert!(perm.definition.card_types.contains(&CardType::Creature));
    assert!(perm.definition.has_creature_type(crabomination::card::CreatureType::Human));
    assert!(perm.definition.has_creature_type(crabomination::card::CreatureType::Advisor));
    let view = g.computed_permanent(id).unwrap();
    assert_eq!(view.power, 2);
    assert_eq!(view.toughness, 3);
}

// ── Silverquill, the Disputant ───────────────────────────────────────

#[test]
fn silverquill_the_disputant_enters_as_4_4_flying_vigilance() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_the_disputant());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Silverquill castable for {2}{W}{B}");
    drain_stack(&mut g);

    let perm = g.battlefield.iter().find(|c| c.id == id)
        .expect("Silverquill should be on battlefield");
    assert!(perm.definition.card_types.contains(&CardType::Creature));
    assert!(perm.definition.has_creature_type(crabomination::card::CreatureType::Elder));
    assert!(perm.definition.has_creature_type(crabomination::card::CreatureType::Dragon));
    let view = g.computed_permanent(id).unwrap();
    assert_eq!(view.power, 4);
    assert_eq!(view.toughness, 4);
    assert!(view.keywords.contains(&Keyword::Flying),
        "Silverquill should have flying");
    assert!(view.keywords.contains(&Keyword::Vigilance),
        "Silverquill should have vigilance");
}

// ── Quandrix, the Proof ──────────────────────────────────────────────

#[test]
fn quandrix_the_proof_enters_as_6_6_flying_trample() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_the_proof());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Quandrix castable for {4}{G}{U}");
    drain_stack(&mut g);

    let perm = g.battlefield.iter().find(|c| c.id == id)
        .expect("Quandrix should be on battlefield");
    assert!(perm.definition.card_types.contains(&CardType::Creature));
    assert!(perm.definition.has_creature_type(crabomination::card::CreatureType::Elder));
    assert!(perm.definition.has_creature_type(crabomination::card::CreatureType::Dragon));
    let view = g.computed_permanent(id).unwrap();
    assert_eq!(view.power, 6);
    assert_eq!(view.toughness, 6);
    assert!(view.keywords.contains(&Keyword::Flying),
        "Quandrix should have flying");
    assert!(view.keywords.contains(&Keyword::Trample),
        "Quandrix should have trample");
}

// (Applied Geometry's copy behavior is covered by
// `applied_geometry_mints_a_six_six_fractal` and
// `applied_geometry_copies_creature_as_six_six_fractal`. The old
// no-target "vanilla Fractal" test was removed when the card was
// promoted to the real `CreateTokenCopyOf` primitive.)

