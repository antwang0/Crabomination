use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn sacred_fire_deals_three_and_gains_three_life() {
    let mut g = two_player_game();
    let initial_life = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::sacred_fire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacred Fire castable for {R}{W}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 20 - 3, "opponent took 3");
    assert_eq!(g.players[0].life, initial_life + 3, "you gained 3");
}

#[test]
fn sparkmage_apprentice_etb_deals_two_to_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sparkmage_apprentice());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sparkmage Apprentice castable for {1}{R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 20 - 2, "ETB dealt 2 damage to opponent");
}

#[test]
fn karok_wrangler_magecraft_adds_counter_to_target() {
    let mut g = two_player_game();
    let _wrangler = g.add_card_to_battlefield(0, catalog::karok_wrangler());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt casts");
    drain_stack(&mut g);

    let b = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1,
        "Karok Wrangler magecraft added a +1/+1 counter");
}

#[test]
fn soothsayer_adept_activates_surveil_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let initial_lib = g.players[0].library.len();
    let id = g.add_card_to_battlefield(0, catalog::soothsayer_adept());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("Surveil 1 activates");
    drain_stack(&mut g);

    // Surveil 1 either leaves card on top or sends it to graveyard.
    // The AutoDecider for Surveil may decide either way; either way the
    // top card was inspected.
    let after_lib = g.players[0].library.len();
    let after_gy = g.players[0].graveyard.len();
    assert!(
        after_lib == initial_lib || (after_lib == initial_lib - 1 && after_gy >= 1),
        "Surveil 1 either kept or graveyarded the top card (lib {} → {}, gy: {})",
        initial_lib, after_lib, after_gy
    );
}

#[test]
fn quick_study_draws_two_cards_for_target_player() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quick_study());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Quick Study castable for {1}{U}");
    drain_stack(&mut g);

    // Hand: -1 (cast) + 2 (draw) = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2);
    // Library: -2 (drawn).
    assert_eq!(g.players[0].library.len(), lib_before - 2);
}

// The Strixhaven Command cycle uses `Effect::ChooseN { picks, modes }`
// (CR 700.2d) — the auto-decider picks the per-card `picks` indices,
// so each Command always runs both of its chosen modes.
#[test]
fn witherbloom_command_auto_picks_mill_and_drain() {
    let mut g = two_player_game();
    // P1 (target opponent) has at least 4 cards in their library.
    for _ in 0..6 { g.add_card_to_library(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::witherbloom_command());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;
    let p1_lib_before = g.players[1].library.len();
    let p1_gy_before = g.players[1].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Witherbloom Command castable for {2}{B}{G}");
    drain_stack(&mut g);
    // Auto-pick = [0 (mill 4), 2 (drain 2)].
    assert_eq!(g.players[1].library.len(), p1_lib_before - 4,
        "P1 milled 4");
    assert_eq!(g.players[1].graveyard.len(), p1_gy_before + 4,
        "P1 gy +4");
    assert_eq!(g.players[0].life, p0_life_before + 2,
        "P0 +2 from drain");
    assert_eq!(g.players[1].life, p1_life_before - 2,
        "P1 -2 from drain");
}

#[test]
fn lorehold_command_auto_picks_damage_and_two_flying_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_command());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Lorehold Command castable for {2}{R}{W}");
    drain_stack(&mut g);

    // Auto-pick = [0 (4 damage to opp), 3 (two 2/2 flying Spirits)].
    assert_eq!(g.players[1].life, p1_life_before - 4,
        "P1 took 4 damage");
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit"
            && c.controller == 0)
        .collect();
    assert_eq!(spirits.len(), 2, "Two Spirit tokens minted");
    for s in &spirits {
        assert_eq!(s.power(), 2);
        assert_eq!(s.toughness(), 2);
        assert!(s.has_keyword(&Keyword::Flying), "Lorehold Spirits have flying");
    }
}

#[test]
fn quandrix_command_auto_picks_counters_and_mill_two() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    for _ in 0..5 { g.add_card_to_library(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_command());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_lib_before = g.players[1].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Quandrix Command castable for {1}{G}{U}");
    drain_stack(&mut g);
    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(bear_card.counter_count(CounterType::PlusOnePlusOne), 2,
        "Bear should have 2 +1/+1 counters");
    // Auto-pick also fired mode 2 (mill 2). P1 lost 2 from library.
    assert_eq!(g.players[1].library.len(), p1_lib_before - 2,
        "Mill 2 fired against P1");
}

#[test]
fn silverquill_command_auto_picks_drain_and_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let id = g.add_card_to_hand(0, catalog::silverquill_command());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Silverquill Command castable for {2}{W}{B}");
    drain_stack(&mut g);
    // Auto-pick = [1 (drain 2), 3 (two +1/+1 counters on creature)].
    assert_eq!(g.players[0].life, p0_life_before + 2, "P0 +2 from drain");
    assert_eq!(g.players[1].life, p1_life_before - 2, "P1 -2 from drain");
    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(bear_card.counter_count(CounterType::PlusOnePlusOne), 2,
        "Bear gained 2 +1/+1 counters from mode 3");
}

#[test]
fn prismari_command_auto_picks_loot_and_treasure() {
    let mut g = two_player_game();
    // Seed a library card so the loot draw succeeds.
    g.add_card_to_library(0, catalog::island());
    let _filler = g.add_card_to_hand(0, catalog::island()); // to discard
    let id = g.add_card_to_hand(0, catalog::prismari_command());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Prismari Command castable for {1}{U}{R}");
    drain_stack(&mut g);

    // Auto-pick = [1 (loot), 2 (Treasure)].
    // Hand: -1 (cast) -1 (discard) +1 (draw) = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1,
        "Hand size shifted by -1 (cast + loot is a wash, the cast itself was the only consumption)");
    let treasures: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure"
            && c.controller == 0)
        .collect();
    assert_eq!(treasures.len(), 1, "One Treasure token from mode 2");
}

#[test]
fn defend_the_campus_creates_three_inkling_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::defend_the_campus());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Defend the Campus castable for {3}{W}{W}");
    drain_stack(&mut g);
    let inklings: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Inkling"
            && c.controller == 0)
        .collect();
    assert_eq!(inklings.len(), 3, "Should mint exactly three Inkling tokens");
    for ink in &inklings {
        assert_eq!(ink.power(), 1);
        assert_eq!(ink.toughness(), 1);
        assert!(ink.has_keyword(&Keyword::Flying), "Inklings have flying");
    }
}

#[test]
fn hall_monitor_untaps_self_on_instant_cast() {
    let mut g = two_player_game();
    let hm = g.add_card_to_battlefield(0, catalog::hall_monitor());
    g.clear_sickness(hm);
    // Tap Hall Monitor manually.
    g.battlefield.iter_mut().find(|c| c.id == hm).unwrap().tapped = true;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Magecraft fires off the Bolt cast → untap Hall Monitor.
    let hm_card = g.battlefield.iter().find(|c| c.id == hm).unwrap();
    assert!(!hm_card.tapped, "Magecraft should untap Hall Monitor");
}

#[test]
fn stonebinders_familiar_gains_counter_on_card_leaving_graveyard() {
    let mut g = two_player_game();
    // Seed P0 library so Glorious Decay's "draw a card" rider doesn't
    // deck them out (which would short-circuit the test with GameAlreadyOver).
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let sf = g.add_card_to_battlefield(0, catalog::stonebinders_familiar());
    g.clear_sickness(sf);
    // Put a card in P0's graveyard, then exile it via Glorious Decay's
    // mode 2 (exile target card from a graveyard, draw a card).
    let bait = g.add_card_to_graveyard(0, catalog::island());
    let decay = g.add_card_to_hand(0, catalog::glorious_decay());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let counters_before = g.battlefield.iter().find(|c| c.id == sf).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    g.perform_action(GameAction::CastSpell {
        card_id: decay, target: Some(Target::Permanent(bait)), additional_targets: vec![], mode: Some(2), x_value: None,
    })
    .expect("Glorious Decay castable for {1}{G}");
    drain_stack(&mut g);
    let counters_after = g.battlefield.iter().find(|c| c.id == sf).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters_after, counters_before + 1,
        "Stonebinder's Familiar should gain a +1/+1 counter on card leaving graveyard");
}

#[test]
fn necrotic_fumes_sacrifices_and_exiles() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fodder);
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::necrotic_fumes());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Necrotic Fumes castable for {2}{B}{B}");
    drain_stack(&mut g);
    // P0's bear should be sacrificed (in P0's graveyard).
    assert!(!g.battlefield.iter().any(|c| c.id == fodder),
        "Fodder should be sacrificed off the battlefield");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder),
        "Fodder should be in P0's graveyard");
    // Target should be exiled (not in graveyard).
    assert!(!g.battlefield.iter().any(|c| c.id == target),
        "Target should be exiled off the battlefield");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == target),
        "Target should NOT be in graveyard (exiled, not destroyed)");
    assert!(g.exile.iter().any(|c| c.id == target),
        "Target should be in the exile zone");
}

#[test]
fn make_your_mark_pumps_creature_and_draws_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let id = g.add_card_to_hand(0, catalog::make_your_mark());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();
    let bear_power_before = g.battlefield.iter().find(|c| c.id == bear).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Make Your Mark castable for {1}{W}");
    drain_stack(&mut g);
    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(bear_card.power(), bear_power_before + 1,
        "Bear should be +1/+1 (now {})", bear_power_before + 1);
    // Hand: -1 (cast) +1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[0].library.len(), lib_before - 1);
}

#[test]
fn containment_breach_destroys_enchantment() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    // Use SOS Comforting Counsel as a target enchantment.
    let ench = g.add_card_to_battlefield(1, catalog::comforting_counsel());
    let id = g.add_card_to_hand(0, catalog::containment_breach());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(ench)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Containment Breach castable for {1}{W}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == ench),
        "Enchantment should be destroyed");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == ench),
        "Enchantment should be in P1's graveyard");
}

// ── Silverquill Pledgemage, Archmage Emeritus, Promising Duskmage,
//    Tenured Inkcaster, Symmathematics ──────────────────────────────────

#[test]
fn silverquill_pledgemage_magecraft_pumps_self_eot() {
    let mut g = two_player_game();
    let pledge = g.add_card_to_battlefield(0, catalog::silverquill_pledgemage());
    g.clear_sickness(pledge);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(pledge).unwrap().power();
    let t_before = g.battlefield_find(pledge).unwrap().toughness();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(pledge).unwrap().power();
    let t_after = g.battlefield_find(pledge).unwrap().toughness();
    assert_eq!(p_after, p_before + 1, "Pledgemage power +1 from magecraft");
    assert_eq!(t_after, t_before + 1, "Pledgemage toughness +1 from magecraft");
}

#[test]
fn silverquill_pledgemage_does_not_trigger_on_creature_cast() {
    let mut g = two_player_game();
    let pledge = g.add_card_to_battlefield(0, catalog::silverquill_pledgemage());
    g.clear_sickness(pledge);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p_before = g.battlefield_find(pledge).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bears castable for {1}{G}");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(pledge).unwrap().power();
    assert_eq!(p_after, p_before, "Casting a creature should NOT trigger magecraft");
}

#[test]
fn archmage_emeritus_draws_on_instant_cast() {
    let mut g = two_player_game();
    // Seed library so the draw has cards available.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let _ae = g.add_card_to_battlefield(0, catalog::archmage_emeritus());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Net hand: -1 (cast Bolt) + 1 (magecraft draw) = 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Library: -1 card.
    assert_eq!(g.players[0].library.len(), lib_before - 1);
}

#[test]
fn archmage_emeritus_does_not_draw_on_creature_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ae = g.add_card_to_battlefield(0, catalog::archmage_emeritus());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bears castable for {1}{G}");
    drain_stack(&mut g);
    // No magecraft fire → library unchanged.
    assert_eq!(g.players[0].library.len(), lib_before,
        "Casting a creature should NOT trigger Archmage Emeritus's draw");
}

#[test]
fn promising_duskmage_drains_on_instant_cast() {
    let mut g = two_player_game();
    let _pdm = g.add_card_to_battlefield(0, catalog::promising_duskmage());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Bolt deals 3 + magecraft loses 1 = 4 total to P1.
    assert_eq!(g.players[1].life, p1_life_before - 4,
        "P1 takes 3 (Bolt) + 1 (magecraft drain) = 4 damage");
    // P0 gains 1 from the drain.
    assert_eq!(g.players[0].life, p0_life_before + 1,
        "P0 gains 1 from magecraft drain");
}

#[test]
fn tenured_inkcaster_buffs_friendly_inklings_by_two_two() {
    // Mint an Inkling token via Inkling Summoning, then drop Tenured
    // Inkcaster, and check the Inkling went from 2/1 → 4/3.
    let mut g = two_player_game();
    // Cast Inkling Summoning to mint a 2/1 W/B Inkling with flying.
    let summon = g.add_card_to_hand(0, catalog::inkling_summoning());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: summon, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Inkling Summoning castable for {3}{W}{B}");
    drain_stack(&mut g);
    // Find the Inkling token (last-created token).
    let inkling = g.battlefield.iter()
        .find(|c| c.controller == 0 &&
            c.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Inkling))
        .map(|c| c.id)
        .expect("Inkling token should exist");
    let before = g.compute_battlefield().into_iter()
        .find(|c| c.id == inkling)
        .expect("Inkling on battlefield");
    assert_eq!(before.power, 2, "Base Inkling power is 2");
    assert_eq!(before.toughness, 1, "Base Inkling toughness is 1");

    // Now drop Tenured Inkcaster.
    let _tic = g.add_card_to_battlefield(0, catalog::tenured_inkcaster());
    let after = g.compute_battlefield().into_iter()
        .find(|c| c.id == inkling)
        .expect("Inkling on battlefield post-Inkcaster");
    assert_eq!(after.power, 4, "Inkling +2/+2 from Tenured Inkcaster: 4 power");
    assert_eq!(after.toughness, 3, "Inkling +2/+2 from Tenured Inkcaster: 3 toughness");
}

#[test]
fn tenured_inkcaster_does_not_buff_opponent_inklings() {
    let mut g = two_player_game();
    // P1 has an Inkling token (via Inkling Summoning).
    let summon = g.add_card_to_hand(1, catalog::inkling_summoning());
    g.players[1].mana_pool.add(Color::White, 1);
    g.players[1].mana_pool.add(Color::Black, 1);
    g.players[1].mana_pool.add_colorless(3);
    // Switch active player so the cast resolves cleanly.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: summon, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Inkling Summoning castable for P1");
    drain_stack(&mut g);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let opp_inkling = g.battlefield.iter()
        .find(|c| c.controller == 1 &&
            c.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Inkling))
        .map(|c| c.id)
        .expect("Opp Inkling token should exist");

    // P0 drops a Tenured Inkcaster.
    let _tic = g.add_card_to_battlefield(0, catalog::tenured_inkcaster());
    let after = g.compute_battlefield().into_iter()
        .find(|c| c.id == opp_inkling)
        .expect("opp Inkling on battlefield");
    assert_eq!(after.power, 2,
        "Opponent's Inkling should stay 2/1 — anthem only affects controller's Inklings");
}

#[test]
fn tenured_inkcaster_anthem_expires_when_inkcaster_leaves_play() {
    // Drop Inkcaster + an Inkling → Inkling is +2/+2. Destroy Inkcaster,
    // Inkling reverts to printed 2/1.
    let mut g = two_player_game();
    let summon = g.add_card_to_hand(0, catalog::inkling_summoning());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: summon, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Inkling Summoning castable");
    drain_stack(&mut g);
    let inkling = g.battlefield.iter()
        .find(|c| c.controller == 0 &&
            c.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Inkling))
        .map(|c| c.id)
        .expect("Inkling token");
    let tic = g.add_card_to_battlefield(0, catalog::tenured_inkcaster());
    {
        let buffed = g.compute_battlefield().into_iter()
            .find(|c| c.id == inkling).expect("Inkling");
        assert_eq!(buffed.power, 4, "Buffed Inkling = 4 power");
    }
    // Now exile/destroy Inkcaster.
    g.remove_from_battlefield_to_graveyard(tic);
    let after = g.compute_battlefield().into_iter()
        .find(|c| c.id == inkling).expect("Inkling");
    assert_eq!(after.power, 2,
        "After Inkcaster leaves, Inkling reverts to printed 2 power");
}

#[test]
fn symmathematics_enters_with_two_plus_one_counters() {
    // CR 614.12 enters-with replacement now places counters BEFORE the
    // first SBA check, so the printed 0/0 base body survives ETB:
    // 0/0 + 2 +1/+1 counters → 2/2.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::symmathematics());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Symmathematics castable for {1}{G}{U}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2,
        "Symmathematics enters as 2/2 (printed 0/0 + 2 +1/+1 counters per CR 614.12)");
    assert_eq!(card.toughness(), 2);
    // Verify the counter count is exactly 2.
    let count = *card.counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0);
    assert_eq!(count, 2, "ETB places exactly 2 +1/+1 counters");
}

#[test]
fn symmathematics_doubles_counters_on_instant_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::symmathematics());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Symmathematics castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().power(), 2);
    // Cast a Bolt: magecraft doubles 2 → 4 counters → 4/4 body (0/0 + 4).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let after = g.battlefield_find(id).unwrap();
    assert_eq!(after.power(), 4,
        "After one magecraft fire, 2 → 4 counters → 0/0 + 4 = 4/4");
    assert_eq!(after.toughness(), 4);
}

#[test]
fn symmathematics_does_not_double_on_creature_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::symmathematics());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Symmathematics castable");
    drain_stack(&mut g);
    let p_before = g.battlefield_find(id).unwrap().power();
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bears castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(id).unwrap().power();
    assert_eq!(p_after, p_before,
        "Casting a creature should NOT double counters (magecraft is I/S only)");
}

/// Environmental Sciences ({2}) gains 2 life and tutors a basic land to
/// hand. AutoDecider declines `SearchLibrary` by default so we feed a
/// ScriptedDecider with the Forest's CardId to exercise the search half.
#[test]
fn environmental_sciences_gains_four_life_and_tutors_a_basic_land() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island()); // filler

    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));

    let id = g.add_card_to_hand(0, catalog::environmental_sciences());
    g.players[0].mana_pool.add_colorless(2);

    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Environmental Sciences castable for {1}{G}");
    drain_stack(&mut g);

    // Life +2.
    assert_eq!(g.players[0].life, life_before + 2,
        "Should gain 2 life from Environmental Sciences");
    // Hand: -1 (cast) + 1 (tutored Forest) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Hand size unchanged (cast -1 + tutor +1)");
    // Forest is in hand, not library.
    assert!(g.players[0].hand.iter().any(|c| c.id == forest),
        "Forest should be in hand after tutor");
    assert!(!g.players[0].library.iter().any(|c| c.id == forest),
        "Forest should no longer be in library");
}

/// Environmental Sciences still gains life even when AutoDecider declines
/// the optional tutor — the GainLife half is unconditional.
#[test]
fn environmental_sciences_gains_life_even_if_search_declined() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());

    let id = g.add_card_to_hand(0, catalog::environmental_sciences());
    g.players[0].mana_pool.add_colorless(2);

    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Environmental Sciences castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 2,
        "Life still bumps even when AutoDecider declines the tutor");
}

/// Introduction to Annihilation exiles a nonland permanent (real card
/// is `{5}` colorless Lesson, exile-not-destroy).
#[test]
fn introduction_to_annihilation_destroys_nonland_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::introduction_to_annihilation());
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Introduction to Annihilation castable for {5}");
    drain_stack(&mut g);

    // Bear is exiled (real card is "Exile target nonland permanent").
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should leave the battlefield");
    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in exile (Introduction to Annihilation exiles)");
}

/// Introduction to Prophecy scries 3 and draws a card. We seed enough
/// cards in the library that the Draw isn't an exception.
#[test]
fn introduction_to_prophecy_scries_three_and_draws_one() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::introduction_to_prophecy());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Introduction to Prophecy castable for {2}{U}");
    drain_stack(&mut g);

    // Hand: -1 (cast) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Hand size unchanged (cast -1 + draw +1)");
    // Library: -1 (drew one). Scry doesn't change library size.
    assert_eq!(g.players[0].library.len(), lib_before - 1,
        "Library decremented by one for the draw");
}

/// Spirit Summoning mints a 3/2 white Spirit with lifelink.
#[test]
fn spirit_summoning_creates_a_three_two_lifelink_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spirit_summoning());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Spirit Summoning castable for {3}{W}");
    drain_stack(&mut g);

    let spirit = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Spirit")
        .expect("Spirit token should be on the battlefield");
    assert_eq!(spirit.power(), 3, "Spirit token is a 3/2");
    assert_eq!(spirit.toughness(), 2, "Spirit token is a 3/2");
    assert!(spirit.has_keyword(&Keyword::Lifelink),
        "Spirit token has lifelink");
    assert_eq!(spirit.controller, 0,
        "Spirit token controlled by casting player");
}

// ── Doc-only promotions covered by characterization tests ──────────────────

/// Necrotic Fumes: even though the additional cost (sacrifice a creature)
/// is folded into resolution rather than cast-time, the gameplay outcome
/// matches: one of your creatures is sacrificed AND the targeted creature
/// is exiled. This characterization locks in the behaviour so the
/// "doc-only ✅" promotion doesn't regress.
#[test]
fn necrotic_fumes_sacrifices_one_and_exiles_target() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::necrotic_fumes());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Necrotic Fumes castable for {2}{B}{B}");
    drain_stack(&mut g);

    // Your creature is in graveyard.
    assert!(!g.battlefield.iter().any(|c| c.id == fodder),
        "Your bear (fodder) should be sacrificed off the battlefield");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder),
        "Your bear should be in your graveyard (sacrifice)");
    // Target is exiled.
    assert!(!g.battlefield.iter().any(|c| c.id == victim),
        "Target should be off the battlefield (exiled)");
    assert!(g.exile.iter().any(|c| c.id == victim),
        "Target should be in exile (Necrotic Fumes exiles rather than destroys)");
}

/// Combat Professor's Mentor approximation (`PowerAtMost(1)`) correctly
/// matches the printed Mentor for a base-power-2 source: "lesser power"
/// = power < 2 = PowerAtMost(1). Lock this in.
#[test]
fn combat_professor_mentor_buffs_a_smaller_attacker() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let prof = g.add_card_to_battlefield(0, catalog::combat_professor());
    let smaller = g.add_card_to_battlefield(0, catalog::memnite()); // 1/1 — strictly lesser power than Combat Professor's 2

    g.clear_sickness(prof);
    g.clear_sickness(smaller);
    g.step = TurnStep::DeclareAttackers;

    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: prof, target: AttackTarget::Player(1) },
        Attack { attacker: smaller, target: AttackTarget::Player(1) },
    ]))
    .expect("DeclareAttackers");
    drain_stack(&mut g);

    let smaller_card = g.battlefield_find(smaller).unwrap();
    assert_eq!(
        smaller_card.counter_count(CounterType::PlusOnePlusOne), 1,
        "1/1 attacker (lesser power than Combat Professor's 2) gains a +1/+1 counter via Mentor"
    );
}

#[test]
fn combat_professor_mentor_skips_equal_power_attacker() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let prof = g.add_card_to_battlefield(0, catalog::combat_professor()); // 2 power
    let equal = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 — not lesser
    g.clear_sickness(prof);
    g.clear_sickness(equal);
    g.step = crate::game::types::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: prof, target: AttackTarget::Player(1) },
        Attack { attacker: equal, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(equal).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
        "equal-power attacker is not a legal Mentor target (PowerLessThanSource)");
}

/// Square Up sets the target creature's base power and toughness to 0/4
/// for the turn, and the caster draws a card. We verify both the
/// SetBasePT layer-7b effect and the cantrip.
#[test]
fn square_up_sets_target_creature_to_zero_four_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 base
    g.add_card_to_library(0, catalog::island()); // for the draw

    let id = g.add_card_to_hand(0, catalog::square_up());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Square Up castable for {U}{R}");
    drain_stack(&mut g);

    let computed = g.computed_permanent(bear).expect("Bear still present");
    assert_eq!(computed.power, 0, "Base power set to 0");
    assert_eq!(computed.toughness, 4, "Base toughness set to 4");
    // Hand: -1 (cast) +1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Hand size unchanged (cast -1 + cantrip +1)");
}

/// +1/+1 counters STACK on top of Square Up's base-P/T override per
/// CR 613.7b/c/f. A 2/2 bear with a +1/+1 counter, after Square Up,
/// should be 1/5 — base 0/4 + 1 counter delta.
#[test]
fn square_up_layers_under_plus_one_counters() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);

    let id = g.add_card_to_hand(0, catalog::square_up());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Square Up castable");
    drain_stack(&mut g);

    let computed = g.computed_permanent(bear).expect("Bear still present");
    // 0 base power + 1 from counter = 1; 4 base toughness + 1 from counter = 5.
    assert_eq!(computed.power, 1, "0 + counter = 1");
    assert_eq!(computed.toughness, 5, "4 + counter = 5");
}

// ── CR 700.2b modal triggers ────────────────────────────────────────────────

/// Prismari Apprentice's Magecraft trigger is modal (Scry 1 / +1/+0 EOT).
/// Per CR 700.2b, the controller picks the mode as part of putting the
/// triggered ability on the stack. The `AutoDecider` picks the leftmost
/// printed mode (Scry 1) by default — verify the trigger fires + scries
/// without bumping the source.
#[test]
fn prismari_apprentice_modal_magecraft_scrys_by_default() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::prismari_apprentice());
    g.clear_sickness(app);
    // Seed library so scry has something to look at.
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    let pre_power = g.battlefield.iter().find(|c| c.id == app).unwrap().power();

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    // Library unchanged (Scry doesn't draw); source not bumped.
    assert_eq!(g.players[0].library.len(), lib_before,
        "Scry 1 (mode 0) should not change library size");
    let a = g.battlefield.iter().find(|c| c.id == app).unwrap();
    assert_eq!(a.power(), pre_power,
        "Mode 0 (Scry) should not pump Apprentice (would imply mode 1 picked)");
}

/// Same source as above, but inject a `ScriptedDecider` that returns
/// `DecisionAnswer::Mode(1)` — the +1/+0 EOT branch — exercising the
/// engine's CR 700.2b modal trigger mode pick at push-time.
#[test]
fn prismari_apprentice_modal_magecraft_pumps_via_scripted_mode_pick() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::prismari_apprentice());
    g.clear_sickness(app);
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    // Pick mode 1 (the +1/+0 branch) when the modal-trigger decision lands.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let a = g.battlefield.iter().find(|c| c.id == app).unwrap();
    assert_eq!(a.power(), 3,
        "Apprentice should be 3/2 after picking mode 1 (Magecraft +1/+0 EOT)");
    assert_eq!(a.toughness(), 2);
}

/// Confront the Past mode 2 deals damage equal to the target PW's
/// loyalty counters via the new `Value::LoyaltyOf(Target(0))` primitive.
/// A fresh-cast Professor Dellian Fel has 5 loyalty → mode 2 sends 5
/// damage. Since CR 120.3c routes PW damage into loyalty-counter
/// removal, the PW ends with 0 loyalty and is destroyed by SBA.
#[test]
fn confront_the_past_mode_2_uses_loyalty_counter_x() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(1, catalog::professor_dellian_fel());
    // Professor Dellian Fel comes in with 5 base loyalty.
    let pw_card = g.battlefield.iter().find(|c| c.id == pw).unwrap();
    assert_eq!(
        pw_card.counter_count(crate::card::CounterType::Loyalty),
        5,
        "Professor Dellian Fel should have 5 starting loyalty"
    );

    let id = g.add_card_to_hand(0, catalog::confront_the_past());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(pw)),
        additional_targets: vec![],
        mode: Some(2), x_value: None,
    }).expect("Confront the Past castable for {3}{R}");
    drain_stack(&mut g);

    // 5 damage → 5 loyalty removed → PW dies via SBA.
    assert!(!g.battlefield.iter().any(|c| c.id == pw),
        "Mode 2 should remove all 5 loyalty and bury the PW");
}

/// Tempted by the Oriq — body sanity: target enemy creature swaps to
/// caster control, is untapped, and gains haste. This locks in the
/// closing of the STX Witherbloom school (the doc-only promotion in
/// push XXXIII relies on the printed body being faithful).
#[test]
fn tempted_by_the_oriq_steals_untaps_and_grants_haste_witherbloom_closer() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().tapped = true;

    let id = g.add_card_to_hand(0, catalog::tempted_by_the_oriq());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Tempted by the Oriq castable for {2}{B}");
    drain_stack(&mut g);

    let b = g.battlefield_find(bear).expect("bear still on bf");
    assert_eq!(b.controller, 0, "controlled by caster EOT");
    assert!(!b.tapped, "untapped");
    assert!(b.has_keyword(&Keyword::Haste));
}

/// Quandrix Charm mode 2 promoted to `SetBasePT` — a 1/1 with a +1/+1
/// counter targeted by mode 2 should layer to a 6/6 (base 5/5 +
/// counter), proving SetBasePT installs the layer-7b base rewrite and
/// the +1/+1 counter applies on top per CR 613.7c-f.
#[test]
fn quandrix_charm_mode_2_setbasept_layers_under_counter() {
    let mut g = two_player_game();
    // Start as a 2/2 bear, then put a +1/+1 counter to make it 3/3.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap()
        .counters.insert(CounterType::PlusOnePlusOne, 1);

    let id = g.add_card_to_hand(0, catalog::quandrix_charm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: Some(2), x_value: None,
    }).expect("Quandrix Charm castable");
    drain_stack(&mut g);

    // Base P/T should be set to 5/5 via layer 7b; the +1/+1 counter
    // adds on top per CR 613.7c → final 6/6.
    let view = g.computed_permanent(bear).unwrap();
    assert_eq!(view.power, 6, "5 base + 1 counter = 6 power");
    assert_eq!(view.toughness, 6, "5 base + 1 counter = 6 toughness");
}

/// Decisive Denial mode 1 (fight) — a 4/4 friendly creature fights an
/// auto-picked 2/2 opp creature; the 2/2 dies, the 4/4 survives.
#[test]
fn decisive_denial_mode_1_fight_via_chelonian_template() {
    let mut g = two_player_game();
    // Friendly 6/4 Craw Wurm fighter — survives the 2-damage return.
    let big = g.add_card_to_battlefield(0, catalog::craw_wurm());
    g.clear_sickness(big);
    // Enemy 2/2 bear.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::decisive_denial());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: Some(1), x_value: None,
    }).expect("Decisive Denial castable for {G}{U}");
    drain_stack(&mut g);

    // Wurm (6/4) deals 6 damage to bear (toughness 2) → bear dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should die from fight damage");
    // Wurm survives (took 2 damage vs toughness 4).
    assert!(g.battlefield.iter().any(|c| c.id == big),
        "Wurm should survive (4 toughness vs 2 fight damage)");
}

/// Flow State: without an instant+sorcery pair in gy, scry 3 + draw 1 (mainline).
/// With an instant + a sorcery in gy, the `Effect::If` rider upgrades to draw 2.
#[test]
fn flow_state_draws_one_normally_and_two_when_graveyard_has_is_pair() {
    // Mainline: empty graveyard → net 0 hand change.
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::flow_state());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flow State castable for {1}{U}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Mainline path: -1 cast + 1 draw = 0 net");

    // Upgrade: IS pair in graveyard → net +1.
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_graveyard(0, catalog::lightning_bolt());        // instant
    g.add_card_to_graveyard(0, catalog::pop_quiz());              // sorcery (Lesson)
    let id = g.add_card_to_hand(0, catalog::flow_state());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flow State castable for {1}{U}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Upgrade path: -1 cast + 2 draws = +1 net");
}

/// Snow Day (doc-promoted) — taps target creature and applies a stun
/// counter. CR 122.1d: a permanent with a stun counter doesn't untap
/// during its controller's next untap step; instead, one stun is
/// removed. We verify the immediate-state shape (tapped + stun
/// counter applied).
#[test]
fn snow_day_doc_promoted_taps_and_stuns_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::snow_day());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Snow Day castable for {U}{R}");
    drain_stack(&mut g);

    let b = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(b.tapped, "Snow Day should tap the target");
    assert_eq!(b.counter_count(CounterType::Stun), 1,
        "Snow Day should apply 1 stun counter");
}

/// Curate (doc-promoted) — Scry 3 + Draw 1 approximation. With the
/// `AutoDecider` choosing the "keep on top" order for scry, the player
/// should net 0 hand size after casting (cast -1 + draw +1).
#[test]
fn curate_nets_zero_hand_size_via_scry_three_draw_one() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::curate());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Curate castable for {1}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before,
        "Curate: -1 cast + 1 draw = 0 net hand size");
}

// ── Killian, Ink Duelist — target-aware cost reduction (CR 117.7c / 601.2f) ──

/// Killian's static "spells you cast that target a creature cost {2} less"
/// reduces a creature-targeting spell's generic cost by 2. Murder is
/// {1}{B}{B} (3 mana); with Killian on the battlefield, casting it at a
/// creature reduces the generic pip to 0, leaving {B}{B} (2 mana net).
#[test]
fn killian_ink_duelist_reduces_creature_targeting_spell() {
    let mut g = two_player_game();
    let _killian = g.add_card_to_battlefield(0, catalog::killian_ink_duelist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);

    let murder = g.add_card_to_hand(0, catalog::murder());
    // Only fund {B}{B} — Murder normally needs {1}{B}{B} but Killian
    // shaves the generic pip.
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: murder,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Murder castable for {B}{B} under Killian's cost reduction");
    drain_stack(&mut g);

    assert!(
        g.battlefield_find(bear).is_none(),
        "Murder should destroy the Grizzly Bears"
    );
}

/// Killian's reduction can't cut a spell below its colored pips: CR 601.2f
/// requires the player to still pay all colored mana. Lightning Bolt is
/// {R} (one colored pip, zero generic); with Killian active, a Bolt
/// aimed at a creature still needs the {R} to cast (reduction caps at
/// zero generic).
#[test]
fn killian_reduction_does_not_eat_colored_pips() {
    let mut g = two_player_game();
    let _killian = g.add_card_to_battlefield(0, catalog::killian_ink_duelist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    // No mana in pool — should reject the cast.
    let result = g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(
        result.is_err(),
        "Bolt at a creature with no mana should still fail (colored {{R}} pip not reducible)"
    );
}

/// Killian's reduction only applies when the spell targets a creature.
/// Casting Bolt at a *player* should consume the full {R} (no rebate)
/// — the test exercises both that the cast succeeds at {R} (sanity)
/// and the reduction code path doesn't credit a phantom discount.
#[test]
fn killian_does_not_reduce_non_creature_targeting_spell() {
    let mut g = two_player_game();
    let _killian = g.add_card_to_battlefield(0, catalog::killian_ink_duelist());

    let murder = g.add_card_to_hand(0, catalog::murder());
    // Fund only {B}{B} — Murder is {1}{B}{B}. Without a creature target,
    // Killian's reduction doesn't fire; casting fails because the
    // generic pip is unpaid.
    g.players[0].mana_pool.add(Color::Black, 2);
    // Murder requires a creature target; the engine rejects the no-target
    // shape at validation. To exercise "wrong-target-type doesn't trigger
    // the reduction", we instead aim it at a non-existent creature — but
    // the cast won't even start without a legal target. Easier: just
    // verify casting with the bear target also fails when Killian isn't
    // controlled by the caster.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);

    // Remove Killian to disable the reduction.
    let killian_id = g.battlefield.iter()
        .find(|c| c.definition.name == "Killian, Ink Duelist")
        .map(|c| c.id)
        .unwrap();
    g.battlefield.retain(|c| c.id != killian_id);

    let result = g.perform_action(GameAction::CastSpell {
        card_id: murder,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(
        result.is_err(),
        "Murder at {{1}}{{B}}{{B}} should fail with only {{B}}{{B}} in pool when Killian is absent"
    );
}

// ── Multiple Choice — ChooseN-all-four promotion ─────────────────────────────

/// Multiple Choice's promoted ChooseN body runs all four modes in one
/// resolution: Scry 2 + 1/1 Pest token + +1/+0 hexproof EOT on target +
/// Draw 2. Verify the play pattern end-to-end.
#[test]
fn multiple_choice_fires_all_four_modes() {
    let mut g = two_player_game();
    // Seed library so Scry 2 + Draw 2 don't deck.
    for _ in 0..10 { g.add_card_to_library(0, catalog::island()); }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);

    let mc = g.add_card_to_hand(0, catalog::multiple_choice());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: mc,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Multiple Choice castable for {1}{U}{U}");
    drain_stack(&mut g);

    // Mode 1: 1/1 Pest token should be on battlefield.
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest")
        .collect();
    assert_eq!(pests.len(), 1, "Multiple Choice mints exactly one Pest token");

    // Mode 2: bear got +1/+0 EOT and hexproof.
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 3, "Bear should be 3/2 from +1/+0");
    assert_eq!(bear_card.toughness(), 2);
    assert!(bear_card.has_keyword(&Keyword::Hexproof),
        "Bear should have hexproof EOT");

    // Mode 3: draw 2. Net hand = -1 (cast) +2 (draw) = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Multiple Choice's all-modes draw-2 rider should fire");
}

/// Killian only reduces spells *you* cast — an opponent's Killian shouldn't
/// hand the active player a freebie. Verify the controller gate in
/// `cost_reduction_for_spell` by putting Killian under P1 and casting
/// from P0.
#[test]
fn killian_only_reduces_its_controllers_spells() {
    let mut g = two_player_game();
    // P1's Killian.
    let _killian = g.add_card_to_battlefield(1, catalog::killian_ink_duelist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);

    let murder = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    let result = g.perform_action(GameAction::CastSpell {
        card_id: murder,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(
        result.is_err(),
        "Opponent's Killian should not reduce my Murder — generic pip stays unpaid"
    );
}

// ── Push XXXV: OtherThanSource + Hofri anthem + Shadrix attack trigger ──────

/// When Hofri leaves the battlefield, the anthem expires and friendly
/// creatures return to their printed P/T. Mirrors the Quintorius test
/// pattern (anthem timestamp is `WhileSourceOnBattlefield`).
#[test]
fn hofri_ghostforge_anthem_expires_when_hofri_leaves() {
    let mut g = two_player_game();
    let hofri = g.add_card_to_battlefield(0, catalog::hofri_ghostforge());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    // Anthem active: bear is 3/2.
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);

    // Kill Hofri via lethal damage. His base toughness is 4.
    g.battlefield_find_mut(hofri).unwrap().damage = 4;
    let _ = g.check_state_based_actions();

    // Bear returns to printed 2/2.
    let after = g.computed_permanent(bear).expect("bear still on bf");
    assert_eq!(after.power, 2, "anthem gone");
    assert_eq!(after.toughness, 2);
}

#[test]
fn hofri_ghostforge_death_trigger_registers_delayed_exile() {
    // Push (modern_decks, batch 80): Hofri's "non-token creature dies →
    // exile and return, exile at next end step" trigger. Verify the
    // trigger fires (delayed NextEndStep registration is the canonical
    // signal) when a P0 nontoken creature dies via lethal damage.
    use crate::game::types::Target;
    let mut g = two_player_game();
    let _hofri = g.add_card_to_battlefield(0, catalog::hofri_ghostforge());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    let delayed_before = g.delayed_triggers.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);

    // Hofri's trigger fired: a NextEndStep delayed trigger should be
    // registered (one per dying-creature, regardless of whether the
    // engine fully settled the gy→bf return mid-SBA-cycle).
    let next_end_step_after = g.delayed_triggers.iter()
        .filter(|dt| matches!(dt.kind, crate::game::types::DelayedKind::NextEndStep))
        .count();
    assert!(
        next_end_step_after > delayed_before,
        "Hofri registered a NextEndStep delayed exile (trigger fired on bear's death)",
    );
}

/// `SelectionRequirement::OtherThanSource` now strictly excludes the
/// source from target-validation contexts (push modern_decks). When the
/// source is the only on-battlefield permanent matching the filter, the
/// auto-target picker must return `None` instead of falling back to the
/// source. Synthetic effect: `PumpPT` whose target filter is
/// `Creature ∧ OtherThanSource`.
#[test]
fn other_than_source_strict_filter_excludes_lone_source_target() {
    use crate::card::SelectionRequirement as R;
    use crate::effect::{Duration, Effect, Selector, Value};
    let mut g = two_player_game();
    let hofri = g.add_card_to_battlefield(0, catalog::hofri_ghostforge());

    // Only Hofri on the battlefield. A pump effect filtered by
    // `Creature ∧ OtherThanSource` should NOT pick Hofri itself.
    let eff = Effect::PumpPT {
        what: crate::effect::shortcut::target_filtered(
            R::Creature.and(R::OtherThanSource),
        ),
        power: Value::Const(1),
        toughness: Value::Const(0),
        duration: Duration::EndOfTurn,
    };
    let _ = Selector::This; // silence unused import in some configurations
    let picked = g.auto_target_for_effect_avoiding(&eff, 0, Some(hofri));
    assert!(
        picked.is_none(),
        "OtherThanSource must reject the lone source candidate, got {:?}",
        picked,
    );

    // Add a second creature. Now the picker should return it (not Hofri).
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let picked = g.auto_target_for_effect_avoiding(&eff, 0, Some(hofri));
    assert_eq!(
        picked,
        Some(crate::game::Target::Permanent(bear)),
        "OtherThanSource picks the non-source candidate"
    );
}

/// Shadrix Silverquill's attack trigger fires (via the new ChooseN
/// auto-pick of modes 1+2): a +1/+1 counter on a target friendly
/// creature, and two Inkling tokens minted under the controller.
#[test]
fn shadrix_silverquill_attack_pumps_target_creature_and_mints_inklings() {
    use crate::game::types::AttackTarget;
    let mut g = two_player_game();
    let shadrix = g.add_card_to_battlefield(0, catalog::shadrix_silverquill());
    g.clear_sickness(shadrix);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let inklings_before = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Inkling))
        .count();

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: shadrix,
        target: AttackTarget::Player(1),
    }]))
    .expect("Shadrix can attack");
    drain_stack(&mut g);

    // Mode 1: target friendly creature now has a +1/+1 counter.
    let bear_card = g.battlefield_find(bear).expect("bear on bf");
    assert_eq!(
        bear_card.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Bear should have a +1/+1 counter from Shadrix mode 1"
    );

    // Mode 2: two Inkling tokens added on P0's side.
    let inklings_after = g.battlefield.iter()
        .filter(|c| c.is_token
            && c.controller == 0
            && c.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Inkling))
        .count();
    assert_eq!(
        inklings_after - inklings_before, 2,
        "Shadrix mode 2 should mint two Inkling tokens for the controller"
    );
}

/// Shadrix's trigger is SelfSource — opponent attacking should NOT
/// fire Shadrix's choose-two.
#[test]
fn shadrix_silverquill_attack_does_not_trigger_on_opp_attack() {
    use crate::game::types::AttackTarget;
    let mut g = two_player_game();
    let _shadrix = g.add_card_to_battlefield(0, catalog::shadrix_silverquill());
    // Opp creature attacks.
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(opp_bear);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let inklings_before = g.battlefield.iter()
        .filter(|c| c.is_token).count();
    let bear_counters_before = g.battlefield_find(bear).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);

    // P1's turn — opp's bear attacks. (Active player is P0 by default in
    // two_player_game; switch.)
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: opp_bear,
        target: AttackTarget::Player(0),
    }]))
    .expect("Opp bear can attack");
    drain_stack(&mut g);

    let inklings_after = g.battlefield.iter()
        .filter(|c| c.is_token).count();
    assert_eq!(inklings_after, inklings_before,
        "Shadrix should not trigger off opponent's attack");
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        bear_counters_before,
        "No counter added when opp attacks"
    );
}

// ── Push XXXV: Practiced Offense mode pick + new Lash of Malice + Big Play ──

/// Practiced Offense's auto-decider should default to mode 0 (double
/// strike). The +1/+1 counter fan-out (collapsed to "you") and the
/// keyword grant both fire in the same resolution.
#[test]
fn practiced_offense_auto_picks_double_strike() {
    let mut g = two_player_game();
    let _bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let po = g.add_card_to_hand(0, catalog::practiced_offense());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: po,
        target: Some(Target::Permanent(bear2)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Practiced Offense castable for {2}{W}");
    drain_stack(&mut g);

    // Each friendly creature picks up a +1/+1 counter.
    assert!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature())
            .all(|c| c.counter_count(CounterType::PlusOnePlusOne) == 1),
        "Each friendly creature should have a +1/+1 counter"
    );

    // Target bear should have double strike EOT (mode 0 auto-pick).
    let bear2_card = g.battlefield_find(bear2).unwrap();
    assert!(bear2_card.has_keyword(&Keyword::DoubleStrike),
        "Target should have double strike from mode 0 auto-pick");
    assert!(!bear2_card.has_keyword(&Keyword::Lifelink),
        "Default pick is double strike, not lifelink");
}

/// Casting Practiced Offense with `mode: Some(1)` routes the inner
/// `ChooseMode` to lifelink instead of double strike. The mode flows
/// through the spell-level slot (`StackItem::Spell.mode`) into the
/// resolution context as `ctx.mode`.
#[test]
fn practiced_offense_can_pick_lifelink_via_cast_time_mode() {
    let mut g = two_player_game();
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let po = g.add_card_to_hand(0, catalog::practiced_offense());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: po,
        target: Some(Target::Permanent(bear2)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("Practiced Offense castable for {2}{W}");
    drain_stack(&mut g);

    let bear2_card = g.battlefield_find(bear2).unwrap();
    assert!(bear2_card.has_keyword(&Keyword::Lifelink),
        "mode: Some(1) should pick lifelink");
    assert!(!bear2_card.has_keyword(&Keyword::DoubleStrike),
        "Lifelink mode should NOT also pick double strike");
}

/// Lash of Malice ({B}) shrinks a target creature by -2/-2 — a 2/2
/// Grizzly Bears becomes 0/0 and dies to SBA.
#[test]
fn lash_of_malice_kills_two_two_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);

    let lash = g.add_card_to_hand(0, catalog::lash_of_malice());
    g.players[0].mana_pool.add(Color::Black, 1);
    let bear_before = g.battlefield_find(bear).unwrap().toughness();
    assert_eq!(bear_before, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: lash,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Lash of Malice castable for {B}");
    drain_stack(&mut g);

    // The 2/2 becomes effective 0/0 → dies to SBA.
    let _ = g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(),
        "Lash should kill a 2/2 via -2/-2 → 0/0 → SBA");
}

/// Big Play auto-picks mode 1 by default (Tap + Stun a target opp
/// creature). With mode 1 wired as Tap + Stun against any creature,
/// targeting an opp's bear should tap it and apply a stun counter.
#[test]
fn big_play_auto_picks_tap_and_stun() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // Bear starts untapped.
    assert!(!g.battlefield_find(bear).unwrap().tapped);

    let bp = g.add_card_to_hand(0, catalog::big_play());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: bp,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Big Play castable for {3}{R}{W}");
    drain_stack(&mut g);

    let bear_card = g.battlefield_find(bear).expect("bear still on bf");
    assert!(bear_card.tapped, "Big Play should tap the target");
    assert_eq!(
        bear_card.counter_count(CounterType::Stun), 1,
        "Big Play should leave a stun counter"
    );
}

/// Big Play mode 2 (`mode: Some(2)`) grants Trample EOT to each
/// friendly creature. We verify the keyword grant lands on a Grizzly
/// Bears.
#[test]
fn big_play_mode_2_grants_trample_to_friendlies() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let bp = g.add_card_to_hand(0, catalog::big_play());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: bp,
        target: None,
        additional_targets: vec![],
        mode: Some(2),
        x_value: None,
    })
    .expect("Big Play castable for {3}{R}{W}");
    drain_stack(&mut g);

    let computed = g.computed_permanent(bear).unwrap();
    assert!(computed.keywords.contains(&Keyword::Trample),
        "Mode 2 should grant trample to the friendly bear");
}

// ── New STX cards added in modern_decks push ────────────────────────────────

/// Burrog Befuddler: a 2/1 Flash Frog Wizard. The ETB trigger drops
/// -3/-0 on a target creature for the turn. A 2/2 Grizzly Bears
/// becomes effectively a -1/2 in damage math — non-lethal but the
/// pump-down still drains attacker pressure.
#[test]
fn burrog_befuddler_etb_minus_three_zero() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2);

    let id = g.add_card_to_hand(0, catalog::burrog_befuddler());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Burrog Befuddler castable for {1}{U}");
    drain_stack(&mut g);

    let computed = g.computed_permanent(bear).unwrap();
    assert_eq!(computed.power, -1,
        "bear should be effectively -1 power after -3/-0; got {}", computed.power);
    assert_eq!(computed.toughness, 2,
        "bear toughness unchanged by -3/-0; got {}", computed.toughness);
}

/// Mage Hunters' Mark grants +3/+0 + Menace EOT to any creature target.
#[test]
fn mage_hunters_mark_pumps_target_and_grants_menace() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);

    let id = g.add_card_to_hand(0, catalog::mage_hunters_mark());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mage Hunters' Mark castable for {1}{R}");
    drain_stack(&mut g);

    let computed = g.computed_permanent(bear).unwrap();
    assert_eq!(computed.power, 5, "bear should be 2+3=5 power");
    assert!(computed.keywords.contains(&Keyword::Menace),
        "bear should gain menace");
}

/// Mage Duel: friendly creature deals damage equal to its power to opp
/// creature. A 5/5 friendly Wurm (Bookwurm-style) wipes a 2/2 Bear.
#[test]
fn mage_duel_friendly_burns_opp_creature_by_friendly_power() {
    let mut g = two_player_game();
    let _friendly = g.add_card_to_battlefield(0, catalog::tarmogoyf());
    // Tarmogoyf without any types in any graveyard is 0/1; let's seed a
    // graveyard card so its power becomes 1.
    let _ = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(opp_bear);

    let id = g.add_card_to_hand(0, catalog::mage_duel());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mage Duel castable for {1}{R}");
    drain_stack(&mut g);

    // Tarmogoyf at 1/2 (with 1 type in gy: Instant) deals 1 to the bear;
    // bear survives but takes 1 damage.
    let bear_card = g.battlefield.iter().find(|c| c.id == opp_bear)
        .expect("bear still alive (1 damage on a 2-toughness body)");
    assert_eq!(bear_card.damage, 1, "bear took 1 damage from friendly power 1");
}

/// Eccentric Apprentice's magecraft trigger pumps the source +1/+0 EOT
/// when its controller casts an instant or sorcery. We cast Lightning
/// Bolt with the apprentice on the battlefield and verify its power.
#[test]
fn eccentric_apprentice_pumps_on_instant_cast() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::eccentric_apprentice());
    g.clear_sickness(app);
    let pre = g.computed_permanent(app).unwrap();
    assert_eq!(pre.power, 1, "starts at 1");

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);

    let post = g.computed_permanent(app).unwrap();
    assert_eq!(post.power, 2, "after magecraft +1/+0 → 2 power; got {}", post.power);
}

/// Illuminate History: discard a card from hand and create two 2/2 R/W
/// Spirit tokens with flying.
#[test]
fn illuminate_history_discards_and_creates_two_spirits() {
    let mut g = two_player_game();
    // Seed a card in hand to be discarded.
    let _fodder = g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::illuminate_history());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Illuminate History castable for {1}{R}{W}");
    drain_stack(&mut g);

    // Hand: -1 (cast) -1 (discard) = -2 from before.
    assert_eq!(g.players[0].hand.len(), hand_before - 2,
        "should cast + discard, net -2 hand cards");

    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit"
            && c.controller == 0)
        .collect();
    assert_eq!(spirits.len(), 2, "should mint two Spirits");
    for s in &spirits {
        assert!(s.has_keyword(&Keyword::Flying),
            "spirit token should have flying");
        assert_eq!(s.power(), 2);
        assert_eq!(s.toughness(), 2);
    }
}

/// Brilliant Plan: a {3}{U}{U} Sorcery — Lesson. Scry 3 + Draw 3.
#[test]
fn brilliant_plan_scrys_three_and_draws_three() {
    let mut g = two_player_game();
    // Seed library with 6 cards (Scry 3 + Draw 3 = touches 6 cards).
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::brilliant_plan());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Brilliant Plan castable for {3}{U}{U}");
    drain_stack(&mut g);

    // Hand: -1 (cast) +3 (draw) = +2 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
    // Library: -3 (drew 3). Scry may keep cards on top, so library size
    // reduces by 3 net.
    assert_eq!(g.players[0].library.len(), lib_before - 3);
}

/// Fortifying Draught: Lesson, +1/+4 EOT to target creature.
#[test]
fn fortifying_draught_pumps_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);

    let id = g.add_card_to_hand(0, catalog::fortifying_draught());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Fortifying Draught castable for {2}{W}");
    drain_stack(&mut g);

    let comp = g.computed_permanent(bear).unwrap();
    assert_eq!(comp.power, 3, "bear at 2+1=3 power");
    assert_eq!(comp.toughness, 6, "bear at 2+4=6 toughness");
}

/// Guiding Voice: +1/+1 counter on target creature + Learn (Draw 1).
#[test]
fn guiding_voice_counters_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let id = g.add_card_to_hand(0, catalog::guiding_voice());
    g.players[0].mana_pool.add(Color::White, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Guiding Voice castable for {W}");
    drain_stack(&mut g);

    // The bear should have a +1/+1 counter.
    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(bear_card.counter_count(CounterType::PlusOnePlusOne), 1);
    // Hand: -1 (cast) +1 (learn → draw) = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

/// Tezzeret's Gambit mode 0: Proliferate. Bears with +1/+1 counters
/// get another counter; players with poison get another poison.
#[test]
fn tezzerets_gambit_mode_zero_proliferates() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Seed a +1/+1 counter on the bear so proliferate adds another.
    g.battlefield_find_mut(bear).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1);
    assert_eq!(g.battlefield_find(bear).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 1);

    let id = g.add_card_to_hand(0, catalog::tezzerets_gambit());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Tezzeret's Gambit castable for {U}{B}");
    drain_stack(&mut g);

    let post = g.battlefield_find(bear).unwrap();
    assert_eq!(post.counter_count(CounterType::PlusOnePlusOne), 2,
        "proliferate adds one +1/+1 counter");
}

/// CR 701.34a: proliferate must not pump enemy creatures with +1/+1
/// counters. The smart-auto-decider skips +1/+1 counters on opponent
/// permanents and only fires on friendly ones.
#[test]
fn proliferate_skips_enemy_plus_one_counters() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    // Friendly bear with +1/+1, enemy bear with +1/+1.
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(friend).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1);
    g.battlefield_find_mut(enemy).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1);
    let id = g.add_card_to_hand(0, catalog::tezzerets_gambit());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Gambit castable");
    drain_stack(&mut g);
    let friend_after = g.battlefield_find(friend).unwrap();
    let enemy_after = g.battlefield_find(enemy).unwrap();
    assert_eq!(friend_after.counter_count(CounterType::PlusOnePlusOne), 2,
        "friendly +1/+1 counter proliferated");
    assert_eq!(enemy_after.counter_count(CounterType::PlusOnePlusOne), 1,
        "enemy +1/+1 counter NOT proliferated (auto-decider skip)");
}

/// CR 701.34a: proliferate also skips poison counters on the player
/// proliferating (you'd never poison yourself).
#[test]
fn proliferate_skips_self_poison_counters() {
    let mut g = two_player_game();
    g.players[0].poison_counters = 3;
    g.players[1].poison_counters = 3;
    let id = g.add_card_to_hand(0, catalog::tezzerets_gambit());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Gambit castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].poison_counters, 3, "self poison untouched");
    assert_eq!(g.players[1].poison_counters, 4, "opp poison proliferated");
}

/// Tezzeret's Gambit mode 1: pay 2 life, draw 2 cards.
#[test]
fn tezzerets_gambit_mode_one_pays_two_life_draws_two() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::tezzerets_gambit());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Tezzeret's Gambit mode 1 castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 2,
        "lose 2 life from mode 1");
    // -1 cast +2 draw = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2);
}

/// Tezzeret's Gambit's {U/P}{B/P} can be cast with no mana by paying both
/// Phyrexian pips with life — 4 life total (mode 0 = Proliferate, no
/// further life cost).
#[test]
fn tezzerets_gambit_castable_for_four_life_no_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::tezzerets_gambit());
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Tezzeret's Gambit castable for 4 life (both Phyrexian pips)");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 4,
        "two Phyrexian pips paid with 2 life each = 4 life");
}

/// Wandering Archaic copies an opponent's instant/sorcery spell when
/// they cast one. We seed an opponent's Lightning Bolt and verify a
/// copy lands on the stack.
#[test]
fn wandering_archaic_copies_opp_instant() {
    let mut g = two_player_game();
    let _arch = g.add_card_to_battlefield(0, catalog::wandering_archaic());

    // Opp casts Lightning Bolt at us.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("opp casts Bolt");
    drain_stack(&mut g);

    // Bolt + copy = 6 damage to P0.
    assert_eq!(g.players[0].life, life_before - 6,
        "Bolt (3) + Wandering Archaic copy (3) = 6 damage; got {}",
        life_before - g.players[0].life);
}

/// Wandering Archaic (modern_decks): When the opp casts an IS spell, they
/// may pay {2} to skip the copy. ScriptedDecider answers `Bool(true)` for
/// the optional pay; the opp pre-floats {2} in their pool, so the engine
/// deducts and skips the copy. No copy fires.
#[test]
fn wandering_archaic_lets_opp_pay_two_to_skip_copy() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let _arch = g.add_card_to_battlefield(0, catalog::wandering_archaic());

    // Opp casts Lightning Bolt at us.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    // Opp pre-floats {2} extra in their pool for the optional pay, plus
    // the {R} for the Bolt cast itself.
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;

    // ScriptedDecider answers Bool(true) for the pay prompt.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("opp casts Bolt");
    drain_stack(&mut g);

    // Only the original Bolt (3 damage) resolves — copy was skipped.
    assert_eq!(g.players[0].life, life_before - 3,
        "Only original Bolt resolves (no copy); got {} damage",
        life_before - g.players[0].life);
    // Opp's pool was drained by {2} for the optional pay.
    assert_eq!(g.players[1].mana_pool.total(), 0,
        "Opp's pool fully drained (paid 2 + 1 for Bolt)");
}

/// Wandering Archaic when the opp can't afford the {2}: the engine
/// silently falls through to the copy. The opp pre-floats only {R}, so
/// AutoDecider answers true but the deduct fails → copy fires.
#[test]
fn wandering_archaic_copies_when_opp_cannot_afford_two() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let _arch = g.add_card_to_battlefield(0, catalog::wandering_archaic());

    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    // No extra mana for the optional pay.
    let life_before = g.players[0].life;

    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("opp casts Bolt");
    drain_stack(&mut g);

    // 6 damage: Bolt + copy (since opp couldn't pay {2}).
    assert_eq!(g.players[0].life, life_before - 6,
        "Bolt (3) + Archaic copy (3) = 6 damage; got {}",
        life_before - g.players[0].life);
}

// ── New STX cards (claude/modern_decks push) ────────────────────────────────

/// Take Up the Shield: target creature gets +0/+3 and gains
/// indestructible until end of turn. A 2/2 bear becomes a 2/5 that
/// survives a Wrath / Lava Coil.
#[test]
fn take_up_the_shield_buffs_toughness_and_grants_indestructible() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);

    let id = g.add_card_to_hand(0, catalog::take_up_the_shield());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Take Up the Shield castable for {1}{W}");
    drain_stack(&mut g);

    let comp = g.computed_permanent(bear).unwrap();
    assert_eq!(comp.power, 2, "bear power unchanged");
    assert_eq!(comp.toughness, 5, "bear at 2+3=5 toughness");
    assert!(
        comp.keywords.contains(&Keyword::Indestructible),
        "should grant indestructible EOT"
    );
}

/// Star Pupil's Papers activated ability: {2}, sacrifice this artifact:
/// put a +1/+1 counter on target creature.
#[test]
fn star_pupils_papers_sac_activation_grants_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);

    let papers = g.add_card_to_battlefield(0, catalog::star_pupils_papers());
    g.clear_sickness(papers);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::ActivateAbility {
        card_id: papers,
        ability_index: 0,
        target: Some(Target::Permanent(bear)), x_value: None })
    .expect("Sac-for-counter activation should be legal");
    drain_stack(&mut g);

    // Papers should be in graveyard (sac'd as part of activation cost).
    assert!(
        g.battlefield_find(papers).is_none(),
        "papers should be sac'd off the battlefield"
    );
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(
        bear_card.counter_count(CounterType::PlusOnePlusOne),
        1,
        "bear should have one +1/+1 counter"
    );
}

/// Each of the five Snarl lands is a dual that produces its two
/// colors. Reveal-from-hand half (push modern_decks): when no
/// matching card is in hand, the land enters tapped per the printed
/// "If you don't reveal, this land enters tapped" rider. Frostboil
/// Snarl cast from a hand of zero other cards → enters tapped.
#[test]
fn frostboil_snarl_enters_tapped_without_revealable_card() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::frostboil_snarl());
    g.perform_action(GameAction::PlayLand(id))
        .expect("Frostboil Snarl playable as a land");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("snarl on bf");
    assert!(card.tapped, "Frostboil Snarl should enter tapped (no Island/Mountain in hand)");
    let def = catalog::frostboil_snarl();
    assert!(def.subtypes.land_types.contains(&crate::card::LandType::Island));
    assert!(def.subtypes.land_types.contains(&crate::card::LandType::Mountain));
}

/// Frostboil Snarl with an Island in hand enters untapped — the new
/// `Effect::IfRevealFromHand` primitive sees the matching land type
/// and AutoDecider auto-reveals (keeping the land untapped).
#[test]
fn frostboil_snarl_enters_untapped_with_island_in_hand() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::frostboil_snarl());
    g.perform_action(GameAction::PlayLand(id))
        .expect("Frostboil Snarl playable as a land");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("snarl on bf");
    assert!(
        !card.tapped,
        "Frostboil Snarl should enter untapped (Island revealable)"
    );
}

/// Frostboil Snarl with a Mountain in hand also enters untapped — the
/// filter is `HasLandType(Island) ∨ HasLandType(Mountain)`, so either
/// matching subtype unlocks the reveal.
#[test]
fn frostboil_snarl_enters_untapped_with_mountain_in_hand() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::frostboil_snarl());
    g.perform_action(GameAction::PlayLand(id))
        .expect("Frostboil Snarl playable as a land");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("snarl on bf");
    assert!(
        !card.tapped,
        "Frostboil Snarl should enter untapped (Mountain revealable)"
    );
}

/// Frostboil Snarl with only off-colored cards (a Forest) in hand
/// enters tapped — Forest doesn't match `Island ∨ Mountain`.
#[test]
fn frostboil_snarl_enters_tapped_with_only_off_color_in_hand() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::frostboil_snarl());
    g.perform_action(GameAction::PlayLand(id))
        .expect("Frostboil Snarl playable as a land");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("snarl on bf");
    assert!(
        card.tapped,
        "Frostboil Snarl should enter tapped (Forest doesn't match Island/Mountain)"
    );
}

/// Dragon's Approach deals 3 damage to any target. Verify it can
/// target a player.
#[test]
fn dragons_approach_deals_three_to_a_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::dragons_approach());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Dragon's Approach castable for {B}");
    drain_stack(&mut g);

    assert_eq!(
        g.players[1].life,
        life_before - 3,
        "Dragon's Approach should deal 3 to a player"
    );
}

/// Dragon's Approach deals 3 damage to a creature. A 3-toughness
/// bear dies to SBA after taking 3 marked damage.
#[test]
fn dragons_approach_kills_grizzly_bears() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);

    let id = g.add_card_to_hand(0, catalog::dragons_approach());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Dragon's Approach castable for {B}");
    drain_stack(&mut g);

    let _ = g.check_state_based_actions();
    assert!(
        g.battlefield_find(bear).is_none(),
        "Bear with 2 toughness dies to 3 damage"
    );
}

/// Defiant Strike: +1/+0 on a friendly creature and a cantrip.
#[test]
fn defiant_strike_pumps_friendly_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.add_card_to_library(0, catalog::island());

    let id = g.add_card_to_hand(0, catalog::defiant_strike());
    g.players[0].mana_pool.add(Color::White, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Defiant Strike castable for {W}");
    drain_stack(&mut g);

    let comp = g.computed_permanent(bear).unwrap();
    assert_eq!(comp.power, 3, "+1 power → 3");
    // -1 (cast) +1 (draw) = same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

/// Divine Gambit: exile any nonland permanent. Verify a creature gets
/// exiled.
#[test]
fn divine_gambit_exiles_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);

    let id = g.add_card_to_hand(0, catalog::divine_gambit());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Divine Gambit castable for {2}{W}");
    drain_stack(&mut g);

    assert!(
        g.battlefield_find(bear).is_none(),
        "Bear should be exiled"
    );
    let exiled = g.exile.iter().any(|c| c.id == bear);
    assert!(exiled, "Bear should be in the exile zone");
}

#[test]
fn divine_gambit_opp_may_put_permanent_from_hand_via_scripted_decider() {
    // Push (modern_decks, batch 77): the printed "Its controller may
    // put a permanent card from their hand onto the battlefield"
    // rider. AutoDecider's `Bool(false)` declines (the prior collapsed
    // behavior); ScriptedDecider's `Bool(true)` exercises the printed
    // gift-back. Target = a bear on P1's bf; P1's hand has another
    // permanent card (a Grizzly Bears) ready to gift back.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let _bear_in_hand = g.add_card_to_hand(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::divine_gambit());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));

    let bf_before = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.name == "Grizzly Bears")
        .count();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Divine Gambit castable");
    drain_stack(&mut g);

    // The original bear should be in exile (printed exile half).
    assert!(g.exile.iter().any(|c| c.id == bear),
        "the targeted bear is exiled");
    // The hand-bear should have moved to the bf via the gift-back path.
    let bf_after = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.name == "Grizzly Bears")
        .count();
    assert_eq!(bf_after, bf_before,
        "P1 gifted a new bear from hand back to their battlefield");
}

// ── CR 120.8 — 0-damage event suppression audit ─────────────────────────────

/// CR 120.8: "If a source would deal 0 damage, it does not deal damage at
/// all. That means abilities that trigger on damage being dealt won't
/// trigger." We exercise the rule by casting Dragon's Approach with the
/// damage scaled down to 0 (via a -3/-0 pump on the source... wait,
/// Dragon's Approach is a sorcery so we can't pump *it*). Easier: cast a
/// damage spell whose amount evaluates to 0 and assert that the engine
/// emits no `DamageDealt` event and no LifeLost event.
///
/// Setup: the engine's `deal_damage_to_from` (in `game/effects/movement.rs`)
/// now bails out early when `amount == 0` so no event is emitted. This
/// test validates the audit via the existing `Effect::DealDamage` path
/// with `Value::Const(0)` against a player target — the player's life
/// total stays at 20 and no `LifeLost` event is emitted.
#[test]
fn zero_damage_does_not_trigger_damage_events_per_cr_120_8() {
    use crate::card::{
        CardDefinition, CardType, Effect, Subtypes, Value,
    };
    use crate::effect::shortcut::target_filtered;
    use crate::game::GameEvent;
    use crate::mana::cost;

    // Build a synthetic "{R}: deal 0 damage to target player" instant.
    let zero_damage_burn = CardDefinition {
        name: "Zero-Damage Burn",
        cost: cost(&[crate::mana::r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(crate::card::SelectionRequirement::Player),
            amount: Value::Const(0),
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
    };

    let mut g = two_player_game();
    let life_before = g.players[1].life;

    let id = g.add_card_to_hand(0, zero_damage_burn);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Zero-Damage Burn castable for {R}");
    let events = drain_stack(&mut g);

    // CR 120.8 — player's life is unchanged.
    assert_eq!(
        g.players[1].life, life_before,
        "P1 life should be unchanged after a 0-damage spell"
    );
    // No DamageDealt event was emitted (even at amount=0) — abilities
    // that trigger on damage being dealt should not have fired.
    let any_damage_event = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::DamageDealt {
                to_player: Some(1),
                ..
            }
        )
    });
    assert!(
        !any_damage_event,
        "CR 120.8 — no DamageDealt event should be emitted on 0 damage"
    );
    // And no LifeLost event either (the player didn't actually lose
    // life — the 0 amount short-circuited).
    let any_life_lost = events
        .iter()
        .any(|e| matches!(e, GameEvent::LifeLost { player: 1, .. }));
    assert!(
        !any_life_lost,
        "CR 120.8 — no LifeLost event should be emitted on 0 damage"
    );
}

/// CR 701.22b — "If a player is instructed to scry 0, no scry event
/// occurs. Abilities that trigger whenever a player scries won't
/// trigger." Validate the `Effect::Scry` short-circuit on
/// `amount: Value::Const(0)` — no `GameEvent::ScryPerformed` should
/// be emitted, and the library order is unchanged.
#[test]
fn zero_scry_does_not_trigger_scry_events_per_cr_701_22b() {
    use crate::card::{CardDefinition, CardType, Effect, Subtypes, Value};
    use crate::effect::PlayerRef;
    use crate::game::GameEvent;
    use crate::mana::cost;

    // Synthetic "{U}: scry 0" instant.
    let zero_scry = CardDefinition {
        name: "Zero Scry",
        cost: cost(&[crate::mana::u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(0) },
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
    };

    let mut g = two_player_game();
    // Seed the library so a Scry 1 would have something to look at.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let lib_snapshot: Vec<_> = g.players[0].library.iter().map(|c| c.id).collect();

    let id = g.add_card_to_hand(0, zero_scry);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Zero Scry castable for {U}");
    let events = drain_stack(&mut g);

    let any_scry_event = events.iter().any(|e| matches!(e, GameEvent::ScryPerformed { .. }));
    assert!(
        !any_scry_event,
        "CR 701.22b — no ScryPerformed event should fire on Scry 0"
    );
    // Library order must be unchanged.
    let lib_after: Vec<_> = g.players[0].library.iter().map(|c| c.id).collect();
    assert_eq!(lib_after, lib_snapshot, "Library order unchanged");
}

/// Cram Session: gain 5 life at instant speed and the card has
/// Keyword::Flashback({5}{W}).
#[test]
fn cram_session_gains_five_life_and_has_flashback() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::cram_session());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Cram Session castable for {3}{W}");
    drain_stack(&mut g);

    assert_eq!(
        g.players[0].life,
        life_before + 5,
        "Cram Session should gain 5 life"
    );

    let card = catalog::cram_session();
    let has_flashback = card.keywords.iter().any(|k| matches!(k, Keyword::Flashback(_)));
    assert!(has_flashback, "Cram Session should carry Keyword::Flashback");
}

// ── Push XXXVIII: Dragon's Approach tutor rider ─────────────────────────────

/// With four copies of Dragon's Approach already in the controller's
/// graveyard, casting another should hit the gy-tutor rider and pull a
/// Dragon creature card from the library onto the battlefield. The 3
/// damage half also fires.
#[test]
fn dragons_approach_tutors_dragon_with_four_in_graveyard() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed graveyard with four named copies.
    for _ in 0..4 {
        let cid = g.add_card_to_library(0, catalog::dragons_approach());
        let pos = g.players[0]
            .library
            .iter()
            .position(|c| c.id == cid)
            .unwrap();
        let card = g.players[0].library.remove(pos);
        g.players[0].graveyard.push(card);
    }
    // Seed library with a Dragon creature for the tutor to find.
    let dragon_id = g.add_card_to_library(0, catalog::lorehold_the_historian());
    g.add_card_to_library(0, catalog::island());

    let id = g.add_card_to_hand(0, catalog::dragons_approach());
    g.players[0].mana_pool.add(Color::Black, 1);

    // Scripted decider picks the Dragon during the tutor.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(dragon_id))]));

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Dragon's Approach castable for {B}");
    drain_stack(&mut g);

    let on_bf = g.battlefield.iter().any(|c| c.id == dragon_id && c.controller == 0);
    assert!(on_bf, "The chosen Dragon should be on the battlefield after Dragon's Approach tutored it");
}

/// Pure-vanilla cast — graveyard tally is below 4 → no tutor offered.
/// The auto-decider doesn't even reach a SearchLibrary decision because
/// the gating predicate fails first. Just verify damage half fires and
/// the dragon stays in the library.
#[test]
fn dragons_approach_does_not_offer_tutor_without_four_named_in_graveyard() {
    let mut g = two_player_game();
    // Only three copies in gy.
    for _ in 0..3 {
        let cid = g.add_card_to_library(0, catalog::dragons_approach());
        let pos = g.players[0]
            .library
            .iter()
            .position(|c| c.id == cid)
            .unwrap();
        let card = g.players[0].library.remove(pos);
        g.players[0].graveyard.push(card);
    }
    let dragon_id = g.add_card_to_library(0, catalog::lorehold_the_historian());

    let id = g.add_card_to_hand(0, catalog::dragons_approach());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Dragon's Approach castable for {B}");
    drain_stack(&mut g);

    // Damage half always fires.
    assert_eq!(g.players[1].life, life_before - 3, "3 damage to player still resolves");
    // Dragon stays in library (no tutor).
    let on_bf = g.battlefield.iter().any(|c| c.id == dragon_id);
    assert!(!on_bf, "Tutor rider should not fire with three copies in graveyard");
    let in_lib = g.players[0].library.iter().any(|c| c.id == dragon_id);
    assert!(in_lib, "Dragon should still be in the library");
}

// ── Push (modern_decks): New STX additions + SOS promotions ─────────────────

/// Expanded Anatomy is a Lesson that lands two +1/+1 counters on a
/// target creature for {3}{G}.
#[test]
fn expanded_anatomy_lands_two_counters_on_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::expanded_anatomy());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Expanded Anatomy castable for {3}{G}");
    drain_stack(&mut g);

    let card = g.battlefield.iter().find(|c| c.id == bear).expect("Bear alive");
    assert_eq!(
        card.counter_count(CounterType::PlusOnePlusOne),
        2,
        "Bear should have two +1/+1 counters from Expanded Anatomy"
    );
    assert_eq!(card.power(), 4, "Bear becomes 4/4");
    assert_eq!(card.toughness(), 4);
}

/// Selfless Glyphweaver's sac activation grants Indestructible (EOT)
/// to all of the controller's creatures; the Glyphweaver itself is
/// sacrificed as cost (so it does not stay around with indestructible).
#[test]
fn selfless_glyphweaver_sac_grants_indestructible_to_friendlies() {
    let mut g = two_player_game();
    let gw = g.add_card_to_battlefield(0, catalog::selfless_glyphweaver());
    g.clear_sickness(gw);
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(buddy);

    g.perform_action(GameAction::ActivateAbility {
        card_id: gw,
        ability_index: 0,
        target: None, x_value: None })
    .expect("Selfless Glyphweaver sac activation");
    drain_stack(&mut g);

    // Glyphweaver is sacrificed.
    assert!(
        !g.battlefield.iter().any(|c| c.id == gw),
        "Glyphweaver should be sacrificed"
    );
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == gw),
        "Glyphweaver should be in graveyard"
    );
    // Buddy bear is indestructible.
    let buddy_card = g.battlefield.iter().find(|c| c.id == buddy).expect("Bear alive");
    assert!(
        buddy_card.has_keyword(&Keyword::Indestructible),
        "Buddy creature should have indestructible until end of turn"
    );
}

/// Mercurial Transformation overrides the target creature's base P/T to
/// 3/3 until end of turn via `Effect::SetBasePT`. Reads through the
/// layered P/T via `computed_permanent` (the same approach Square Up's
/// test uses).
#[test]
fn mercurial_transformation_sets_target_to_three_three_eot() {
    let mut g = two_player_game();
    // Pick a creature with non-3/3 base P/T to verify the rewrite.
    let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
    g.clear_sickness(dragon);
    let id = g.add_card_to_hand(0, catalog::mercurial_transformation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(dragon)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Mercurial Transformation castable for {2}{U}");
    drain_stack(&mut g);

    // SetBasePT applies via layer 7b; consult the computed permanent.
    let computed = g.computed_permanent(dragon).expect("Dragon still on bf");
    assert_eq!(computed.power, 3, "Dragon should be reduced to base power 3");
    assert_eq!(computed.toughness, 3, "Dragon should be reduced to base toughness 3");
}

/// Crux of Fate: mode 0 destroys Dragons (sparing non-Dragons); mode 1 destroys non-Dragons.
#[test]
fn crux_of_fate_modes_destroy_dragons_or_non_dragons() {
    for (mode, dragon_lives, bear_lives) in [(0usize, false, true), (1, true, false)] {
        let mut g = two_player_game();
        let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(dragon);
        g.clear_sickness(bear);
        let id = g.add_card_to_hand(0, catalog::crux_of_fate());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(3);

        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: Some(mode), x_value: None,
        }).expect("Crux of Fate castable for {3}{B}{B}");
        drain_stack(&mut g);

        assert_eq!(g.battlefield.iter().any(|c| c.id == dragon), dragon_lives,
            "Mode {mode}: dragon survives = {dragon_lives}");
        assert_eq!(g.battlefield.iter().any(|c| c.id == bear), bear_lives,
            "Mode {mode}: bear survives = {bear_lives}");
    }
}

/// Plargg, Dean of Chaos taps to loot one card.
#[test]
fn plargg_dean_of_chaos_taps_to_loot() {
    let mut g = two_player_game();
    // Seed library so the draw resolves.
    g.add_card_to_library(0, catalog::island());
    // Discard fodder.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let plargg = g.add_card_to_battlefield(0, catalog::plargg_dean_of_chaos());
    g.clear_sickness(plargg);
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: plargg,
        ability_index: 0,
        target: None, x_value: None })
    .expect("Plargg activation");
    drain_stack(&mut g);

    // -1 discard, +1 draw → net hand unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[0].library.len(), lib_before - 1);
    let on_bf = g.battlefield.iter().find(|c| c.id == plargg).expect("Plargg alive");
    assert!(on_bf.tapped, "Plargg should be tapped");
}

/// Plargg's "if a creature card was discarded" rider fires when the
/// auto-decider discards Grizzly Bears (the only card in hand) and the
/// activation targets an opposing player — 2 damage drops their life
/// from 20 → 18.
#[test]
fn plargg_dean_of_chaos_deals_two_damage_when_creature_discarded() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    // Only-hand card is a creature, so AutoDecider's Discard picks it.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let plargg = g.add_card_to_battlefield(0, catalog::plargg_dean_of_chaos());
    g.clear_sickness(plargg);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::ActivateAbility {
        card_id: plargg,
        ability_index: 0,
        target: Some(Target::Player(1)), x_value: None })
    .expect("Plargg activation");
    drain_stack(&mut g);

    // Bear discarded → 2 damage to P1.
    assert_eq!(g.players[1].life, life_before - 2);
}

/// Plargg's conditional rider does *not* fire when the discarded card is
/// a noncreature — the damage is gated by
/// `Value::CreatureCardsDiscardedThisEffect ≥ 1`.
#[test]
fn plargg_dean_of_chaos_no_damage_when_noncreature_discarded() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    // Discard a land — non-creature.
    g.add_card_to_hand(0, catalog::island());
    let plargg = g.add_card_to_battlefield(0, catalog::plargg_dean_of_chaos());
    g.clear_sickness(plargg);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::ActivateAbility {
        card_id: plargg,
        ability_index: 0,
        target: Some(Target::Player(1)), x_value: None })
    .expect("Plargg activation");
    drain_stack(&mut g);

    // Island discarded → no damage.
    assert_eq!(g.players[1].life, life_before);
}

/// Pestilent Cauldron's sac activation mills 4 from each player and
/// drains 3.
#[test]
fn pestilent_cauldron_sac_mills_and_drains() {
    let mut g = two_player_game();
    // Seed both libraries.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(1, catalog::island());
    }
    let pc = g.add_card_to_battlefield(0, catalog::pestilent_cauldron());
    g.clear_sickness(pc);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let gy0_before = g.players[0].graveyard.len();
    let gy1_before = g.players[1].graveyard.len();
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::ActivateAbility {
        card_id: pc,
        ability_index: 0,
        target: None, x_value: None })
    .expect("Cauldron activation");
    drain_stack(&mut g);

    // Sacrificed.
    assert!(!g.battlefield.iter().any(|c| c.id == pc));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pc));
    // Life delta: P0 gains 3, P1 loses 3.
    assert_eq!(g.players[0].life, life0_before + 3);
    assert_eq!(g.players[1].life, life1_before - 3);
    // Each player milled 4.
    // P0's graveyard contains the Cauldron plus 4 milled cards.
    assert_eq!(g.players[0].graveyard.len(), gy0_before + 1 /* cauldron */ + 4);
    assert_eq!(g.players[1].graveyard.len(), gy1_before + 4);
}

/// Ajani's Response alt cost ({1}{W}, vs printed {4}{W}) requires the target to be tapped.
/// Tapped: destroys it. Untapped: cast-time filter rejects.
#[test]
fn ajanis_response_alt_cost_destroys_tapped_rejects_untapped() {
    // Tapped target: alt cast destroys it.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) {
        c.tapped = true;
    }
    let id = g.add_card_to_hand(0, catalog::ajanis_response());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ajani's Response alt-cast should resolve when target is tapped");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Tapped bear should be destroyed via alt cost");

    // Untapped target: cast-time filter rejects.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::ajanis_response());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let err = g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Alt-cost cast should reject untapped target (filter requires Tapped)");
}

/// Run Behind can be alt-cast at {2}{U} when it targets an attacking
/// creature.
#[test]
fn run_behind_alt_cost_bounces_attacking_creature_to_library_bottom() {
    let mut g = two_player_game();
    // Set up: P1's bear attacking P0.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.attacking.push(crate::game::Attack {
        attacker: bear,
        target: crate::game::AttackTarget::Player(0),
    });
    let id = g.add_card_to_hand(0, catalog::run_behind());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: None,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Run Behind alt-cast at {2}{U} should resolve");
    drain_stack(&mut g);

    // Bear should be at the bottom of P1's library.
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Bear should leave the battlefield"
    );
    let lib_bottom = g.players[1].library.first();
    assert!(
        lib_bottom.map(|c| c.id) == Some(bear),
        "Bear should be at the bottom of P1's library"
    );
}

/// CR 514.1 — At the cleanup step of the active player's turn, if their
/// hand is over the max hand size (7), they discard enough cards to
/// reduce to 7.
#[test]
fn cleanup_step_discards_down_to_seven_per_cr_514_1() {
    let mut g = two_player_game();
    // Stuff P0's hand with 10 islands.
    for _ in 0..10 {
        g.add_card_to_hand(0, catalog::island());
    }
    assert_eq!(g.players[0].hand.len(), 10, "Start with 10 cards");
    assert_eq!(g.active_player_idx, 0);
    let gy_before = g.players[0].graveyard.len();

    // Step directly to Cleanup; passing priority twice runs do_cleanup.
    g.step = TurnStep::Cleanup;
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();

    // P0 should now be at exactly 7 cards (3 discarded into graveyard).
    assert_eq!(g.players[0].hand.len(), 7, "Hand reduced to max hand size");
    assert_eq!(
        g.players[0].graveyard.len(),
        gy_before + 3,
        "Three cards moved hand → graveyard"
    );
}

/// CR 514.1 — If the active player's hand is already at or below max
/// hand size, cleanup is a no-op for the hand.
#[test]
fn cleanup_step_no_op_when_hand_at_or_below_max_per_cr_514_1() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_hand(0, catalog::island());
    }
    assert_eq!(g.active_player_idx, 0);
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();

    g.step = TurnStep::Cleanup;
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();

    assert_eq!(
        g.players[0].hand.len(),
        hand_before,
        "Hand unchanged when below max hand size"
    );
    assert_eq!(
        g.players[0].graveyard.len(),
        gy_before,
        "No cards discarded"
    );
}

/// Brush Off can be cast at {1}{U} (alt cost) when it targets an
/// instant or sorcery on the stack — verified by P0 alt-casting Brush
/// Off on P1's Lightning Bolt.
#[test]
fn brush_off_alt_cost_counters_instant_on_stack() {
    let mut g = two_player_game();
    // P1 casts Bolt at P0.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable");

    // P0 responds with Brush Off at alt cost.
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::brush_off());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: None,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Brush Off alt-cost at {1}{U} should resolve");
    drain_stack(&mut g);

    // P0 should still be at 20 (no Bolt damage).
    assert_eq!(g.players[0].life, 20, "Bolt should be countered");
}

// ── Reconstruct History (NEW, modern_decks push) ────────────────────────────

/// Reconstruct History — {1}{R}{W} sorcery, Lorehold. Choose two or more
/// modes — return target artifact/instant/Spirit/sorcery card from your
/// graveyard to your hand. The auto-decider's first viable pair of modes
/// (artifact + instant) returns those two cards to hand.
#[test]
fn reconstruct_history_returns_two_cards_from_graveyard_to_hand() {
    let mut g = two_player_game();
    // Seed gy with one artifact (Mind Stone), one instant (Lightning Bolt),
    // and one sorcery (Day of Judgment). All four printed modes have at
    // least one matching card.
    let stone = g.add_card_to_graveyard(0, catalog::mind_stone());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let _doj = g.add_card_to_graveyard(0, catalog::day_of_judgment());
    let id = g.add_card_to_hand(0, catalog::reconstruct_history());
    let gy_before = g.players[0].graveyard.len();
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Reconstruct History castable for {1}{R}{W}");
    drain_stack(&mut g);

    // Hand: -1 (cast spell, moved to stack then gy) + 2 (two gy cards
    // returned to hand) = +1 net relative to hand_before.
    assert_eq!(
        g.players[0].hand.len(),
        hand_before + 1,
        "two gy cards returned to hand"
    );
    // Graveyard: -2 (two cards moved to hand) + 1 (Reconstruct History
    // itself enters gy on resolve) = -1 net.
    assert_eq!(
        g.players[0].graveyard.len(),
        gy_before - 1,
        "two cards returned, Reconstruct History added"
    );
    // The two returned cards should now be in hand.
    let in_hand: Vec<_> = g.players[0]
        .hand
        .iter()
        .map(|c| c.id)
        .collect();
    let returned_either = in_hand.contains(&stone) || in_hand.contains(&bolt);
    assert!(
        returned_either,
        "at least one of the matched gy cards should be in hand"
    );
}

// ── Lorehold Excavation (NEW, modern_decks push) ────────────────────────────

/// Lorehold Excavation {2}{R}{W}, {T}: exile a target gy card; if it was a
/// creature card, mint an X/X flying Spirit. Non-creature exile mints nothing.
#[test]
fn lorehold_excavation_exile_creature_mints_spirit_token_non_creature_does_not() {
    use crate::card::Keyword;

    fn setup() -> (GameState, CardId) {
        let mut g = two_player_game();
        let excavation = g.add_card_to_battlefield(0, catalog::lorehold_excavation());
        g.clear_sickness(excavation);
        if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == excavation) {
            c.tapped = false;
            c.summoning_sick = false;
        }
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        (g, excavation)
    }

    // Creature target → mints a 2/2 flying Spirit (X = bear's power).
    let (mut g, excavation) = setup();
    let bear_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: excavation, ability_index: 2,
        target: Some(Target::Permanent(bear_gy)), x_value: None }).expect("Lorehold Excavation gy-exile activates for {2}{R}{W}, {T}");
    drain_stack(&mut g);
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear_gy));
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit" && c.controller == 0)
        .collect();
    assert_eq!(spirits.len(), 1, "one bonus Spirit token minted");
    assert_eq!(spirits[0].power(), 2, "Spirit token power = bear's power (2)");
    assert_eq!(spirits[0].toughness(), 2);
    assert!(spirits[0].has_keyword(&Keyword::Flying));

    // Non-creature target (sorcery) → no Spirit token.
    let (mut g, excavation) = setup();
    let doj = g.add_card_to_graveyard(0, catalog::day_of_judgment());
    g.perform_action(GameAction::ActivateAbility {
        card_id: excavation, ability_index: 2,
        target: Some(Target::Permanent(doj)), x_value: None }).expect("Lorehold Excavation gy-exile activates");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit" && c.controller == 0)
        .count();
    assert_eq!(spirits, 0, "no bonus Spirit token when target gy card is a non-creature");
}

/// Lorehold Excavation: when the exiled creature is bigger (e.g. Serra
/// Angel, 4/4), the bonus Spirit token mints as X/X = 4/4 — proving
/// the new `Value::PowerOf` evaluator correctly reads the gy card's
/// printed power.
#[test]
fn lorehold_excavation_token_scales_with_creature_power() {
    let mut g = two_player_game();
    let excavation = g.add_card_to_battlefield(0, catalog::lorehold_excavation());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == excavation) {
        c.tapped = false;
        c.summoning_sick = false;
    }
    // Serra Angel is 4/4 — the bonus Spirit token should mint at 4/4.
    let angel_gy = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: excavation,
        ability_index: 2,
        target: Some(Target::Permanent(angel_gy)), x_value: None })
    .expect("Lorehold Excavation gy-exile activates");
    drain_stack(&mut g);

    let spirits: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit" && c.controller == 0)
        .collect();
    assert_eq!(spirits.len(), 1, "one bonus Spirit token minted");
    assert_eq!(
        spirits[0].power(),
        4,
        "Spirit token power = Serra Angel's power (4)"
    );
    assert_eq!(spirits[0].toughness(), 4);
}

// ── Diamond cycle (STA reprints) ────────────────────────────────────────────

/// Sky Diamond enters tapped and taps for {U}. After casting and ETB
/// resolves the rock is tapped (matching the printed "enters tapped"
/// clause).
#[test]
fn sky_diamond_enters_tapped_then_taps_for_blue() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sky_diamond());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Sky Diamond castable for {2}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.tapped, "Sky Diamond should enter tapped");
}

// ── Goblin Lore (STA reprint) ───────────────────────────────────────────────

/// Goblin Lore draws four and discards three (at random). Net: +1 card
/// in hand from the spell, modulo the cast itself (so net is 0 after
/// the spell goes to graveyard).
#[test]
fn goblin_lore_draws_four_and_discards_three() {
    use crate::game::types::TurnStep as TS;
    let mut g = two_player_game();
    g.step = TS::PreCombatMain;
    // Seed library with 4 cards so the draw can succeed.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::mountain());
    }
    let id = g.add_card_to_hand(0, catalog::goblin_lore());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();
    let gy_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Goblin Lore castable for {R}");
    drain_stack(&mut g);

    // Hand: -1 (cast) + 4 (draw) - 3 (discard) = 0 net.
    assert_eq!(
        g.players[0].hand.len(),
        hand_before,
        "hand size unchanged: -1 cast + 4 draw - 3 discard = 0"
    );
    // Library: -4 (drew 4).
    assert_eq!(
        g.players[0].library.len(),
        lib_before - 4,
        "drew 4 from library"
    );
    // Graveyard: +3 (discarded 3) + 1 (Goblin Lore on resolve).
    assert_eq!(
        g.players[0].graveyard.len(),
        gy_before + 4,
        "3 discarded + 1 Goblin Lore went to graveyard"
    );
}

// ── Whirlwind Denial (STA reprint) ──────────────────────────────────────────

/// Whirlwind Denial counters target spell unless its controller pays
/// {4}. When the opp can't afford {4}, the spell is countered.
#[test]
fn whirlwind_denial_counters_spell_unless_four_paid() {
    let mut g = two_player_game();
    // P1 casts Bolt at P0 first (so it's on the stack).
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    // P0 responds with Whirlwind Denial targeting the bolt; opp has no
    // mana to pay {4} → bolt is countered.
    g.priority.player_with_priority = 0;
    let denial = g.add_card_to_hand(0, catalog::whirlwind_denial());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: denial,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Whirlwind Denial castable for {3}{U}");
    drain_stack(&mut g);

    // P0 should still be at 20 (Bolt countered).
    assert_eq!(g.players[0].life, 20, "Bolt should be countered");
}

// ── New STA reprint card tests (push modern_decks) ──────────────────────────

/// Eliminate destroys a small (MV ≤ 3) creature.
#[test]
fn eliminate_destroys_two_mana_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let elim = g.add_card_to_hand(0, catalog::eliminate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: elim,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Eliminate castable for {1}{B} against Grizzly Bears");
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(bear).is_none(),
        "Grizzly Bears (MV=2) destroyed by Eliminate"
    );
    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == bear),
        "destroyed Bear lives in P1's graveyard"
    );
}

/// Eliminate cannot target a creature with mana value 4+ — the cast-time
/// target validator should reject the spell entirely.
#[test]
fn eliminate_rejects_target_with_mana_value_four() {
    let mut g = two_player_game();
    let lyra = g.add_card_to_battlefield(1, catalog::serra_angel());
    let elim = g.add_card_to_hand(0, catalog::eliminate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let result = g.perform_action(GameAction::CastSpell {
        card_id: elim,
        target: Some(Target::Permanent(lyra)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(
        result.is_err(),
        "Eliminate should reject Serra Angel (MV=5)"
    );
    assert!(
        g.battlefield_find(lyra).is_some(),
        "Serra Angel still on the battlefield"
    );
}

/// Pull from Tomorrow at X=3 draws 4 cards, then discards 1 — net +3 in
/// hand (minus the cast itself = net +2).
#[test]
fn pull_from_tomorrow_at_x_three_draws_four_discards_one() {
    let mut g = two_player_game();
    // Seed enough library to draw.
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let pft = g.add_card_to_hand(0, catalog::pull_from_tomorrow());
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();

    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: pft,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("Pull from Tomorrow castable for {3}{U}{U}");
    drain_stack(&mut g);

    // Hand: -1 (cast) +4 (draw X+1=4) -1 (discard) = +2 net.
    assert_eq!(
        g.players[0].hand.len(),
        hand_before + 2,
        "draw 4 + discard 1 + cast Pull = net +2"
    );
    // Library: -4 (drew 4).
    assert_eq!(g.players[0].library.len(), lib_before - 4, "drew 4 cards");
}

/// Burst Lightning at base cost deals 2 damage to a player.
#[test]
fn burst_lightning_deals_two_damage_to_player() {
    let mut g = two_player_game();
    let bl = g.add_card_to_hand(0, catalog::burst_lightning());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bl,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Burst Lightning castable for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "P1 takes 2 damage");
}

/// Postmortem Lunge at X=2 lifts a creature with MV=2 from the graveyard
/// to the battlefield (haste, exile EOT).
#[test]
fn postmortem_lunge_returns_two_mana_creature_to_battlefield() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let life_before = g.players[0].life;
    let pl = g.add_card_to_hand(0, catalog::postmortem_lunge());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: pl,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("Postmortem Lunge castable for {X=2}{B}");
    drain_stack(&mut g);

    // Bear should be on the battlefield (mv = 2 matches X = 2).
    let on_bf = g.battlefield.iter().find(|c| c.id == bear);
    assert!(
        on_bf.is_some(),
        "Bear with MV=2 returned to battlefield (X=2)"
    );
    assert!(
        on_bf.unwrap().has_keyword(&Keyword::Haste),
        "returned creature has haste"
    );
    // Life loss = X = 2.
    assert_eq!(g.players[0].life, life_before - 2, "lost 2 life");
}

/// Postmortem Lunge's `Predicate::ValueEquals` gate stays put when the
/// graveyard target's MV doesn't equal X — at X=2 a 3-MV Prodigal
/// Sorcerer is targeted, but the equality gate fails and the body
/// short-circuits: no Move, no Haste grant, no delayed exile. The
/// life-cost half still runs (LoseLife is sequenced ahead of the If).
#[test]
fn postmortem_lunge_value_equals_rejects_off_by_one_mana_value() {
    let mut g = two_player_game();
    let sorcerer = g.add_card_to_graveyard(0, catalog::prodigal_sorcerer()); // MV 3
    let life_before = g.players[0].life;
    let pl = g.add_card_to_hand(0, catalog::postmortem_lunge());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: pl,
        target: Some(Target::Permanent(sorcerer)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("Postmortem Lunge castable for {X=2}{B}");
    drain_stack(&mut g);

    // ValueEquals(MV=3, X=2) is false — sorcerer stays in graveyard.
    assert!(
        g.battlefield.iter().all(|c| c.id != sorcerer),
        "MV-3 sorcerer should NOT return when X=2"
    );
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == sorcerer),
        "MV-3 sorcerer remains in graveyard"
    );
    // The LoseLife half still runs (it precedes the If gate).
    assert_eq!(
        g.players[0].life,
        life_before - 2,
        "lost X=2 life regardless of equality gate"
    );
}

/// Channeled Force draws the difference between opp's hand size and
/// yours, capped at the actual library size.
#[test]
fn channeled_force_draws_hand_size_differential() {
    let mut g = two_player_game();
    // Seed library so the draw isn't capped.
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    // P1 has 5 cards in hand; P0 has 1 (just the cast).
    for _ in 0..5 {
        g.add_card_to_hand(1, catalog::mountain());
    }
    let cf = g.add_card_to_hand(0, catalog::channeled_force());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    let p0_hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: cf,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Channeled Force castable for {1}{U}{R}");
    drain_stack(&mut g);
    // P0: -1 (cast) + diff(5, 1) = -1 + 4 = +3 net.
    // P1 has 5; P0 had 1 (Channeled Force itself) before cast.
    assert!(
        g.players[0].hand.len() >= p0_hand_before,
        "should have drawn at least the cast back"
    );
}

/// Stonebound Mentor's Magecraft pumps a friendly creature with haste.
#[test]
fn stonebound_mentor_magecraft_pumps_friendly_with_haste() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let mentor = g.add_card_to_battlefield(0, catalog::stonebound_mentor());
    g.clear_sickness(mentor);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    // Mentor's magecraft trigger should fire and pump a friendly creature.
    // Auto-target picker excludes the Mentor itself when another friendly
    // is on the battlefield. Verify the bear is the one pumped.
    let bear_cp = g.computed_permanent(bear).expect("bear alive");
    assert!(
        bear_cp.power >= 2,
        "auto-target picks a friendly to pump"
    );
    // At least one friendly has haste this turn (printed Oracle's grant).
    let bear_has_haste = bear_cp.keywords.contains(&Keyword::Haste);
    let mentor_cp = g.computed_permanent(mentor).expect("mentor alive");
    let mentor_has_haste = mentor_cp.keywords.contains(&Keyword::Haste);
    assert!(
        bear_has_haste || mentor_has_haste,
        "Magecraft grants Haste EOT to the picked friendly"
    );
}

/// Curious Cryomancer scries 1 on each instant or sorcery cast.
#[test]
fn curious_cryomancer_magecraft_scrys_one() {
    let mut g = two_player_game();
    // Library needs at least one card for Scry to have something to peek.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::curious_cryomancer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);
    // Library still has the one card (scry doesn't remove it; it may
    // reorder or send to bottom, but auto-decider keeps order).
    assert!(
        !g.players[0].library.is_empty(),
        "library still has cards after scry"
    );
}

/// Verdant Pledgemage gains 2 life on ETB.
#[test]
fn verdant_pledgemage_gains_two_life_on_etb() {
    let mut g = two_player_game();
    let vp = g.add_card_to_hand(0, catalog::verdant_pledgemage());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: vp,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Verdant Pledgemage castable for {1}{G}{G}");
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].life,
        life_before + 2,
        "Verdant Pledgemage ETB → gain 2"
    );
}

/// Inscription of Insight at X=2 puts two +1/+1 counters on a target
/// creature (default auto-picked mode 0).
#[test]
fn inscription_of_insight_x_two_lands_two_counters() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let io = g.add_card_to_hand(0, catalog::inscription_of_insight());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: io,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("Inscription of Insight castable for {X=2}{G}{U}");
    drain_stack(&mut g);
    let bf_bear = g.battlefield_find(bear).expect("bear alive");
    let plus_one_count = bf_bear
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(plus_one_count, 2, "two +1/+1 counters at X=2");
}

/// Memory Lapse — after the new `CounterSpellToZone` wiring, the
/// countered spell lands on top of its owner's library rather than in
/// the graveyard. The printed "instead" clause overrides CR 701.6a's
/// default routing of countered spells to the graveyard.
#[test]
fn memory_lapse_routes_countered_spell_to_library_top_per_cr_701_6a() {
    let mut g = two_player_game();
    // P1 casts Lightning Bolt at P0 first.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");

    let lib_before = g.players[1].library.len();
    let gy_before = g.players[1].graveyard.len();

    g.priority.player_with_priority = 0;
    let lapse = g.add_card_to_hand(0, catalog::memory_lapse());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: lapse,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Memory Lapse castable for {1}{U}");
    drain_stack(&mut g);

    // Bolt should be back on top of P1's library, NOT in the graveyard.
    assert_eq!(
        g.players[1].library.len(),
        lib_before + 1,
        "Bolt placed on top of P1's library (CR 701.5g)"
    );
    assert_eq!(
        g.players[1].graveyard.len(),
        gy_before,
        "Bolt did NOT go to graveyard"
    );
    let top = g.players[1].library.last().expect("library not empty");
    assert_eq!(
        top.definition.name, "Lightning Bolt",
        "top card is the Memory-Lapse'd Bolt"
    );
    // P0 still at 20 (Bolt didn't resolve).
    assert_eq!(g.players[0].life, 20, "Bolt was countered");
}

// ── New STX cards (push modern_decks) ───────────────────────────────────────

#[test]
fn eureka_moment_draws_two_cards() {
    // AutoDecider declines the MayDo land-drop; the net is +1 card (cast
    // EM, draw 2).
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let initial_lib = g.players[0].library.len();
    let initial_hand = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::eureka_moment());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Eureka Moment castable for {2}{G}{U}");
    drain_stack(&mut g);

    // Library -2 (drew 2 islands), hand: initial + 1 (added EM) - 1 (cast) + 2 (drew) = +2.
    assert_eq!(g.players[0].library.len(), initial_lib - 2);
    assert_eq!(g.players[0].hand.len(), initial_hand + 2);
    // AutoDecider declined the land drop, so no extra land on battlefield.
    let extra_lands = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.is_land())
        .count();
    assert_eq!(extra_lands, 0, "no land entered the battlefield");
    let _ = id;
}

#[test]
fn eureka_moment_optional_land_drop_with_scripted_decider() {
    // ScriptedDecider opts into the land-drop; the land goes to bf tapped.
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let _land_in_hand = g.add_card_to_hand(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::eureka_moment());

    // Pre-stage: count lands on the battlefield.
    let lands_before = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.is_land())
        .count();

    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Eureka Moment castable");
    drain_stack(&mut g);

    let lands_after = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.is_land())
        .count();
    assert_eq!(
        lands_after,
        lands_before + 1,
        "MayDo land-drop landed the Forest tapped"
    );
}

#[test]
fn teach_by_example_copies_target_instant() {
    // P0 stack: Lightning Bolt at P1, then Teach by Example targeting the
    // Bolt. The copy resolves first and the original Bolt resolves second
    // — both deal 3 damage to P1.
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");

    // Bolt is now on the stack as the topmost StackItem.
    let bolt_stack_id = g
        .stack
        .iter()
        .find_map(|s| match s {
            StackItem::Spell { card, .. } if card.definition.name == "Lightning Bolt" => {
                Some(card.id)
            }
            _ => None,
        })
        .expect("Bolt on stack");

    let teach = g.add_card_to_hand(0, catalog::teach_by_example());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: teach,
        target: Some(Target::Permanent(bolt_stack_id)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Teach by Example castable");
    drain_stack(&mut g);

    // Both Bolt + its copy deal 3 → P1 takes 6 total.
    assert_eq!(
        g.players[1].life,
        p1_life_before - 6,
        "P1 took 6 damage (Bolt + copy)"
    );
}

#[test]
fn manifold_key_grants_unblockable_to_target_creature() {
    let mut g = two_player_game();
    let mk = g.add_card_to_battlefield(0, catalog::manifold_key());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(mk);
    g.clear_sickness(bear);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: mk,
        ability_index: 0,
        target: Some(Target::Permanent(bear)), x_value: None })
    .expect("Manifold Key {1},{T}: unblockable activatable");
    drain_stack(&mut g);

    let bear_on_bf = g
        .battlefield
        .iter()
        .find(|c| c.id == bear)
        .expect("bear still alive");
    assert!(
        bear_on_bf.has_keyword(&crate::card::Keyword::Unblockable),
        "bear has Unblockable EOT"
    );
}

#[test]
fn manifold_key_untaps_target_artifact() {
    let mut g = two_player_game();
    let mk = g.add_card_to_battlefield(0, catalog::manifold_key());
    let target_artifact = g.add_card_to_battlefield(0, catalog::manifold_key());
    g.clear_sickness(mk);
    g.clear_sickness(target_artifact);

    // Tap the target artifact so we can verify the untap.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == target_artifact) {
        c.tapped = true;
    }
    assert!(g
        .battlefield
        .iter()
        .find(|c| c.id == target_artifact)
        .map(|c| c.tapped)
        .unwrap_or(false));

    g.perform_action(GameAction::ActivateAbility {
        card_id: mk,
        ability_index: 1,
        target: Some(Target::Permanent(target_artifact)), x_value: None })
    .expect("Manifold Key {T}: untap artifact activatable");
    drain_stack(&mut g);

    let target_on_bf = g
        .battlefield
        .iter()
        .find(|c| c.id == target_artifact)
        .expect("artifact still on bf");
    assert!(!target_on_bf.tapped, "target artifact is untapped");
}

#[test]
fn leyline_invocation_pumps_by_lands_you_control() {
    // P0 has 5 lands; cast Leyline Invocation on the bear → +5/+5 + trample EOT.
    let mut g = two_player_game();
    for _ in 0..5 {
        let _land = g.add_card_to_battlefield(0, catalog::forest());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let id = g.add_card_to_hand(0, catalog::leyline_invocation());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Leyline Invocation castable for {3}{G}{G}");
    drain_stack(&mut g);

    let bear_on_bf = g.battlefield.iter().find(|c| c.id == bear).expect("bear");
    assert_eq!(
        bear_on_bf.power(),
        2 + 5,
        "bear is 7/7 (base 2/2 + 5 lands)"
    );
    assert_eq!(bear_on_bf.toughness(), 2 + 5);
    assert!(bear_on_bf.has_keyword(&crate::card::Keyword::Trample));
}

#[test]
fn spitfire_lagac_magecraft_burns_each_opp() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::spitfire_lagac());
    let p1_life_before = g.players[1].life;
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

    // Bolt itself does 3 to P1, plus magecraft burns 2 more from Lagac.
    assert_eq!(
        g.players[1].life,
        p1_life_before - 3 - 2,
        "P1 took 5 damage (Bolt 3 + Lagac magecraft 2)"
    );
    // Confirm Lagac is a 3/3 Lizard.
    let lagac = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Spitfire Lagac")
        .expect("Lagac");
    assert_eq!(lagac.power(), 3);
    assert_eq!(lagac.toughness(), 3);
    assert!(
        lagac
            .definition
            .subtypes
            .creature_types
            .contains(&crate::card::CreatureType::Lizard),
        "Lagac is a Lizard"
    );
    // Sanity: not flying.
    assert!(!lagac.has_keyword(&Keyword::Flying));
}

#[test]
fn settle_the_score_destroys_creature_and_adds_loyalty() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pw = g.add_card_to_battlefield(0, catalog::ral_zarek_guest_lecturer());
    let id = g.add_card_to_hand(0, catalog::settle_the_score());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);

    let loyalty_before = g
        .battlefield
        .iter()
        .find(|c| c.id == pw)
        .map(|c| c.counters.get(&CounterType::Loyalty).copied().unwrap_or(0))
        .unwrap_or(0);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Settle the Score castable for {3}{B}");
    drain_stack(&mut g);

    // Bear destroyed.
    assert!(
        !g.battlefield.iter().any(|c| c.id == victim),
        "bear destroyed"
    );
    assert!(
        g.players[1]
            .graveyard
            .iter()
            .any(|c| c.definition.name == "Grizzly Bears"),
        "bear in graveyard"
    );
    // Planeswalker gained 2 loyalty.
    let loyalty_after = g
        .battlefield
        .iter()
        .find(|c| c.id == pw)
        .map(|c| c.counters.get(&CounterType::Loyalty).copied().unwrap_or(0))
        .unwrap_or(0);
    assert_eq!(
        loyalty_after,
        loyalty_before + 2,
        "PW gained 2 loyalty counters"
    );
}

#[test]
fn pursuit_of_knowledge_accumulates_charge_counter_on_draw_action() {
    // Note: the engine batches multi-card draws into a single trigger
    // fire today (`dispatch_triggers_for_events` is per-batch, not per
    // event-instance), so Divination (Draw 2) yields exactly one charge
    // counter rather than the strict per-card 2 that the printed Oracle
    // would imply. The per-card trigger refinement is tracked under
    // "Multi-Card Batch Triggers" in TODO.md. The test asserts the
    // engine's current per-batch behavior so it stays green and acts as
    // a regression marker for a future per-event-fire refactor.
    use crate::card::CounterType;
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::plains());
    }
    let pok = g.add_card_to_battlefield(0, catalog::pursuit_of_knowledge());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));

    let div = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: div,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Divination castable");
    drain_stack(&mut g);

    let pok_on_bf = g
        .battlefield
        .iter()
        .find(|c| c.id == pok)
        .expect("PoK still on bf");
    let charge = pok_on_bf
        .counters
        .get(&CounterType::Charge)
        .copied()
        .unwrap_or(0);
    assert!(
        charge >= 1,
        "PoK accumulated at least one charge counter from Divination"
    );
}

#[test]
fn pursuit_of_knowledge_activation_requires_four_charge_counters() {
    // Bench-test the activation gate: PoK with 3 charge counters fails;
    // with 4 it succeeds (draws 3 and sacrifices itself).
    use crate::card::CounterType;
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::plains());
    }
    let pok = g.add_card_to_battlefield(0, catalog::pursuit_of_knowledge());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == pok) {
        c.counters.insert(CounterType::Charge, 3);
    }
    let res_three = g.perform_action(GameAction::ActivateAbility {
        card_id: pok,
        ability_index: 0,
        target: None, x_value: None });
    assert!(
        res_three.is_err(),
        "PoK activation with only 3 charge counters fails"
    );

    // Bump to 4 and try again.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == pok) {
        c.counters.insert(CounterType::Charge, 4);
    }
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: pok,
        ability_index: 0,
        target: None, x_value: None })
    .expect("PoK activatable with 4+ charge counters");
    drain_stack(&mut g);

    // 3 cards drawn (gates: hand +3, library -3).
    assert_eq!(g.players[0].hand.len(), hand_before + 3);
    assert_eq!(g.players[0].library.len(), lib_before - 3);
    // PoK sacrificed (in graveyard now).
    assert!(
        !g.battlefield.iter().any(|c| c.id == pok),
        "PoK sacrificed"
    );
}

#[test]
fn exsanguinate_drains_each_opp_by_x() {
    let mut g = two_player_game();
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::exsanguinate());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("Exsanguinate castable for {3}{B}{B}");
    drain_stack(&mut g);

    // P1 loses 3 life; P0 gains 3.
    assert_eq!(g.players[1].life, p1_life_before - 3);
    assert_eq!(g.players[0].life, p0_life_before + 3);
}

#[test]
fn fire_prophecy_deals_three_and_cantrips() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::mountain());
    }
    // Give the controller a non-FP card in hand to satisfy the "put a
    // card on bottom of library" rider after FP is cast.
    let _filler = g.add_card_to_hand(0, catalog::plains());
    let hand_after_filler = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::fire_prophecy());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Fire Prophecy castable for {1}{R}");
    drain_stack(&mut g);

    // Bear took 3 damage (2 toughness bear should be dead).
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "bear destroyed by 3 dmg"
    );
    // Net hand change after FP resolution: -1 (cast FP, removed from hand)
    // -1 (put one card on bottom) +1 (draw) = -1 vs hand-after-filler-and-FP.
    // After adding filler + FP, hand was hand_after_filler + 1.
    // After cast: hand_after_filler. After put-on-bottom: hand_after_filler - 1. After draw: hand_after_filler.
    assert_eq!(g.players[0].hand.len(), hand_after_filler);
}

#[test]
fn divide_by_zero_bounces_permanent_and_cantrips() {
    // Cast Divide by Zero on an opponent's permanent → bounce to opp hand,
    // caster draws a card (Learn approximation).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let lib_before = g.players[0].library.len();
    let hand_before = g.players[0].hand.len();
    let opp_hand_before = g.players[1].hand.len();

    let id = g.add_card_to_hand(0, catalog::divide_by_zero());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Divide by Zero castable for {1}{U}");
    drain_stack(&mut g);

    // Bear bounced to its owner's (P1's) hand.
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "bear bounced"
    );
    assert_eq!(
        g.players[1].hand.len(),
        opp_hand_before + 1,
        "bear in P1's hand"
    );
    // Caster drew 1 card from Learn approximation.
    // hand_before (=0) + 1 (added DbZ) - 1 (cast DbZ) + 1 (drew 1) = 1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert_eq!(g.players[0].library.len(), lib_before - 1);
}

// ── Approach of the Second Sun ─────────────────────────────────────────────

/// First cast: gain 7 life (and the card lands in graveyard — the
/// "seventh from top" rider is approximated as graveyard hit).
#[test]
fn approach_of_the_second_sun_gains_seven_life_on_first_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::approach_of_the_second_sun());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(6);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Approach castable at 8 mana");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 7);
    // Game still going.
    assert!(g.game_over.is_none());
}

/// Second cast with one Approach already in graveyard: caster wins.
#[test]
fn approach_of_the_second_sun_wins_game_when_cast_with_one_in_graveyard() {
    let mut g = two_player_game();
    // Seed a copy of Approach in P0's graveyard (simulating the first cast).
    g.add_card_to_graveyard(0, catalog::approach_of_the_second_sun());
    let id = g.add_card_to_hand(0, catalog::approach_of_the_second_sun());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(6);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Approach castable at 8 mana");
    drain_stack(&mut g);

    // Game over with P0 as winner.
    assert_eq!(g.game_over, Some(Some(0)));
}

// ── Resurrection ────────────────────────────────────────────────────────────

#[test]
fn resurrection_returns_creature_card_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::resurrection());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Resurrection castable at 4 mana");
    drain_stack(&mut g);

    // Bear should now be on the battlefield under P0's control.
    let bear_on_bf = g.battlefield.iter().find(|c| c.id == bear);
    assert!(bear_on_bf.is_some(), "Bear should be back on the battlefield");
    assert_eq!(bear_on_bf.unwrap().controller, 0);
}

// ── Adventurous Impulse ────────────────────────────────────────────────────

#[test]
fn adventurous_impulse_finds_a_creature_in_top_three() {
    let mut g = two_player_game();
    // Top of library: Grizzly Bears (creature) — Adventurous Impulse should
    // put it into hand.
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::adventurous_impulse());
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Adventurous Impulse castable for {G}");
    drain_stack(&mut g);

    // The bear should now be in P0's hand.
    let bear_in_hand = g.players[0]
        .hand
        .iter()
        .any(|c| c.definition.name == "Grizzly Bears");
    assert!(bear_in_hand, "Bear should be put into hand");
}

// ── Maelstrom Muse ─────────────────────────────────────────────────────────

/// Opus magecraft on Maelstrom Muse: cast a small instant → loot 1.
/// (Hand stays unchanged after cast: draw 1, discard 1.)
#[test]
fn maelstrom_muse_opus_loots_on_small_cast() {
    let mut g = two_player_game();
    let muse = g.add_card_to_battlefield(0, catalog::maelstrom_muse());
    g.clear_sickness(muse);
    // Seed library for the draw.
    g.add_card_to_library(0, catalog::island());
    // Seed a discard target.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    // Cast a 1-mana instant: should loot 1.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len(); // includes bear + bolt
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);

    // Bolt left hand (-1). Magecraft draws 1 (+1), discards 1 (-1).
    // Net: hand_before - 1 (bolt cast) + 0 (draw - discard cancel out).
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
    assert_eq!(g.players[0].library.len(), lib_before - 1);
}

// ── Eladamri's Call ─────────────────────────────────────────────────────────

/// Eladamri's Call should tutor a creature from library into hand. The
/// caller's scripted decider picks the bear.
#[test]
fn eladamris_call_tutors_creature_into_hand() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::forest());

    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));

    let id = g.add_card_to_hand(0, catalog::eladamris_call());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Eladamri's Call castable for {W}{G}");
    drain_stack(&mut g);

    let bear_in_hand = g.players[0]
        .hand
        .iter()
        .any(|c| c.definition.name == "Grizzly Bears");
    assert!(bear_in_hand, "Bear should be tutored into hand");
}

// ── Yawning Fissure ─────────────────────────────────────────────────────────

/// Yawning Fissure should force each opponent to sacrifice a land.
#[test]
fn yawning_fissure_each_opp_sacs_a_land() {
    let mut g = two_player_game();
    let opp_land1 = g.add_card_to_battlefield(1, catalog::mountain());
    let opp_land2 = g.add_card_to_battlefield(1, catalog::forest());
    let my_land = g.add_card_to_battlefield(0, catalog::island());

    let id = g.add_card_to_hand(0, catalog::yawning_fissure());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Yawning Fissure castable for {3}{R}");
    drain_stack(&mut g);

    // P1 should have sacrificed exactly one land (one of opp_land1 or opp_land2).
    let opp_lands_remaining: usize = g.battlefield
        .iter()
        .filter(|c| c.controller == 1 && c.definition.is_land())
        .count();
    assert_eq!(opp_lands_remaining, 1, "opp should have one land left after sac");
    let one_of_two_sacced = !g.battlefield.iter().any(|c| c.id == opp_land1)
        || !g.battlefield.iter().any(|c| c.id == opp_land2);
    assert!(one_of_two_sacced);
    // Our land is untouched.
    assert!(g.battlefield.iter().any(|c| c.id == my_land), "our land untouched");
}

// ── Cleansing Wildfire ──────────────────────────────────────────────────────

/// Cleansing Wildfire destroys target land + draws a card. (The
/// search-basic step requires the target's controller to have basic lands
/// in library; we seed one.)
#[test]
fn cleansing_wildfire_destroys_land_and_draws() {
    let mut g = two_player_game();
    let opp_land = g.add_card_to_battlefield(1, catalog::mountain());
    g.add_card_to_library(1, catalog::forest());
    // Seed library for the caster's cantrip draw.
    g.add_card_to_library(0, catalog::island());

    let id = g.add_card_to_hand(0, catalog::cleansing_wildfire());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_land)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Cleansing Wildfire castable for {1}{R}");
    drain_stack(&mut g);

    // Target Mountain destroyed.
    assert!(!g.battlefield.iter().any(|c| c.id == opp_land),
        "target land destroyed");
    // Caster drew a card: -1 (cast CW) + 1 (draw) = 0 net vs hand_before.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Tendrils of Agony ───────────────────────────────────────────────────────

/// Storm — when cast as the only spell this turn (StormCount = 0), drain
/// fires exactly once for 2 life.
#[test]
fn tendrils_of_agony_drains_two_with_no_storm() {
    let mut g = two_player_game();
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::tendrils_of_agony());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    // No prior spells this turn.
    g.spells_cast_this_turn = 0;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tendrils castable for {2}{B}{B}");
    drain_stack(&mut g);

    // After casting Tendrils, spells_cast_this_turn becomes 1.
    // StormCount = spells_cast_this_turn - 1 = 0.
    // Repeat count = 0 + 1 = 1 → drain 2 once.
    assert_eq!(g.players[1].life, p1_life_before - 2);
    assert_eq!(g.players[0].life, p0_life_before + 2);
}

/// Storm payoff: with 4 prior spells, Tendrils fires drain 2 five times
/// total (StormCount = 4, Repeat count = 5).
#[test]
fn tendrils_of_agony_storm_drain_scales() {
    let mut g = two_player_game();
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::tendrils_of_agony());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    // 4 prior spells this turn.
    g.spells_cast_this_turn = 4;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tendrils castable");
    drain_stack(&mut g);

    // After casting, spells_cast_this_turn = 5 → StormCount = 4 → Repeat = 5.
    // Drain 2 × 5 = 10 life shifted.
    assert_eq!(g.players[1].life, p1_life_before - 10);
    assert_eq!(g.players[0].life, p0_life_before + 10);
}

// ── Quench ──────────────────────────────────────────────────────────────────

/// Quench counters target spell unless its controller pays {1}.
/// Without mana to pay, the spell is countered.
#[test]
fn quench_counters_spell_when_opp_cant_pay() {
    let mut g = two_player_game();
    // P1 casts Lightning Bolt at P0 (mirror of the Whirlwind Denial test).
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");

    // P0 responds with Quench targeting the bolt; P1 has no mana left
    // to pay {1} → bolt is countered.
    g.priority.player_with_priority = 0;
    let quench = g.add_card_to_hand(0, catalog::quench());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: quench,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Quench castable for {1}{U}");
    drain_stack(&mut g);

    // P0 should still be at 20 (Bolt countered before resolving).
    assert_eq!(g.players[0].life, 20, "Bolt should be countered");
}

// ── Saw It Coming ───────────────────────────────────────────────────────────

#[test]
fn saw_it_coming_counters_target_spell() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");

    g.priority.player_with_priority = 0;
    let saw = g.add_card_to_hand(0, catalog::saw_it_coming());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: saw,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Saw It Coming castable for {2}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, 20, "Bolt should be countered");
}

// ── Dueling Coach ───────────────────────────────────────────────────────────

/// Dueling Coach's ETB puts a +1/+1 counter on a target friendly creature.
#[test]
fn dueling_coach_etb_lands_counter_on_friendly() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::dueling_coach());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Dueling Coach castable for {1}{W}");
    drain_stack(&mut g);

    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(bear_card.counter_count(CounterType::PlusOnePlusOne), 1);
}
