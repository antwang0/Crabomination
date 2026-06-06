use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


/// Dueling Coach's activation puts a +1/+1 counter on each of the
/// controller's creatures with a +1/+1 counter (including itself if it
/// has one — Mr Coach is 1/2 with no counters by default, so it's only
/// gated to creatures that already have a counter).
#[test]
fn dueling_coach_activation_doubles_counters() {
    let mut g = two_player_game();
    let coach = g.add_card_to_battlefield(0, catalog::dueling_coach());
    g.clear_sickness(coach);
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Seed counter on bear1 only — bear2 should NOT get a counter from
    // the activation (gate is "creatures you control with +1/+1
    // counter").
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear1) {
        c.counters.insert(CounterType::PlusOnePlusOne, 1);
    }
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::ActivateAbility {
        card_id: coach,
        ability_index: 0,
        target: None, x_value: None })
    .expect("Dueling Coach activation works for {2}{W}");
    drain_stack(&mut g);

    let bear1_c = g.battlefield.iter().find(|c| c.id == bear1).unwrap();
    let bear2_c = g.battlefield.iter().find(|c| c.id == bear2).unwrap();
    assert_eq!(bear1_c.counter_count(CounterType::PlusOnePlusOne), 2,
        "bear1 should get a counter (had a counter to begin with)");
    assert_eq!(bear2_c.counter_count(CounterType::PlusOnePlusOne), 0,
        "bear2 should NOT get a counter (had no +1/+1 to begin with)");
}

// ── Increasing Vengeance ────────────────────────────────────────────────────

#[test]
fn increasing_vengeance_copies_target_instant() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Cast Lightning Bolt at bear.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");

    // Now cast Increasing Vengeance copying the bolt.
    let iv = g.add_card_to_hand(0, catalog::increasing_vengeance());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: iv,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Increasing Vengeance castable for {R}{R}");
    drain_stack(&mut g);

    // Bear took 3 dmg from bolt + 3 dmg from the IV copy of the bolt = 6 dmg.
    // 2-toughness bear is destroyed by either bolt alone.
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear destroyed");
    // P1 (target's controller) takes either 3 dmg from bolt or unchanged.
    // The bear was originally controlled by P1, so the bolt damage went
    // to the bear, not the player.
}

/// Increasing Vengeance's "if cast from a graveyard, copy that spell
/// twice instead" rider. The printed card has an implicit "exile from
/// anywhere" replacement so the only way to cast it from a graveyard
/// is via a granted Flashback (Past in Flames, Yawgmoth's Will-style
/// effect). We synthesize the scenario by adding a Flashback {R}{R}
/// cost to the card definition for the test, then casting via
/// `GameAction::CastFlashback`. The cast_from_hand flag is false for
/// flashback casts → `Predicate::CastFromGraveyard` evaluates true →
/// the `If` branch runs `CopySpell { count: 2 }`.
#[test]
fn increasing_vengeance_double_copies_when_flashed_back_from_graveyard() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Cast Lightning Bolt at bear (the spell to be copied).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");

    // Put a synthetic Increasing Vengeance (with Flashback {R}{R}) into
    // P0's graveyard. Bolt is on the stack above.
    let mut iv_def = catalog::increasing_vengeance();
    iv_def.keywords.push(Keyword::Flashback(crate::mana::cost(&[
        crate::mana::r(),
        crate::mana::r(),
    ])));
    let iv = g.add_card_to_graveyard(0, iv_def);

    // Pay the {R}{R} flashback cost.
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastFlashback {
        card_id: iv,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Increasing Vengeance castable via Flashback for {R}{R}");
    drain_stack(&mut g);

    // Bolt + TWO copies of Bolt = 9 damage total (each deals 3 to the
    // bear). The bear (2 toughness) dies from any single bolt.
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed");

    // The IV card should end up in exile (flashback resolution exiles
    // the cast card).
    assert!(g.exile.iter().any(|c| c.id == iv),
        "Increasing Vengeance should be exiled after flashback resolves");
}

// ── Push (modern_decks) NEW STX cards ────────────────────────────────────

// ── Spined Karok ────────────────────────────────────────────────────────

#[test]
fn spined_karok_etb_lands_counter_on_friendly() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::spined_karok());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Spined Karok castable for {2}{G}{U}");
    drain_stack(&mut g);

    let bear_card = g.battlefield_find(bear).expect("bear still on bf");
    assert_eq!(bear_card.counter_count(CounterType::PlusOnePlusOne), 1);
}

// ── Inspiring Veteran ───────────────────────────────────────────────────

#[test]
fn inspiring_veteran_buffs_other_friendly_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let veteran = g.add_card_to_battlefield(0, catalog::inspiring_veteran());

    let bear_view = g.computed_permanent(bear).expect("bear on bf");
    let vet_view = g.computed_permanent(veteran).expect("veteran on bf");
    assert_eq!(bear_view.power, 3, "bear should be 2+1=3 power from anthem");
    assert_eq!(bear_view.toughness, 3, "bear should be 2+1=3 toughness from anthem");
    // The Veteran itself should be unaffected (OtherThanSource).
    assert_eq!(vet_view.power, 2, "Veteran should not buff itself");
    assert_eq!(vet_view.toughness, 2, "Veteran should not buff itself");
}

#[test]
fn inspiring_veteran_anthem_expires_when_it_leaves_play() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let veteran = g.add_card_to_battlefield(0, catalog::inspiring_veteran());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);

    // Remove veteran.
    let vet_card = g.battlefield.iter().position(|c| c.id == veteran).unwrap();
    let card = g.battlefield.remove(vet_card);
    g.players[0].graveyard.push(card);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2,
        "bear should be back to 2 after veteran leaves");
}

// ── Snipe ───────────────────────────────────────────────────────────────

#[test]
fn snipe_deals_two_to_creature_without_cantrip() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::snipe());
    let hand_before = g.players[0].hand.len();
    // Library has enough cards.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Snipe castable for {U}{R}");
    drain_stack(&mut g);

    // Bear 2/2 takes 2 damage and dies.
    let _ = g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed by 2 damage");

    // No cantrip: spells_cast_this_turn = 1 (only Snipe), gate is "≥2".
    // So hand size should be unchanged (cast Snipe = -1, no draw = 0 net).
    assert_eq!(g.players[0].hand.len(), hand_before - 1,
        "no cantrip on first spell");
}

#[test]
fn snipe_cantrips_on_second_spell_cast() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Library has cards to draw.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    // Fake a prior spell having been cast this turn.
    g.players[0].spells_cast_this_turn = 1;
    g.spells_cast_this_turn = 1;

    let id = g.add_card_to_hand(0, catalog::snipe());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Snipe castable for {U}{R}");
    drain_stack(&mut g);

    let _ = g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed");
    // Cast Snipe = -1, draw = +1, net 0.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "cantrip fires on second spell");
}

// ── Witherbloom Pest Eater ──────────────────────────────────────────────

#[test]
fn witherbloom_pest_eater_etb_creates_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pest_eater());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);

    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Witherbloom Pest Eater castable for {3}{B}{G}");
    drain_stack(&mut g);

    // Eater enters + Pest token enters = +2 permanents.
    assert_eq!(g.battlefield.len(), bf_before + 2);
    let pest = g.battlefield.iter().find(|c|
        c.definition.has_creature_type(crate::card::CreatureType::Pest)
            && c.id != id
    ).expect("Pest token should be created");
    assert_eq!(pest.controller, 0, "Pest controlled by you");
}

#[test]
fn witherbloom_pest_eater_grows_when_another_pest_dies() {
    let mut g = two_player_game();
    let eater = g.add_card_to_battlefield(0, catalog::witherbloom_pest_eater());
    g.clear_sickness(eater);
    let pest = g.add_card_to_battlefield(0, catalog::eyetwitch());
    g.clear_sickness(pest);

    let p_before = g.battlefield_find(eater).unwrap().power();
    let t_before = g.battlefield_find(eater).unwrap().toughness();

    // Kill the pest.
    let pos = g.battlefield.iter().position(|c| c.id == pest).unwrap();
    let card = g.battlefield.remove(pos);
    g.players[0].graveyard.push(card);
    // Fire the death-trigger manually (the SBA loop would normally have
    // emitted this).
    use crate::game::types::GameEvent;
    let events = vec![GameEvent::CreatureDied { card_id: pest }];
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);

    let p_after = g.battlefield_find(eater).unwrap().power();
    let t_after = g.battlefield_find(eater).unwrap().toughness();
    assert_eq!(p_after, p_before + 1, "Eater should pump +1 power on Pest death");
    assert_eq!(t_after, t_before + 1, "Eater should pump +1 toughness on Pest death");
}

// ── Inkmoth Initiate ────────────────────────────────────────────────────

#[test]
fn inkmoth_initiate_etb_shrinks_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let id = g.add_card_to_hand(0, catalog::inkmoth_initiate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Inkmoth Initiate castable for {W}{B}");
    drain_stack(&mut g);

    // Bear 2/2 becomes 1/1 EOT.
    let bear_view = g.battlefield_find(bear).expect("bear still alive");
    assert_eq!(bear_view.toughness(), 1, "bear shrunk to 1 toughness");
    assert_eq!(bear_view.power(), 1, "bear shrunk to 1 power");
}

// ── Stoic Tutelage ──────────────────────────────────────────────────────

#[test]
fn stoic_tutelage_draws_two_and_drains_each_opp() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::stoic_tutelage());
    let hand_before = g.players[0].hand.len();
    let opp_life_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Stoic Tutelage castable for {3}{W}");
    drain_stack(&mut g);

    // -1 (cast) +2 (draw) = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert_eq!(g.players[1].life, opp_life_before - 1, "opp loses 1 life");
}

// ── Lorehold Recovery ───────────────────────────────────────────────────

#[test]
fn lorehold_recovery_reanimates_with_haste() {
    let mut g = two_player_game();
    let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_recovery());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear_in_gy)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lorehold Recovery castable for {2}{R}{W}");
    drain_stack(&mut g);

    let bear_card = g.battlefield_find(bear_in_gy).expect("bear reanimated");
    assert!(bear_card.has_keyword(&Keyword::Haste),
        "reanimated bear should have haste EOT");
}

// ── Quandrix Surge ──────────────────────────────────────────────────────

#[test]
fn quandrix_surge_doubles_each_creatures_counters() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Seed counters: bear1 has 2, bear2 has 3.
    if let Some(c) = g.battlefield_find_mut(bear1) {
        c.counters.insert(CounterType::PlusOnePlusOne, 2);
    }
    if let Some(c) = g.battlefield_find_mut(bear2) {
        c.counters.insert(CounterType::PlusOnePlusOne, 3);
    }

    let id = g.add_card_to_hand(0, catalog::quandrix_surge());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Quandrix Surge castable for {1}{G}{U}");
    drain_stack(&mut g);

    let b1 = g.battlefield_find(bear1).unwrap();
    let b2 = g.battlefield_find(bear2).unwrap();
    assert_eq!(b1.counter_count(CounterType::PlusOnePlusOne), 4,
        "bear1 counters doubled from 2 to 4");
    assert_eq!(b2.counter_count(CounterType::PlusOnePlusOne), 6,
        "bear2 counters doubled from 3 to 6");
}

#[test]
fn quandrix_surge_noop_on_counterless_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_surge());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Quandrix Surge castable");
    drain_stack(&mut g);

    let bear_view = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_view.counter_count(CounterType::PlusOnePlusOne), 0,
        "no counters to double → no counters land");
}

// ── Magecraft Insight ───────────────────────────────────────────────────

#[test]
fn magecraft_insight_draws_two_cards() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::magecraft_insight());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Magecraft Insight castable for {2}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "draws 2, casts 1 = net +1");
}

// ── Sparkmage's Mantra ──────────────────────────────────────────────────

#[test]
fn sparkmages_mantra_pings_and_scrys() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::sparkmages_mantra());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Sparkmage's Mantra castable for {R}");
    drain_stack(&mut g);

    // Bear takes 1 damage; 2-toughness, still alive.
    let bear_view = g.battlefield_find(bear).expect("bear still alive after 1 dmg");
    assert_eq!(bear_view.damage, 1, "bear should have 1 damage");
}

#[test]
fn sparkmages_mantra_can_target_player() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::sparkmages_mantra());
    let life_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Sparkmage's Mantra castable for {R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 1, "opp took 1 damage");
}

// ── Witherbloom Drainage ────────────────────────────────────────────────

#[test]
fn witherbloom_drainage_drains_each_opp_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_drainage());
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Witherbloom Drainage castable for {1}{B}{G}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, p1_before - 2, "opp loses 2 life");
    assert_eq!(g.players[0].life, p0_before + 2, "you gain 2 life");
}

// ── Mizzium Mortars (STA reprint, RTR) ─────────────────────────────────────

#[test]
fn mizzium_mortars_burns_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mizzium_mortars());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Mizzium Mortars castable for {1}{R}");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Bear destroyed by 4 damage from Mizzium Mortars"
    );
}

// ── Electrolyze (STA reprint, Guildpact) ───────────────────────────────────

#[test]
fn electrolyze_deals_two_damage_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::electrolyze());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Electrolyze castable for {1}{U}{R}");
    drain_stack(&mut g);

    // Bear took 2 damage — a 2/2 dies to 2.
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Bear destroyed by 2 damage from Electrolyze"
    );
    // Caster drew a card: hand went from before+1(electrolyze) → before
    // (cast removed) +1 (draw) = before+0. We need to net -1 (cast) +1
    // (draw) = +0 vs hand_before which includes the card itself.
    assert_eq!(
        g.players[0].hand.len(),
        hand_before,
        "Electrolyze trades the cast card for a draw"
    );
}

#[test]
fn electrolyze_targets_a_player_for_two_damage() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::electrolyze());
    let p1_life_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Electrolyze targetable at a player");
    drain_stack(&mut g);

    assert_eq!(
        g.players[1].life,
        p1_life_before - 2,
        "Opp loses 2 life from Electrolyze"
    );
}

// ── Show of Aggression (STX 2021) ──────────────────────────────────────────

#[test]
fn show_of_aggression_pumps_each_friendly_creature_and_grants_haste() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::show_of_aggression());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Show of Aggression castable for {2}{R}{R}");
    drain_stack(&mut g);

    // Both friendly bears now 4/2 with haste.
    for bid in [bear1, bear2] {
        let bv = g.battlefield_find(bid).expect("bear still alive");
        let cv = g.computed_permanent(bid).expect("bear computed");
        assert_eq!(
            cv.power, 4,
            "bear {:?} should be 4/2 (+2/+0 from Show of Aggression), got P/T {}/{}",
            bid, bv.power(), bv.toughness()
        );
        assert!(
            cv.keywords.contains(&Keyword::Haste),
            "bear {:?} should have Haste from Show of Aggression",
            bid
        );
    }
    // Opp bear unchanged.
    let opp = g.computed_permanent(opp_bear).expect("opp bear computed");
    assert_eq!(opp.power, 2, "opp bear stays 2/2");
    assert!(!opp.keywords.contains(&Keyword::Haste), "no haste for opp");
}

// ── Past in Flames (STA reprint, Innistrad) ────────────────────────────────

#[test]
fn past_in_flames_returns_instants_and_sorceries_from_graveyard_to_hand() {
    let mut g = two_player_game();
    // Two instants in graveyard.
    let _bolt1 = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let _bolt2 = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    // A non-IS card (creature) in gy — should NOT come back.
    let _bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::past_in_flames());
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Past in Flames castable for {3}{R}");
    drain_stack(&mut g);

    // -1 (cast) + 2 (bolts returned) = +1 vs hand_before.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    // gy: -2 (bolts left) +1 (Past in Flames went to gy after resolving) = -1.
    assert_eq!(g.players[0].graveyard.len(), gy_before - 1);
    // Verify bear stayed in graveyard.
    let bear_in_zone = g
        .players[0]
        .graveyard
        .iter()
        .any(|c| c.definition.name == "Grizzly Bears");
    assert!(bear_in_zone, "Grizzly Bears (non-IS) stays in graveyard");
}

// ── Inspired Idea ──────────────────────────────────────────────────────────

#[test]
fn inspired_idea_draws_three_then_stacks_two_on_top() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::inspired_idea());
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Inspired Idea castable for {1}{U}{U}");
    drain_stack(&mut g);

    // Hand: -1 (cast) +3 (draw) -2 (top of library) = +0 vs hand_before.
    assert_eq!(
        g.players[0].hand.len(),
        hand_before,
        "Net hand: -1 cast + 3 draws - 2 to top of library"
    );
    // Library: -3 (drawn) +2 (returned) = -1 vs lib_before.
    assert_eq!(g.players[0].library.len(), lib_before - 1);
}

// ── Resurgent Belief ───────────────────────────────────────────────────────

#[test]
fn resurgent_belief_returns_each_enchantment_from_graveyard() {
    let mut g = two_player_game();
    // Two enchantment cards in graveyard.
    // Living History is enchantment with an ETB Spirit-token trigger.
    let _ench1 = g.add_card_to_graveyard(0, catalog::living_history());
    let _ench2 = g.add_card_to_graveyard(0, catalog::living_history());
    // A non-enchantment in gy.
    let _bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::resurgent_belief());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);

    let gy_size_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Resurgent Belief castable for {3}{W}");
    drain_stack(&mut g);

    // Both enchantments left graveyard (Living History each has an ETB
    // Spirit token rider, so the battlefield can grow by 2 to 4). We
    // assert via graveyard delta: gy lost 2 enchantments and gained
    // Resurgent Belief itself (sorcery), net -1.
    assert_eq!(
        g.players[0].graveyard.len(),
        gy_size_before - 1,
        "gy size: -2 enchantments + 1 sorcery (Resurgent Belief) = -1"
    );
    // Bear stayed in graveyard.
    assert!(
        g.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition.name == "Grizzly Bears"),
        "Grizzly Bears (non-enchantment) stays in graveyard"
    );
    // Both enchantments are on the battlefield now.
    let ench_on_bf = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Living History")
        .count();
    assert_eq!(ench_on_bf, 2, "both Living History copies returned to bf");
}

// ── Academic Dispute ───────────────────────────────────────────────────────

/// Academic Dispute now reads "Target creature gets +2/+0 and gains
/// reach until end of turn" (the printed STX oracle, not the prior
/// Fight body the engine had approximated). This test pins the
/// current effect — the bear gets +2/+0 and reach EOT.
#[test]
fn academic_dispute_pumps_friendly_and_grants_reach() {
    let mut g = two_player_game();
    let friendly = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(friendly);
    let id = g.add_card_to_hand(0, catalog::academic_dispute());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(friendly)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Academic Dispute castable for {R}");
    drain_stack(&mut g);

    // Bear is still alive (no fight); gained Reach EOT.
    let bear = g.battlefield_find(friendly).expect("bear still alive");
    assert!(bear.has_keyword(&Keyword::Reach), "Academic Dispute grants Reach EOT");
}

// ── Enthusiastic Study ─────────────────────────────────────────────────────

#[test]
fn enthusiastic_study_pumps_target_creature_and_grants_trample_after_second_spell() {
    // After casting Enthusiastic Study as the second spell of the turn,
    // the +2/+2 lands AND the trample rider fires.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].spells_cast_this_turn = 1; // Pretend we already cast something
    let id = g.add_card_to_hand(0, catalog::enthusiastic_study());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Enthusiastic Study castable for {1}{G}");
    drain_stack(&mut g);

    let cv = g.computed_permanent(bear).expect("bear computed");
    assert_eq!(cv.power, 4, "bear pumped to 4/4 (+2/+2)");
    assert_eq!(cv.toughness, 4, "bear pumped to 4/4 (+2/+2)");
    assert!(
        cv.keywords.contains(&Keyword::Trample),
        "trample granted (second spell this turn)"
    );
}

#[test]
fn enthusiastic_study_skips_trample_on_first_spell_this_turn() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // spells_cast_this_turn = 0 — Enthusiastic Study is the first spell.
    let id = g.add_card_to_hand(0, catalog::enthusiastic_study());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Enthusiastic Study castable for {1}{G}");
    drain_stack(&mut g);

    let cv = g.computed_permanent(bear).expect("bear computed");
    assert_eq!(cv.power, 4, "bear still pumped to 4/4 (+2/+2)");
    assert!(
        !cv.keywords.contains(&Keyword::Trample),
        "no trample on the first spell of the turn"
    );
}

// ── Forked Bolt (STA reprint) ──────────────────────────────────────────────

#[test]
fn forked_bolt_deals_two_damage_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::forked_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Forked Bolt castable for {R}");
    drain_stack(&mut g);

    // 2/2 bear takes 2 damage → dies.
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Grizzly Bears (2/2) should die to Forked Bolt's 2 damage"
    );
}

#[test]
fn forked_bolt_targets_player_for_two_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::forked_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Forked Bolt can target a player");
    drain_stack(&mut g);

    assert_eq!(
        g.players[1].life,
        p1_life_before - 2,
        "Opp loses 2 life from Forked Bolt"
    );
}

// ── Storm's Wrath (STX) ─────────────────────────────────────────────────────

#[test]
fn storms_wrath_destroys_each_creature() {
    let mut g = two_player_game();
    let bear_p0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear_p1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::storms_wrath());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Storm's Wrath castable for {2}{R}{R}");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear_p0),
        "P0 bear should die to 4 damage"
    );
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear_p1),
        "P1 bear should die to 4 damage"
    );
}

// ── Cinderclasm (STX) ──────────────────────────────────────────────────────

#[test]
fn cinderclasm_pings_each_creature_for_one() {
    let mut g = two_player_game();
    // 1/1 token-like creature with toughness 1 dies; 2/2 bear survives.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Add a 1/1-toughness Pest token-like creature on the opp side.
    let pest = g.add_card_to_battlefield(1, catalog::eyetwitch()); // 1/1 Pest
    let id = g.add_card_to_hand(0, catalog::cinderclasm());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Cinderclasm castable for {1}{R}{R}");
    drain_stack(&mut g);

    // 2/2 bear took 1 damage but survives.
    let bv = g.battlefield_find(bear).expect("bear should survive");
    assert_eq!(bv.damage, 1, "bear takes 1 damage from Cinderclasm");
    // 1/1 Pest dies.
    assert!(
        !g.battlefield.iter().any(|c| c.id == pest),
        "1/1 creature dies to Cinderclasm's 1 damage"
    );
}

// ── Cathartic Pyre (STX) ───────────────────────────────────────────────────

#[test]
fn cathartic_pyre_default_mode_burns_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cathartic_pyre());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("Cathartic Pyre castable for {1}{R}");
    drain_stack(&mut g);

    // 2/2 bear takes 3 damage → dies.
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Cathartic Pyre mode 0 deals 3 damage → bear dies"
    );
}

// ── Stern Dismissal (STX) ──────────────────────────────────────────────────

#[test]
fn stern_dismissal_bounces_creature_to_owner_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::stern_dismissal());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let p1_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Stern Dismissal castable for {U}");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "bear bounced off the battlefield"
    );
    assert_eq!(
        g.players[1].hand.len(),
        p1_hand_before + 1,
        "bear returned to owner (P1) hand"
    );
}

// ── Krosan Grip (STA reprint) ──────────────────────────────────────────────

#[test]
fn krosan_grip_destroys_artifact() {
    let mut g = two_player_game();
    let sol = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::krosan_grip());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(sol)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Krosan Grip castable for {2}{G}");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == sol),
        "Sol Ring destroyed by Krosan Grip"
    );
}

// ── Sublime Epiphany (STA reprint) ─────────────────────────────────────────

#[test]
fn sublime_epiphany_resolves_counter_bounce_draw() {
    // Default picks are [0, 2, 4] — counter target spell, bounce nonland,
    // draw a card. Test the draw + bounce halves (no spell on stack so
    // mode 0 is a no-op which the engine handles cleanly).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::sublime_epiphany());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Sublime Epiphany castable for {4}{U}{U}");
    drain_stack(&mut g);

    // Bear was bounced.
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Sublime Epiphany mode 2 bounces bear"
    );
    // Card was drawn (hand_before -1 cast +1 draw = same).
    assert!(
        g.players[0].hand.len() >= hand_before,
        "Sublime Epiphany mode 4 draws a card"
    );
}

// ── Persist (STA reprint) ──────────────────────────────────────────────────

#[test]
fn persist_returns_creature_card_with_minus_one_counter() {
    let mut g = two_player_game();
    let bear_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::persist());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear_gy)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Persist castable for {1}{B}{G}");
    drain_stack(&mut g);

    // The 2/2 bear returns with -1/-1 counter → 1/1 on battlefield.
    let bear = g.battlefield_find(bear_gy).expect("bear back on battlefield");
    assert_eq!(
        bear.counters.get(&CounterType::MinusOneMinusOne).copied().unwrap_or(0),
        1,
        "Persist's bear should have one -1/-1 counter"
    );
    let cv = g.computed_permanent(bear_gy).expect("bear computed");
    assert_eq!(cv.power, 1, "2/2 - 1/1 = 1/1");
    assert_eq!(cv.toughness, 1, "2/2 - 1/1 = 1/1");
}

// ── Bone to Ash (STX) ───────────────────────────────────────────────────────

#[test]
fn bone_to_ash_counters_creature_spell_and_cantrips() {
    let mut g = two_player_game();
    // P1 (active player) casts Grizzly Bears.
    let bear_card = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear_card,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Grizzly Bears castable for {1}{G}");

    // Bears now on the stack — find its CardId.
    let bone_to_ash_target = match g.stack.last().expect("a spell is on the stack") {
        crate::game::types::StackItem::Spell { card, .. } => card.id,
        _ => panic!("expected spell on stack"),
    };

    // P0 responds with Bone to Ash (instant speed counter).
    g.add_card_to_library(0, catalog::island());
    let bta_id = g.add_card_to_hand(0, catalog::bone_to_ash());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    let p0_hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bta_id,
        target: Some(Target::Permanent(bone_to_ash_target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bone to Ash castable for {1}{U}{U}");
    drain_stack(&mut g);

    // Bears were countered → in P1's graveyard.
    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == bear_card),
        "countered bears land in graveyard"
    );
    // P0 drew a card (cast -1 + draw 1 = same hand size).
    assert!(
        g.players[0].hand.len() >= p0_hand_before,
        "Bone to Ash drew a card"
    );
}

// ── Ingenious Mastery (STX) ────────────────────────────────────────────────

#[test]
fn ingenious_mastery_draws_three_stacks_two_and_opp_draws() {
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::ingenious_mastery());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    let p0_hand_before = g.players[0].hand.len();
    let p1_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Ingenious Mastery castable for {3}{U}{U}");
    drain_stack(&mut g);

    // P0 played Ingenious Mastery (-1), drew 3, stacked 2 → net +0
    assert_eq!(
        g.players[0].hand.len(),
        p0_hand_before,
        "P0 hand: -1 cast + 3 draw - 2 stack = +0"
    );
    // P1 drew a card.
    assert_eq!(
        g.players[1].hand.len(),
        p1_hand_before + 1,
        "opp drew a card from Ingenious Mastery"
    );
}

// ── Acolyte of Affliction (STX) ────────────────────────────────────────────

#[test]
fn acolyte_of_affliction_mills_each_player_three() {
    let mut g = two_player_game();
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::acolyte_of_affliction());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    let p0_lib_before = g.players[0].library.len();
    let p1_lib_before = g.players[1].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Acolyte of Affliction castable for {3}{B}{B}");
    drain_stack(&mut g);

    assert_eq!(
        g.players[0].library.len(),
        p0_lib_before - 3,
        "Acolyte ETB mills P0 three cards"
    );
    assert_eq!(
        g.players[1].library.len(),
        p1_lib_before - 3,
        "Acolyte ETB mills P1 three cards"
    );
}

// ── Skywarp Skaab (STX) ────────────────────────────────────────────────────

#[test]
fn skywarp_skaab_etb_declines_by_default() {
    // AutoDecider declines the "you may discard" — Skywarp Skaab just
    // enters as a vanilla 2/3 flier.
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::skywarp_skaab());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    // Hand size before cast — should be -1 cast + 0 discard = -1.
    let p0_hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Skywarp Skaab castable for {1}{U}{U}");
    drain_stack(&mut g);

    // AutoDecider declines: opp bear stays on battlefield, P0 hand size = before - 1 (cast cost).
    assert!(
        g.battlefield.iter().any(|c| c.id == opp_bear),
        "Opp bear stays on battlefield when Skywarp Skaab declines"
    );
    assert_eq!(
        g.players[0].hand.len(),
        p0_hand_before - 1,
        "P0 hand size = before - 1 (cast cost) — no discard since AutoDecider declines"
    );
}

// ── STA reprint tests (push: modern_decks) ─────────────────────────────────

#[test]
fn damnable_pact_at_x_three_draws_three_loses_three() {
    let mut g = two_player_game();
    // Seed P1's library so it can actually draw 3.
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::damnable_pact());
    let p1_hand_before = g.players[1].hand.len();
    let p1_life_before = g.players[1].life;

    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: Some(3),
    })
    .expect("Damnable Pact castable for {X=3}{B}{B}");
    drain_stack(&mut g);

    assert_eq!(
        g.players[1].hand.len() as i32, p1_hand_before as i32 + 3,
        "Target player drew X=3 cards"
    );
    assert_eq!(
        g.players[1].life, p1_life_before - 3,
        "Target player lost X=3 life"
    );
}

#[test]
fn shore_up_untaps_and_grants_hexproof() {
    let mut g = two_player_game();
    // Tap your own bear so we can verify the untap half.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.battlefield_find_mut(bear).unwrap().tapped = true;

    let id = g.add_card_to_hand(0, catalog::shore_up());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Shore Up castable for {U}");
    drain_stack(&mut g);

    let bear_card = g.battlefield_find(bear).unwrap();
    assert!(!bear_card.tapped, "Bear should be untapped");
    let cp = g.computed_permanent(bear).unwrap();
    assert!(
        cp.keywords.contains(&Keyword::Hexproof),
        "Bear should have Hexproof for the turn"
    );
}

#[test]
fn symbol_of_strength_pumps_two_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let id = g.add_card_to_hand(0, catalog::symbol_of_strength());
    let hand_before = g.players[0].hand.len();

    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Symbol of Strength castable for {2}{G}");
    drain_stack(&mut g);

    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "Bear power +2 EOT (2 → 4)");
    assert_eq!(cp.toughness, 4, "Bear toughness +2 EOT (2 → 4)");
    // -1 cast + 1 draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn magmatic_sinkhole_surveils_and_deals_four_damage() {
    let mut g = two_player_game();
    // Seed P0's library so surveil 2 has cards to look at.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    // 4/4 opp creature — Magmatic Sinkhole's 4 damage should kill it.
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(target);
    let id = g.add_card_to_hand(0, catalog::magmatic_sinkhole());

    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Magmatic Sinkhole castable for {1}{B}{R}");
    drain_stack(&mut g);

    // Serra Angel is 4/4 — 4 damage kills it.
    assert!(
        !g.battlefield.iter().any(|c| c.id == target),
        "Serra Angel destroyed by 4 damage"
    );
}

#[test]
fn sevinnes_reclamation_returns_low_mv_permanent_from_graveyard() {
    let mut g = two_player_game();
    // Put a 2-MV creature in P0's graveyard.
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::sevinnes_reclamation());

    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Sevinne's Reclamation castable for {2}{W}");
    drain_stack(&mut g);

    // Bear should now be on the battlefield.
    assert!(
        g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be on battlefield after Sevinne's Reclamation"
    );
    // Graveyard shouldn't still contain it.
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.id == bear),
        "Bear should leave graveyard"
    );
}

#[test]
fn sevinnes_reclamation_rejects_high_mv_target() {
    // MV-cap is 3 — a 4-MV target like Serra Angel should not be a legal target.
    let mut g = two_player_game();
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::sevinnes_reclamation());

    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let res = g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(res.is_err(), "Should reject 4-MV target (cap is 3)");
}

#[test]
fn light_of_promise_adds_counter_on_lifegain_event() {
    // Push (modern_decks): NEW STX card. The printed "Whenever you gain
    // life, put that many +1/+1 counters on target creature you control"
    // trigger places exactly 1 +1/+1 counter for a 1-life event.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let _enc = g.add_card_to_battlefield(0, catalog::light_of_promise());

    g.players[0].life += 1;
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 1 }]);
    drain_stack(&mut g);

    let counters = g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 1,
        "Bear should have exactly one +1/+1 counter from a 1-life event");
}

#[test]
fn light_of_promise_scales_with_lump_sum_lifegain() {
    // Push (modern_decks): the "that many" rider now reads
    // `Value::TriggerEventAmount` (the firing event's amount field). A
    // lump-sum 4-life gain (Bookwurm-style) should place 4 +1/+1
    // counters, not 1. Verifies the new `event_amount` thread through
    // the trigger dispatcher → StackItem::Trigger → EffectContext.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let _enc = g.add_card_to_battlefield(0, catalog::light_of_promise());

    g.players[0].life += 4;
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 4 }]);
    drain_stack(&mut g);

    let counters = g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 4,
        "Bear should scale with the event's amount (4 life → 4 counters)");
}

#[test]
fn filth_in_graveyard_grants_swampwalk_with_swamp() {
    use crate::card::LandType;
    // Filth's graveyard anthem grants swampwalk to your creatures while you
    // control a Swamp (reuses the graveyard_anthem_for_name table + the new
    // Keyword::Landwalk).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bear).unwrap().keywords
        .contains(&Keyword::Landwalk(LandType::Swamp)), "no swampwalk without Filth");
    g.add_card_to_graveyard(0, catalog::filth());
    g.add_card_to_battlefield(0, catalog::swamp());
    assert!(g.computed_permanent(bear).unwrap().keywords
        .contains(&Keyword::Landwalk(LandType::Swamp)),
        "bear gains swampwalk from Filth in gy + Swamp controlled");
}

#[test]
fn anger_in_graveyard_grants_haste_with_mountain() {
    // Push (modern_decks): NEW STA reprint. Anger's graveyard-resident
    // anthem grants Haste to your creatures while you control a Mountain.
    // Verified via the new `graveyard_anthem_for_name` helper-table walk
    // in `GameState::compute_battlefield`.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    // No Anger anywhere — bear has no haste.
    let base = g.computed_permanent(bear).unwrap();
    assert!(!base.keywords.contains(&Keyword::Haste),
        "bear has no haste without Anger in gy");

    // Anger in gy + you control a Mountain → haste granted.
    g.add_card_to_graveyard(0, catalog::anger());
    g.add_card_to_battlefield(0, catalog::mountain());
    let with_anger = g.computed_permanent(bear).unwrap();
    assert!(with_anger.keywords.contains(&Keyword::Haste),
        "bear gains haste from Anger in gy + Mountain controlled");
}

#[test]
fn anger_in_graveyard_requires_mountain_to_grant_haste() {
    // Without a Mountain on the battlefield, Anger's gy-anthem does not
    // fire. The gate is keyed off the gy-resident card's owner's lands.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::anger());
    // Add only Plains, not Mountain.
    g.add_card_to_battlefield(0, catalog::plains());

    let cp = g.computed_permanent(bear).unwrap();
    assert!(!cp.keywords.contains(&Keyword::Haste),
        "Anger anthem requires a Mountain — Plains doesn't trigger it");
}

#[test]
fn anger_only_grants_haste_to_its_owners_creatures() {
    // Anger in P0's graveyard with P0 controlling a Mountain — P0's
    // creatures get Haste, but P1's do not.
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::anger());
    g.add_card_to_battlefield(0, catalog::mountain());
    let my_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let mine = g.computed_permanent(my_bear).unwrap();
    let theirs = g.computed_permanent(opp_bear).unwrap();
    assert!(mine.keywords.contains(&Keyword::Haste), "my bear gets haste");
    assert!(!theirs.keywords.contains(&Keyword::Haste),
        "opp bear does not get haste (not Anger's owner)");
}

// ── Wonder (STA reprint, gy-anthem) ─────────────────────────────────────────

#[test]
fn wonder_in_graveyard_grants_flying_with_island() {
    // Push (modern_decks): NEW STA reprint. Wonder's graveyard-resident
    // anthem grants Flying to your creatures while you control an Island.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let base = g.computed_permanent(bear).unwrap();
    assert!(!base.keywords.contains(&Keyword::Flying),
        "bear has no flying without Wonder in gy");

    g.add_card_to_graveyard(0, catalog::wonder());
    g.add_card_to_battlefield(0, catalog::island());
    let with_wonder = g.computed_permanent(bear).unwrap();
    assert!(with_wonder.keywords.contains(&Keyword::Flying),
        "bear gains flying from Wonder in gy + Island controlled");
}

// ── Brawn (STA reprint, gy-anthem) ──────────────────────────────────────────

#[test]
fn brawn_in_graveyard_grants_trample_with_forest() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let base = g.computed_permanent(bear).unwrap();
    assert!(!base.keywords.contains(&Keyword::Trample),
        "bear has no trample without Brawn in gy");

    g.add_card_to_graveyard(0, catalog::brawn());
    g.add_card_to_battlefield(0, catalog::forest());
    let with_brawn = g.computed_permanent(bear).unwrap();
    assert!(with_brawn.keywords.contains(&Keyword::Trample),
        "bear gains trample from Brawn in gy + Forest controlled");
}

// ── Valor (STA reprint, gy-anthem) ──────────────────────────────────────────

#[test]
fn valor_in_graveyard_grants_first_strike_with_plains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let base = g.computed_permanent(bear).unwrap();
    assert!(!base.keywords.contains(&Keyword::FirstStrike),
        "bear has no first strike without Valor in gy");

    g.add_card_to_graveyard(0, catalog::valor());
    g.add_card_to_battlefield(0, catalog::plains());
    let with_valor = g.computed_permanent(bear).unwrap();
    assert!(with_valor.keywords.contains(&Keyword::FirstStrike),
        "bear gains first strike from Valor in gy + Plains controlled");
}

// ── Triskaidekaphile (STX 2021) ────────────────────────────────────────────

#[test]
fn triskaidekaphile_etb_draws_a_card_and_lifts_max_hand_size() {
    // ETB body: draw 1 + flip Player.no_maximum_hand_size flag.
    // Cast via the spell pipeline so the ETB trigger fires.
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::triskaidekaphile());
    let hand_before = g.players[0].hand.len();
    let max_before = g.players[0].max_hand_size;
    assert_eq!(max_before, Some(7), "default max hand size is seven");

    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Triskaidekaphile castable for {1}{U}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].max_hand_size, None,
        "Triskaidekaphile ETB should remove the maximum hand size");
    // Hand: -1 (cast) + 1 (ETB draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Triskaidekaphile stays on the battlefield");
}

#[test]
fn triskaidekaphile_does_not_win_at_upkeep_with_other_hand_size() {
    // If hand size != 13, the trigger predicate fails and no win fires.
    // Cast via the spell pipeline so the trigger is registered.
    let mut g = two_player_game();
    // Seed libraries so both players don't deck out across the turn loop.
    for _ in 0..100 { g.add_card_to_library(0, catalog::island()); }
    for _ in 0..100 { g.add_card_to_library(1, catalog::island()); }
    for _ in 0..7 { g.add_card_to_hand(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::triskaidekaphile());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Triskaidekaphile castable for {1}{U}{U}");
    drain_stack(&mut g);

    // After ETB, P0 has at most 8 cards in hand. Walk to next upkeep.
    assert!(g.players[0].hand.len() < 13);
    use crate::game::types::TurnStep;
    let mut iters = 0;
    while !(g.active_player_idx == 0 && g.step == TurnStep::Upkeep && g.turn_number >= 2)
        && iters < 200
    {
        let _ = g.pass_priority();
        drain_stack(&mut g);
        iters += 1;
        if g.game_over.is_some() { break; }
    }
    drain_stack(&mut g);

    assert!(g.game_over.is_none(),
        "Triskaidekaphile shouldn't fire on P0's upkeep without 13 cards");
}

#[test]
fn triskaidekaphile_wins_at_upkeep_with_exactly_thirteen_cards() {
    // Cast Triskaidekaphile, then force P0's hand to 13 cards via a
    // direct hand-push before the next upkeep step, and confirm the
    // engine recognises the predicate and wins the game.
    let mut g = two_player_game();
    for _ in 0..100 { g.add_card_to_library(0, catalog::island()); }
    for _ in 0..100 { g.add_card_to_library(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::triskaidekaphile());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Triskaidekaphile castable for {1}{U}{U}");
    drain_stack(&mut g);

    // Top up P0's hand to exactly 13.
    while g.players[0].hand.len() < 13 {
        g.add_card_to_hand(0, catalog::island());
    }
    assert_eq!(g.players[0].hand.len(), 13);

    use crate::game::types::TurnStep;
    let mut iters = 0;
    while !(g.active_player_idx == 0 && g.step == TurnStep::Upkeep && g.turn_number >= 2)
        && iters < 200
    {
        // Keep P0 hand at exactly 13 — the draw step would push it to 14
        // and break the predicate; we want the upkeep trigger on T2 P0
        // to see exactly 13 cards.
        let _ = g.pass_priority();
        drain_stack(&mut g);
        iters += 1;
        if g.game_over.is_some() { break; }
        // After P0's T2 untap-step, refresh hand to 13 if it changed.
        if g.active_player_idx == 0 && g.turn_number >= 2 {
            while g.players[0].hand.len() < 13 {
                g.add_card_to_hand(0, catalog::island());
            }
            while g.players[0].hand.len() > 13 {
                g.players[0].hand.pop();
            }
        }
    }
    drain_stack(&mut g);

    assert!(g.game_over.is_some(),
        "Triskaidekaphile should fire on P0's upkeep with exactly 13 cards");
    assert_eq!(g.game_over, Some(Some(0)),
        "P0 should be declared the winner");
}

// ── Excellent Education (STX 2021) ──────────────────────────────────────────

#[test]
fn excellent_education_gives_target_player_life_and_draw() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::excellent_education());
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;

    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Excellent Education castable for {2}{W}");
    drain_stack(&mut g);

    // -1 hand for cast, +1 hand for draw. Net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // +4 life.
    assert_eq!(g.players[0].life, life_before + 4);
}

#[test]
fn excellent_education_can_target_opponent() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::excellent_education());
    let p1_hand_before = g.players[1].hand.len();
    let p1_life_before = g.players[1].life;

    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Excellent Education castable targeting opp");
    drain_stack(&mut g);

    assert_eq!(g.players[1].hand.len(), p1_hand_before + 1, "opp drew 1");
    assert_eq!(g.players[1].life, p1_life_before + 4, "opp gained 4 life");
}

// ── Sproutback Trudge (STX 2021) ────────────────────────────────────────────

#[test]
fn sproutback_trudge_gains_life_per_creature_in_graveyard() {
    let mut g = two_player_game();
    // Seed three creature cards in P0's graveyard.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    // Plus one non-creature for control — shouldn't count.
    g.add_card_to_graveyard(0, catalog::island());

    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::sproutback_trudge());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Sproutback Trudge castable for {3}{G}{G}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 3,
        "Sproutback Trudge gains life equal to creature cards in your graveyard (3)");
}

// ── Deep Analysis (STA reprint) ─────────────────────────────────────────────

#[test]
fn deep_analysis_draws_two_and_loses_two_life() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::deep_analysis());
    let hand_before = g.players[0].hand.len();

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Deep Analysis castable for {3}{U}");
    drain_stack(&mut g);

    // Hand: -1 (cast) + 2 (draw) = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn deep_analysis_caster_draws_two() {
    // Deep Analysis draws for the caster (target-player collapsed).
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::deep_analysis());
    let hand_before = g.players[0].hand.len();

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Deep Analysis castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1, "cast(-1) + draw(+2) = +1");
}

// ── Tribute to Hunger (STA reprint) ─────────────────────────────────────────

#[test]
fn tribute_to_hunger_sacrifices_opp_creature_and_gains_life_equal_to_toughness() {
    use crate::game::Target;
    let mut g = two_player_game();
    // P1 controls a Serra Angel (4/4 flying-vigilance). Sac it; gain 4 life.
    let _angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::tribute_to_hunger());
    let life_before = g.players[0].life;
    let opp_bf_before = g.battlefield.iter().filter(|c| c.controller == 1).count();

    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tribute to Hunger castable for {2}{B}");
    drain_stack(&mut g);

    // P1's creature should be sacrificed.
    let opp_bf_after = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(opp_bf_after, opp_bf_before - 1, "opp sacrificed a creature");
    // P0 gains life equal to sacrificed creature's toughness (4 for Serra Angel).
    assert_eq!(g.players[0].life, life_before + 4,
        "caster gains 4 life = sacrificed creature's toughness");
}

// ── Kasmina's Transmutation (STA reprint) ───────────────────────────────────

#[test]
fn kasminas_transmutation_sets_target_to_one_one_eot() {
    // Use Serra Angel (4/4 flying-vigilance) as a target. After Kasmina's,
    // it should become 1/1 EOT.
    use crate::game::Target;
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());

    let id = g.add_card_to_hand(0, catalog::kasminas_transmutation());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Kasmina's Transmutation castable for {1}{U}{U}");
    drain_stack(&mut g);

    let cp = g.computed_permanent(angel).expect("Serra Angel still on bf");
    assert_eq!(cp.power, 1, "power becomes 1");
    assert_eq!(cp.toughness, 1, "toughness becomes 1");
}

#[test]
fn kasminas_transmutation_strips_flying_from_target() {
    // Serra Angel (4/4 flying-vigilance) becomes 1/1 with no abilities.
    use crate::game::Target;
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());

    let id = g.add_card_to_hand(0, catalog::kasminas_transmutation());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Kasmina's castable");
    drain_stack(&mut g);

    let cp = g.computed_permanent(angel).expect("Angel still on bf");
    assert!(!cp.keywords.contains(&Keyword::Flying), "Flying stripped");
    assert!(!cp.keywords.contains(&Keyword::Vigilance), "Vigilance stripped");
    assert!(cp.lost_all_abilities, "lost_all_abilities flag set");
}

// ── Crippling Fear (STA reprint) ────────────────────────────────────────────

#[test]
fn crippling_fear_kills_two_toughness_creatures() {
    // All creatures get -3/-3 EOT. 2/2 Grizzly Bears (toughness 2) dies
    // (toughness goes to -1, SBA puts in graveyard).
    let mut g = two_player_game();
    let _b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _b3 = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::crippling_fear());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Crippling Fear castable for {3}{B}");
    drain_stack(&mut g);

    let bears_left = g.battlefield.iter().filter(|c| c.definition.is_creature()).count();
    assert_eq!(bears_left, 0,
        "all three 2/2 Grizzly Bears die to -3/-3 EOT");
}

#[test]
fn crippling_fear_does_not_kill_high_toughness_creatures() {
    let mut g = two_player_game();
    let _angel = g.add_card_to_battlefield(0, catalog::serra_angel());

    let id = g.add_card_to_hand(0, catalog::crippling_fear());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Crippling Fear castable");
    drain_stack(&mut g);

    // Serra Angel is 4/4. After -3/-3 it becomes 1/1, still alive.
    let creatures_left = g.battlefield.iter().filter(|c| c.definition.is_creature()).count();
    assert_eq!(creatures_left, 1, "4/4 Serra Angel survives -3/-3 EOT");
}

#[test]
fn crippling_fear_spares_chosen_creature_type() {
    // Printed: "Choose a creature type. Creatures other than creatures
    // of the chosen type get -3/-3 EOT." AutoDecider picks Demon, so
    // Beledros Witherbloom (6/6 Demon) stays untouched while a 2/2
    // Grizzly Bears (Bear) dies under the -3/-3 fan-out.
    let mut g = two_player_game();
    let _demon = g.add_card_to_battlefield(0, catalog::beledros_witherbloom());
    let _bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::crippling_fear());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Crippling Fear castable for {3}{B}");
    drain_stack(&mut g);

    let creatures: Vec<&str> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.is_creature())
        .map(|c| c.definition.name)
        .collect();
    assert_eq!(
        creatures,
        vec!["Beledros Witherbloom"],
        "Demon stays, Bear dies under AutoDecider's Demon pick"
    );
}

#[test]
fn tribute_to_hunger_no_creature_to_sac_gives_no_life() {
    // If opp has no creatures, the sac fails silently and no life is gained.
    use crate::game::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::tribute_to_hunger());
    let life_before = g.players[0].life;

    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tribute to Hunger castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before,
        "no creature to sac → no life gained");
}

#[test]
fn sproutback_trudge_with_empty_graveyard_gains_zero_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::sproutback_trudge());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Sproutback Trudge castable for {3}{G}{G}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before,
        "Sproutback Trudge with empty gy gains 0 life");
}

// ── Pigment Storm ───────────────────────────────────────────────────────────

#[test]
fn pigment_storm_deals_four_damage_to_target_creature() {
    use crate::game::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pigment_storm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Pigment Storm castable for {3}{R}");
    drain_stack(&mut g);

    // Bear (2/2) should die — 4 damage exceeds 2 toughness.
    assert!(
        g.battlefield_find(bear).is_none(),
        "Grizzly Bears killed by 4 damage"
    );
    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == bear),
        "bear is in graveyard"
    );
}

// ── Step Through (STA reprint) ──────────────────────────────────────────────

#[test]
fn step_through_tutors_instant_or_sorcery_from_library() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let target_card = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::step_through());
    let hand_before = g.players[0].hand.len();

    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target_card))]));

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Step Through castable for {U}");
    drain_stack(&mut g);

    // The Lightning Bolt should land in the caster's hand.
    assert!(
        g.players[0].hand.iter().any(|c| c.id == target_card),
        "tutored Lightning Bolt into hand"
    );
    // Hand size: -1 (cast Step Through) +1 (tutored card) = 0 net relative
    // to pre-cast hand size.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Inkfathom Witch ─────────────────────────────────────────────────────────

#[test]
fn inkfathom_witch_etb_makes_opp_discard_a_nonland_card() {
    let mut g = two_player_game();
    // Seed opp hand with a creature card so we have something to discard.
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let opp_hand_before = g.players[1].hand.len();

    let id = g.add_card_to_hand(0, catalog::inkfathom_witch());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Inkfathom Witch castable for {3}{U}{B}");
    drain_stack(&mut g);

    assert_eq!(
        g.players[1].hand.len(),
        opp_hand_before - 1,
        "opp discarded one nonland card"
    );
}

// ── Inscription of Ruin ─────────────────────────────────────────────────────

#[test]
fn inscription_of_ruin_destroys_creature_and_discards() {
    use crate::game::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let opp_hand_before = g.players[1].hand.len();

    let id = g.add_card_to_hand(0, catalog::inscription_of_ruin());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Inscription of Ruin castable for {2}{B}{B}");
    drain_stack(&mut g);

    // Both modes should fire — opp discards 2 + target creature destroyed.
    assert!(
        g.battlefield_find(bear).is_none(),
        "target bear destroyed"
    );
    // Discard 2: opp had only 1 card so loses everything available (1 card).
    assert!(
        g.players[1].hand.len() < opp_hand_before,
        "opp discarded at least one card"
    );
}

#[test]
fn choose_n_decider_overrides_the_default_mode_picks() {
    // CR 700.2d — a ScriptedDecider can pick modes other than the card's
    // default. Inscription of Ruin defaults to [discard, destroy]; scripting
    // mode [1] (reanimate only) returns a creature from gy and leaves the
    // opponent's creature alive.
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    use crate::game::Target;
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gy_bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // reanimation target
    let id = g.add_card_to_hand(0, catalog::inscription_of_ruin());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![1])]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp_bear).is_some(),
        "destroy mode was NOT chosen — opponent's creature survives");
    assert!(g.players[0].hand.iter().any(|c| c.id == gy_bear),
        "reanimate mode ran — the gy creature is back in hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == gy_bear),
        "the reanimated creature left the graveyard");
}

// ── Tome of the Infinite ────────────────────────────────────────────────────

#[test]
fn tome_of_the_infinite_etb_scrys_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::tome_of_the_infinite());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Tome of the Infinite castable for {1}");
    drain_stack(&mut g);

    // Tome is on the battlefield.
    let tome = g.battlefield_find(id).expect("tome on battlefield");
    assert_eq!(tome.controller, 0);
}

// ── Drannith Stinger ────────────────────────────────────────────────────────

#[test]
fn drannith_stinger_pings_opp_on_noncreature_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::drannith_stinger());

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let opp_life_before = g.players[1].life;

    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);

    // Bolt does 3 damage + Stinger does 1 = 4 damage total.
    assert_eq!(g.players[1].life, opp_life_before - 4,
        "Stinger pings on Bolt cast for +1 damage");
}

#[test]
fn drannith_stinger_does_not_ping_on_creature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::drannith_stinger());

    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let opp_life_before = g.players[1].life;

    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bears castable");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, opp_life_before,
        "Stinger should NOT ping on creature cast");
}

// ── Mage Mauler ─────────────────────────────────────────────────────────────

#[test]
fn mage_mauler_deals_three_to_creature_and_gains_one_life() {
    use crate::game::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mage_mauler());
    let life_before = g.players[0].life;

    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Mage Mauler castable");
    drain_stack(&mut g);

    assert!(g.battlefield_find(bear).is_none(), "bear destroyed");
    assert_eq!(g.players[0].life, life_before + 1,
        "caster gains 1 life");
}

// ── Heirloom Mirror ─────────────────────────────────────────────────────────

#[test]
fn heirloom_mirror_tap_for_mana_then_sac_to_draw() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::heirloom_mirror());
    g.add_card_to_library(0, catalog::island());

    // Tap for mana (any color)
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None, x_value: None })
    .expect("first ability is a mana ability");

    // Untap to allow the second activation.
    let mirror = g.battlefield_find_mut(id).expect("mirror on bf");
    mirror.tapped = false;

    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 1,
        target: None, x_value: None })
    .expect("second ability sacs and draws");
    drain_stack(&mut g);

    assert!(g.battlefield_find(id).is_none(), "mirror sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

// ── Quandrix Mascot ─────────────────────────────────────────────────────────

#[test]
fn quandrix_mascot_doubles_counters_on_target() {
    use crate::game::Target;
    let mut g = two_player_game();
    // Friendly creature with a +1/+1 counter on it.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let card = g.battlefield_find_mut(bear).expect("bear on bf");
    card.add_counters(CounterType::PlusOnePlusOne, 3);

    let id = g.add_card_to_hand(0, catalog::quandrix_mascot());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Quandrix Mascot castable");
    drain_stack(&mut g);

    let bear_card = g.battlefield_find(bear).expect("bear still alive");
    assert_eq!(bear_card.counter_count(CounterType::PlusOnePlusOne), 6,
        "3 counters doubled to 6");
}

// ── Witherbloom Mascot ──────────────────────────────────────────────────────

#[test]
fn witherbloom_mascot_dies_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_mascot());
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;

    // Mark lethal damage to make it die to SBA.
    let card = g.battlefield_find_mut(id).expect("mascot on bf");
    card.damage = 99;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);

    // Drain 2: P1 loses 2, P0 gains 2.
    assert_eq!(g.players[0].life, p0_life_before + 2);
    assert_eq!(g.players[1].life, p1_life_before - 2);
}

// ── Lorehold Mascot ─────────────────────────────────────────────────────────

/// CR 701.25c — "If a player is instructed to surveil 0, no surveil
/// event occurs. Abilities that trigger whenever a player surveils
/// won't trigger." Validate the `Effect::Surveil` short-circuit on
/// `amount: Value::Const(0)` — no Decision is asked of the decider
/// and library order is unchanged. Shares the same n==0 short-circuit
/// path as CR 701.22b (Scry 0) in `game/effects/mod.rs::resolve_effect`.
#[test]
fn zero_surveil_does_not_trigger_surveil_events_per_cr_701_25c() {
    use crate::card::{CardDefinition, CardType, Effect, Subtypes, Value};
    use crate::effect::PlayerRef;
    use crate::mana::cost;

    let zero_surveil = CardDefinition {
        name: "Zero Surveil",
        cost: cost(&[crate::mana::u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(0) },
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
    };

    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let lib_snapshot: Vec<_> = g.players[0].library.iter().map(|c| c.id).collect();
    let gy_before = g.players[0].graveyard.len();

    let id = g.add_card_to_hand(0, zero_surveil);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Zero Surveil castable for {U}");
    drain_stack(&mut g);

    // Library order is unchanged.
    let lib_after: Vec<_> = g.players[0].library.iter().map(|c| c.id).collect();
    assert_eq!(lib_after, lib_snapshot, "Library order unchanged on Surveil 0");
    // Only Zero Surveil itself is in the graveyard (no surveiled cards).
    assert_eq!(g.players[0].graveyard.len(), gy_before + 1,
        "Only the spell itself entered graveyard");
}

// ── Spiteful Squad ──────────────────────────────────────────────────────────

#[test]
fn spiteful_squad_dies_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::spiteful_squad());
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;

    let card = g.battlefield_find_mut(id).expect("squad on bf");
    card.damage = 99;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, p0_life_before + 2);
    assert_eq!(g.players[1].life, p1_life_before - 2);
}

// ── Master Symmetrist ───────────────────────────────────────────────────────

#[test]
fn master_symmetrist_doubles_counters_on_friendlies() {
    let mut g = two_player_game();
    // Friendly creature with a +1/+1 counter on it.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let card = g.battlefield_find_mut(bear).expect("bear on bf");
    card.add_counters(CounterType::PlusOnePlusOne, 2);

    let id = g.add_card_to_hand(0, catalog::master_symmetrist());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Master Symmetrist castable");
    drain_stack(&mut g);

    let bear_card = g.battlefield_find(bear).expect("bear still alive");
    assert_eq!(
        bear_card.counter_count(CounterType::PlusOnePlusOne),
        4,
        "2 counters doubled to 4"
    );
}

// ── Stinging Cave Crawler ───────────────────────────────────────────────────

#[test]
fn stinging_cave_crawler_etb_scrys_two() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::stinging_cave_crawler());

    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Stinging Cave Crawler castable for {3}{B}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield_find(id).is_some(), "Crawler resolved");
}

// ── Cogwork Archivist ───────────────────────────────────────────────────────

#[test]
fn cogwork_archivist_etb_mills_four_from_target_opponent() {
    let mut g = two_player_game();
    for _ in 0..8 { g.add_card_to_library(1, catalog::island()); }
    let opp_gy_before = g.players[1].graveyard.len();
    let opp_lib_before = g.players[1].library.len();
    let self_lib_before = g.players[0].library.len();

    let id = g.add_card_to_hand(0, catalog::cogwork_archivist());
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Archivist castable for {6}");
    drain_stack(&mut g);

    // Auto-target picks the opponent (mill is a hostile player-side effect).
    assert_eq!(g.players[1].library.len(), opp_lib_before - 4, "opponent milled 4");
    assert_eq!(g.players[1].graveyard.len(), opp_gy_before + 4,
        "opponent's graveyard gained 4");
    assert_eq!(g.players[0].library.len(), self_lib_before,
        "controller's own library untouched");
}

#[test]
fn lorehold_mascot_attack_gains_life_and_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_mascot());
    // Skip summoning sickness.
    let card = g.battlefield_find_mut(id).expect("mascot on bf");
    card.summoning_sick = false;
    let life_before = g.players[0].life;

    // Move to declare attackers step.
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }

    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("declare attackers");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 1, "gained 1 on attack");
    // Power was 3, now 4 after pump.
    let cp = g.compute_battlefield().iter()
        .find(|c| c.id == id).cloned()
        .expect("computed permanent");
    assert_eq!(cp.power, 4, "Pumped to 4 on attack");
}

// ── Adrix and Nev, Twincasters (modern_decks push) ─────────────────────────

#[test]
fn adrix_and_nev_doubles_own_tokens_but_not_opponents() {
    // Own cast: Adrix doubles one Pest → two.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::adrix_and_nev_twincasters());
    let id = g.add_card_to_hand(0, catalog::hunt_for_specimens());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let pests_before = g.battlefield.iter().filter(|c| c.definition.name == "Pest").count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hunt for Specimens castable");
    drain_stack(&mut g);
    let pests_after = g.battlefield.iter().filter(|c| c.definition.name == "Pest").count();
    assert_eq!(pests_after - pests_before, 2,
        "Adrix doubles the mint: one Pest → two Pests");

    // Opponent's cast: Adrix is on P0's side, so opp's token mint stays at 1.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::adrix_and_nev_twincasters());
    let id = g.add_card_to_hand(1, catalog::hunt_for_specimens());
    g.players[1].mana_pool.add(Color::Black, 1);
    g.players[1].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    g.step = crate::game::types::TurnStep::PreCombatMain;
    let pests_before = g.battlefield.iter().filter(|c| c.definition.name == "Pest").count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Opp casts Hunt for Specimens");
    drain_stack(&mut g);
    let pests_after = g.battlefield.iter().filter(|c| c.definition.name == "Pest").count();
    assert_eq!(pests_after - pests_before, 1,
        "Adrix doesn't double opp's token mint");
}

// ── Strixhaven Stadium (modern_decks push) ─────────────────────────────────

#[test]
fn strixhaven_stadium_pumps_attacker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::strixhaven_stadium());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);

    // Move to declare attackers step.
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }])).expect("declare attackers");
    drain_stack(&mut g);

    // Bear was 2/2, Stadium pumps it +1/+1 EOT.
    let cp = g.compute_battlefield().iter()
        .find(|c| c.id == bear).cloned()
        .expect("bear on bf");
    assert_eq!(cp.power, 3, "Bear pumped to 3 power on attack");
    assert_eq!(cp.toughness, 3, "Bear pumped to 3 toughness on attack");
}

#[test]
fn strixhaven_stadium_activation_costs_three_charge_counters_and_draws_two() {
    let mut g = two_player_game();
    let stadium = g.add_card_to_battlefield(0, catalog::strixhaven_stadium());
    // Seed three charge counters.
    {
        let s = g.battlefield_find_mut(stadium).expect("stadium");
        s.add_counters(CounterType::Charge, 3);
    }
    // Seed library for the two draws.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand_before = g.players[0].hand.len();

    // Activate the ability (no mana cost — just tap + remove 3 charge).
    g.perform_action(GameAction::ActivateAbility {
        card_id: stadium,
        ability_index: 0,
        target: None, x_value: None }).expect("activation succeeds with 3 charge counters");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew 2 cards");
    let s = g.battlefield_find(stadium).expect("stadium still on bf");
    assert_eq!(s.counter_count(CounterType::Charge), 0, "3 charges removed");
}

#[test]
fn strixhaven_stadium_activation_rejected_without_enough_charge_counters() {
    let mut g = two_player_game();
    let stadium = g.add_card_to_battlefield(0, catalog::strixhaven_stadium());
    // Only 2 charge counters — below the 3 threshold.
    {
        let s = g.battlefield_find_mut(stadium).expect("stadium");
        s.add_counters(CounterType::Charge, 2);
    }
    // Activation should be rejected.
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: stadium,
        ability_index: 0,
        target: None, x_value: None });
    assert!(res.is_err(), "activation requires 3 charge counters");
}

// ── Awesome Presentation (modern_decks push) ───────────────────────────────

#[test]
fn awesome_presentation_mints_two_inkling_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::awesome_presentation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);

    let inklings_before = g.battlefield.iter()
        .filter(|c| c.definition.name == "Inkling").count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Awesome Presentation castable for {3}{W}{B}");
    drain_stack(&mut g);
    let inklings_after = g.battlefield.iter()
        .filter(|c| c.definition.name == "Inkling").count();
    assert_eq!(inklings_after - inklings_before, 2);
}

// ── Rise of Extus (modern_decks push) ──────────────────────────────────────

#[test]
fn rise_of_extus_deals_five_damage_and_returns_is_from_graveyard() {
    let mut g = two_player_game();
    // Seed library for Learn's Draw.
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::rise_of_extus());

    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Rise of Extus castable");
    drain_stack(&mut g);

    // 5 damage kills the 2/2 bear.
    let bear_dead = g.battlefield.iter().all(|c| c.id != opp_bear);
    assert!(bear_dead, "5 damage kills the bear");
    // Lightning Bolt should be back in hand. Plus Learn (Draw 1).
    let hand_after = g.players[0].hand.len();
    let bolt_in_hand = g.players[0].hand.iter().any(|c| c.id == bolt);
    assert!(bolt_in_hand, "Lightning Bolt returned to hand");
    // Hand: -1 (cast Rise of Extus) +1 (Lightning Bolt return) +1 (Learn) = +1.
    assert_eq!(hand_after - hand_before, 1);
}

// ── Brackish Trudge (modern_decks push) ────────────────────────────────────

// ── Lurking Deadeye (modern_decks push) ────────────────────────────────────

#[test]
fn lurking_deadeye_etb_minus_two_target_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lurking_deadeye());

    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Lurking Deadeye castable at flash speed");
    drain_stack(&mut g);

    // Bear was 2/2, -2/-2 kills it.
    let bear_dead = g.battlefield.iter().all(|c| c.id != opp_bear);
    assert!(bear_dead, "-2/-2 kills the bear via SBA");
}

// ── Aether Helix (modern_decks push) ────────────────────────────────────────

#[test]
fn aether_helix_bounces_nonland_and_burns_opp() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::aether_helix());

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);

    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Aether Helix castable");
    drain_stack(&mut g);

    // Bear bounced to opp's hand.
    let bear_on_bf = g.battlefield.iter().any(|c| c.id == opp_bear);
    assert!(!bear_on_bf, "Bear bounced off battlefield");
    let bear_in_hand = g.players[1].hand.iter().any(|c| c.id == opp_bear);
    assert!(bear_in_hand, "Bear in opp's hand");
    // Opp lost 2 life from the burn.
    assert_eq!(g.players[1].life, life_before - 2);
}

// ── Reflective Golem (modern_decks push) ────────────────────────────────────

// ── Tempest Caller (modern_decks push) ──────────────────────────────────────

#[test]
fn tempest_caller_etb_taps_opponent_creatures() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::tempest_caller());

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tempest Caller castable");
    drain_stack(&mut g);

    let b1 = g.battlefield_find(bear1).expect("bear1 still on bf");
    let b2 = g.battlefield_find(bear2).expect("bear2 still on bf");
    assert!(b1.tapped, "bear1 tapped");
    assert!(b2.tapped, "bear2 tapped");
}

// ── Pillardrop Warden (modern_decks push) ──────────────────────────────────

#[test]
fn pillardrop_warden_etb_may_pay_returns_creature_card() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let dead_bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pillardrop_warden());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(5); // 3 base + 2 for MayPay

    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pillardrop Warden castable");
    drain_stack(&mut g);

    // Bear came back to hand.
    let bear_in_hand = g.players[0].hand.iter().any(|c| c.id == dead_bear);
    assert!(bear_in_hand, "Bear returned to hand via MayPay");
}

// ── Devourer of Memory (modern_decks push) ─────────────────────────────────

#[test]
fn devourer_of_memory_magecraft_pumps_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::devourer_of_memory());
    g.clear_sickness(id);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());

    g.players[0].mana_pool.add(Color::Red, 1);

    // Cast Lightning Bolt at opp; Magecraft pumps Devourer +1/+0 EOT.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);

    let cp = g.compute_battlefield().iter()
        .find(|c| c.id == id).cloned()
        .expect("devourer on bf");
    assert_eq!(cp.power, 3, "Devourer pumped to 3 power");
}

// ── Mavinda's Verdict (modern_decks push) ──────────────────────────────────

#[test]
fn mavindas_verdict_exiles_creature_and_gains_life() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mavindas_verdict());

    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Mavinda's Verdict castable");
    drain_stack(&mut g);

    // Bear exiled (not in any zone but exile).
    let bear_exiled = g.exile.iter().any(|c| c.id == opp_bear);
    assert!(bear_exiled, "Bear in exile");
    // Gain life = bear's toughness (2).
    assert_eq!(g.players[0].life, life_before + 2);
}

// ── Witherbloom Skillchaser (modern_decks push) ────────────────────────────

#[test]
fn witherbloom_skillchaser_etb_creates_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_skillchaser());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    let pests_before = g.battlefield.iter()
        .filter(|c| c.definition.name == "Pest").count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skillchaser castable");
    drain_stack(&mut g);
    let pests_after = g.battlefield.iter()
        .filter(|c| c.definition.name == "Pest").count();
    assert_eq!(pests_after - pests_before, 1);
}

// ── Quandrix Pop Quiz (modern_decks push) ──────────────────────────────────

#[test]
fn quandrix_pop_quiz_creates_fractal_with_x_counters() {
    let mut g = two_player_game();
    // Put four lands you control on the battlefield.
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_pop_quiz());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quandrix Pop Quiz castable");
    drain_stack(&mut g);

    // Fractal should be 4/4 (X = 4 lands).
    let fractal_id = g.battlefield.iter()
        .find(|c| c.definition.name == "Fractal").map(|c| c.id);
    assert!(fractal_id.is_some(), "Fractal token created");
    let cp = g.compute_battlefield().iter()
        .find(|c| c.id == fractal_id.unwrap()).cloned()
        .expect("computed fractal");
    assert_eq!(cp.power, 4);
    assert_eq!(cp.toughness, 4);
}

// ── Inkwood Scrivener (modern_decks push) ──────────────────────────────────

#[test]
fn inkwood_scrivener_etb_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkwood_scrivener());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scrivener castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, p0_life + 1);
    assert_eq!(g.players[1].life, p1_life - 1);
}

// ── Furnace Hellkite (modern_decks push) ───────────────────────────────────

#[test]
fn furnace_hellkite_etb_burns_each_opp_for_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::furnace_hellkite());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);

    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hellkite castable");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, p1_life - 2);
}

// ── Pinion Lecturer (modern_decks push) ────────────────────────────────────

// ── Sparkling Insight (modern_decks push) ──────────────────────────────────

#[test]
fn sparkling_insight_scries_two_then_draws_two() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::sparkling_insight());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkling Insight castable");
    drain_stack(&mut g);
    // Net: -1 (cast) + 2 (draw 2) = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

// ── Pop Quiz Coach (modern_decks push) ─────────────────────────────────────

#[test]
fn pop_quiz_coach_magecraft_adds_counter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pop_quiz_coach());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());

    g.players[0].mana_pool.add(Color::Red, 1);

    // Cast Lightning Bolt; Magecraft puts a +1/+1 counter on the bear.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);

    let cp = g.compute_battlefield().iter()
        .find(|c| c.id == bear).cloned()
        .expect("bear on bf");
    // Bear was 2/2, now 3/3 with the +1/+1 counter.
    assert_eq!(cp.power, 3);
    assert_eq!(cp.toughness, 3);
}

// ── Soothing Hush (modern_decks push) ─────────────────────────────────────

#[test]
fn soothing_hush_counters_creature_spell() {
    let mut g = two_player_game();
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);

    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts Bears");

    let hush = g.add_card_to_hand(0, catalog::soothing_hush());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: hush,
        target: Some(crate::game::types::Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Soothing Hush castable");
    drain_stack(&mut g);

    // Bears should be countered (in graveyard, not battlefield).
    let bears_on_bf = g.battlefield.iter().any(|c| c.id == bears);
    assert!(!bears_on_bf, "Bears countered, not on battlefield");
    let bears_in_gy = g.players[1].graveyard.iter().any(|c| c.id == bears);
    assert!(bears_in_gy, "Bears in graveyard");
}

// ── Vortex Runner (modern_decks push) ─────────────────────────────────────

// ── Sage of the Beyond (modern_decks push) ────────────────────────────────

#[test]
fn sage_of_the_beyond_combat_damage_makes_opp_discard() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sage_of_the_beyond());
    g.clear_sickness(id);
    // Seed opp's hand with a card to discard.
    let target_card = g.add_card_to_hand(1, catalog::lightning_bolt());

    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("declare attackers");
    drain_stack(&mut g);

    // Skip past blockers (none).
    while g.step != crate::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);

    // Opp discarded a card from hand.
    let card_in_hand = g.players[1].hand.iter().any(|c| c.id == target_card);
    let card_in_gy = g.players[1].graveyard.iter().any(|c| c.id == target_card);
    assert!(
        !card_in_hand || card_in_gy,
        "Opp's card discarded (in gy, not hand)"
    );
}

// ── Frostpyre Arcanist (modern_decks push) ────────────────────────────────

#[test]
fn frostpyre_arcanist_magecraft_returns_is_from_graveyard() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::frostpyre_arcanist());
    // Seed an IS card in graveyard.
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    // Cast another bolt to trigger Magecraft.
    let live_bolt = g.add_card_to_hand(0, catalog::lightning_bolt());

    g.players[0].mana_pool.add(Color::Red, 1);

    // ScriptedDecider answers Bool(true) to take the optional return.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: live_bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);

    // Original bolt should be back in hand.
    let bolt_in_hand = g.players[0].hand.iter().any(|c| c.id == bolt);
    assert!(bolt_in_hand, "Bolt returned to hand via Frostpyre's Magecraft");
}

// ── Inkfathom Divers (modern_decks push) ──────────────────────────────────

#[test]
fn inkfathom_divers_etb_strips_opp_nonland_from_hand() {
    let mut g = two_player_game();
    // Seed opp's hand with a nonland card.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::inkfathom_divers());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkfathom Divers castable");
    drain_stack(&mut g);

    // Bolt stripped from hand.
    let bolt_in_hand = g.players[1].hand.iter().any(|c| c.id == bolt);
    let bolt_in_gy = g.players[1].graveyard.iter().any(|c| c.id == bolt);
    assert!(!bolt_in_hand, "Bolt removed from opp's hand");
    assert!(bolt_in_gy, "Bolt in opp's graveyard");
}

// ── Quandrix Quickener (modern_decks push) ─────────────────────────────────

#[test]
fn quandrix_quickener_scries_and_untaps_target_land() {
    let mut g = two_player_game();
    // Seed library for scry.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    // Tap a land we can target.
    let land = g.add_card_to_battlefield(0, catalog::forest());
    {
        let l = g.battlefield_find_mut(land).expect("forest");
        l.tapped = true;
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_quickener());

    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(land)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Quandrix Quickener castable");
    drain_stack(&mut g);

    let l = g.battlefield_find(land).expect("forest on bf");
    assert!(!l.tapped, "Forest untapped via Quickener");
    // Look at top 3, one to hand (net: cast -1 + pick +1 = 0), rest stay in
    // the library (on the bottom).
    assert_eq!(g.players[0].hand.len(), hand_before, "one card picked into hand");
    assert_eq!(g.players[0].library.len(), lib_before - 1, "only the picked card left the library");
}

// ── Search for Glory (modern_decks push) ────────────────────────────────

#[test]
fn search_for_glory_tutors_a_legendary_card_to_hand() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let lib_card = g.add_card_to_library(0, catalog::quintorius_field_historian());
    let id = g.add_card_to_hand(0, catalog::search_for_glory());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.decider = Box::new(ScriptedDecider::new(vec![
        // Scry decision (none — pass)
        DecisionAnswer::ScryOrder { kept_top: vec![], bottom: vec![] },
        DecisionAnswer::Search(Some(lib_card)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Search for Glory castable");
    drain_stack(&mut g);

    let in_hand = g.players[0].hand.iter().any(|c| c.id == lib_card);
    assert!(in_hand, "Quintorius tutored into hand");
}

// ── Fervent Strike (modern_decks push) ──────────────────────────────────

#[test]
fn fervent_strike_pumps_target_and_grants_trample() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fervent_strike());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Fervent Strike castable");
    drain_stack(&mut g);

    let b = g.computed_permanent(bear).expect("bear");
    assert_eq!(b.power, 4, "+2/+0 pump");
    assert!(b.keywords.contains(&Keyword::Trample), "trample granted");
}

// ── Elemental Summoning (modern_decks push) ─────────────────────────────

#[test]
fn elemental_summoning_mints_a_four_four_elemental() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::elemental_summoning());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Elemental Summoning castable");
    drain_stack(&mut g);

    let elementals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Elemental" && c.controller == 0)
        .collect();
    assert_eq!(elementals.len(), 1, "one elemental token created");
    assert_eq!(elementals[0].definition.power, 4);
    assert_eq!(elementals[0].definition.toughness, 4);
}

// ── Humiliate (modern_decks push) ───────────────────────────────────────

#[test]
fn humiliate_strips_opp_nonland_and_drains_one() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::humiliate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let life_p0_before = g.players[0].life;
    let life_p1_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Humiliate castable");
    drain_stack(&mut g);

    let bolt_in_hand = g.players[1].hand.iter().any(|c| c.id == bolt);
    assert!(!bolt_in_hand, "Bolt discarded from opp's hand");
    assert_eq!(g.players[0].life, life_p0_before + 1, "you gain 1 life");
    assert_eq!(g.players[1].life, life_p1_before - 1, "opp loses 1 life");
}

// ── Elite Spellbinder (modern_decks push) ───────────────────────────────

#[test]
fn elite_spellbinder_etb_strips_opp_nonland() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::elite_spellbinder());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Elite Spellbinder castable");
    drain_stack(&mut g);

    let bolt_in_hand = g.players[1].hand.iter().any(|c| c.id == bolt);
    assert!(!bolt_in_hand, "Bolt removed from opp's hand");
}

// ── Waker of Waves (modern_decks push) ──────────────────────────────────

#[test]
fn waker_of_waves_etb_loots_two() {
    let mut g = two_player_game();
    // Seed library
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    // Seed hand with two cards to discard
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::waker_of_waves());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Waker of Waves castable");
    drain_stack(&mut g);
    // -1 cast, +2 draw, -2 discard = -1
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

// ── Discover the Formula (modern_decks push) ───────────────────────────

#[test]
fn discover_the_formula_draws_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::discover_the_formula());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Discover the Formula castable");
    drain_stack(&mut g);
    // -1 cast + 3 draw = +2
    assert_eq!(g.players[0].hand.len(), hand_before + 2);
}

// ── Mortician Beetle (modern_decks push) ───────────────────────────────

#[test]
fn mortician_beetle_grows_on_creature_death() {
    let mut g = two_player_game();
    let beetle = g.add_card_to_battlefield(0, catalog::mortician_beetle());
    g.clear_sickness(beetle);
    // Give opp a creature to kill.
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Cast lightning bolt on opp's bear
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);

    // Bear dies → Mortician Beetle gets +1/+1 counter
    let b = g.computed_permanent(beetle).expect("beetle");
    assert!(b.power >= 2, "Mortician Beetle pumped to 2+: {}/{}", b.power, b.toughness);
}

// ── Vespine Strix (modern_decks push) ──────────────────────────────────

// ── Witherbloom Apprenticeship (modern_decks push) ─────────────────────

#[test]
fn witherbloom_apprenticeship_creates_pests_and_pumps_board() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_apprenticeship());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Witherbloom Apprenticeship castable");
    drain_stack(&mut g);

    let pests = g.battlefield.iter()
        .filter(|c| c.definition.name == "Pest" && c.controller == 0)
        .count();
    assert_eq!(pests, 2, "two Pest tokens minted");
    // bear got +1/+1 counter
    let b = g.computed_permanent(bear).expect("bear");
    assert_eq!(b.power, 3, "bear pumped via counter");
}

// ── Wandering Mind (modern_decks push) ─────────────────────────────────

// ── Lecturing Loxodon (modern_decks push) ──────────────────────────────

#[test]
fn lecturing_loxodon_etb_pumps_other_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lecturing_loxodon());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lecturing Loxodon castable");
    drain_stack(&mut g);

    let b = g.computed_permanent(bear).expect("bear");
    assert_eq!(b.power, 3, "bear pumped +1/+1");
    assert_eq!(b.toughness, 3);
}

// ── Curriculum Crab (modern_decks push) ────────────────────────────────

#[test]
fn curriculum_crab_etb_counters_with_scripted_decider() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::curriculum_crab());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Curriculum Crab castable");
    drain_stack(&mut g);

    let b = g.battlefield_find(bear).expect("bear on bf");
    let counters = b.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert!(counters >= 1, "bear got at least one +1/+1 counter");
}

// ── Pyrotechnics (modern_decks push) ───────────────────────────────────

#[test]
fn pyrotechnics_burns_target_creature_for_four() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pyrotechnics());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrotechnics castable");
    drain_stack(&mut g);

    // 4 damage kills the 2/2 bear via SBA
    let bear_dead = !g.battlefield.iter().any(|c| c.id == opp_bear);
    assert!(bear_dead, "bear destroyed by 4 damage");
}

// ── Tome of the Guildpact (modern_decks push) ──────────────────────────

// ── Stormwild Capridor (modern_decks push) ─────────────────────────────

// ── Final Payment (modern_decks push) ──────────────────────────────────

#[test]
fn final_payment_destroys_creature_or_planeswalker() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::final_payment());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Final Payment castable");
    drain_stack(&mut g);

    let bear_dead = !g.battlefield.iter().any(|c| c.id == opp_bear);
    assert!(bear_dead, "bear destroyed by Final Payment");
}

// ── Witch's Cauldron (modern_decks push) ───────────────────────────────

#[test]
fn witchs_cauldron_sac_gains_two_life_and_draws() {
    let mut g = two_player_game();
    let cauldron = g.add_card_to_battlefield(0, catalog::witchs_cauldron());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(cauldron);
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }

    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: cauldron,
        ability_index: 0,
        target: None, x_value: None }).expect("Cauldron activation");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 2, "gained 2 life");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    let bear_dead = !g.battlefield.iter().any(|c| c.id == bear);
    assert!(bear_dead, "bear sacrificed");
}

// ── Steady Stance (modern_decks push) ──────────────────────────────────

#[test]
fn steady_stance_pumps_three_toughness_and_grants_vigilance() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::steady_stance());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Steady Stance castable");
    drain_stack(&mut g);

    let b = g.computed_permanent(bear).expect("bear");
    assert_eq!(b.toughness, 5, "+0/+3 toughness pump");
    assert!(b.keywords.contains(&Keyword::Vigilance), "vigilance granted");
}

#[test]
fn tome_of_the_guildpact_activation_draws_a_card() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let tome = g.add_card_to_battlefield(0, catalog::tome_of_the_guildpact());
    g.clear_sickness(tome);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: tome,
        ability_index: 0,
        target: None, x_value: None }).expect("Tome activation");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

// ─────────────────────────────────────────────────────────────────────────────
// modern_decks push: 21 NEW STX / STA cards
// ─────────────────────────────────────────────────────────────────────────────

// ── Revitalize ─────────────────────────────────────────────────────────────

#[test]
fn revitalize_gains_three_and_draws() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::revitalize());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Revitalize castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 3, "gained 3 life");
    // Hand: -1 cast + 1 draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before, "net 0 cards");
}

// ── Grim Bounty ────────────────────────────────────────────────────────────

#[test]
fn grim_bounty_destroys_target_creature_and_creates_treasure() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::grim_bounty());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);

    let treasures_before = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.name == "Treasure"
    }).count();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Grim Bounty castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear), "bear destroyed");
    let treasures_after = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.name == "Treasure"
    }).count();
    assert_eq!(treasures_after, treasures_before + 1, "created a Treasure");
}

// ── Growth Spiral ──────────────────────────────────────────────────────────

#[test]
fn growth_spiral_draws_a_card() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::growth_spiral());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Growth Spiral castable");
    drain_stack(&mut g);

    // -1 cast + 1 draw = 0 net (auto-decider declines the land drop)
    assert_eq!(g.players[0].hand.len(), hand_before, "net 0 cards");
}

#[test]
fn growth_spiral_optional_land_drop_with_scripted_decider() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed hand with a land + library has a draw target
    let forest_in_hand = g.add_card_to_hand(0, catalog::forest());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    let id = g.add_card_to_hand(0, catalog::growth_spiral());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Growth Spiral castable");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == forest_in_hand),
        "forest from hand should be on the battlefield");
}

// ── Idyllic Tutor ──────────────────────────────────────────────────────────

#[test]
fn idyllic_tutor_searches_an_enchantment_to_hand() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed library with an enchantment + filler
    let lurk = g.add_card_to_library(0, catalog::lurking_predators());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(lurk))]));

    let id = g.add_card_to_hand(0, catalog::idyllic_tutor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Idyllic Tutor castable");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == lurk),
        "Lurking Predators should be in hand after tutor");
}

// ── Gift of Estates ────────────────────────────────────────────────────────

#[test]
fn gift_of_estates_searches_three_plains_when_opp_has_more_lands() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Give P1 (opp) two lands, P0 (caster) zero lands so opp leads.
    g.add_card_to_battlefield(1, catalog::forest());
    g.add_card_to_battlefield(1, catalog::forest());
    // Seed library with 3 Plains
    let p1 = g.add_card_to_library(0, catalog::plains());
    let p2 = g.add_card_to_library(0, catalog::plains());
    let p3 = g.add_card_to_library(0, catalog::plains());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(p1)),
        DecisionAnswer::Search(Some(p2)),
        DecisionAnswer::Search(Some(p3)),
    ]));

    let id = g.add_card_to_hand(0, catalog::gift_of_estates());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Gift of Estates castable");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == p1), "p1 in hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == p2), "p2 in hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == p3), "p3 in hand");
}

#[test]
fn gift_of_estates_skips_search_when_lands_equal() {
    let mut g = two_player_game();
    // Both players have one land — gate fails (P1 doesn't have *more* than P0).
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(1, catalog::forest());
    let p1 = g.add_card_to_library(0, catalog::plains());

    let id = g.add_card_to_hand(0, catalog::gift_of_estates());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Gift of Estates castable");
    drain_stack(&mut g);

    // Predicate fails (equal lands), so no Search fires. Plains stays in library.
    assert!(
        !g.players[0].hand.iter().any(|c| c.id == p1),
        "no plains tutored when lands are equal"
    );
}

// ── Pillage ────────────────────────────────────────────────────────────────

#[test]
fn pillage_destroys_target_land_or_artifact() {
    // Land target.
    let mut g = two_player_game();
    let opp_land = g.add_card_to_battlefield(1, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::pillage());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp_land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pillage castable on land");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_land), "land destroyed");

    // Artifact target.
    let mut g = two_player_game();
    let opp_artifact = g.add_card_to_battlefield(1, catalog::sungrass_egg());
    let id = g.add_card_to_hand(0, catalog::pillage());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp_artifact)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pillage castable on artifact");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_artifact), "artifact destroyed");
}

// ── Slip Through Space ─────────────────────────────────────────────────────

#[test]
fn slip_through_space_grants_unblockable_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::slip_through_space());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Slip Through Space castable");
    drain_stack(&mut g);

    let b = g.computed_permanent(bear).expect("bear");
    assert!(b.keywords.contains(&Keyword::Unblockable), "granted unblockable");
    // -1 cast + 1 draw = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before, "drew a card");
}

// ── Doomskar ───────────────────────────────────────────────────────────────

#[test]
fn doomskar_destroys_each_creature() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear3 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::doomskar());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Doomskar castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear1), "bear1 destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == bear2), "bear2 destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == bear3), "bear3 destroyed");
}

/// Doomskar can be foretold ({2}, exile face-down) and cast from exile for
/// its foretell cost ({1}{W}{W}) on a later turn (CR 702.143).
#[test]
fn doomskar_foretell_then_cast_next_turn() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::doomskar());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Foretell { card_id: id }).expect("foretell for {2}");
    let card = g.exile.iter().find(|c| c.id == id).expect("foretold card in exile");
    assert!(card.face_down, "foretold cards are exiled face-down");
    // Can't cast it the turn it was foretold.
    assert!(g.perform_action(GameAction::CastForetold {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "not castable the turn it was foretold");

    // Simulate a later turn: foretold-this-turn cleared, board has a creature.
    g.foretold_this_turn.clear();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastForetold {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast foretold Doomskar for {1}{W}{W}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Doomskar wraths the board");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "cast Doomskar goes to graveyard");
}

// ── Battle Mammoth ─────────────────────────────────────────────────────────

// ── Mind Drain ─────────────────────────────────────────────────────────────

#[test]
fn mind_drain_makes_each_opp_discard_two() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::mind_drain());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    let opp_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mind Drain castable");
    drain_stack(&mut g);

    assert_eq!(g.players[1].hand.len(), opp_hand_before - 2, "opp discarded 2");
}

// ── Hindering Light ────────────────────────────────────────────────────────

#[test]
fn hindering_light_counters_target_spell_and_draws() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }

    // Opponent casts Lightning Bolt at us (switch active player)
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");

    // We respond with Hindering Light targeting the bolt
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::hindering_light());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hindering Light castable on bolt");
    drain_stack(&mut g);

    // Bolt should be countered → no damage to us
    assert_eq!(g.players[0].life, life_before, "no bolt damage");
    // -1 cast + 1 draw = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before, "drew a card");
}

// ── Soul Shatter ───────────────────────────────────────────────────────────

#[test]
fn soul_shatter_each_opp_sacrifices_a_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::soul_shatter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soul Shatter castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear), "opp sacrificed bear");
}

#[test]
fn soul_shatter_picks_greatest_mana_value_creature() {
    let mut g = two_player_game();
    // Opp has a 2-MV bear and a 5-MV Serra Angel — Soul Shatter should
    // hit the Angel (greater MV), not the bear.
    let cheap_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big_angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::soul_shatter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soul Shatter castable");
    drain_stack(&mut g);
    // The Angel (5-MV) should die; bear (2-MV) stays.
    assert!(!g.battlefield.iter().any(|c| c.id == big_angel),
        "5-MV Angel sacrificed (greatest MV picker)");
    assert!(g.battlefield.iter().any(|c| c.id == cheap_bear),
        "2-MV bear survives (lowest MV)");
}

// ── Lurking Predators ──────────────────────────────────────────────────────

#[test]
fn lurking_predators_drops_creature_when_opp_casts() {
    let mut g = two_player_game();
    let _predators = g.add_card_to_battlefield(0, catalog::lurking_predators());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());

    // Opp casts a spell (switch active player to opp)
    let opp_spell = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: opp_spell,
        target: Some(crate::game::types::Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp bolt castable");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "creature off top should land via Lurking Predators trigger");
}

// ── Prowling Caracal ───────────────────────────────────────────────────────

// ── Elvish Visionary ───────────────────────────────────────────────────────

#[test]
fn elvish_visionary_draws_on_etb() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::elvish_visionary());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Elvish Visionary castable");
    drain_stack(&mut g);
    // Hand: -1 cast + 1 ETB draw = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before, "ETB draw fires");
}

// ── Sungrass Egg ───────────────────────────────────────────────────────────

#[test]
fn sungrass_egg_sac_adds_two_mana_of_one_color() {
    let mut g = two_player_game();
    let egg = g.add_card_to_battlefield(0, catalog::sungrass_egg());
    g.clear_sickness(egg);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: egg,
        ability_index: 0,
        target: None, x_value: None }).expect("Egg activation");
    drain_stack(&mut g);

    // Egg sacrificed off the battlefield.
    assert!(!g.battlefield.iter().any(|c| c.id == egg), "egg sacrificed");
    // 2 mana of any one color in pool (auto-decider picks white).
    let total = g.players[0].mana_pool.amount(Color::White)
        + g.players[0].mana_pool.amount(Color::Blue)
        + g.players[0].mana_pool.amount(Color::Black)
        + g.players[0].mana_pool.amount(Color::Red)
        + g.players[0].mana_pool.amount(Color::Green);
    assert_eq!(total, 2, "added 2 colored mana of one color");
}

// ── Mascot Summoning ───────────────────────────────────────────────────────

#[test]
fn mascot_summoning_creates_a_two_two_lifelink_cat() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mascot_summoning());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mascot Summoning castable");
    drain_stack(&mut g);

    let cat = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.definition.name == "Cat"
    }).expect("Cat token minted");
    assert_eq!(cat.definition.power, 2);
    assert_eq!(cat.definition.toughness, 2);
    assert!(cat.definition.keywords.contains(&Keyword::Lifelink));
}

// ── Scry Inversion ─────────────────────────────────────────────────────────

#[test]
fn scry_inversion_scrys_and_draws_two() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::scry_inversion());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scry Inversion castable");
    drain_stack(&mut g);

    // -1 cast + 2 draws = +1 net
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew 2 net 1");
}

// ── Cunning Rhetoric ───────────────────────────────────────────────────────

#[test]
fn cunning_rhetoric_drains_on_opp_cast() {
    let mut g = two_player_game();
    let _rhetoric = g.add_card_to_battlefield(0, catalog::cunning_rhetoric());
    let life_us_before = g.players[0].life;
    let life_opp_before = g.players[1].life;

    // Opp casts a spell (switch active player to opp)
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);

    // After bolt and drain trigger:
    // Bolt: 3 dmg to us = life_us -3
    // Drain: us gain 1, opp loses 1
    assert_eq!(g.players[0].life, life_us_before - 3 + 1, "drain gain 1");
    assert_eq!(g.players[1].life, life_opp_before - 1, "drain loss 1");
}
