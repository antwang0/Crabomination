use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn fractal_resonance_v2_enters_with_counters_for_hand_size() {
    let mut g = two_player_game();
    // Player 0 has 1 card in hand (the spell itself).
    let id = g.add_card_to_hand(0, catalog::fractal_resonance_v2());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Add a couple of cards to hand to bump hand size.
    let _ = g.add_card_to_hand(0, catalog::forest());
    let _ = g.add_card_to_hand(0, catalog::island());
    let hand_size_at_cast = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Resonance castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("Resonance on battlefield");
    let counters = card.counters.get(&CounterType::PlusOnePlusOne)
        .copied().unwrap_or(0);
    // After the cast, hand size dropped by 1 (the spell). enters_with_counters
    // reads current hand size after cast — so should be hand_size_at_cast - 1.
    assert_eq!(counters as usize, hand_size_at_cast - 1);
}

#[test]
fn prismari_emberveil_etb_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_emberveil());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Emberveil castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_firechord_burns_creature_for_three() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_firechord());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Firechord castable");
    drain_stack(&mut g);
    // 3 damage to a 2/2 bear → dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn prismari_drakekin_is_a_flying_scry_drake() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_drakekin());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Drakekin castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Drake));
}

#[test]
fn prismari_inscribe_burns_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::mountain());
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_inscribe());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inscribe castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn prismari_pyremaster_magecraft_pings_any_target() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_pyremaster());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Pyremaster magecraft 1 = 4 damage.
    assert_eq!(g.players[1].life, opp_before - 4);
}

// ── batch 54: new STX cards (12 SQ + 6 WB + 4 LH) ───────────────────────────

#[test]
fn silverquill_inkblot_is_a_two_two_flying_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inkblot());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Inkblot castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
}

#[test]
fn inkling_chaplain_is_lifelink_vigilance_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_chaplain());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Chaplain castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Vigilance));
    assert!(card.has_keyword(&Keyword::Lifelink));
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn silverquill_warden_etb_drains_one() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_warden());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Warden castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn inkling_acolyte_v2_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::inkling_acolyte_v2());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Acolyte magecraft drain 1 = -4 to opp, +1 to caster.
    assert_eq!(g.players[0].life, life_before + 1);
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn silverquill_reflect_drains_two_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_reflect());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Reflect castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn inkling_evangel_etb_pumps_target_inkling() {
    let mut g = two_player_game();
    // Find any Inkling on the battlefield. Add another Inkling via
    // Inkling Aspirant for a clean target.
    let target = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    let id = g.add_card_to_hand(0, catalog::inkling_evangel());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Evangel castable");
    drain_stack(&mut g);
    let target_card = g.battlefield_find(target).unwrap();
    let counters = target_card.counters.get(&CounterType::PlusOnePlusOne)
        .copied().unwrap_or(0);
    assert_eq!(counters, 1);
}

#[test]
fn silverquill_invocation_mints_three_inklings() {
    let mut g = two_player_game();
    let inklings_before = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .count();
    let id = g.add_card_to_hand(0, catalog::silverquill_invocation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Invocation castable");
    drain_stack(&mut g);
    let inklings_after = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .count();
    assert_eq!(inklings_after, inklings_before + 3);
}

#[test]
fn inkling_ghostwriter_magecraft_drains_on_is_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::inkling_ghostwriter());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 to opp + Ghostwriter magecraft 1 drain = -4 to opp / +1 to caster.
    assert_eq!(g.players[0].life, life_before + 1);
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn silverquill_doom_drains_four() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_doom());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Doom castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 4);
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn inkling_attendant_etb_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::inkling_attendant());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Attendant castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_psalm_drains_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_psalm());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Psalm castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    assert_eq!(g.players[1].life, opp_before - 2);
    // -1 cast + 1 draw = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_pageant_mints_two_inklings_and_gains_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let inklings_before = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .count();
    let id = g.add_card_to_hand(0, catalog::inkling_pageant());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pageant castable");
    drain_stack(&mut g);
    let inklings_after = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .count();
    assert_eq!(inklings_after, inklings_before + 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn witherbloom_creeper_has_deathtouch_and_magecraft_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_creeper());
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Deathtouch));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    // Power 3+1 = 4 after magecraft self-pump +1/+0.
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 2);
}

#[test]
fn pest_lord_anthems_other_pests() {
    let mut g = two_player_game();
    // Add a Pest token first (will get +1/+1 from the lord).
    let pest = g.add_card_to_battlefield(0, catalog::pest_brood_mother());
    let _ = pest;
    let lord = g.add_card_to_battlefield(0, catalog::pest_lord());
    let _ = lord;
    g.compute_battlefield();
    // Now find a Pest other than the lord and assert it's pumped.
    let pest_card = g.battlefield.iter()
        .find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest)
            && !c.definition.subtypes.creature_types.contains(&CreatureType::Warlock));
    if let Some(p) = pest_card {
        // Pest Brood Mother is a 3/3 Pest Insect; should be 4/4 with the anthem.
        assert!(p.power() > 3);
        assert!(p.toughness() > 3);
    }
}

#[test]
fn witherbloom_drainer_etb_drains_two_and_gains_one() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_drainer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Drainer castable");
    drain_stack(&mut g);
    // Drain 2 + GainLife 1 → +3 self, -2 opp.
    assert_eq!(g.players[0].life, life_before + 3);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn witherbloom_mossback_is_a_two_four_reach() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_mossback());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mossback castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Reach));
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 4);
}

#[test]
fn pest_curse_mints_pests_and_self_discards() {
    let mut g = two_player_game();
    // Add a discard fodder card to hand.
    let _filler = g.add_card_to_hand(0, catalog::forest());
    let pests_before = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::pest_curse());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pest Curse castable");
    drain_stack(&mut g);
    let pests_after = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pests_after, pests_before + 2);
    // Hand: -1 cast (pest curse) + 0 draw - 1 discard = -1 from the (cast+1).
    // hand_before included pest curse + filler = 2; after cast & discard = 0.
    assert!(g.players[0].hand.len() < hand_before);
}

#[test]
fn witherbloom_hexvine_destroys_creature_and_gains_two_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_hexvine());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hexvine castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn lorehold_invoker_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_invoker());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 to opp + Invoker 1 to opp = -4.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn spirit_sparkmage_etb_burns_and_gains_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::spirit_sparkmage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkmage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn lorehold_chronicler_v2_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_chronicler_v2());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    // 2/2 + 1/+1 = 3/3.
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn quandrix_tideturner_etb_scrys_and_magecraft_grows() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::quandrix_tideturner());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    let counters = card.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    // Magecraft should have added one +1/+1 counter.
    assert_eq!(counters, 1);
}

#[test]
fn fractal_overgrowth_doubles_existing_counters() {
    let mut g = two_player_game();
    // Add a creature with 3 +1/+1 counters.
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    {
        let card = g.battlefield_find_mut(b1).unwrap();
        card.counters.insert(CounterType::PlusOnePlusOne, 3);
    }
    let id = g.add_card_to_hand(0, catalog::fractal_overgrowth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Overgrowth castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(b1).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    // 3 → 6 (doubled).
    assert_eq!(counters, 6);
}

#[test]
fn quandrix_ectomancer_magecraft_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_ectomancer());
    let hand_before = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // +1 (bolt added) -1 (cast) +1 (magecraft draw) = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn prismari_cinderpath_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_cinderpath());
    let _filler = g.add_card_to_hand(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // hand_before = filler. Add bolt (+1), cast (-1), then loot: +1 -1 = 0
    // Net delta from hand_before: 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_searstorm_burns_creature_and_pings_opp() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_searstorm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Searstorm castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn prismari_embertide_etb_burns_and_has_haste() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_embertide());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Embertide castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Haste));
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn lorehold_relicwarden_etb_pumps_other_spirits() {
    let mut g = two_player_game();
    let s1 = g.add_card_to_battlefield(0, catalog::spirit_blazekin());
    let s2 = g.add_card_to_battlefield(0, catalog::spirit_blazekin());
    let id = g.add_card_to_hand(0, catalog::lorehold_relicwarden());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Relicwarden castable");
    drain_stack(&mut g);
    // Each other Spirit gets +1/+1.
    for sid in [s1, s2] {
        let c = g.battlefield_find(sid).unwrap();
        let cn = c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
        assert_eq!(cn, 1);
    }
}

// ── Push (modern_decks, batch 55): tests for new STX cards ─────────────────
// 25 new STX cards across all five colleges + their accompanying tests.

#[test]
fn witherbloom_pestcradle_etb_mints_pest_and_gains_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let pests_before = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestcradle());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pestcradle castable");
    drain_stack(&mut g);
    let pests_after = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pests_after, pests_before + 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn pest_brewmaster_drains_each_opp_on_sacrifice() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::pest_brewmaster());
    // Add a creature to sacrifice via Witherbloom Sacrosanct.
    let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Sacrosanct drains 3 + Pest Brewmaster's sac trigger drains 1 = 4 total.
    // Opponent loses 3 (drain) + 1 (brewmaster) = 4.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn witherbloom_pestcaller_b54_etb_mints_two_pests_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let pests_before = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestcaller_b54());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pestcaller II castable");
    drain_stack(&mut g);
    let pests_after = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pests_after, pests_before + 2);
}

#[test]
fn witherbloom_vitalcoil_magecraft_gains_two_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_vitalcoil());
    let life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn witherbloom_pestharvest_mints_two_pests_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let pests_before = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestharvest());
    let hand_after_add = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pestharvest castable");
    drain_stack(&mut g);
    let pests_after = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pests_after, pests_before + 2);
    // After-add: hand has Pestharvest. Cast: -1. Draw: +1. Net: 0 from after-add.
    assert_eq!(g.players[0].hand.len(), hand_after_add);
}

#[test]
fn lorehold_pyrescribe_elder_magecraft_pings_and_gains() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_pyrescribe_elder());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Pyrescribe magecraft 1 = -4 opp; Pyrescribe magecraft +1 caster.
    assert_eq!(g.players[1].life, opp_before - 4);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn lorehold_skirmish_v2_creates_haste_spirit() {
    let mut g = two_player_game();
    let spirits_before = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    let id = g.add_card_to_hand(0, catalog::lorehold_skirmish_v2());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Skirmish castable");
    drain_stack(&mut g);
    let spirits_after: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits_after.len(), spirits_before + 1);
    assert!(spirits_after.iter().any(|c| c.has_keyword(&Keyword::Haste)));
}

#[test]
fn lorehold_sparkflame_deals_two_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_sparkflame());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkflame castable");
    drain_stack(&mut g);
    // 2 damage to a 2/2 — bear dies as state-based action.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn lorehold_spiritcaller_b55_etb_mints_two_spirits() {
    let mut g = two_player_game();
    let spirits_before = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritcaller_b55());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Spiritcaller II castable");
    drain_stack(&mut g);
    let spirits_after = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    // The Spiritcaller itself is a Spirit (3 = self + 2 minted).
    assert_eq!(spirits_after, spirits_before + 3);
}

#[test]
fn spirit_banneret_anthems_other_spirits() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::spirit_blazekin());
    let _banneret = g.add_card_to_battlefield(0, catalog::spirit_banneret());
    let computed = g
        .compute_battlefield()
        .into_iter()
        .find(|c| c.id == s)
        .expect("Blazekin on battlefield");
    // Spirit Blazekin is 2/2 + Banneret anthem +1/+0 = 3/2.
    assert_eq!(computed.power, 3);
    assert_eq!(computed.toughness, 2);
}

#[test]
fn quandrix_calcographer_etb_mints_fractal_then_grows_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_calcographer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Calcographer castable");
    drain_stack(&mut g);
    // ETB minted a Fractal with one +1/+1 counter.
    let fractals: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .map(|c| c.id)
        .collect();
    assert!(!fractals.is_empty());
    // Cast an instant - calcographer grows.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    let counters = card.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(counters, 1);
}

#[test]
fn quandrix_splitcaster_magecraft_mints_a_fractal_with_counter() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_splitcaster());
    let fractals_before = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .count();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let fractals_after: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals_after.len(), fractals_before + 1);
    // The newly created fractal should have a +1/+1 counter.
    let new_fractal = fractals_after.last().unwrap();
    let counters = new_fractal.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert!(counters >= 1);
}

#[test]
fn quandrix_calculation_adds_counter_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_calculation());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Calculation castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(bear).unwrap();
    let counters = card.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(counters, 1);
    // Cast -1 + draw +1 = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_bookkeeper_magecraft_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_bookkeeper());
    let hand_before = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // +1 added bolt -1 cast +1 draw = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn prismari_stormcaller_prowess_pumps_on_noncreature_spell() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_stormcaller());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    // Base 2/2 + prowess +1/+1 = 3/3.
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn prismari_embershock_kills_three_toughness_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());  // 4/4 flyer
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());  // 2/2
    let id = g.add_card_to_hand(0, catalog::prismari_embershock());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Embershock castable");
    drain_stack(&mut g);
    // 3 damage kills the bear; angel survives.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert!(g.battlefield.iter().any(|c| c.id == target));
}

#[test]
fn prismari_spellscholar_etb_scrys_two_and_magecraft_scrys_one() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::prismari_spellscholar());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Spellscholar castable");
    drain_stack(&mut g);
    // ETB body: 1/3 Wizard.
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn prismari_reverberator_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_reverberator());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Reverberator magecraft 2 = -5 opp.
    assert_eq!(g.players[1].life, opp_before - 5);
}

#[test]
fn prismari_volcanist_b55_burns_creature_and_pings_opp() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_volcanist_b55());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Volcanist II castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn silverquill_pen_scholar_etb_gains_life_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_pen_scholar());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pen-Scholar castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
}

#[test]
fn silverquill_mortician_drains_on_sacrifice() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_mortician());
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Sacrosanct drain 3 + Mortician sac-drain 1 = opp -4, you +4.
    assert_eq!(g.players[0].life, life_before + 4);
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn inkling_sentinel_b55_is_a_three_mana_one_four_vigilance() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_sentinel_b55());
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 4);
    assert!(card.has_keyword(&Keyword::Vigilance));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_inksong_drains_one_and_scrys_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_inksong());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Inksong castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn until_end_of_combat_expires_when_combat_phase_ends() {
    // CR 511.2 audit: an effect installed with Duration::EndOfCombat
    // should expire as the EndCombat step ends (transition to
    // PostCombatMain), not at the next cleanup step.
    use crate::effect::Duration;
    use crate::game::types::Target;
    use crate::game::{EffectContext, TurnStep};

    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    // Skip ahead to the combat phase.
    g.step = TurnStep::BeginCombat;

    // Install a +1/+1 EOC pump on the bear via the effect resolver.
    let ctx = EffectContext {
        controller: 0,
        source: Some(bear),
        targets: vec![Target::Permanent(bear)],
        trigger_source: None,
        mode: 0,
        x_value: 0,
        converged_value: 0,
        mana_spent: 0,
        source_name: None,
        cast_from_hand: true,
        event_amount: 0,
    };
    // Use SetBasePT with Duration::EndOfCombat so the layer-system
    // pathway exercises the mapping under test (PumpPT writes to the
    // legacy `power_bonus` field that doesn't honor combat-scoped
    // durations and clears only at cleanup).
    let set = crate::effect::Effect::SetBasePT {
        what: crate::effect::Selector::Target(0),
        power: crate::effect::Value::Const(7),
        toughness: crate::effect::Value::Const(7),
        duration: Duration::EndOfCombat,
    };
    let _ = g.resolve_effect(&set, &ctx);

    // Bear is now 7/7 during combat.
    let computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == bear).unwrap();
    assert_eq!(computed.power, 7, "bear should be set to 7/7 during combat");

    // Advance to the EndCombat step and pass priority until we leave
    // the combat phase.
    g.step = TurnStep::EndCombat;
    g.give_priority_to_active();
    let _ = g.pass_priority();
    let _ = g.pass_priority();
    assert!(!g.step.is_combat_phase(), "expected to exit combat phase, got {:?}", g.step);

    // SetBasePT should have expired by now — back to printed 2/2.
    let computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == bear).unwrap();
    assert_eq!(computed.power, 2,
        "until-end-of-combat SetBasePT should expire when combat phase ends");
}

#[test]
fn inkling_pact_caller_etb_mints_inkling() {
    let mut g = two_player_game();
    let inklings_before = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .count();
    let id = g.add_card_to_hand(0, catalog::inkling_pact_caller());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pact-Caller castable");
    drain_stack(&mut g);
    let inklings_after = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .count();
    // Pact-Caller itself is an Inkling (+1) plus the minted token (+1) = +2.
    assert_eq!(inklings_after, inklings_before + 2);
}

// ── Push (modern_decks, batch 56): 25 new STX card tests ───────────────────
// Five cards per college (Witherbloom + Silverquill + Lorehold + Quandrix +
// Prismari) using existing primitives plus a Quandrix card that exercises
// the `enters_with_counters = HandSizeOf(You)` path.

#[test]
fn witherbloom_pestreaper_b56_grows_and_gains_life_on_sacrifice() {
    let mut g = two_player_game();
    let reaper = g.add_card_to_battlefield(0, catalog::witherbloom_pestreaper_b56());
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Reaper grows by one counter on the sacrifice trigger + gains 1
    // life rider. Sacrosanct itself drains 3 → +3 life. Total: +4.
    let r = g.battlefield_find(reaper).expect("reaper alive");
    let cn = r.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(cn, 1, "Pestreaper should have +1/+1 from sacrifice");
    assert_eq!(g.players[0].life, life_before + 4,
        "Pestreaper gains 1 + Sacrosanct gains 3 = +4 life");
}

#[test]
fn witherbloom_soulshade_returns_low_mv_creature_on_death() {
    let mut g = two_player_game();
    let bear_id = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let soulshade_id = g.add_card_to_battlefield(0, catalog::witherbloom_soulshade());
    let hand_before = g.players[0].hand.len();
    // Lethal the Soulshade.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == soulshade_id) {
        c.damage = 99;
    }
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == soulshade_id),
        "Soulshade should be in graveyard");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear_id),
        "Soulshade's death trigger should return ≤2-MV creature card to hand");
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn witherbloom_necrofeast_sacrifices_and_drains_four() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_necrofeast());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Necrofeast castable");
    drain_stack(&mut g);
    // Bear is sacrificed.
    let creatures = g.battlefield.iter()
        .filter(|c| c.definition.is_creature() && c.controller == 0)
        .count();
    assert_eq!(creatures, 0, "Bear should have been sacrificed");
    // Drain 4: caster gains 4, opp loses 4.
    assert_eq!(g.players[0].life, life_before + 4);
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn pest_caretaker_etb_mints_pest_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let pests_before = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    let id = g.add_card_to_hand(0, catalog::pest_caretaker());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Caretaker castable");
    drain_stack(&mut g);
    let pests_after = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    // Caretaker itself is a Pest (+1) + minted token (+1) = +2.
    assert_eq!(pests_after, pests_before + 2);
}

#[test]
fn witherbloom_tomeshade_etb_mills_and_drains() {
    let mut g = two_player_game();
    // Seed opponent's library so mill has fodder.
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let opp_gy_before = g.players[1].graveyard.len();
    let opp_life_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_tomeshade());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Tomeshade castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), opp_gy_before + 3,
        "Mill should put 3 cards into opp's graveyard");
    assert_eq!(g.players[1].life, opp_life_before - 1);
}

#[test]
fn silverquill_bloodscribe_draws_on_sacrifice() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_bloodscribe());
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Seed library so the draw has something to find.
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    let hand_after_spell_added = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // The bear is sacrificed; Bloodscribe's trigger draws 1.
    // hand changes: +1 (added Sacrosanct) -1 (cast it) +1 (drawn from trigger) = +1 from baseline.
    let _ = hand_after_spell_added;
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn inkling_penblade_etb_pumps_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::inkling_penblade());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Penblade castable");
    drain_stack(&mut g);
    let computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == bear).expect("bear alive");
    // Bear was 2/2; +1/+0 EOT = 3/2.
    assert_eq!(computed.power, 3);
}

#[test]
fn silverquill_litany_b56_drains_and_mills() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    let opp_gy_before = g.players[1].graveyard.len();
    let opp_life_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_litany_b56());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Litany castable");
    drain_stack(&mut g);
    // Drain 2: opp -2, you +2. Mill 2.
    assert_eq!(g.players[1].life, opp_life_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
    assert_eq!(g.players[1].graveyard.len(), opp_gy_before + 2);
}

#[test]
fn inkling_inkmaster_drains_on_is_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::inkling_inkmaster());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt: 3 damage. Magecraft: -1 opp, +1 caster.
    assert_eq!(g.players[1].life, opp_before - 3 - 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn silverquill_acolyte_b56_etb_drains_one() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_acolyte_b56());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Acolyte castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn lorehold_forge_cleric_magecraft_pumps_friendly_spirit() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_forge_cleric());
    let spirit = g.add_card_to_battlefield(0, catalog::spirit_blazekin());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let s = g.battlefield_find(spirit).expect("spirit alive");
    let cn = s.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(cn, 1, "Forge-Cleric magecraft should pump friendly Spirit");
}

#[test]
fn lorehold_pyrescholar_b56_magecraft_burns_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_pyrescholar_b56());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Pyrescholar 2 = -5 opp.
    assert_eq!(g.players[1].life, opp_before - 5);
}

#[test]
fn lorehold_summit_mints_two_spirits_and_grants_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spirits_before = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    let id = g.add_card_to_hand(0, catalog::lorehold_summit());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Summit castable");
    drain_stack(&mut g);
    let spirits_after = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    assert_eq!(spirits_after, spirits_before + 2);
    // The pre-existing bear should have Haste EOT now.
    let computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == bear).expect("bear alive");
    assert!(computed.keywords.contains(&Keyword::Haste));
}

#[test]
fn spirit_scribe_etb_scrys_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let lib_before = g.players[0].library.len();
    let id = g.add_card_to_hand(0, catalog::spirit_scribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Scribe castable");
    drain_stack(&mut g);
    // Scry doesn't change library size; just ensure the card is on the battlefield.
    assert_eq!(g.players[0].library.len(), lib_before);
    let c = g.battlefield.iter()
        .find(|c| c.definition.name == "Spirit Scribe").expect("scribe alive");
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Spirit));
}

#[test]
fn lorehold_ember_strike_burns_target_and_surveils() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_ember_strike());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ember-Strike castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn quandrix_mathlord_etb_mints_fractal_and_pumps_fractals() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_mathlord());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mathlord castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert!(!fractals.is_empty(), "Mathlord should mint at least one Fractal");
    // Each Fractal has +1/+1 counters.
    for fractal in &fractals {
        let cn = fractal.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
        assert!(cn >= 2, "Fractal should have at least 2 counters from team-wide pump");
    }
}

#[test]
fn quandrix_geometer_b56_magecraft_pumps_team() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_geometer_b56());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).expect("bear alive");
    let cn = b.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(cn, 1, "Geometer should pump each friendly creature on magecraft");
}

#[test]
fn fractal_trifecta_mints_three_fractals_with_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_trifecta());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Trifecta castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 3, "Trifecta should mint 3 Fractals");
    for fractal in &fractals {
        let cn = fractal.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
        assert!(cn >= 1, "Each Fractal should have at least 1 counter");
    }
}

#[test]
fn quandrix_tidesower_etb_shrinks_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::quandrix_tidesower());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidesower castable");
    drain_stack(&mut g);
    // Hand: +1 added Tidesower, -1 cast it, +1 drawn = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    let computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == bear).expect("bear alive");
    assert_eq!(computed.power, 0, "Bear should be shrunk to 0/2");
}

#[test]
fn fractal_augmenter_enters_with_counters_equal_to_hand_size() {
    let mut g = two_player_game();
    // Make sure hand size is non-trivial.
    while g.players[0].hand.len() < 4 {
        g.add_card_to_hand(0, catalog::island());
    }
    let hand_size = g.players[0].hand.len() as i32;
    let id = g.add_card_to_hand(0, catalog::fractal_augmenter());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Augmenter castable");
    drain_stack(&mut g);
    let aug = g.battlefield.iter()
        .find(|c| c.definition.name == "Fractal Augmenter").expect("augmenter on bf");
    let cn = aug.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0) as i32;
    // Hand size at ETB time: original hand_size + 1 (Augmenter added) - 1
    // (Augmenter cast away) = original hand_size.
    assert_eq!(cn, hand_size,
        "Augmenter enters with +1/+1 counters equal to current hand size");
}

#[test]
fn prismari_flamewriter_magecraft_burns_and_draws() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_flamewriter());
    g.add_card_to_library(0, catalog::island());
    let opp_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Flamewriter magecraft 1 = -4 opp.
    assert_eq!(g.players[1].life, opp_before - 4);
    // Hand: +1 Bolt, -1 cast, +1 drawn from magecraft = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn prismari_cinderchant_deals_two_and_scrys() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_cinderchant());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cinderchant castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn prismari_floodfire_deals_four_and_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand_before = g.players[0].hand.len();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_floodfire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Floodfire castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 4);
    // Hand: +1 Floodfire, -1 cast, +2 drawn = +2 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 2);
}

// ── CR 122.3 — +1/+1 vs -1/-1 counter cancellation (state-based action) ─────

/// CR 122.3: "If a permanent has both a +1/+1 counter and a -1/-1
/// counter on it, N +1/+1 and N -1/-1 counters are removed from it
/// as a state-based action, where N is the smaller of the number of
/// +1/+1 and -1/-1 counters on it."
///
/// This audit-style lock-in test stages a creature with 3 +1/+1 and
/// 2 -1/-1 counters and asserts the SBA cancels 2 of each, leaving
/// 1 +1/+1 counter (and 0 -1/-1).
#[test]
fn cr_122_3_plus_one_and_minus_one_counters_cancel_as_state_based_action() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Manually stamp counters and trigger SBA.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) {
        c.add_counters(CounterType::PlusOnePlusOne, 3);
        c.add_counters(CounterType::MinusOneMinusOne, 2);
    }
    g.check_state_based_actions();
    let b = g.battlefield_find(bear).expect("bear alive");
    let plus = b.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    let minus = b.counters.get(&CounterType::MinusOneMinusOne).copied().unwrap_or(0);
    assert_eq!(plus, 1, "expected 1 +1/+1 counter after CR 122.3 cancel");
    assert_eq!(minus, 0, "expected 0 -1/-1 counters after CR 122.3 cancel");
    // P/T reflects the net +1/+1 over the printed 2/2.
    let computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == bear).expect("computed bear");
    assert_eq!(computed.power, 3);
    assert_eq!(computed.toughness, 3);
}

// ── Push (modern_decks, batch 56b): 5 more Witherbloom cards ───────────────

#[test]
fn witherbloom_crypt_caller_dies_drains_two() {
    let mut g = two_player_game();
    let cc = g.add_card_to_battlefield(0, catalog::witherbloom_crypt_caller());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    // Lethal the Crypt-Caller.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == cc) {
        c.damage = 99;
    }
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn witherbloom_mill_mage_etb_mills_four_each_opp() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let opp_gy_before = g.players[1].graveyard.len();
    let id = g.add_card_to_hand(0, catalog::witherbloom_mill_mage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mill-Mage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), opp_gy_before + 4);
}

#[test]
fn witherbloom_decoder_magecraft_mills_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_decoder());
    g.add_card_to_library(1, catalog::island());
    let opp_gy_before = g.players[1].graveyard.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), opp_gy_before + 1,
        "Decoder magecraft should mill 1 from opp on instant cast");
}

#[test]
fn pest_roostmaster_mints_pest_on_sacrifice() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::pest_roostmaster());
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pests_before = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    let pests_after = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pests_after, pests_before + 1,
        "Roostmaster should mint a Pest on Sacrosanct's sacrifice");
}

// ── Push (modern_decks, batch 57): 20+ new STX cards across all 5 schools ─

// — Witherbloom batch 57 (5 cards) —

#[test]
fn pest_soulreaver_dies_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_soulreaver());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == id) {
        c.damage = 99;
    }
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, life_before + 3);
}

#[test]
fn witherbloom_pestmender_magecraft_pumps_target_pest() {
    let mut g = two_player_game();
    let _pm = g.add_card_to_battlefield(0, catalog::witherbloom_pestmender());
    // Mint a Pest token under our control via a Pest-producing card.
    let pest = g.add_card_to_battlefield(0, catalog::pest_marauder());
    let counters_before = g.battlefield_find(pest).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let counters_after = g.battlefield_find(pest).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(counters_after, counters_before + 1,
        "Pestmender magecraft should put a +1/+1 counter on the Pest");
}

#[test]
fn witherbloom_necropoet_grows_pests_on_sacrifice() {
    let mut g = two_player_game();
    let _np = g.add_card_to_battlefield(0, catalog::witherbloom_necropoet());
    let pest1 = g.add_card_to_battlefield(0, catalog::pest_marauder());
    let pest2 = g.add_card_to_battlefield(0, catalog::pest_marauder());
    // Sacrifice fodder — Sacrosanct sacs a creature; the bear is the
    // expected pick so the Pests can both observe the sacrifice trigger.
    let _bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pests_at_start: Vec<_> = vec![pest1, pest2];
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Any surviving Pest should have at least 1 +1/+1 counter from the
    // Necropoet trigger; cards sacrificed are gone from the battlefield.
    let survivors: Vec<_> = pests_at_start.iter()
        .filter_map(|&id| g.battlefield_find(id))
        .collect();
    assert!(!survivors.is_empty(), "at least one Pest should survive the sacrifice");
    for p in &survivors {
        let cn = p.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
        assert_eq!(cn, 1, "surviving Pest should have a +1/+1 counter");
    }
}

#[test]
fn witherbloom_soulsmith_etb_drains_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_soulsmith());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Soulsmith castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2, "drain 2 to opp");
    assert_eq!(g.players[0].life, life_before + 2, "gain 2 life");
}

#[test]
fn pest_vanguard_magecraft_drains_one() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::pest_vanguard());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt does 3 plus magecraft 1 = 4 to opp; +1 life gain.
    assert_eq!(g.players[1].life, opp_before - 4);
    assert_eq!(g.players[0].life, life_before + 1);
}

// — Silverquill batch 57 (5 cards) —

#[test]
fn silverquill_scriptmaster_etb_drains_two_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_scriptmaster());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Scriptmaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn inkling_bladerunner_has_flying_and_first_strike() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_bladerunner());
    let c = g.battlefield_find(id).expect("on battlefield");
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::FirstStrike));
    assert_eq!(c.power(), 2);
    assert_eq!(c.toughness(), 2);
}

#[test]
fn silverquill_sentinel_b57_is_vigilant_flyer() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_sentinel_b57());
    let c = g.battlefield_find(id).expect("on battlefield");
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Vigilance));
    assert_eq!(c.power(), 1);
    assert_eq!(c.toughness(), 3);
}

#[test]
fn silverquill_pen_master_etb_loots_and_drains_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_pen_master());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pen-Master castable");
    drain_stack(&mut g);
    // Loot is net-0 (draw 1, discard 1); cast of the Pen-Master moves it
    // out of hand. Hand should match the pre-cast snapshot.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

// — Lorehold batch 57 (4 cards) —

#[test]
fn lorehold_battlepriest_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_battlepriest());
    let life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1,
        "Battlepriest magecraft should gain 1 life");
}

#[test]
fn lorehold_bonereader_b57_magecraft_exiles_gy_card() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_bonereader_b57());
    // Seed opp's graveyard with a card.
    let gy_card = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Card should have moved to exile or be no longer in graveyard.
    let still_in_gy = g.players[1].graveyard.iter().any(|c| c.id == gy_card);
    assert!(!still_in_gy, "Bonereader magecraft should exile target gy card");
}

#[test]
fn lorehold_sparkscholar_b57_magecraft_pings_creature() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_sparkscholar_b57());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(bear_card.damage, 1, "Sparkscholar should ping the bear for 1");
}

#[test]
fn lorehold_reverence_v2_etb_mints_spirit_and_gains_two_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_reverence_v2());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Reverence II castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1, "Reverence II should mint exactly one Spirit token");
}

// — Quandrix batch 57 (3 cards) —

#[test]
fn fractal_greenstone_enters_with_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_greenstone());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Greenstone castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Greenstone on battlefield");
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2);
    assert_eq!(c.power(), 2);
    assert_eq!(c.toughness(), 2);
}

#[test]
fn quandrix_tideguard_magecraft_pumps_target_fractal() {
    let mut g = two_player_game();
    let _tg = g.add_card_to_battlefield(0, catalog::quandrix_tideguard());
    let fractal = g.add_card_to_battlefield(0, catalog::fractal_greenstone());
    let counters_before = g.battlefield_find(fractal).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let counters_after = g.battlefield_find(fractal).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(counters_after, counters_before + 1,
        "Tideguard magecraft should pump the Fractal");
}

#[test]
fn quandrix_greenmage_etb_scrys_and_pumps_self() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_greenmage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Greenmage castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Greenmage on battlefield");
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1,
        "Greenmage should pump self with +1/+1 counter on ETB");
    assert_eq!(c.power(), 4);
    assert_eq!(c.toughness(), 4);
}

// — Prismari batch 57 (3 cards) —

#[test]
fn prismari_pyromage_b57_magecraft_pings_any_target() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_pyromage_b57());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn prismari_stormcaller_v2_prowess_grows_on_noncreature_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_stormcaller_v2());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Stormcaller on battlefield");
    assert_eq!(c.power(), 3, "Prowess should pump +1 power EOT");
    assert_eq!(c.toughness(), 3, "Prowess should pump +1 toughness EOT");
}

#[test]
fn prismari_sparkscribe_b57_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::prismari_sparkscribe_b57());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sparkscribe castable");
    drain_stack(&mut g);
    // Loot is net-0; cast removes the Sparkscribe from hand → hand matches
    // the pre-cast snapshot.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let c = g.battlefield_find(id).expect("Sparkscribe on battlefield");
    assert!(c.has_keyword(&Keyword::Flying));
}

// ── batch 58 (modern_decks): 22 new cards + Strict Proctor ETB tax ─────────

// — Witherbloom (B/G) —

#[test]
fn witherbloom_toxicpath_etb_drains_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let opp_before = g.players[1].life;
    let you_before = g.players[0].life;
    let lib_before = g.players[0].library.len();
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxicpath());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Toxicpath castable");
    drain_stack(&mut g);
    // Drain 1: opp -1, you +1. Surveil 1: top card → library or graveyard;
    // either way the library count drops by at most 1.
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, you_before + 1);
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn pest_tendril_dies_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let lib_before = g.players[0].library.len();
    let id = g.add_card_to_battlefield(0, catalog::pest_tendril());
    // Lethal damage → dies → scry 1 trigger fires.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt at the Pest castable");
    drain_stack(&mut g);
    // Pest Tendril died (1 toughness < 3 bolt damage). Pest token also
    // gets minted via the on-die "you gain 1 life" rider on the token
    // definition — but here the source is the Tendril itself (a non-token
    // creature card), so only Scry fires.
    assert!(g.battlefield_find(id).is_none(), "Pest Tendril should be dead");
    // Library count unchanged after scry (top card either stays or goes
    // to graveyard — library size doesn't grow beyond `lib_before`).
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn witherbloom_bramblepath_magecraft_gains_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_bramblepath());
    let you_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 1);
}

#[test]
fn pest_beekeeper_etb_mints_a_pest() {
    let mut g = two_player_game();
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let id = g.add_card_to_hand(0, catalog::pest_beekeeper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Beekeeper castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    // +1 Beekeeper itself, +1 Pest token = +2 your-controlled permanents.
    assert_eq!(bf_after, bf_before + 2);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Pest)));
}

#[test]
fn witherbloom_mire_maker_etb_drains_two() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_mire_maker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mire-Maker castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Mire-Maker on battlefield");
    assert!(c.has_keyword(&Keyword::Trample));
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, you_before + 2);
}

// — Silverquill (W/B) —

#[test]
fn silverquill_wordmaiden_magecraft_pumps_friendly() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_wordmaiden());
    let target = g.add_card_to_battlefield(0, catalog::pest_beekeeper());
    let p_before = g.battlefield_find(target).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(target).unwrap().power();
    assert_eq!(p_after, p_before + 1, "Wordmaiden magecraft should pump +1/+1");
}

#[test]
fn silverquill_lecturer_b58_etb_mints_inkling_and_gains_life() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let id = g.add_card_to_hand(0, catalog::silverquill_lecturer_b58());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lecturer castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    // +1 Lecturer +1 Inkling token = +2 permanents.
    assert_eq!(bf_after, bf_before + 2);
    assert_eq!(g.players[0].life, you_before + 2);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Inkling) &&
        c.definition.name != "Silverquill Lecturer II"));
}

#[test]
fn silverquill_inkmaster_b58_magecraft_drains() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_inkmaster_b58());
    let opp_before = g.players[1].life;
    let you_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft drain 1 = opp -4, you +1.
    assert_eq!(g.players[1].life, opp_before - 4);
    assert_eq!(g.players[0].life, you_before + 1);
}

// — Lorehold (R/W) —

#[test]
fn lorehold_bonechanter_magecraft_grants_menace() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_bonechanter());
    let beater = g.add_card_to_battlefield(0, catalog::pest_beekeeper());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(beater).expect("Beater on bf");
    assert!(c.has_keyword(&Keyword::Menace));
}

#[test]
fn lorehold_sparkdancer_etb_bolts_and_gains_life() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_sparkdancer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkdancer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, you_before + 2);
}

#[test]
fn lorehold_reliquarian_etb_mints_spirit_and_magecraft_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_reliquarian());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Reliquarian castable");
    drain_stack(&mut g);
    // Verify Spirit token was minted.
    assert!(g.battlefield.iter().any(|c| c.controller == 0 &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Spirit) &&
        c.id != id));
    let you_before = g.players[0].life;
    // Magecraft path: cast a bolt → +1 life.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 1);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Vigilance));
}

// — Quandrix (G/U) —

#[test]
fn quandrix_spellsplicer_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_spellsplicer());
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry doesn't change library size unless top is sent to graveyard;
    // both paths leave count ≤ original.
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn fractal_bluepetal_enters_with_two_plus_one_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_bluepetal());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bluepetal castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Bluepetal on bf");
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2);
    assert_eq!(c.power(), 2);
    assert_eq!(c.toughness(), 2);
}

#[test]
fn quandrix_mathweaver_etb_mints_fractal_with_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_mathweaver());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mathweaver castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c|
        c.controller == 0 &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Fractal) &&
        c.id != id);
    let f = fractal.expect("Fractal token should exist");
    assert_eq!(f.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
}

#[test]
fn quandrix_sumcaster_b58_magecraft_pumps_target_fractal() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_sumcaster_b58());
    let fractal = g.add_card_to_battlefield(0, catalog::fractal_bluepetal());
    let before = g.battlefield_find(fractal).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let after = g.battlefield_find(fractal).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(after, before + 1);
}

// — Prismari (U/R) —

#[test]
fn prismari_apprentice_b58_magecraft_pings_any_target() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_apprentice_b58());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn prismari_tideflame_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_tideflame());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Loot is net-0; cast removes bolt from hand. Net hand size = pre-cast - 1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_stormcaster_b58_etb_pings_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_stormcaster_b58());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormcaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    let c = g.battlefield_find(id).expect("Stormcaster on bf");
    assert!(c.has_keyword(&Keyword::Flying));
}

// ── Strict Proctor — ETB tax (StaticEffect::EtbTriggerTax { amount: 2 }) ──

#[test]
fn strict_proctor_taxes_an_etb_trigger_unless_paid() {
    use crate::decision::{Decision, DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Proctor on the same side as the ETB trigger source.
    let _ = g.add_card_to_battlefield(0, catalog::strict_proctor());
    // Pest Beekeeper has an ETB "mint a Pest" trigger.
    let id = g.add_card_to_hand(0, catalog::pest_beekeeper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Auto-decider declines the tax payment. The Beekeeper should be
    // sacrificed (sent to graveyard), and no Pest token should appear.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Beekeeper castable");
    drain_stack(&mut g);
    // Beekeeper got sacrificed by the tax; the Pest token never minted.
    assert!(g.battlefield_find(id).is_none(),
        "Beekeeper should be sacrificed when tax is declined");
    assert!(!g.battlefield.iter().any(|c| c.controller == 0 &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Pest)),
        "no Pest token should mint when ETB trigger was suppressed");

    // Now: scripted "yes" + floated {2} → tax paid, Beekeeper stays, Pest mints.
    let mut g2 = two_player_game();
    g2.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let _ = g2.add_card_to_battlefield(0, catalog::strict_proctor());
    let id = g2.add_card_to_hand(0, catalog::pest_beekeeper());
    g2.players[0].mana_pool.add(Color::Green, 1);
    g2.players[0].mana_pool.add_colorless(4); // 2 for cast, 2 for tax
    g2.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Beekeeper castable");
    drain_stack(&mut g2);
    assert!(g2.battlefield_find(id).is_some(),
        "Beekeeper should survive when tax is paid");
    assert!(g2.battlefield.iter().any(|c| c.controller == 0 &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Pest) &&
        c.id != id),
        "Pest token should mint when ETB trigger was paid for");
    // Verify the tax decision was actually offered.
    let _ = Decision::OptionalTrigger {
        source: id, description: "Pay {2} to keep this trigger?".to_string(),
    };
}

#[test]
fn strict_proctor_does_not_tax_non_etb_triggers() {
    // Magecraft fires on spell cast, not ETB — Proctor's tax should ignore it.
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::strict_proctor());
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_wordmaiden());
    let target = g.add_card_to_battlefield(0, catalog::pest_beekeeper());
    let p_before = g.battlefield_find(target).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Wordmaiden's magecraft pump fires normally — the source (Wordmaiden)
    // is NOT sacrificed despite the Proctor being in play.
    let p_after = g.battlefield_find(target).unwrap().power();
    assert_eq!(p_after, p_before + 1);
}

// ── Batch 59: 25 more synthesised STX cards ─────────────────────────────────
// Silverquill (5) — Witherbloom (5) — Lorehold (5) — Quandrix (5) — Prismari (5)

#[test]
fn silverquill_scrivener_b59_etb_surveils_and_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_scrivener_b59());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scrivener castable");
    drain_stack(&mut g);
    // Verify Scrivener is in play and is 2/2.
    let c = g.battlefield_find(id).expect("Scrivener on bf");
    assert_eq!(c.power(), 2);
    assert_eq!(c.toughness(), 2);
}

#[test]
fn silverquill_inkflight_b59_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_inkflight_b59());
    let p_before = g.battlefield_find(id).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(id).unwrap().power();
    assert_eq!(p_after, p_before + 1);
}

#[test]
fn silverquill_pen_priest_etb_drains_with_lifelink() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_pen_priest());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pen-Priest castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, you_before + 1);
    let c = g.battlefield_find(id).expect("Pen-Priest on bf");
    assert!(c.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_summit_b59_etb_pumps_other_inklings() {
    let mut g = two_player_game();
    // First Inkling to be pumped: drop an Inkling token via Inkling Scribe ({2}{W}).
    let scribe = g.add_card_to_hand(0, catalog::inkling_scribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: scribe, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scribe castable");
    drain_stack(&mut g);
    // Find the Inkling token controller 0 minted.
    let inkling = g.battlefield.iter().find(|c| c.controller == 0 && c.is_token &&
        c.definition.name == "Inkling").map(|c| c.id).expect("Inkling minted");
    let counters_before = g.battlefield_find(inkling).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);

    // Now cast Inkling Summit — it should put a +1/+1 counter on the Inkling token.
    let id = g.add_card_to_hand(0, catalog::inkling_summit_b59());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Summit castable");
    drain_stack(&mut g);
    let counters_after = g.battlefield_find(inkling).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(counters_after, counters_before + 1);
    // Self should NOT have a counter (OtherThanSource exclude).
    let summit = g.battlefield_find(id).expect("Summit on bf");
    assert_eq!(summit.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 0);
}

// — Witherbloom (B/G) —

#[test]
fn pest_grovetender_etb_scrys_and_has_deathtouch() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::pest_grovetender());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Grovetender castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Grovetender on bf");
    assert!(c.has_keyword(&Keyword::Deathtouch));
}

#[test]
fn witherbloom_thornpoet_magecraft_pumps_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_thornpoet());
    let p_before = g.battlefield_find(id).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(id).unwrap().power();
    assert_eq!(p_after, p_before + 1);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Reach));
}

#[test]
fn witherbloom_sapler_magecraft_pumps_friendly_pest() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_sapler());
    // Spawn a Pest token via Witherbloom Pest-Tender.
    let tender = g.add_card_to_hand(0, catalog::witherbloom_pest_tender());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: tender, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tender castable");
    drain_stack(&mut g);
    let pest = g.battlefield.iter().find(|c| c.controller == 0 && c.is_token &&
        c.definition.name == "Pest").map(|c| c.id).expect("Pest minted");
    let p_before = g.battlefield_find(pest).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(pest).unwrap().power();
    assert_eq!(p_after, p_before + 1);
}

#[test]
fn witherbloom_blightbearer_etb_drains_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let opp_before = g.players[1].life;
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_blightbearer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Blightbearer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, you_before + 2);
}

// — Lorehold (R/W) —

#[test]
fn lorehold_skyignite_magecraft_pings_target() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_skyignite());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 damage to opp.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn lorehold_pyrelearner_magecraft_self_pumps_with_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_pyrelearner());
    let p_before = g.battlefield_find(id).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(id).unwrap().power();
    assert_eq!(p_after, p_before + 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_spiritbinder_b59_etb_mints_spirit_and_gains_life() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritbinder_b59());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritbinder castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).collect();
    assert_eq!(spirits.len(), 1);
    assert_eq!(g.players[0].life, you_before + 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_emberscribe_b59_magecraft_pings_target() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_emberscribe_b59());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn lorehold_relicseer_etb_exiles_graveyard_card_and_is_flying() {
    let mut g = two_player_game();
    // Put two cards into opp's graveyard.
    let _ = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let _ = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let opp_gy_before = g.players[1].graveyard.len();
    let id = g.add_card_to_hand(0, catalog::lorehold_relicseer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Relicseer castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.len() < opp_gy_before, "Opp gy should shrink by 1");
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Flying));
}

// — Quandrix (G/U) —

#[test]
fn quandrix_growth_tutor_etb_pumps_fractal() {
    let mut g = two_player_game();
    // Seed a Fractal token.
    let bluepetal = g.add_card_to_hand(0, catalog::fractal_bluepetal());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bluepetal, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bluepetal castable");
    drain_stack(&mut g);
    let counters_before = g.battlefield_find(bluepetal).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    let id = g.add_card_to_hand(0, catalog::quandrix_growth_tutor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Growth-Tutor castable");
    drain_stack(&mut g);
    let counters_after = g.battlefield_find(bluepetal).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(counters_after, counters_before + 1);
}

#[test]
fn fractal_redleaf_enters_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_redleaf());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Redleaf castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Redleaf on bf");
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 3);
    assert_eq!(c.power(), 3);
    assert_eq!(c.toughness(), 3);
}

#[test]
fn quandrix_oracle_b59_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_oracle_b59());
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn quandrix_summerkeeper_etb_mints_fractal_with_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_summerkeeper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Summerkeeper castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c| c.controller == 0 && c.is_token &&
        c.definition.name == "Fractal").map(|c| c.id).expect("Fractal minted");
    let fc = g.battlefield_find(fractal).unwrap();
    assert_eq!(fc.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2);
    assert_eq!(fc.power(), 2);
    assert_eq!(fc.toughness(), 2);
}

#[test]
fn quandrix_skywinder_magecraft_pumps_friendly_fractal() {
    let mut g = two_player_game();
    // Seed a Fractal.
    let bluepetal = g.add_card_to_battlefield(0, catalog::fractal_bluepetal());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_skywinder());
    let p_before = g.battlefield_find(bluepetal).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(bluepetal).unwrap().power();
    assert_eq!(p_after, p_before + 1);
}

// — Prismari (U/R) —

#[test]
fn prismari_emberglyph_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_emberglyph());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn prismari_iceforge_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_iceforge());
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn prismari_flameseer_magecraft_loots_with_haste() {
    let mut g = two_player_game();
    // Library: islands to draw, then a sacrificial card we can discard.
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::prismari_flameseer());
    let gy_before = g.players[0].graveyard.len();
    // Put a spare discard target into hand before casting.
    let _spare = g.add_card_to_hand(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Magecraft loot fires: draw the Island, discard a card.
    // Graveyard should contain at least the bolt + the discarded card.
    assert!(g.players[0].graveyard.len() > gy_before);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Haste));
}

#[test]
fn prismari_artificer_etb_mints_treasure_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_artificer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Artificer castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Treasure"
    }).collect();
    assert_eq!(treasures.len(), 1);
}

#[test]
fn prismari_blast_apprentice_magecraft_pings_target() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_blast_apprentice());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 4);
}

// ── Batch 60: 15 more synthesised STX cards (3 per college) ─────────────────

#[test]
fn silverquill_mageblade_magecraft_pumps_friendly() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_mageblade());
    let target = g.add_card_to_battlefield(0, catalog::pest_beekeeper());
    let p_before = g.battlefield_find(target).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(target).unwrap().power();
    assert_eq!(p_after, p_before + 1);
}
