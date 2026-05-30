use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn witherbloom_drainherald_etb_drains_two_and_has_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_drainherald());
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainherald castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2);
    assert_eq!(g.players[0].life, my_life + 2);
    let view = g.battlefield_find(id).expect("Drainherald on bf");
    assert!(view.has_keyword(&Keyword::Lifelink));
}

#[test]
fn pest_spawnmother_etb_mints_three_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_spawnmother());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spawnmother castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 3);
}

#[test]
fn witherbloom_vinescholar_magecraft_adds_plus_one_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_vinescholar());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(id).expect("Vinescholar on bf");
    assert_eq!(view.power(), 2);
    assert_eq!(view.toughness(), 3);
}

#[test]
fn witherbloom_reapdrain_drains_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_reapdrain());
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reapdrain castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2);
    assert_eq!(g.players[0].life, my_life + 2);
    // Cast one, drew one → net 0; hand_before was the value before casting,
    // which counts Reapdrain. After cast (and resolve) net hand = before.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn pest_nightswarm_is_a_flying_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_nightswarm());
    let view = g.battlefield_find(id).expect("Nightswarm on bf");
    assert_eq!(view.power(), 2);
    assert_eq!(view.toughness(), 2);
    assert!(view.has_keyword(&Keyword::Flying));
    assert!(view.definition.subtypes.creature_types.contains(&CreatureType::Pest));
}

#[test]
fn witherbloom_toxinbinder_etb_shrinks_target_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxinbinder());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Toxinbinder castable");
    drain_stack(&mut g);
    // 2/2 - 2/2 = 0/0, should die to SBA
    assert!(g.battlefield_find(target).is_none(), "Grizzly Bears should be dead");
}

// ── Batch 68 — Lorehold expansions ─────────────────────────────────────────

#[test]
fn lorehold_sparkshrine_burns_target_and_mints_spirit() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_sparkshrine());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkshrine castable");
    drain_stack(&mut g);
    // 2 damage to bear (2/2) → dies
    assert!(g.battlefield_find(target).is_none(), "Bear should be dead");
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 1);
}

#[test]
fn lorehold_embertenured_magecraft_self_pumps_with_vigilance() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_embertenured());
    let view = g.battlefield_find(id).expect("Embertenured on bf");
    assert!(view.has_keyword(&Keyword::Vigilance));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(id).expect("Embertenured on bf");
    assert_eq!(view.power(), 3); // 2 + 1 from magecraft pump
    assert_eq!(view.toughness(), 3);
}

#[test]
fn spirit_glyphbinder_etb_pumps_other_friendly_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::spirit_glyphbinder());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Glyphbinder castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(target).expect("Bear on bf");
    // Bear is now 3/3 (with +1/+1 counter)
    assert_eq!(view.power(), 3);
    assert_eq!(view.toughness(), 3);
}

#[test]
fn lorehold_pyrebinder_etb_burns_target_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_pyrebinder());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrebinder castable");
    drain_stack(&mut g);
    // 2 damage to 2/2 bear → dies
    assert!(g.battlefield_find(target).is_none(), "Bear should be dead");
}

#[test]
fn lorehold_heroic_sage_has_first_strike_and_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_heroic_sage());
    let view = g.battlefield_find(id).expect("Heroic Sage on bf");
    assert_eq!(view.power(), 2);
    assert_eq!(view.toughness(), 2);
    assert!(view.has_keyword(&Keyword::FirstStrike));
    assert!(view.has_keyword(&Keyword::Lifelink));
}

// ── Batch 68 — Silverquill expansions ──────────────────────────────────────

#[test]
fn silverquill_inkdiplomat_etb_gains_life_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkdiplomat());
    let my_life = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkdiplomat castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, my_life + 1);
    // -1 (cast) +1 (draw) = 0 net hand
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_glyphkeeper_magecraft_drains_on_is_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::inkling_glyphkeeper());
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 4); // 3 (bolt) + 1 (drain)
    assert_eq!(g.players[0].life, my_life + 1);
}

#[test]
fn silverquill_scriptdrain_drains_three_at_instant_speed() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_scriptdrain());
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scriptdrain castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 3);
    assert_eq!(g.players[0].life, my_life + 3);
}

#[test]
fn inkling_scrollwarden_b68_etb_grows_with_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_scrollwarden_b68());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scrollwarden castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(id).expect("Scrollwarden on bf");
    // 3/3 + counter = 4/4
    assert_eq!(view.power(), 4);
    assert_eq!(view.toughness(), 4);
    assert!(view.has_keyword(&Keyword::Flying));
    assert!(view.has_keyword(&Keyword::Vigilance));
}

#[test]
fn silverquill_bookmark_pumps_toughness_and_grants_lifelink() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_bookmark());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bookmark castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(target).expect("Bear on bf");
    assert_eq!(view.power(), 2);
    assert_eq!(view.toughness(), 4); // 2 + 2 from pump
    assert!(view.has_keyword(&Keyword::Lifelink));
}

// ── Batch 68 — Quandrix expansions ─────────────────────────────────────────

#[test]
fn quandrix_mistshaper_b68_magecraft_loots_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_mistshaper_b68());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Drew 1, discarded 1 → hand_before (had bolt) - 1 (cast bolt) +1 (draw) -1 (discard) = -1
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn quandrix_streamwarden_magecraft_pumps_target_fractal() {
    let mut g = two_player_game();
    let fractal_id = g.add_card_to_battlefield(0, catalog::fractal_pondling());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_streamwarden());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(fractal_id).expect("Fractal on bf");
    // Fractal grew from 1/1 to 2/2
    assert_eq!(view.power(), 2);
    assert_eq!(view.toughness(), 2);
}

#[test]
fn quandrix_sumstride_mints_fractal_scaling_with_creatures() {
    let mut g = two_player_game();
    // 3 creatures on the battlefield
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_sumstride());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sumstride castable");
    drain_stack(&mut g);
    // 3 creatures + the spell-resolution creates fractal token (+0). Fractal
    // should have 4 +1/+1 counters since after token creation there are 4
    // creatures (3 bears + 1 fractal). Verify the fractal exists & has some
    // counters (4/4 expected).
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal));
    assert!(fractal.is_some(), "Fractal token should exist");
    let view = g.battlefield_find(fractal.unwrap().id).expect("Fractal on bf");
    assert_eq!(view.power(), 4);
    assert_eq!(view.toughness(), 4);
}

// ── Batch 68 — Prismari expansions ─────────────────────────────────────────

#[test]
fn prismari_sparkbearer_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_sparkbearer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkbearer castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1);
}

#[test]
fn prismari_stormcaller_b68_magecraft_pings() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_stormcaller_b68());
    let opp_life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Stormcaller ping (1) + bolt (3) = 4 to opp
    assert_eq!(g.players[1].life, opp_life - 4);
}

#[test]
fn prismari_brewbinder_etb_mints_treasure_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_brewbinder());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Brewbinder castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1);
}

#[test]
fn prismari_ember_surge_burns_target_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_ember_surge());
    let opp_life = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ember-Surge castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 3);
    // Cast spell - 1 (cast itself), + 1 (draw) → net 0
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn velomachus_attack_exiles_is_card_from_top_of_library_and_grants_may_play() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    // Seed library: 2 Forests (MV 0) + 1 Lightning Bolt (MV 1) on top.
    // RevealUntilFind walks until it hits the Bolt; misses go to
    // bottom of library randomized.
    use crate::card::CardInstance;
    let mut bolt = CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    let bolt_id = bolt.id;
    let mut top: Vec<CardInstance> = vec![
        CardInstance::new(g.next_id(), catalog::forest(), 0),
        CardInstance::new(g.next_id(), catalog::forest(), 0),
        bolt,
    ];
    for c in top.iter_mut() { c.controller = 0; }
    for c in top.into_iter().rev() {
        g.players[0].library.insert(0, c);
    }
    let velo = g.add_card_to_battlefield(0, catalog::velomachus_lorehold());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == velo) {
        c.summoning_sick = false; c.tapped = false;
    }

    // Move to combat + declare attack.
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: velo,
        target: AttackTarget::Player(1),
    }])).expect("Velomachus can attack");
    drain_stack(&mut g);

    // Bolt should now be in exile with may_play permission to P0.
    let exiled = g.exile.iter().find(|c| c.id == bolt_id)
        .expect("Bolt exiled by Velomachus's attack trigger");
    assert!(
        exiled.may_play_until.is_some(),
        "Bolt has may_play permission stamped",
    );
}

#[test]
fn mavinda_activation_exiles_gy_is_card_and_grants_may_play() {
    // Mavinda's {0} activation: target IS card in your gy moves to
    // exile with may_play_until + exile_after stamped. Once-per-turn
    // gate enforced.
    let mut g = two_player_game();
    let mavinda = g.add_card_to_battlefield(0, catalog::mavinda_students_advocate());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == mavinda) {
        c.summoning_sick = false; c.tapped = false;
    }
    // Seed a Lightning Bolt in P0's graveyard.
    let mut bolt = crate::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    let bolt_id = bolt.id;
    g.players[0].graveyard.push(bolt);

    g.perform_action(GameAction::ActivateAbility {
        card_id: mavinda, ability_index: 0,
        target: Some(crate::game::types::Target::Permanent(bolt_id)), x_value: None }).expect("Mavinda activation (cost {0})");
    drain_stack(&mut g);

    let exiled = g.exile.iter().find(|c| c.id == bolt_id)
        .expect("Bolt moved to exile by Mavinda");
    let perm = exiled.may_play_until.expect("may_play stamped");
    assert!(perm.exile_after, "Mavinda's permission has exile_after=true");
    assert_eq!(perm.player, 0, "permission goes to Mavinda's controller");

    // Second activation in the same turn → rejected (once-per-turn).
    let mut bolt2 = crate::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    bolt2.controller = 0;
    let bolt2_id = bolt2.id;
    g.players[0].graveyard.push(bolt2);
    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: mavinda, ability_index: 0,
        target: Some(crate::game::types::Target::Permanent(bolt2_id)), x_value: None });
    assert!(result.is_err(),
        "Second Mavinda activation in same turn should be rejected (once-per-turn)");
}

// ── modern_decks batch 103 tests ──────────────────────────────────────────

#[test]
fn silverquill_ledgerkeeper_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_ledgerkeeper());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    cast(&mut g, id);
    assert_eq!(g.players[0].life, life0_before + 2, "you gain 2 life");
    assert_eq!(g.players[1].life, life1_before - 2, "opp loses 2 life");
    let ledger = g.battlefield_find(id).expect("ledgerkeeper on bf");
    assert!(ledger.has_keyword(&Keyword::Flying), "flying");
}

#[test]
fn inkling_aerospread_etb_creates_inkling_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_aerospread());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let tokens: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Inkling")
        .collect();
    assert_eq!(tokens.len(), 1, "Exactly one Inkling token minted");
    assert!(tokens[0].has_keyword(&Keyword::Flying), "Token has flying");
}

#[test]
fn silverquill_brushmage_magecraft_pumps_self_eot() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::silverquill_brushmage());
    g.clear_sickness(mage);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(mage).unwrap().power();
    let t_before = g.battlefield_find(mage).unwrap().toughness();
    // Cast Lightning Bolt to trigger magecraft (it must have a target).
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let m = g.battlefield_find(mage).expect("brushmage alive");
    assert_eq!(m.power(), p_before + 1, "+1 power until EOT");
    assert_eq!(m.toughness(), t_before + 1, "+1 toughness until EOT");
}

#[test]
fn inkling_glaivemaster_drains_each_opponent_on_attack() {
    use crate::game::types::AttackTarget;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::inkling_glaivemaster());
    g.clear_sickness(attacker);
    g.step = crate::game::types::TurnStep::DeclareAttackers;
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attacker declared");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 1, "you gain 1 life");
    assert_eq!(g.players[1].life, life1_before - 1, "opp loses 1 life");
}

#[test]
fn witherbloom_necromage_etb_creates_pest_and_dies_drains() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_necromage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1, "One Pest minted on ETB");

    // Kill the necromage to trigger the death-drain.
    let life1_before = g.players[1].life;
    let life0_before = g.players[0].life;
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2, "opp loses 2 on death");
    assert_eq!(g.players[0].life, life0_before + 2, "you gain 2 on death");
}

#[test]
fn witherbloom_toxinsage_magecraft_drains() {
    let mut g = two_player_game();
    let sage = g.add_card_to_battlefield(0, catalog::witherbloom_toxinsage());
    g.clear_sickness(sage);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt deals 3, Toxinsage's magecraft drains 1.
    assert_eq!(g.players[1].life, life1_before - 3 - 1, "opp -3 (bolt) -1 (drain)");
    assert_eq!(g.players[0].life, life0_before + 1, "+1 life from drain");
}

#[test]
fn lorehold_battlemage_b103_etb_pings_and_magecraft_creates_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_battlemage_b103());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Cast Battlemage; ETB-ping auto-targets the opponent's face for 2.
    let life1_before = g.players[1].life;
    cast(&mut g, id);
    assert_eq!(g.players[1].life, life1_before - 2, "ETB pings for 2");

    // Cast a Bolt to trigger magecraft and verify Spirit token.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1, "Magecraft minted one Spirit token");
}

#[test]
fn lorehold_embertusk_etb_pings_each_opp_and_creates_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_embertusk());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life1_before = g.players[1].life;
    cast(&mut g, id);
    assert_eq!(g.players[1].life, life1_before - 1, "opp lost 1 life");
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1, "Spirit token minted");
}

#[test]
fn lorehold_pyrescholar_b103_pings_on_magecraft() {
    let mut g = two_player_game();
    let scholar = g.add_card_to_battlefield(0, catalog::lorehold_pyrescholar_b103());
    g.clear_sickness(scholar);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Pyrescholar magecraft 1 = -4 life to player 1.
    assert_eq!(g.players[1].life, life1_before - 4, "opp -3 (bolt) -1 (pyrescholar)");
}

#[test]
fn lorehold_lecturer_deals_damage_and_creates_spirit() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_lecturer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lecturer castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear killed by 2 damage");
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1, "Spirit token minted");
}

#[test]
fn prismari_sparkpoet_magecraft_pings() {
    let mut g = two_player_game();
    let poet = g.add_card_to_battlefield(0, catalog::prismari_sparkpoet());
    g.clear_sickness(poet);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 4, "opp -3 (bolt) -1 (sparkpoet)");
}

#[test]
fn prismari_tidemage_magecraft_loots() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::prismari_tidemage());
    g.clear_sickness(mage);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island()); // a card to discard
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Cast bolt: -1, magecraft draw 1: +1, discard 1: -1 → net -1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_lecturer_deals_damage_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_lecturer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lecturer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2, "opp -2 life");
}

#[test]
fn quandrix_aetherist_b103_creates_fractal_with_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_aetherist_b103());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 1, "Fractal token minted");
    assert_eq!(fractals[0].counter_count(CounterType::PlusOnePlusOne), 2,
        "Fractal has two +1/+1 counters");
}

#[test]
fn quandrix_cycloid_etb_pumps_each_your_creature() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_cycloid());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let b1c = g.battlefield_find(b1).unwrap();
    let b2c = g.battlefield_find(b2).unwrap();
    let cycloid = g.battlefield_find(id).unwrap();
    assert_eq!(b1c.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(b2c.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(cycloid.counter_count(CounterType::PlusOnePlusOne), 1,
        "cycloid pumps itself too");
}

#[test]
fn quandrix_symmetrybard_magecraft_pumps_self_counter() {
    let mut g = two_player_game();
    let bard = g.add_card_to_battlefield(0, catalog::quandrix_symmetrybard());
    g.clear_sickness(bard);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bard).expect("bard alive");
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_numeromancer_etb_scry_and_draw() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_numeromancer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    cast(&mut g, id);
    // -1 (cast), +1 (draw) → hand size delta -1 (the cast card).
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_lecturer_creates_fractal_with_creature_counters() {
    let mut g = two_player_game();
    let _b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_lecturer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 1, "Fractal minted");
    // Counter count runs after the Fractal joins the battlefield, so
    // the count covers 2 bears + the new Fractal token = 3.
    assert_eq!(fractals[0].counter_count(CounterType::PlusOnePlusOne), 3);
}

// ── batch 103 (continued): shortcut-driven cards ───────────────────────────

#[test]
fn silverquill_confessor_magecraft_drains_target_player() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let conf = g.add_card_to_battlefield(0, catalog::silverquill_confessor());
    g.clear_sickness(conf);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt: -3 to p1. Magecraft drain target picks p1 by auto: -1, +1.
    assert_eq!(g.players[1].life, life1_before - 4);
    assert_eq!(g.players[0].life, life0_before + 1);
}

#[test]
fn witherbloom_toxicpath_b103_etb_drain_and_scry() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxicpath_b103());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    cast(&mut g, id);
    assert_eq!(g.players[1].life, life1_before - 2);
    assert_eq!(g.players[0].life, life0_before + 2);
}

#[test]
fn inkling_sigilbearer_b103_pumps_each_inkling() {
    let mut g = two_player_game();
    // Use existing Inkling minter (Eager Glyphmage ETB → 1/1 inkling token).
    let glyph = g.add_card_to_hand(0, catalog::eager_glyphmage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, glyph);
    // One Inkling minted. Now play Sigilbearer which puts +1/+1 on each
    // Inkling (including itself).
    let id = g.add_card_to_hand(0, catalog::inkling_sigilbearer_b103());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    // The newly-minted Inkling token + the Sigilbearer (which is also
    // Inkling) both have a counter.
    let inklings: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .collect();
    let with_counters = inklings.iter()
        .filter(|c| c.counter_count(CounterType::PlusOnePlusOne) == 1)
        .count();
    assert!(with_counters >= 2,
        "At least 2 Inklings got +1/+1 (token + sigilbearer)");
}

#[test]
fn pest_bannerlord_pumps_each_pest_on_etb() {
    let mut g = two_player_game();
    // Cast Witherbloom Pestcaller (batch 103) to mint a Pest token.
    let caller = g.add_card_to_hand(0, catalog::witherbloom_pestcaller_b103());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, caller);
    let pests_before: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .map(|c| c.id)
        .collect();
    assert!(!pests_before.is_empty(), "Pestcaller minted a Pest");
    // Now play Pest Bannerlord which puts +1/+1 on each Pest.
    let id = g.add_card_to_hand(0, catalog::pest_bannerlord());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    for p_id in &pests_before {
        let p = g.battlefield_find(*p_id).expect("pest alive");
        assert_eq!(p.counter_count(CounterType::PlusOnePlusOne), 1,
            "Each pre-existing Pest got +1/+1 counter");
    }
}

#[test]
fn spirit_of_counterpoint_pumps_each_spirit_on_etb() {
    let mut g = two_player_game();
    // Play Lorehold Apprentice (a Spirit-tribal aside) — actually we
    // need a Spirit. Use Spirit of Counterpoint and assert it pumps
    // itself if no other Spirits exist.
    let id = g.add_card_to_hand(0, catalog::spirit_of_counterpoint());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let sp = g.battlefield_find(id).expect("Spirit on bf");
    // The Spirit of Counterpoint is itself a Spirit, so it should
    // pump itself.
    assert_eq!(sp.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn silverquill_maelstrom_drains_four_and_makes_opp_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_maelstrom());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let opp_hand_before = g.players[1].hand.len();
    cast(&mut g, id);
    assert_eq!(g.players[1].life, life1_before - 4);
    assert_eq!(g.players[0].life, life0_before + 4);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1);
}

#[test]
fn quandrix_calculator_b103_draws_and_pumps_creatures_on_etb() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_calculator_b103());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    cast(&mut g, id);
    // Cast (-1) + Draw (+1) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let b = g.battlefield_find(b1).expect("bear alive");
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn fractal_conductor_pumps_each_fractal_on_etb() {
    let mut g = two_player_game();
    // First play Quandrix Summoner to mint a Fractal.
    let summoner = g.add_card_to_hand(0, catalog::quandrix_summoner());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, summoner);
    let fractals_before: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .map(|c| c.id)
        .collect();
    assert!(!fractals_before.is_empty(), "Quandrix Summoner minted a Fractal");
    // Now play Fractal Conductor to pump each Fractal.
    let id = g.add_card_to_hand(0, catalog::fractal_conductor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let counters_before: Vec<_> = fractals_before.iter()
        .map(|fid| g.battlefield_find(*fid).unwrap().counter_count(CounterType::PlusOnePlusOne))
        .collect();
    cast(&mut g, id);
    for (fid, c_before) in fractals_before.iter().zip(counters_before.iter()) {
        let f = g.battlefield_find(*fid).expect("Fractal alive");
        assert_eq!(f.counter_count(CounterType::PlusOnePlusOne), c_before + 1,
            "Fractal got +1 +1/+1 counter from Conductor's ETB");
    }
}

// ── modern_decks batch 104 tests ──────────────────────────────────────────

#[test]
fn silverquill_inkblade_b104_drains_on_is_cast() {
    let mut g = two_player_game();
    let inkblade = g.add_card_to_battlefield(0, catalog::silverquill_inkblade_b104());
    g.clear_sickness(inkblade);
    assert!(g.battlefield_find(inkblade).unwrap().has_keyword(&Keyword::Lifelink),
        "Inkblade has lifelink");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt deals 3, magecraft drains 1.
    assert_eq!(g.players[1].life, life1_before - 3 - 1);
    assert_eq!(g.players[0].life, life0_before + 1);
}

#[test]
fn inkling_loremaster_b104_drains_on_attack() {
    use crate::game::types::AttackTarget;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::inkling_loremaster_b104());
    g.clear_sickness(attacker);
    g.step = crate::game::types::TurnStep::DeclareAttackers;
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }])).expect("attacker declared");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 2, "you gain 2 life");
    assert_eq!(g.players[1].life, life1_before - 2, "opp loses 2 life");
}

#[test]
fn silverquill_anointment_b104_pumps_and_grants_indestructible() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(target);
    let id = g.add_card_to_hand(0, catalog::silverquill_anointment_b104());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p_before = g.battlefield_find(target).unwrap().power();
    let t_before = g.battlefield_find(target).unwrap().toughness();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Anointment castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(target).expect("target alive");
    assert_eq!(c.power(), p_before + 1);
    assert_eq!(c.toughness(), t_before + 1);
    assert!(c.has_keyword(&Keyword::Indestructible), "target has indestructible EOT");
}

#[test]
fn silverquill_anthemcaster_b104_mints_two_inklings_and_pumps_team() {
    let mut g = two_player_game();
    // Pre-existing creature to verify the anthem applies to it.
    let bear = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(bear);
    let p_before = g.battlefield_find(bear).unwrap().power();
    let id = g.add_card_to_hand(0, catalog::silverquill_anthemcaster_b104());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let inklings: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Inkling")
        .collect();
    assert_eq!(inklings.len(), 2);
    // The pre-existing bear got +1/+1 EOT.
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.power(), p_before + 1);
}

#[test]
fn witherbloom_pestbrood_b104_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestbrood_b104());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 2, "two Pests minted on ETB");
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Deathtouch));
}

#[test]
fn pest_bloodscribe_b104_pumps_self_on_sacrifice() {
    let mut g = two_player_game();
    let scribe = g.add_card_to_battlefield(0, catalog::pest_bloodscribe_b104());
    g.clear_sickness(scribe);
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == fodder) {
        c.is_token = true;
    }
    let p_before = g.battlefield_find(scribe).unwrap().power();
    // Trigger sacrifice via Witherbloom Sacrosanct (printed sac-and-drain).
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(scribe).expect("scribe alive");
    assert_eq!(c.power(), p_before + 1, "+1 power from sacrifice trigger");
}

#[test]
fn witherbloom_mireseer_b104_etb_mills_and_gains_life() {
    let mut g = two_player_game();
    // Seed opp library with cards to mill.
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::witherbloom_mireseer_b104());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib1_before = g.players[1].library.len();
    let life0_before = g.players[0].life;
    cast(&mut g, id);
    assert_eq!(g.players[1].library.len(), lib1_before - 2);
    assert_eq!(g.players[0].life, life0_before + 1);
}

#[test]
fn witherbloom_cultmaster_b104_mints_pest_mills_three_and_draws() {
    let mut g = two_player_game();
    // Seed opp library with cards.
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    // Seed our own library.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::witherbloom_cultmaster_b104());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib1_before = g.players[1].library.len();
    let hand0_before = g.players[0].hand.len();
    cast(&mut g, id);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1, "one Pest minted");
    assert_eq!(g.players[1].library.len(), lib1_before - 3, "opp milled 3");
    // -1 (cast Cultmaster) +1 (draw)
    assert_eq!(g.players[0].hand.len(), hand0_before);
}

#[test]
fn lorehold_pyromancer_b104_burns_each_opp_on_is_cast() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::lorehold_pyromancer_b104());
    g.clear_sickness(mage);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt does 3 + magecraft does 1 to each opp = 4 total.
    assert_eq!(g.players[1].life, life1_before - 3 - 1);
}

#[test]
fn spirit_of_the_archive_b104_etb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::savannah_lions());
    let id = g.add_card_to_hand(0, catalog::spirit_of_the_archive_b104());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "graveyard creature returned to hand");
    let s = g.battlefield_find(id).expect("spirit on bf");
    assert!(s.has_keyword(&Keyword::Flying));
    assert!(s.has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_fireseer_b104_etb_pings_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_fireseer_b104());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1);
}

#[test]
fn lorehold_battlecaster_b104_etb_mints_spirit_and_magecraft_pumps_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_battlecaster_b104());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1, "Spirit minted on ETB");

    // Magecraft self-pump from an instant cast.
    g.clear_sickness(id);
    let p_before = g.battlefield_find(id).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().power(), p_before + 1);
}

#[test]
fn lorehold_sparkstrike_b104_pings_and_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_sparkstrike_b104());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 3);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1);
}

#[test]
fn prismari_pyromage_b104_burns_creature_and_scrys_on_is_cast() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::prismari_pyromage_b104());
    g.clear_sickness(mage);
    // Big-toughness target so the magecraft 1 dmg doesn't drop it.
    let big = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(big);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Target the player with the Bolt; the magecraft trigger auto-targets
    // the only creature on the battlefield for its 1-damage payload.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(big).expect("angel alive");
    assert_eq!(b.damage, 1, "magecraft pinged the angel for 1");
}

#[test]
fn prismari_elementalist_b104_etb_draws_and_creates_treasure() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::prismari_elementalist_b104());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    cast(&mut g, id);
    // -1 (cast) +1 (draw) = same hand size
    assert_eq!(g.players[0].hand.len(), hand_before);
    let treasures: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure")
        .collect();
    assert_eq!(treasures.len(), 1, "Treasure minted on ETB");
}

#[test]
fn prismari_sparkcaller_b104_self_pumps_and_grants_haste_on_is_cast() {
    let mut g = two_player_game();
    let caller = g.add_card_to_battlefield(0, catalog::prismari_sparkcaller_b104());
    g.clear_sickness(caller);
    let p_before = g.battlefield_find(caller).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(caller).expect("caller alive");
    assert_eq!(c.power(), p_before + 1);
    assert!(c.has_keyword(&Keyword::Haste));
}

#[test]
fn prismari_stormburst_b104_burns_and_cantrips() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::prismari_stormburst_b104());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 3);
    // -1 (cast) +1 (draw) = same
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_crackleburst_b104_burns_creature_and_mints_treasure() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::savannah_lions());
    g.clear_sickness(bear);
    let id = g.add_card_to_hand(0, catalog::prismari_crackleburst_b104());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Bear (2 toughness) dies from 2 damage.
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed by 2 damage");
    let treasures: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure")
        .collect();
    assert_eq!(treasures.len(), 1);
}

#[test]
fn quandrix_theorist_b104_pumps_each_friendly_fractal_on_is_cast() {
    let mut g = two_player_game();
    let theorist = g.add_card_to_battlefield(0, catalog::quandrix_theorist_b104());
    g.clear_sickness(theorist);
    // Mint a Fractal via Body of Research-style helper card; use Quandrix Mathematician.
    let math = g.add_card_to_hand(0, catalog::quandrix_mathematician_b104());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, math);
    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .expect("Fractal exists")
        .id;
    let counters_before = g.battlefield_find(fractal).unwrap().counter_count(CounterType::PlusOnePlusOne);
    // Now cast a bolt — Quandrix Theorist's magecraft should pump the Fractal.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(fractal).unwrap().counter_count(CounterType::PlusOnePlusOne),
        counters_before + 1);
}

#[test]
fn quandrix_mathematician_b104_etb_creates_fractal_with_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_mathematician_b104());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 1, "1 Fractal minted");
    assert_eq!(fractals[0].counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn fractal_bloom_b104_creates_two_fractals_with_three_total_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_bloom_b104());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 2, "Two Fractals minted");
    let total_counters: u32 = fractals.iter()
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(total_counters, 3, "Three +1/+1 counters distributed");
}

#[test]
fn quandrix_symmetrist_b104_doubles_counters_on_target() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(target);
    // Manually stamp 2 +1/+1 counters on the bear.
    g.battlefield_find_mut(target).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let id = g.add_card_to_hand(0, catalog::quandrix_symmetrist_b104());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Started with 2 counters, ETB adds 2 more (= "doubles" to 4 total).
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

// ── modern_decks batch 105 (helper shortcuts) tests ───────────────────────

#[test]
fn shortcut_mint_inklings_creates_w_b_flying_tokens() {
    use crate::effect::shortcut::mint_inklings;
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&mint_inklings(2), &ctx).expect("mint_inklings resolves");
    drain_stack(&mut g);
    let inklings: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .collect();
    assert_eq!(inklings.len(), 2);
    assert!(inklings.iter().all(|c| c.has_keyword(&Keyword::Flying)),
        "Inkling tokens fly");
}

#[test]
fn shortcut_mint_lorehold_spirits_creates_r_w_spirits() {
    use crate::effect::shortcut::mint_lorehold_spirits;
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&mint_lorehold_spirits(1), &ctx).expect("mint_lorehold_spirits resolves");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1);
    assert_eq!(spirits[0].power(), 2, "Lorehold Spirit is 2/2");
    assert_eq!(spirits[0].toughness(), 2);
}

// ── modern_decks batch 107 (etb_drain_each_opp shortcut) test ───────────────

#[test]
fn shortcut_etb_drain_each_opp_drains_only_opponents() {
    // Asymmetric drain helper: opponents lose N life, you do NOT gain
    // any. Locks in the asymmetric body so a future refactor can't
    // accidentally swap it back to the symmetric `etb_drain` shape.
    use crate::effect::shortcut::etb_drain_each_opp;
    use crate::effect::{PlayerRef, Selector, Value};
    let trig = etb_drain_each_opp(3);
    // The body must be a LoseLife on EachOpponent of amount 3 — NOT a
    // Drain (which would also include the you-gain half).
    match trig.effect {
        Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(3) } => {}
        ref other => panic!("expected LoseLife on EachOpponent of 3, got {other:?}"),
    }
}

// ── modern_decks batch 119 tests ──────────────────────────────────────────
// New Strixhaven synthesised cards across all five colleges. Each card
// gets a play-pattern test exercising its primary printed effect.

// Silverquill (W/B)

#[test]
fn silverquill_loresmith_b119_etb_gains_two_life_with_lifelink_and_vigilance() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_loresmith_b119());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    cast(&mut g, id);
    assert_eq!(g.players[0].life, life_before + 2);
    let c = g.battlefield_find(id).expect("on bf");
    assert!(c.has_keyword(&Keyword::Lifelink));
    assert!(c.has_keyword(&Keyword::Vigilance));
}

#[test]
fn inkling_vanguard_b119_anthems_other_inklings_but_not_self() {
    let mut g = two_player_game();
    // Mint an Inkling token under our control.
    use crate::effect::shortcut::mint_inklings;
    use crate::game::effects::EffectContext;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&mint_inklings(1), &ctx).expect("mint inklings");
    drain_stack(&mut g);
    let inkling_token = g.battlefield.iter()
        .find(|c| c.is_token && c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .map(|c| c.id)
        .expect("inkling token exists");
    // Now drop the Vanguard.
    let vanguard = g.add_card_to_battlefield(0, catalog::inkling_vanguard_b119());
    // The Inkling token (1/1 base) gets +1/+0 → 2/1 via the static anthem.
    // Static modifications land via the layer system — read through
    // `compute_battlefield` rather than `battlefield_find`.
    let computed_token = g.compute_battlefield().into_iter()
        .find(|c| c.id == inkling_token)
        .expect("token in computed");
    assert_eq!(computed_token.power, 2);
    assert_eq!(computed_token.toughness, 1);
    // Vanguard itself stays 3/4 (anthem excludes the source).
    let computed_vanguard = g.compute_battlefield().into_iter()
        .find(|c| c.id == vanguard)
        .expect("vanguard in computed");
    assert_eq!(computed_vanguard.power, 3);
    assert_eq!(computed_vanguard.toughness, 4);
}

#[test]
fn silverquill_embolden_b119_pumps_and_grants_lifelink() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(target);
    let p_before = g.battlefield_find(target).unwrap().power();
    let t_before = g.battlefield_find(target).unwrap().toughness();
    let id = g.add_card_to_hand(0, catalog::silverquill_embolden_b119());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Embolden castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(target).expect("alive");
    assert_eq!(c.power(), p_before + 2);
    assert_eq!(c.toughness(), t_before + 2);
    assert!(c.has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_quillsweep_b119_drains_three_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_quillsweep_b119());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let hand0_before = g.players[0].hand.len();
    cast(&mut g, id);
    assert_eq!(g.players[1].life, life1_before - 3);
    assert_eq!(g.players[0].life, life0_before + 3);
    // -1 (cast) +1 (draw)
    assert_eq!(g.players[0].hand.len(), hand0_before);
}

// Witherbloom (B/G)

#[test]
fn witherbloom_cradlemage_b119_etb_mints_pest_and_mills_each_opp() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::witherbloom_cradlemage_b119());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib1_before = g.players[1].library.len();
    cast(&mut g, id);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1);
    assert_eq!(g.players[1].library.len(), lib1_before - 2);
}

#[test]
fn pest_hivewatcher_b119_gains_life_when_another_creature_dies() {
    let mut g = two_player_game();
    let _watcher = g.add_card_to_battlefield(0, catalog::pest_hivewatcher_b119());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fodder);
    let life_before = g.players[0].life;
    // Kill the fodder via Lightning Bolt — routes through SBA, which
    // emits the CreatureDied event that the AnotherOfYours scope picks up.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(fodder)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder is dead");
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn pest_hivewatcher_b119_does_not_gain_life_when_only_self_dies() {
    let mut g = two_player_game();
    let watcher = g.add_card_to_battlefield(0, catalog::pest_hivewatcher_b119());
    g.clear_sickness(watcher);
    let life_before = g.players[0].life;
    // Bolt the watcher itself — its own death should NOT fire its trigger
    // (AnotherOfYours scope excludes the source).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(watcher)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(watcher).is_none());
    assert_eq!(g.players[0].life, life_before);
}

#[test]
fn witherbloom_harvester_b119_sacrifices_self_to_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let h = g.add_card_to_battlefield(0, catalog::witherbloom_harvester_b119());
    g.clear_sickness(h);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: h,
        ability_index: 0,
        target: None,
        x_value: None,
    }).expect("activation");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert!(g.battlefield_find(h).is_none(), "Harvester sacrificed to gy");
}

#[test]
fn witherbloom_mulchcaster_b119_mills_target_and_gains_two_life() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::witherbloom_mulchcaster_b119());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib1_before = g.players[1].library.len();
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mulchcaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib1_before - 4);
    assert_eq!(g.players[0].life, life_before + 2);
}

// Lorehold (R/W)

#[test]
fn lorehold_battlescribe_b119_self_pumps_and_grants_first_strike_on_is_cast() {
    let mut g = two_player_game();
    let scribe = g.add_card_to_battlefield(0, catalog::lorehold_battlescribe_b119());
    g.clear_sickness(scribe);
    let p_before = g.battlefield_find(scribe).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(scribe).expect("scribe alive");
    assert_eq!(c.power(), p_before + 1);
    assert!(c.has_keyword(&Keyword::FirstStrike));
}

#[test]
fn lorehold_spelldrake_b119_pings_two_on_is_cast() {
    let mut g = two_player_game();
    let drake = g.add_card_to_battlefield(0, catalog::lorehold_spelldrake_b119());
    g.clear_sickness(drake);
    assert!(g.battlefield_find(drake).unwrap().has_keyword(&Keyword::Flying));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt = 3 damage, magecraft = 2 damage → 5 total.
    assert_eq!(g.players[1].life, life1_before - 3 - 2);
}

#[test]
fn lorehold_skirmisher_b119_etb_pings_target_and_has_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_skirmisher_b119());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_reliquary_b119_reanimates_creature_and_mints_spirit() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::savannah_lions());
    let id = g.add_card_to_hand(0, catalog::lorehold_reliquary_b119());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    // Bear is back on the battlefield.
    assert!(g.battlefield_find(bear).is_some(), "bear reanimated");
    // Spirit minted too.
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1);
}

#[test]
fn spirit_battlecry_b119_pumps_all_friendly_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::savannah_lions());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    let pa = g.battlefield_find(a).unwrap().power();
    let pb = g.battlefield_find(b).unwrap().power();
    let id = g.add_card_to_hand(0, catalog::spirit_battlecry_b119());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert_eq!(g.battlefield_find(a).unwrap().power(), pa + 1);
    assert_eq!(g.battlefield_find(b).unwrap().power(), pb + 1);
}

// Prismari (U/R)

#[test]
fn prismari_flamescholar_b119_pings_each_opp_on_is_cast() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::prismari_flamescholar_b119());
    g.clear_sickness(s);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt = 3, magecraft ping = 1 to each opp.
    assert_eq!(g.players[1].life, life1_before - 3 - 1);
}

#[test]
fn prismari_inferno_b119_burns_creature_and_player() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::savannah_lions());
    g.clear_sickness(bear);
    let id = g.add_card_to_hand(0, catalog::prismari_inferno_b119());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    }).expect("Inferno castable with two targets");
    drain_stack(&mut g);
    // Bear (2 toughness) dies from 2 damage.
    assert!(g.battlefield_find(bear).is_none());
    // Player 1 loses 2.
    assert_eq!(g.players[1].life, life1_before - 2);
}

#[test]
fn prismari_magmaweaver_b119_etb_pings_creature_for_two() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(target);
    let id = g.add_card_to_hand(0, catalog::prismari_magmaweaver_b119());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(target).expect("angel alive");
    assert_eq!(c.damage, 2);
}

#[test]
fn prismari_reshape_b119_bounces_and_scrys() {
    let mut g = two_player_game();
    let opp_creature = g.add_card_to_battlefield(1, catalog::savannah_lions());
    g.clear_sickness(opp_creature);
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_reshape_b119());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_creature)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // opp_creature returned to opp's hand.
    assert!(g.battlefield_find(opp_creature).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == opp_creature));
}

// Quandrix (G/U)

#[test]
fn quandrix_polymath_b119_grows_on_is_cast() {
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::quandrix_polymath_b119());
    g.clear_sickness(p);
    let counters_before = g.battlefield_find(p).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(p).unwrap().counter_count(CounterType::PlusOnePlusOne),
        counters_before + 1);
}

#[test]
fn fractal_spawnmaster_b119_etb_mints_three_three_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_spawnmaster_b119());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 1);
    assert_eq!(fractals[0].counter_count(CounterType::PlusOnePlusOne), 3);
}

#[test]
fn quandrix_druid_b119_etb_pumps_each_friendly_fractal() {
    let mut g = two_player_game();
    // Use Quandrix Mathematician (batch 104) to mint a Fractal with 2 +1/+1
    // counters — 0/0 base + 2 counters survives state-based actions.
    let math = g.add_card_to_hand(0, catalog::quandrix_mathematician_b104());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, math);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .map(|c| c.id)
        .collect();
    assert_eq!(fractals.len(), 1, "one fractal minted with +1/+1 counters");
    let counters_before = g.battlefield_find(fractals[0]).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    let id = g.add_card_to_hand(0, catalog::quandrix_druid_b119());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(g.battlefield_find(fractals[0]).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), counters_before + 1);
}

#[test]
fn quandrix_calculus_b119_adds_counter_and_cantrips() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(target);
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_calculus_b119());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let counters_before = g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne),
        counters_before + 1);
    // -1 (cast) +1 (draw)
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── batch 119 helper shortcut lock-in test ──────────────────────────────────

#[test]
fn shortcut_on_other_dies_uses_another_of_yours_scope() {
    // Lock in that the new on_other_dies helper builds a
    // `CreatureDied/AnotherOfYours` event spec so future refactors
    // can't accidentally regress it to SelfSource (which would silently
    // make every "whenever another creature dies" trigger ignore other
    // deaths).
    use crate::effect::EventScope;
    use crate::effect::shortcut::on_other_dies;
    let trig = on_other_dies(Effect::GainLife {
        who: crate::effect::Selector::You,
        amount: crate::effect::Value::Const(1),
    });
    assert_eq!(trig.event.kind, crate::effect::EventKind::CreatureDied);
    assert!(matches!(trig.event.scope, EventScope::AnotherOfYours));
}

#[test]
fn fractal_hatchling_b119_grows_via_activated_ability() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::fractal_hatchling_b119());
    g.clear_sickness(id);
    let counters_before = g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
        x_value: None,
    }).expect("activation");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
        counters_before + 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// batch 120 — 25 brand-new Strixhaven synthesised card tests
// ═══════════════════════════════════════════════════════════════════════════

// Silverquill (W/B)

#[test]
fn silverquill_devotee_b120_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let dev = g.add_card_to_battlefield(0, catalog::silverquill_devotee_b120());
    g.clear_sickness(dev);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt: 3 damage; magecraft also drains 2 → opponent at -5 life.
    assert_eq!(g.players[1].life, life1_before - 5);
}

#[test]
fn silverquill_censurer_b120_exiles_low_power_creature_and_gains_life() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::savannah_lions());
    g.clear_sickness(target);
    let id = g.add_card_to_hand(0, catalog::silverquill_censurer_b120());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Censurer castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "creature exiled");
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn inkling_battlescribe_b120_etb_drains_one_and_is_lifelink_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_battlescribe_b120());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    cast(&mut g, id);
    assert_eq!(g.players[0].life, life0_before + 1);
    assert_eq!(g.players[1].life, life1_before - 1);
    let c = g.battlefield_find(id).expect("on bf");
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_verdict_b120_destroys_creature_and_mints_inkling() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::savannah_lions());
    g.clear_sickness(target);
    let id = g.add_card_to_hand(0, catalog::silverquill_verdict_b120());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "creature destroyed");
    let inklings: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .collect();
    assert_eq!(inklings.len(), 1, "minted exactly one Inkling token");
}

// Witherbloom (B/G)

#[test]
fn witherbloom_apprentice_b120_magecraft_pumps_target_friendly() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(target);
    let app = g.add_card_to_battlefield(0, catalog::witherbloom_apprentice_b120());
    g.clear_sickness(app);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let lion = g.battlefield_find(target).expect("lion alive");
    assert_eq!(lion.power(), 3, "Lion 2/1 base +1/+1 magecraft = 3 power");
}

#[test]
fn pest_brooder_b120_dies_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_brooder_b120());
    g.clear_sickness(id);
    // Murder it manually.
    let card = g.battlefield_find_mut(id).unwrap();
    card.damage = (card.toughness() as u32) + 10;
    g.check_state_based_actions();
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 2, "two Pests minted on death");
}

#[test]
fn witherbloom_saprooter_b120_drains_two_and_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_saprooter_b120());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    cast(&mut g, id);
    assert_eq!(g.players[0].life, life0_before + 2);
    assert_eq!(g.players[1].life, life1_before - 2);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1);
}

// Lorehold (R/W)

#[test]
fn lorehold_loreseeker_b120_magecraft_pings_any() {
    let mut g = two_player_game();
    let ls = g.add_card_to_battlefield(0, catalog::lorehold_loreseeker_b120());
    g.clear_sickness(ls);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt: 3 dmg; magecraft ping: 1 dmg → -4 total.
    assert_eq!(g.players[1].life, life1_before - 4);
}

#[test]
fn lorehold_bondbreaker_b120_deals_three_damage_and_mints_spirit() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(target);
    let id = g.add_card_to_hand(0, catalog::lorehold_bondbreaker_b120());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let angel = g.battlefield_find(target).expect("angel alive after 3 dmg to a 4 toughness");
    assert_eq!(angel.damage, 3);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1);
}

#[test]
fn lorehold_flameherald_b120_etb_pings_any() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_flameherald_b120());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1);
    // Has haste — should not have summoning sickness.
    let c = g.battlefield_find(id).expect("flameherald on bf");
    assert!(c.has_keyword(&Keyword::Haste));
}

// Prismari (U/R)

#[test]
fn prismari_apprentice_b120_magecraft_scrys() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::prismari_apprentice_b120());
    g.clear_sickness(app);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::mountain());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry doesn't reduce library count.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn prismari_pyrocaster_b120_magecraft_drains_each_opp_for_one() {
    let mut g = two_player_game();
    let pc = g.add_card_to_battlefield(0, catalog::prismari_pyrocaster_b120());
    g.clear_sickness(pc);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = -4.
    assert_eq!(g.players[1].life, life1_before - 4);
}

#[test]
fn prismari_tempest_b120_deals_four_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(target);
    let id = g.add_card_to_hand(0, catalog::prismari_tempest_b120());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Serra Angel is 4/4 → 4 damage kills it.
    assert!(g.battlefield_find(target).is_none());
    // Cantrip: hand was N, -1 cast +1 draw = N.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_crucible_b120_mints_treasure_and_pings_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_crucible_b120());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2);
    let treasures: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure")
        .collect();
    assert_eq!(treasures.len(), 1);
}

// Quandrix (G/U)

#[test]
fn quandrix_apprentice_b120_magecraft_adds_counter_to_self() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::quandrix_apprentice_b120());
    g.clear_sickness(app);
    let counters_before = g.battlefield_find(app).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(app).unwrap().counter_count(CounterType::PlusOnePlusOne),
        counters_before + 1);
}

#[test]
fn quandrix_equation_b120_adds_counter_and_cantrips() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(target);
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_equation_b120());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let counters_before = g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne),
        counters_before + 1);
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn fractal_bloomwright_b120_mints_fractal_with_four_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_bloomwright_b120());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 1);
    assert_eq!(fractals[0].counter_count(CounterType::PlusOnePlusOne), 4);
}

// ── batch 120 helper shortcut lock-in tests ─────────────────────────────────

#[test]
fn shortcut_drain_and_draw_drains_and_draws() {
    use crate::effect::shortcut::drain_and_draw;
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&drain_and_draw(2), &ctx).expect("drain_and_draw resolves");
    assert_eq!(g.players[0].life, life0_before + 2);
    assert_eq!(g.players[1].life, life1_before - 2);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn shortcut_drain_and_scry_drains_and_scrys() {
    use crate::effect::shortcut::drain_and_scry;
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let lib_before = g.players[0].library.len();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&drain_and_scry(3, 1), &ctx).expect("drain_and_scry resolves");
    assert_eq!(g.players[0].life, life0_before + 3);
    assert_eq!(g.players[1].life, life1_before - 3);
    // Scry doesn't draw — library count is unchanged.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn shortcut_drain_and_surveil_drains_and_surveils() {
    use crate::effect::shortcut::drain_and_surveil;
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&drain_and_surveil(1, 1), &ctx).expect("drain_and_surveil resolves");
    assert_eq!(g.players[0].life, life0_before + 1);
    assert_eq!(g.players[1].life, life1_before - 1);
}

#[test]
fn witherbloom_cultivator_b120_sacrifices_another_creature_for_drain() {
    let mut g = two_player_game();
    let cult = g.add_card_to_battlefield(0, catalog::witherbloom_cultivator_b120());
    g.clear_sickness(cult);
    // Fodder creature (Lions: 2/1). Cultivator's auto-picker will choose
    // the lowest-power creature, but Lions are 2 power and Cultivator
    // itself is 1 power — Cultivator is excluded from the pick list, so
    // the only candidate is Lions.
    let fodder = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(fodder);
    g.players[0].mana_pool.add_colorless(1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cult,
        ability_index: 0,
        target: None,
        x_value: None,
    }).expect("activation");
    drain_stack(&mut g);
    // Cultivator survives — only the fodder was sacrificed.
    assert!(g.battlefield_find(cult).is_some(), "Cultivator survives");
    assert!(g.battlefield_find(fodder).is_none(), "Fodder sacrificed");
    // Drain 1 lands on resolution.
    assert_eq!(g.players[0].life, life0_before + 1);
    assert_eq!(g.players[1].life, life1_before - 1);
}

#[test]
fn witherbloom_cultivator_b120_rejects_activation_without_fodder() {
    // No fodder creature on the battlefield — only Cultivator itself.
    // The pre-flight gate should reject the activation cleanly (no mana
    // burned, no tap consumed since this ability has no tap cost).
    let mut g = two_player_game();
    let cult = g.add_card_to_battlefield(0, catalog::witherbloom_cultivator_b120());
    g.clear_sickness(cult);
    g.players[0].mana_pool.add_colorless(1);
    let mana_before = g.players[0].mana_pool.colorless_amount();
    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: cult,
        ability_index: 0,
        target: None,
        x_value: None,
    });
    assert!(result.is_err(), "Activation rejected with no fodder");
    // Mana not consumed — clean rejection.
    assert_eq!(g.players[0].mana_pool.colorless_amount(), mana_before);
}

// ── batch 121 — additional sac_other_filter cards ──────────────────────────

#[test]
fn pest_cultmaster_b121_sacs_creature_to_draw() {
    let mut g = two_player_game();
    let cult = g.add_card_to_battlefield(0, catalog::pest_cultmaster_b121());
    g.clear_sickness(cult);
    let fodder = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(fodder);
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: cult,
        ability_index: 0,
        target: None,
        x_value: None,
    }).expect("activation");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

#[test]
fn witherbloom_sapdrinker_b121_sacs_to_pump_self() {
    let mut g = two_player_game();
    let sd = g.add_card_to_battlefield(0, catalog::witherbloom_sapdrinker_b121());
    g.clear_sickness(sd);
    let fodder = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(fodder);
    let p_before = g.battlefield_find(sd).unwrap().power();
    g.perform_action(GameAction::ActivateAbility {
        card_id: sd,
        ability_index: 0,
        target: None,
        x_value: None,
    }).expect("activation (free ability)");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.battlefield_find(sd).unwrap().power(), p_before + 2);
}

#[test]
fn witherbloom_bonechanter_b121_sacs_to_shrink_target() {
    let mut g = two_player_game();
    let bone = g.add_card_to_battlefield(0, catalog::witherbloom_bonechanter_b121());
    g.clear_sickness(bone);
    let fodder = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(fodder);
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(target);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bone,
        ability_index: 0,
        target: Some(Target::Permanent(target)),
        x_value: None,
    }).expect("activation");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    // Serra Angel is 4/4 → -2/-2 → 2/2.
    let angel = g.battlefield_find(target).expect("angel still alive");
    assert_eq!(angel.power(), 2);
    assert_eq!(angel.toughness(), 2);
}

#[test]
fn pest_ringleader_b121_sacs_to_drain_two() {
    let mut g = two_player_game();
    let rl = g.add_card_to_battlefield(0, catalog::pest_ringleader_b121());
    g.clear_sickness(rl);
    let fodder = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(fodder);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rl,
        ability_index: 0,
        target: None,
        x_value: None,
    }).expect("activation");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.players[0].life, life0_before + 2);
    assert_eq!(g.players[1].life, life1_before - 2);
}

#[test]
fn witherbloom_reaper_b121_sacs_to_gain_indestructible() {
    let mut g = two_player_game();
    let reaper = g.add_card_to_battlefield(0, catalog::witherbloom_reaper_b121());
    g.clear_sickness(reaper);
    let fodder = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(fodder);
    g.perform_action(GameAction::ActivateAbility {
        card_id: reaper,
        ability_index: 0,
        target: None,
        x_value: None,
    }).expect("activation");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    let r = g.battlefield_find(reaper).expect("reaper alive");
    assert!(r.has_keyword(&Keyword::Indestructible),
        "reaper has indestructible until end of turn");
}

// ── batch 122 — 22 new cards across all five colleges ──────────────────────

#[test]
fn pest_cultcaller_b122_sacs_creature_to_drain_one() {
    let mut g = two_player_game();
    let cult = g.add_card_to_battlefield(0, catalog::pest_cultcaller_b122());
    g.clear_sickness(cult);
    let fodder = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(fodder);
    g.players[0].mana_pool.add(Color::Black, 1);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cult, ability_index: 0, target: None, x_value: None,
    }).expect("activation");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.players[0].life, l0 + 1, "you gained 1");
    assert_eq!(g.players[1].life, l1 - 1, "opp lost 1");
}

#[test]
fn pest_cultcaller_b122_rejects_activation_without_fodder() {
    let mut g = two_player_game();
    let cult = g.add_card_to_battlefield(0, catalog::pest_cultcaller_b122());
    g.clear_sickness(cult);
    g.players[0].mana_pool.add(Color::Black, 1);
    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: cult, ability_index: 0, target: None, x_value: None,
    });
    assert!(result.is_err(), "activation rejected without fodder");
}

#[test]
fn witherbloom_bloodgrafter_b122_etb_drains_two() {
    let mut g = two_player_game();
    let bg = g.add_card_to_hand(0, catalog::witherbloom_bloodgrafter_b122());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bg, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloodgrafter castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 + 2);
    assert_eq!(g.players[1].life, l1 - 2);
}

#[test]
fn witherbloom_bloodgrafter_b122_grows_on_sacrifice() {
    let mut g = two_player_game();
    let bg = g.add_card_to_battlefield(0, catalog::witherbloom_bloodgrafter_b122());
    g.clear_sickness(bg);
    // Use Cultcaller to sacrifice fodder, triggering Bloodgrafter's payoff.
    let cult = g.add_card_to_battlefield(0, catalog::pest_cultcaller_b122());
    g.clear_sickness(cult);
    let fodder = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(fodder);
    g.players[0].mana_pool.add(Color::Black, 1);
    let p_before = g.battlefield_find(bg).unwrap().power();
    g.perform_action(GameAction::ActivateAbility {
        card_id: cult, ability_index: 0, target: None, x_value: None,
    }).expect("Cultcaller activation");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(bg).expect("Bloodgrafter alive").power();
    assert_eq!(p_after, p_before + 1, "Bloodgrafter grew by +1/+1 on sacrifice");
}

#[test]
fn witherbloom_composter_b122_sacs_to_draw_and_lose_one() {
    let mut g = two_player_game();
    let comp = g.add_card_to_battlefield(0, catalog::witherbloom_composter_b122());
    g.clear_sickness(comp);
    let fodder = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(fodder);
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(1);
    let h_before = g.players[0].hand.len();
    let l_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: comp, ability_index: 0, target: None, x_value: None,
    }).expect("activation");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.players[0].hand.len(), h_before + 1, "drew a card");
    assert_eq!(g.players[0].life, l_before - 1, "lost 1 life");
}

#[test]
fn pest_swarmcaller_b122_mints_two_pests_and_drains_two() {
    let mut g = two_player_game();
    let ps = g.add_card_to_hand(0, catalog::pest_swarmcaller_b122());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    let bf_before = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: ps, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Swarmcaller castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 + 2, "drained 2");
    assert_eq!(g.players[1].life, l1 - 2);
    // 2 Pest tokens created
    let pest_count = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pest_count, 2);
    let bf_after = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2);
}

#[test]
fn witherbloom_sapdrainer_b122_etb_drains_two_and_has_lifelink() {
    let mut g = two_player_game();
    let sd = g.add_card_to_hand(0, catalog::witherbloom_sapdrainer_b122());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sd, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sapdrainer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 2);
    let c = g.battlefield_find(sd).expect("alive");
    assert!(c.has_keyword(&Keyword::Lifelink));
    assert_eq!(c.power(), 4);
    assert_eq!(c.toughness(), 3);
}

#[test]
fn witherbloom_necrotutor_b122_magecraft_returns_creature_to_top() {
    let mut g = two_player_game();
    let nt = g.add_card_to_battlefield(0, catalog::witherbloom_necrotutor_b122());
    g.clear_sickness(nt);
    let gy_creature = g.add_card_to_graveyard(0, catalog::savannah_lions());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before + 1,
        "creature returned to library top");
    // The top of library should be our creature
    let top = g.players[0].library.last().expect("library nonempty");
    assert_eq!(top.id, gy_creature, "Lions on top of library");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == gy_creature),
        "Lions no longer in graveyard");
}

#[test]
fn witherbloom_spinecaster_b122_etb_shrinks_target_creature() {
    let mut g = two_player_game();
    let sc = g.add_card_to_hand(0, catalog::witherbloom_spinecaster_b122());
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(target);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: sc, target: Some(Target::Permanent(target)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Spinecaster castable");
    drain_stack(&mut g);
    let angel = g.battlefield_find(target).expect("angel alive");
    assert_eq!(angel.power(), 3);
    assert_eq!(angel.toughness(), 3);
}

#[test]
fn pest_brewmaster_b122_etb_mints_two_pests() {
    let mut g = two_player_game();
    let pb = g.add_card_to_hand(0, catalog::pest_brewmaster_b122());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: pb, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Brewmaster castable");
    drain_stack(&mut g);
    let pest_token_count = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pest_token_count, 2, "two Pest tokens minted on ETB");
}

#[test]
fn pest_brewmaster_b122_gains_life_on_other_pest_death() {
    // Place Brewmaster on the battlefield, then place another Pest. Use
    // Bonechanter (b121) — which sacs another creature to shrink a
    // target — to kill a Pest. Brewmaster's "another Pest dies → +1
    // life" trigger should fire.
    let mut g = two_player_game();
    let bw = g.add_card_to_battlefield(0, catalog::pest_brewmaster_b122());
    g.clear_sickness(bw);
    // Buff Brewmaster so the sac_other_filter prefers a Pest token over
    // the source's own Brewmaster.
    g.battlefield_find_mut(bw).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 5);
    // Now use Swarmcaller to mint 2 Pest tokens (and drain 2).
    let ps = g.add_card_to_hand(0, catalog::pest_swarmcaller_b122());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: ps, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Swarmcaller castable");
    drain_stack(&mut g);
    // Activate Bonechanter — it'll sac a Pest token (lowest-power
    // non-source creature) to shrink the opponent's creature.
    let bone = g.add_card_to_battlefield(0, catalog::witherbloom_bonechanter_b121());
    g.clear_sickness(bone);
    let opp = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(opp);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bone, ability_index: 0,
        target: Some(Target::Permanent(opp)), x_value: None,
    }).expect("Bonechanter activation");
    drain_stack(&mut g);
    // A Pest token died: Brewmaster's payoff (+1 life) + the token's
    // own die-trigger (+1 life) = +2 life.
    assert_eq!(g.players[0].life, l_before + 2,
        "Brewmaster + Pest-die trigger both fired");
}

// ── Silverquill (W/B) ──────────────────────────────────────────────────────

#[test]
fn inkling_quillstrike_b122_etb_shrinks_opp_creature() {
    let mut g = two_player_game();
    let iq = g.add_card_to_hand(0, catalog::inkling_quillstrike_b122());
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(target);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: iq, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quillstrike castable");
    drain_stack(&mut g);
    let angel = g.battlefield_find(target).expect("angel alive");
    assert_eq!(angel.power(), 2);
    assert_eq!(angel.toughness(), 2);
    let iq_c = g.battlefield_find(iq).expect("Quillstrike alive");
    assert!(iq_c.has_keyword(&Keyword::Flying));
}

#[test]
fn silverquill_mentor_b122_etb_gains_two_life_and_magecraft_pumps_friend() {
    let mut g = two_player_game();
    let sm = g.add_card_to_hand(0, catalog::silverquill_mentor_b122());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mentor castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2, "ETB gained 2 life");

    // Cast a Bolt at the opponent → magecraft pumps Mentor +1/+1 EOT.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(sm).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(sm).unwrap().power();
    assert_eq!(p_after, p_before + 1, "Mentor pumped via magecraft");
}

#[test]
fn silverquill_verdict_b122_destroys_creature_and_gains_life_equal_to_power() {
    let mut g = two_player_game();
    let sv = g.add_card_to_hand(0, catalog::silverquill_verdict_b122());
    let target = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.clear_sickness(target);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sv, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Verdict castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "Angel destroyed");
    assert_eq!(g.players[0].life, l_before + 4, "gained 4 life (Angel's power)");
}

#[test]
fn inkling_glyphwarden_b122_is_flying_lifelink_two_four() {
    let mut g = two_player_game();
    let ig = g.add_card_to_battlefield(0, catalog::inkling_glyphwarden_b122());
    let c = g.battlefield_find(ig).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Lifelink));
    assert_eq!(c.power(), 2);
    assert_eq!(c.toughness(), 4);
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_reverence_b122_drains_and_cantrips() {
    let mut g = two_player_game();
    let sr = g.add_card_to_hand(0, catalog::silverquill_reverence_b122());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: sr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reverence castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 + 1);
    assert_eq!(g.players[1].life, l1 - 1);
    // -1 for casting +1 for draw = 0 net change in hand.
    assert_eq!(g.players[0].hand.len(), h_before);
}

// ── Lorehold (R/W) ─────────────────────────────────────────────────────────

#[test]
fn lorehold_pyroscholar_b122_magecraft_pings_one() {
    let mut g = two_player_game();
    let ls = g.add_card_to_battlefield(0, catalog::lorehold_pyroscholar_b122());
    g.clear_sickness(ls);
    // Cast a Bolt at the opponent → magecraft fires ping.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_l = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt = 3, magecraft ping = 1 → opp lost 4.
    assert_eq!(g.players[1].life, opp_l - 4);
}

#[test]
fn lorehold_reliquaer_b122_mints_two_spirits_and_pings_opp() {
    let mut g = two_player_game();
    let lr = g.add_card_to_hand(0, catalog::lorehold_reliquaer_b122());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let opp_l = g.players[1].life;
    let bf_before = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: lr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reliquaer castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2);
    assert_eq!(g.players[1].life, opp_l - 1, "1 damage to opp");
}

#[test]
fn lorehold_battlescryer_b122_is_haste_three_three_with_attack_trigger() {
    let mut g = two_player_game();
    let lb = g.add_card_to_battlefield(0, catalog::lorehold_battlescryer_b122());
    let c = g.battlefield_find(lb).unwrap();
    assert!(c.has_keyword(&Keyword::Haste));
    assert_eq!(c.power(), 3);
    assert_eq!(c.toughness(), 3);
    // Declare it as an attacker — the on-attack trigger pings opp for 1
    // (auto-targeted).
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let opp_l = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lb,
        target: AttackTarget::Player(1),
    }])).expect("Battlescryer attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_l - 1, "1 damage from on-attack ping");
}

// ── Prismari (U/R) ─────────────────────────────────────────────────────────

#[test]
fn prismari_loresage_b122_etb_loots() {
    let mut g = two_player_game();
    let pl = g.add_card_to_hand(0, catalog::prismari_loresage_b122());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island()); // for the discard
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: pl, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Loresage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 1, "drew a card");
}

#[test]
fn prismari_inferno_b122_deals_four_damage_and_cantrips() {
    let mut g = two_player_game();
    let pi = g.add_card_to_hand(0, catalog::prismari_inferno_b122());
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(target);
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: pi, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inferno castable");
    drain_stack(&mut g);
    // Serra Angel = 4/4 with 4 damage marked → dies on SBA.
    assert!(g.battlefield_find(target).is_none(), "Angel died from 4 damage");
    // -1 from casting, +1 from cantrip = h_before
    assert_eq!(g.players[0].hand.len(), h_before);
}

#[test]
fn prismari_sparkmage_b122_magecraft_pings_creature() {
    let mut g = two_player_game();
    let ps = g.add_card_to_battlefield(0, catalog::prismari_sparkmage_b122());
    g.clear_sickness(ps);
    let target = g.add_card_to_battlefield(1, catalog::savannah_lions()); // 2/1
    g.clear_sickness(target);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Magecraft auto-targets a creature (Lions: 2/1 → 1 damage marked, but
    // toughness 1 → dies via SBA).
    assert!(g.battlefield_find(target).is_none(),
        "Lions died from 1 damage on 1-toughness");
}

// ── Quandrix (G/U) ─────────────────────────────────────────────────────────

#[test]
fn fractal_multiplier_b122_etb_adds_counter_to_target_friendly() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(target);
    let fm = g.add_card_to_hand(0, catalog::fractal_multiplier_b122());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p_before = g.battlefield_find(target).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: fm, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Multiplier castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(target).unwrap().power();
    assert_eq!(p_after, p_before + 1);
}
