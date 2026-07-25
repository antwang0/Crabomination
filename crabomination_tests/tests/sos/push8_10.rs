#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use crate::prepared_on_battlefield;

// ── push VIII: Lesson cycle + bodies + Resonating Lute ───────────────────────

#[test]
fn primary_research_etb_returns_low_mv_card_from_graveyard() {
    let mut g = two_player_game();
    let bear_in_grave = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::primary_research());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear_in_grave)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Primary Research castable for {4}{W}");
    drain_stack(&mut g);

    // Bear should be back on the battlefield, owned by P0.
    assert!(g.battlefield.iter().any(|c| c.id == bear_in_grave),
        "Grizzly Bears should be back on the battlefield");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear_in_grave),
        "Grizzly Bears should no longer be in the graveyard");
}

#[test]
fn primary_research_end_step_draws_when_card_left_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::primary_research());
    drain_stack(&mut g);
    g.players[0].cards_left_graveyard_this_turn = 1;
    g.add_card_to_library(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Primary Research should draw at end step when a card left graveyard");

    // Quiet turn: no draw.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::primary_research());
    drain_stack(&mut g);
    g.players[0].cards_left_graveyard_this_turn = 0;
    g.add_card_to_library(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before,
        "Primary Research should NOT draw when nothing left graveyard");
}

#[test]
fn artistic_process_mode0_deals_six_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::artistic_process());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: Some(0), x_value: None,
    })
    .expect("Artistic Process castable for {3}{R}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Grizzly Bears should be dead from 6 damage");
}

#[test]
fn artistic_process_mode2_creates_haste_elemental() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::artistic_process());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(2), x_value: None,
    })
    .expect("Artistic Process mode 2 castable");
    drain_stack(&mut g);

    // The Elemental token entered the battlefield; the spell itself
    // resolved into the graveyard. Net battlefield change = +1 token.
    assert_eq!(g.battlefield.len(), bf_before + 1);
    let tok = g.battlefield.iter()
        .find(|c| c.definition.name == "Elemental")
        .expect("Elemental token created");
    assert_eq!(tok.definition.power, 3);
    assert_eq!(tok.definition.toughness, 3);
    assert!(tok.definition.keywords.contains(&Keyword::Flying));
    // Haste was granted via duration::EOT — verify at the engine level
    // by checking the transient keyword count on the token.
    assert!(g.permanent_has_keyword(tok.id, &Keyword::Haste),
        "freshly-minted Elemental should have transient Haste");
}

#[test]
fn decorum_dissertation_draws_two_loses_two() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::decorum_dissertation());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;

    // Push (modern_decks): Decorum Dissertation now uses
    // `target_filtered(Player)` — target self for the printed asymmetric
    // "you draw 2, you lose 2" trade.
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Decorum Dissertation castable for {3}{B}{B}");
    drain_stack(&mut g);

    // Hand: -1 cast + 2 drawn = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert_eq!(g.players[0].life, life_before - 2);

    // Decorum Dissertation can also target an opponent — letting the caster
    // asymmetrically give the opp 2 cards in exchange for draining them
    // 2 life. Push (modern_decks) multi-target promotion.
    let mut g = two_player_game();
    g.add_card_to_library(1, catalog::lightning_bolt());
    g.add_card_to_library(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::decorum_dissertation());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    let opp_hand_before = g.players[1].hand.len();
    let opp_life_before = g.players[1].life;
    let self_hand_before = g.players[0].hand.len();
    let self_life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Decorum Dissertation castable for {3}{B}{B}");
    drain_stack(&mut g);

    // Opp drew 2 + lost 2 life.
    assert_eq!(g.players[1].hand.len(), opp_hand_before + 2);
    assert_eq!(g.players[1].life, opp_life_before - 2);
    // Caster's hand and life unchanged (apart from the cast).
    assert_eq!(g.players[0].hand.len(), self_hand_before - 1);
    assert_eq!(g.players[0].life, self_life_before);
}

#[test]
fn germination_practicum_pumps_each_creature() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::germination_practicum());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Germination Practicum castable for {3}{G}{G}");
    drain_stack(&mut g);

    let count_on = |g: &GameState, cid| g.battlefield_find(cid)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne) as i32)
        .unwrap_or(0);
    assert_eq!(count_on(&g, bear1), 2,
        "Friendly bear 1 should get 2 +1/+1 counters");
    assert_eq!(count_on(&g, bear2), 2,
        "Friendly bear 2 should get 2 +1/+1 counters");
    assert_eq!(count_on(&g, opp_bear), 0,
        "Opp bear should get 0 +1/+1 counters");
}

#[test]
fn restoration_seminar_returns_permanent_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::restoration_seminar());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Restoration Seminar castable for {5}{W}{W}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "Grizzly Bears should be on the battlefield");
}

#[test]
fn ennis_debate_moderator_etb_exiles_friendly_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::ennis_debate_moderator());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Ennis castable for {1}{W}");
    drain_stack(&mut g);

    // Auto-target picker chose the bear; bear is now in exile.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be flickered off the battlefield");
    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in the shared exile zone");
}

#[test]
fn ennis_debate_moderator_end_step_counter_when_card_exiled() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ennis_debate_moderator());
    drain_stack(&mut g);
    // Set the per-turn exile tally to gate `CardsExiledThisTurnAtLeast`.
    g.players[0].cards_exiled_this_turn = 1;

    let counters_before = g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);

    let counters_after = g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters_after, counters_before + 1,
        "Ennis should get +1/+1 counter at end step when a card was exiled");

    // No exile this turn: no counter.
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ennis_debate_moderator());
    drain_stack(&mut g);
    g.players[0].cards_exiled_this_turn = 0;
    let counters_before = g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);

    let counters_after = g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters_after, counters_before,
        "Ennis should NOT get +1/+1 counter when no card was exiled");
}

#[test]
fn cards_exiled_this_turn_tally_bumps_on_exile_effect() {
    // Verify the new `cards_exiled_this_turn` tally bumps when a
    // creature is exiled by Wander Off. Active player casts Wander
    // Off on their own bear (a self-removal nonsense play).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wander = g.add_card_to_hand(0, catalog::wander_off());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    assert_eq!(g.players[0].cards_exiled_this_turn, 0);

    g.perform_action(GameAction::CastSpell {
        card_id: wander, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Wander Off castable for {3}{B}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in exile");
    assert_eq!(g.players[0].cards_exiled_this_turn, 1,
        "Per-turn exile tally should bump for the player who cast the exile spell");
}

/// Tragedy Feaster's Infusion: at end step, if you didn't gain life this
/// turn, sacrifice a permanent. With no life gain, P0 sacrifices the
/// cheapest creature available.
#[test]
fn tragedy_feaster_infusion_forces_sacrifice_when_no_life_gained() {
    let mut g = two_player_game();
    let feaster = g.add_card_to_battlefield(0, catalog::tragedy_feaster());
    g.clear_sickness(feaster);
    // Add a cheaper fodder creature to be sac'd first.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // No life gained this turn.
    g.players[0].life_gained_this_turn = 0;
    let bf_count_before = g.battlefield.iter().filter(|c| c.controller == 0).count();

    g.fire_step_triggers(crabomination::game::types::TurnStep::End);
    drain_stack(&mut g);

    let bf_count_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(
        bf_count_after,
        bf_count_before - 1,
        "Tragedy Feaster's Infusion should force a sacrifice when no life was gained"
    );
}

/// Tragedy Feaster's Infusion is suppressed when lifegain happened.
#[test]
fn tragedy_feaster_infusion_skips_sacrifice_when_life_gained() {
    let mut g = two_player_game();
    let feaster = g.add_card_to_battlefield(0, catalog::tragedy_feaster());
    g.clear_sickness(feaster);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.players[0].life_gained_this_turn = 1; // any lifegain bypasses the sac.
    let bf_count_before = g.battlefield.iter().filter(|c| c.controller == 0).count();

    g.fire_step_triggers(crabomination::game::types::TurnStep::End);
    drain_stack(&mut g);

    let bf_count_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(
        bf_count_after, bf_count_before,
        "No sac when life was gained — every permanent stays"
    );
}

#[test]
fn forum_necroscribe_repartee_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let _necro = g.add_card_to_battlefield(0, catalog::forum_necroscribe());
    let bear_in_grave = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);

    // Cast a creature-targeting instant — Lightning Bolt at the opp bear.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    // Opp bear dead from bolt; friendly bear back on battlefield from
    // Repartee return.
    assert!(g.battlefield.iter().any(|c| c.id == bear_in_grave),
        "Forum Necroscribe Repartee should return the gy bear to bf");
}

#[test]
fn paradox_surveyor_etb_reveals_to_find_basic_land() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let forest_id = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt());

    let id = g.add_card_to_hand(0, catalog::paradox_surveyor());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Paradox Surveyor castable for {G}{G}{U}");
    drain_stack(&mut g);

    // Surveyor entered the bf. The Forest among the top 5 should have
    // been moved to hand.
    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Paradox Surveyor should be on the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == forest_id),
        "Forest should have been pulled into hand");
    // Hand size: -1 cast + 1 forest = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn magmablood_archaic_pumps_friendly_creatures_on_two_color_cast() {
    // Push (modern_decks): IS-cast trigger pumps each of your
    // creatures by +X/+0 EOT where X = colors of mana spent on the
    // iterated cast. Cast a 2-color IS spell with Magmablood out;
    // assert friendly bear gains +2/+0 (its power becomes 4).
    let mut g = two_player_game();
    let _mb = g.add_card_to_battlefield(0, catalog::magmablood_archaic());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    // Cast Quandrix Charm (a 2-color IS spell — {G}{U}).
    let charm = g.add_card_to_hand(0, catalog::quandrix_charm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    // ChooseMode 1 (destroy target enchantment) to avoid relying on
    // a stack target for the spell body. Modes 0 / 2 also work but
    // need targets.
    g.perform_action(GameAction::CastSpell {
        card_id: charm,
        target: None,
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("Quandrix Charm castable for {G}{U}");
    drain_stack(&mut g);
    // Magmablood's spell-cast trigger should have pumped the bear by
    // +2/+0 EOT (G + U = 2 distinct colors).
    let bear_after = g.battlefield_find(bear).unwrap();
    assert!(
        bear_after.power() >= 4,
        "Bear should be pumped +2/+0 by Magmablood's per-cast pump (was 2; now {})",
        bear_after.power(),
    );
}

/// Wildgrowth Archaic: cast it with 2 colors of mana spent (G + 2
/// converted = colors-of-mana = 1 if generic doesn't count, but our
/// convergence counts distinct colors paid). Verify it lands with X
/// +1/+1 counters per CR 614.12, where X is the number of colors of
/// mana spent. Casting at 1 color = 1 counter (survives ETB).
#[test]
fn wildgrowth_archaic_enters_with_one_counter_per_color_spent() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::wildgrowth_archaic());
    // Pay {2}{2}{G}{G} -> 1 color (Green) spent.
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wildgrowth castable for 6 mana");
    drain_stack(&mut g);
    let view = g.computed_permanent(id).unwrap();
    // 1 color spent → 1 +1/+1 counter → 1/1 with Trample + Reach.
    assert_eq!(view.power, 1, "Wildgrowth at 1 color = 1/1");
    assert_eq!(view.toughness, 1);
}

/// Wildgrowth Archaic's "creature spells you cast enter with X
/// additional +1/+1 counters" static — cast Grizzly Bears AFTER
/// Wildgrowth Archaic is on the battlefield. Bears (1 color of mana
/// spent, Green) should land as a 3/3 (2+1).
#[test]
fn wildgrowth_archaic_grants_extra_counter_to_creature_spells() {
    let mut g = two_player_game();
    // Seed the Archaic directly on battlefield (skip cast to focus on
    // the static rider).
    let _archaic = g.add_card_to_battlefield(0, catalog::wildgrowth_archaic());
    drain_stack(&mut g);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bears castable for {1}{G}");
    drain_stack(&mut g);
    let view = g.computed_permanent(bears).unwrap();
    // Bears (2/2) + 1 +1/+1 counter (1 color spent) = 3/3.
    assert_eq!(view.power, 3,
        "Grizzly Bears should land as 3/3 (printed 2/2 + 1 counter from Wildgrowth)");
    assert_eq!(view.toughness, 3);
}

/// Wildgrowth Archaic's static does NOT apply to creature spells cast
/// by an opponent (it's gated on `src.controller == caster`).
#[test]
fn wildgrowth_archaic_static_does_not_grant_to_opp_creature_spells() {
    let mut g = two_player_game();
    let _archaic = g.add_card_to_battlefield(0, catalog::wildgrowth_archaic());
    drain_stack(&mut g);
    let opp_bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    // Pass turn so opp can cast at sorcery speed.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: opp_bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Opp Bears castable for {1}{G}");
    drain_stack(&mut g);
    let view = g.computed_permanent(opp_bears).unwrap();
    // Opp's Bears land vanilla 2/2 (Wildgrowth doesn't pump opp's spells).
    assert_eq!(view.power, 2);
    assert_eq!(view.toughness, 2);
}

#[test]
fn ambitious_augmenter_increments_on_three_mana_cast() {
    // Increment: when a 3-mana spell is cast (3 > toughness of 1), gain
    // a +1/+1 counter. Same `increment_self_plus_one()` helper exercised
    // by Hungry Graffalon / Cuboid Colony.
    let mut g = two_player_game();
    let aug = g.add_card_to_battlefield(0, catalog::ambitious_augmenter());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::shock());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Shock castable for {R}");
    drain_stack(&mut g);
    // Shock costs 1 — equal to Augmenter's power/toughness (both 1).
    // The Increment trigger checks "mana spent > P or T", so 1 > 1 is
    // false; no counter should be added.
    let aug_after = g.battlefield_find(aug).unwrap();
    assert_eq!(aug_after.counter_count(CounterType::PlusOnePlusOne), 0,
        "1-mana spell does not trigger Increment");
    // Now cast a 2-mana spell — 2 > 1 should trigger Increment.
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Grizzly Bears castable for {1}{G}");
    drain_stack(&mut g);
    let aug_after = g.battlefield_find(aug).unwrap();
    assert_eq!(aug_after.counter_count(CounterType::PlusOnePlusOne), 1,
        "2-mana spell triggers Increment, +1/+1 counter on Augmenter");
}

#[test]
fn ambitious_augmenter_increments_when_paid_via_auto_tap() {
    // Regression for the auto-tap path: in actual gameplay the player
    // casts with an empty floating pool and the engine auto-taps lands to
    // pay. Previously `pool_before` was captured pre-auto-tap, so
    // `mana_spent = pool_before(0) - pool_after(0) = 0` and Increment
    // silently failed. Build a board with two untapped Forests and a
    // 2-mana spell — auto-tap should produce mana_spent = 2 and the
    // Augmenter should pick up a +1/+1 counter.
    let mut g = two_player_game();
    let aug = g.add_card_to_battlefield(0, catalog::ambitious_augmenter());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0,
        "starting pool is empty — Augmenter must rely on auto-tap");
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Grizzly Bears castable for {1}{G} via auto-tapped Forests");
    drain_stack(&mut g);
    let aug_after = g.battlefield_find(aug).unwrap();
    assert_eq!(aug_after.counter_count(CounterType::PlusOnePlusOne), 1,
        "auto-tapped 2-mana cast should still trigger Increment");
}

#[test]
fn ambitious_augmenter_death_with_counters_creates_fractal_with_counters() {
    // CR 122.2 + push (modern_decks) death trigger: when Augmenter dies
    // with N +1/+1 counters on it, create a Fractal token and transfer
    // the N counters onto the Fractal.
    let mut g = two_player_game();
    let aug = g.add_card_to_battlefield(0, catalog::ambitious_augmenter());
    drain_stack(&mut g);
    // Manually stack three +1/+1 counters on Augmenter to simulate
    // accumulated Increment.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == aug) {
        c.add_counters(CounterType::PlusOnePlusOne, 3);
    }
    // Send Augmenter to the graveyard via the engine's die path so the
    // CreatureDied trigger fires.
    let _ = g.remove_to_graveyard_with_triggers(aug);
    drain_stack(&mut g);
    // Augmenter should be in graveyard and a Fractal token on the
    // battlefield with 3 +1/+1 counters.
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == aug),
        "Augmenter dies → goes to graveyard",
    );
    let fractal = g.battlefield.iter().find(|c| c.definition.name == "Fractal");
    let Some(fractal) = fractal else {
        panic!("Expected a Fractal token on the battlefield after Augmenter dies");
    };
    assert!(fractal.is_token, "Fractal is a token");
    assert_eq!(
        fractal.counter_count(CounterType::PlusOnePlusOne),
        3,
        "Fractal token should carry the dying Augmenter's 3 +1/+1 counters",
    );
}

#[test]
fn ambitious_augmenter_death_without_counters_does_not_create_fractal() {
    let mut g = two_player_game();
    let aug = g.add_card_to_battlefield(0, catalog::ambitious_augmenter());
    drain_stack(&mut g);
    let _ = g.remove_to_graveyard_with_triggers(aug);
    drain_stack(&mut g);
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == aug),
        "Augmenter dies → goes to graveyard",
    );
    assert!(
        !g.battlefield.iter().any(|c| c.definition.name == "Fractal"),
        "No Fractal token should be created when Augmenter dies without counters",
    );
}

#[test]
fn resonating_lute_draw_blocked_when_hand_below_seven() {
    let mut g = two_player_game();
    let lute = g.add_card_to_battlefield(0, catalog::resonating_lute());
    drain_stack(&mut g);
    // P0's hand is empty (well below 7).
    assert!(g.players[0].hand.is_empty());
    g.add_card_to_library(0, catalog::lightning_bolt());

    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: lute, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None });
    assert!(res.is_err(),
        "Resonating Lute should reject activation with hand size < 7");
    assert!(g.players[0].hand.is_empty(),
        "No card should be drawn when activation is rejected");
    assert!(!g.battlefield_find(lute).unwrap().tapped,
        "Lute should not have been tapped (cost roll-back on gate fail)");
}

#[test]
fn resonating_lute_draw_succeeds_at_seven_in_hand() {
    let mut g = two_player_game();
    let lute = g.add_card_to_battlefield(0, catalog::resonating_lute());
    drain_stack(&mut g);
    for _ in 0..7 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    g.add_card_to_library(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: lute, ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Resonating Lute should activate when hand size ≥ 7");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert!(g.battlefield_find(lute).unwrap().tapped,
        "Lute should be tapped after activation");
}

#[test]
fn resonating_lute_grants_lands_tap_for_any_color() {
    // "Lands you control have '{T}: Add two mana of any one color. Spend
    // this mana only to cast instant and sorcery spells.'" — wired via
    // StaticEffect::GrantActivatedAbility. A Forest (1 printed mana
    // ability) gets the grant at index 1; tapping it adds two restricted
    // mana of one chosen color.
    let mut g = two_player_game();
    let _lute = g.add_card_to_battlefield(0, catalog::resonating_lute());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    let free_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("Resonating Lute grants lands a tap-for-any-color ability at index 1");
    assert_eq!(g.players[0].mana_pool.total(), free_before,
        "granted mana is restricted, so the free total is unchanged");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2,
        "two mana of one color, spend-restricted to instants/sorceries");
    assert!(g.battlefield_find(forest).unwrap().tapped, "land tapped for the grant");
}

#[test]
fn potioners_trove_lifegain_blocked_without_spell_cast() {
    let mut g = two_player_game();
    let trove = g.add_card_to_battlefield(0, catalog::potioners_trove());
    drain_stack(&mut g);
    g.players[0].spells_cast_this_turn = 0;

    // Lifegain ability index 1 (mana ability is index 0).
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: trove, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None });
    assert!(res.is_err(),
        "Potioner's Trove lifegain should reject without IS-cast this turn");
    assert!(!g.battlefield_find(trove).unwrap().tapped,
        "Trove should not be tapped (cost roll-back)");
}

#[test]
fn potioners_trove_lifegain_succeeds_after_spell_cast() {
    let mut g = two_player_game();
    let trove = g.add_card_to_battlefield(0, catalog::potioners_trove());
    drain_stack(&mut g);
    // Set the IS tally directly (the printed predicate uses
    // `instants_or_sorceries_cast_this_turn`, not the generic count).
    g.players[0].instants_or_sorceries_cast_this_turn = 1;
    let life_before = g.players[0].life;

    g.perform_action(GameAction::ActivateAbility {
        card_id: trove, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Potioner's Trove lifegain should activate when a spell was cast");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 2);
}

// ── Witherbloom finisher + Surveil-anchored cards ──────────────────────────

#[test]
fn essenceknit_scholar_etb_creates_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::essenceknit_scholar());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Green, 1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Essenceknit Scholar castable for {B}{B}{G}");
    drain_stack(&mut g);

    // Scholar + Pest token = 2 new battlefield permanents.
    assert_eq!(g.battlefield.len(), bf_before + 2,
        "Essenceknit Scholar should ETB and mint a Pest token");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Pest"));
}

#[test]
fn essenceknit_scholar_end_step_draws_when_creature_died() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::essenceknit_scholar());
    drain_stack(&mut g);
    // Set the controller's "creatures died this turn" tally directly.
    g.players[0].creatures_died_this_turn = 1;
    g.add_card_to_library(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Essenceknit Scholar should draw at end step when a creature died this turn");
}

#[test]
fn essenceknit_scholar_no_draw_on_quiet_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::essenceknit_scholar());
    drain_stack(&mut g);
    // Scholar's own ETB drops a Pest, but no creatures have died.
    g.players[0].creatures_died_this_turn = 0;
    g.add_card_to_library(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before,
        "Essenceknit Scholar should NOT draw when no creature died");
}

#[test]
fn creatures_died_this_turn_no_bump_on_exile() {
    // Sanity check: the dies-this-turn tally should NOT bump when a
    // creature is removed via exile (it didn't actually die — exile
    // bypasses the SBA dies handler and `remove_to_graveyard_with_triggers`).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wander = g.add_card_to_hand(0, catalog::wander_off());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    assert_eq!(g.players[0].creatures_died_this_turn, 0);

    g.perform_action(GameAction::CastSpell {
        card_id: wander, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Wander Off castable for {3}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be off the battlefield (exiled by Wander Off)");
    assert_eq!(g.players[0].creatures_died_this_turn, 0,
        "Exile (not destroy) should NOT bump the dies-this-turn tally");
    // The exile tally, on the other hand, SHOULD bump.
    assert_eq!(g.players[0].cards_exiled_this_turn, 1,
        "Wander Off exile should bump the per-turn exile tally");
}

#[test]
fn creatures_died_this_turn_tally_bumps_on_lethal_damage() {
    // Combat / damage-lethal path: the bear takes lethal damage via
    // SBA and the tally bumps for the bear's controller.
    let mut g = two_player_game();
    let _bear_owned = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.players[0].creatures_died_this_turn, 0);
    // Apply 2 damage directly via the damage helper: bear dies to SBA.
    let bear_id = g.battlefield.iter()
        .find(|c| c.definition.name == "Grizzly Bears")
        .unwrap()
        .id;
    if let Some(bear) = g.battlefield.iter_mut().find(|c| c.id == bear_id) {
        bear.damage = 5;
    }
    let mut events = g.check_state_based_actions();
    let _ = events.drain(..);

    assert!(!g.battlefield.iter().any(|c| c.id == bear_id),
        "Bear should be dead from lethal damage");
    assert_eq!(g.players[0].creatures_died_this_turn, 1,
        "Per-turn died-creature tally should bump on lethal damage");
}

#[test]
fn professor_dellian_fel_plus_two_gains_three_life() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::professor_dellian_fel());
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    let loyalty_before = g.battlefield_find(pw)
        .unwrap()
        .counter_count(CounterType::Loyalty);

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: pw, ability_index: 0,
        target: None,
    })
    .expect("Professor Dellian Fel +2 should activate");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 3);
    assert_eq!(g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty),
        loyalty_before + 2);
}

#[test]
fn professor_dellian_fel_minus_three_destroys_creature() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::professor_dellian_fel());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: pw, ability_index: 2,
        target: Some(Target::Permanent(bear)),
    })
    .expect("Professor Dellian Fel -3 should activate");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "-3 should destroy the targeted bear");
}

#[test]
fn professor_dellian_fel_minus_six_activates_lifegain_drain_emblem() {
    // Push (modern_decks, batch 90): Dellian Fel's -6 emblem ult. After
    // activation, the player owns an emblem whose LifeGained trigger
    // fires "each opponent loses that much life" via the dispatcher's
    // player-emblem branch.
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::professor_dellian_fel());
    // Bump loyalty so -6 is payable.
    {
        let c = g.battlefield_find_mut(pw).unwrap();
        c.add_counters(crabomination::card::CounterType::Loyalty, 1);
    }
    assert!(g.players[0].emblems.is_empty());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: pw, ability_index: 3, target: None,
    }).expect("Dellian -6 castable at 6 loyalty");
    drain_stack(&mut g);
    assert_eq!(g.players[0].emblems.len(), 1,
        "Emblem added to the command zone after -6 activation");

    // Now gain 5 life on P0 — the emblem should drain P1 by 5.
    let p1_life_before = g.players[1].life;
    g.adjust_life(0, 5);
    // Manually emit + dispatch the LifeGained event so the unified
    // dispatcher fires the emblem trigger.
    let evs = vec![crabomination::game::GameEvent::LifeGained { player: 0, amount: 5 }];
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life_before - 5,
        "Emblem fired: P1 lost 5 life when P0 gained 5");
}

#[test]
fn unsubtle_mockery_deals_4_and_surveils() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::unsubtle_mockery());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Stack a card on top of library so Surveil has something to look at.
    g.add_card_to_library(0, catalog::lightning_bolt());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Unsubtle Mockery castable for {2}{R}");
    drain_stack(&mut g);

    // Bear (2/2) takes 4 damage = lethal.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Unsubtle Mockery should kill the 2/2 bear with 4 damage");
}

#[test]
fn muses_encouragement_creates_3_3_flying_elemental_and_surveils() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::muses_encouragement());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::grizzly_bears());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Muse's Encouragement castable for {4}{U}");
    drain_stack(&mut g);

    let elemental = g.battlefield.iter()
        .find(|c| c.definition.name == "Elemental")
        .expect("Muse's Encouragement should mint an Elemental");
    assert_eq!(elemental.power(), 3);
    assert_eq!(elemental.toughness(), 3);
    assert!(elemental.definition.keywords.contains(&Keyword::Flying));
}

#[test]
fn prismari_charm_mode2_bounces_nonland_to_owner() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_charm());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: Some(2), x_value: None,
    })
    .expect("Prismari Charm castable for {U}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Mode 2 should bounce the opp bear to its owner's hand");
    assert!(g.players[1].hand.iter().any(|c| c.id == opp_bear),
        "Bounced bear should be in opp's hand");
}

#[test]
fn prismari_charm_mode1_deals_one_damage() {
    let mut g = two_player_game();
    // Use a 1-toughness creature so the 1 damage is lethal.
    let savannah_lions = g.add_card_to_battlefield(1, catalog::savannah_lions());
    let id = g.add_card_to_hand(0, catalog::prismari_charm());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(savannah_lions)),
        additional_targets: vec![],
        mode: Some(1), x_value: None,
    })
    .expect("Prismari Charm castable for {U}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == savannah_lions),
        "Mode 1 should kill the 2/1 Savannah Lions with 1 damage");
}

#[test]
fn prismari_charm_mode1_hits_two_targets_including_a_player() {
    // Printed "deals 1 damage to each of one or two targets" — mode 1 is
    // `ApplyToTargets { max_targets: 2, min_targets: 1 }` with no type
    // restriction, so a creature and a player can both be hit.
    let mut g = two_player_game();
    let lions = g.add_card_to_battlefield(1, catalog::savannah_lions());
    let id = g.add_card_to_hand(0, catalog::prismari_charm());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(lions)),
        additional_targets: vec![Target::Player(1)],
        mode: Some(1),
        x_value: None,
    })
    .expect("Prismari Charm castable at two targets");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == lions),
        "1 damage kills the 2/1");
    assert_eq!(g.players[1].life, 19, "player took 1 damage");
}

#[test]
fn textbook_tabulator_etb_surveils_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::textbook_tabulator());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::grizzly_bears());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Textbook Tabulator castable for {2}{U}");
    drain_stack(&mut g);

    let perm = g.battlefield_find(id).expect("Tabulator should ETB");
    assert_eq!(perm.power(), 0);
    assert_eq!(perm.toughness(), 3);
}

#[test]
fn deluge_virtuoso_etb_taps_and_stuns_target() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::deluge_virtuoso());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Deluge Virtuoso castable for {2}{U}");
    drain_stack(&mut g);

    let bear = g.battlefield_find(opp_bear).expect("bear still on battlefield");
    assert!(bear.tapped, "Deluge Virtuoso ETB should tap the target");
    assert!(bear.counter_count(CounterType::Stun) >= 1,
        "Deluge Virtuoso ETB should add a stun counter");
}

#[test]
fn moseo_veins_new_dean_is_2_1_flying_pest_etb_minter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::moseo_veins_new_dean());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Moseo castable for {2}{B}");
    drain_stack(&mut g);

    let moseo = g.battlefield_find(id).expect("Moseo should ETB");
    assert_eq!(moseo.power(), 2);
    assert_eq!(moseo.toughness(), 1);
    assert!(moseo.definition.keywords.contains(&Keyword::Flying));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Pest"),
        "Moseo's ETB should mint a Pest token");
}

/// Moseo's Infusion end-step trigger: when you've gained life this turn,
/// return a creature card with MV <= life gained from your graveyard to
/// the BATTLEFIELD (audit fix — was to hand, ungated).
#[test]
fn moseo_veins_new_dean_infusion_reanimates_when_life_gained() {
    let mut g = two_player_game();
    // Seed a creature in P0's graveyard.
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let moseo = g.add_card_to_battlefield(0, catalog::moseo_veins_new_dean());
    g.clear_sickness(moseo);
    // Also seed an over-MV creature: 2 life gained gates X=2 (Bear MV 2
    // is eligible; Serra Angel MV 5 is not).
    let big = g.add_card_to_graveyard(0, catalog::serra_angel());
    // Simulate gaining life this turn.
    g.players[0].life_gained_this_turn = 2;

    // Fire end-step trigger by advancing to end step.
    g.fire_step_triggers(crabomination::game::types::TurnStep::End);
    drain_stack(&mut g);

    // Bear reanimated to the battlefield; the MV-5 Angel stays dead.
    assert!(
        g.battlefield.iter().any(|c| c.id == bear),
        "Moseo's Infusion returns the MV<=X creature to the battlefield"
    );
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == big),
        "creature above the life-gained MV gate stays in the graveyard"
    );
}

/// Moseo's Infusion end-step trigger is gated: no life gained → no return.
#[test]
fn moseo_veins_new_dean_infusion_no_return_without_life_gain() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let moseo = g.add_card_to_battlefield(0, catalog::moseo_veins_new_dean());
    g.clear_sickness(moseo);
    // No life gained this turn.

    g.fire_step_triggers(crabomination::game::types::TurnStep::End);
    drain_stack(&mut g);

    // Bear should still be in P0's graveyard.
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == bear),
        "Bear should remain in graveyard when no life was gained"
    );
}

#[test]
fn page_loose_leaf_taps_for_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::page_loose_leaf());
    g.clear_sickness(id);
    drain_stack(&mut g);

    let mana_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Page, Loose Leaf {T}: Add {C} should activate");
    drain_stack(&mut g);

    assert_eq!(g.players[0].mana_pool.total(), mana_before + 1);
    assert!(g.battlefield_find(id).unwrap().tapped);
}

#[test]
fn page_loose_leaf_grandeur_rejected_without_another_page_in_hand() {
    // Push (modern_decks, batch 92): Grandeur activation requires
    // another Page in hand. With only one Page on the battlefield and
    // no other Page in hand, the activation gate (Predicate::
    // SameNamedInZoneAtLeast in hand ≥ 1) rejects.
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::page_loose_leaf());
    g.clear_sickness(id);
    drain_stack(&mut g);

    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None });
    assert!(result.is_err(),
        "Grandeur rejected without another Page in hand");
}

#[test]
fn page_loose_leaf_grandeur_with_another_page_reveals_is_card() {
    // With another Page in hand, the Grandeur activation succeeds: it
    // discards the other Page (auto-picker picks first hand card),
    // then reveals until an instant or sorcery card → hand, rest →
    // bottom of library randomized.
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::page_loose_leaf());
    g.clear_sickness(id);
    // Seed another Page in hand.
    let _other_page = g.add_card_to_hand(0, catalog::page_loose_leaf());
    // Seed library: 2 lands + 1 Lightning Bolt (IS) on top.
    use crabomination::card::CardInstance;
    let mut top: Vec<CardInstance> = vec![
        CardInstance::new(g.next_id(), catalog::forest(), 0),
        CardInstance::new(g.next_id(), catalog::forest(), 0),
        CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0),
    ];
    for c in top.iter_mut() { c.controller = 0; }
    for c in top.into_iter().rev() {
        g.players[0].library.insert(0, c);
    }
    drain_stack(&mut g);

    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None });
    assert!(result.is_ok(),
        "Grandeur should activate when another Page is in hand");
    drain_stack(&mut g);

    // Lightning Bolt should be in hand.
    let bolt_in_hand = g.players[0].hand.iter()
        .any(|c| c.definition.name == "Lightning Bolt");
    assert!(bolt_in_hand, "Grandeur revealed and put an IS card into hand");
}

#[test]
fn ral_zarek_minus_two_returns_low_mv_creature_from_graveyard() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::ral_zarek_guest_lecturer());
    let bear_in_grave = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    drain_stack(&mut g);

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: pw, ability_index: 2,
        target: Some(Target::Permanent(bear_in_grave)),
    })
    .expect("Ral Zarek -2 should activate");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bear_in_grave),
        "Ral Zarek -2 should return the bear from graveyard to battlefield");
}

#[test]
fn ral_zarek_minus_seven_skips_target_opp_turns_via_coin_flip() {
    // ScriptedDecider answers `Bool(true)` for every coin flip = all
    // 5 heads → P1 (the only opp) gains skip_turns += 5.
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::ral_zarek_guest_lecturer());
    // Bump loyalty to 7 so the -7 cost is payable.
    {
        let c = g.battlefield_find_mut(pw).unwrap();
        c.add_counters(crabomination::card::CounterType::Loyalty, 4);
    }
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: pw, ability_index: 3, target: None,
    }).expect("Ral Zarek -7 should activate at 7 loyalty");
    drain_stack(&mut g);

    assert_eq!(g.players[1].skip_turns, 5,
        "All 5 heads → P1 skips 5 turns");
}

#[test]
fn skip_turns_counter_decrements_on_turn_advance() {
    // Player 1 has skip_turns=2. When the engine would hand the turn
    // to P1, decrement and skip past — P1 should never become the
    // active player until the counter reaches 0.
    let mut g = two_player_game();
    g.players[1].skip_turns = 2;
    assert_eq!(g.active_player_idx, 0, "starts at P0");

    // Advance through cleanup (P0 → P1 normally; with skip, P0 → P0).
    g.do_cleanup(&mut Vec::new());
    assert_eq!(g.active_player_idx, 0, "P1's turn 1 skipped, lands back on P0");
    assert_eq!(g.players[1].skip_turns, 1, "skip counter decremented");

    g.do_cleanup(&mut Vec::new());
    assert_eq!(g.active_player_idx, 0, "P1's turn 2 skipped, still on P0");
    assert_eq!(g.players[1].skip_turns, 0, "all skip turns consumed");

    g.do_cleanup(&mut Vec::new());
    assert_eq!(g.active_player_idx, 1, "back to normal — P1's turn");
}

#[test]
fn flow_state_scrys_three_then_draws() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::flow_state());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Prepare library cards so scry+draw has something to operate on.
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Flow State castable for {1}{U}");
    drain_stack(&mut g);

    // Hand size increased by 1 (the spell itself left hand on cast +
    // +1 from draw = net same size; the spell's own card isn't there
    // pre-cast, so net hand size before-cast = N-1, post-cast = N).
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Flow State should net +1 card to hand (spell itself left, draw 1 added)");
}

// ── Flashback wirings (push X) ──────────────────────────────────────────────

#[test]
fn daydream_flashback_replays_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_graveyard(0, catalog::daydream());
    // Flashback {2}{W}.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastFlashback {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Daydream flashback castable for {2}{W}");
    drain_stack(&mut g);

    // The flickered bear is back with a +1/+1 counter, and the spell is exiled.
    let view = g.computed_permanent(bear).expect("Bear should be back");
    assert_eq!(view.power, 3, "Bear with +1/+1 counter = 3 power");
    assert!(g.exile.iter().any(|c| c.id == id),
        "Flashback-cast Daydream should be in exile, not graveyard");
}

#[test]
fn pursue_the_past_flashback_replays_loot() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::pursue_the_past());
    // Stash a card to discard, and seed library for two draws.
    g.add_card_to_hand(0, catalog::lightning_bolt());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    // Flashback cost {2}{R}{W}.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Opt into the may-discard.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastFlashback {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pursue the Past flashback castable for {2}{R}{W}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 2, "Gain 2 life on flashback");
    // Hand: -1 from discard, +2 from draw → net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Net +1 card after discard 1 / draw 2");
    assert!(g.exile.iter().any(|c| c.id == id),
        "Flashback-cast Pursue the Past should land in exile");
}

// ── Inkshape Demonstrator (new card; push X) ────────────────────────────────

#[test]
fn inkshape_demonstrator_repartee_pumps_and_grants_lifelink() {
    let mut g = two_player_game();
    let demo = g.add_card_to_battlefield(0, catalog::inkshape_demonstrator());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Cast a creature-targeting instant — Inkshape's Repartee fires.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    let v = g.computed_permanent(demo).expect("Demo should still be on battlefield");
    assert_eq!(v.power, 4, "Inkshape Demonstrator should be +1/+0 → 4 power");
    assert!(v.keywords.contains(&Keyword::Lifelink),
        "Repartee should grant Lifelink EOT");
}

// ── Ward enforcement (CR 702.21) ────────────────────────────────────────────

#[test]
fn ward_counters_opp_spell_when_payer_cannot_afford() {
    // Opp casts Lightning Bolt at P0's Inkshape Demonstrator (Ward 2).
    // Opp has only {R} in pool — no spare {2} for the Ward tax — so the
    // Ward trigger counters the bolt. The Demonstrator survives at full
    // toughness.
    let mut g = two_player_game();
    let demo = g.add_card_to_battlefield(0, catalog::inkshape_demonstrator());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    // Only enough mana for the spell, not the Ward tax.
    g.players[1].mana_pool.add(Color::Red, 1);
    // Hand priority to P1 so they can cast at instant speed.
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(demo)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R} (Ward is paid at trigger resolution)");
    drain_stack(&mut g);

    let v = g.computed_permanent(demo).expect("Demonstrator should survive Ward 2");
    assert_eq!(v.toughness, 4, "Demonstrator at full 4 toughness — bolt was countered by Ward");
    let bolt_in_gy = g.players[1].graveyard.iter().any(|c| c.id == bolt);
    assert!(bolt_in_gy, "Countered spell goes to its owner's graveyard");
}

#[test]
fn ward_allows_opp_spell_when_payer_can_afford() {
    // Opp has enough mana to pay the Ward 2 tax on top of the bolt cost.
    // Auto-pay covers it; the bolt resolves and the Demonstrator dies
    // (3 damage to a 4-toughness creature wouldn't kill it, so we add a
    // -1/-1 counter rider via Crippling Fear-style: instead, just verify
    // 3 damage lands).
    let mut g = two_player_game();
    let demo = g.add_card_to_battlefield(0, catalog::inkshape_demonstrator());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    // {R} for the bolt + {2} generic for the Ward tax = enough.
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(demo)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable; Ward auto-paid");
    drain_stack(&mut g);

    let bolt_in_gy = g.players[1].graveyard.iter().any(|c| c.id == bolt);
    assert!(bolt_in_gy, "Bolt resolved and went to graveyard");
    let demo_card = g.battlefield.iter().find(|c| c.id == demo)
        .expect("Demonstrator survives 3 damage to a 4-toughness body");
    assert_eq!(demo_card.damage, 3, "Bolt's 3 damage should land — Ward was paid");
}

#[test]
fn ward_does_not_trigger_on_caster_own_spell() {
    // P0 owns the Ward 2 creature. P0 casts a buff on it — Ward doesn't
    // fire (Ward only triggers on opp-controlled spells per CR 702.21a).
    let mut g = two_player_game();
    let demo = g.add_card_to_battlefield(0, catalog::inkshape_demonstrator());
    // Use Inkshape's own Repartee Lightning Bolt as the test — bolt P0's own
    // bear so it's a creature-targeting spell. Verify the bolt resolves
    // without being countered.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // P0 casts the bolt at their OWN Demonstrator (illegal in normal play
    // — but the bolt's filter is Creature/Player/PW, no targeting
    // restriction. Ward should NOT fire because P0 is the caster.)
    // To keep the test clean, bolt P0's own bear instead — same caster
    // identity check.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt cast on own bear");
    drain_stack(&mut g);

    // Demonstrator was never targeted — Ward never had a reason to fire,
    // but the broader check is that the bolt resolved (didn't get tangled
    // up by anyone's Ward) and the bear died.
    let bear_dead = g.players[0].graveyard.iter().any(|c| c.id == bear);
    assert!(bear_dead, "Bear dies to bolt — Ward did not interfere with P0's own cast");
    assert!(
        g.computed_permanent(demo).is_some(),
        "Demonstrator untouched by P0's own cast"
    );
}

// ── Ward—Pay N life (Mica, Reader of Ruins) ─────────────────────────────────

#[test]
fn ward_pay_life_counters_when_payer_has_insufficient_life() {
    // P0's Mica has Ward—Pay 3 life. P1 has 2 life and casts Bolt at Mica.
    // 2 < 3, so the Ward trigger can't pay → bolt is countered.
    let mut g = two_player_game();
    let mica = g.add_card_to_battlefield(0, catalog::mica_reader_of_ruins());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].life = 2;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(mica)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    // Mica survives (bolt countered).
    let mica_card = g.battlefield.iter().find(|c| c.id == mica)
        .expect("Mica survives — Ward—Pay 3 life triggered with insufficient life");
    assert_eq!(mica_card.damage, 0, "no damage dealt — bolt was countered");
    let bolt_in_gy = g.players[1].graveyard.iter().any(|c| c.id == bolt);
    assert!(bolt_in_gy, "Countered bolt goes to its owner's graveyard");
    assert_eq!(g.players[1].life, 2, "no life paid — payment failed pre-deduction");
}

#[test]
fn ward_pay_life_resolves_when_payer_has_sufficient_life() {
    // P1 has 20 life, can pay the 3-life Ward, bolt resolves and Mica
    // (a 4/4) takes 3 damage but survives. P1 ends at 17 life.
    let mut g = two_player_game();
    let mica = g.add_card_to_battlefield(0, catalog::mica_reader_of_ruins());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].life = 20;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(mica)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);

    let mica_card = g.battlefield.iter().find(|c| c.id == mica)
        .expect("Mica survives — bolt resolved, 3 damage to a 4-toughness body");
    assert_eq!(mica_card.damage, 3, "bolt's 3 damage lands");
    assert_eq!(g.players[1].life, 17, "Ward—Pay 3 life deducted from P1");
}

// ── Ward—Discard a card (Forum Necroscribe) ─────────────────────────────────

#[test]
fn ward_discard_counters_when_payer_has_no_other_cards_in_hand() {
    // P0's Necroscribe has Ward—Discard a card. P1's only hand card is the
    // bolt itself — once cast, the hand is empty. Ward trigger can't
    // collect 1 discard → bolt countered.
    let mut g = two_player_game();
    let necro = g.add_card_to_battlefield(0, catalog::forum_necroscribe());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(necro)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);

    let necro_card = g.battlefield.iter().find(|c| c.id == necro)
        .expect("Necroscribe survives — bolt was countered by Ward—Discard");
    assert_eq!(necro_card.damage, 0, "no damage dealt — bolt was countered");
    let bolt_in_gy = g.players[1].graveyard.iter().any(|c| c.id == bolt);
    assert!(bolt_in_gy, "Countered bolt goes to graveyard");
}

#[test]
fn ward_discard_resolves_when_payer_has_a_spare_card() {
    // P1 has a spare card in hand. Ward—Discard auto-pays by discarding
    // the first hand card; bolt resolves and deals 3 to Necroscribe (a
    // 5/4 — survives).
    let mut g = two_player_game();
    let necro = g.add_card_to_battlefield(0, catalog::forum_necroscribe());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let spare = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(necro)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);

    let necro_card = g.battlefield.iter().find(|c| c.id == necro)
        .expect("Necroscribe survives bolt: 3 damage to 4-toughness body");
    assert_eq!(necro_card.damage, 3, "bolt resolved — 3 damage");
    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == spare),
        "Ward—Discard moved the spare card to graveyard"
    );
}

// ── Ward on activated abilities (CR 702.21a "spell or ability") ─────────────

#[test]
fn ward_counters_opp_activated_ability_when_payer_cannot_afford() {
    // P0 has Inkshape Demonstrator (Ward 2). P1 has Prodigal Sorcerer with
    // its {T}: deal 1 damage activation. P1 has no spare mana — the Ward
    // trigger can't auto-pay {2}, so the activation is countered. The
    // Demonstrator takes no damage.
    let mut g = two_player_game();
    let demo = g.add_card_to_battlefield(0, catalog::inkshape_demonstrator());
    let sorcerer = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
    // Clear summoning sickness so the tap activation is legal.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == sorcerer) {
        c.summoning_sick = false;
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sorcerer,
        ability_index: 0,
        target: Some(Target::Permanent(demo)), additional_targets: Vec::new(), x_value: None })
    .expect("Activation legal at the cost line — Ward fires after the ability is queued");
    drain_stack(&mut g);

    let demo_card = g.battlefield.iter().find(|c| c.id == demo)
        .expect("Demonstrator still on battlefield");
    assert_eq!(demo_card.damage, 0, "Sorcerer's ping was countered by Ward");
}

#[test]
fn ward_allows_opp_activated_ability_when_payer_can_afford() {
    // Same setup, but P1 has {2} colorless in pool to auto-pay Ward 2.
    // Activation resolves; Demonstrator takes 1 damage from the ping.
    let mut g = two_player_game();
    let demo = g.add_card_to_battlefield(0, catalog::inkshape_demonstrator());
    let sorcerer = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == sorcerer) {
        c.summoning_sick = false;
    }
    g.players[1].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sorcerer,
        ability_index: 0,
        target: Some(Target::Permanent(demo)), additional_targets: Vec::new(), x_value: None })
    .expect("Activation legal");
    drain_stack(&mut g);

    let demo_card = g.battlefield.iter().find(|c| c.id == demo)
        .expect("Demonstrator still on battlefield");
    assert_eq!(demo_card.damage, 1, "Ward was paid — ping landed");
}

// ── Studious First-Year preparation card ────────────────────────────────────

#[test]
fn studious_first_year_is_not_an_mdfc_castspellback_rejected() {
    // Preparation cards are creature cards with an inset prepare spell —
    // not MDFCs. CastSpellBack from hand must reject (no back face).
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::studious_first_year());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(err, Err(GameError::NotALand(_))),
        "preparation card has no back face — CastSpellBack must reject");
}

#[test]
fn studious_first_year_prepare_spell_fetches_tapped_forest() {
    // Cast the inset Rampant Growth via CastPrepareSpell for {1}{G} off a
    // prepared Studious First-Year; the Forest lands tapped.
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let id = prepared_on_battlefield(&mut g, 0, catalog::studious_first_year());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
    ]));

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("prepare spell castable for {1}{G} (Rampant Growth)");
    drain_stack(&mut g);

    // A tapped Forest should be on the battlefield under P0's control.
    let forest_view = g
        .battlefield
        .iter()
        .find(|c| c.id == forest)
        .expect("Search should put the Forest onto the battlefield");
    assert!(forest_view.tapped, "Rampant Growth fetches the basic land tapped");
}

// ── Fractal Tender / Thornfist Striker / Lumaret's Favor (push X) ───────────

#[test]
fn thornfist_striker_infusion_pumps_friendly_creatures_when_life_gained() {
    // Push (modern_decks): Infusion lifegain-anthem now wired via the new
    // `lifegain_anthem_for_name` compute-time injection. When the controller
    // has gained life this turn, the Striker grants +1/+0 and Trample to
    // every creature they control (including the Striker itself).
    let mut g = two_player_game();
    let striker = g.add_card_to_battlefield(0, catalog::thornfist_striker());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(striker);
    g.clear_sickness(bear);

    // Without lifegain: bear is a vanilla 2/2 with no trample.
    let bear_base = g.computed_permanent(bear).unwrap();
    assert_eq!(bear_base.power, 2, "bear is 2/2 without lifegain");
    assert!(!bear_base.keywords.contains(&Keyword::Trample),
        "bear has no trample without lifegain");

    // After lifegain: bear is 3/2 with trample.
    g.players[0].life_gained_this_turn = 1;
    let bear_pumped = g.computed_permanent(bear).unwrap();
    assert_eq!(bear_pumped.power, 3, "bear is 3/2 with lifegain");
    assert!(bear_pumped.keywords.contains(&Keyword::Trample),
        "bear gains trample with lifegain");
    let striker_pumped = g.computed_permanent(striker).unwrap();
    assert_eq!(striker_pumped.power, 4, "striker is 4/3 with lifegain (3+1)");
    assert!(striker_pumped.keywords.contains(&Keyword::Trample),
        "striker also gets trample (inclusive 'creatures you control')");
}

#[test]
fn thornfist_striker_infusion_does_not_buff_opponent_creatures() {
    // The anthem is keyed off the controller's life_gained_this_turn and
    // only buffs the Striker controller's creatures.
    let mut g = two_player_game();
    let _striker = g.add_card_to_battlefield(0, catalog::thornfist_striker());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].life_gained_this_turn = 5;

    let opp_pt = g.computed_permanent(opp_bear).unwrap();
    assert_eq!(opp_pt.power, 2, "opp bear unaffected by friendly anthem");
    assert!(!opp_pt.keywords.contains(&Keyword::Trample),
        "opp bear does not gain trample");
}

#[test]
fn lumarets_favor_pumps_creature_plus_two_plus_four() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lumarets_favor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Lumaret's Favor castable for {1}{G}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear should still be alive");
    assert_eq!(v.power, 2 + 2, "Bear should be +2 power → 4");
    assert_eq!(v.toughness, 2 + 4, "Bear should be +4 toughness → 6");
}

#[test]
fn lumarets_favor_infusion_copies_when_life_gained_this_turn() {
    // Infusion trigger fires on cast when life-gained-this-turn,
    // copying via `Effect::CopySpell` → +2/+4 pump stacks.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].life_gained_this_turn = 1; // simulate a prior lifegain trigger
    let id = g.add_card_to_hand(0, catalog::lumarets_favor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Lumaret's Favor castable for {1}{G}");
    drain_stack(&mut g);

    // Two pumps applied: +2/+4 twice → +4/+8 over the bear's printed 2/2.
    let v = g.computed_permanent(bear).expect("Bear should still be alive");
    assert_eq!(v.power, 2 + 4, "Bear should be pumped twice via Infusion copy → 6 power");
    assert_eq!(v.toughness, 2 + 8, "Bear should be pumped twice → 10 toughness");
}

#[test]
fn social_snub_copies_when_caster_controls_a_creature_and_decider_agrees() {
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Bob controls a creature so each-player sac works on resolution.
    let _bob_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::social_snub());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Decider answers Bool(true) so the MayDo runs CopySpell.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let alice_life_before = g.players[0].life;
    let bob_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Social Snub castable for {1}{W}{B}");
    drain_stack(&mut g);

    // Drain 1 happens twice (original + copy) → Bob -2, Alice +2.
    assert_eq!(g.players[0].life, alice_life_before + 2,
        "Alice should gain 1 life from each resolution (×2)");
    assert_eq!(g.players[1].life, bob_life_before - 2,
        "Bob should lose 1 life from each resolution (×2)");
}

#[test]
fn copied_spell_does_not_linger_in_graveyard_after_resolution() {
    // CR 707.10a: a copy of a spell ceases to exist in any zone other
    // than the stack. Our implementation marks the copy `is_token =
    // true` so the existing token-cleanup SBA path drops it from
    // graveyard / hand / library / exile after resolution.
    let mut g = two_player_game();
    let aziza = g.add_card_to_battlefield(0, catalog::aziza_mage_tower_captain());
    g.clear_sickness(aziza);
    for _ in 0..3 {
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let gy_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    // Only the original bolt goes to graveyard. The copy ceases to
    // exist after resolution (CR 707.10a).
    assert_eq!(g.players[0].graveyard.len(), gy_before + 1,
        "Copy of Lightning Bolt should not linger in graveyard");
    let bolt_count = g.players[0].graveyard.iter()
        .filter(|c| c.definition.name == "Lightning Bolt")
        .count();
    assert_eq!(bolt_count, 1, "Only the original bolt should be in gy");
}

#[test]
fn social_snub_does_not_copy_without_a_creature() {
    let mut g = two_player_game();
    // No creatures controlled by the caster — trigger filter rejects.
    let id = g.add_card_to_hand(0, catalog::social_snub());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let bob_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Social Snub castable for {1}{W}{B}");
    drain_stack(&mut g);

    // Only one drain — no copy.
    assert_eq!(g.players[1].life, bob_life_before - 1);
}

#[test]
fn lumarets_favor_infusion_does_not_copy_without_lifegain() {
    // No life gained this turn → trigger filter blocks the copy.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // life_gained_this_turn defaults to 0.
    let id = g.add_card_to_hand(0, catalog::lumarets_favor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Lumaret's Favor castable for {1}{G}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear should still be alive");
    assert_eq!(v.power, 2 + 2);
    assert_eq!(v.toughness, 2 + 4);
}

#[test]
fn cast_spell_back_face_rejected_cast_restores_front_face() {
    // Regression: a rejected back-face cast must restore the front-face
    // definition in hand so the player can still cast either face. Before
    // the fix the in-hand card's definition stayed swapped to the back
    // face on rejection, burning the front face for the rest of the game.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::studious_first_year());
    // Pay only {G} — Rampant Growth (back face) costs {1}{G}, so the
    // cast attempt should fail on mana payment.
    g.players[0].mana_pool.add(Color::Green, 1);

    let err = g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "underpaying the back-face cost should reject");

    // Front-face definition must be back in place: name + types unchanged.
    let card = g.players[0].hand.iter().find(|c| c.id == id)
        .expect("card stays in hand on rejected cast");
    assert_eq!(card.definition.name, catalog::studious_first_year().name);
    assert!(card.definition.is_creature(),
        "front face is a creature; if this fails the back face leaked through");
}

#[test]
fn cast_spell_back_face_rejects_card_without_back_face() {
    // A card without `back_face` (most cards) should reject CastSpellBack
    // cleanly with `NotALand`. This ensures the new path doesn't crash
    // when called against the wrong card.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    let err = g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(err, Err(GameError::NotALand(_))),
        "CastSpellBack on a card without a back face should error cleanly");
}

// ── Preparation cards (mdfcs.rs) ───────────────────────────────────────────
// Each preparation-card test exercises the inset prepare spell via
// CastPrepareSpell off a prepared creature on the battlefield.

// Emeritus of Truce // Swords to Plowshares — exile creature, owner gains
// life equal to its power.
#[test]
fn emeritus_of_truce_prepare_spell_exiles_creature_and_grants_life() {
    let mut g = two_player_game();
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::emeritus_of_truce());
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(opp_creature)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Swords to Plowshares castable for {W}");
    drain_stack(&mut g);

    // Bear has power 2; opponent gains 2 life when exiled (target's owner).
    assert!(!g.battlefield.iter().any(|c| c.id == opp_creature),
        "Bear should be exiled by Swords");
    assert_eq!(g.players[1].life, life_before + 2,
        "Owner should gain life equal to creature's power (2)");
}

// Honorbound Page // Forum's Favor — +1/+0 and flying until end of turn.
#[test]
fn honorbound_page_prepare_spell_pumps_and_grants_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::honorbound_page());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Forum's Favor castable for {W}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear should still be alive");
    assert_eq!(v.power, 2 + 1, "Bear should be +1 power");
    assert_eq!(v.toughness, 2, "Forum's Favor grants +1/+0 — toughness unchanged");
    assert!(v.keywords.contains(&Keyword::Flying), "Bear gains flying EOT");
}

// Joined Researchers // Secret Rendezvous — each player draws 3.
#[test]
fn joined_researchers_prepare_spell_each_player_draws_three() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::lightning_bolt());
        g.add_card_to_library(1, catalog::lightning_bolt());
    }
    let id = prepared_on_battlefield(&mut g, 0, catalog::joined_researchers());
    let caster_hand_before = g.players[0].hand.len();
    let opp_hand_before = g.players[1].hand.len();
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        // "You and TARGET OPPONENT each draw three."
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Secret Rendezvous castable for {1}{W}{W}");
    drain_stack(&mut g);

    // The copy never lived in hand — both players just gain 3 draws.
    assert_eq!(g.players[0].hand.len(), caster_hand_before + 3);
    assert_eq!(g.players[1].hand.len(), opp_hand_before + 3);
}

// Elite Interceptor // Rejoinder — "You may tap or untap target creature.
// Draw a card." Mode 0 taps; the draw is unconditional.
#[test]
fn elite_interceptor_rejoinder_taps_target_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let id = prepared_on_battlefield(&mut g, 0, catalog::elite_interceptor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("Rejoinder castable for {1}{W}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().find(|c| c.id == bear).unwrap().tapped,
        "mode 0 taps the target creature");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "Rejoinder draws a card");
}

// Quill-Blade Laureate // Twofold Intent — +1/+0 + double strike EOT.
#[test]
fn quill_blade_laureate_prepare_spell_pumps_and_grants_double_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::quill_blade_laureate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Twofold Intent castable for {1}{W}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear pumped");
    assert_eq!(v.power, 3, "Twofold Intent grants +1/+0");
    assert_eq!(v.toughness, 2, "toughness unchanged");
    assert!(v.keywords.contains(&Keyword::DoubleStrike), "Bear gains double strike EOT");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Inkling"),
        "Twofold Intent makes no token");
}

// Spiritcall Enthusiast // Scrollboost — single target +2/+2 EOT.
#[test]
fn spiritcall_enthusiast_prepare_spell_pumps_target_two_two() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::spiritcall_enthusiast());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(b1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Scrollboost castable for {1}{W}");
    drain_stack(&mut g);

    let v1 = g.computed_permanent(b1).expect("Bear alive");
    assert_eq!(v1.power, 4, "targeted bear is +2/+2");
    assert_eq!(v1.toughness, 4);
    let v2 = g.computed_permanent(b2).expect("Bear alive");
    assert_eq!(v2.power, 2, "untargeted bear unchanged");
}

// Encouraging Aviator // Jump — grant flying EOT.
#[test]
fn encouraging_aviator_prepare_spell_grants_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::encouraging_aviator());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Jump castable for {U}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear alive");
    assert!(v.keywords.contains(&Keyword::Flying), "Bear gains Flying EOT");
}

// Harmonized Trio // Brainstorm — draw 3 + put 2 back on top.
#[test]
fn harmonized_trio_prepare_spell_draws_three_then_puts_two_back() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let id = prepared_on_battlefield(&mut g, 0, catalog::harmonized_trio());
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Brainstorm castable for {U}");
    drain_stack(&mut g);

    // Net hand change: +3 (draw) -2 (back) = +1 (the copy never lived in hand)
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    // Library: -3 drawn + 2 back = -1
    assert_eq!(g.players[0].library.len(), lib_before - 1);
}

// Cheerful Osteomancer // Raise Dead — return creature card from gy.
#[test]
fn cheerful_osteomancer_prepare_spell_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::cheerful_osteomancer());
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_size = g.players[0].hand.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bear_in_gy)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Raise Dead castable for {B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_size + 1, "Bear returned to hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear_in_gy),
        "Bear should be back in hand");
}

// A prepare-spell cast that suspends mid-way (CR 601.2g float-spend
// confirmation) must not unprepare the creature early, and the resumed
// cast — which replays as a plain `CastSpell` of the copy — must still
// get the token flag + unprepare bookkeeping.
#[test]
fn prepare_spell_survives_float_spend_suspension() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    // Regrowth ({1}{G}) off a prepared Emeritus of Abundance.
    let id = prepared_on_battlefield(&mut g, 0, catalog::emeritus_of_abundance());
    // Off-colour float the generic pip could consume, plus untapped
    // Forests that could pay instead — the CR 601.2g confirmation must
    // ask before auto-spending the float.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    let hand_size = g.players[0].hand.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bear_in_gy)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast suspends for the float-spend confirmation");
    let pd = g.pending_decision.as_ref().expect("a float-spend confirmation is pending");
    assert!(matches!(pd.decision, crabomination::decision::Decision::OptionalTrigger { .. }));
    // Suspended — the creature must still be prepared and nothing on the stack.
    assert!(g.stack.is_empty(), "copy is parked in hand during the suspension");
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::Prepared), 1,
        "creature stays prepared until the copy actually hits the stack");

    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(true)))
        .expect("confirm spending the float");
    // The copy is now a real cast: creature unprepared, copy flagged token.
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::Prepared), 0,
        "casting the copy unprepares the creature");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bear_in_gy),
        "Regrowth resolved — Bear back in hand");
    // The resolved copy ceases to exist: not in hand, not in the graveyard.
    assert_eq!(g.players[0].hand.len(), hand_size + 1,
        "only the Bear was added to hand — the copy is gone");
    assert!(!g.players[0].graveyard.iter().any(|c| c.definition.name == "Regrowth"),
        "the copy never reaches the graveyard");
    assert!(g.pending_prepare_copies.is_empty(), "registration settled");
}

// CR 707.10a — a countered prepare copy ceases to exist without ever
// transiting the graveyard (no CardPutIntoGraveyard / descend bookkeeping).
#[test]
fn countered_prepare_copy_never_touches_the_graveyard() {
    let mut g = two_player_game();
    let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::cheerful_osteomancer());
    g.players[0].mana_pool.add(Color::Black, 1);
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bear_in_gy)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Raise Dead castable for {B}");
    let copy_id = match g.stack.last().expect("copy on the stack") {
        StackItem::Spell { card, .. } => card.id,
        _ => panic!("top of stack should be the prepare copy"),
    };
    let gy_count_before = g.players[0].cards_to_graveyard_this_turn;

    // Hand priority to the opponent so they can respond with the counter.
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(copy_id)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Counterspell castable for {U}{U}");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear_in_gy),
        "Bear stays in the graveyard (Raise Dead was countered)");
    assert!(g.players[0].graveyard.iter().all(|c| c.definition.name != "Raise Dead"),
        "the countered copy ceases to exist — it never enters the graveyard");
    assert_eq!(g.players[0].cards_to_graveyard_this_turn, gy_count_before,
        "no graveyard bookkeeping fires for the countered copy");
    assert!(!g.players[0].descended_this_turn,
        "a countered copy does not count as descending");
}

// Emeritus of Woe // Demonic Tutor — search library for any card to hand.
#[test]
fn emeritus_of_woe_prepare_spell_searches_library() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let id = prepared_on_battlefield(&mut g, 0, catalog::emeritus_of_woe());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bolt)),
    ]));

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Demonic Tutor castable for {1}{B}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt fetched to hand");
}

// Scheming Silvertongue // Sign in Blood.
#[test]
fn scheming_silvertongue_prepare_spell_draws_two_and_drains_two() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let id = prepared_on_battlefield(&mut g, 0, catalog::scheming_silvertongue());
    g.players[0].mana_pool.add(Color::Black, 2);
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Sign in Blood castable for {B}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 2);
    assert_eq!(g.players[0].life, life_before - 2);
}

// Emeritus of Conflict // Lightning Bolt — 3 damage to any target.
#[test]
fn emeritus_of_conflict_prepare_spell_burns_three() {
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::emeritus_of_conflict());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 3);
}

// Goblin Glasswright // Craft with Pride — create a Treasure token.
#[test]
fn goblin_glasswright_prepare_spell_creates_treasure() {
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::goblin_glasswright());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Craft with Pride castable for {R}");
    drain_stack(&mut g);

    let treasure = g.battlefield.iter().find(|c| c.definition.name == "Treasure")
        .expect("Craft with Pride mints a Treasure token");
    assert_eq!(treasure.controller, 0);
}

// Emeritus of Abundance // Regrowth — return any card from gy.
#[test]
fn emeritus_of_abundance_prepare_spell_returns_any_card_from_graveyard() {
    let mut g = two_player_game();
    let bolt_in_gy = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = prepared_on_battlefield(&mut g, 0, catalog::emeritus_of_abundance());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_size = g.players[0].hand.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bolt_in_gy)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Regrowth castable for {1}{G}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_size + 1, "Bolt returned to hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_in_gy),
        "Bolt should be back in hand");
}

// Vastlands Scavenger // Bind to Life — mill 7, then put a creature card
// FROM AMONG THE MILLED SEVEN onto the battlefield (audit fix: a
// pre-existing graveyard creature is NOT eligible).
#[test]
fn vastlands_scavenger_prepare_spell_mills_seven_and_reanimates() {
    let mut g = two_player_game();
    // Library top-7 contains one creature among spells.
    g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    // A pre-existing graveyard creature that must NOT be picked.
    let old = g.add_card_to_graveyard(0, catalog::serra_angel());
    let id = prepared_on_battlefield(&mut g, 0, catalog::vastlands_scavenger());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bind to Life castable for {4}{G}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].library.len(), lib_before - 7, "milled 7 cards");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the milled creature was put onto the battlefield");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == old),
        "a pre-existing graveyard creature is not eligible");
}

// Adventurous Eater // Have a Bite — +1/+1 counter + gain 1 life.
#[test]
fn adventurous_eater_prepare_spell_adds_counter_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::adventurous_eater());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Have a Bite castable for {B}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear alive");
    assert_eq!(v.power, 3, "Bear got a +1/+1 counter");
    assert_eq!(v.toughness, 3);
    assert_eq!(g.players[0].life, life_before + 1, "caster gains 1 life");
}

// Leech Collector // Bloodletting — each opponent loses 2 (no lifegain).
#[test]
fn leech_collector_prepare_spell_opponents_lose_two() {
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::leech_collector());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before_p1 = g.players[1].life;
    let life_before_p0 = g.players[0].life;

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bloodletting castable for {B}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before_p1 - 2, "each opponent loses 2");
    assert_eq!(g.players[0].life, life_before_p0, "Bloodletting grants NO lifegain");
}

// Spellbook Seeker // Careful Study — draw 2, discard 2.
#[test]
fn spellbook_seeker_prepare_spell_loots_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::spellbook_seeker());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();
    let gy_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Careful Study castable for {U}");
    drain_stack(&mut g);

    // Draw 2, discard 2 → hand unchanged, library -2, graveyard +2.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[0].library.len(), lib_before - 2);
    assert_eq!(g.players[0].graveyard.len(), gy_before + 2,
        "two discarded cards land in the graveyard (the copy itself does not)");
}

// Skycoach Conductor // All Aboard — flicker a non-Pilot creature you control.
#[test]
fn skycoach_conductor_prepare_spell_flickers_own_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::skycoach_conductor());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("All Aboard castable for {U}");
    drain_stack(&mut g);

    // Exile + return: the bear is back on the battlefield (a flicker, not
    // a bounce) — not in hand, not stuck in exile.
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"
            && c.controller == 0),
        "All Aboard flickers the creature back to the battlefield");
    assert!(!g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "not a bounce — the creature must not be in hand");
    assert!(!g.exile.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the creature does not linger in exile");
}

// Landscape Painter // Vibrant Idea — draw 2.
#[test]
fn landscape_painter_prepare_spell_draws_two() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    let id = prepared_on_battlefield(&mut g, 0, catalog::landscape_painter());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Vibrant Idea castable for {4}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 2, "Vibrant Idea draws two");
}

// Blazing Firesinger // Seething Song — add {R}{R}{R}{R}{R}.
#[test]
fn blazing_firesinger_prepare_spell_adds_five_red_mana() {
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::blazing_firesinger());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Seething Song castable for {2}{R}");
    drain_stack(&mut g);

    let red_mana = g.players[0].mana_pool.amount(Color::Red);
    assert!(red_mana >= 5, "Should have 5+ red mana after Seething Song; got {}", red_mana);
}

// Maelstrom Artisan // Rocket Volley — destroy target nonbasic land.
#[test]
fn maelstrom_artisan_prepare_spell_destroys_nonbasic_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::fields_of_strife());
    let id = prepared_on_battlefield(&mut g, 0, catalog::maelstrom_artisan());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: Some(Target::Permanent(land)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Rocket Volley castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == land),
        "Rocket Volley destroys the nonbasic land");
}

// Maelstrom Artisan // Rocket Volley — can't target a basic land.
#[test]
fn maelstrom_artisan_prepare_spell_rejects_basic_land_target() {
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let id = prepared_on_battlefield(&mut g, 0, catalog::maelstrom_artisan());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: Some(Target::Permanent(forest)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "Rocket Volley only targets NONBASIC lands");
}

// Scathing Shadelock // Venomous Words — your creature +2/+0 + deathtouch EOT.
#[test]
fn scathing_shadelock_prepare_spell_pumps_and_grants_deathtouch() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::scathing_shadelock());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Venomous Words castable for {B}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear alive");
    assert_eq!(v.power, 2 + 2, "Venomous Words grants +2/+0");
    assert_eq!(v.toughness, 2, "toughness unchanged");
    assert!(v.keywords.contains(&Keyword::Deathtouch), "Bear gains deathtouch EOT");
}

// Infirmary Healer // Stream of Life — target player gains X life.
#[test]
fn infirmary_healer_prepare_spell_gains_x_life() {
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::infirmary_healer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(5);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: Some(5),
    })
    .expect("Stream of Life castable for {X=5}{G}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 5);
}

// Jadzi // Oracle's Gift — create X Fractals with X +1/+1 counters each.
//
// Regression guard for the `Selector::LastCreatedTokens` fan-out: a
// per-Fractal ForEach let the SBA sweep the still-0/0 stragglers
// between iterations (X=2 yielded ONE 2/2 Fractal instead of two), so
// the counters must land on the whole freshly-minted batch in one shot.
#[test]
fn jadzi_prepare_spell_creates_x_fractals_with_x_counters() {
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::jadzi_steward_of_fate());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4); // X=2 → {X}{X} = 4 generic

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Oracle's Gift castable for {X=2}{X=2}{U}");
    drain_stack(&mut g);

    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Fractal")
        .collect();
    assert_eq!(fractals.len(), 2, "X=2 mints two Fractal tokens");
    for f in &fractals {
        assert_eq!(f.counter_count(CounterType::PlusOnePlusOne), 2,
            "each Fractal carries X (=2) +1/+1 counters");
    }
}

// Sanar // Wild Idea — tutor an instant or sorcery to hand.
#[test]
fn sanar_prepare_spell_tutors_instant_or_sorcery() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let id = prepared_on_battlefield(&mut g, 0, catalog::sanar_unfinished_genius());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bolt)),
    ]));

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Wild Idea castable for {3}{U}{R}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Wild Idea fetched the instant to hand");
}

// Tam // Deep Sight — draw 1 + gain 1 life.
#[test]
fn tam_prepare_spell_draws_one_and_gains_one() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    let id = prepared_on_battlefield(&mut g, 0, catalog::tam_observant_sequencer());
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Deep Sight castable for {G}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1, "Deep Sight draws one");
    assert_eq!(g.players[0].life, life_before + 1, "Deep Sight gains 1 life");
}

// Kirol // Pack a Punch — mill 1, two +1/+1 counters + trample EOT.
#[test]
fn kirol_prepare_spell_mills_one_and_packs_a_punch() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::kirol_history_buff());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Pack a Punch castable for {1}{R}{W}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].library.len(), lib_before - 1, "milled 1 card");
    let b = g.battlefield_find(bear).expect("Bear alive");
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 2,
        "two +1/+1 counters on target creature");
    let v = g.computed_permanent(bear).unwrap();
    assert!(v.keywords.contains(&Keyword::Trample), "Bear gains trample EOT");
}

// Abigale // Heroic Stanza — put a +1/+1 counter on target creature.
#[test]
fn abigale_prepare_spell_adds_plus_one_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::abigale_poet_laureate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Heroic Stanza castable for {1}{W/B} paying black");
    drain_stack(&mut g);

    let b = g.battlefield_find(bear).expect("Bear alive");
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1,
        "Heroic Stanza puts a +1/+1 counter on the target");
}

// GameEvent::SpellCast.face audit log.
#[test]
fn cast_spell_emits_front_face_event() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let events = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    let face = events.iter().find_map(|e| match e {
        GameEvent::SpellCast { face, .. } => Some(*face),
        _ => None,
    }).expect("SpellCast event");
    assert_eq!(face, CastFace::Front);
}

#[test]
fn cast_spell_back_emits_back_face_event() {
    // Uses a real MDFC (Pestilent Cauldron // Restorative Burst) — SOS
    // preparation cards no longer carry a back face.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pestilent_cauldron());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    let events = g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Restorative Burst castable for {3}{G}{G}");
    let face = events.iter().find_map(|e| match e {
        GameEvent::SpellCast { face, .. } => Some(*face),
        _ => None,
    }).expect("SpellCast event");
    assert_eq!(face, CastFace::Back, "Back-face cast should be tagged Back");
}

// Per-spell-type tallies.
#[test]
fn instants_or_sorceries_cast_tally_bumps_only_for_is_casts() {
    let mut g = two_player_game();
    // Cast a creature (Grizzly Bears) — does NOT bump the IS tally.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert_eq!(g.players[0].instants_or_sorceries_cast_this_turn, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bear castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].instants_or_sorceries_cast_this_turn, 0,
        "Creature spell should NOT bump IS tally");
    assert_eq!(g.players[0].creatures_cast_this_turn, 1,
        "Creature spell SHOULD bump creature tally");

    // Cast Lightning Bolt — bumps the IS tally.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].instants_or_sorceries_cast_this_turn, 1,
        "Instant cast bumps IS tally");
    assert_eq!(g.players[0].creatures_cast_this_turn, 1,
        "Instant cast does NOT bump creature tally");
}

#[test]
fn potioners_trove_lifegain_rejects_after_creature_cast_only() {
    // `InstantsOrSorceriesCastThisTurnAtLeast` should NOT trip on a turn
    // where only creatures were cast.
    let mut g = two_player_game();
    let trove = g.add_card_to_battlefield(0, catalog::potioners_trove());
    drain_stack(&mut g);
    // Simulate having cast a creature this turn (no IS spells).
    g.players[0].spells_cast_this_turn = 1;
    g.players[0].creatures_cast_this_turn = 1;
    g.players[0].instants_or_sorceries_cast_this_turn = 0;

    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: trove, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None });
    assert!(matches!(result, Err(GameError::AbilityConditionNotMet)),
        "Should reject without IS cast (got {:?})", result);
}

// Pigment Wrangler // Striking Palette — copy your next instant/sorcery
// this turn.
#[test]
fn pigment_wrangler_prepare_spell_copies_next_instant_or_sorcery() {
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::pigment_wrangler());
    g.players[0].mana_pool.add(Color::Red, 2);
    let life_before = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Striking Palette castable for {R}");
    drain_stack(&mut g);

    // The next instant this turn gets copied: Bolt deals 3 + 3 via copy.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 6,
        "Striking Palette copied the Bolt (3 + 3 damage)");
}

