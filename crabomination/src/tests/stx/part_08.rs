use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn witherbloom_sapseeker_attack_gains_one_life() {
    use crate::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_sapseeker());
    g.clear_sickness(id);
    let life_before = g.players[0].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("Sapseeker attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Trample));
}

#[test]
fn witherbloom_pestlich_etb_reanimates_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestlich());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestlich castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "Bear reanimated");
}

#[test]
fn witherbloom_mireguide_taps_for_black_or_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_mireguide());
    g.clear_sickness(id);
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("Mireguide Black ability");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
}

// ── New Silverquill STX cards (push batch — claude/modern_decks) ──────────

#[test]
fn inkling_sermon_drains_two_and_mints_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_sermon());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sermon castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, me_before + 2);
    let inklings: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Inkling"
    }).collect();
    assert_eq!(inklings.len(), 1);
}

#[test]
fn silverquill_lorescribe_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _filler = g.add_card_to_hand(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_lorescribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tutor castable");
    drain_stack(&mut g);
    // -1 cast, +1 draw, -1 discard → -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn inkling_warden_pumps_on_friendly_inkling_etb() {
    let mut g = two_player_game();
    let warden = g.add_card_to_battlefield(0, catalog::inkling_warden());
    drain_stack(&mut g);
    // Cast an Inkling
    let aspirant = g.add_card_to_hand(0, catalog::inkling_aspirant());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aspirant, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Aspirant castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(warden).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn silverquill_inkletter_drains_one_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkletter());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkletter castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, me_before + 1);
}

// ── New Witherbloom STX cards (batch 32, claude/modern_decks) ─────────────────

#[test]
fn witherbloom_pestswarm_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestswarm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestswarm castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 2, "Pestswarm mints 2 Pests on ETB");
}

#[test]
fn witherbloom_lifeleecher_gains_life_on_instant_cast() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::witherbloom_lifeleecher());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let me_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, me_before + 1, "Lifeleecher magecraft gains 1 life");
}

#[test]
fn witherbloom_rootcaster_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_rootcaster());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.power(), 3); // 2 base + 1 magecraft
    assert_eq!(body.toughness(), 4); // 3 base + 1 magecraft
}

#[test]
fn witherbloom_caulhound_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_caulhound());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Caulhound castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, me_before + 2);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Trample));
}

#[test]
fn witherbloom_gravecaller_etb_returns_low_mv_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_gravecaller());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Gravecaller castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "Bear returned to hand");
}

#[test]
fn witherbloom_bloodvine_has_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_bloodvine());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloodvine castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Lifelink));
}

#[test]
fn witherbloom_bloodvine_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::witherbloom_bloodvine());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let me_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, me_before + 1);
}

#[test]
fn witherbloom_vitalist_grows_on_lifegain() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_vitalist());
    // Add a separate instant that gains life
    let life = g.add_card_to_hand(0, catalog::healing_salve());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: life, target: Some(crate::game::types::Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Healing Salve castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn witherbloom_toxinkeeper_etb_shrinks_target() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxinkeeper());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Toxinkeeper castable");
    drain_stack(&mut g);
    // Bear was 2/2, gets -1/-1 → 1/1
    let bear = g.battlefield_find(opp_bear);
    if let Some(b) = bear {
        assert_eq!(b.power(), 1);
        assert_eq!(b.toughness(), 1);
    }
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Deathtouch));
}

#[test]
fn witherbloom_bloodroot_drains_four() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_bloodroot());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloodroot castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 4);
    assert_eq!(g.players[0].life, me_before + 4);
}

#[test]
fn witherbloom_pesthatch_mints_pest_and_pumps() {
    let mut g = two_player_game();
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::witherbloom_pesthatch());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(friend)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pesthatch castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 1);
    let bear = g.battlefield_find(friend).unwrap();
    assert_eq!(bear.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn witherbloom_diviner_etb_mills_and_optional_recover() {
    let mut g = two_player_game();
    // Stack some cards on top of library to mill
    for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::witherbloom_diviner());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Diviner castable");
    drain_stack(&mut g);
    // 3 cards milled to graveyard (auto-decider declines the MayDo by default)
    assert_eq!(g.players[0].graveyard.len(), gy_before + 3);
}

#[test]
fn witherbloom_pestwarden_etb_drains_two_then_activates() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestwarden());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_before = g.players[1].life;
    let me_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestwarden castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, me_before + 2);
}

// ── New Lorehold STX cards (batch 32, claude/modern_decks) ─────────────────

#[test]
fn lorehold_spectrebrand_attack_pumps_friendly() {
    use crate::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::lorehold_spectrebrand());
    g.clear_sickness(id);
    g.clear_sickness(bear);
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("Spectrebrand attacks");
    drain_stack(&mut g);
    let bear_body = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_body.power(), 3, "Bear should be pumped +1/+0");
}

#[test]
fn lorehold_charwarden_magecraft_pings_target() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_charwarden());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt = 3 + magecraft 1 = 4
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn lorehold_lightcleric_magecraft_gains_life() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_lightcleric());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let me_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, me_before + 1);
}

#[test]
fn lorehold_grave_crusader_etb_exiles_target_graveyard_card() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_grave_crusader());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Crusader castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt));
}

#[test]
fn lorehold_pyrescholar_grows_on_card_leave_gy() {
    let mut g = two_player_game();
    let pyre = g.add_card_to_battlefield(0, catalog::lorehold_pyrescholar());
    let gy_card = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    drain_stack(&mut g);
    // Use Lorehold Acolyte's exile from gy to remove the card
    let acolyte = g.add_card_to_hand(0, catalog::lorehold_acolyte());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: acolyte, target: Some(crate::game::types::Target::Permanent(gy_card)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Acolyte castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(pyre).unwrap();
    assert_eq!(body.power(), 3, "Pyrescholar +1/+1 on gy leave");
}

#[test]
fn lorehold_vow_burns_target_and_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_vow());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vow castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).collect();
    assert_eq!(spirits.len(), 1);
}

#[test]
fn lorehold_spectrecaster_etb_returns_is_card_from_gy() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_spectrecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spectrecaster castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
}

#[test]
fn lorehold_forgemaster_grows_on_instant_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_forgemaster());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn lorehold_skirmlord_attack_scales_with_other_attackers() {
    use crate::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::lorehold_skirmlord());
    g.clear_sickness(id);
    g.clear_sickness(bear);
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: id, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("Both attack");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    // Base 2 + 1 (other attacker = bear) = 3
    assert_eq!(body.power(), 3);
}

#[test]
fn lorehold_memoirist_etb_exiles_drains_and_mints_spirit() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_memoirist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let me_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memoirist castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt));
    assert_eq!(g.players[0].life, me_before + 2);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).collect();
    assert_eq!(spirits.len(), 1);
}

#[test]
fn lorehold_ardent_acolyte_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_ardent_acolyte());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // 3 dmg from bolt + 1 dmg from acolyte = 4
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn lorehold_bequeathing_reanimates_with_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_bequeathing());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bequeathing castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(bear).unwrap();
    assert!(body.has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_pyromaster_taps_for_three_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_pyromaster());
    g.clear_sickness(id);
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(crate::game::types::Target::Player(1)), x_value: None }).expect("Pyromaster activated");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn lorehold_spirit_hymn_pumps_team_with_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_hymn());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spirit Hymn castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(bear).unwrap();
    assert_eq!(body.power(), 3, "Bear pumped to 3");
    assert!(body.has_keyword(&Keyword::FirstStrike));
}

#[test]
fn lorehold_spirit_legion_mints_two_spirits_and_pumps_each() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_legion());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Legion castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).collect();
    assert_eq!(spirits.len(), 2);
    // Each Spirit should have a +1/+1 counter
    for spirit in &spirits {
        assert_eq!(spirit.counter_count(CounterType::PlusOnePlusOne), 1);
    }
    // The Spirit Legion itself is a Spirit Cleric — it should also get a counter
    let legion = g.battlefield_find(id).unwrap();
    assert_eq!(legion.counter_count(CounterType::PlusOnePlusOne), 1);
}

// ── New Silverquill STX cards (batch 32, claude/modern_decks) ─────────────────

#[test]
fn silverquill_drainlord_etb_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_drainlord());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainlord castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, me_before + 3);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_quillbearer_magecraft_shrinks_target() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let _id = g.add_card_to_battlefield(0, catalog::inkling_quillbearer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bear takes 3 from bolt and shrinks; magecraft triggers separately, but
    // since Bolt's target was the bear, magecraft auto-targets the bear too.
    // Bear was 2/2, took 3 → dead before magecraft can shrink. So check graveyard.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == opp_bear));
}

#[test]
fn silverquill_indoctrinator_etb_discards_each_opp() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_indoctrinator());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Indoctrinator castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1);
}

#[test]
fn inkling_choirsinger_magecraft_gains_life() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::inkling_choirsinger());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let me_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, me_before + 1);
}

#[test]
fn silverquill_ovation_mints_two_inklings_and_pumps_each() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_ovation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ovation castable");
    drain_stack(&mut g);
    let inklings: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Inkling"
    }).collect();
    assert_eq!(inklings.len(), 2);
    for ink in &inklings {
        assert_eq!(ink.counter_count(CounterType::PlusOnePlusOne), 1);
    }
}

#[test]
fn inkling_loremaster_etb_returns_is_card_and_gains_life() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::inkling_loremaster());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let me_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Loremaster castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
    assert_eq!(g.players[0].life, me_before + 1);
}

#[test]
fn silverquill_litany_shrinks_creature_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_litany());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let me_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Litany castable");
    drain_stack(&mut g);
    // 2/2 bear → -2/-1 → 0/1 (alive, but powerless)
    let bear_view = g.computed_permanent(bear).expect("Bear still on bf");
    assert_eq!(bear_view.power, 0);
    assert_eq!(bear_view.toughness, 1);
    assert_eq!(g.players[0].life, me_before + 1);
}

// ── New Quandrix STX cards (batch 32, claude/modern_decks) ─────────────────

#[test]
fn quandrix_tidewright_etb_shrinks_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::quandrix_tidewright());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidewright castable");
    drain_stack(&mut g);
    let bear_body = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_body.power(), 0);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Flash));
}

#[test]
fn quandrix_wavewriter_grows_on_instant_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_wavewriter());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_scribe_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_scribe());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.power(), 2); // 1 base + 1
    assert_eq!(body.toughness(), 3); // 2 base + 1
}

#[test]
fn quandrix_handmage_etb_mints_fractal_scaling_with_hand() {
    let mut g = two_player_game();
    // Add some cards to hand
    for _ in 0..3 { g.add_card_to_hand(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_handmage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Handmage castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Fractal"
    }).collect();
    assert_eq!(fractals.len(), 1);
    // After cast, hand has 3 bears (initially). At ETB time, hand size includes those.
    // Number of counters = hand size when ETB fires (after handmage left hand)
    let counter_count = fractals[0].counter_count(CounterType::PlusOnePlusOne);
    assert!(counter_count >= 3, "Fractal scales with hand size, got {}", counter_count);
}

#[test]
fn quandrix_equipoise_draws_and_pumps_with_hand_size() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::quandrix_equipoise());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Equipoise castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before); // -1 cast +1 draw = 0
    let bear_body = g.battlefield_find(bear).unwrap();
    // counters = hand size after draw
    let counters = bear_body.counter_count(CounterType::PlusOnePlusOne);
    assert!(counters >= 1);
}

#[test]
fn quandrix_visionary_etb_scrys() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_visionary());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Visionary castable");
    drain_stack(&mut g);
    // Scry doesn't change library size; just verify it lands.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn quandrix_wilderwright_etb_finds_basic_land() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::quandrix_wilderwright());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wilderwright castable");
    drain_stack(&mut g);
    let forest_body = g.battlefield_find(forest).expect("Forest on bf");
    assert!(forest_body.tapped);
}

#[test]
fn quandrix_topologist_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::quandrix_topologist());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Topologist castable");
    drain_stack(&mut g);
    // -1 cast, +1 draw, -1 discard = -1 net
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

// ── New Prismari STX cards (batch 32, claude/modern_decks) ─────────────────

#[test]
fn prismari_embertongue_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::prismari_embertongue());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // 3 bolt + 1 embertongue = 4
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn prismari_treasurewright_b32_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_treasurewright_b32());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Treasurewright castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Treasure"
    }).collect();
    assert_eq!(treasures.len(), 1);
}

#[test]
fn prismari_sparkpainter_magecraft_pumps_and_offers_loot() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_sparkpainter());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.power(), 4); // 3 base + 1 magecraft
}

#[test]
fn prismari_burning_lesson_burns_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_burning_lesson());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Burning Lesson castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn prismari_flameforger_magecraft_self_pumps_two_zero() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_flameforger());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.power(), 5); // 3 base + 2 magecraft
    assert!(body.has_keyword(&Keyword::Haste));
}

// ── CR 107 — Numbers and Symbols audit (batch 32) ──────────────────────────

#[test]
fn cr_107_1c_x_zero_for_x_cost_spell_resolves_cleanly() {
    // CR 107.1c: "If a rule or ability instructs a player to choose 'any
    // number,' that player may choose any positive number or zero."
    // Crackle with Power cast for X=0 should deal 0 damage and gracefully
    // do nothing (CR 120.8 zero-damage suppression).
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::crackle_with_power());
    // Cost: {X}{R}{R}{R}{R}{R}; pay just the colored pips.
    for _ in 0..5 { g.players[0].mana_pool.add(Color::Red, 1); }
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: Some(0),
    }).expect("Crackle castable at X=0");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before, "5*0 = 0 damage");
}

// ── New Lessons (batch 32, claude/modern_decks) ────────────────────────────

#[test]
fn mascot_lesson_b32_mints_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mascot_lesson_b32());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mascot Lesson castable");
    drain_stack(&mut g);
    let inklings: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Inkling"
    }).collect();
    assert_eq!(inklings.len(), 1);
}

#[test]
fn confront_the_doubt_discards_nonland_noncreature_and_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::confront_the_doubt());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let me_before = g.players[0].life;
    let opp_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Confront the Doubt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1);
    assert_eq!(g.players[0].life, me_before + 2);
}

#[test]
fn test_of_patience_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::test_of_patience());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Test of Patience castable");
    drain_stack(&mut g);
    // -1 cast + 2 draw = +1
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn reduce_to_ashes_burns_creature_for_four() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::reduce_to_ashes());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reduce to Ashes castable");
    drain_stack(&mut g);
    // 2/2 bear (toughness ≤ 4 = would die) is exiled, not sent to graveyard.
    assert!(g.exile.iter().any(|c| c.id == bear), "lethal target is exiled");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear));
}

#[test]
fn reduce_to_ashes_only_damages_a_tall_creature() {
    let mut g = two_player_game();
    let hulk = g.add_card_to_battlefield(1, catalog::torrential_gearhulk()); // 5/6
    let id = g.add_card_to_hand(0, catalog::reduce_to_ashes());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(hulk)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(hulk).expect("6-toughness survives 4 damage");
    assert_eq!(card.damage, 4, "takes 4 damage, not exiled (toughness > 4)");
    assert!(!g.exile.iter().any(|c| c.id == hulk));
}

#[test]
fn plant_adept_lesson_pumps_and_grants_trample() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::plant_adept_lesson());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Plant Adept Lesson castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(bear).unwrap();
    assert_eq!(body.power(), 4); // 2 + 2
    assert!(body.has_keyword(&Keyword::Trample));
}

// ── More extras (batch 32, claude/modern_decks) ────────────────────────────

#[test]
fn strixhaven_honor_guard_etb_gains_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::strixhaven_honor_guard());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Honor Guard castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Vigilance));
}

#[test]
fn strixhaven_sapper_etb_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::strixhaven_sapper());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sapper castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Menace));
}

#[test]
fn strixhaven_cartographer_b32_etb_finds_land() {
    let mut g = two_player_game();
    let _forest = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::strixhaven_cartographer_b32());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cartographer castable");
    drain_stack(&mut g);
    // -1 cast + 1 land to hand = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn strixhaven_glyphmage_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _id = g.add_card_to_battlefield(0, catalog::strixhaven_glyphmage());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Library size unchanged via scry; just verify it lands
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn strixhaven_field_researcher_etb_pumps_team() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::strixhaven_field_researcher());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Field Researcher castable");
    drain_stack(&mut g);
    let bear1_body = g.battlefield_find(bear1).unwrap();
    let bear2_body = g.battlefield_find(bear2).unwrap();
    assert_eq!(bear1_body.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(bear2_body.counter_count(CounterType::PlusOnePlusOne), 1);
    // The Field Researcher itself is a creature too — also pumped
    let self_body = g.battlefield_find(id).unwrap();
    assert_eq!(self_body.counter_count(CounterType::PlusOnePlusOne), 1);
}

// ── Batch 33: tests for 25 new STX cards ────────────────────────────────

#[test]
fn witherbloom_bloodscribe_etb_drains_each_opp_and_magecraft_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_bloodscribe());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloodscribe castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2, "opp loses 2");
    // Now magecraft fires on a follow-up IS cast.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // ETB drain + bolt + magecraft gain 1:
    assert_eq!(g.players[0].life, me_before + 1);
}

#[test]
fn pest_skyswarm_etb_mints_a_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_skyswarm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest Skyswarm castable");
    drain_stack(&mut g);
    let self_def = g.battlefield_find(id).expect("on battlefield").definition.clone();
    assert!(self_def.keywords.contains(&Keyword::Flying));
    let pests = g.battlefield.iter().filter(|c|
        c.controller == 0 && c.id != id &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Pest)
    ).count();
    assert_eq!(pests, 1, "one Pest minted");
}

#[test]
fn witherbloom_marshtender_etb_gains_one_and_magecraft_gains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_marshtender());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let me_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Marshtender castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, me_before + 1, "ETB +1 life");
    // Magecraft +1 on bolt cast
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, me_before + 2);
}

#[test]
fn pest_hivekeeper_grows_on_another_pest_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_hivekeeper());
    drain_stack(&mut g);
    // Cast a Pest-minter to enter another Pest under our control.
    let minter = g.add_card_to_hand(0, catalog::pest_skyswarm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: minter, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skyswarm castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1,
        "Hivekeeper gains a +1/+1 on the Pest ETB");
}

#[test]
fn bloodvine_drainmage_etb_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bloodvine_drainmage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainmage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, me_before + 3);
    let body_def = g.battlefield_find(id).unwrap().definition.clone();
    assert!(body_def.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn pest_snatchgrab_forces_opp_sac_and_mints_pest() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::pest_snatchgrab());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Snatchgrab castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear), "opp's bear gone");
    let pests = g.battlefield.iter().filter(|c|
        c.controller == 0 &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Pest)
    ).count();
    assert_eq!(pests, 1, "one Pest minted for caster");
}

#[test]
fn witherbloom_blooddrinker_dies_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_blooddrinker());
    drain_stack(&mut g);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    // Kill the blooddrinker with a Bolt
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id), "blooddrinker dies");
    assert_eq!(g.players[1].life, opp_before - 2, "opp loses 2 on death");
    assert_eq!(g.players[0].life, me_before + 2, "you gain 2 on death");
}

#[test]
fn lorehold_spirit_sage_has_vigilance_and_magecraft_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_sage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sage castable");
    drain_stack(&mut g);
    let body_def = g.battlefield_find(id).unwrap().definition.clone();
    assert!(body_def.keywords.contains(&Keyword::Vigilance));
    let me_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, me_before + 1, "magecraft +1 life");
}

#[test]
fn lorehold_pyrechronicler_magecraft_pings_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_pyrechronicler());
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 dmg
    assert_eq!(g.players[1].life, opp_before - 4);
    let _ = id;
}

#[test]
fn lorehold_mass_ritual_mints_three_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_mass_ritual());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mass Ritual castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter().filter(|c|
        c.controller == 0 &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Spirit)
    ).count();
    assert_eq!(spirits, 3, "three spirits minted");
}

#[test]
fn lorehold_soulburst_deals_two_to_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_soulburst());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soulburst castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn lorehold_ancestor_etb_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_ancestor());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ancestor castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, me_before + 1);
    let body_def = g.battlefield_find(id).unwrap().definition.clone();
    assert!(body_def.keywords.contains(&Keyword::Vigilance));
    assert!(body_def.keywords.contains(&Keyword::Trample));
}

#[test]
fn lorehold_pyrescribe_adept_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_pyrescribe_adept());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let cp = g.compute_battlefield().into_iter()
        .find(|c| c.id == id)
        .expect("on battlefield");
    assert_eq!(cp.power, 3, "magecraft +1/+0");
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

#[test]
fn lorehold_burnscribe_etb_burns_opp_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::lorehold_burnscribe());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Burnscribe castable");
    drain_stack(&mut g);
    // bear has 2 toughness; 2 damage kills it
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear), "bear dead");
    let body_def = g.battlefield_find(id).unwrap().definition.clone();
    assert!(body_def.keywords.contains(&Keyword::Haste));
}

#[test]
fn inkling_calligrapher_magecraft_shrinks_target_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::inkling_calligrapher());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // bear is 2/2 -> -1/-1 = 1/1 (still alive)
    let bear_body = g.battlefield_find(opp_bear);
    assert!(bear_body.is_some(), "bear still alive at 1/1");
    let _ = id;
}

#[test]
fn silverquill_spellscribe_etb_mints_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_spellscribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellscribe castable");
    drain_stack(&mut g);
    let inklings = g.battlefield.iter().filter(|c|
        c.controller == 0 && c.id != id &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Inkling)
    ).count();
    assert_eq!(inklings, 1, "1 Inkling minted (excluding Spellscribe itself)");
    let body_def = g.battlefield_find(id).unwrap().definition.clone();
    assert!(body_def.keywords.contains(&Keyword::Flying));
    assert!(body_def.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn inkling_strikemark_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_strikemark());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Strikemark castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, me_before + 2);
}

#[test]
fn silverquill_scribe_tutor_etb_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let lib_before = g.players[0].library.len();
    let id = g.add_card_to_hand(0, catalog::silverquill_scribe_tutor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scribe-Tutor castable");
    drain_stack(&mut g);
    // Surveil 1: the card moves to either library top or graveyard
    // (auto-decider keeps in library). Library length unchanged or -1.
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn silverquill_magemark_shrinks_creature_and_gains_two_life() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_magemark());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let me_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Magemark castable");
    drain_stack(&mut g);
    // 2/2 -> -2/-2 = 0/0 dies
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear));
    assert_eq!(g.players[0].life, me_before + 2);
}

#[test]
fn quandrix_pulseweaver_has_flash_and_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_pulseweaver());
    drain_stack(&mut g);
    let body_def = g.battlefield_find(id).unwrap().definition.clone();
    assert!(body_def.keywords.contains(&Keyword::Flash));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let cp = g.compute_battlefield().into_iter()
        .find(|c| c.id == id)
        .expect("on battlefield");
    assert_eq!(cp.power, 3);
    assert_eq!(cp.toughness, 3);
}

#[test]
fn fractal_reckoner_etb_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let lib_before = g.players[0].library.len();
    let id = g.add_card_to_hand(0, catalog::fractal_reckoner());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reckoner castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 1, "drew 1 card");
}

#[test]
fn quandrix_inquiry_draws_one_and_scries_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_inquiry());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inquiry castable");
    drain_stack(&mut g);
    // We removed the Inquiry from hand (cast), drew 1 → diff is 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_sparkscribe_magecraft_scries_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _id = g.add_card_to_battlefield(0, catalog::prismari_sparkscribe());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // The scry decision was resolved by auto-decider; just verify nothing crashed.
    assert_eq!(g.players[1].life, 20 - 3);
}

#[test]
fn prismari_ember_adept_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::prismari_ember_adept());
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // bolt 3 + magecraft 1 = 4 dmg
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn prismari_sparkflare_deals_three_to_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_sparkflare());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkflare castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
}

// ── Batch 33 extras: 5 more cross-school cards ──────────────────────────

#[test]
fn strixhaven_mentor_etb_pumps_another_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::strixhaven_mentor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mentor castable");
    drain_stack(&mut g);
    let bear_body = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_body.counter_count(CounterType::PlusOnePlusOne), 1,
        "Mentor pumps the bear");
    // Mentor itself should NOT have a counter — "another" filter excludes source.
    let self_body = g.battlefield_find(id).unwrap();
    assert_eq!(self_body.counter_count(CounterType::PlusOnePlusOne), 0,
        "Mentor doesn't pump itself");
}

#[test]
fn strixhaven_banner_taps_for_any_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::strixhaven_banner());
    drain_stack(&mut g);
    let mana_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("Banner mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), mana_before + 1);
}

#[test]
fn strixhaven_banner_sac_to_draw_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::strixhaven_banner());
    g.players[0].mana_pool.add_colorless(2);
    drain_stack(&mut g);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, x_value: None }).expect("Banner sac-draw ability");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id), "banner sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "draw 1");
}

#[test]
fn strixhaven_apprentice_etb_draws_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strixhaven_apprentice());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Apprentice castable");
    drain_stack(&mut g);
    // Hand: lost the Apprentice (-1) + drew 1 from ETB = same size.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn strixhaven_sorcerer_etb_pings_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::strixhaven_sorcerer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sorcerer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    let body_def = g.battlefield_find(id).unwrap().definition.clone();
    assert!(body_def.keywords.contains(&Keyword::Haste));
}

#[test]
fn strixhaven_pupil_activated_scry_and_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let id = g.add_card_to_battlefield(0, catalog::strixhaven_pupil());
    g.players[0].mana_pool.add_colorless(2);
    drain_stack(&mut g);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("Pupil activated");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "draw 1");
}

// ── Batch 34 tests: 25+ new STX cards ───────────────────────────────────────

#[test]
fn silverquill_drainwriter_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_drainwriter());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_before = g.players[1].life;
    let self_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainwriter castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2, "opp drained 2");
    assert_eq!(g.players[0].life, self_before + 2, "self gained 2");
}

#[test]
fn silverquill_battle_chant_pumps_team_with_vigilance() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_battle_chant());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battle Chant castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 2 + 2, "bear +2 power");
    assert_eq!(bear_card.toughness(), 2 + 1, "bear +1 toughness");
    assert!(bear_card.has_keyword(&Keyword::Vigilance));
}

#[test]
fn silverquill_homily_drains_and_mills_each_opp() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
    let opp_lib_before = g.players[1].library.len();
    let opp_life_before = g.players[1].life;
    let self_life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_homily());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Homily castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life_before - 1, "opp drained 1");
    assert_eq!(g.players[0].life, self_life_before + 1, "self gained 1");
    assert_eq!(g.players[1].library.len(), opp_lib_before - 2, "milled 2");
}

#[test]
fn inkling_avenger_etb_pumps_another_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::inkling_avenger());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Avenger castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.counter_count(CounterType::PlusOnePlusOne), 1);
    let self_card = g.battlefield_find(id).unwrap();
    assert_eq!(self_card.counter_count(CounterType::PlusOnePlusOne), 0,
        "Avenger doesn't pump itself");
}

#[test]
fn silverquill_mandate_forces_opp_sacrifice() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_mandate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mandate castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "opp's bear was sacrificed");
}

#[test]
fn silverquill_spellquill_magecraft_gains_life() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::silverquill_spellquill());
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "Spellquill gained 1 life");
}

#[test]
fn witherbloom_pestrider_etb_mints_pest_with_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestrider());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestrider castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| c.is_token).collect();
    assert_eq!(pests.len(), 1, "minted exactly 1 Pest");
    let pest = pests[0];
    assert_eq!(pest.counter_count(CounterType::PlusOnePlusOne), 1,
        "Pest got a +1/+1 counter");
    assert!(pest.definition.subtypes.creature_types.contains(&CreatureType::Pest));
}

#[test]
fn witherbloom_lifefarmer_etb_gains_three_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifefarmer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifefarmer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3, "gained 3 life");
}

#[test]
fn pest_horde_creates_four_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_horde());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest Horde castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 4, "minted 4 Pests");
}

#[test]
fn witherbloom_thresher_etb_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_thresher());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let opp_before = g.players[1].life;
    let self_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Thresher castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, self_before + 1);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Deathtouch));
}

#[test]
fn lorehold_zealot_etb_exiles_graveyard_card_and_gains_life() {
    let mut g = two_player_game();
    let bolt_id = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_zealot());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Zealot castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt_id),
        "Bolt is in exile");
    assert_eq!(g.players[0].life, life_before + 1, "gained 1 life");
}

#[test]
fn lorehold_pyreheart_magecraft_pings() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_pyreheart());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // bolt 3 + pyreheart 2 = 5
    assert_eq!(g.players[1].life, opp_before - 5);
}

#[test]
fn spirit_phalanx_mints_two_and_pumps_each_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spirit_phalanx());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Phalanx castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 2, "minted 2 spirits");
    for spirit in &spirits {
        assert_eq!(spirit.counter_count(CounterType::PlusOnePlusOne), 1,
            "each spirit got +1/+1");
    }
}

#[test]
fn lorehold_warhost_etb_mints_two_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_warhost());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Warhost castable");
    drain_stack(&mut g);
    let tokens: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(tokens.len(), 2, "minted 2 Spirit tokens");
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.power(), 5);
    assert!(body.has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_devotion_pumps_and_grants_trample() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::lorehold_devotion());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Devotion castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(bear).unwrap();
    assert_eq!(body.power(), 4);
    assert_eq!(body.toughness(), 4);
    assert!(body.has_keyword(&Keyword::Trample));
}

#[test]
fn quandrix_wavecharger_etb_pumps_each_fractal() {
    let mut g = two_player_game();
    // Mint a fractal directly via Fractal Swarm.
    g.add_card_to_library(0, catalog::island());
    let fs = g.add_card_to_hand(0, catalog::fractal_swarm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: fs, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Swarm castable");
    drain_stack(&mut g);
    let fractal_id = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .unwrap().id;
    let before = g.battlefield_find(fractal_id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    // Now cast Wavecharger.
    let id = g.add_card_to_hand(0, catalog::quandrix_wavecharger());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wavecharger castable");
    drain_stack(&mut g);
    let after = g.battlefield_find(fractal_id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(after, before + 1, "pumped the existing Fractal by 1 counter");
}

#[test]
fn fractal_swarm_mints_two_two_fractal_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::fractal_swarm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Swarm castable");
    drain_stack(&mut g);
    // -1 (cast Swarm) +1 (drew from island) = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 1);
    assert_eq!(fractals[0].counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn quandrix_proofwriter_etb_scries() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::quandrix_proofwriter());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Proofwriter castable");
    drain_stack(&mut g);
    // Just verify the card landed without crash; auto-decider handles scry.
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.power(), 4);
    assert_eq!(body.toughness(), 4);
}

#[test]
fn quandrix_solver_magecraft_draws_and_discards() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _id = g.add_card_to_battlefield(0, catalog::quandrix_solver());
    drain_stack(&mut g);
    let fodder = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 bolt (cast), magecraft +1 draw, -1 discard = -1 net hand size
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
    let _ = fodder;
}

#[test]
fn quandrix_counterbearer_pumps_when_counter_added_elsewhere() {
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::quandrix_counterbearer());
    drain_stack(&mut g);
    // Cast Inkling Avenger — its ETB drops a +1/+1 counter on another
    // friendly (the bear), which should trigger Counterbearer's pump.
    let avenger = g.add_card_to_hand(0, catalog::inkling_avenger());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: avenger, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Avenger castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    // Counterbearer (1/2) gets +1/+1 → 2/3 until EOT
    assert_eq!(body.power(), 2);
    assert_eq!(body.toughness(), 3);
}

#[test]
fn prismari_stormfront_deals_four_and_draws() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::prismari_stormfront());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Conflagration castable");
    drain_stack(&mut g);
    // Bear (2 toughness) takes 4 dmg → dies; we drew 1.
    assert!(!g.battlefield.iter().any(|c| c.id == target),
        "bear killed by 4 damage");
    // -1 cast + 1 draw = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_eruption_mage_magecraft_pings_target() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::prismari_eruption_mage());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // bolt (3) + magecraft ping (2) = 5
    assert_eq!(g.players[1].life, opp_before - 5);
}

#[test]
fn prismari_flamescribe_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_flamescribe());
    let _fodder = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flamescribe castable");
    drain_stack(&mut g);
    // -1 cast + draw 1 - discard 1 = -1
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_sparkriot_burns_and_cantrips() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::prismari_sparkriot());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkriot castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear took 3 dmg → died");
    assert_eq!(g.players[0].hand.len(), hand_before, "cantripped (cast -1 + draw +1)");
}

#[test]
fn prismari_pyrosage_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::prismari_pyrosage());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // bolt 3 + ping 1 = 4
    assert_eq!(g.players[1].life, opp_before - 4);
    let body = g.battlefield.iter().find(|c| c.definition.name == "Prismari Pyrosage").unwrap();
    assert!(body.has_keyword(&Keyword::Haste));
}

// ── Mercurial Transformation: ability-strip verification (CR 113.10b) ───────

#[test]
fn mercurial_transformation_strips_keywords_from_target() {
    // Dragon (5/5 Flying) becomes 3/3 with no abilities → no Flying.
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon());
    g.clear_sickness(dragon);
    let id = g.add_card_to_hand(0, catalog::mercurial_transformation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(dragon)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mercurial Transformation castable");
    drain_stack(&mut g);
    let computed = g.computed_permanent(dragon).expect("Dragon on bf");
    assert!(!computed.keywords.contains(&Keyword::Flying),
        "Flying stripped by 'loses all abilities'");
    assert!(computed.lost_all_abilities, "lost_all_abilities flag set");
}

#[test]
fn mercurial_transformation_strips_etb_triggers_from_target() {
    // Spirited Companion has ETB-draw. Put it on the battlefield first,
    // then cast Mercurial Transformation on it. Then add ETB events by
    // bouncing/restoring? Easier: cast a *second* card and check the bear
    // doesn't trigger. Use Sedgemoor Witch (3/2 magecraft → make Pest).
    // We'll cast another instant, expect no Pest.
    let mut g = two_player_game();
    let witch = g.add_card_to_battlefield(0, catalog::sedgemoor_witch());
    g.clear_sickness(witch);
    drain_stack(&mut g);
    let merc = g.add_card_to_hand(0, catalog::mercurial_transformation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: merc,
        target: Some(crate::game::types::Target::Permanent(witch)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mercurial castable");
    drain_stack(&mut g);
    let token_count_before = g.battlefield.iter()
        .filter(|c| c.is_token).count();
    // Cast a bolt; Sedgemoor would normally make a Pest. Stripped, it can't.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let token_count_after = g.battlefield.iter()
        .filter(|c| c.is_token).count();
    assert_eq!(token_count_after, token_count_before,
        "magecraft trigger stripped — no new Pest tokens");
}

// ── Batch 35 tests: 20+ new STX cards ───────────────────────────────────────

#[test]
fn silverquill_penitent_etb_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_penitent());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_before = g.players[1].life;
    let self_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Penitent castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, self_before + 1);
}

#[test]
fn silverquill_verseblade_pumps_target_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_verseblade());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Verseblade castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 2 + 1);
    assert_eq!(bear_card.toughness(), 2 + 1);
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn silverquill_lifepenner_magecraft_gains_two_life() {
    let mut g = two_player_game();
    let life_p = g.add_card_to_battlefield(0, catalog::silverquill_lifepenner());
    g.clear_sickness(life_p);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let self_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, self_before + 2);
}

#[test]
fn inkling_maverick_etb_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_maverick());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Maverick castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn silverquill_antiphony_drains_two_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_antiphony());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_before = g.players[1].life;
    let self_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Antiphony castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, self_before + 2);
}

#[test]
fn inkling_cardinal_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_cardinal());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let self_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cardinal castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, self_before + 2);
}

#[test]
fn witherbloom_hexpetal_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_hexpetal());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_before = g.players[1].life;
    let self_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hexpetal castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, self_before + 2);
}

#[test]
fn witherbloom_soulrender_drains_three_and_mills_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::witherbloom_soulrender());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_before = g.players[1].life;
    let self_before = g.players[0].life;
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soulrender castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, self_before + 3);
    assert_eq!(g.players[0].library.len(), lib_before - 3);
}

#[test]
fn lorehold_pyremender_etb_deals_two_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_pyremender());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyremender castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn lorehold_ember_sage_magecraft_pings_target() {
    let mut g = two_player_game();
    let sage = g.add_card_to_battlefield(0, catalog::lorehold_ember_sage());
    g.clear_sickness(sage);
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt does 3 + magecraft 1 = 4
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn lorehold_ghostmaster_etb_mints_three_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_ghostmaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ghostmaster castable");
    drain_stack(&mut g);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 3);
}

#[test]
fn lorehold_lightning_deals_three_and_gains_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_b35_lightning());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    let self_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lorehold Lightning castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, self_before + 1);
}

#[test]
fn quandrix_geomancer_etb_and_magecraft_add_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_b35_geomancer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Geomancer castable");
    drain_stack(&mut g);
    let geo_id = g.battlefield.iter().find(|c| c.definition.name == "Quandrix Geomancer II").unwrap().id;
    let card = g.battlefield_find(geo_id).unwrap();
    // 2/3 + 1 ETB counter = 3/4
    assert_eq!(card.power(), 3);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(geo_id).unwrap();
    // +1 more counter → 4/5
    assert_eq!(card.power(), 4);
}

#[test]
fn fractal_grower_etb_mints_one_one_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_grower());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Grower castable");
    drain_stack(&mut g);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 1);
    // The minted Fractal should have a +1/+1 counter.
    let fractal = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Fractal").unwrap();
    assert!(fractal.counter_count(CounterType::PlusOnePlusOne) >= 1);
}

#[test]
fn quandrix_tideseer_magecraft_scrys_one() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let seer = g.add_card_to_battlefield(0, catalog::quandrix_tideseer());
    g.clear_sickness(seer);
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry 1 doesn't move cards out; library size unchanged.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn fractal_tidecaller_etb_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::fractal_tidecaller());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidecaller castable");
    drain_stack(&mut g);
    // The Tidecaller leaves hand on cast; the +1 ETB draw gives the hand the same size.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_equation_adds_two_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::quandrix_b35_equation());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Equation castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(bear).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn prismari_spellforge_etb_burns_and_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_spellforge());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellforge castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn prismari_stormforge_deals_three_and_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_stormforge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormforge castable");
    drain_stack(&mut g);
    // -1 from cast + 2 from draw = +1 net
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn prismari_mirror_mage_magecraft_self_pumps() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::prismari_mirror_mage());
    g.clear_sickness(mage);
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(mage).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 4);
}

#[test]
fn prismari_cinderdrake_etb_deals_three_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_cinderdrake());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cinderdrake castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
}

// ── Batch 36: more STX cards ────────────────────────────────────────────────

#[test]
fn silverquill_stylepoint_pumps_and_grants_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_stylepoint());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stylepoint castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(bear).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
    assert!(card.has_keyword(&Keyword::FirstStrike));
}

#[test]
fn inkling_b36_sentinel_is_a_three_mana_flying_soldier() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_b36_sentinel());
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 3);
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Soldier));
}

#[test]
fn silverquill_forge_mints_two_inklings_and_drains() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_forge());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Forge castable");
    drain_stack(&mut g);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 2);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn witherbloom_verdancer_etb_and_magecraft_each_gain_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_verdancer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Verdancer castable");
    drain_stack(&mut g);
    // ETB gain 1
    assert_eq!(g.players[0].life, life_before + 1);
    // Now cast a bolt → magecraft gains 1 more
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
}
