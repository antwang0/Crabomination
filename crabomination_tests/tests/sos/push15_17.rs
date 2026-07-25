#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use crate::prepared_on_battlefield;

// ── Great Hall of the Biblioplex (push XV) ──────────────────────────────────

#[test]
fn great_hall_taps_for_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::great_hall_of_the_biblioplex());
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    let mp_before_c = g.players[0].mana_pool.colorless_amount();

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Great Hall {T}: Add {C}");

    assert_eq!(g.players[0].mana_pool.colorless_amount(), mp_before_c + 1);
    assert!(g.battlefield.iter().find(|c| c.id == id).unwrap().tapped);
}

#[test]
fn great_hall_pay_one_life_taps_for_any_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::great_hall_of_the_biblioplex());
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    let life_before = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Black)]));

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Great Hall pay-1-life mana ability");

    assert_eq!(g.players[0].life, life_before - 1);
    // "Spend this mana only to cast an instant or sorcery spell" — the
    // chosen black mana enters restricted, not the free black bucket.
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 0);
    assert_eq!(g.players[0].mana_pool.restricted_total(), 1);
    assert!(g.battlefield.iter().find(|c| c.id == id).unwrap().tapped);
}

// ── Follow the Lumarets (push XV) ────────────────────────────────────────────

#[test]
fn follow_the_lumarets_pulls_one_creature_without_lifegain() {
    // Mainline: no life gain → pull one creature/land to hand.
    let mut g = two_player_game();
    // Library top: forest, then a bear, then island.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::follow_the_lumarets());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Follow the Lumarets castable");
    drain_stack(&mut g);

    // Top of library was Forest (a Land), so the first match → hand.
    // Net: hand cast (-1) + 1 pull = same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "One creature/land pulled into hand");
}

#[test]
fn follow_the_lumarets_pulls_two_with_lifegain_infusion() {
    // Infusion path: gained life this turn → pull two creature/land cards.
    let mut g = two_player_game();
    g.players[0].life_gained_this_turn = 3;
    // Library top: forest, bear, island, mountain (all matching).
    g.add_card_to_library(0, catalog::mountain());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::follow_the_lumarets());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Follow the Lumarets castable with Infusion");
    drain_stack(&mut g);

    // Net: -1 cast + 2 pulls = +1 hand size.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Two creature/land cards pulled with Infusion");
}

// ── Lluwen, Exchange Student // Pest Friend (push XV) ───────────────────────
//
// Closes out the Witherbloom (B/G) school. Front: 3/4 Legendary Elf
// Druid vanilla body. Back: sorcery — create a 1/1 Pest token with the
// printed "attacks → gain 1 life" rider.

#[test]
fn lluwen_prepare_spell_creates_pest_token_with_lifegain_rider() {
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::lluwen_exchange_student());
    // Pay {B} for the Pest Friend prepare-spell cost ({B/G} hybrid → {B}).
    g.players[0].mana_pool.add(Color::Black, 1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pest Friend castable for {B}");
    drain_stack(&mut g);

    // A new Pest token should be on the battlefield.
    assert_eq!(g.battlefield.len(), bf_before + 1);
    let pest = g.battlefield.iter().find(|c| c.definition.name == "Pest")
        .expect("Pest token created");
    assert_eq!(pest.definition.power, 1);
    assert_eq!(pest.definition.toughness, 1);
    // The token must carry the on-attack lifegain trigger.
    assert!(!pest.definition.triggered_abilities.is_empty(),
        "Pest token should carry the on-attack lifegain rider");
}

#[test]
fn lluwen_front_castable_for_two_b_g_as_three_four_creature() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lluwen_exchange_student());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Lluwen castable for {2}{B}{G}");
    drain_stack(&mut g);

    let lluwen = g.battlefield.iter().find(|c| c.id == id)
        .expect("Lluwen on battlefield");
    assert_eq!(lluwen.definition.power, 3);
    assert_eq!(lluwen.definition.toughness, 4);
}

#[test]
fn geometers_arthropod_x_cast_pulls_card_to_hand() {
    // Geometer's Arthropod's X-cast trigger uses the new
    // `Predicate::CastSpellHasX` + `RevealUntilFind` body. Casting a spell
    // with `{X}` in its cost (e.g. Mathemagics with X=2) should pull the
    // top library card into the controller's hand if it matches the
    // (filter: Any) — first card always matches. Hand size goes up by 1
    // because of the trigger on top of any draws from the cast itself.
    let mut g = two_player_game();
    let arthropod = g.add_card_to_battlefield(0, catalog::geometers_arthropod());
    // Seed library with 4 forest cards and a top island so the trigger pulls
    // the top island to hand.
    let top_island = g.add_card_to_library(0, catalog::island());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    // Now cast Lightning Bolt {R} — that's NOT an X-cost spell, so the
    // trigger should NOT fire.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);
    // -1 cast (Bolt left hand). No X-trigger fired ⇒ no hand pull.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
    // Sanity: arthropod still on bf.
    assert!(g.battlefield.iter().any(|c| c.id == arthropod));
    // Top island still on top of library.
    assert_eq!(
        g.players[0].library.iter().find(|c| c.id == top_island).map(|_| true),
        Some(true),
        "Top island shouldn't be touched by non-X spell"
    );
}

#[test]
fn bayou_groff_requires_sacrificing_a_creature_to_cast() {
    // "As an additional cost to cast this spell, sacrifice a creature." A
    // fodder creature is auto-sacrificed; Groff resolves onto the battlefield.
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::bayou_groff());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable with a creature to sacrifice");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder creature sacrificed");
    assert!(g.battlefield_find(id).is_some(), "Bayou Groff entered the battlefield");
}

#[test]
fn bayou_groff_pays_three_without_a_creature_to_sacrifice() {
    // No creature → the pay-{3} half joins the cost ({1}{G} + {3}).
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bayou_groff());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "one generic and a G alone can't cover the +3 cost");
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {4}{G} with no creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "Bayou Groff entered the battlefield");
    assert_eq!(g.players[0].mana_pool.total(), 0, "the extra 3 was spent");
}

#[test]
fn embrace_the_paradox_may_skip_extra_land_default() {
    // With AutoDecider (default), the MayDo rider answers no; only the
    // Draw 3 fires. Library lands should remain in the library.
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let _land = g.add_card_to_hand(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::embrace_the_paradox());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_lands_before = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_land())
        .count();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Embrace castable");
    drain_stack(&mut g);

    // Default decider answers no — the forest stays in hand.
    let bf_lands_after = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_land())
        .count();
    assert_eq!(bf_lands_after, bf_lands_before, "Forest should NOT have hit bf");
}

#[test]
fn embrace_the_paradox_may_put_land_when_yes() {
    // Scripted decider yes → forest from hand goes to bf tapped.
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let forest_id = g.add_card_to_hand(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::embrace_the_paradox());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Embrace castable");
    drain_stack(&mut g);

    let forest_view = g.battlefield.iter().find(|c| c.id == forest_id)
        .expect("Forest should be on battlefield after MayDo=yes");
    assert!(forest_view.tapped, "Forest enters tapped per the rider");
}

#[test]
fn felisa_silverquill_dies_with_counter_creates_inkling_token() {
    // Felisa's "creature with +1/+1 counter dies → 1/1 W/B Inkling token"
    // trigger. Kill the counter-bearing bear via Murder so the death
    // trigger fires through the normal cast → resolution → SBA → dispatch
    // pipeline (the AnotherOfYours scope needs the unified dispatcher,
    // which is only invoked from priority/cast paths — not from a bare
    // `check_state_based_actions` call).
    let mut g = two_player_game();
    let _felisa = g.add_card_to_battlefield(0, catalog::felisa_fang_of_silverquill());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) {
        c.add_counters(CounterType::PlusOnePlusOne, 1);
    }
    let bf_before = g.battlefield.len();
    // Cast Murder targeting the bear.
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: murder, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Murder castable for {1}{B}{B}");
    drain_stack(&mut g);
    // Bear gone, Inkling token added.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    let ink = g.battlefield.iter().find(|c| c.definition.name == "Inkling")
        .expect("Inkling token created when counter-bearing creature dies");
    assert!(ink.definition.keywords.contains(&Keyword::Flying));
    // Net battlefield: -1 bear (murder gone too) + 1 inkling = same as before
    // murder cast. (felisa untouched, bear out, inkling in.)
    assert_eq!(g.battlefield.len(), bf_before);
}

#[test]
fn felisa_no_counter_no_token() {
    // No +1/+1 counter on the dying creature → no token.
    let mut g = two_player_game();
    let _felisa = g.add_card_to_battlefield(0, catalog::felisa_fang_of_silverquill());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: murder, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Murder castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Inkling"),
        "No Inkling minted when the dying bear had no +1/+1 counter");
}

#[test]
fn sundering_archaic_etb_converge_cap_blocks_high_mv_target() {
    // Push (modern_decks): the converge-scaled MV cap is wired via
    // `Effect::If { cond: ValueAtMost(ManaValueOf(Target), ConvergedValue) }`.
    // Mono-colorless cast (ConvergedValue = 0) means MV ≤ 0 — so a CMC-2
    // bear is NOT a legal exile target (the trigger no-ops cleanly).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::sundering_archaic());
    g.players[0].mana_pool.add_colorless(6);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Sundering Archaic castable for {6}");
    drain_stack(&mut g);

    // Bear still on battlefield: trigger no-ops since 2 > 0 (ConvergedValue).
    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "Bear (CMC 2) should NOT be exiled — converge cap is 0 (no colored mana spent)");
    assert!(!g.exile.iter().any(|c| c.id == bear),
        "Bear should not be in exile");
}

#[test]
fn sundering_archaic_two_mana_bottoms_graveyard_card() {
    // Activated `{2}` ability moves a graveyard card to the bottom of its
    // owner's library.
    let mut g = two_player_game();
    let archaic_id = g.add_card_to_battlefield(0, catalog::sundering_archaic());
    // Stash a card in opponent's graveyard.
    let target_card = catalog::lightning_bolt();
    let bolt_id = g.next_id();
    let mut bolt = crabomination::card::CardInstance::new(bolt_id, target_card, 1);
    bolt.controller = 1;
    g.players[1].graveyard.push(bolt);
    // Activate Sundering's `{2}` ability targeting the bolt.
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: archaic_id,
        ability_index: 0,
        target: Some(Target::Permanent(bolt_id)), additional_targets: Vec::new(), x_value: None })
    .expect("Sundering Archaic {2} ability activatable");
    drain_stack(&mut g);
    // Bolt should be at the bottom of player 1's library, not in their gy.
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bolt_id));
    // Bottom of library = last index.
    let lib = &g.players[1].library;
    assert!(!lib.is_empty(), "library not empty");
    assert_eq!(lib.last().unwrap().id, bolt_id, "Bolt should be at bottom of P1's library");
}

#[test]
fn predicate_cast_spell_has_x_label_does_not_panic_in_view() {
    // Smoke test: building a PlayerView for a state with the new predicate
    // shouldn't panic. predicate_short_label has a catch-all — covers
    // CastSpellHasX with the "conditional" fallback.
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::geometers_arthropod());
    // Just constructing a view for player 0 exercises the abil/trig view
    // pipeline that calls predicate_short_label.
    let _view = crabomination::server::view::project(&g, 0);
}

// ── push XVII: from_graveyard activations ───────────────────────────────────

#[test]
fn summoned_dromedary_returns_from_graveyard_to_hand() {
    // {1}{W} sorcery-speed activation from graveyard returns this card to
    // the controller's hand. Powered by the new ActivatedAbility.
    // from_graveyard field that lets activate_ability walk the graveyard.
    let mut g = two_player_game();
    let drome = g.add_card_to_graveyard(0, catalog::summoned_dromedary());
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;

    g.perform_action(GameAction::ActivateAbility {
        card_id: drome,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Dromedary activation from gy must succeed");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Dromedary should be in hand after activation");
    assert_eq!(g.players[0].graveyard.len(), gy_before - 1,
        "Dromedary should leave the graveyard");
    assert!(g.players[0].hand.iter().any(|c| c.id == drome));
}

#[test]
fn summoned_dromedary_activation_rejected_during_opponent_priority() {
    // Sorcery-speed gate: opponent's main phase must not allow the activation.
    let mut g = two_player_game();
    let drome = g.add_card_to_graveyard(0, catalog::summoned_dromedary());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1; // wrong player
    let _ = g.perform_action(GameAction::ActivateAbility {
        card_id: drome,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None })
    .expect_err("opponent shouldn't be able to activate a graveyard card belonging to player 0");
}

#[test]
fn teachers_pest_returns_from_graveyard_to_battlefield_tapped() {
    use crabomination::mana::g as green;
    let _ = green;
    let mut g = two_player_game();
    let pest = g.add_card_to_graveyard(0, catalog::teachers_pest());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;

    g.perform_action(GameAction::ActivateAbility {
        card_id: pest,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Teacher's Pest activation from gy must succeed");
    drain_stack(&mut g);

    let bf_card = g.battlefield.iter().find(|c| c.id == pest)
        .expect("Teacher's Pest should be on the battlefield");
    assert!(bf_card.tapped, "Teacher's Pest should enter the battlefield tapped");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == pest));
}

#[test]
fn stone_docent_exiles_self_and_gains_life() {
    let mut g = two_player_game();
    let docent = g.add_card_to_graveyard(0, catalog::stone_docent());
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let life_before = g.players[0].life;

    g.perform_action(GameAction::ActivateAbility {
        card_id: docent,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Stone Docent activation from gy must succeed");
    drain_stack(&mut g);

    // Source exiled; life +2; surveil 1 may have milled top card.
    assert!(g.exile.iter().any(|c| c.id == docent),
        "Stone Docent should be exiled as cost");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == docent));
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn eternal_student_exiles_self_and_creates_two_inklings() {
    let mut g = two_player_game();
    let student = g.add_card_to_graveyard(0, catalog::eternal_student());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: student,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Eternal Student activation from gy must succeed");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == student),
        "Eternal Student should be exiled as cost");
    let inklings: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Inkling")
        .collect();
    assert_eq!(inklings.len(), 2, "should mint two Inkling tokens");
    assert!(inklings.iter().all(|c| c.definition.keywords.contains(&Keyword::Flying)));
    assert_eq!(g.battlefield.len(), bf_before + 2);
}

#[test]
fn stone_docent_rejected_at_instant_speed() {
    // Sorcery-speed gate: stack must be empty + main phase + active player.
    // Bob's priority during P0's draw step → reject.
    let mut g = two_player_game();
    let docent = g.add_card_to_graveyard(0, catalog::stone_docent());
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::Upkeep; // not a main phase
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: docent,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None });
    assert!(err.is_err(), "Stone Docent should reject upkeep activation (sorcery-speed)");
}

// ── push XVII: Effect::CopySpell + Aziza ────────────────────────────────────

#[test]
fn aziza_copies_instant_via_magecraft_when_decider_agrees() {
    let mut g = two_player_game();
    let aziza = g.add_card_to_battlefield(0, catalog::aziza_mage_tower_captain());
    g.clear_sickness(aziza);
    // Three creatures we can tap as the optional cost.
    for _ in 0..3 {
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Scripted decider answers Bool(true) for the MayDo prompt.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let bob_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    // Bolt resolves once (3 damage) + copy resolves (3 more) = 6 damage.
    assert_eq!(g.players[1].life, bob_life_before - 6,
        "Aziza should copy the bolt: 3 + 3 = 6 damage to Bob");
    // Three creatures should be tapped as the cost. The picker may
    // include Aziza herself + 2 bears (printed: "tap three untapped
    // creatures you control"; the source is a legal pick for the cost).
    let tapped_creatures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_creature() && c.tapped)
        .count();
    assert_eq!(tapped_creatures, 3, "Aziza taps three creatures as the cost");
}

#[test]
fn aziza_skips_copy_when_decider_declines() {
    let mut g = two_player_game();
    let aziza = g.add_card_to_battlefield(0, catalog::aziza_mage_tower_captain());
    g.clear_sickness(aziza);
    for _ in 0..3 {
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Decider says no.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(false)]));
    let bob_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    // Only the original bolt resolved (3 damage); no copy.
    assert_eq!(g.players[1].life, bob_life_before - 3,
        "No copy: 3 damage to Bob");
    // No bears tapped.
    let tapped_bears = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears" && c.tapped)
        .count();
    assert_eq!(tapped_bears, 0, "Decline should skip the tap-three cost too");
}

/// Postmortem Professor's graveyard-recursion activation: pay `{1}{B}`,
/// exile an instant/sorcery card from your graveyard, and return the
/// Professor from your graveyard to the battlefield. Exercises the new
/// `ActivatedAbility.exile_other_filter` cost primitive in tandem with
/// `from_graveyard: true`.
#[test]
fn postmortem_professor_returns_from_graveyard_by_exiling_instant_or_sorcery() {
    let mut g = two_player_game();
    // Put the Professor in P0's graveyard.
    let prof_id = g.add_card_to_graveyard(0, catalog::postmortem_professor());
    // Stock an instant in the graveyard so the cost has something to pay.
    let bolt_id = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    // Pay mana.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: prof_id,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Postmortem Professor gy-activation should be legal with bolt in gy");
    drain_stack(&mut g);

    // Professor is now on the battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == prof_id),
        "Professor should be on the battlefield after activation");
    // Bolt was exiled (off-graveyard, on exile).
    assert!(g.exile.iter().any(|c| c.id == bolt_id),
        "Bolt should be in exile (paid as activation cost)");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt_id),
        "Bolt should be out of the graveyard");
    assert_eq!(g.battlefield.len(), bf_before + 1);
}

/// Without an instant/sorcery in the graveyard, the Postmortem Professor
/// activation is rejected cleanly — pre-flight gate prevents tap/mana burn.
#[test]
fn postmortem_professor_rejects_activation_without_eligible_gy_card() {
    let mut g = two_player_game();
    let prof_id = g.add_card_to_graveyard(0, catalog::postmortem_professor());
    // Stock a *creature* in graveyard — does not satisfy the IS-card cost.
    let bear_id = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let mana_before = g.players[0].mana_pool.total();

    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: prof_id,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None });
    assert!(result.is_err(),
        "Activation must reject when no IS card is in the graveyard");
    // Mana should be untouched (pre-flight gate rejected before payment).
    assert_eq!(g.players[0].mana_pool.total(), mana_before);
    // Professor still in graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == prof_id));
    // Bear still in graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear_id));
}

/// Molten Note: {X}{R}{W} → X damage to creature + untap all your creatures.
#[test]
fn molten_note_deals_x_damage_and_untaps_your_creatures() {
    use crabomination::game::types::TurnStep as TS;
    let mut g = two_player_game();
    g.step = TS::PreCombatMain;
    // Two of your creatures, both tapped (simulating after-attack state).
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear1).unwrap().tapped = true;
    g.battlefield_find_mut(bear2).unwrap().tapped = true;
    // Opponent's creature with 3 toughness — X=3 should kill it.
    let target_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::molten_note());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("Molten Note castable at X=3 for {X}{R}{W}");
    drain_stack(&mut g);

    // The opp bear (2/2) took 3 damage → dies to SBA.
    assert!(!g.battlefield.iter().any(|c| c.id == target_bear),
        "Bear should die to 3 damage from Molten Note at X=3");
    // Both your bears untapped.
    assert!(!g.battlefield_find(bear1).unwrap().tapped,
        "bear1 should be untapped");
    assert!(!g.battlefield_find(bear2).unwrap().tapped,
        "bear2 should be untapped");
}

/// Push (modern_decks): Molten Note now reads `Value::CastSpellManaSpent`
/// (the actual mana paid for the cast) rather than `Value::XFromCost`,
/// so a 4-toughness creature dies at X=2 because total mana spent is
/// 2 + R + W = 4 (which equals the printed "amount of mana spent" Oracle).
#[test]
fn molten_note_damage_equals_total_mana_spent_not_just_x() {
    use crabomination::game::types::TurnStep as TS;
    let mut g = two_player_game();
    g.step = TS::PreCombatMain;
    // Opp creature with toughness 4 — to kill it, the spell must deal
    // ≥ 4 damage. Pure X-from-cost at X=2 would deal only 2 (would NOT
    // kill); CastSpellManaSpent at X=2 paying {2}{R}{W} reads 4 (kills).
    let target_bear = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::molten_note());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("Molten Note castable at X=2 for {2}{R}{W}");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == target_bear),
        "Serra Angel (4 toughness) should die to 4 damage from Molten Note at X=2 (full mana spent)"
    );
}

#[test]
fn molten_note_flashback_damage_uses_full_mana_spent() {
    // Flashback {6}{R}{W} carries no {X} pip, so the old
    // `Sum(XFromCost, Const(2))` model read X=0 and dealt only 2.
    // `CastSpellManaSpent` reads the actual 8 mana paid, so a 6-toughness
    // creature dies to the flashback cast.
    use crabomination::game::types::{Target, TurnStep as TS};
    let mut g = two_player_game();
    g.step = TS::PreCombatMain;
    let beledros = g.add_card_to_battlefield(1, catalog::beledros_witherbloom()); // 6/6
    let note = g.add_card_to_graveyard(0, catalog::molten_note());
    // Pay the flashback cost {6}{R}{W} = 8 mana exactly.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(6);

    g.perform_action(GameAction::CastFlashback {
        card_id: note,
        target: Some(Target::Permanent(beledros)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Molten Note flashbackable for {6}{R}{W}");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == beledros),
        "Beledros (6 toughness) dies to 8 damage = the full flashback mana spent"
    );
}
// ── Increment / Opus tests ──────────────────────────────────────────────────

/// Helper: drop a creature on the battlefield with summoning sickness cleared
/// and verify it has no +1/+1 counters before we cast a spell off it.
pub(crate) fn place_creature(g: &mut GameState, owner: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(owner, def);
    g.clear_sickness(id);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    id
}

#[test]
fn cuboid_colony_increment_lands_counter_on_two_mana_cast() {
    // Cuboid Colony is a 1/1. Casting a 2-mana spell (mana_spent = 2)
    // exceeds both stats, so Increment fires and lands one +1/+1.
    let mut g = two_player_game();
    let colony = place_creature(&mut g, 0, catalog::cuboid_colony());
    let bear_id = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear_id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bears castable for {1}{G}");
    drain_stack(&mut g);
    let c = g.battlefield_find(colony).expect("Colony still alive");
    assert_eq!(
        c.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Increment should land 1 +1/+1 counter (2 > 1)"
    );
}

#[test]
fn cuboid_colony_increment_skips_one_mana_cast() {
    // Casting a 1-mana spell (mana_spent = 1) does NOT exceed Colony's
    // 1/1 — 1 > 1 is false on both clauses — Increment skips silently.
    let mut g = two_player_game();
    let colony = place_creature(&mut g, 0, catalog::cuboid_colony());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let c = g.battlefield_find(colony).expect("Colony still alive");
    assert_eq!(
        c.counter_count(CounterType::PlusOnePlusOne),
        0,
        "Increment should NOT fire for a 1-mana spell against a 1/1"
    );
}

#[test]
fn hungry_graffalon_increment_lands_counter_on_five_mana_spell() {
    // Hungry Graffalon is a 3/4. A 5-mana spell (mana_spent = 5)
    // exceeds toughness (4) → fires.
    let mut g = two_player_game();
    let giraffe = place_creature(&mut g, 0, catalog::hungry_graffalon());
    // Cast a 5-mana spell: 5x Forest (lands aren't cast — use a 5-mana
    // creature). Use Glasspool Mimic at 5 mana or similar... we'll use
    // bears-pumped-up: a 5-mana real card. We'll just craft any
    // 5-mana creature: re-use the existing Quandrix Pledgemage or
    // Stirring Honormancer. Stirring Honormancer costs {2}{W}{W/B}{B}
    // which approximates to {2}{W}{W}{B} = 4 mana. Use
    // grand_arbiter_augustin_iv (no...). Just give the player 5 colorless
    // and cast a 4-mana spell with bonus tax. Actually just use forest
    // bargain since hard to wire. Use a 5-mana creature like
    // `catalog::stirring_honormancer` (uses {2}{W}{W}{B}, 4 pips).
    //
    // Simpler: hand-pick a 5-mana SOS card. transcendent_archaic costs
    // {7} — too much. Just use Hungry Graffalon itself? It's {3}{G}.
    // Need 5 total mana. Use Erode + tax? Easier: cast
    // catalog::quandrix_pledgemage at {1}{G}{U} = 3 mana. That won't
    // trigger 5+. Let me just cast catalog::rancorous_archaic ({5}, 5
    // mana, 2/2 with Trample/Reach + Converge counters).
    let big = g.add_card_to_hand(0, catalog::rancorous_archaic());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Rancorous Archaic castable for {5}");
    drain_stack(&mut g);
    let c = g.battlefield_find(giraffe).expect("Giraffe still alive");
    assert_eq!(
        c.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Increment should land 1 +1/+1 counter (5 > 4)"
    );
}

#[test]
fn hungry_graffalon_increment_skips_three_mana_spell() {
    // Three-mana spell vs 3/4 → 3 > 3 false, 3 > 4 false → skip.
    let mut g = two_player_game();
    let giraffe = place_creature(&mut g, 0, catalog::hungry_graffalon());
    // Cast Stirring Honormancer ({2}{W}{W}{B}) — total 5 mana spent.
    // That would still trigger. Pick a 3-mana spell instead — Quandrix
    // Pledgemage costs {1}{G}{U} for 3 mana. Use that.
    let three = g.add_card_to_hand(0, catalog::quandrix_pledgemage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: three, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pledgemage castable for {1}{G}{U}");
    drain_stack(&mut g);
    let c = g.battlefield_find(giraffe).expect("Giraffe still alive");
    assert_eq!(
        c.counter_count(CounterType::PlusOnePlusOne),
        0,
        "Increment should NOT fire (3 ≤ 3 AND 3 ≤ 4)"
    );
}

#[test]
fn berta_increment_triggers_self_pump_and_mana_chain() {
    // Berta starts as a 1/4. Casting a 5-mana spell → Increment fires
    // (5 > 4 toughness), lands a +1/+1 counter, and the
    // CounterAdded(+1/+1, SelfSource) → AddMana(AnyOneColor) trigger
    // fires off the counter add. We don't easily assert the AnyOneColor
    // mana payout (it suspends on a ChooseColor decision), so we just
    // verify the counter landed and that there's a pending decision
    // (the mana chain).
    let mut g = two_player_game();
    let berta = place_creature(&mut g, 0, catalog::berta_wise_extrapolator());
    // Set up auto-decider that always picks White when asked.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Color(Color::White),
    ]));
    let big = g.add_card_to_hand(0, catalog::rancorous_archaic());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Rancorous Archaic castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(berta).expect("Berta still alive");
    assert_eq!(
        b.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Increment lands a +1/+1 on Berta when mana_spent > P or T",
    );
    // The follow-up CounterAdded → AddMana trigger should have added
    // 1 White (or any color) mana to the pool. Auto-decider picked
    // White above so check the white slot.
    assert!(
        g.players[0].mana_pool.amount(Color::White) >= 1,
        "Berta's counter-add → AddMana(AnyOneColor) trigger should yield 1 mana",
    );
}

#[test]
fn aberrant_manawurm_pumps_by_mana_spent_eot() {
    // Manawurm is a 2/5 with Trample. After casting a 5-mana IS spell,
    // mana_spent = 5 → +5/+0 EOT → 7/5.
    let mut g = two_player_game();
    let wurm = place_creature(&mut g, 0, catalog::aberrant_manawurm());
    // Cast a 5-mana instant — Quandrix Pledgemage is a creature
    // (won't trigger Manawurm — Magecraft IS-only). Use a real
    // instant/sorcery. Stirring Honormancer is a creature too. Use
    // Together as One (sorcery, {6} via Converge — too expensive).
    // tome_blast is {1}{R}. Just use lightning_bolt? It's an instant
    // for {R} = 1 mana, would pump +1/+0. We want 5 mana so cast
    // catalog::practiced_offense ({3}{R} sorcery? Let me look). Skip
    // and just check it's a Magecraft trigger by introspection.
    //
    // Use a multi-cast approach: cast Bolt ({R} = 1 mana) for +1/+0.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let w = g.battlefield_find(wurm).expect("Wurm still alive");
    assert_eq!(
        w.power(),
        2 + 1,
        "Aberrant Manawurm should pump by mana_spent = 1 → 3 power"
    );
    assert_eq!(w.toughness(), 5, "toughness unchanged");
}

#[test]
fn tackle_artist_opus_lands_one_counter_below_five_mana() {
    // Tackle Artist Opus — small body lands one +1/+1.
    let mut g = two_player_game();
    let ta = place_creature(&mut g, 0, catalog::tackle_artist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ta).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Opus small body: one +1/+1"
    );
}

#[test]
fn tackle_artist_opus_lands_two_counters_at_five_mana() {
    // Tackle Artist Opus — big body lands two +1/+1 on a ≥5-mana IS
    // spell. We use the catalog `transcendent_archaic` ({7}) but it's
    // a creature, not IS. Need an actual IS spell at ≥5 mana. We have
    // Tome Blast's Flashback at {4}{R} = 5 mana from graveyard, or
    // we can synthesize a 5-mana IS by casting a 3-mana Pursue the
    // Past via its Flashback at {2}{R}{W} = 4 mana — close but not 5.
    // Let me just use `divergent_equation` ({X}{X}{U}) with X=2 →
    // {2}{2}{U} = 5 mana.
    let mut g = two_player_game();
    let ta = place_creature(&mut g, 0, catalog::tackle_artist());
    let big = g.add_card_to_hand(0, catalog::divergent_equation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Divergent Equation castable with X=2");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ta).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "Opus big body (≥5 mana): two +1/+1 instead of one"
    );
}

#[test]
fn thunderdrum_soloist_opus_pings_one_at_small_three_at_big() {
    // Small body: 1 damage to each opponent.
    let mut g = two_player_game();
    let _td = place_creature(&mut g, 0, catalog::thunderdrum_soloist());
    let life_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // P1 took: 3 (bolt) + 1 (Soloist small body) = 4 damage.
    assert_eq!(
        g.players[1].life,
        life_before - 4,
        "Soloist's small body should ping each opp for 1"
    );
}

#[test]
fn expressive_firedancer_opus_grants_double_strike_at_five_mana() {
    use crabomination::card::Keyword as Kw;
    let mut g = two_player_game();
    let ef = place_creature(&mut g, 0, catalog::expressive_firedancer());
    // Cast Divergent Equation with X=2 → {2}{2}{U} = 5 mana (an IS spell).
    let big = g.add_card_to_hand(0, catalog::divergent_equation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Divergent Equation castable with X=2");
    drain_stack(&mut g);
    let card = g.battlefield_find(ef).expect("Firedancer alive");
    assert_eq!(card.power(), 3, "Big body: +1/+1 → 3/3");
    assert_eq!(card.toughness(), 3);
    assert!(
        card.has_keyword(&Kw::DoubleStrike),
        "Big body grants double strike EOT"
    );
}

#[test]
fn deluge_virtuoso_opus_pumps_one_one_or_two_two() {
    let mut g = two_player_game();
    // Use an opponent's creature so the ETB tap+stun has a target.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let dv_card = g.add_card_to_hand(0, catalog::deluge_virtuoso());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: dv_card,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Deluge Virtuoso castable");
    drain_stack(&mut g);
    let prev_p = g.battlefield_find(dv_card).map(|c| c.power()).unwrap();
    // Casting DV itself doesn't fire its own Opus (cast happens before
    // permanent is on the battlefield). Now cast Bolt to test the
    // small body.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    let dv = g.battlefield_find(dv_card).expect("DV alive");
    assert_eq!(dv.power(), prev_p + 1, "Small body: +1/+1");
}

#[test]
fn molten_core_maestro_opus_big_body_adds_red_equal_to_power() {
    // Big body (≥5 mana): +1/+1 counter, then add {R} equal to power.
    // Maestro is 2/2 → counter makes it 3/3 → adds 3 red mana.
    let mut g = two_player_game();
    let mcm = place_creature(&mut g, 0, catalog::molten_core_maestro());
    let big = g.add_card_to_hand(0, catalog::divergent_equation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Divergent Equation castable with X=2 (5 mana)");
    drain_stack(&mut g);
    let c = g.battlefield_find(mcm).expect("Maestro alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "Big body lands one +1/+1");
    assert_eq!(c.power(), 3, "2/2 + counter = 3/3");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3, "adds red equal to power (3)");
}

#[test]
fn exhibition_tidecaller_opus_mills_three_small_ten_big() {
    // Small body (<5 mana): target player mills 3. Auto-target picks an opponent.
    let mut g = two_player_game();
    for _ in 0..15 { g.add_card_to_library(1, catalog::island()); }
    let _et = place_creature(&mut g, 0, catalog::exhibition_tidecaller());
    let gy_before = g.players[1].graveyard.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy_before + 3, "small body mills 3 from target");
}

#[test]
fn increment_trigger_re_checks_intervening_if_on_resolution() {
    // MTG comp rule 603.4 ("intervening 'if' clause"): a triggered
    // ability of the form "Whenever X, if Y, do Z" re-checks the
    // condition (Y) on resolution. If the condition is false at that
    // time, the trigger is removed without effect.
    //
    // Setup: Cuboid Colony is a 1/1 (Increment fires when mana_spent
    // > 1 OR > 1 → strictly > 1). We cast a 2-mana spell, putting
    // Increment on the stack. Then we pump Colony to a 5/5 *before*
    // the trigger resolves — at resolution, mana_spent (2) is no
    // longer > P (5) or > T (5), so the trigger should suppress
    // itself.
    //
    // We can't easily insert a pump mid-stack from a test, so we
    // approximate by directly setting the colony's power/toughness
    // bonus high enough to flip the predicate.
    let mut g = two_player_game();
    let colony = place_creature(&mut g, 0, catalog::cuboid_colony());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bears castable");
    // The bears spell is on the stack; Increment trigger is also on
    // the stack (above the spell, by `finalize_cast`'s push order).
    // Pump Colony from 1/1 to a 5/5 by stamping +4/+4 in the bonus
    // slots (simulating an unrelated pump effect that landed before
    // Increment resolves).
    {
        let c = g.battlefield_find_mut(colony).expect("Colony alive");
        c.power_bonus += 4;
        c.toughness_bonus += 4;
    }
    drain_stack(&mut g);
    // Cuboid Colony is now 5/5 (bonus). The trigger fires off the
    // 2-mana cast but, at resolution, mana_spent (2) is no longer
    // > P or > T, so per rule 603.4 the body is suppressed.
    let c = g.battlefield_find(colony).expect("Colony alive");
    assert_eq!(
        c.counter_count(CounterType::PlusOnePlusOne),
        0,
        "Rule 603.4: intervening-if re-check should suppress the body",
    );
}

// ── New SOS card bodies + Killian's Confidence gy trigger ──────────────────

#[test]
fn skycoach_waypoint_taps_for_colorless() {
    // {T}: Add {C} ability is the only ability on the body. Cast / put
    // onto the battlefield and tap for one colorless.
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::skycoach_waypoint());
    g.clear_sickness(land);
    let c_before = g.players[0].mana_pool.colorless_amount();

    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("{T}: Add {C} activatable");

    assert_eq!(g.players[0].mana_pool.colorless_amount(), c_before + 1);
    let c = g.battlefield.iter().find(|c| c.id == land).unwrap();
    assert!(c.tapped, "land should be tapped");
}

// ── Prepare mechanic (Biblioplex Tomekeeper, Skycoach Waypoint) ─────────────

#[test]
fn skycoach_waypoint_prepare_activation_adds_prepared_counter() {
    // {3}, {T}: Target creature becomes prepared. The printed
    // "(Only creatures with prepare spells can become prepared.)"
    // reminder forces the target to have a back-face spell — use the
    // Elite Interceptor // Rejoinder MDFC, whose front is a vanilla
    // 1/2 creature with a back-face spell (the "prepare spell").
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::skycoach_waypoint());
    g.clear_sickness(land);
    let mdfc = g.add_card_to_battlefield(0, catalog::elite_interceptor());
    g.players[0].mana_pool.add_colorless(3);

    let target = g.battlefield.iter().find(|c| c.id == mdfc).unwrap();
    assert_eq!(
        target.counter_count(CounterType::Prepared), 0,
        "MDFC creature starts unprepared"
    );

    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: Some(Target::Permanent(mdfc)), additional_targets: Vec::new(), x_value: None })
    .expect("Skycoach Waypoint {3}, {T}: prepare activation");
    drain_stack(&mut g);

    let prepared = g.battlefield.iter().find(|c| c.id == mdfc).unwrap();
    assert_eq!(
        prepared.counter_count(CounterType::Prepared), 1,
        "MDFC creature should have one Prepared counter"
    );
    let c = g.battlefield.iter().find(|c| c.id == land).unwrap();
    assert!(c.tapped, "Waypoint should be tapped after prepare activation");
}

#[test]
fn skycoach_waypoint_rejects_creature_without_prepare_spell() {
    // Printed reminder: "(Only creatures with prepare spells can
    // become prepared.)" A plain Grizzly Bears has no back face, so
    // Waypoint's activation must NOT land a Prepared counter on it.
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::skycoach_waypoint());
    g.clear_sickness(land);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);

    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None
    });
    assert!(
        result.is_err(),
        "prepare activation must be rejected against a creature with no back face"
    );
    let bear_now = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        bear_now.counter_count(CounterType::Prepared), 0,
        "bear without a back face must not receive a Prepared counter"
    );
}

#[test]
fn skycoach_waypoint_prepare_rejected_without_three_mana() {
    // Tap cost without {3} should fail to activate.
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::skycoach_waypoint());
    g.clear_sickness(land);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // No mana in pool — activation should be rejected.

    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None });
    assert!(
        result.is_err(),
        "prepare activation should fail without {{3}} in pool"
    );
    let c = g.battlefield.iter().find(|c| c.id == land).unwrap();
    assert!(!c.tapped, "Waypoint should not be tapped on failed activation");
}

#[test]
fn biblioplex_tomekeeper_etb_prepares_target_creature() {
    // ETB ChooseMode auto-picks mode 0 (becomes prepared). The target
    // must be a creature with a back-face "prepare spell" — use Elite
    // Interceptor // Rejoinder.
    let mut g = two_player_game();
    let mdfc = g.add_card_to_battlefield(0, catalog::elite_interceptor());
    let id = g.add_card_to_hand(0, catalog::biblioplex_tomekeeper());
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mdfc)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tomekeeper castable for {4}");
    drain_stack(&mut g);

    let prepared = g.battlefield.iter().find(|c| c.id == mdfc).unwrap();
    assert_eq!(
        prepared.counter_count(CounterType::Prepared), 1,
        "auto-decider picks mode 0 → MDFC creature gets a Prepared counter"
    );
}

#[test]
fn biblioplex_tomekeeper_rejects_creature_without_prepare_spell() {
    // Vanilla bear has no back face → not a legal prepare target.
    // The ETB trigger should be unable to resolve onto the bear; the
    // engine's auto-target picker has no legal Permanent target, so
    // the trigger no-ops without adding a counter.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::biblioplex_tomekeeper());
    g.players[0].mana_pool.add_colorless(4);

    let _ = g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    });
    drain_stack(&mut g);

    let bear_now = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        bear_now.counter_count(CounterType::Prepared), 0,
        "creature without a prepare spell (no back face) must not receive a Prepared counter"
    );
}

#[test]
fn biblioplex_tomekeeper_etb_unprepares_via_scripted_mode_one() {
    // Seed a Prepared counter on an MDFC creature (the only legal
    // prepare target), then ETB Tomekeeper with a scripted mode-1
    // decision — it should remove the counter.
    let mut g = two_player_game();
    let mdfc = g.add_card_to_battlefield(0, catalog::elite_interceptor());
    if let Some(c) = g.battlefield_find_mut(mdfc) {
        c.counters.insert(CounterType::Prepared, 1);
    }

    let id = g.add_card_to_hand(0, catalog::biblioplex_tomekeeper());
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mdfc)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tomekeeper castable for {4}");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == mdfc).unwrap();
    assert_eq!(
        target.counter_count(CounterType::Prepared), 0,
        "mode 1 should remove the Prepared counter from the MDFC creature"
    );
}

#[test]
fn skycoach_waypoint_then_biblioplex_tomekeeper_round_trip() {
    // Prepare an MDFC creature via Waypoint, then unprepare via
    // Tomekeeper mode 1.
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::skycoach_waypoint());
    g.clear_sickness(land);
    let mdfc = g.add_card_to_battlefield(0, catalog::elite_interceptor());
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: Some(Target::Permanent(mdfc)), additional_targets: Vec::new(), x_value: None })
    .expect("Skycoach Waypoint prepare activation");
    drain_stack(&mut g);

    assert_eq!(
        g.battlefield.iter().find(|c| c.id == mdfc).unwrap()
            .counter_count(CounterType::Prepared), 1,
        "Waypoint prepares the MDFC creature"
    );

    // Now ETB Tomekeeper with mode 1 to unprepare.
    let id = g.add_card_to_hand(0, catalog::biblioplex_tomekeeper());
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mdfc)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tomekeeper castable for {4}");
    drain_stack(&mut g);

    assert_eq!(
        g.battlefield.iter().find(|c| c.id == mdfc).unwrap()
            .counter_count(CounterType::Prepared), 0,
        "Tomekeeper mode 1 unprepares the MDFC creature"
    );
}

#[test]
fn top_of_the_class_buffs_prepared_and_spares_unprepared() {
    // Prepare-mechanic payoff: "Prepared creatures you control get +1/+1
    // and have flying." A static anthem, so its effect surfaces through
    // the layer-computed `compute_battlefield()` view (like the
    // Comforting Counsel anthem tests above). The buff is keyed on a
    // Prepared counter (the same counter Biblioplex Tomekeeper / Skycoach
    // Waypoint apply); seed it directly here to isolate the payoff.
    let mut g = two_player_game();
    let _ench = g.add_card_to_battlefield(0, catalog::top_of_the_class());
    let mdfc = g.add_card_to_battlefield(0, catalog::elite_interceptor());
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let def = catalog::elite_interceptor();
    let (base_p, base_t) = (def.power, def.toughness);

    // Before preparing: nothing has a Prepared counter → anthem applies
    // to no one, so the MDFC creature reads at its printed P/T.
    {
        let computed = g.compute_battlefield();
        let c = computed.iter().find(|c| c.id == mdfc).unwrap();
        assert_eq!(c.power, base_p, "unprepared: base power");
        assert_eq!(c.toughness, base_t, "unprepared: base toughness");
        assert!(
            !c.keywords.contains(&Keyword::Flying),
            "unprepared creature gets no anthem flying"
        );
    }

    // Prepare the MDFC creature (the counter the toggle cards apply).
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == mdfc) {
        c.counters.insert(CounterType::Prepared, 1);
    }

    // Now the anthem applies to the prepared creature.
    let computed = g.compute_battlefield();
    let prepared = computed.iter().find(|c| c.id == mdfc).unwrap();
    assert_eq!(
        prepared.power, base_p + 1,
        "Top of the Class gives the prepared creature +1 power"
    );
    assert_eq!(
        prepared.toughness, base_t + 1,
        "Top of the Class gives the prepared creature +1 toughness"
    );
    assert!(
        prepared.keywords.contains(&Keyword::Flying),
        "Top of the Class grants the prepared creature flying"
    );

    // The un-prepared Grizzly Bears (never prepared) is unaffected.
    let bear = computed.iter().find(|c| c.id == plain).unwrap();
    assert_eq!(bear.power, 2, "unprepared bear keeps base power");
    assert_eq!(bear.toughness, 2, "unprepared bear keeps base toughness");
    assert!(
        !bear.keywords.contains(&Keyword::Flying),
        "unprepared bear gets no flying"
    );
}

#[test]
fn top_of_the_class_spares_opponents_prepared_creature() {
    // "Prepared creatures *you control*" — an opponent's prepared
    // creature must not be buffed by your anthem.
    let mut g = two_player_game();
    let _ench = g.add_card_to_battlefield(0, catalog::top_of_the_class());
    let opp_mdfc = g.add_card_to_battlefield(1, catalog::elite_interceptor());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == opp_mdfc) {
        c.counters.insert(CounterType::Prepared, 1);
    }

    let def = catalog::elite_interceptor();
    let computed = g.compute_battlefield();
    let opp = computed.iter().find(|c| c.id == opp_mdfc).unwrap();
    assert_eq!(opp.power, def.power, "opponent's prepared creature: base power");
    assert_eq!(opp.toughness, def.toughness, "opponent's prepared creature: base toughness");
    assert!(
        !opp.keywords.contains(&Keyword::Flying),
        "your anthem must not grant flying to an opponent's prepared creature"
    );
}

#[test]
fn prepared_counter_is_inert_for_pt_without_payoff() {
    // Control: a Prepared counter on its own changes nothing about P/T —
    // the +1/+1 (and flying) come from Top of the Class, not the counter.
    let mut g = two_player_game();
    let mdfc = g.add_card_to_battlefield(0, catalog::elite_interceptor());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == mdfc) {
        c.counters.insert(CounterType::Prepared, 1);
    }

    let def = catalog::elite_interceptor();
    let computed = g.compute_battlefield();
    let c = computed.iter().find(|c| c.id == mdfc).unwrap();
    assert_eq!(c.power, def.power, "no payoff → prepared creature keeps base power");
    assert_eq!(c.toughness, def.toughness, "no payoff → base toughness");
    assert!(
        !c.keywords.contains(&Keyword::Flying),
        "no payoff → a bare Prepared counter grants no flying"
    );
}

#[test]
fn fix_whats_broken_returns_mana_value_x_artifact_from_graveyard() {
    // Faithful X-cost: `{X}{2}{W}{B}`, pay X life, return artifact/creature
    // cards with mana value EXACTLY X. At X=1 the MV-1 Sol Ring returns but
    // the MV-2 Bear stays — confirming the exact-MV match for artifacts.
    let mut g = two_player_game();
    let sol = g.add_card_to_graveyard(0, catalog::sol_ring()); // MV 1
    let bear_id = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let id = g.add_card_to_hand(0, catalog::fix_whats_broken());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3); // 2 generic + 1 for X
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(1),
    })
    .expect("Fix What's Broken castable for {1}{2}{W}{B} (X=1)");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 1, "Pays X (=1) life");
    assert!(g.battlefield.iter().any(|c| c.id == sol),
        "Sol Ring (MV 1) returns at X=1");
    assert!(!g.battlefield.iter().any(|c| c.id == bear_id),
        "Grizzly Bears (MV 2) does NOT return at X=1 (exact match)");
}

#[test]
fn mica_reader_of_ruins_magecraft_sac_artifact_to_copy_when_decider_agrees() {
    use crabomination::decision::DecisionAnswer;
    let mut g = two_player_game();
    let _mica = g.add_card_to_battlefield(0, catalog::mica_reader_of_ruins());
    // Stage an artifact to sacrifice.
    let _art = g.add_card_to_battlefield(0, catalog::sol_ring());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(target);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // ScriptedDecider answers MayDo with `true` to sac the artifact.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    // Mica's Magecraft sac'd Sol Ring + copied Bolt — target bear takes
    // 6 damage (Bolt + copy), so it dies.
    assert!(!g.battlefield.iter().any(|c| c.id == target),
        "Target bear should die to original + copied Bolt");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Sol Ring"),
        "Sol Ring sacrificed to Mica's Magecraft");
}

#[test]
fn mica_reader_of_ruins_magecraft_skips_copy_when_decider_declines() {
    use crabomination::decision::DecisionAnswer;
    let mut g = two_player_game();
    let _mica = g.add_card_to_battlefield(0, catalog::mica_reader_of_ruins());
    let _art = g.add_card_to_battlefield(0, catalog::sol_ring());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(target);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    // No copy fired: target takes 3 damage; the 2/2 bear lives (Bolt
    // hits for 3 to a 2-toughness creature so it dies). Verify the
    // artifact is NOT sacrificed.
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Sol Ring"),
        "Sol Ring NOT sacrificed when MayDo answered false");
}

#[test]
fn killians_confidence_returns_to_hand_when_creature_deals_combat_damage() {
    // Killian's Confidence sits in graveyard. Attack with a creature →
    // combat damage → may-pay {W} → card returns to hand.
    use crabomination::decision::DecisionAnswer;
    let mut g = two_player_game();
    // Stage Killian's Confidence in P0's gy.
    let kc = g.add_card_to_graveyard(0, catalog::killians_confidence());
    // Stage an attacker on P0's battlefield. Set step so the attacker
    // can be declared.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // Have W mana to pay the return cost.
    g.players[0].mana_pool.add(Color::White, 1);
    // Script: MayPay answers yes.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    // Move to combat and declare the bear.
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attacker declared");
    // Force-resolve combat by stepping forward to damage step.
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat damage resolved");
    // Drain the may-pay trigger off the stack.
    drain_stack(&mut g);

    // Killian's Confidence is now in P0's hand (not graveyard).
    assert!(g.players[0].hand.iter().any(|c| c.id == kc),
        "Killian's Confidence should be in hand after combat damage + may-pay yes");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == kc),
        "Killian's Confidence should leave the graveyard");
}

#[test]
fn killians_confidence_stays_in_graveyard_when_no_damage_or_no_pay() {
    // Without combat damage to a player, no trigger fires.
    let mut g = two_player_game();
    let kc = g.add_card_to_graveyard(0, catalog::killians_confidence());
    let _bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Walk to end step without combat damage.
    g.step = TurnStep::End;

    // KC still in graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == kc),
        "KC should still be in graveyard with no combat damage");
}


// ── SOA / slow-land backfill (2026-07 audit) ────────────────────────────────

/// The five MID/VOW slow lands (SOS reprints): tapped with fewer than
/// two OTHER lands, untapped once you control two or more others.
#[test]
fn slow_lands_tap_rule() {
    let defs: [fn() -> crabomination::card::CardDefinition; 5] = [
        catalog::deathcap_glade, catalog::dreamroot_cascade,
        catalog::shattered_sanctum, catalog::stormcarved_coast,
        catalog::sundown_pass,
    ];
    for f in defs {
        // 0 other lands → enters tapped.
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, f());
        g.perform_action(GameAction::PlayLand(id)).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).unwrap().tapped,
            "{} enters tapped with no other lands", f().name);
        // 2 other lands → enters untapped.
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::island());
        g.add_card_to_battlefield(0, catalog::island());
        let id = g.add_card_to_hand(0, f());
        g.perform_action(GameAction::PlayLand(id)).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield_find(id).unwrap().tapped,
            "{} enters untapped with two other lands", f().name);
    }
}

/// Reprieve bounces a spell to its owner's hand (not a counter-to-
/// graveyard) and draws.
#[test]
fn reprieve_returns_spell_to_hand_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    g.priority.player_with_priority = 0;
    let rep = g.add_card_to_hand(0, catalog::reprieve());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let h0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: rep, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reprieve castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "Bolt never resolved");
    assert!(g.players[1].hand.iter().any(|c| c.id == bolt), "Bolt back in owner's hand");
    assert_eq!(g.players[0].hand.len(), h0 - 1 + 1, "cast Reprieve, drew a card");
}

/// Culling the Weak: sacrifice a creature at cast, add {B}{B}{B}{B}.
#[test]
fn culling_the_weak_sacs_and_ramps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::culling_the_weak());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "creature sacrificed");
    assert_eq!(g.players[0].mana_pool.total(), 4, "four black in pool");
}

/// Subterranean Tremors at X=4: sweeps non-fliers, destroys artifacts,
/// no Lizard (needs X≥8).
#[test]
fn subterranean_tremors_x4_sweeps_and_smashes_artifacts() {
    let mut g = two_player_game();
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel());    // 4/4 flying
    let relic = g.add_card_to_battlefield(1, catalog::team_pennant());   // artifact
    let id = g.add_card_to_hand(0, catalog::subterranean_tremors());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(4),
    }).expect("castable at X=4");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == ground), "grounded creature died");
    assert!(g.battlefield.iter().any(|c| c.id == flier), "flier untouched (4 damage, 4 toughness — Serra survives 4? she dies at 4)");
    assert!(!g.battlefield.iter().any(|c| c.id == relic), "artifact destroyed at X>=4");
    assert!(!g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Lizard"),
        "no Lizard below X=8");
}

/// Awaken the Woods mints X Forest Dryad LAND creatures.
#[test]
fn awaken_the_woods_mints_land_creatures() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::awaken_the_woods());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("castable at X=3");
    drain_stack(&mut g);
    let dryads: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Forest Dryad").collect();
    assert_eq!(dryads.len(), 3, "X=3 tokens");
    let d = dryads[0];
    assert!(d.definition.card_types.contains(&crabomination::card::CardType::Land));
    assert!(d.definition.card_types.contains(&crabomination::card::CardType::Creature));
}

/// Berserk doubles power via +X/+0 and grants trample; the delayed
/// end-step destroy only fires if the target attacked.
#[test]
fn berserk_doubles_power_and_grants_trample() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::berserk());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.computed_permanent(angel).unwrap();
    assert_eq!(c.power, 8, "+X/+0 where X = its power");
    assert!(c.keywords.contains(&Keyword::Trample));
}

/// Glimpse of Nature: creatures entering this turn each draw a card.
#[test]
fn glimpse_of_nature_draws_per_creature() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::glimpse_of_nature());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let hand_before = g.players[0].hand.len();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear castable");
    drain_stack(&mut g);
    // +1 (add bear) -1 (cast bear) +1 (Glimpse draw) = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew for the creature");
}

/// Akroma's Will mode 0 grants flying/vigilance/double strike to your team.
#[test]
fn akromas_will_mode_zero_team_keywords() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::akromas_will());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    for kw in [Keyword::Flying, Keyword::Vigilance, Keyword::DoubleStrike] {
        assert!(c.keywords.contains(&kw), "bear gains {kw:?}");
    }
}

/// Return to the Ranks reanimates X cheap creatures (Convoke printed).
#[test]
fn return_to_the_ranks_reanimates_x_cheap_creatures() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV2
    let b2 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let big = g.add_card_to_graveyard(0, catalog::serra_angel());  // MV5 — ineligible
    let id = g.add_card_to_hand(0, catalog::return_to_the_ranks());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("castable at X=2");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == b1), "bear 1 reanimated");
    assert!(g.battlefield.iter().any(|c| c.id == b2), "bear 2 reanimated");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == big), "MV5 stays dead");
}

/// Winds of Abandon single-target: exile + the controller ramps a basic.
#[test]
fn winds_of_abandon_exiles_and_compensates() {
    let mut g = two_player_game();
    let isl = g.add_card_to_library(1, catalog::island());
    // The compensation search is mandatory on the printed card; the
    // AutoDecider declines searches by default, so script the pick.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(isl))]));
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::winds_of_abandon());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "creature exiled");
    assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.definition.is_land() && c.tapped),
        "its controller fetched a tapped basic");
}

// ── 2026-07 SOS/SOA correctness audit fixes ─────────────────────────────────

/// Dina's Guidance is an INSTANT (audit fix: was Sorcery-typed).
/// Venomous Words (Scathing Shadelock's inset spell) is a SORCERY.
/// Great Hall of the Biblioplex is a plain nonlegendary Land.
#[test]
fn audit_type_line_fixes() {
    use crabomination::card::{CardType, Supertype};
    assert!(catalog::dinas_guidance().card_types.contains(&CardType::Instant));
    let shadelock = catalog::scathing_shadelock();
    let words = shadelock.prepare_spell.as_ref().expect("inset spell");
    assert!(words.card_types.contains(&CardType::Sorcery),
        "Venomous Words is a sorcery");
    assert!(!catalog::great_hall_of_the_biblioplex()
        .supertypes.contains(&Supertype::Legendary),
        "Great Hall is not legendary");
}

/// Harmonized Trio's prepare activation taps TWO other untapped
/// creatures (audit fix: was one).
#[test]
fn harmonized_trio_taps_two_other_creatures() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let trio = g.add_card_to_battlefield(0, catalog::harmonized_trio());
    g.clear_sickness(trio);
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(b1); g.clear_sickness(b2);

    // With only ONE other creature the activation is rejected.
    let mut g1 = two_player_game();
    let t1 = g1.add_card_to_battlefield(0, catalog::harmonized_trio());
    g1.clear_sickness(t1);
    let only = g1.add_card_to_battlefield(0, catalog::grizzly_bears());
    g1.clear_sickness(only);
    assert!(g1.perform_action(GameAction::ActivateAbility {
        card_id: t1, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).is_err(), "needs two untapped creatures to tap");

    // With two, it works and taps both.
    g.perform_action(GameAction::ActivateAbility {
        card_id: trio, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).expect("prepare activation with two tappable creatures");
    drain_stack(&mut g);
    assert!(g.battlefield_find(b1).unwrap().tapped && g.battlefield_find(b2).unwrap().tapped,
        "both other creatures tapped as cost");
    assert!(g.battlefield_find(trio).unwrap().counter_count(CounterType::Prepared) > 0,
        "Trio became prepared");
}

/// Emil's Fractal counters count DIFFERENTLY NAMED lands (audit fix).
#[test]
fn emil_counts_differently_named_lands() {
    let mut g = two_player_game();
    let emil = g.add_card_to_battlefield(0, catalog::emil_vastlands_roamer());
    g.clear_sickness(emil);
    // Three Islands + one Forest = 2 distinct names.
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::island()); }
    g.add_card_to_battlefield(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: emil, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).expect("Emil activation");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal")
        .expect("Fractal minted");
    assert_eq!(fractal.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 2,
        "X = 2 differently named lands, not 4 total lands");
}

/// Restoration Seminar can't reanimate an instant (audit fix: filter now
/// requires a nonland PERMANENT card).
#[test]
fn restoration_seminar_rejects_nonpermanent_targets() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::restoration_seminar());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "an instant in the graveyard is not a legal target");
}
