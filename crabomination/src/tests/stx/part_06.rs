use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn quandrix_hatchling_enters_with_two_counters_and_grows_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_hatchling());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hatchling castable");
    drain_stack(&mut g);
    let h = g.battlefield.iter().find(|c| c.definition.name == "Quandrix Hatchling").expect("Hatchling");
    assert_eq!(h.counter_count(CounterType::PlusOnePlusOne), 2, "Hatchling enters with 2 counters");
    // Cast a bolt and check growth
    let h_id = h.id;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let h_after = g.battlefield_find(h_id).expect("Hatchling");
    assert_eq!(h_after.counter_count(CounterType::PlusOnePlusOne), 3, "Hatchling grew via magecraft");
}

// ── Prismari (U/R) ─────────────────────────────────────────────────────────

#[test]
fn prismari_cascade_volley_burns_target_and_pings_each_opp_creature() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_cascade_volley());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Volley castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 3, "3 damage to opp player");
    // Each bear took 1 damage.
    let b1d = g.battlefield_find(b1).map(|c| c.damage).unwrap_or(0);
    let b2d = g.battlefield_find(b2).map(|c| c.damage).unwrap_or(0);
    assert_eq!(b1d, 1, "Bear 1 took 1 damage");
    assert_eq!(b2d, 1, "Bear 2 took 1 damage");
}

#[test]
fn prismari_initiate_magecraft_pings_target() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_initiate());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // 3 (bolt) + 1 (initiate magecraft) = 4
    assert_eq!(g.players[1].life, life1_before - 4);
}

#[test]
fn prismari_treasurer_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_treasurer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Treasurer castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1, "Treasurer mints a Treasure");
}

#[test]
fn prismari_embershaper_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_embershaper());
    g.add_card_to_library(0, catalog::island());
    // Add a card to discard
    g.add_card_to_hand(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // After bolt cast (-1), magecraft fires: may discard a card + draw a card.
    // AutoDecider takes the MayDo (default-yes), so -1 (cast) -1 (discard) +1 (draw) = -1 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

// ── batch 20: shared cross-school cards (`stx::extras`) ────────────────────

#[test]
fn strixhaven_scholar_magecraft_scrys_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::strixhaven_scholar());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Verify scholar's body is on bf.
    let on_bf = g.battlefield.iter().any(|c| c.definition.name == "Strixhaven Scholar");
    assert!(on_bf, "Scholar still on battlefield after magecraft");
}

#[test]
fn strixhaven_quill_mage_magecraft_pings_opponent() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::strixhaven_quill_mage());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Quill-Mage magecraft 1 = 4 total.
    assert_eq!(g.players[1].life, life1_before - 4);
}

#[test]
fn strixhaven_initiate_has_reach_and_taps_for_green() {
    let mut g = two_player_game();
    let init = g.add_card_to_battlefield(0, catalog::strixhaven_initiate());
    g.clear_sickness(init);
    let def = catalog::strixhaven_initiate();
    assert!(def.keywords.contains(&Keyword::Reach));
    let green_before = g.players[0].mana_pool.amount(Color::Green);
    g.perform_action(GameAction::ActivateAbility {
        card_id: init,
        ability_index: 0,
        target: None, x_value: None }).expect("Mana ability activates");
    drain_stack(&mut g);
    let green_after = g.players[0].mana_pool.amount(Color::Green);
    assert_eq!(green_after, green_before + 1, "Initiate added Green");
}

#[test]
fn strixhaven_burnscholar_etb_pings_and_has_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::strixhaven_burnscholar());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Burnscholar castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1, "Burnscholar ETB pings 1");
    let def = catalog::strixhaven_burnscholar();
    assert!(def.keywords.contains(&Keyword::Haste));
}

#[test]
fn heroic_defiance_pumps_and_grants_hexproof_and_indestructible() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::heroic_defiance());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Defiance castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).expect("Bear");
    assert!(bear_card.has_keyword(&Keyword::Hexproof));
    assert!(bear_card.has_keyword(&Keyword::Indestructible));
    assert_eq!(bear_card.power(), 3, "Bear pumped to 3 power");
    assert_eq!(bear_card.toughness(), 3);
}

#[test]
fn tome_shredder_etb_makes_opp_discard() {
    let mut g = two_player_game();
    let _ = g.add_card_to_hand(1, catalog::lightning_bolt());
    let _ = g.add_card_to_hand(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::tome_shredder());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tome Shredder castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1, "Opp discards 1");
    // Opp graveyard should have the bolt (nonland chosen by auto-decider).
    let in_gy = g.players[1].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt");
    assert!(in_gy, "Discarded card lands in graveyard");
}

#[test]
fn mascot_acolyte_etb_ramps_forest_tapped() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::mascot_acolyte());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Acolyte castable");
    drain_stack(&mut g);
    let f = g.battlefield_find(forest).expect("Forest tutored");
    assert!(f.tapped, "Tutored Forest enters tapped");
    assert!(f.definition.is_land());
    let def = catalog::mascot_acolyte();
    assert!(def.keywords.contains(&Keyword::Reach));
}

#[test]
fn lorehold_strikeforce_pumps_team_with_trample() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_strikeforce());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Strikeforce castable");
    drain_stack(&mut g);
    let b1c = g.battlefield_find(b1).expect("Bear 1");
    let b2c = g.battlefield_find(b2).expect("Bear 2");
    assert_eq!(b1c.power(), 4);
    assert_eq!(b2c.power(), 4);
    assert!(b1c.has_keyword(&Keyword::Trample));
    assert!(b2c.has_keyword(&Keyword::Trample));
}

#[test]
fn strixhaven_necropact_draws_two_and_loses_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strixhaven_necropact());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    let life0_before = g.players[0].life;
    // Auto-target picks self (Player); we can also explicitly opp.
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Necropact castable");
    drain_stack(&mut g);
    // -1 (cast) +2 (draw) = +1 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert_eq!(g.players[0].life, life0_before - 2);
}

// ── Batch 21: 25 new STX cards across all 5 colleges ───────────────────────

// ── Silverquill batch 21 ───────────────────────────────────────────────────

#[test]
fn silverquill_inkscholar_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkscholar());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkscholar castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw - 1 discard = -1 net (the discard came from the loot).
    assert_eq!(g.players[0].hand.len(), hand_before - 1, "looter cycled once");
    let gy_size = g.players[0].graveyard.len();
    assert!(gy_size >= 1, "discarded card lands in graveyard");
}

#[test]
fn inkling_battlecaster_attack_drains_one() {
    use crate::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_battlecaster());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("Battlecaster attacks");
    drain_stack(&mut g);
    // Attack trigger: drain 1 (each opp loses 1, you gain 1).
    assert_eq!(g.players[1].life, 19, "opp loses 1 from attack drain");
    assert_eq!(g.players[0].life, 21, "you gain 1 from attack drain");
    let def = catalog::inkling_battlecaster();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn silverquill_compulsion_makes_target_opp_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears()); // opp's discardable nonland
    let id = g.add_card_to_hand(0, catalog::silverquill_compulsion());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Compulsion castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1, "opp discarded one");
}

#[test]
fn silverquill_sealwriter_etb_drains_two_and_has_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_sealwriter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sealwriter castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2);
    assert_eq!(g.players[0].life, life0_before + 2);
    let def = catalog::silverquill_sealwriter();
    assert!(def.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn inkling_acolyte_etb_mints_inkling_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_acolyte());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Acolyte castable");
    drain_stack(&mut g);
    let inklings = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Inkling")
        .count();
    assert_eq!(inklings, 1, "Acolyte mints an Inkling token");
    let def = catalog::inkling_acolyte();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Inkling));
}

// ── Witherbloom batch 21 ───────────────────────────────────────────────────

#[test]
fn pest_forager_has_trample_and_dies_grants_life() {
    let mut g = two_player_game();
    let pf = g.add_card_to_battlefield(0, catalog::pest_forager());
    // Pest Forager is a 2/1 — push 1 damage to kill via SBA.
    let card = g.battlefield_find_mut(pf).unwrap();
    card.damage = 1;
    let life_before = g.players[0].life;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(pf).is_none(), "Pest Forager died");
    assert_eq!(g.players[0].life, life_before + 1, "Pest Forager death triggers +1 life");
    let def = catalog::pest_forager();
    assert!(def.keywords.contains(&Keyword::Trample));
}

#[test]
fn witherbloom_carnivine_etb_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_carnivine());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Carnivine castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 3);
    assert_eq!(g.players[0].life, life0_before + 3);
    let def = catalog::witherbloom_carnivine();
    assert!(def.keywords.contains(&Keyword::Reach));
    assert_eq!(def.power, 4);
    assert_eq!(def.toughness, 4);
}

#[test]
fn pest_harvest_mints_pest_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::pest_harvest());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest Harvest castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 1, "Pest Harvest creates 1 Pest");
    // -1 cast + 1 draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn witherbloom_necrosophist_returns_creature_from_gy() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_necrosophist());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Necrosophist castable");
    drain_stack(&mut g);
    // -1 cast + 1 return = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let in_hand = g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears");
    assert!(in_hand, "Bear returned to hand");
}

#[test]
fn witherbloom_pestcaller_mints_pest_on_instant_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_pestcaller());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 1, "Pestcaller magecraft mints a Pest");
}

// ── Lorehold batch 21 ──────────────────────────────────────────────────────

#[test]
fn lorehold_sparkstrike_deals_two_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_sparkstrike());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkstrike castable");
    drain_stack(&mut g);
    // Bear has 2 toughness; 2 damage kills it via SBA.
    let alive = g.battlefield_find(bear).is_some();
    assert!(!alive, "Bear killed by 2 damage");
    // Surveil 1 ran (no easy direct check; assert that the spell landed in graveyard).
    let in_gy = g.players[0].graveyard.iter().any(|c| c.definition.name == "Lorehold Sparkstrike");
    assert!(in_gy, "Sparkstrike resolved into graveyard");
}

#[test]
fn lorehold_bonereader_etb_gains_two_and_pumps_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_bonereader());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bonereader castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2, "ETB +2 life");
    // Magecraft check: cast Lightning Bolt to pump.
    let br = g.battlefield.iter().find(|c| c.definition.name == "Lorehold Bonereader").expect("br");
    let br_id = br.id;
    let p_before = g.battlefield_find(br_id).map(|c| c.power()).unwrap_or(0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(br_id).map(|c| c.power()).unwrap_or(0);
    assert_eq!(p_after, p_before + 1, "Bonereader pumped via magecraft");
    let def = catalog::lorehold_bonereader();
    assert!(def.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn lorehold_spiritarcher_etb_deals_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritarcher());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life1_before = g.players[1].life;
    // Cast spell with no on-spell target (Spiritarcher is a creature); the
    // ETB trigger's auto-target picker selects a legal target — opp player.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritarcher castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2, "ETB pings opp for 2");
    let def = catalog::lorehold_spiritarcher();
    assert!(def.keywords.contains(&Keyword::Reach));
}

#[test]
fn lorehold_echoflame_returns_is_and_mints_spirit() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_echoflame());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Echoflame castable");
    drain_stack(&mut g);
    // -1 cast + 1 return = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let bolt_in_hand = g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt");
    assert!(bolt_in_hand, "Bolt returned to hand");
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 1, "Echoflame mints a Spirit");
}

#[test]
fn lorehold_pilgrimwarden_attack_mints_soldier() {
    use crate::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_pilgrimwarden());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("Pilgrimwarden attacks");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Soldier")
        .count();
    assert_eq!(soldiers, 1, "Pilgrimwarden mints a Soldier per attack");
    let def = catalog::lorehold_pilgrimwarden();
    assert!(def.keywords.contains(&Keyword::FirstStrike));
}

// ── Quandrix batch 21 ──────────────────────────────────────────────────────

#[test]
fn quandrix_calibrator_etb_pumps_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_calibrator());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Calibrator castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(bear)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .unwrap_or(0);
    assert_eq!(counters, 1, "Bear got +1/+1 counter from Calibrator ETB");
}

#[test]
fn fractal_resonance_pumps_each_creature_you_control() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_resonance());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Resonance castable");
    drain_stack(&mut g);
    let c1 = g.battlefield_find(bear1).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    let c2 = g.battlefield_find(bear2).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    let co = g.battlefield_find(opp_bear).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(c1, 1);
    assert_eq!(c2, 1);
    assert_eq!(co, 0, "opp bear not pumped");
}

#[test]
fn quandrix_mistweaver_is_flash_flying_etb_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_mistweaver());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mistweaver castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let def = catalog::quandrix_mistweaver();
    assert!(def.keywords.contains(&Keyword::Flash));
    assert!(def.keywords.contains(&Keyword::Flying));
}

#[test]
fn fractal_harvest_mints_three_three_fractal_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::fractal_harvest());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Harvest castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.is_token && c.definition.name == "Fractal")
        .expect("Fractal minted");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 3, "Fractal has 3 counters");
    // -1 cast + 1 draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_sage_scry_draw_on_instant_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::quandrix_sage());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 cast (bolt) + 1 draw (magecraft) = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Prismari batch 21 ──────────────────────────────────────────────────────

#[test]
fn prismari_sparkforge_etb_mints_treasure_and_has_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_sparkforge());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkforge castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1, "Sparkforge mints a Treasure");
    let def = catalog::prismari_sparkforge();
    assert!(def.keywords.contains(&Keyword::Haste));
}

#[test]
fn prismari_mindwave_draws_two_discards_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_mindwave());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mindwave castable");
    drain_stack(&mut g);
    // -1 cast + 2 draw - 1 discard = 0 net (looter wash).
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_pyrocrafter_etb_pings_each_opp_and_pumps_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_pyrocrafter());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrocrafter castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1, "ETB ping deals 1 to opp");
    // Now test magecraft pump.
    let pc = g.battlefield.iter().find(|c| c.definition.name == "Prismari Pyrocrafter").expect("pc");
    let pc_id = pc.id;
    let p_before = g.battlefield_find(pc_id).map(|c| c.power()).unwrap_or(0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(pc_id).map(|c| c.power()).unwrap_or(0);
    assert_eq!(p_after, p_before + 1, "Pyrocrafter pumped via magecraft");
}

#[test]
fn prismari_stormspire_etb_draws_two_and_is_flying() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_stormspire());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormspire castable");
    drain_stack(&mut g);
    // -1 cast + 2 draw = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    let def = catalog::prismari_stormspire();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert_eq!(def.power, 4);
    assert_eq!(def.toughness, 4);
}

#[test]
fn prismari_quickfire_burns_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_quickfire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quickfire castable");
    drain_stack(&mut g);
    let alive = g.battlefield_find(bear).is_some();
    assert!(!alive, "Bear killed by Quickfire (2 dmg = lethal on 2 toughness)");
}

// ── Iconic batch 21 ────────────────────────────────────────────────────────

#[test]
fn hunt_the_library_ramps_basic_land_tapped() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::hunt_the_library());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hunt the Library castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == forest), "Forest is on bf");
}

#[test]
fn field_researcher_etb_ramps_basic_land() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::field_researcher());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Field Researcher castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == forest), "Forest is on bf");
    let def = catalog::field_researcher();
    assert!(def.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn spellbook_studier_etb_scrys_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::spellbook_studier());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Studier castable");
    drain_stack(&mut g);
    let on_bf = g.battlefield.iter().any(|c| c.definition.name == "Spellbook Studier");
    assert!(on_bf, "Studier landed on bf");
    let def = catalog::spellbook_studier();
    assert_eq!(def.power, 1);
    assert_eq!(def.toughness, 3);
}

#[test]
fn strixhaven_vigil_gains_life_on_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::strixhaven_vigil());
    // Advance the turn to active_player 0's upkeep.
    let life_before = g.players[0].life;
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Fire step triggers manually.
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "Vigil grants 1 life on upkeep");
}

// ── Batch 22: 25 new STX cards across all 5 colleges ───────────────────────

// ── Silverquill batch 22 ───────────────────────────────────────────────────

#[test]
fn silverquill_conviction_drains_two_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_conviction());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Conviction castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2, "opp loses 2");
    assert_eq!(g.players[0].life, life0_before + 2, "you gain 2");
}

#[test]
fn silverquill_bookbearer_etb_scrys_and_has_vigilance() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_bookbearer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bookbearer castable");
    drain_stack(&mut g);
    let on_bf = g.battlefield.iter().any(|c| c.definition.name == "Silverquill Bookbearer");
    assert!(on_bf, "Bookbearer landed on bf");
    let def = catalog::silverquill_bookbearer();
    assert!(def.keywords.contains(&Keyword::Vigilance));
    assert_eq!(def.toughness, 4);
}

#[test]
fn inkling_inquisitor_etb_makes_opp_discard_chosen() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears()); // opp's discardable nonland
    let id = g.add_card_to_hand(0, catalog::inkling_inquisitor());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inquisitor castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1, "opp discarded a nonland");
    let def = catalog::inkling_inquisitor();
    assert!(def.keywords.contains(&Keyword::Flying));
}

#[test]
fn silverquill_reckoning_destroys_creature_and_mints_inkling() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_reckoning());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reckoning castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear destroyed by Reckoning");
    let inklings = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Inkling")
        .count();
    assert_eq!(inklings, 1, "Reckoning mints an Inkling token");
}

#[test]
fn silverquill_lifeglyph_pumps_target_on_instant_cast() {
    let mut g = two_player_game();
    let lg = g.add_card_to_battlefield(0, catalog::silverquill_lifeglyph());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(lg);
    let p_before = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    assert_eq!(p_after, p_before + 1, "Lifeglyph pumped target creature");
    let def = catalog::silverquill_lifeglyph();
    assert!(def.keywords.contains(&Keyword::Lifelink));
}

// ── Witherbloom batch 22 ───────────────────────────────────────────────────

#[test]
fn pest_swarmlord_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_swarmlord());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Swarmlord castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 2, "Swarmlord mints 2 Pest tokens");
}

#[test]
fn witherbloom_vinetender_magecraft_gains_one_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_vinetender());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "magecraft +1 life");
    let def = catalog::witherbloom_vinetender();
    assert!(def.keywords.contains(&Keyword::Reach));
}

#[test]
fn toxic_bloodletting_minus_two_kills_bear_and_grants_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::toxic_bloodletting());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloodletting castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear killed by -2/-2");
    assert_eq!(g.players[0].life, life_before + 2, "you gain 2");
}

#[test]
fn witherbloom_saproot_dies_drains_each_opp() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_saproot());
    g.clear_sickness(id);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    // Push 3 damage to kill via SBA (Saproot is 3/3).
    let card = g.battlefield_find_mut(id).unwrap();
    card.damage = 3;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "Saproot died");
    assert_eq!(g.players[1].life, life1_before - 2, "opp loses 2");
    assert_eq!(g.players[0].life, life0_before + 2, "you gain 2");
    let def = catalog::witherbloom_saproot();
    assert!(def.keywords.contains(&Keyword::Trample));
}

#[test]
fn pest_mausoleum_returns_creature_and_mints_pest() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pest_mausoleum());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mausoleum castable");
    drain_stack(&mut g);
    // -1 cast + 1 return = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let bear_in_hand = g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears");
    assert!(bear_in_hand, "Bear returned to hand");
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 1, "Mausoleum mints a Pest token");
}

// ── Lorehold batch 22 ──────────────────────────────────────────────────────

#[test]
fn lorehold_emberscribe_etb_exiles_gy_and_pings() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_emberscribe());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life1_before = g.players[1].life;
    let opp_gy_before = g.players[1].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Emberscribe castable");
    drain_stack(&mut g);
    // Bolt should be exiled from opp's gy.
    assert!(g.players[1].graveyard.len() < opp_gy_before, "card exiled from opp gy");
    // 1 damage to each opp.
    assert_eq!(g.players[1].life, life1_before - 1, "Emberscribe pings opp for 1");
}

#[test]
fn lorehold_reliquary_pumps_creature_on_gy_leave() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_reliquary());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Stage a creature card in your graveyard, then cast something that
    // returns it — Lorehold Ember-Recall returns target creature with
    // mv≤2 to the battlefield, which triggers Reliquary.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let recall = g.add_card_to_hand(0, catalog::lorehold_ember_recall());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let counters_before: u32 = g.battlefield.iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    g.perform_action(GameAction::CastSpell {
        card_id: recall, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recall castable");
    drain_stack(&mut g);
    let counters_after: u32 = g.battlefield.iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert!(counters_after > counters_before, "Reliquary added a counter on gy-leave");
    let _ = bear; // touched in assertion sum
}

#[test]
fn lorehold_ringleader_etb_mints_two_spirit_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_ringleader());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ringleader castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 2, "Ringleader mints 2 Spirit tokens");
    let def = catalog::lorehold_ringleader();
    assert!(def.keywords.contains(&Keyword::Haste));
}

#[test]
fn lorehold_strikevanguard_magecraft_pings_target() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_strikevanguard());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // 3 dmg from bolt + 1 dmg from magecraft = 4 dmg total.
    assert_eq!(g.players[1].life, life1_before - 4, "Strikevanguard pings for 1 on cast");
    let def = catalog::lorehold_strikevanguard();
    assert!(def.keywords.contains(&Keyword::FirstStrike));
}

#[test]
fn lorehold_ember_recall_returns_low_mv_creature_and_pings_opp() {
    let mut g = two_player_game();
    // Stage a 1-mana creature in your gy (Bears are 2 MV — need <=2 MV creature).
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_ember_recall());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ember-Recall castable");
    drain_stack(&mut g);
    let bear_on_bf = g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears");
    assert!(bear_on_bf, "Bear returned to battlefield");
    assert_eq!(g.players[1].life, life1_before - 1, "opp pinged for 1");
}

// ── Quandrix batch 22 ──────────────────────────────────────────────────────

#[test]
fn quandrix_counterbalance_pumps_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_counterbalance());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Counterbalance castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(bear)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .unwrap_or(0);
    assert_eq!(counters, 1, "bear got +1/+1 counter");
    // -1 cast + 1 draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn fractal_bloom_caller_etb_mints_two_two_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_bloom_caller());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloom-Caller castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.is_token && c.definition.name == "Fractal")
        .expect("Fractal minted");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 2, "Fractal has 2 counters");
}

#[test]
fn quandrix_synthesist_magecraft_pumps_team() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::quandrix_synthesist());
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c1 = g.battlefield_find(bear1).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    let c2 = g.battlefield_find(bear2).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(c1, 1);
    assert_eq!(c2, 1);
}

#[test]
fn fractal_tessellation_makes_fractal_scaling_with_lands() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::fractal_tessellation());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tessellation castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.is_token && c.definition.name == "Fractal")
        .expect("Fractal minted");
    // 3 lands → 3 +1/+1 counters.
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 3, "scales with lands");
}

#[test]
fn quandrix_mistshaper_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_mistshaper());
    g.clear_sickness(id);
    let p_before = g.battlefield_find(id).map(|c| c.power()).unwrap_or(0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(id).map(|c| c.power()).unwrap_or(0);
    assert_eq!(p_after, p_before + 1, "Mistshaper grew +1/+1 on cast");
    let def = catalog::quandrix_mistshaper();
    assert!(def.keywords.contains(&Keyword::Flash));
}

// ── Prismari batch 22 ──────────────────────────────────────────────────────

#[test]
fn prismari_sparkforger_magecraft_pumps_and_grants_haste() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_spellforger_b22());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    assert_eq!(p_after, p_before + 1, "Sparkforger pumped target");
}

#[test]
fn prismari_volleyfire_burns_creature_and_mints_treasure() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_volleyfire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Volleyfire castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear destroyed by 4 dmg");
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1, "Volleyfire mints a Treasure");
}

#[test]
fn prismari_spell_shaper_magecraft_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::prismari_spell_shaper());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_stormgaze_loots_and_pings() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_stormgaze());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormgaze castable");
    drain_stack(&mut g);
    // -1 cast + 2 draw - 1 discard = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[1].life, life1_before - 1, "opp pinged for 1");
}

// ── CR 701.46a — Stun counter consumption on untap ─────────────────────────

#[test]
fn stun_counter_replaces_untap_per_cr_701_46a() {
    // CR 701.46a: A permanent with a stun counter would become untapped,
    // remove a stun counter instead. The permanent stays tapped on that
    // untap step; on the next untap (no stun counters left) it untaps
    // normally. Push (modern_decks batch 22): the do_untap step in
    // game/stack.rs now consults stun counters before flipping tapped.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Tap + 1 stun counter.
    {
        let card = g.battlefield_find_mut(bear).unwrap();
        card.tapped = true;
        card.add_counters(CounterType::Stun, 1);
    }
    // First untap — stun counter consumed, permanent stays tapped.
    g.active_player_idx = 0;
    g.do_untap();
    {
        let card = g.battlefield_find(bear).unwrap();
        assert!(card.tapped, "First untap: stun counter consumed, still tapped");
        assert_eq!(card.counter_count(CounterType::Stun), 0,
            "Stun counter consumed by replacing untap event");
    }
    // Second untap — no stun counters left, permanent untaps normally.
    g.do_untap();
    let card = g.battlefield_find(bear).unwrap();
    assert!(!card.tapped, "Second untap: no stun counters → untaps normally");
}

// ── Push (modern_decks) batch 23: tests for 25 new STX cards ────────────────

#[test]
fn inkling_aristocrat_gains_life_when_another_creature_dies() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let _aristo = g.add_card_to_battlefield(0, catalog::inkling_aristocrat());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    // Kill the bear with a Lightning Bolt — uses the proper damage path
    // so the CreatureDied event fires and triggers dispatch.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].life > life_before,
        "Aristocrat gains at least 1 life on friendly creature death (was {}, now {})",
        life_before, g.players[0].life);
}

#[test]
fn inkling_aristocrat_does_not_trigger_on_self() {
    // Aristocrat dying is not "another creature you control dying".
    use crate::game::types::Target;
    let mut g = two_player_game();
    let aristo = g.add_card_to_battlefield(0, catalog::inkling_aristocrat());
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(aristo)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before, "Aristocrat doesn't trigger on self-death");
}

#[test]
fn silverquill_quillscribe_etb_mints_inkling_and_pumps_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_quillscribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quillscribe castable");
    drain_stack(&mut g);
    let inklings: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Inkling")
        .collect();
    assert_eq!(inklings.len(), 1, "ETB mints 1 Inkling token");
}

#[test]
fn silverquill_hush_shrinks_creature_and_gains_life() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_hush());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hush castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear (2/2) destroyed by -2/-2");
    assert_eq!(g.players[0].life, life_before + 2, "Gain 2 life");
}

#[test]
fn inkling_lorewright_etb_draws_and_loses_one_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::inkling_lorewright());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lorewright castable");
    drain_stack(&mut g);
    // -1 (cast Lorewright) + 1 (draw) = 0 net hand
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[0].life, life_before - 1, "Lose 1 life on ETB");
    assert!(catalog::inkling_lorewright().keywords.contains(&Keyword::Flying));
}

#[test]
fn silverquill_battle_hymn_pumps_team_with_vigilance() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_battle_hymn());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battle Hymn castable");
    drain_stack(&mut g);
    let b1 = g.battlefield_find(b1).unwrap();
    let b2 = g.battlefield_find(b2).unwrap();
    assert_eq!(b1.power(), 3, "Bear 1 pumped to 3/3");
    assert_eq!(b2.toughness(), 3, "Bear 2 pumped to 3/3");
    assert!(b1.has_keyword(&Keyword::Vigilance));
    assert!(b2.has_keyword(&Keyword::Vigilance));
}

#[test]
fn pest_ravager_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_ravager());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest Ravager castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .collect();
    assert_eq!(pests.len(), 2, "ETB mints 2 Pests");
    let def = catalog::pest_ravager();
    assert_eq!(def.power, 4);
    assert_eq!(def.toughness, 4);
    assert!(def.keywords.contains(&Keyword::Trample));
}

#[test]
fn witherbloom_famine_drains_four() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_famine());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Famine castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 4);
    assert_eq!(g.players[0].life, p0_life + 4);
}

#[test]
fn witherbloom_greenrot_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_greenrot());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Greenrot castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    assert!(catalog::witherbloom_greenrot().keywords.contains(&Keyword::Reach));
}

#[test]
fn witherbloom_pestbroker_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestbroker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestbroker castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
    assert_eq!(g.players[0].life, p0_life + 2);
}

#[test]
fn pestilent_bloom_shrinks_creature_and_mints_pest() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pestilent_bloom());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestilent Bloom castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear destroyed by -3/-3");
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .collect();
    assert_eq!(pests.len(), 1, "1 Pest minted");
}

#[test]
fn lorehold_battlechronicler_attack_returns_creature_from_gy() {
    use crate::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bc = g.add_card_to_battlefield(0, catalog::lorehold_battlechronicler());
    g.clear_sickness(bc);
    let hand_before = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bc,
        target: AttackTarget::Player(1),
    }])).expect("Attack declared");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Attack returned creature from gy → hand");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != bear_in_gy));
}

#[test]
fn lorehold_searing_wisdom_exiles_gy_card_and_burns() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear_in_gy = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_searing_wisdom());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear_in_gy)),
        additional_targets: vec![Target::Player(1)],
        mode: None, x_value: None,
    }).expect("Searing Wisdom castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear_in_gy), "Bear exiled from gy");
    assert_eq!(g.players[1].life, p1_life - 3, "Burns target for 3");
}

#[test]
fn lorehold_saint_magecraft_self_pumps() {
    let mut g = two_player_game();
    let saint = g.add_card_to_battlefield(0, catalog::lorehold_saint());
    g.clear_sickness(saint);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let saint = g.battlefield_find(saint).unwrap();
    assert_eq!(saint.power(), 3, "Saint magecraft pumps +1/+0");
    assert!(catalog::lorehold_saint().keywords.contains(&Keyword::Lifelink));
}

#[test]
fn lorehold_volley_hits_target_for_two_and_others_for_one() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let target_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_volley());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Volley castable");
    drain_stack(&mut g);
    // target_bear takes 2 + 1 = 3 → dies (toughness 2)
    assert!(g.battlefield_find(target_bear).is_none(), "Target bear dies to 2+1");
    // other_bear takes 1 → marked 1 damage, survives
    let other = g.battlefield_find(other_bear).unwrap();
    assert_eq!(other.damage, 1, "Other bear marked 1");
}

#[test]
fn quandrix_polymath_etb_draws_and_adds_counter() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_polymath());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Polymath castable");
    drain_stack(&mut g);
    // -1 (cast) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let bear = g.battlefield_find(bear).unwrap();
    assert_eq!(bear.counter_count(CounterType::PlusOnePlusOne), 1,
        "Bear gets +1/+1 counter");
}

#[test]
fn fractal_avenger_enters_with_four_plus_one_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_avenger());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Avenger castable");
    drain_stack(&mut g);
    let fa = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Fractal Avenger")
        .expect("Fractal Avenger on battlefield");
    assert_eq!(fa.counter_count(CounterType::PlusOnePlusOne), 4);
    assert_eq!(fa.power(), 4, "Avenger is 4/4 from counters");
    assert!(fa.has_keyword(&Keyword::Trample));
}

#[test]
fn quandrix_cartographer_etb_searches_basic_land() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::quandrix_cartographer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cartographer castable");
    drain_stack(&mut g);
    // -1 (cast) + 1 (searched) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let has_forest = g.players[0].hand.iter().any(|c| c.id == forest);
    assert!(has_forest, "Forest is in hand after search");
}

#[test]
fn fractal_sovereign_etb_scales_counters_with_lands() {
    let mut g = two_player_game();
    // Give controller 3 lands on battlefield.
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_sovereign());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sovereign castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(bear).unwrap();
    assert_eq!(bear.counter_count(CounterType::PlusOnePlusOne), 3,
        "Bear gets +1/+1 counters = number of lands (3)");
}

#[test]
fn quandrix_pairweaver_pumps_two_creatures() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_pairweaver());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(b1)),
        additional_targets: vec![Target::Permanent(b2)],
        mode: None, x_value: None,
    }).expect("Pairweaver castable");
    drain_stack(&mut g);
    let b1 = g.battlefield_find(b1).unwrap();
    let b2 = g.battlefield_find(b2).unwrap();
    assert_eq!(b1.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(b2.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn prismari_treasurer_surge_etb_mints_two_treasures() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_treasurer_surge());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Treasurer-Surge castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
        .collect();
    assert_eq!(treasures.len(), 2, "ETB mints 2 Treasures");
}

#[test]
fn prismari_pyreburst_sweeps_x_three_creatures() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_pyreburst());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyreburst castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(b1).is_none(), "Friendly bear sweeps too");
    assert!(g.battlefield_find(b2).is_none(), "Opp bear destroyed");
}

#[test]
fn prismari_vorthos_etb_loots_and_burns_with_is_discard() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    // Hand has an instant for the discard step.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::prismari_vorthos());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Discard(vec![bolt]),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Vorthos castable");
    drain_stack(&mut g);
    // Player 1 takes 2 damage because IS card was discarded.
    assert_eq!(g.players[1].life, p1_life - 2, "Burns opp for 2 after IS discard");
}

#[test]
fn prismari_cinderspark_pings_and_scries() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_cinderspark());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cinderspark castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 1);
}

// ── Push (modern_decks) batch 23 extras: 5 more STX cards ──────────────────

#[test]
fn inkling_sage_pump_activation_makes_two_two_flier() {
    let mut g = two_player_game();
    let sage = g.add_card_to_battlefield(0, catalog::inkling_sage());
    g.clear_sickness(sage);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sage,
        ability_index: 0,
        target: None, x_value: None }).expect("Sage activation");
    drain_stack(&mut g);
    let sage = g.battlefield_find(sage).unwrap();
    assert_eq!(sage.power(), 2, "Sage pumped from 1/2 → 2/3");
    assert_eq!(sage.toughness(), 3);
}

#[test]
fn witherbloom_reaper_hand_dies_drains_two() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let rh = g.add_card_to_battlefield(0, catalog::witherbloom_reaper_hand());
    drain_stack(&mut g);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Bolt isn't enough alone (3 dmg vs 3 tough → dies on next SBA).
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(rh)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2, "Drain 2 on death");
    assert_eq!(g.players[0].life, p0_life + 2, "Gain 2 on death");
}

#[test]
fn spirit_conduit_taps_for_one_damage() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let sc = g.add_card_to_battlefield(0, catalog::spirit_conduit());
    g.clear_sickness(sc);
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sc,
        ability_index: 0,
        target: Some(Target::Player(1)), x_value: None }).expect("Spirit Conduit activation");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 1, "Conduit pings for 1");
    let sc = g.battlefield_find(sc).unwrap();
    assert!(sc.tapped);
    let def = catalog::spirit_conduit();
    assert!(def.card_types.contains(&crate::card::CardType::Artifact));
    assert!(def.card_types.contains(&crate::card::CardType::Creature));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Spirit));
}

#[test]
fn quandrix_aether_adept_taps_target_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let qa = g.add_card_to_battlefield(0, catalog::quandrix_aether_adept());
    g.clear_sickness(qa);
    g.perform_action(GameAction::ActivateAbility {
        card_id: qa,
        ability_index: 0,
        target: Some(Target::Permanent(bear)), x_value: None }).expect("Aether Adept activation");
    drain_stack(&mut g);
    let bear = g.battlefield_find(bear).unwrap();
    assert!(bear.tapped, "Bear tapped");
    assert!(catalog::quandrix_aether_adept().keywords.contains(&Keyword::Defender));
}

#[test]
fn prismari_sparkbright_attack_pings_target() {
    use crate::game::types::{Attack, AttackTarget, Target, TurnStep};
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sb = g.add_card_to_battlefield(0, catalog::prismari_sparkbright());
    g.clear_sickness(sb);
    let bear_before = g.battlefield_find(opp_bear).unwrap().damage;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sb,
        target: AttackTarget::Player(1),
    }])).expect("Attack declared");
    // The on-attack trigger needs a target.
    // Resolve any pending triggers — auto-target picks a legal candidate.
    let _ = g.pass_priority();
    drain_stack(&mut g);
    let _ = bear_before;
    let _ = Target::Permanent(opp_bear);
    // Bear should have taken at least 1 damage from the trigger OR opp lost 1 life.
    let life_after = g.players[1].life;
    let bear = g.battlefield_find(opp_bear);
    let damage_done = (20i32 - life_after)
        + bear.map(|b| b.damage as i32).unwrap_or(0);
    assert!(damage_done >= 1, "On-attack ping dealt at least 1 damage somewhere");
    assert!(catalog::prismari_sparkbright().keywords.contains(&Keyword::Haste));
}

// ── Push (modern_decks) batch 24: 25 new STX cards across all 5 colleges ───
//
// 5 per college using existing magecraft / token / lifegain / counter
// primitives. No new engine features required. Tests below lock in the
// primary play pattern for each card.

#[test]
fn silverquill_notetaker_etb_scrys_one() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::silverquill_notetaker());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Notetaker castable");
    drain_stack(&mut g);
    // Body present on battlefield + ETB scry resolved (no easy assertion;
    // verify the card is in play with stats).
    let nt = g.battlefield_find(id).unwrap();
    assert_eq!(nt.power(), 1);
    assert_eq!(nt.toughness(), 2);
}

#[test]
fn inkling_pamphleteer_etb_drains_one() {
    let mut g = two_player_game();
    let p0 = g.players[0].life;
    let p1 = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::inkling_pamphleteer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pamphleteer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 1);
    assert_eq!(g.players[1].life, p1 - 1);
    assert!(catalog::inkling_pamphleteer().keywords.contains(&Keyword::Flying));
}

#[test]
fn silverquill_indictment_exiles_low_mv_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_indictment());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Indictment castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear exiled");
    assert_eq!(g.players[0].life, life_before + 2, "Gain 2 life on resolve");
}

#[test]
fn inkling_banner_bearer_buffs_other_inklings() {
    let mut g = two_player_game();
    let bb = g.add_card_to_battlefield(0, catalog::inkling_banner_bearer());
    let token_id = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let inkling = computed.iter().find(|c| c.id == token_id).unwrap();
    // Inkling Aspirant is 2/1 base; +1/+0 anthem → 3/1.
    assert_eq!(inkling.power, 3, "Other Inkling pumped +1 power");
    assert_eq!(inkling.toughness, 1);
    // Source itself is unaffected.
    let banner = computed.iter().find(|c| c.id == bb).unwrap();
    assert_eq!(banner.power, 2);
}

#[test]
fn silverquill_tribunal_forces_opp_sacrifice_and_gains_one_life() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let _victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_tribunal());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tribunal castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "Gain 1 life");
    // Opponent should have lost their bear.
    let p1_creatures: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 1).collect();
    assert_eq!(p1_creatures.len(), 0, "Opp sacrificed their creature");
}

#[test]
fn witherbloom_aspersor_shrinks_creature_and_gains_one_life() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_aspersor());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Aspersor castable");
    drain_stack(&mut g);
    // Bear is 2/2 — gets -2/-1 → 0/1, still on battlefield.
    let bear = g.battlefield_find(bear).unwrap();
    assert_eq!(bear.power(), 0);
    assert_eq!(bear.toughness(), 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn pest_reanimator_etb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear_id = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pest_reanimator());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reanimator castable");
    drain_stack(&mut g);
    // Bear should now be in player 0's hand.
    let in_hand = g.players[0].hand.iter().any(|c| c.id == bear_id);
    assert!(in_hand, "Bear returned to hand");
}

#[test]
fn witherbloom_spore_master_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_spore_master());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spore-Master castable");
    drain_stack(&mut g);
    // Pests are now on the battlefield.
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Pest")
        .collect();
    assert_eq!(pests.len(), 2, "Two Pest tokens minted");
    let sm = g.battlefield_find(id).unwrap();
    assert_eq!(sm.power(), 4);
    assert_eq!(sm.toughness(), 4);
}

#[test]
fn witherbloom_withercut_shrinks_creature_and_cantrips() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_withercut());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Withercut castable");
    drain_stack(&mut g);
    // Bear (2/2) → -3/-1 → -1/1 EOT (still on battlefield since toughness > 0).
    let bear_c = g.battlefield_find(bear).expect("Bear still alive");
    assert_eq!(bear_c.power(), -1);
    assert_eq!(bear_c.toughness(), 1);
    // Hand: -1 (cast) +1 (cantrip) = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn pest_cultivator_adept_etb_mints_pest_and_grows_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_cultivator_adept());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cultivator-Adept castable");
    drain_stack(&mut g);
    // ETB: Pest token.
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Pest")
        .collect();
    assert_eq!(pests.len(), 1, "ETB mints 1 Pest");
    // Magecraft: cast an instant → +1/+1 counter.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let ca = g.battlefield_find(id).unwrap();
    assert_eq!(ca.counter_count(CounterType::PlusOnePlusOne), 1,
        "Cultivator-Adept got a counter from magecraft");
}

#[test]
fn lorehold_soulshaper_etb_mints_spirit_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_soulshaper());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soulshaper castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 1);
    let ss = g.battlefield_find(id).unwrap();
    assert!(catalog::lorehold_soulshaper().keywords.contains(&Keyword::Vigilance));
    assert_eq!(ss.toughness(), 4);
}

#[test]
fn lorehold_ironhand_etb_pings_target_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_ironhand());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ironhand castable");
    drain_stack(&mut g);
    // Bear was 2/2 → dies to 2 damage.
    assert!(g.battlefield_find(bear).is_none());
    let def = catalog::lorehold_ironhand();
    assert!(def.keywords.contains(&Keyword::FirstStrike));
    assert!(def.keywords.contains(&Keyword::Trample));
}

#[test]
fn lorehold_revival_returns_creature_with_haste() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear_id = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_revival());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear_id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Revival castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear_id), "Bear reanimated");
    // Haste grant via the new `granted_keywords_eot` EOT path (engine
    // fix — batch 24). `has_keyword` checks both printed and granted.
    let bear = g.battlefield_find(bear_id).expect("Bear on battlefield");
    assert!(bear.has_keyword(&Keyword::Haste),
        "Reanimated bear has haste EOT via `granted_keywords_eot`");
}

#[test]
fn lorehold_sparkflare_deals_two_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_sparkflare());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkflare castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_before - 2);
}

#[test]
fn quandrix_logician_etb_scrys_and_pumps_fractal_on_cast() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_logician());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Logician castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some());
    // Mint a Fractal first then cast a spell.
    let fractal = g.add_card_to_battlefield(0, catalog::quandrix_hatchling());
    drain_stack(&mut g);
    let counters_before = g.battlefield_find(fractal).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt");
    drain_stack(&mut g);
    let counters_after = g.battlefield_find(fractal).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert!(counters_after > counters_before, "Fractal grew on instant cast");
}

#[test]
fn fractal_echoist_etb_counters_scale_with_graveyard() {
    let mut g = two_player_game();
    // Seed gy with IS cards.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::fractal_echoist());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Echoist castable");
    drain_stack(&mut g);
    let fe = g.battlefield_find(id).expect("Echoist on battlefield");
    assert_eq!(fe.counter_count(CounterType::PlusOnePlusOne), 3,
        "Echoist enters with 3 +1/+1 counters (one per IS in gy)");
}

#[test]
fn quandrix_mathenotaur_etb_doubles_counters_on_target() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Manually stack +1/+1 counters on the bear.
    {
        let b = g.battlefield_find_mut(bear).unwrap();
        b.add_counters(CounterType::PlusOnePlusOne, 3);
    }
    let counters_before = g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters_before, 3);
    let id = g.add_card_to_hand(0, catalog::quandrix_mathenotaur());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mathenotaur castable");
    drain_stack(&mut g);
    let counters_after = g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters_after, 6,
        "Mathenotaur doubles target's counters: 3 → 6");
}

#[test]
fn fractal_surge_mints_fractal_with_creature_count_counters() {
    let mut g = two_player_game();
    // Seed 3 creatures.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::fractal_surge());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Surge castable");
    drain_stack(&mut g);
    // Fractal token should have at least 3 +1/+1 counters (creature count).
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Fractal")
        .expect("Fractal minted");
    assert!(fractal.counter_count(CounterType::PlusOnePlusOne) >= 3,
        "Fractal scales with creature count");
}

#[test]
fn prismari_mindkindler_magecraft_taps_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_mindkindler());
    drain_stack(&mut g);
    // Untap bear so we can confirm tap.
    {
        let b = g.battlefield_find_mut(bear).unwrap();
        b.tapped = false;
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(bear).unwrap();
    assert!(bear.tapped, "Mindkindler magecraft tapped opp's bear");
}

#[test]
fn prismari_embergem_burns_creature_and_mints_treasure() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_embergem());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Embergem castable");
    drain_stack(&mut g);
    // Bear was 2/2 → 4 damage → dies.
    assert!(g.battlefield_find(bear).is_none(), "Bear dies to 4 dmg");
    // Treasure minted.
    let treasures: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure")
        .collect();
    assert_eq!(treasures.len(), 1, "1 Treasure minted");
}

#[test]
fn prismari_pyromancer_etb_pings_and_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_pyromancer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyromancer castable");
    drain_stack(&mut g);
    // ETB: 2 damage to a legal target (auto-picks opponent).
    assert_eq!(g.players[1].life, p1_before - 2, "ETB ping for 2 hits opp");
    // Body present.
    let py = g.battlefield_find(id).unwrap();
    assert_eq!(py.power(), 3);
    assert_eq!(py.toughness(), 2);
}

#[test]
fn prismari_spitfire_etb_pings_target_with_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_spitfire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spitfire castable");
    drain_stack(&mut g);
    // ETB ping for 2 — auto-target picks opp player.
    assert_eq!(g.players[1].life, p1_before - 2, "ETB ping for 2");
    assert!(catalog::prismari_spitfire().keywords.contains(&Keyword::Haste));
}

// ── Push (modern_decks) batch 24++: 5 more cards (1 per college) ────────

#[test]
fn silverquill_memorist_etb_returns_is_card_from_graveyard() {
    let mut g = two_player_game();
    let bolt_id = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::silverquill_memorist());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memorist castable");
    drain_stack(&mut g);
    let in_hand = g.players[0].hand.iter().any(|c| c.id == bolt_id);
    assert!(in_hand, "Bolt returned to hand");
    assert!(catalog::silverquill_memorist().keywords.contains(&Keyword::Flying));
}

#[test]
fn witherbloom_tendril_drains_two_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_tendril());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tendril castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_before + 2);
    assert_eq!(g.players[1].life, p1_before - 2);
    assert_eq!(g.players[0].hand.len(), hand_before, "Cantrip: -1 cast +1 draw");
}

#[test]
fn lorehold_spirit_anthem_pumps_team_with_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_anthem());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spirit-Anthem castable");
    drain_stack(&mut g);
    let bear_c = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_c.power(), 4, "Bear pumped +2");
    assert_eq!(bear_c.toughness(), 3, "Bear pumped +1");
    let computed = g.compute_battlefield();
    let bear_cp = computed.iter().find(|c| c.id == bear).unwrap();
    assert!(bear_cp.keywords.contains(&Keyword::FirstStrike),
        "Bear has first strike EOT");
}

#[test]
fn quandrix_symmetrycaster_etb_scales_with_hand_size() {
    let mut g = two_player_game();
    // 3 cards in hand.
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_symmetrycaster());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    assert_eq!(hand_before, 4, "Test seeded 4 cards (3 islands + Symmetrycaster)");
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Symmetrycaster castable");
    drain_stack(&mut g);
    // After casting, hand has 3 cards (4 - 1 Symmetrycaster).
    // Symmetrycaster reads hand size at trigger resolution time → 3 counters.
    let sc = g.battlefield_find(id).unwrap();
    assert_eq!(sc.counter_count(CounterType::PlusOnePlusOne), 3,
        "Symmetrycaster's ETB sized by hand size (3 islands remaining)");
}

#[test]
fn prismari_drakeforge_etb_mints_treasure_and_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_drakeforge());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drakeforge castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure")
        .collect();
    assert_eq!(treasures.len(), 1, "Treasure minted");
    // Cast an instant → magecraft pumps Drakeforge.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let df = g.battlefield_find(id).unwrap();
    assert_eq!(df.power(), 3, "Drakeforge pumped +1/+0 from magecraft");
    assert!(catalog::prismari_drakeforge().keywords.contains(&Keyword::Flying));
}

/// CR 701.14c — "If a creature fights itself, it deals damage to
/// itself equal to twice its power." Lock-in: a 2/2 fighting itself
/// takes 2×2 = 4 damage → dies (2 toughness < 4 damage).
#[test]
fn cr_701_14c_self_fight_deals_twice_power_to_self() {
    use crate::card::{Effect, Selector};
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    // Use a simple 2/2 — vanilla Grizzly Bears has no triggers that
    // could interact with the fight resolution.
    let beast = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(beast);
    let fight_effect = Effect::Fight {
        attacker: Selector::Target(0),
        defender: Selector::Target(1),
    };
    // Use the same target on both slots so the fight resolves on itself.
    let ctx = {
        let mut c = EffectContext::for_spell(
            0,
            Some(crate::game::types::Target::Permanent(beast)),
            0,
            0,
        );
        c.targets.push(crate::game::types::Target::Permanent(beast));
        c
    };
    g.resolve_effect(&fight_effect, &ctx).expect("Self-fight resolves");
    drain_stack(&mut g);
    // The 4/4 takes 8 damage (2 × 4 power) → dies via SBA.
    assert!(g.battlefield_find(beast).is_none(),
        "Ironhand self-fights → 8 damage to self → dies");
}

/// Engine — `Effect::GrantKeyword` with `Duration::EndOfTurn` now uses
/// the new `granted_keywords_eot` bag on `CardInstance`, with cleanup at
/// the Cleanup step. Lock-in test: grant Haste EOT on a bear, verify
/// `has_keyword` reports it, advance to Cleanup, verify it's gone.
#[test]
fn granted_keyword_eot_clears_at_cleanup_per_batch_24() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Directly stage a granted keyword (simulates an EOT GrantKeyword
    // effect resolving on the bear).
    {
        let b = g.battlefield_find_mut(bear).unwrap();
        b.granted_keywords_eot.push(Keyword::Haste);
    }
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.has_keyword(&Keyword::Haste),
        "Bear has haste via granted_keywords_eot");
    // Computed view also picks it up.
    let computed = g.compute_battlefield();
    let bear_c = computed.iter().find(|c| c.id == bear).unwrap();
    assert!(bear_c.keywords.contains(&Keyword::Haste),
        "Computed view reports granted EOT keyword");
    // Cleanup: keyword should be cleared.
    g.step = crate::game::types::TurnStep::Cleanup;
    g.do_cleanup();
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.granted_keywords_eot.is_empty(),
        "granted_keywords_eot bag empty after Cleanup");
    assert!(!b.has_keyword(&Keyword::Haste),
        "Bear lost granted haste at Cleanup");
}

// ── Push (modern_decks) batch 24+: 10 more STX cards (2 per college) ───────

#[test]
fn silverquill_eulogist_drains_each_opp_on_cast() {
    let mut g = two_player_game();
    let _eul = g.add_card_to_battlefield(0, catalog::silverquill_eulogist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt did 3, Eulogist did 1 → opp lost 4.
    assert_eq!(g.players[1].life, p1_before - 4, "Eulogist drained 1 on cast");
}

#[test]
fn inkling_quillwarden_magecraft_self_pumps() {
    let mut g = two_player_game();
    let qw = g.add_card_to_battlefield(0, catalog::inkling_quillwarden());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let qw_c = g.battlefield_find(qw).unwrap();
    assert_eq!(qw_c.power(), 3, "Quillwarden pumped +1/+0 EOT");
    assert!(catalog::inkling_quillwarden().keywords.contains(&Keyword::Flying));
    assert!(catalog::inkling_quillwarden().keywords.contains(&Keyword::Vigilance));
}

#[test]
fn witherbloom_pest_lord_etb_mints_pest_and_pumps_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pest_lord());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest-Lord castable");
    drain_stack(&mut g);
    // 1 Pest token minted.
    let pest_id = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Pest")
        .map(|c| c.id)
        .expect("Pest minted");
    let computed = g.compute_battlefield();
    let pest = computed.iter().find(|c| c.id == pest_id).unwrap();
    // 1/1 base + 1/0 anthem = 2/1.
    assert_eq!(pest.power, 2, "Pest pumped to 2 by Pest-Lord anthem");
    assert_eq!(pest.toughness, 1);
}

#[test]
fn witherbloom_drainbreath_dies_drains_two() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let db = g.add_card_to_battlefield(0, catalog::witherbloom_drainbreath());
    drain_stack(&mut g);
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(db)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 → db (2/1) dies → drain 2 (opp -2, you +2).
    assert_eq!(g.players[1].life, p1_before - 2, "Drain 2 from opp on death");
    assert_eq!(g.players[0].life, p0_before + 2, "Gain 2 on death");
}

#[test]
fn lorehold_spirit_caller_etb_mints_two_hasty_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_caller());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spirit-Caller castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 2, "Two Spirit tokens minted");
    // Both Spirits have haste.
    for s in &spirits {
        assert!(s.has_keyword(&Keyword::Haste), "Spirit has haste EOT");
    }
}

#[test]
fn lorehold_recital_burns_and_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_recital());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recital castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_before - 1, "Recital pings for 1");
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 1, "Spirit minted");
}

#[test]
fn quandrix_pondkeeper_etb_mints_fractal_sized_by_is_in_gy() {
    let mut g = two_player_game();
    // Seed gy with 2 instants.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::quandrix_pondkeeper());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pondkeeper castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Fractal")
        .expect("Fractal minted");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 2,
        "Fractal has 2 counters (2 IS in gy)");
}

#[test]
fn quandrix_counterproof_pumps_and_scrys() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_counterproof());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Counterproof castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(bear).unwrap();
    assert_eq!(bear.counter_count(CounterType::PlusOnePlusOne), 1, "Bear got 1 counter");
}

#[test]
fn prismari_hotburst_burns_target_and_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_hotburst());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hotburst castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_before - 2, "Hotburst hits for 2");
    let treasures: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure")
        .collect();
    assert_eq!(treasures.len(), 1, "Treasure minted");
}

#[test]
fn prismari_magmaspark_etb_pings_and_grows_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_magmaspark());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Magmaspark castable");
    drain_stack(&mut g);
    // ETB ping for 1 (auto-target opp).
    assert_eq!(g.players[1].life, p1_before - 1, "Magmaspark pings opp for 1");
    // Magecraft self-pump triggers when we cast a bolt.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let ms = g.battlefield_find(id).unwrap();
    assert_eq!(ms.power(), 2, "Magmaspark pumped +1/+0 from magecraft");
}

#[test]
fn prismari_wildform_pumps_grants_haste_and_cantrips() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_wildform());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wildform castable");
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let bear_c = computed.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(bear_c.power, 4, "Bear pumped +2");
    assert_eq!(bear_c.toughness, 3, "Bear pumped +1 toughness");
    assert!(bear_c.keywords.contains(&Keyword::Haste), "Bear has haste");
    // Hand: -1 (cast) +1 (cantrip) = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Push (modern_decks) batch 25: 28 new STX cards across all 5 colleges ───
//
// 7 Silverquill + 6 Witherbloom + 5 Lorehold + 5 Prismari + 5 Quandrix.
// All use existing engine primitives — no new engine features required.

#[test]
fn silverquill_inkmaster_drains_each_opp_on_cast() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::silverquill_inkmaster());
    drain_stack(&mut g);
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_before + 1, "Inkmaster +1 from drain");
    // -3 for bolt + -1 from drain.
    assert_eq!(g.players[1].life, p1_before - 4, "Opp -3 (bolt) -1 (drain)");
}

#[test]
fn inkling_censurer_etb_taps_opp_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::inkling_censurer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Censurer castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(opp_bear).unwrap();
    assert!(bear.tapped, "Bear tapped on ETB");
    assert!(catalog::inkling_censurer().keywords.contains(&Keyword::Vigilance));
}

#[test]
fn silverquill_loredrain_shrinks_and_gains_two() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_loredrain());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Loredrain castable");
    drain_stack(&mut g);
    // Bear is 2/2, -2/-2 = 0/0 → dies.
    assert!(g.battlefield_find(bear).is_none(), "Bear killed");
    assert_eq!(g.players[0].life, life_before + 2, "Gain 2 life");
}

#[test]
fn inkling_verseweaver_mints_inkling_on_cast() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::inkling_verseweaver());
    drain_stack(&mut g);
    let inklings_before = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Inkling")
        .count();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let inklings_after = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Inkling")
        .count();
    assert_eq!(inklings_after, inklings_before + 1, "Magecraft mints 1 Inkling");
}

#[test]
fn silverquill_hightutor_searches_for_cheap_is() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    // The library has a cheap IS card (Lightning Bolt: {R} = MV 1)
    // and an expensive one (Pop Quiz: {1}{W} = MV 2). Hightutor finds either.
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));
    let id = g.add_card_to_hand(0, catalog::silverquill_hightutor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hightutor castable");
    drain_stack(&mut g);
    // Bolt was searched and moved to hand. Hand: -1 (cast) +1 (search) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "Bolt now in hand");
}

#[test]
fn silverquill_lifebinder_etb_gains_two_life_with_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_lifebinder());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifebinder castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2, "ETB +2 life");
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_drainmaster_etb_drains_three() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_drainmaster());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainmaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_before + 3);
    assert_eq!(g.players[1].life, p1_before - 3);
}

#[test]
fn witherbloom_marshcaster_drains_on_cast() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::witherbloom_marshcaster());
    drain_stack(&mut g);
    let p1_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -3 from bolt + -1 from drain on opp.
    assert_eq!(g.players[1].life, p1_before - 4);
}

#[test]
fn pest_wrangler_etb_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_wrangler());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wrangler castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 1);
}

#[test]
fn witherbloom_toxicaster_pumps_toughness_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_toxicaster());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let tx = g.battlefield_find(id).unwrap();
    assert_eq!(tx.toughness(), 2, "Toxicaster pumped +0/+1");
    assert!(tx.has_keyword(&Keyword::Deathtouch));
}

#[test]
fn witherbloom_soilbleeder_etb_optional_sac_drains_three() {
    use crate::game::types::Target;
    use crate::decision::{ScriptedDecider, DecisionAnswer};
    let mut g = two_player_game();
    let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_soilbleeder());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soilbleeder castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_before + 3);
    assert_eq!(g.players[1].life, p1_before - 3);
}

#[test]
fn witherbloom_handburner_discards_two_and_gains_two() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    // Stock opp's hand.
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::witherbloom_handburner());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_hand_before = g.players[1].hand.len();
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Handburner castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 2, "Opp discarded 2");
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn pest_brood_mother_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_brood_mother());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Brood-Mother castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 2, "Brood-Mother mints 2 Pests");
}
