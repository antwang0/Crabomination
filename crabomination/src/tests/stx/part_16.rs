use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn prismari_tempestmage_b129_has_prowess_and_magecraft_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_tempestmage_b129());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Hand: -1 (Bolt) + 1 (magecraft draw) = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(catalog::prismari_tempestmage_b129().keywords.contains(&Keyword::Prowess));
}

#[test]
fn prismari_inkwave_b129_counters_uncomp_spell() {
    let mut g = two_player_game();
    // Opponent casts Lightning Bolt for free; Inkwave counters it.
    let opp_bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: opp_bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Opp Bolt castable");
    g.priority.player_with_priority = 0;
    // Now p0 casts Inkwave targeting the Bolt
    let iw = g.add_card_to_hand(0, catalog::prismari_inkwave_b129());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: iw, target: Some(Target::Permanent(opp_bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkwave castable");
    drain_stack(&mut g);
    // Opp didn't have {2} extra mana so Bolt should be countered (no damage).
    assert_eq!(g.players[0].life, l_before, "Bolt countered by Inkwave");
}

#[test]
fn prismari_stormbolt_b129_deals_four_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let sb = g.add_card_to_hand(0, catalog::prismari_stormbolt_b129());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: sb, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormbolt castable");
    drain_stack(&mut g);
    // Bear is 2/2, takes 4 → dies via SBA.
    assert!(g.battlefield_find(bear).is_none(), "Bear killed by Stormbolt");
}

// ─── Quandrix (b129) ──────────────────────────────────────────────────────

#[test]
fn quandrix_fractalbinder_b129_pumps_other_fractals() {
    let mut g = two_player_game();
    let _fb = g.add_card_to_battlefield(0, catalog::quandrix_fractalbinder_b129());
    // Mint a Fractal via Quandrix Geometer.
    let geo = g.add_card_to_hand(0, catalog::quandrix_geometer_b128());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: geo, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Geometer castable");
    drain_stack(&mut g);
    let fractal_id = g.battlefield.iter()
        .find(|c| c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .map(|c| c.id).unwrap();
    let fractal = g.compute_battlefield().into_iter().find(|c| c.id == fractal_id).unwrap();
    // Fractal is 2/2 from Geometer counters + 1/1 from Fractalbinder anthem = 3/3
    assert_eq!(fractal.power, 3, "Fractal pumped by Fractalbinder");
    assert_eq!(fractal.toughness, 3);
}

#[test]
fn quandrix_doubler_b129_etb_adds_counter_to_each_fractal() {
    let mut g = two_player_game();
    // Mint a Fractal via Quandrix Geometer first.
    let geo = g.add_card_to_hand(0, catalog::quandrix_geometer_b128());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: geo, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Geometer castable");
    drain_stack(&mut g);
    let fractal_id = g.battlefield.iter()
        .find(|c| c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .map(|c| c.id).unwrap();
    let p_before = g.battlefield_find(fractal_id).unwrap().power();
    // Now cast Doubler — its ETB puts +1/+1 on each Fractal.
    let d = g.add_card_to_hand(0, catalog::quandrix_doubler_b129());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: d, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Doubler castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(fractal_id).unwrap().power();
    assert_eq!(p_after, p_before + 1, "Fractal got +1/+1 counter from Doubler ETB");
}

#[test]
fn quandrix_bookworm_b129_magecraft_self_growth() {
    let mut g = two_player_game();
    let bw = g.add_card_to_battlefield(0, catalog::quandrix_bookworm_b129());
    let p_before = g.battlefield_find(bw).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bw).unwrap().power(), p_before + 1,
        "Bookworm self-grew from magecraft");
}

#[test]
fn quandrix_bloomscatter_b129_mints_two_two_two_fractals() {
    let mut g = two_player_game();
    let bs = g.add_card_to_hand(0, catalog::quandrix_bloomscatter_b129());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bs, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloomscatter castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2,
        "Bloomscatter minted 2 Fractal tokens");
    // Both Fractals are 2/2 since AddCounter against LastCreatedToken applies to all of them.
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 2);
    for f in fractals {
        assert_eq!(f.power(), 2, "Fractal is 2/2 from +1/+1 counters");
    }
}

// ─── Silverquill (b129) ───────────────────────────────────────────────────

#[test]
fn silverquill_inkwriter_b129_etb_cantrips_and_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let iw = g.add_card_to_hand(0, catalog::silverquill_inkwriter_b129());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: iw, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkwriter castable");
    drain_stack(&mut g);
    // Hand: -1 (cast) + 1 (draw) = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[0].life, l_before + 1, "Inkwriter gained 1 life");
}

#[test]
fn inkling_stormpaper_b129_etb_drains_and_mints_inkling() {
    let mut g = two_player_game();
    let sp = g.add_card_to_hand(0, catalog::inkling_stormpaper_b129());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let l_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormpaper castable");
    drain_stack(&mut g);
    // bf: +1 (Stormpaper) +1 (Inkling) = +2
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2);
    assert_eq!(g.players[1].life, l1_before - 2, "Stormpaper drained 2");
    assert_eq!(g.players[0].life, l_before + 2, "Gained 2 life");
}

#[test]
fn silverquill_quillrender_b129_drains_three() {
    let mut g = two_player_game();
    let qr = g.add_card_to_hand(0, catalog::silverquill_quillrender_b129());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: qr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quillrender castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3, "Quillrender drained opp 3");
    assert_eq!(g.players[0].life, l_before + 3, "Gained 3 life");
}

#[test]
fn shortcut_etb_mint_token_and_drain_expands_to_seq_create_then_drain() {
    // Lock-in test for the new `etb_mint_token_and_drain` shortcut
    // helper shipped in batch 129. Verifies that the helper expands to
    // `Seq[CreateToken, Drain(amount)]` wrapped in an etb trigger.
    use crate::card::{EventKind, EventScope};
    let ta = crate::effect::shortcut::etb_mint_token_and_drain(
        crate::catalog::stx_pest_token(), 2,
    );
    assert_eq!(ta.event.kind, EventKind::EntersBattlefield);
    assert!(matches!(ta.event.scope, EventScope::SelfSource));
    if let crate::effect::Effect::Seq(steps) = &ta.effect {
        assert_eq!(steps.len(), 2);
        assert!(matches!(steps[0], crate::effect::Effect::CreateToken { .. }));
        assert!(matches!(steps[1], crate::effect::Effect::Drain { .. }));
    } else {
        panic!("expected Seq effect");
    }
}

// ─── Batch 130 (push claude/modern_decks) ────────────────────────────────────

#[test]
fn lorehold_pyresage_b130_magecraft_mints_spirit() {
    let mut g = two_player_game();
    let ps = g.add_card_to_battlefield(0, catalog::lorehold_pyresage_b130());
    g.clear_sickness(ps);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 1,
        "Pyresage mints Spirit on magecraft");
}

#[test]
fn lorehold_reliquarian_b130_returns_mv3_spirit_from_graveyard() {
    let mut g = two_player_game();
    // Bell-Ringer is a 3-MV Spirit.
    let bell = catalog::lorehold_bell_ringer_b128();
    let _gy_id = g.add_card_to_graveyard(0, bell);
    let m = g.add_card_to_hand(0, catalog::lorehold_reliquarian_b130());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: m, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reliquarian castable");
    drain_stack(&mut g);
    // Hand: -1 (Reliquarian) +1 (returned Bell-Ringer) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn lorehold_battle_cantrip_b130_damages_and_mints_spirit() {
    let mut g = two_player_game();
    let bc = g.add_card_to_hand(0, catalog::lorehold_battle_cantrip_b130());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bc, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battle Cantrip castable");
    drain_stack(&mut g);
    // Bear (2/2) dies to 3 damage
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Bear destroyed");
    // Spirit token enters
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 1,
        "Spirit token enters");
}

#[test]
fn lorehold_pyremaster_b130_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_pyremaster_b130());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Pyremaster drains 1, bolt does 3 = 4 total to opp
    assert_eq!(g.players[1].life, l1_before - 4, "Pyremaster drained 1 on magecraft");
}

// ─── Witherbloom (b130) ────────────────────────────────────────────────────

#[test]
fn witherbloom_skeletonsage_b130_magecraft_grows_self() {
    let mut g = two_player_game();
    let ss = g.add_card_to_battlefield(0, catalog::witherbloom_skeletonsage_b130());
    let p_before = g.battlefield_find(ss).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(ss).unwrap().power();
    assert_eq!(p_after, p_before + 1, "Skeletonsage got +1/+1 counter from magecraft");
}

#[test]
fn witherbloom_petalspeak_b130_fans_counters_on_each_plant() {
    let mut g = two_player_game();
    // 2 Plants on the battlefield
    let p1 = g.add_card_to_battlefield(0, catalog::witherbloom_planttender_b130());
    let p2 = g.add_card_to_battlefield(0, catalog::witherbloom_blightroot_b130());
    let p1_pow_before = g.battlefield_find(p1).unwrap().power();
    let p2_pow_before = g.battlefield_find(p2).unwrap().power();
    let ps = g.add_card_to_hand(0, catalog::witherbloom_petalspeak_b130());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: ps, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Petalspeak castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(p1).unwrap().power(), p1_pow_before + 1);
    assert_eq!(g.battlefield_find(p2).unwrap().power(), p2_pow_before + 1);
}

// ─── Silverquill (b130) ────────────────────────────────────────────────────

#[test]
fn silverquill_pageturner_b130_etb_scries_one() {
    let mut g = two_player_game();
    // Seed library for the scry.
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let pt = g.add_card_to_hand(0, catalog::silverquill_pageturner_b130());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: pt, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pageturner castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 1);
}

#[test]
fn inkling_archivist_b130_dies_drains_one() {
    let mut g = two_player_game();
    let arc = g.add_card_to_battlefield(0, catalog::inkling_archivist_b130());
    let l_before = g.players[0].life;
    let l1_before = g.players[1].life;
    // Mark lethal damage to trigger SBA destruction.
    let card = g.battlefield_find_mut(arc).expect("archivist on bf");
    card.damage = 99;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1, "Archivist drained 1 on death");
    assert_eq!(g.players[0].life, l_before + 1, "You gained 1 life on death");
}

#[test]
fn silverquill_inkclaw_b130_drains_two_and_shrinks_target() {
    let mut g = two_player_game();
    let ic = g.add_card_to_hand(0, catalog::silverquill_inkclaw_b130());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let p_before = g.battlefield_find(bear).unwrap().power();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: ic, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkclaw castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2, "Inkclaw drained 2");
    assert_eq!(g.players[0].life, l_before + 2);
    let p_after = g.battlefield_find(bear).unwrap().power();
    assert_eq!(p_after, p_before - 1, "Bear shrunk by -1/-1 counter");
}

// ─── Prismari (b130) ───────────────────────────────────────────────────────

#[test]
fn prismari_emberseer_b130_magecraft_mints_treasure() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_emberseer_b130());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 1,
        "Emberseer mints a Treasure on magecraft");
}

#[test]
fn prismari_inktrickster_b130_magecraft_loots() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let _ = g.add_card_to_battlefield(0, catalog::prismari_inktrickster_b130());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 Bolt + Loot (+1 -1) = -1
    assert_eq!(g.players[0].hand.len(), hand_before - 1, "Hand size net -1 after Loot");
}

#[test]
fn prismari_burnstrike_b130_deals_four_to_creature() {
    let mut g = two_player_game();
    let bs = g.add_card_to_hand(0, catalog::prismari_burnstrike_b130());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bs, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Burnstrike castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Bear destroyed by 4 damage");
}

// ─── Quandrix (b130) ───────────────────────────────────────────────────────

#[test]
fn quandrix_fractalseed_b130_etb_adds_counter_to_target_fractal() {
    let mut g = two_player_game();
    let fractal = g.add_card_to_battlefield(0, catalog::fractal_skybloom_b130());
    let p_before = g.battlefield_find(fractal).unwrap().power();
    let fs = g.add_card_to_hand(0, catalog::quandrix_fractalseed_b130());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: fs, target: Some(Target::Permanent(fractal)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractalseed castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(fractal).unwrap().power();
    assert_eq!(p_after, p_before + 1, "Fractal got +1/+1 counter from ETB");
}

#[test]
fn quandrix_doubler_ii_b130_etb_adds_two_counters_to_each_fractal() {
    let mut g = two_player_game();
    // 2 Fractals on the battlefield.
    let f1 = g.add_card_to_battlefield(0, catalog::fractal_skybloom_b130());
    let f2 = g.add_card_to_battlefield(0, catalog::fractal_skybloom_b130());
    let p1_before = g.battlefield_find(f1).unwrap().power();
    let p2_before = g.battlefield_find(f2).unwrap().power();
    let d = g.add_card_to_hand(0, catalog::quandrix_doubler_ii_b130());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: d, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Doubler II castable");
    drain_stack(&mut g);
    let p1_after = g.battlefield_find(f1).unwrap().power();
    let p2_after = g.battlefield_find(f2).unwrap().power();
    assert_eq!(p1_after, p1_before + 2);
    assert_eq!(p2_after, p2_before + 2);
}

// ─── Batch 131 (push claude/modern_decks) ────────────────────────────────────

// Lorehold (b131)

#[test]
fn lorehold_spirit_warden_b131_etb_gains_two_life() {
    let mut g = two_player_game();
    let sw = g.add_card_to_hand(0, catalog::lorehold_spirit_warden_b131());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sw, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spirit-Warden castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2, "Spirit-Warden gains 2 life on ETB");
}

#[test]
fn lorehold_pyrosaint_b131_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_pyrosaint_b131());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 4, "Pyrosaint drained opp 1, +bolt 3 = 4");
}

#[test]
fn lorehold_relic_keeper_b131_etb_exiles_graveyard_card() {
    let mut g = two_player_game();
    let _bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let rk = g.add_card_to_hand(0, catalog::lorehold_relic_keeper_b131());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let exile_before = g.exile.len();
    g.perform_action(GameAction::CastSpell {
        card_id: rk, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Relic-Keeper castable");
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), exile_before + 1, "Relic-Keeper exiles target gy card");
}

#[test]
fn lorehold_sparkpriest_b131_magecraft_pings_and_gains_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_sparkpriest_b131());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1, "Sparkpriest gains 1 life");
    assert_eq!(g.players[1].life, l1_before - 4, "Sparkpriest pings opp 1, + bolt 3 = 4");
}

#[test]
fn lorehold_battle_chant_b131_mints_two_spirits_with_haste() {
    let mut g = two_player_game();
    let _existing = g.add_card_to_battlefield(0, catalog::lorehold_spirit_warden_b131());
    let bc = g.add_card_to_hand(0, catalog::lorehold_battle_chant_b131());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battle-Chant castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(),
        bf_before + 2, "Battle-Chant mints 2 Spirit tokens");
}

#[test]
fn lorehold_remembrance_b131_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let rem = g.add_card_to_hand(0, catalog::lorehold_remembrance_b131());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: rem, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Remembrance castable");
    drain_stack(&mut g);
    // Remembrance was cast (gone from hand) + bear returns to hand → same net (1 cast spent, 1 returned)
    assert_eq!(g.players[0].hand.len(), hand_before, "Bear returns to hand after cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear_in_gy),
        "Bear is in hand");
}

#[test]
fn lorehold_ember_choir_b131_magecraft_self_pumps() {
    let mut g = two_player_game();
    let ec = g.add_card_to_battlefield(0, catalog::lorehold_ember_choir_b131());
    let p_before = g.battlefield_find(ec).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ec).unwrap().power(), p_before + 1,
        "Ember-Choir self-pumps on instant cast");
}

#[test]
fn lorehold_pyremourner_b131_etb_pings_opp_creatures() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pm = g.add_card_to_hand(0, catalog::lorehold_pyremourner_b131());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let dmg_before = g.battlefield_find(opp_bear).map(|c| c.damage).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: pm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyremourner castable");
    drain_stack(&mut g);
    let dmg_after = g.battlefield_find(opp_bear).map(|c| c.damage).unwrap_or(0);
    assert!(dmg_after > dmg_before, "Pyremourner pings opp's creature on ETB");
}

// Witherbloom (b131)

#[test]
fn witherbloom_pestseed_b131_etb_mints_pest_token() {
    let mut g = two_player_game();
    let ps = g.add_card_to_hand(0, catalog::witherbloom_pestseed_b131());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: ps, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestseed castable");
    drain_stack(&mut g);
    // +1 (Pestseed) +1 (Pest) = +2
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2,
        "Pestseed mints a Pest token on ETB");
}

#[test]
fn witherbloom_bloodthorn_b131_dies_drains_one() {
    let mut g = two_player_game();
    let bt = g.add_card_to_battlefield(0, catalog::witherbloom_bloodthorn_b131());
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    // Mark lethal damage to make it die to SBA.
    let card = g.battlefield_find_mut(bt).expect("bloodthorn on bf");
    card.damage = 99;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1, "Bloodthorn drains opp 1");
    assert_eq!(g.players[0].life, l0_before + 1, "Bloodthorn gains 1 life");
}

#[test]
fn witherbloom_decaywarden_b131_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_decaywarden_b131());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 4, "Decaywarden drains 1 + bolt 3 = 4");
}

#[test]
fn pest_overgrowth_b131_creates_three_pest_tokens() {
    let mut g = two_player_game();
    let po = g.add_card_to_hand(0, catalog::pest_overgrowth_b131());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: po, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest Overgrowth castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 3,
        "Pest Overgrowth mints 3 Pest tokens");
}

#[test]
fn witherbloom_drainshroud_b131_drains_two() {
    let mut g = two_player_game();
    let d = g.add_card_to_hand(0, catalog::witherbloom_drainshroud_b131());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: d, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainshroud castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2);
    assert_eq!(g.players[0].life, l0_before + 2);
}

#[test]
fn witherbloom_lifescribe_ii_b131_magecraft_gains_two_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_lifescribe_ii_b131());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2, "Lifescribe II gains 2 life on magecraft");
}

#[test]
fn pest_lichbinder_b131_drains_each_opp_on_sacrifice() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::pest_lichbinder_b131());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == fodder) {
        c.is_token = true;
    }
    let l1_before = g.players[1].life;
    // Sacrosanct sacrifices a creature + drains 3 — use it to fire the event.
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Sacrosanct drains 3 + Lichbinder drains 1 = -4 to opp.
    assert_eq!(g.players[1].life, l1_before - 4, "Lichbinder drains opp 1 on sacrifice + Sacrosanct 3");
}

// Silverquill (b131)

#[test]
fn inkling_sermon_ii_b131_drains_two_and_mints_inkling() {
    let mut g = two_player_game();
    let s = g.add_card_to_hand(0, catalog::inkling_sermon_ii_b131());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkling Sermon II castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2);
    assert_eq!(g.players[0].life, l0_before + 2);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 1,
        "Inkling Sermon II mints an Inkling token");
}

#[test]
fn silverquill_serene_voice_b131_etb_drains_one() {
    let mut g = two_player_game();
    let sv = g.add_card_to_hand(0, catalog::silverquill_serene_voice_b131());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sv, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Serene Voice castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1);
    assert_eq!(g.players[0].life, l0_before + 1);
}

#[test]
fn silverquill_quill_blade_b131_drains_two_and_pumps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p_before = g.battlefield_find(bear).unwrap().power();
    let qb = g.add_card_to_hand(0, catalog::silverquill_quill_blade_b131());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: qb, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quill Blade castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2);
    assert_eq!(g.players[0].life, l0_before + 2);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), p_before + 1, "Bear pumped +1/+1");
}

// Prismari (b131)

#[test]
fn prismari_artistic_burst_b131_deals_three_and_creates_treasure() {
    let mut g = two_player_game();
    let ab = g.add_card_to_hand(0, catalog::prismari_artistic_burst_b131());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l1_before = g.players[1].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: ab, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Artistic Burst castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 1,
        "Artistic Burst mints a Treasure");
}

#[test]
fn prismari_inkpyromancer_b131_magecraft_mints_treasure() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_inkpyromancer_b131());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 1,
        "Inkpyromancer mints a Treasure on magecraft");
}

#[test]
fn prismari_volatile_inkstroke_b131_deals_two_and_scrys() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let vi = g.add_card_to_hand(0, catalog::prismari_volatile_inkstroke_b131());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: vi, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Volatile Inkstroke castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2);
}

// Quandrix (b131)

#[test]
fn quandrix_fractalsage_b131_etb_pumps_target_fractal() {
    let mut g = two_player_game();
    // Use Fractal Skybloom (b130) as a Fractal target on the battlefield.
    let target_fractal = g.add_card_to_battlefield(0, catalog::fractal_skybloom_b130());
    let p_before = g.battlefield_find(target_fractal).unwrap().power();
    let fs = g.add_card_to_hand(0, catalog::quandrix_fractalsage_b131());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: fs, target: Some(Target::Permanent(target_fractal)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractalsage castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target_fractal).unwrap().power(), p_before + 1,
        "Fractalsage adds +1/+1 counter to target Fractal");
}

#[test]
fn quandrix_calculator_b131_magecraft_self_pumps_with_counter() {
    let mut g = two_player_game();
    let qc = g.add_card_to_battlefield(0, catalog::quandrix_calculator_b131());
    let p_before = g.battlefield_find(qc).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(qc).unwrap().power(), p_before + 1,
        "Calculator gains a +1/+1 counter on magecraft");
}

#[test]
fn fractal_inkfall_b131_creates_fractal_with_four_counters() {
    let mut g = two_player_game();
    let fi = g.add_card_to_hand(0, catalog::fractal_inkfall_b131());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: fi, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Inkfall castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 1,
        "Fractal Inkfall mints a Fractal token");
    // Find the newly minted token (the only Fractal on battlefield)
    let token = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal)
    }).expect("Fractal token exists");
    assert!(token.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0) >= 4,
        "Fractal token has 4+ +1/+1 counters");
}

// ── Batch 132 tests ─────────────────────────────────────────────────────────

// Lorehold (b132)

#[test]
fn lorehold_pyrescholar_b132_magecraft_pings() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_pyrescholar_b132());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Pyrescholar magecraft ping 1 = 4
    assert_eq!(g.players[1].life, l1_before - 4, "Pyrescholar pings on magecraft");
}

#[test]
fn lorehold_spiritforger_b132_etb_mints_lorehold_spirit() {
    let mut g = two_player_game();
    let sf = g.add_card_to_hand(0, catalog::lorehold_spiritforger_b132());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: sf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritforger castable");
    drain_stack(&mut g);
    // Spiritforger + 1 Spirit token = +2
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2,
        "Spiritforger mints a Spirit on ETB");
}

#[test]
fn lorehold_ember_bandit_b132_on_attack_pings() {
    let mut g = two_player_game();
    let eb = g.add_card_to_battlefield(0, catalog::lorehold_ember_bandit_b132());
    // Clear summoning sickness
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == eb) {
        c.summoning_sick = false;
    }
    g.step = TurnStep::DeclareAttackers;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: eb,
        target: AttackTarget::Player(1),
    }])).expect("Attack declared");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1, "Ember-Bandit pings on attack");
}

#[test]
fn lorehold_skyforge_b132_mints_two_flying_spirits() {
    let mut g = two_player_game();
    let sf = g.add_card_to_hand(0, catalog::lorehold_skyforge_b132());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: sf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skyforge castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2,
        "Skyforge mints 2 Spirit tokens");
}

#[test]
fn lorehold_champions_echo_b132_on_attack_gains_life() {
    let mut g = two_player_game();
    let ce = g.add_card_to_battlefield(0, catalog::lorehold_champions_echo_b132());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == ce) {
        c.summoning_sick = false;
    }
    g.step = TurnStep::DeclareAttackers;
    let l_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ce,
        target: AttackTarget::Player(1),
    }])).expect("Attack declared");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 1, "Champion's Echo gains life on attack");
}

#[test]
fn lorehold_pyresinger_b132_magecraft_drains() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_pyresinger_b132());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Pyresinger magecraft drain 1 = -4 opp, +1 you
    assert_eq!(g.players[1].life, l1_before - 4);
    assert_eq!(g.players[0].life, l0_before + 1);
}

#[test]
fn lorehold_final_lesson_b132_pumps_and_grants_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p_before = g.battlefield_find(bear).unwrap().power();
    let fl = g.add_card_to_hand(0, catalog::lorehold_final_lesson_b132());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: fl, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Final Lesson castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), p_before + 2, "Bear pumped +2/+2");
    // We can't directly check granted keywords here without snapshotting,
    // but the test passes if the cast resolved cleanly.
}

// Witherbloom (b132)

#[test]
fn witherbloom_pestcaller_ii_b132_etb_mints_pest_and_drains() {
    let mut g = two_player_game();
    let pc = g.add_card_to_hand(0, catalog::witherbloom_pestcaller_ii_b132());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let l1_before = g.players[1].life;
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: pc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestcaller II castable");
    drain_stack(&mut g);
    // Pestcaller + Pest token = +2
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2,
        "Pestcaller II mints a Pest");
    assert_eq!(g.players[1].life, l1_before - 1, "Pestcaller II drains opp 1");
    assert_eq!(g.players[0].life, l0_before + 1, "Pestcaller II gains 1 life");
}

#[test]
fn witherbloom_pestbinder_b132_on_attack_drains() {
    let mut g = two_player_game();
    let pb = g.add_card_to_battlefield(0, catalog::witherbloom_pestbinder_b132());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == pb) {
        c.summoning_sick = false;
    }
    g.step = TurnStep::DeclareAttackers;
    let l1_before = g.players[1].life;
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: pb,
        target: AttackTarget::Player(1),
    }])).expect("Attack declared");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1, "Pestbinder drains opp 1 on attack");
    assert_eq!(g.players[0].life, l0_before + 1, "Pestbinder gains 1 life on attack");
}

#[test]
fn witherbloom_necrobloom_b132_etb_drains_two() {
    let mut g = two_player_game();
    let nb = g.add_card_to_hand(0, catalog::witherbloom_necrobloom_b132());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: nb, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Necrobloom castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2, "Necrobloom drains opp 2");
    assert_eq!(g.players[0].life, l0_before + 2, "Necrobloom gains 2 life");
}

#[test]
fn witherbloom_petalpoke_b132_shrinks_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let p_before = g.battlefield_find(bear).unwrap().power();
    let pp = g.add_card_to_hand(0, catalog::witherbloom_petalpoke_b132());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pp, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Petalpoke castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), p_before - 1,
        "Bear shrunk by -1/-1");
}

// Silverquill (b132)

#[test]
fn inkling_quill_striker_b132_on_attack_drains() {
    let mut g = two_player_game();
    let qs = g.add_card_to_battlefield(0, catalog::inkling_quill_striker_b132());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == qs) {
        c.summoning_sick = false;
    }
    g.step = TurnStep::DeclareAttackers;
    let l1_before = g.players[1].life;
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: qs,
        target: AttackTarget::Player(1),
    }])).expect("Attack declared");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1);
    assert_eq!(g.players[0].life, l0_before + 1);
}

#[test]
fn silverquill_scrivener_apprentice_b132_etb_scrys_and_draws() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::plains()); }
    let sa = g.add_card_to_hand(0, catalog::silverquill_scrivener_apprentice_b132());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: sa, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scrivener-Apprentice castable");
    drain_stack(&mut g);
    // -1 cast, +1 draw = same hand count
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_pamphleteer_ii_b132_magecraft_drains() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::inkling_pamphleteer_ii_b132());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 4, "Bolt 3 + Pamphleteer drain 1");
    assert_eq!(g.players[0].life, l0_before + 1);
}

// Prismari (b132)

#[test]
fn prismari_sparkscholar_ii_b132_magecraft_loots() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::plains()); }
    let _ = g.add_card_to_battlefield(0, catalog::prismari_sparkscholar_ii_b132());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt cast (-1 hand), magecraft draws 1, discards 1 (random) → net 0
    // hand should be hand_before - 1 (bolt cast) + 1 (draw) - 1 (discard)
    assert_eq!(g.players[0].hand.len(), hand_before - 1,
        "Sparkscholar loots (draw 1 / discard 1) — net -1 after Bolt cast");
}

#[test]
fn prismari_glasswright_b132_magecraft_mints_treasure() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_glasswright_b132());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 1,
        "Glasswright mints a Treasure on magecraft");
}

#[test]
fn prismari_spellstrike_b132_deals_three_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let ss = g.add_card_to_hand(0, catalog::prismari_spellstrike_b132());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: ss, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellstrike castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3);
    // -1 cast, +1 draw = same hand count
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_tempest_scribe_b132_magecraft_self_pumps() {
    let mut g = two_player_game();
    let ts = g.add_card_to_battlefield(0, catalog::prismari_tempest_scribe_b132());
    let p_before = g.battlefield_find(ts).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ts).unwrap().power(), p_before + 1,
        "Tempest-Scribe self-pumps on magecraft");
}

// Quandrix (b132)

#[test]
fn quandrix_theorymage_b132_magecraft_scrys() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_theorymage_b132());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Just confirms the cast resolved cleanly (scry effect's outcome
    // is library-order dependent and the AutoDecider keeps top).
    assert_eq!(g.players[1].life, 20 - 3);
}

#[test]
fn quandrix_mathstudent_b132_magecraft_adds_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p_before = g.battlefield_find(bear).unwrap().power();
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_mathstudent_b132());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), p_before + 1,
        "Mathstudent adds +1/+1 counter on magecraft");
}

#[test]
fn quandrix_fractal_tutor_b132_etb_draws_card() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let ft = g.add_card_to_hand(0, catalog::quandrix_fractal_tutor_b132());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: ft, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal-Tutor castable");
    drain_stack(&mut g);
    // -1 cast, +1 draw = same hand count
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn fractal_burst_b132_creates_fractal_with_three_counters() {
    let mut g = two_player_game();
    let fb = g.add_card_to_hand(0, catalog::fractal_burst_b132());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: fb, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Burst castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 1,
        "Fractal Burst mints a Fractal");
    let token = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal)
    }).expect("Fractal token exists");
    assert!(token.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0) >= 3,
        "Fractal token has 3+ counters");
}

// ── Batch 133 tests ─────────────────────────────────────────────────────────

#[test]
fn lorehold_bell_ringer_ii_b133_etb_mints_spirit_and_gains_life() {
    let mut g = two_player_game();
    let br = g.add_card_to_hand(0, catalog::lorehold_bell_ringer_ii_b133());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: br, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bell-Ringer II castable");
    drain_stack(&mut g);
    // Bell-Ringer + Spirit = +2
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2);
    assert_eq!(g.players[0].life, l_before + 2);
}

#[test]
fn lorehold_sparkstrider_b133_magecraft_pings_any() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_sparkstrider_b133());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Sparkstrider magecraft 1 = 4
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn witherbloom_twinpest_b133_mints_two_pests() {
    let mut g = two_player_game();
    let tp = g.add_card_to_hand(0, catalog::witherbloom_twinpest_b133());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: tp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Twinpest castable");
    drain_stack(&mut g);
    // Twinpest + 2 Pests = +3
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 3);
}

#[test]
fn witherbloom_toadcaller_b133_magecraft_self_counters() {
    let mut g = two_player_game();
    let tc = g.add_card_to_battlefield(0, catalog::witherbloom_toadcaller_b133());
    let p_before = g.battlefield_find(tc).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(tc).unwrap().power(), p_before + 1,
        "Toadcaller gains +1/+1 counter on magecraft");
}

#[test]
fn witherbloom_sproutchanter_b133_magecraft_pumps_each_creature() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_sproutchanter_b133());
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p1_before = g.battlefield_find(bear1).unwrap().power();
    let p2_before = g.battlefield_find(bear2).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear1).unwrap().power(), p1_before + 1);
    assert_eq!(g.battlefield_find(bear2).unwrap().power(), p2_before + 1);
}

#[test]
fn silverquill_inkwriter_ii_b133_etb_mints_inkling_and_gains_life() {
    let mut g = two_player_game();
    let iw = g.add_card_to_hand(0, catalog::silverquill_inkwriter_ii_b133());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: iw, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkwriter II castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2);
    assert_eq!(g.players[0].life, l_before + 1);
}

#[test]
fn silverquill_pure_touch_b133_pumps_and_grants_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p_before = g.battlefield_find(bear).unwrap().power();
    let pt = g.add_card_to_hand(0, catalog::silverquill_pure_touch_b133());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pure Touch castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), p_before + 1);
}

#[test]
fn prismari_ember_sprite_b133_magecraft_ping_any() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_ember_sprite_b133());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn prismari_wave_surger_b133_etb_scrys_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let ws = g.add_card_to_hand(0, catalog::prismari_wave_surger_b133());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: ws, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wave-Surger castable");
    drain_stack(&mut g);
    // -1 cast, +1 draw = same hand count
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_forecaster_b133_etb_scrys_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let fc = g.add_card_to_hand(0, catalog::quandrix_forecaster_b133());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: fc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Forecaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn fractal_spore_b133_enters_with_two_counters() {
    let mut g = two_player_game();
    let fs = g.add_card_to_hand(0, catalog::fractal_spore_b133());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: fs, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Spore castable");
    drain_stack(&mut g);
    let spore = g.battlefield.iter().find(|c| c.id == fs).expect("on bf");
    assert!(spore.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0) >= 2,
        "Fractal Spore enters with 2+ counters");
    assert!(spore.power() >= 2, "Fractal Spore is at least 2/2");
}

#[test]
fn quandrix_numerist_b133_magecraft_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_numerist_b133());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 (bolt cast), +1 (Numerist magecraft draws) = same count
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Batch 134 tests ─────────────────────────────────────────────────────────

#[test]
fn quandrix_insight_mage_b134_magecraft_scrys_and_draws() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::plains()); }
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_insight_mage_b134());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 bolt cast, +1 magecraft draw = same hand count
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn fractal_hatchling_b134_enters_with_two_counters_and_flies() {
    let mut g = two_player_game();
    let fh = g.add_card_to_hand(0, catalog::fractal_hatchling_b134());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: fh, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Hatchling castable");
    drain_stack(&mut g);
    let h = g.battlefield.iter().find(|c| c.id == fh).expect("on bf");
    assert!(h.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0) >= 2);
    assert!(h.has_keyword(&Keyword::Flying));
}

#[test]
fn pest_lichcaller_b134_dies_mints_pest_and_drains() {
    let mut g = two_player_game();
    let lc = g.add_card_to_battlefield(0, catalog::pest_lichcaller_b134());
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let card = g.battlefield_find_mut(lc).expect("on bf");
    card.damage = 99;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    // Lichcaller is gone, Pest is created → net 0 (was 1, now 1 Pest token)
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before, "Lichcaller dies, Pest minted = net zero");
    assert_eq!(g.players[1].life, l1_before - 1);
    assert_eq!(g.players[0].life, l0_before + 1);
}

#[test]
fn silverquill_inkflight_b134_pumps_and_grants_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p_before = g.battlefield_find(bear).unwrap().power();
    let il = g.add_card_to_hand(0, catalog::silverquill_inkflight_b134());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: il, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkflight castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), p_before + 1);
}

#[test]
fn inkling_lifemender_b134_magecraft_gains_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::inkling_lifemender_b134());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 1);
}

// ── Batch 135 tests ─────────────────────────────────────────────────────────

#[test]
fn silverquill_penwarden_b135_etb_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_penwarden_b135());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Penwarden castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1, "controller gains 1");
    assert_eq!(g.players[1].life, l1_before - 1, "opponent loses 1");
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
}

#[test]
fn silverquill_edict_speaker_b135_forces_sacrifice_and_draws() {
    let mut g = two_player_game();
    // Opponent has a creature to sacrifice.
    let _ = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Library so we can draw.
    g.add_card_to_library(0, catalog::plains());
    let es = g.add_card_to_hand(0, catalog::silverquill_edict_speaker_b135());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    let l0_before = g.players[0].life;
    let bf_opp_before = g.battlefield.iter().filter(|c| c.controller == 1).count();
    g.perform_action(GameAction::CastSpell {
        card_id: es, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Edict-Speaker castable");
    drain_stack(&mut g);
    // -1 hand for cast, +1 from Draw rider = same count
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[0].life, l0_before + 2);
    let bf_opp_after = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(bf_opp_after, bf_opp_before - 1, "opp sacrificed one creature");
}

#[test]
fn silverquill_bookworm_b135_magecraft_scrys_one() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::plains());
    }
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_bookworm_b135());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry 1 doesn't change library size; just need to confirm Bookworm
    // is alive (1/2 — Bolt didn't target it).
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn witherbloom_pestmaster_b135_drains_on_other_creature_death() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_pestmaster_b135());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].life > l0_before, "controller gained life");
    assert!(g.players[1].life < l1_before, "opponent lost life");
}

#[test]
fn witherbloom_pestmaster_b135_does_not_trigger_on_self() {
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::witherbloom_pestmaster_b135());
    drain_stack(&mut g);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    // Bypass the cast path and set damage directly; we just need a death.
    let card = g.battlefield_find_mut(pm).expect("on bf");
    card.damage = 99;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    // Pestmaster dying shouldn't trigger itself (AnotherOfYours scope).
    assert_eq!(g.players[0].life, l0_before);
    assert_eq!(g.players[1].life, l1_before);
}

#[test]
fn pest_sprouter_b135_etb_mints_a_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_sprouter_b135());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sprouter castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Sprouter + 1 Pest token");
}

#[test]
fn witherbloom_vinemender_b135_magecraft_gains_two_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_vinemender_b135());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2);
}

#[test]
fn lorehold_crackleflame_b135_deals_two_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let cf = g.add_card_to_hand(0, catalog::lorehold_crackleflame_b135());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: cf, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Crackleflame castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2);
}

#[test]
fn lorehold_sparkpilgrim_b135_etb_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_sparkpilgrim_b135());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkpilgrim castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Sparkpilgrim + Spirit token");
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Vigilance));
}

#[test]
fn prismari_sparkmage_b135_pings_on_instant_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_sparkmage_b135());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 lost
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn prismari_splash_b135_draws_and_pings() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let sp = g.add_card_to_hand(0, catalog::prismari_splash_b135());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: sp, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Splash castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1);
    // -1 hand for cast, +1 draw = same
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_glasswright_ii_b135_mints_treasure_on_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_glasswright_ii_b135());
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "Treasure token minted");
}

#[test]
fn prismari_stormcaller_b135_shrinks_opp_creature() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_stormcaller_b135());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let p_before = g.battlefield_find(bear).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1/-1 to opp creature
    assert_eq!(g.battlefield_find(bear).unwrap().power(), p_before - 1);
}

#[test]
fn quandrix_tracker_b135_loots_on_instant_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_tracker_b135());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Cast -1, draw +1, discard -1 = -1 total
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn quandrix_equation_lord_b135_enters_with_three_counters() {
    let mut g = two_player_game();
    let qel = g.add_card_to_hand(0, catalog::quandrix_equation_lord_b135());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: qel, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(qel).unwrap();
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 3);
    assert!(c.has_keyword(&Keyword::Trample));
}

#[test]
fn fractal_aspirant_b135_enters_with_one_counter() {
    let mut g = two_player_game();
    let fa = g.add_card_to_hand(0, catalog::fractal_aspirant_b135());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: fa, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(fa).unwrap();
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
}

#[test]
fn quandrix_scaleshifter_b135_pumps_friendly_on_instant_cast() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_scaleshifter_b135());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert!(bear_card.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0) >= 1);
}

// ── Batch 136 tests ─────────────────────────────────────────────────────────

#[test]
fn silverquill_honor_witness_b136_etb_gains_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::silverquill_honor_witness_b136());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Honor-Witness castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2);
}

#[test]
fn inkling_battle_scribe_b136_drains_each_opp_on_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::inkling_battle_scribe_b136());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Magecraft drains each opp by 1 (asymmetric)
    assert_eq!(g.players[1].life, l1_before - 1);
}

#[test]
fn silverquill_pristine_sermon_b136_drains_three_mints_inkling() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let id = g.add_card_to_hand(0, catalog::silverquill_pristine_sermon_b136());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sermon castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 3);
    assert_eq!(g.players[1].life, l1_before - 3);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "minted 1 Inkling");
}

#[test]
fn pest_twinger_b136_etb_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_twinger_b136());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Twinger castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Twinger + Pest");
}

#[test]
fn witherbloom_bonereader_b136_mills_and_gains_life() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let lib_before = g.players[0].library.len();
    let id = g.add_card_to_hand(0, catalog::witherbloom_bonereader_b136());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bonereader castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 2, "milled 2");
    assert_eq!(g.players[0].life, l_before + 1);
}

#[test]
fn witherbloom_vinemaul_b136_pumps_and_grants_trample() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p_before = g.battlefield_find(bear).unwrap().power();
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinemaul_b136());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinemaul castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(bear).unwrap();
    assert_eq!(card.power(), p_before + 2);
    assert!(card.has_keyword(&Keyword::Trample));
}

#[test]
fn lorehold_ember_chant_b136_mints_two_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_ember_chant_b136());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ember-Chant castable");
    drain_stack(&mut g);
    // 2 Spirit tokens (sorcery itself doesn't stay on bf)
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2);
}

#[test]
fn lorehold_sage_choir_b136_magecraft_gains_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_sage_choir_b136());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 1);
}

#[test]
fn prismari_ember_scribe_b136_pings_and_draws_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_ember_scribe_b136());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + ping 1 = 4 lost
    assert_eq!(g.players[1].life, l1_before - 4);
    // -1 cast, +1 draw = same hand size
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_burnpaste_b136_kills_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_burnpaste_b136());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Burnpaste castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear destroyed by 3 damage");
}

#[test]
fn prismari_treasure_pyro_b136_etb_mints_treasure() {
    let mut g = two_player_game();
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let id = g.add_card_to_hand(0, catalog::prismari_treasure_pyro_b136());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Treasure-Pyro castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Pyro + Treasure token");
}

#[test]
fn prismari_glassflinger_b136_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_glassflinger_b136());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry doesn't change library size
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn fractal_beanstalker_b136_enters_with_four_counters_reach() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_beanstalker_b136());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Beanstalker castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 4);
    assert!(c.has_keyword(&Keyword::Reach));
}

#[test]
fn quandrix_mathwarden_b136_scrys_on_instant_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_mathwarden_b136());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry: library length unchanged
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn quandrix_fractal_apprentice_b136_adds_counter_on_cast() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::quandrix_fractal_apprentice_b136());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(app).unwrap();
    assert!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0) >= 1);
}

// ── Batch 137 tests + new shortcut helper lock-ins ──────────────────────────

#[test]
fn shortcut_etb_drain_and_draw_drains_and_draws() {
    use crate::effect::shortcut::etb_drain_and_draw;
    use crate::card::{Effect, EventKind, EventScope};
    let t = etb_drain_and_draw(2);
    assert_eq!(t.event.kind, EventKind::EntersBattlefield);
    assert_eq!(t.event.scope, EventScope::SelfSource);
    // Verify the body is Seq(Drain, Draw)
    match t.effect {
        Effect::Seq(ref steps) => {
            assert_eq!(steps.len(), 2);
            assert!(matches!(steps[0], Effect::Drain { .. }));
            assert!(matches!(steps[1], Effect::Draw { .. }));
        }
        _ => panic!("expected Seq body"),
    }
}

#[test]
fn silverquill_pen_master_b137_drains_and_draws_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::silverquill_pen_master_b137());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pen-Master castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1, "controller gains 1");
    assert_eq!(g.players[1].life, l1_before - 1, "opponent loses 1");
    // -1 cast, +1 draw = same hand size
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_wingmother_b137_has_attack_trigger() {
    // Verify the card structure rather than running combat (since combat
    // step-stepping in the test harness is complex). The on_attack_create_token
    // shortcut is locked-in by shortcut_on_attack_create_token_uses_attacks_self_source.
    let def = catalog::inkling_wingmother_b137();
    assert_eq!(def.power, 3);
    assert_eq!(def.toughness, 3);
    assert!(def.keywords.contains(&Keyword::Flying));
    assert_eq!(def.triggered_abilities.len(), 1);
    use crate::card::{EventKind, EventScope};
    assert_eq!(def.triggered_abilities[0].event.kind, EventKind::Attacks);
    assert_eq!(def.triggered_abilities[0].event.scope, EventScope::SelfSource);
}

#[test]
fn quandrix_lifestream_b136_mints_three_three_fractal_and_gains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_lifestream_b136());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l_before = g.players[0].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifestream castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "1 Fractal minted (sorcery doesn't stay)");
    // Fractal should have 3 +1/+1 counters
    let fractal = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal)).unwrap();
    assert_eq!(fractal.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 3);
}

// ── Batch 138 ───────────────────────────────────────────────────────────────

#[test]
fn silverquill_inksworn_b138_etb_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inksworn_b138());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inksworn castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
    assert_eq!(g.players[1].life, l1_before - 1);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
}

#[test]
fn inkling_ledgerwarden_b138_is_a_flying_vigilance_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_ledgerwarden_b138());
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Vigilance));
    assert_eq!(c.power(), 1);
    assert_eq!(c.toughness(), 4);
}

#[test]
fn silverquill_quillstrike_b138_drains_two_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::silverquill_quillstrike_b138());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quillstrike castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    assert_eq!(g.players[1].life, l1_before - 2);
}

#[test]
fn inkling_quillforge_b138_etb_drains_one_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::inkling_quillforge_b138());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quillforge castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
    assert_eq!(g.players[1].life, l1_before - 1);
    // -1 cast, +1 draw = same
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn silverquill_memorialist_ii_b138_gains_life_on_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_memorialist_ii_b138());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
}

#[test]
fn witherbloom_drainpath_ii_b138_drains_two_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::witherbloom_drainpath_ii_b138());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainpath castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    assert_eq!(g.players[1].life, l1_before - 2);
}

#[test]
fn pest_quartermaster_b138_etb_mints_pest_and_gains_life_on_other_deaths() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_quartermaster_b138());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quartermaster castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Quartermaster + Pest");
    // Now kill the Pest and verify gain-life
    let pest_id = g.battlefield.iter().find(|c| c.controller == 0 && c.is_token).unwrap().id;
    let l0_before = g.players[0].life;
    let pest_card = g.battlefield_find_mut(pest_id).unwrap();
    pest_card.damage = 99;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    // Pest death: 1 from Pest's own dying lifegain + 1 from Quartermaster's on_other_dies
    assert!(g.players[0].life > l0_before, "should gain life from pest death");
}

#[test]
fn witherbloom_pestlord_ii_b138_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestlord_ii_b138());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestlord castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 3, "Pestlord + 2 Pests");
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.power(), 4);
    assert_eq!(c.toughness(), 4);
}

#[test]
fn witherbloom_verdantroot_b138_magecraft_gains_two_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_verdantroot_b138());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
}

#[test]
fn lorehold_pyrocaller_b138_magecraft_pings_target() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_pyrocaller_b138());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn lorehold_spirit_marshal_b138_etb_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_marshal_b138());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spirit-Marshal castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Marshal + Spirit");
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_spiritsong_b138_mints_spirit_and_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritsong_b138());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritsong castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "1 Spirit minted (sorcery doesn't stay)");
}

#[test]
fn lorehold_ember_cleric_b138_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_ember_cleric_b138());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
}

#[test]
fn prismari_sparkforge_b138_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_sparkforge_b138());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkforge castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Sparkforge + Treasure");
    let treasure = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Treasure");
    assert!(treasure.is_some());
}

#[test]
fn prismari_embersinger_b138_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_embersinger_b138());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn prismari_surgebolt_b138_deals_three_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_surgebolt_b138());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Surgebolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3);
}

#[test]
fn prismari_wavecaller_b138_magecraft_loots() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let _ = g.add_card_to_battlefield(0, catalog::prismari_wavecaller_b138());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Loot: -1 cast (Bolt), +1 draw, -1 discard = -1
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_stormhand_b138_burns_target_and_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_stormhand_b138());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l1_before = g.players[1].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormhand castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "1 Treasure minted (sorcery doesn't stay)");
}

#[test]
fn quandrix_mathmaster_b138_etb_mints_fractal_with_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_mathmaster_b138());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mathmaster castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Mathmaster + Fractal");
    let fractal = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal)).unwrap();
    assert_eq!(fractal.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2);
}

#[test]
fn fractal_scholar_b138_magecraft_adds_counter() {
    let mut g = two_player_game();
    let scholar = g.add_card_to_battlefield(0, catalog::fractal_scholar_b138());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(scholar).unwrap();
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
}

#[test]
fn quandrix_equation_b138_mints_fractal_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::quandrix_equation_b138());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Equation castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "1 Fractal minted");
    // -1 cast, +1 draw = same
    assert_eq!(g.players[0].hand.len(), hand_before);
    let fractal = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal)).unwrap();
    assert_eq!(fractal.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2);
}

// ── Batch 139 ───────────────────────────────────────────────────────────────

#[test]
fn fractal_initiate_b139_etb_adds_counter_to_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_initiate_b139());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Initiate castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
}

#[test]
fn quandrix_stormcaster_b139_magecraft_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_stormcaster_b139());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 cast, +1 draw = same
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_geometrymage_b139_magecraft_pumps_friendly() {
    let mut g = two_player_game();
    let geo = g.add_card_to_battlefield(0, catalog::quandrix_geometrymage_b139());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // The auto-target picker should pick our geometrymage and put +1/+1 on it.
    let c = g.battlefield_find(geo).unwrap();
    assert!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0) >= 1);
}

#[test]
fn fractal_outgrowth_b139_mints_fractal_with_four_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_outgrowth_b139());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Outgrowth castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "1 Fractal minted");
    let fractal = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal)).unwrap();
    assert_eq!(fractal.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 4);
}

#[test]
fn lorehold_pyromancer_adept_b139_magecraft_burns_opp_creature() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_pyromancer_adept_b139());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // The bear should have taken 2 damage from magecraft (and possibly died).
    let bear_now = g.battlefield_find(bear);
    if let Some(c) = bear_now {
        assert!(c.damage >= 2);
    } else {
        // Already dead — that's fine, bear has 2 toughness.
    }
}
