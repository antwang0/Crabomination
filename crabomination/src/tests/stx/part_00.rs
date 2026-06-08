use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;

// ─────────────────────────────────────────────────────────────────────────
// Batch 187 (modern_decks) — Silverquill keyword counter / Inkling tribal
// expansion + Witherbloom / Lorehold / Prismari / Quandrix additions.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_reachseal_b187_grants_reach_via_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_reachseal_b187());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.has_keyword(&Keyword::Reach),
        "CR 122.1b: reach counter grants Reach");
}

#[test]
fn silverquill_mentordrain_b187_magecraft_drains_one_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::silverquill_mentordrain_b187());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 4, "bolt 3 + drain 1");
    assert_eq!(g.players[0].life, p0_life + 1, "drain 1 to caster");
}

#[test]
fn inkling_vigilkeeper_b187_etb_grants_vigilance() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_vigilkeeper_b187());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter()
        .find(|c| c.definition.name == "Inkling Vigilkeeper (b187)")
        .expect("vigilkeeper on bf");
    assert!(c.has_keyword(&Keyword::Vigilance),
        "ETB vigilance counter wires through has_keyword");
}

#[test]
fn silverquill_skytutor_b187_tutors_low_mv_creature_on_etb() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed a low-MV creature to find.
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
    let id = g.add_card_to_hand(0, catalog::silverquill_skytutor_b187());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let bear_in_hand = g.players[0].hand.iter()
        .any(|c| c.definition.name == "Grizzly Bears");
    assert!(bear_in_hand, "low-MV bear tutored into hand");
}

#[test]
fn silverquill_inkletter_ii_b187_drains_two_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::silverquill_inkletter_ii_b187());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
    assert_eq!(g.players[0].life, p0_life + 2);
    // -1 cast + 1 draw = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn witherbloom_brewer_b187_etb_mints_pest_and_magecraft_pumps_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_brewer_b187());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let pest = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Pest");
    assert!(pest.is_some(), "ETB mints a Pest token");
    // Now cast an instant to fire magecraft pump.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let brewer = g.battlefield.iter()
        .find(|c| c.definition.name == "Witherbloom Brewer (b187)").expect("brewer on bf");
    // 2/2 + magecraft self pump (+1/+1 EOT)
    assert_eq!(brewer.power(), 3);
}

#[test]
fn witherbloom_toxinbloom_b187_shrinks_and_drains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxinbloom_b187());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Bear (2/2) gets -2/-2 → dies via SBA.
    assert!(g.battlefield_find(bear).is_none(), "bear dies to -2/-2");
    assert_eq!(g.players[1].life, p1_life - 1, "drain 1");
    assert_eq!(g.players[0].life, p0_life + 1, "you gain 1");
}

#[test]
fn witherbloom_hexblossom_b187_grants_deathtouch_and_mints_pest() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_hexblossom_b187());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.has_keyword(&Keyword::Deathtouch), "deathtouch counter grants Deathtouch");
    let pest = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Pest");
    assert!(pest.is_some(), "Pest token minted");
}

#[test]
fn witherbloom_lifeknotter_b187_drains_on_lifegain() {
    // Cast a lifegain-via-ETB card to fire the LifeGained event that drains.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_lifeknotter_b187());
    let id = g.add_card_to_hand(0, catalog::silverquill_loremender()); // ETB gain 2 life
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // ETB gain 2 life → fires LifeGained → Lifeknotter drain 1.
    assert_eq!(g.players[1].life, p1_life - 1, "drain triggered by lifegain event");
}

#[test]
fn pest_mauler_b187_attack_drains_on_combat_damage() {
    let mut g = two_player_game();
    let mauler = g.add_card_to_battlefield(0, catalog::pest_mauler_b187());
    g.clear_sickness(mauler);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: mauler, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let p1_life_before = g.players[1].life;
    let p0_life_before = g.players[0].life;
    while g.step != crate::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    // Mauler 2 power deals 2 combat damage + drain 1 to P1.
    assert_eq!(g.players[1].life, p1_life_before - 3);
    assert_eq!(g.players[0].life, p0_life_before + 1);
}

#[test]
fn witherbloom_grovecaller_b187_drains_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_grovecaller_b187());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 4, "bolt 3 + drain 1 = 4");
}

#[test]
fn witherbloom_soulreaper_b187_etb_drains_two_and_grows_engine() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_soulreaper_b187());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2, "ETB drain 2");
}

#[test]
fn lorehold_firstrikedoctrine_b187_grants_first_strike_via_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_firstrikedoctrine_b187());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::FirstStrike));
}

#[test]
fn lorehold_battleseer_b187_magecraft_pumps_friend() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_battleseer_b187());
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let pwr_before = g.battlefield_find(friend).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(friend).unwrap().power(), pwr_before + 1);
}

#[test]
fn lorehold_memorymage_b187_etb_returns_is_card_from_graveyard() {
    let mut g = two_player_game();
    let bolt_in_gy = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_memorymage_b187());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let in_hand = g.players[0].hand.iter().any(|c| c.id == bolt_in_gy);
    assert!(in_hand, "bolt returned to hand");
}

#[test]
fn lorehold_spiritcaller_b187_mints_spirit_on_other_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_spiritcaller_b187());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Spirit").count();
    // Bolt the fodder to fire CreatureDied trigger.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(fodder)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt fodder");
    drain_stack(&mut g);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Spirit").count();
    assert_eq!(tokens_after, tokens_before + 1, "spirit minted on fodder death");
}

#[test]
fn lorehold_pyrescribe_b187_pings_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::lorehold_pyrescribe_b187());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2, "2 damage to player");
}

#[test]
fn lorehold_ghostpaladin_b187_etb_taps_opp_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_ghostpaladin_b187());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp_bear).unwrap().tapped, "opp bear tapped");
}

#[test]
fn lorehold_reach_doctrine_b187_grants_reach_via_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_reach_doctrine_b187());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Reach));
}

#[test]
fn prismari_hasterune_b187_grants_haste_via_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_hasterune_b187());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Haste));
}

#[test]
fn prismari_sparkforge_b187_mints_treasure_and_scrys_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::prismari_sparkforge_b187());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let treasures_before = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure").count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let treasures_after = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure").count();
    assert_eq!(treasures_after, treasures_before + 1);
}

#[test]
fn prismari_flameseer_b187_pings_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_flameseer_b187());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // 3 bolt + 1 magecraft ping = 4 total.
    assert_eq!(g.players[1].life, p1_life - 4);
}

#[test]
fn prismari_stormcoach_b187_is_a_five_mana_flying_haste_dragon() {
    let def = catalog::prismari_stormcoach_b187();
    assert_eq!(def.cost.cmc(), 5);
    assert_eq!(def.power, 4);
    assert_eq!(def.toughness, 4);
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Haste));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Dragon));
}

#[test]
fn prismari_echohammer_b187_copies_target_is_spell() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let echo = g.add_card_to_hand(0, catalog::prismari_echohammer_b187());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt on stack");
    let bolt_target = g.stack.iter().find_map(|s| match s {
        StackItem::Spell { card, .. } if card.definition.name == "Lightning Bolt" => Some(card.id),
        _ => None,
    }).expect("bolt on stack");
    g.perform_action(GameAction::CastSpell {
        card_id: echo, target: Some(Target::Permanent(bolt_target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("echohammer on stack");
    drain_stack(&mut g);
    // Original bolt + 1 copy = 6 total damage.
    assert_eq!(g.players[1].life, p1_life - 6);
}

#[test]
fn prismari_pyroshaper_b187_pings_creature_for_three() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_pyroshaper_b187());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Bear dies to 3 damage.
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn prismari_stormcaller_b187_loots_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::prismari_stormcaller_b187());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // -1 cast +1 draw -1 discard = -1 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn quandrix_tramplerune_b187_grants_trample_via_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_tramplerune_b187());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Trample));
}

#[test]
fn quandrix_fractal_tutor_b187_mints_three_counter_flying_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_fractal_tutor_b187());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal").expect("fractal");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 3);
    assert!(fractal.has_keyword(&Keyword::Flying), "flying counter grants Flying");
}

#[test]
fn quandrix_vinescaler_b187_etb_grows_and_pumps_friend_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_vinescaler_b187());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let vinescaler = g.battlefield.iter()
        .find(|c| c.definition.name == "Quandrix Vinescaler (b187)").expect("vinescaler");
    assert_eq!(vinescaler.counter_count(CounterType::PlusOnePlusOne), 1, "ETB +1/+1 counter");
}

#[test]
fn quandrix_treestrider_b187_is_a_three_mana_reach_trampler() {
    let def = catalog::quandrix_treestrider_b187();
    assert_eq!(def.cost.cmc(), 3);
    assert_eq!(def.power, 3);
    assert_eq!(def.toughness, 3);
    assert!(def.keywords.contains(&Keyword::Reach));
    assert!(def.keywords.contains(&Keyword::Trample));
}

#[test]
fn quandrix_quickdraw_b187_counters_when_unable_to_pay() {
    let mut g = two_player_game();
    // P1 casts Lightning Bolt at instant speed with no extra mana for tax.
    let bolt_hand = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt_hand, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt on stack");
    // P0 quick-draws — bolt's controller has no mana left to pay {2}.
    g.priority.player_with_priority = 0;
    let qd = g.add_card_to_hand(0, catalog::quandrix_quickdraw_b187());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: qd, target: Some(Target::Permanent(bolt_hand)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("quickdraw on stack");
    drain_stack(&mut g);
    // Bolt countered → P0 untouched.
    assert_eq!(g.players[0].life, p0_life, "bolt was countered");
}

#[test]
fn quandrix_mossglider_b187_etb_grows_with_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_mossglider_b187());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter()
        .find(|c| c.definition.name == "Quandrix Mossglider (b187)").expect("mossglider");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
    // 2/3 + 1/+1 → 3/4.
    assert_eq!(c.power(), 3);
}

#[test]
fn quandrix_resonator_b187_magecraft_self_pumps() {
    let mut g = two_player_game();
    let resonator = g.add_card_to_battlefield(0, catalog::quandrix_resonator_b187());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let pwr_before = g.battlefield_find(resonator).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(resonator).unwrap().power(), pwr_before + 1);
}

#[test]
fn silverquill_wardlock_b187_fans_shield_counters_to_friendly_creatures() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_wardlock_b187());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(b1).unwrap().counter_count(CounterType::Shield), 1);
    assert_eq!(g.battlefield_find(b2).unwrap().counter_count(CounterType::Shield), 1);
    assert_eq!(g.battlefield_find(opp).unwrap().counter_count(CounterType::Shield), 0,
        "opp creatures not affected");
}

// ── Mono-color additions ────────────────────────────────────────────────────

#[test]
fn pop_quiz_draws_a_card_and_learns() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::pop_quiz());
    let hand_before = g.players[0].hand.len();

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pop Quiz castable for {2}{U}");
    drain_stack(&mut g);

    // -1 (cast) +1 (draw) +1 (Learn → Draw fallback, no sideboard) = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn mascot_exhibition_creates_three_distinct_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mascot_exhibition());
    g.players[0].mana_pool.add_colorless(7);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Mascot Exhibition castable for {7}");
    drain_stack(&mut g);

    let tokens: Vec<_> = g.battlefield.iter().filter(|c| c.is_token).collect();
    assert_eq!(tokens.len(), 3, "should mint exactly three tokens");
    let inkling = tokens.iter().find(|c| c.definition.name == "Inkling")
        .expect("2/1 Inkling flyer present");
    assert_eq!((inkling.power(), inkling.toughness()), (2, 1));
    assert!(inkling.has_keyword(&Keyword::Flying));
    let spirit = tokens.iter().find(|c| c.definition.name == "Spirit")
        .expect("3/2 Spirit present");
    assert_eq!((spirit.power(), spirit.toughness()), (3, 2));
    let elemental = tokens.iter().find(|c| c.definition.name == "Elemental")
        .expect("4/4 Elemental present");
    assert_eq!((elemental.power(), elemental.toughness()), (4, 4));
}

#[test]
fn plumb_the_forbidden_at_x_two_sacs_two_draws_two_loses_two() {
    let mut g = two_player_game();
    let _b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b3 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }

    let id = g.add_card_to_hand(0, catalog::plumb_the_forbidden());
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;
    let bf_creatures_before = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_creature())
        .count();

    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Plumb the Forbidden castable for {X=2}{B}{B}");
    drain_stack(&mut g);

    let bf_creatures_after = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_creature())
        .count();
    // Sacrificed 2 creatures.
    assert_eq!(bf_creatures_after, bf_creatures_before - 2,
        "two creatures sacrificed");
    // Hand: -1 (cast) +2 (draw) = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2);
    // Life: -2.
    assert_eq!(g.players[0].life, life_before - 2);
}

#[test]
fn owlin_shieldmage_is_a_warding_flyer() {
    use crate::card::WardCost;
    let c = catalog::owlin_shieldmage();
    assert_eq!(c.cost.cmc(), 5);
    assert_eq!((c.power, c.toughness), (3, 3));
    assert!(c.keywords.contains(&Keyword::Flying));
    assert!(c.keywords.contains(&Keyword::Ward(WardCost::Life(3))), "Ward—Pay 3 life");
}

#[test]
fn frost_trickster_taps_and_stuns_target_on_etb() {
    let mut g = two_player_game();
    // Untapped creature on opponent's battlefield.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let id = g.add_card_to_hand(0, catalog::frost_trickster());

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Frost Trickster castable for {2}{U}");
    drain_stack(&mut g);

    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(bear_card.tapped, "target should be tapped");
    assert_eq!(bear_card.counter_count(CounterType::Stun), 1,
        "target should have a stun counter");
}

#[test]
fn body_of_research_creates_fractal_with_counters_from_library() {
    let mut g = two_player_game();
    // Seed P0's library with 5 cards.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::body_of_research());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Body of Research castable for {4}{G}{U}");
    drain_stack(&mut g);

    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal")
        .expect("Fractal token present");
    // The Fractal should have +1/+1 counters equal to library size.
    let counters = fractal.counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, lib_before as u32,
        "Fractal +1/+1 counter count should equal library size before cast; got {}, expected {}",
        counters, lib_before);
    assert_eq!(fractal.power(), counters as i32);
    assert_eq!(fractal.toughness(), counters as i32);
}

#[test]
fn show_of_confidence_pumps_with_storm_count() {
    let mut g = two_player_game();
    // Cast a Lightning Bolt first to bump the storm counter, then Show of
    // Confidence — the spell should add `storm_count + 1` counters.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let soc = g.add_card_to_hand(0, catalog::show_of_confidence());

    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    g.perform_action(GameAction::CastSpell {
        card_id: soc, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Show of Confidence castable for {1}{W}");
    drain_stack(&mut g);

    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    let counters = bear_card.counter_count(CounterType::PlusOnePlusOne);
    // Storm count = 1 (Bolt) → Show of Confidence adds 1 + 1 = 2 counters.
    assert_eq!(counters, 2, "Should add storm_count + 1 = 2 counters");
}

#[test]
fn bury_in_books_returns_target_to_top_of_library() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::bury_in_books());

    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bury in Books castable for {3}{U}");
    drain_stack(&mut g);

    // Bear is off the battlefield and on top of P1's library.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    let top = g.players[1].library.last().expect("library not empty");
    assert_eq!(top.id, bear, "bear should be on top of P1's library");
}

#[test]
fn test_of_talents_counters_target_instant() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    // Bolt is on the stack; P0 responds.
    g.priority.player_with_priority = 0;
    let tot = g.add_card_to_hand(0, catalog::test_of_talents());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: tot, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Test of Talents castable for {1}{U}{U}");
    drain_stack(&mut g);

    // P0's life is unchanged — Bolt was countered.
    assert_eq!(g.players[0].life, 20, "Bolt should have been countered");
    // Bolt is in the graveyard.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

// ── Repartee plumbing ──────────────────────────────────────────────────────

#[test]
fn rehearsed_debater_pumps_when_instant_targets_creature() {
    // Repartee: cast Lightning Bolt targeting a creature → Debater +1/+1 EOT.
    let mut g = two_player_game();
    let debater = g.add_card_to_battlefield(0, catalog::rehearsed_debater());
    g.clear_sickness(debater);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let d = g.battlefield.iter().find(|c| c.id == debater).unwrap();
    assert_eq!(d.power(), 4, "Debater should be 4/4 from Repartee");
    assert_eq!(d.toughness(), 4);
}

#[test]
fn rehearsed_debater_does_not_pump_when_targeting_player() {
    // Repartee fires on instant/sorcery that targets a CREATURE — bolting
    // a player should NOT trigger.
    let mut g = two_player_game();
    let debater = g.add_card_to_battlefield(0, catalog::rehearsed_debater());
    g.clear_sickness(debater);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let d = g.battlefield.iter().find(|c| c.id == debater).unwrap();
    assert_eq!(d.power(), 3, "Debater should NOT be pumped (target was a player)");
    assert_eq!(d.toughness(), 3);
}

#[test]
fn lecturing_scornmage_gains_counter_on_creature_targeted_spell() {
    let mut g = two_player_game();
    let scorn = g.add_card_to_battlefield(0, catalog::lecturing_scornmage());
    g.clear_sickness(scorn);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let s = g.battlefield.iter().find(|c| c.id == scorn).unwrap();
    assert_eq!(
        s.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Scornmage should gain a +1/+1 counter from Repartee"
    );
}

#[test]
fn melancholic_poet_drains_on_creature_targeted_spell() {
    let mut g = two_player_game();
    let _poet = g.add_card_to_battlefield(0, catalog::melancholic_poet());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Bolt: 3 to bear (kills); Repartee: drain 1 (P1 -1, P0 +1).
    assert_eq!(g.players[0].life, 21, "P0 +1 from Repartee drain");
    assert_eq!(g.players[1].life, 19, "P1 -1 from Repartee drain");
}

#[test]
fn multiple_choice_mode_one_creates_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::multiple_choice());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("Multiple Choice castable for {1}{U}{U}");
    drain_stack(&mut g);

    // Mode 1 minted a 1/1 Pest token.
    let pest = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Pest")
        .expect("Pest token present");
    assert_eq!(pest.power(), 1);
    assert_eq!(pest.toughness(), 1);
}

// ── Lorehold (R/W) ──────────────────────────────────────────────────────────

#[test]
fn lorehold_apprentice_gains_life_on_instant_cast() {
    let mut g = two_player_game();
    let _app = g.add_card_to_battlefield(0, catalog::lorehold_apprentice());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Magecraft fires off the Bolt cast; Apprentice's lifegain rider trips.
    assert_eq!(g.players[0].life, life_before + 1,
        "Magecraft should grant +1 life on instant cast");
}

#[test]
fn lorehold_apprentice_does_not_gain_on_creature_spell() {
    // Magecraft only triggers on instant/sorcery, not creature spells.
    let mut g = two_player_game();
    let _app = g.add_card_to_battlefield(0, catalog::lorehold_apprentice());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bears castable for {1}{G}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before,
        "Casting a creature should NOT trigger Magecraft");
}

#[test]
fn pillardrop_rescuer_returns_target_instant_from_graveyard() {
    let mut g = two_player_game();
    // P0 has a Bolt in their graveyard.
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::pillardrop_rescuer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pillardrop Rescuer castable for {3}{R}{W}");
    drain_stack(&mut g);
    // Bolt should be back in P0's hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt should be returned to hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "Bolt should no longer be in graveyard");
}

#[test]
fn heated_debate_deals_4_damage_to_target_creature() {
    let mut g = two_player_game();
    // 4-toughness creature dies to Heated Debate's 4 damage.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::heated_debate());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Heated Debate castable for {2}{R}");
    drain_stack(&mut g);
    // Bear (2/2) takes 4 damage and dies → graveyard.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be off the battlefield");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "Bear should be in P1's graveyard");
}

#[test]
fn storm_kiln_artist_creates_treasure_and_deals_1_damage() {
    let mut g = two_player_game();
    let _ska = g.add_card_to_battlefield(0, catalog::storm_kiln_artist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Storm-Kiln Artist's Magecraft: 1 damage to opponent + Treasure token.
    // Bolt also dealt 3 damage so total is 4.
    assert_eq!(g.players[1].life, p1_life_before - 4,
        "P1 takes 3 (Bolt) + 1 (Magecraft) = 4 damage");
    let treasures = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1, "Storm-Kiln Artist should mint one Treasure");
}

// ── Quandrix (G/U) ──────────────────────────────────────────────────────────

#[test]
fn quandrix_apprentice_pumps_creature_you_control_on_instant_cast() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::quandrix_apprentice());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Magecraft auto-targets a creature you control. With the engine's
    // source-avoidance picker, the Apprentice (trigger source) is avoided
    // when another legal target exists — so the bear gets the pump.
    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        (bear_card.power(), bear_card.toughness()),
        (3, 3),
        "Source-avoidance picker should pump the bear, not the Apprentice",
    );
    let app_card = g.battlefield.iter().find(|c| c.id == app).unwrap();
    assert_eq!(
        (app_card.power(), app_card.toughness()),
        (2, 2),
        "Apprentice (trigger source) should not be the picked target",
    );
}

#[test]
fn quandrix_apprentice_falls_back_to_self_when_no_other_target() {
    // Source-avoidance falls back to the source when it's the only legal
    // pick — the trigger should still resolve, not fizzle.
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::quandrix_apprentice());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let app_card = g.battlefield.iter().find(|c| c.id == app).unwrap();
    assert_eq!(
        (app_card.power(), app_card.toughness()),
        (3, 3),
        "Apprentice pumps itself when it's the only legal Magecraft target",
    );
}

#[test]
fn quandrix_pledgemage_grows_via_activated_ability() {
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::quandrix_pledgemage());
    g.clear_sickness(pm);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pm, ability_index: 0, target: None, x_value: None })
    .expect("Quandrix Pledgemage activatable for {1}{G}{U}");
    drain_stack(&mut g);
    let pm_card = g.battlefield.iter().find(|c| c.id == pm).unwrap();
    assert_eq!(pm_card.counter_count(CounterType::PlusOnePlusOne), 1,
        "should gain 1 +1/+1 counter");
    assert_eq!(pm_card.power(), 3, "Pledgemage now 3/3");
    assert_eq!(pm_card.toughness(), 3);
}

#[test]
fn decisive_denial_counters_noncreature_unless_paid() {
    let mut g = two_player_game();
    // P1 casts a Bolt; P0 responds with Decisive Denial mode 0.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");
    g.priority.player_with_priority = 0;
    let dd = g.add_card_to_hand(0, catalog::decisive_denial());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: dd, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: Some(0), x_value: None,
    })
    .expect("Decisive Denial castable");
    drain_stack(&mut g);
    // Bolt countered (P1 had no extra mana for {2} kicker), P0 unhurt.
    assert_eq!(g.players[0].life, 20, "Bolt should be countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Bolt should be in graveyard");
}

// ── Prismari (U/R) ──────────────────────────────────────────────────────────

#[test]
fn prismari_apprentice_scrys_on_instant_cast() {
    let mut g = two_player_game();
    let _app = g.add_card_to_battlefield(0, catalog::prismari_apprentice());
    // Seed library so there's something to scry.
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Scry doesn't change library size (it just looks at the top card and
    // optionally moves it to bottom). Sanity: library is still seeded.
    assert_eq!(g.players[0].library.len(), lib_before,
        "Scry 1 should not change library size");
}

#[test]
fn symmetry_sage_pumps_self_and_grants_flying_on_instant_cast() {
    let mut g = two_player_game();
    let sage = g.add_card_to_battlefield(0, catalog::symmetry_sage());
    g.clear_sickness(sage);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    let s = g.battlefield.iter().find(|c| c.id == sage).unwrap();
    assert_eq!(s.power(), 1, "Sage 0/2 base +1/+0 Magecraft → 1/2");
    assert_eq!(s.toughness(), 2);
    assert!(s.has_keyword(&Keyword::Flying),
        "Magecraft should grant flying EOT");
}

// ── Witherbloom (B/G) ──────────────────────────────────────────────────────

#[test]
fn witherbloom_pledgemage_magecraft_gains_one_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_pledgemage());
    let life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "magecraft gains 1 life on IS cast");
}

#[test]
fn prismari_pledgemage_is_a_defender_that_pumps_on_magecraft() {
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::prismari_pledgemage());
    assert!(g.battlefield_find(pm).unwrap().has_keyword(&Keyword::Defender));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // 3/3 + magecraft +1/+1 → 4/4 EOT.
    assert_eq!(g.battlefield_find(pm).unwrap().power(), 4, "magecraft self-pump +1/+1");
}

#[test]
fn sparring_regimen_creates_a_2_2_spirit_token_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sparring_regimen());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Sparring Regimen castable for {2}{R}{W}");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 1, "should create one Spirit token");
    let s = spirits[0];
    assert_eq!(s.power(), 2);
    assert_eq!(s.toughness(), 2);
    assert!(s.definition.subtypes.creature_types
        .contains(&crate::card::CreatureType::Spirit),
        "should be a Spirit");
}

#[test]
fn pest_summoning_creates_two_pests() {
    // Real-text fix: was minting 1 Pest, now mints 2.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_summoning());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pest Summoning castable for {B}{G}");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 2, "Pest Summoning should mint two Pest tokens");
}

// ── New iconic STX cards ────────────────────────────────────────────────────

#[test]
fn sedgemoor_witch_magecraft_creates_pest_token() {
    let mut g = two_player_game();
    let witch = g.add_card_to_battlefield(0, catalog::sedgemoor_witch());
    g.clear_sickness(witch);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable");
    drain_stack(&mut g);

    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest")
        .collect();
    assert_eq!(pests.len(), 1, "Sedgemoor Witch should mint one Pest token on instant cast");
}

#[test]
fn mage_hunters_onslaught_destroys_creature_and_draws_card() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mage_hunters_onslaught());
    let bear_id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    // Prime the library so Draw 1 has a card to grab.
    g.add_card_to_library(0, catalog::island());

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear_id)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Mage Hunters' Onslaught castable for {2}{B}{B}");
    drain_stack(&mut g);

    // Bear should be in P1's graveyard.
    assert!(!g.battlefield.iter().any(|c| c.id == bear_id),
        "Grizzly Bears should be destroyed");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear_id),
        "Bear should be in P1's graveyard");
    // P0 should have drawn a card. (The Onslaught itself moved hand → stack
    // → graveyard, leaving hand_before - 1 + 1 (draw) = hand_before.)
    assert_eq!(g.players[0].hand.len(), hand_before);
    // The drawn card should be the Island we seeded.
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Island"),
        "P0 should have drawn the Island we seeded");
}

// ── STX legends (body-only smoke tests) ─────────────────────────────────────

#[test]
fn galazeth_prismari_grants_tap_for_any_color_to_artifacts() {
    // Printed: "Artifacts you control have '{T}: Add one mana of any
    // color.'" The static is surfaced as a virtual activated ability
    // at index = printed_count on each artifact controlled by
    // Galazeth's controller. Strixhaven Skycoach (artifact, 0 printed
    // activated abilities) gets the grant at index 0; tapping it adds
    // one mana of any color via the existing AnyOneColor decision
    // (AutoDecider picks the first legal color).
    let mut g = two_player_game();
    let _galazeth = g.add_card_to_battlefield(0, catalog::galazeth_prismari());
    let skycoach = g.add_card_to_battlefield(0, catalog::strixhaven_skycoach());

    let pool_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: skycoach,
        ability_index: 0,
        target: None, x_value: None })
    .expect("Galazeth grant: {T}: Add one mana of any color");

    let pool_after = g.players[0].mana_pool.total();
    assert_eq!(
        pool_after - pool_before, 1,
        "Galazeth-granted ability adds one mana to caster's pool"
    );

    // Verify the Skycoach is now tapped (paid the tap cost).
    let sc = g.battlefield_find(skycoach).expect("Skycoach still on bf");
    assert!(sc.tapped, "Skycoach paid the tap cost for the granted ability");
}

#[test]
fn galazeth_prismari_grant_requires_galazeth_in_play() {
    // Without Galazeth on the battlefield, an artifact has no virtual
    // tap-for-any-color ability — activating index 0 on a Skycoach
    // (0 printed abilities) is rejected as out-of-bounds.
    let mut g = two_player_game();
    let skycoach = g.add_card_to_battlefield(0, catalog::strixhaven_skycoach());

    let err = g
        .perform_action(GameAction::ActivateAbility {
            card_id: skycoach,
            ability_index: 0,
            target: None, x_value: None })
        .expect_err("no Galazeth → no grant → rejected");
    assert!(
        matches!(err, GameError::AbilityIndexOutOfBounds),
        "expected AbilityIndexOutOfBounds, got {err:?}"
    );
}

#[test]
fn lorehold_apprentice_magecraft_drains_one_to_opponent_and_gains_life() {
    let mut g = two_player_game();
    let apprentice = g.add_card_to_battlefield(0, catalog::lorehold_apprentice());
    g.clear_sickness(apprentice);
    // Cast a Lightning Bolt to trigger magecraft.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    let life_before = g.players[0].life;
    let opp_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    // Bolt itself does 3 to opp; magecraft adds 1 more.
    assert_eq!(g.players[0].life, life_before + 1,
        "Magecraft should gain you 1 life");
    assert_eq!(g.players[1].life, opp_life_before - 3 - 1,
        "Bolt (3) + magecraft damage (1) = 4 to opp");
}

#[test]
fn lorehold_pledgemage_gy_exile_cost_pumps_self() {
    let mut g = two_player_game();
    let pledge = g.add_card_to_battlefield(0, catalog::lorehold_pledgemage());
    g.clear_sickness(pledge);
    let _filler = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    let p_before = g.battlefield_find(pledge).unwrap().power();
    let t_before = g.battlefield_find(pledge).unwrap().toughness();
    g.perform_action(GameAction::ActivateAbility {
        card_id: pledge, ability_index: 0, target: None, x_value: None })
    .expect("Pledgemage activation with bolt in gy");
    drain_stack(&mut g);

    let p_after = g.battlefield_find(pledge).unwrap().power();
    let t_after = g.battlefield_find(pledge).unwrap().toughness();
    assert_eq!(p_after, p_before + 1);
    assert_eq!(t_after, t_before + 1);
    // The bolt was exiled from the graveyard.
    assert!(g.exile.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "Bolt should be in exile (paid as cost)");
    assert!(g.players[0].graveyard.iter().all(|c| c.definition.name != "Lightning Bolt"),
        "Bolt no longer in graveyard");
}

#[test]
fn lorehold_pledgemage_rejects_activation_with_empty_graveyard() {
    let mut g = two_player_game();
    let pledge = g.add_card_to_battlefield(0, catalog::lorehold_pledgemage());
    g.clear_sickness(pledge);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let pool_before = g.players[0].mana_pool.total();

    let r = g.perform_action(GameAction::ActivateAbility {
        card_id: pledge, ability_index: 0, target: None, x_value: None });
    assert!(r.is_err(),
        "Empty graveyard should reject the exile-other cost");
    assert_eq!(g.players[0].mana_pool.total(), pool_before,
        "Mana untouched on rejected activation");
}

#[test]
fn beledros_witherbloom_pay_ten_life_untaps_all_lands() {
    let mut g = two_player_game();
    let beledros = g.add_card_to_battlefield(0, catalog::beledros_witherbloom());
    g.clear_sickness(beledros);
    // Tap some lands.
    let l1 = g.add_card_to_battlefield(0, catalog::forest());
    let l2 = g.add_card_to_battlefield(0, catalog::swamp());
    g.battlefield_find_mut(l1).unwrap().tapped = true;
    g.battlefield_find_mut(l2).unwrap().tapped = true;

    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: beledros, ability_index: 0, target: None, x_value: None })
    .expect("Beledros activatable as sorcery");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 10, "Pay 10 life cost");
    assert!(!g.battlefield_find(l1).unwrap().tapped, "Forest untapped");
    assert!(!g.battlefield_find(l2).unwrap().tapped, "Swamp untapped");
}

#[test]
fn beledros_witherbloom_rejects_activation_with_insufficient_life() {
    let mut g = two_player_game();
    let beledros = g.add_card_to_battlefield(0, catalog::beledros_witherbloom());
    g.clear_sickness(beledros);
    g.players[0].life = 5; // not enough for the 10-life cost.

    let r = g.perform_action(GameAction::ActivateAbility {
        card_id: beledros, ability_index: 0, target: None, x_value: None });
    assert!(r.is_err(), "Activation rejected when life < 10");
    assert_eq!(g.players[0].life, 5, "Life unchanged on rejection");
}

#[test]
fn tanazir_quandrix_attack_trigger_doubles_target_toughness() {
    use crate::game::types::AttackTarget;
    let mut g = two_player_game();
    let tanazir = g.add_card_to_battlefield(0, catalog::tanazir_quandrix());
    g.clear_sickness(tanazir);
    // A friendly creature to target.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let printed_toughness = g.battlefield_find(bear).unwrap().toughness();
    assert_eq!(printed_toughness, 2);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: tanazir,
        target: AttackTarget::Player(1),
    }]))
    .expect("Tanazir can attack");
    drain_stack(&mut g);

    // Tanazir's attack trigger should pump bear's toughness by current
    // toughness (2 + 2 = 4 effective).
    let computed = g.computed_permanent(bear).unwrap();
    assert_eq!(computed.toughness, 4,
        "Bear's toughness should be doubled (2+2=4) for the turn");
}

#[test]
fn spectacle_mage_prowess_fires_only_on_noncreature_spell() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::spectacle_mage());
    g.clear_sickness(mage);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let printed_p = g.battlefield_find(mage).unwrap().power();

    // Noncreature: prowess fires.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable for {R}");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mage).unwrap().power, printed_p + 1,
        "Prowess should pump +1/+1 on noncreature spell cast");

    // Creature: prowess does not fire (still at +1 from above).
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bear castable for {1}{G}");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mage).unwrap().power, printed_p + 1,
        "Prowess should not fire on creature spell cast");
}

#[test]
fn sparring_regimen_creates_spirit_etb_and_pumps_attacker() {
    use crate::game::types::AttackTarget;
    let mut g = two_player_game();
    // ETB through casting so the trigger fires.
    let id = g.add_card_to_hand(0, catalog::sparring_regimen());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Sparring Regimen castable for {2}{R}{W}");
    drain_stack(&mut g);

    // Should have minted a Spirit token.
    let spirit = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Spirit")
        .expect("Spirit token should be present");
    let spirit_id = spirit.id;
    g.clear_sickness(spirit_id);

    // Declare it as attacker.
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: spirit_id,
        target: AttackTarget::Player(1),
    }]))
    .expect("Spirit can attack");
    drain_stack(&mut g);

    // Sparring Regimen's "whenever you attack" trigger should put a +1/+1
    // counter on the attacking Spirit.
    let counters = g.battlefield_find(spirit_id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 1, "Sparring Regimen should pump the attacker");
}

/// CR 605.4 — a life-cost mana ability resolves immediately without going on
/// the stack. Kozilek's Translator's "Pay 1 life: Add {C}" adds the mana
/// synchronously, leaving no StackItem behind.
#[test]
fn life_cost_mana_ability_is_a_mana_ability_per_cr_605() {
    let mut g = two_player_game();
    let pledge = g.add_card_to_battlefield(0, catalog::kozileks_translator());
    g.clear_sickness(pledge);

    let stack_before = g.stack.len();
    let life_before = g.players[0].life;
    let mana_before = g.players[0].mana_pool.total();

    g.perform_action(GameAction::ActivateAbility {
        card_id: pledge, ability_index: 0, target: None, x_value: None })
    .expect("mana ability activatable");

    assert_eq!(g.stack.len(), stack_before, "mana ability should not push onto the stack");
    assert_eq!(g.players[0].life, life_before - 1, "should pay 1 life as cost");
    assert_eq!(g.players[0].mana_pool.total(), mana_before + 1, "adds one mana");
}

/// CR 119.4 — a life-cost ability can't be activated with insufficient life.
#[test]
fn life_cost_mana_ability_rejects_activation_with_zero_life() {
    let mut g = two_player_game();
    let pledge = g.add_card_to_battlefield(0, catalog::kozileks_translator());
    g.clear_sickness(pledge);
    g.players[0].life = 0;

    let r = g.perform_action(GameAction::ActivateAbility {
        card_id: pledge, ability_index: 0, target: None, x_value: None });
    assert!(r.is_err(), "should reject when life < 1");
}

// ── Vanishing Verse: Monocolored predicate ──────────────────────────────────

/// Vanishing Verse should exile a monocolored permanent (single-pip
/// creature). The targeting filter is built on `Monocolored` =
/// `distinct_colors() == 1`.
#[test]
fn vanishing_verse_exiles_monocolored_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vanishing_verse());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Vanishing Verse castable for {W}{B} on monocolored bear");
    drain_stack(&mut g);

    // Bear (mono-green) gets exiled.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be exiled");
    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in exile");
}

/// Vanishing Verse must reject targeting a multicolored permanent —
/// the `Monocolored` filter prevents the cast from being legal.
#[test]
fn vanishing_verse_rejects_multicolored_target() {
    let mut g = two_player_game();
    // Use a known multicolored card from the catalog. Aziza is {R}{W}
    // → multicolored. We bypass cast to plant it directly on the
    // battlefield (the test only cares about target legality).
    let aziza = g.add_card_to_battlefield(1, catalog::aziza_mage_tower_captain());
    let id = g.add_card_to_hand(0, catalog::vanishing_verse());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    let r = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(aziza)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(r.is_err(),
        "Vanishing Verse should reject multicolored target");
    // Aziza still on battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == aziza),
        "Aziza should stay on the battlefield");
}

// ── Tanazir Quandrix: ETB counter doubling ──────────────────────────────────

/// Tanazir's ETB doubles +1/+1 counters on each creature you control.
/// A creature with 2 counters should end with 4 after Tanazir ETBs.
#[test]
fn tanazir_etb_doubles_plus_one_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Manually give the bear two +1/+1 counters.
    {
        let b = g.battlefield_find_mut(bear).unwrap();
        b.add_counters(CounterType::PlusOnePlusOne, 2);
    }
    assert_eq!(g.battlefield_find(bear).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 2);

    // Cast Tanazir through the normal cast pipeline so the ETB trigger fires.
    let tanazir = g.add_card_to_hand(0, catalog::tanazir_quandrix());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: tanazir, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tanazir castable for {2}{G}{G}{U}{U}");
    drain_stack(&mut g);

    // Bear's counters should be doubled (2 → 4).
    let after = g.battlefield_find(bear).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(after, 4,
        "Bear's +1/+1 counters should double (2 → 4) on Tanazir ETB");
}

/// Tanazir's ETB no-ops on a creature with zero +1/+1 counters
/// (doubling 0 still equals 0).
#[test]
fn tanazir_etb_does_not_add_counters_to_counterless_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // No counters on the bear.

    let tanazir = g.add_card_to_hand(0, catalog::tanazir_quandrix());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: tanazir, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tanazir castable");
    drain_stack(&mut g);

    assert_eq!(g.battlefield_find(bear).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 0,
        "Counterless creature should remain counterless");
}

// ── Bookwurm ────────────────────────────────────────────────────────────────

/// Bookwurm: {5}{G}{G} 5/5 trample with ETB "gain 4 life, draw a card".
#[test]
fn bookwurm_etb_gains_four_life_and_draws_a_card() {
    let mut g = two_player_game();
    // Seed library so the draw resolves.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::bookwurm());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bookwurm castable for {5}{G}{G}");
    drain_stack(&mut g);

    // Cast: hand -1, ETB Draw: hand +1 → net 0
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Should have cast Bookwurm and drawn one (net hand change 0)");
    assert_eq!(g.players[0].life, life_before + 4,
        "Should gain 4 life");
    // Bookwurm body on battlefield with Trample.
    let bw = g.battlefield.iter().find(|c| c.definition.name == "Bookwurm")
        .expect("Bookwurm should be on battlefield");
    assert!(bw.has_keyword(&Keyword::Trample));
    assert_eq!(bw.power(), 7);
    assert_eq!(bw.toughness(), 7);
}

// ── Field Trip ──────────────────────────────────────────────────────────────

/// Field Trip: search for a Forest, put it onto the battlefield, then
/// Learn (→ Draw 1 approximation). Uses a scripted decider to pick the
/// Forest (AutoDecider declines `SearchLibrary`).
#[test]
fn field_trip_fetches_forest_and_draws_a_card() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed library with a Forest plus filler.
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island()); // filler for draw
    g.add_card_to_library(0, catalog::island());

    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));

    let id = g.add_card_to_hand(0, catalog::field_trip());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Field Trip castable for {2}{G}");
    drain_stack(&mut g);

    // Forest should be on the battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == forest),
        "Forest should be on the battlefield");
    // Hand: -1 (cast Field Trip) + 1 (Learn → Draw) = 0
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Hand size unchanged (cast -1 + draw +1)");
}

// ── Beledros Witherbloom activated ability ─────────────────────────────────

#[test]
fn beledros_witherbloom_pay_ten_life_untaps_lands() {
    let mut g = two_player_game();
    let bele = g.add_card_to_battlefield(0, catalog::beledros_witherbloom());
    g.clear_sickness(bele);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let island = g.add_card_to_battlefield(0, catalog::island());
    // Tap the lands.
    g.battlefield.iter_mut().find(|c| c.id == forest).unwrap().tapped = true;
    g.battlefield.iter_mut().find(|c| c.id == island).unwrap().tapped = true;
    let life_before = g.players[0].life;

    g.perform_action(GameAction::ActivateAbility {
        card_id: bele, ability_index: 0, target: None,
        x_value: None,
    }).expect("Beledros activated for 10 life");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().find(|c| c.id == forest).unwrap().tapped,
        "Forest should be untapped");
    assert!(!g.battlefield.iter().find(|c| c.id == island).unwrap().tapped,
        "Island should be untapped");
    assert_eq!(g.players[0].life, life_before - 10,
        "Should have paid 10 life");
}

// ── Decisive Denial mode 1 fight ──────────────────────────────────────────

#[test]
fn decisive_denial_mode_one_fights_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // Put a 1/1 on opponent's side.
    let opp = g.add_card_to_battlefield(1, catalog::eyetwitch());
    g.add_card_to_library(1, catalog::island()); // library for Eyetwitch draw
    let id = g.add_card_to_hand(0, catalog::decisive_denial());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    // Target the opponent's creature (defender) — our creature (attacker)
    // is auto-picked via the `Take(EachPermanent(your creature), 1)` selector.
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    }).expect("Decisive Denial mode 1 castable");
    drain_stack(&mut g);

    // Eyetwitch (1/1) takes 2 damage from Bear (2/2) and dies.
    assert!(!g.battlefield.iter().any(|c| c.id == opp),
        "Eyetwitch should be dead from fight");
    // Bear survives (took 1 damage, has 2 toughness).
    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "Bear should survive the fight");
}

// ── Teach by Example ───────────────────────────────────────────────────────

// ── Introduction to Prophecy ───────────────────────────────────────────────

#[test]
fn introduction_to_prophecy_scrys_then_draws() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::introduction_to_prophecy());
    let hand_before = g.players[0].hand.len();

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Introduction to Prophecy castable for {2}{U}");
    drain_stack(&mut g);

    // Hand: -1 (cast) +1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Should draw 1 card (net zero from casting + drawing)");
}

// ── Introduction to Annihilation ───────────────────────────────────────────

#[test]
fn introduction_to_annihilation_exiles_nonland_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::introduction_to_annihilation());

    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Introduction to Annihilation castable for {5}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be off the battlefield (exiled)");
}

// ── Environmental Sciences ─────────────────────────────────────────────────

#[test]
fn environmental_sciences_fetches_basic_land_and_gains_life() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed library with a basic Forest.
    let forest = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::environmental_sciences());
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
    ]));

    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Environmental Sciences castable for {2}");
    drain_stack(&mut g);

    // Hand: -1 (cast) +1 (search to hand) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Should fetch a basic land to hand");
    assert_eq!(g.players[0].life, life_before + 2,
        "Should gain 2 life");
}

// ── Fractal Summoning ──────────────────────────────────────────────────────

#[test]
fn fractal_summoning_creates_token_with_x_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_summoning());

    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    })
    .expect("Fractal Summoning castable for {X=3}{G}{U}");
    drain_stack(&mut g);

    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal")
        .expect("Fractal token present");
    let counters = fractal.counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 3, "Fractal should have 3 +1/+1 counters (X=3)");
    assert_eq!(fractal.power(), 3, "Fractal should be a 3/3");
    assert_eq!(fractal.toughness(), 3);
}

// ── Spirit Summoning ───────────────────────────────────────────────────────

#[test]
fn spirit_summoning_creates_three_two_spirit_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spirit_summoning());

    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Spirit Summoning castable for {3}{W}");
    drain_stack(&mut g);

    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 1, "should create one Spirit token");
    let s = spirits[0];
    assert_eq!(s.power(), 3, "Spirit should be 3/2");
    assert_eq!(s.toughness(), 2);
}

// ── Silverquill Apprentice ─────────────────────────────────────────────────

#[test]
fn silverquill_apprentice_adds_counter_on_instant_cast() {
    // Real STX Silverquill Apprentice: Magecraft puts a +1/+1 counter
    // on target creature you control. (Was previously drain in our
    // catalog; corrected to match the printed Oracle.)
    let mut g = two_player_game();
    let _app = g.add_card_to_battlefield(0, catalog::silverquill_apprentice());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let bear_perm = g.battlefield.iter().find(|c| c.id == bear).expect("bear alive");
    assert!(bear_perm.counter_count(CounterType::PlusOnePlusOne) >= 1,
        "Bear should get a +1/+1 counter from Magecraft");
}

// ── Shadewing Laureate ────────────────────────────────────────────────────

// ── Returned Pastcaller ───────────────────────────────────────────────────

#[test]
fn returned_pastcaller_returns_instant_from_graveyard_on_etb() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::returned_pastcaller());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Returned Pastcaller castable for {4}{R}{W}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt should be returned to hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "Bolt should no longer be in graveyard");
}

// ── Elemental Expressionist ───────────────────────────────────────────────

#[test]
fn elemental_expressionist_flickers_opp_creature_on_magecraft() {
    let mut g = two_player_game();
    let _expr = g.add_card_to_battlefield(0, catalog::elemental_expressionist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Magecraft exiles the opponent's creature and queues a delayed return.
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear exiled");
    assert!(g.exile.iter().any(|c| c.id == bear), "bear in exile");
    assert!(g.delayed_triggers.iter().any(|d| d.controller == 0),
        "delayed end-step return registered");
}

// ── Prowess wiring ─────────────────────────────────────────────────────────

#[test]
fn spectacle_mage_prowess_pumps_on_noncreature_cast() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::spectacle_mage());
    assert_eq!(g.battlefield.iter().find(|c| c.id == mage).unwrap().power(), 2);

    let bolt = g.add_card_to_hand(0, catalog::interjection());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(mage)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);

    let m = g.battlefield.iter().find(|c| c.id == mage).unwrap();
    // Interjection gives +2/+2 EOT, prowess gives +1/+1 EOT
    assert!(m.power() >= 4, "got P={}", m.power());
}

#[test]
fn spectacle_mage_prowess_does_not_fire_on_creature_cast() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::spectacle_mage());

    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);

    let m = g.battlefield.iter().find(|c| c.id == mage).unwrap();
    assert_eq!(m.power(), 2, "Prowess should NOT fire on creature spell");
}

/// Reduce to Memory exiles the targeted permanent and mints a 2/2
/// colorless Inkling artifact creature for its controller.
#[test]
fn reduce_to_memory_exiles_and_creates_inkling() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::reduce_to_memory());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Reduce to Memory castable for {2}{U}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in exile");
    let inkling = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Inkling")
        .expect("Inkling token should exist on battlefield");
    assert_eq!(inkling.power(), 2);
    assert_eq!(inkling.toughness(), 2);
    assert!(inkling.definition.is_artifact(),
        "Inkling should be an artifact");
    assert!(inkling.definition.is_creature(),
        "Inkling should be a creature");
}

// ── Baleful Mastery ─────────────────────────────────────────────────────────

#[test]
fn baleful_mastery_exiles_creature_and_opp_draws() {
    let mut g = two_player_game();
    // Seed opp library so the draw resolves.
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::baleful_mastery());
    // Full cost is now {3}{B}.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);

    let opp_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Baleful Mastery castable for {3}{B}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == bear), "Bear exiled");
    // At full cost ({3}{B}), the opponent does NOT draw.
    assert_eq!(g.players[1].hand.len(), opp_hand_before,
        "At full cost, opponent should not draw a card");
}

// ── Igneous Inspiration ─────────────────────────────────────────────────────

#[test]
fn igneous_inspiration_deals_three_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::igneous_inspiration());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Igneous Inspiration castable for {2}{R}");
    drain_stack(&mut g);

    // Bear (2/2) takes 3 damage → dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should die to 3 damage");
    // Hand: -1 (cast) + 1 (Learn) = 0
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Hand unchanged after cast + Learn");
}

// ── Combat Professor ────────────────────────────────────────────────────────

// ── Beaming Defiance ────────────────────────────────────────────────────────

#[test]
fn beaming_defiance_pumps_and_grants_hexproof() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::beaming_defiance());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    let printed_p = g.battlefield_find(bear).unwrap().power();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Beaming Defiance castable for {1}{W}");
    drain_stack(&mut g);

    let computed = g.computed_permanent(bear).unwrap();
    assert_eq!(computed.power, printed_p + 2, "+2 power applied");
    let bear_card = g.battlefield_find(bear).unwrap();
    assert!(bear_card.has_keyword(&Keyword::Hexproof),
        "Bear should have Hexproof until EOT");
}

// ── Excavated Wall ──────────────────────────────────────────────────────────

#[test]
fn excavated_wall_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::excavated_wall());
    g.players[0].mana_pool.add_colorless(2);

    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Excavated Wall castable for {2}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 2);
    // Body is a 0/4 artifact creature with Defender.
    let wall = g.battlefield.iter().find(|c| c.definition.name == "Excavated Wall")
        .expect("Wall should be on battlefield");
    assert_eq!(wall.power(), 0);
    assert_eq!(wall.toughness(), 4);
    assert!(wall.has_keyword(&Keyword::Defender));
}

// ── Snow Day ────────────────────────────────────────────────────────────────

#[test]
fn snow_day_taps_and_stuns_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::snow_day());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Snow Day castable for {U}{R}");
    drain_stack(&mut g);

    let target = g.battlefield_find(bear).unwrap();
    assert!(target.tapped, "Bear should be tapped");
    assert_eq!(target.counter_count(CounterType::Stun), 1,
        "Bear should have a stun counter");
}

/// Snow Day cast at TWO creatures (push modern_decks multi-target): both
/// targets are tapped and gain a stun counter. Slot 0 is `target`, slot
/// 1 is `additional_targets[0]`.
#[test]
fn snow_day_taps_and_stuns_two_target_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::snow_day());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(angel)],
        mode: None,
        x_value: None,
    })
    .expect("Snow Day castable at two targets");
    drain_stack(&mut g);

    let b = g.battlefield_find(bear).unwrap();
    assert!(b.tapped, "Bear should be tapped");
    assert_eq!(b.counter_count(CounterType::Stun), 1, "Bear has a stun counter");

    let a = g.battlefield_find(angel).unwrap();
    assert!(a.tapped, "Serra Angel should be tapped");
    assert_eq!(a.counter_count(CounterType::Stun), 1, "Angel has a stun counter");
}

// ── Spell Satchel ───────────────────────────────────────────────────────────

#[test]
fn spell_satchel_tap_adds_one_colorless() {
    let mut g = two_player_game();
    let satchel = g.add_card_to_battlefield(0, catalog::spell_satchel());
    g.clear_sickness(satchel);

    let mana_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: satchel, ability_index: 0, target: None, x_value: None })
    .expect("Spell Satchel mana ability activatable");
    assert_eq!(g.players[0].mana_pool.total(), mana_before + 1,
        "Spell Satchel should add 1 colorless");
    assert!(g.battlefield_find(satchel).unwrap().tapped,
        "Spell Satchel should be tapped");
}

#[test]
fn spell_satchel_sacrifice_returns_low_cmc_spell_from_graveyard() {
    let mut g = two_player_game();
    let satchel = g.add_card_to_battlefield(0, catalog::spell_satchel());
    g.clear_sickness(satchel);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());

    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::ActivateAbility {
        card_id: satchel,
        ability_index: 1,
        target: Some(Target::Permanent(bolt)), x_value: None })
    .expect("Spell Satchel grave-return activation");
    drain_stack(&mut g);

    // Bolt should be back in hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt should be in hand");
    // Satchel sacrificed → in graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == satchel),
        "Spell Satchel should be sacrificed to graveyard");
}

#[test]
fn spell_satchel_returns_multiple_low_mv_instants_within_cap() {
    // Three Lightning Bolts (MV 1 each, total MV 3 ≤ 4) — all three
    // should return to hand. Tests the multi-card pickup.
    let mut g = two_player_game();
    let satchel = g.add_card_to_battlefield(0, catalog::spell_satchel());
    g.clear_sickness(satchel);
    let bolt_a = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let bolt_b = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let bolt_c = g.add_card_to_graveyard(0, catalog::lightning_bolt());

    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::ActivateAbility {
        card_id: satchel,
        ability_index: 1,
        target: None, x_value: None })
    .expect("Spell Satchel grave-return activation");
    drain_stack(&mut g);

    let in_hand = [bolt_a, bolt_b, bolt_c]
        .iter()
        .filter(|&&bid| g.players[0].hand.iter().any(|c| c.id == bid))
        .count();
    assert_eq!(in_hand, 3, "all three Bolts (MV 1 each, total 3 ≤ 4) return to hand");
}

#[test]
fn spell_satchel_picks_bolt_and_cancel_at_exactly_four_total() {
    // 1x Bolt (MV 1) + 1x Cancel (MV 3) = total 4 ≤ 4. Both return.
    let mut g = two_player_game();
    let satchel = g.add_card_to_battlefield(0, catalog::spell_satchel());
    g.clear_sickness(satchel);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let cancel = g.add_card_to_graveyard(0, catalog::cancel());

    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::ActivateAbility {
        card_id: satchel,
        ability_index: 1,
        target: None, x_value: None })
    .expect("Spell Satchel grave-return activation");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt should return (MV 1)");
    assert!(g.players[0].hand.iter().any(|c| c.id == cancel),
        "Cancel should return (MV 3, total 4)");
}

#[test]
fn spell_satchel_skips_cards_that_would_overflow_cap() {
    // Two Cancels (MV 3 each, total 6 > 4). First Cancel fits (3 ≤ 4),
    // second would push to 6 → skip.
    let mut g = two_player_game();
    let satchel = g.add_card_to_battlefield(0, catalog::spell_satchel());
    g.clear_sickness(satchel);
    let cancel_a = g.add_card_to_graveyard(0, catalog::cancel());
    let cancel_b = g.add_card_to_graveyard(0, catalog::cancel());

    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::ActivateAbility {
        card_id: satchel,
        ability_index: 1,
        target: None, x_value: None })
    .expect("Spell Satchel grave-return activation");
    drain_stack(&mut g);

    let in_hand = [cancel_a, cancel_b]
        .iter()
        .filter(|&&cid| g.players[0].hand.iter().any(|c| c.id == cid))
        .count();
    assert_eq!(in_hand, 1, "only first Cancel returns (3 ≤ 4); second overflows");
    let in_gy = [cancel_a, cancel_b]
        .iter()
        .filter(|&&cid| g.players[0].graveyard.iter().any(|c| c.id == cid))
        .count();
    assert_eq!(in_gy, 1, "second Cancel stays in graveyard");
}

#[test]
fn spell_satchel_greedy_walk_still_fits_smaller_after_skipping_bigger() {
    // Cancel (MV 3, fits as first) + Cancel (MV 3, skip — running 6) +
    // Bolt (MV 1, fits — running 3+1 = 4). The greedy walk continues
    // past the skipped Cancel and picks up the Bolt that still fits.
    let mut g = two_player_game();
    let satchel = g.add_card_to_battlefield(0, catalog::spell_satchel());
    g.clear_sickness(satchel);
    let cancel_a = g.add_card_to_graveyard(0, catalog::cancel());
    let cancel_b = g.add_card_to_graveyard(0, catalog::cancel());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());

    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::ActivateAbility {
        card_id: satchel,
        ability_index: 1,
        target: None, x_value: None })
    .expect("Spell Satchel grave-return activation");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == cancel_a),
        "first Cancel returns (MV 3)");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cancel_b),
        "second Cancel stays (would overflow)");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt returns (MV 1, fits as 3+1=4)");
}

#[test]
fn spell_satchel_filters_to_instants_and_sorceries() {
    // Bear in gy alongside Bolt — only Bolt comes back, Bear stays.
    let mut g = two_player_game();
    let satchel = g.add_card_to_battlefield(0, catalog::spell_satchel());
    g.clear_sickness(satchel);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());

    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::ActivateAbility {
        card_id: satchel,
        ability_index: 1,
        target: None, x_value: None })
    .expect("Spell Satchel grave-return activation");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt (instant) returns");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear),
        "Grizzly Bears (creature) stays in graveyard");
}

// ── Curate ──────────────────────────────────────────────────────────────────

#[test]
fn curate_draws_after_scry_three() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::curate());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    let lib_before = g.players[0].library.len();
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Curate castable for {1}{U}");
    drain_stack(&mut g);

    // Hand: -1 (cast) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Hand unchanged after cast + draw");
    // Library: -1 (drew one card).
    assert_eq!(g.players[0].library.len(), lib_before - 1,
        "Library should lose one card to draw");
}

// ── Solve the Equation ──────────────────────────────────────────────────────

#[test]
fn solve_the_equation_finds_instant_or_sorcery() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed library with one instant, one creature.
    g.add_card_to_library(0, catalog::island()); // basic land
    g.add_card_to_library(0, catalog::grizzly_bears()); // creature
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt()); // instant

    // Search defaults to None — script the decider to pick Bolt.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));

    let id = g.add_card_to_hand(0, catalog::solve_the_equation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Solve the Equation castable for {2}{U}");
    drain_stack(&mut g);

    // Bolt should now be in hand (tutored).
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Lightning Bolt should be tutored into hand");
    // Library should no longer contain Bolt.
    assert!(!g.players[0].library.iter().any(|c| c.id == bolt),
        "Bolt should have left the library");
}

// ── Resculpt ────────────────────────────────────────────────────────────────

#[test]
fn resculpt_exiles_creature_and_mints_elemental_for_controller() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::resculpt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Resculpt castable for {1}{U}");
    drain_stack(&mut g);

    // Bear exiled → no longer on battlefield.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be exiled");
    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in exile");
    // Opponent (the bear's controller) should now have a 4/4 Elemental.
    let elemental = g.battlefield.iter()
        .find(|c| c.controller == 1 && c.definition.name == "Elemental")
        .expect("Elemental token should be under bear's original controller");
    assert_eq!(elemental.power(), 4);
    assert_eq!(elemental.toughness(), 4);
}

// ── Mortality Spear ────────────────────────────────────────────────────────

#[test]
fn mortality_spear_destroys_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mortality_spear());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Mortality Spear castable for {3}{B}{G}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be destroyed");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "Bear should be in graveyard");
}

// ── Daemogoth Titan ────────────────────────────────────────────────────────

// ── Daemogoth Woe-Eater ────────────────────────────────────────────────────

#[test]
fn daemogoth_titan_attacks_sacrifices_non_source_creature_first() {
    use crate::game::Attack;
    let mut g = two_player_game();
    let titan = g.add_card_to_battlefield(0, catalog::daemogoth_titan());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(titan);
    g.clear_sickness(fodder);
    g.step = TurnStep::DeclareAttackers;

    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: titan,
        target: crate::game::AttackTarget::Player(1),
    }]))
    .expect("Titan can attack");
    drain_stack(&mut g);

    // Sac priority should pick the fodder bear, not the Titan itself.
    assert!(g.battlefield.iter().any(|c| c.id == titan),
        "Daemogoth Titan should NOT have sacrificed itself");
    assert!(!g.battlefield.iter().any(|c| c.id == fodder),
        "Bear (the non-source candidate) should be sacrificed");
}

#[test]
fn daemogoth_titan_blocks_sacrifices_another_creature() {
    // `EventKind::Blocks` fires off BlockerDeclared (CR 509.1i).
    use crate::game::Attack;
    let mut g = two_player_game();
    // Attacker on P0 (active player).
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    // Defender on P1: Daemogoth Titan + a fodder bear.
    let titan = g.add_card_to_battlefield(1, catalog::daemogoth_titan());
    let fodder = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(titan);
    g.clear_sickness(fodder);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: crate::game::AttackTarget::Player(1),
    }]))
    .expect("Bear attacks");

    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(titan, attacker)]))
        .expect("Titan can block the attacking bear");
    drain_stack(&mut g);

    // Titan should still be on bf (sacked the fodder, not itself).
    assert!(g.battlefield.iter().any(|c| c.id == titan),
        "Daemogoth Titan should NOT have sacrificed itself on block");
    assert!(!g.battlefield.iter().any(|c| c.id == fodder),
        "Fodder bear (non-source) should be sacrificed on block");
}

#[test]
fn daemogoth_woe_eater_etb_sacrifices_another_creature() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::daemogoth_woe_eater());

    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Daemogoth Woe-Eater castable for {2}{B}{G}");
    drain_stack(&mut g);

    // Fodder bear should be sacrificed.
    assert!(!g.battlefield.iter().any(|c| c.id == fodder),
        "Bear should have been sacrificed to Woe-Eater ETB");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder),
        "Bear should be in graveyard");
    // Woe-Eater itself should still be on the battlefield.
    let woe = g.battlefield.iter().find(|c| c.definition.name == "Daemogoth Woe-Eater")
        .expect("Woe-Eater should be on the battlefield");
    assert_eq!(woe.power(), 7);
    assert_eq!(woe.toughness(), 6);
}

#[test]
fn daemogoth_woe_eater_attack_optional_sac_can_be_declined() {
    // AutoDecider defaults to "no" on the `MayDo` sac, so neither the
    // sacrifice nor the +1/+1 counter should fire.
    use crate::card::CounterType;
    let mut g = two_player_game();
    let fodder1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let fodder2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let woe = g.add_card_to_battlefield(0, catalog::daemogoth_woe_eater());
    // Sac fodder1 manually so the ETB doesn't eat fodder2.
    g.battlefield.retain(|c| c.id != fodder1);
    g.clear_sickness(woe);
    g.clear_sickness(fodder2);

    // Move to declare-attackers and attack with the Woe-Eater.
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        crate::game::types::Attack {
            attacker: woe,
            target: crate::game::types::AttackTarget::Player(1),
        }
    ])).expect("Woe-Eater attacks");
    drain_stack(&mut g);

    // AutoDecider said no — fodder2 should still be on the battlefield
    // and Woe-Eater should not have a +1/+1 counter.
    assert!(g.battlefield.iter().any(|c| c.id == fodder2),
        "Fodder bear should NOT be sacrificed when controller declines");
    let woe_card = g.battlefield.iter().find(|c| c.id == woe)
        .expect("Woe-Eater on battlefield");
    let counters = woe_card.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(counters, 0,
        "Woe-Eater should NOT have a +1/+1 counter when the attack-trigger is declined");
}

#[test]
fn daemogoth_woe_eater_attack_optional_sac_can_be_accepted() {
    // Scripted decider says yes to the MayDo prompt; the sacrifice
    // fires and the Woe-Eater gains a +1/+1 counter.
    use crate::card::CounterType;
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let fodder1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let fodder2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let woe = g.add_card_to_battlefield(0, catalog::daemogoth_woe_eater());
    g.battlefield.retain(|c| c.id != fodder1);
    g.clear_sickness(woe);
    g.clear_sickness(fodder2);

    // ScriptedDecider: say yes to the optional sacrifice prompt.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.step = crate::game::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        crate::game::types::Attack {
            attacker: woe,
            target: crate::game::types::AttackTarget::Player(1),
        }
    ])).expect("Woe-Eater attacks");
    drain_stack(&mut g);

    // Yes-path: fodder2 is sacrificed and Woe-Eater gets a +1/+1 counter.
    assert!(!g.battlefield.iter().any(|c| c.id == fodder2),
        "Fodder bear should be sacrificed when controller accepts");
    let woe_card = g.battlefield.iter().find(|c| c.id == woe)
        .expect("Woe-Eater on battlefield");
    let counters = woe_card.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(counters, 1,
        "Woe-Eater should have one +1/+1 counter after a successful sac");
}

// ── Honor Troll ────────────────────────────────────────────────────────────

#[test]
fn honor_troll_lifegain_bonus_adds_one() {
    // CR 119.10 — Honor Troll: each life gain is +1.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::honor_troll());
    let before = g.players[0].life;
    g.adjust_life(0, 3); // gain 3 → 4 with the bonus
    assert_eq!(g.players[0].life, before + 4, "gained 3 + 1 bonus");
    // The bonus only applies to genuine gains, not to losses.
    g.adjust_life(0, -2);
    assert_eq!(g.players[0].life, before + 4 - 2, "loss is unaffected by the bonus");
}

#[test]
fn honor_troll_gets_plus_two_one_at_25_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::honor_troll());
    // Below 25 life → base 2/3.
    g.players[0].life = 20;
    let lo = g.computed_permanent(id).unwrap();
    assert_eq!((lo.power, lo.toughness), (2, 3), "base while under 25 life");
    // At 25+ life → +2/+1 → 4/4.
    g.players[0].life = 25;
    let hi = g.computed_permanent(id).unwrap();
    assert_eq!((hi.power, hi.toughness), (4, 4), "+2/+1 at 25+ life");
    assert!(hi.keywords.contains(&Keyword::Vigilance));
}

// ── Quandrix Cultivator ────────────────────────────────────────────────────

#[test]
fn quandrix_cultivator_etb_fetches_basic_forest_or_island() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed library with one Forest + an unrelated card so the search
    // has a legal target.
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());

    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));

    let id = g.add_card_to_hand(0, catalog::quandrix_cultivator());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Quandrix Cultivator castable for {1}{G}{G/U}{U}");
    drain_stack(&mut g);

    // Forest should be on the battlefield, untapped.
    let f = g.battlefield_find(forest).expect("Forest should be in play");
    assert!(!f.tapped, "Tutored Forest enters untapped");
    assert!(f.definition.is_land());
}

// ── Hofri Ghostforge ───────────────────────────────────────────────────────

// ── Tempted by the Oriq ────────────────────────────────────────────────────

#[test]
fn tempted_by_the_oriq_cannot_steal_high_mv_creature() {
    // The MV-3-or-less gate: a 5-MV creature is not a legal target.
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // MV 5

    let id = g.add_card_to_hand(0, catalog::tempted_by_the_oriq());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    let res = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(res.is_err(), "5-MV creature is not a legal target");
    let b = g.battlefield_find(big).expect("still on bf");
    assert_eq!(b.controller, 1, "still controlled by its owner");
}

#[test]
fn confront_the_past_bounces_planeswalker_via_mode_1() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(1, catalog::professor_dellian_fel());
    let id = g.add_card_to_hand(0, catalog::confront_the_past());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(pw)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    }).expect("Confront the Past castable for {3}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == pw), "PW off battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == pw), "PW in opp's hand");
}

#[test]
fn specter_of_the_fens_drains_two() {
    let mut g = two_player_game();
    let spec = g.add_card_to_battlefield(0, catalog::specter_of_the_fens());
    g.clear_sickness(spec);
    let (opp_before, you_before) = (g.players[1].life, g.players[0].life);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: spec, ability_index: 0, target: None, x_value: None,
    }).expect("{5}{B} drain");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2, "opponent loses 2");
    assert_eq!(g.players[0].life, you_before + 2, "you gain 2");
    assert!(g.battlefield_find(spec).unwrap().has_keyword(&Keyword::Flying));
}

#[test]
fn mascot_interception_gains_control_untaps_grants_haste() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == opp_bear) {
        c.tapped = true;
        c.summoning_sick = false;
    }
    let id = g.add_card_to_hand(0, catalog::mascot_interception());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mascot Interception castable for {4}{R}{W}");
    drain_stack(&mut g);

    let bear = g.battlefield.iter().find(|c| c.id == opp_bear)
        .expect("bear still on bf");
    assert_eq!(bear.controller, 0, "control transferred to caster");
    assert!(!bear.tapped, "bear untapped");
    assert!(bear.has_keyword(&Keyword::Haste), "haste granted EOT");
}

#[test]
fn twinscroll_shaman_is_a_double_striking_one_two() {
    let g = catalog::twinscroll_shaman();
    assert_eq!(g.cost.cmc(), 3);
    assert_eq!((g.power, g.toughness), (1, 2));
    assert!(g.keywords.contains(&Keyword::DoubleStrike));
}

#[test]
fn practical_research_draws_four_then_discards_two() {
    let mut g = two_player_game();
    g.players[0].hand.clear();
    for _ in 0..6 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::practical_research());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Practical Research castable");
    drain_stack(&mut g);

    // Started with 0 (after casting the spell from hand): +4 draw − 2 discard = 2.
    assert_eq!(g.players[0].hand.len(), 2, "drew 4, discarded 2");
}

#[test]
fn hall_of_oracles_taps_for_colorless_and_buffs_wizard() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::hall_of_oracles());
    let wiz = g.add_card_to_battlefield(0, catalog::symmetry_sage());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == wiz) {
        c.summoning_sick = false;
    }
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == land) {
        c.summoning_sick = false;
    }

    let c_before = g.players[0].mana_pool.colorless_amount();
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, x_value: None }).expect("Hall {T}: Add {C}");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), c_before + 1);

    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == land) {
        c.tapped = false;
    }
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: Some(Target::Permanent(wiz)), x_value: None }).expect("Hall {2},{T}: +1/+1");
    drain_stack(&mut g);

    let wiz_c = g.battlefield.iter().find(|c| c.id == wiz).unwrap();
    assert_eq!(wiz_c.counter_count(CounterType::PlusOnePlusOne), 1,
        "Wizard got a +1/+1 counter");
}

#[test]
fn star_pupil_enters_with_a_plus_one_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::star_pupil());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Star Pupil castable for {W}");
    drain_stack(&mut g);

    let star = g.battlefield.iter()
        .find(|c| c.definition.name == "Star Pupil")
        .expect("Star Pupil in play");
    assert_eq!(star.counter_count(CounterType::PlusOnePlusOne), 1,
        "Star Pupil enters with one +1/+1 counter");
    // 0/0 base + 1 from counter = 1/1 effective stats.
    assert_eq!(star.power(), 1);
    assert_eq!(star.toughness(), 1);
}

#[test]
fn star_pupil_death_puts_its_counters_on_target_creature() {
    let mut g = two_player_game();
    let star = g.add_card_to_battlefield(0, catalog::star_pupil());
    let recipient = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Give Star Pupil two +1/+1 counters so we exercise "its counters" (all of them).
    g.battlefield_find_mut(star).unwrap()
        .counters.insert(CounterType::PlusOnePlusOne, 2);
    g.clear_sickness(star);
    g.clear_sickness(recipient);

    // Kill Star Pupil with damage.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(star)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);

    let bear = g.battlefield.iter().find(|c| c.id == recipient).unwrap();
    assert_eq!(bear.counter_count(CounterType::PlusOnePlusOne), 2,
        "death moves all of Star Pupil's +1/+1 counters to the target");
}

#[test]
fn ageless_guardian_is_a_vanilla_one_four() {
    let c = catalog::ageless_guardian();
    assert_eq!(c.cost.cmc(), 2);
    assert_eq!((c.power, c.toughness), (1, 4));
    assert!(c.triggered_abilities.is_empty() && c.activated_abilities.is_empty());
}

#[test]
fn returned_pastcaller_etb_returns_instant_from_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::returned_pastcaller());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Returned Pastcaller castable for {4}{W}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt should be back in hand after Pastcaller ETB");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "Bolt should no longer be in gy");
    let p = g.battlefield.iter()
        .find(|c| c.definition.name == "Returned Pastcaller").unwrap();
    assert!(p.has_keyword(&Keyword::Flying), "Pastcaller is a flyer");
}

#[test]
fn letter_of_acceptance_fixes_mana_then_sacs_to_draw() {
    let mut g = two_player_game();
    let letter_id = g.add_card_to_battlefield(0, catalog::letter_of_acceptance());
    g.clear_sickness(letter_id);

    // Tap for one mana of any color.
    g.perform_action(GameAction::ActivateAbility {
        card_id: letter_id, ability_index: 0, target: None, x_value: None }).expect("{T}: Add any color");
    assert_eq!(g.players[0].mana_pool.total(), 1, "added one mana");

    // Untap, then sac to draw.
    g.battlefield_find_mut(letter_id).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(2);
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: letter_id, ability_index: 1, target: None, x_value: None }).expect("{2},{T},Sac: Draw");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    assert!(!g.battlefield.iter().any(|c| c.id == letter_id), "Letter sacrificed");
}

#[test]
fn charge_through_grants_trample_and_cantrips() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::charge_through());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Charge Through castable for {G}");
    drain_stack(&mut g);

    let b = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(b.has_keyword(&Keyword::Trample), "trample granted EOT");
    // Cast (-1) + draw (+1) nets the same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before, "cantrip replaces itself");
}

#[test]
fn devious_cover_up_counters_a_spell_and_exiles_chosen_gy_cards() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // P1 casts Bolt; P0 counters with Devious Cover-Up. Also seed two gy cards.
    let extra0 = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let extra1 = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");

    g.priority.player_with_priority = 0;
    // "Exile any number" — choose both seeded gy cards (across both players).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![extra0, extra1])]));
    let cover = g.add_card_to_hand(0, catalog::devious_cover_up());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: cover, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cover-Up castable for {2}{U}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, 20, "Bolt countered");
    // Both chosen graveyard cards are now in exile; the countered Bolt
    // (not chosen) remains in P1's graveyard.
    assert!(g.exile.iter().any(|c| c.id == extra0), "P0 gy card exiled");
    assert!(g.exile.iter().any(|c| c.id == extra1), "P1 gy card exiled");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "countered Bolt stays");
}

#[test]
fn devious_cover_up_auto_decider_exiles_nothing() {
    // AutoDecider answers ChooseCards with the empty set ("up to" default).
    let mut g = two_player_game();
    let gy = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    g.priority.player_with_priority = 0;
    let cover = g.add_card_to_hand(0, catalog::devious_cover_up());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: cover, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cover-Up castable");
    drain_stack(&mut g);
    // Nothing exiled; both the seed and the countered Bolt remain.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == gy));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
    assert!(g.exile.is_empty(), "AutoDecider exiles nothing");
}

#[test]
fn manifestation_sage_etb_creates_fractal_with_counters_from_hand() {
    let mut g = two_player_game();
    // Seed P0 with 3 cards in hand (excluding the cast spell, which leaves
    // hand before ETB resolves).
    for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::manifestation_sage());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Blue, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Manifestation Sage castable for {G/U}{G/U}{G/U}{G/U}");
    drain_stack(&mut g);

    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal")
        .expect("Fractal token minted");
    // After cast the hand had 3 cards; counters scale to that count.
    let counters = fractal.counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 3,
        "Fractal +1/+1 counters equal cards in hand at resolution; got {}",
        counters);
}

#[test]
fn crackle_with_power_deals_five_x_damage_to_target_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::crackle_with_power());
    // X=2 → 10 damage.
    g.players[0].mana_pool.add(Color::Red, 5);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("Crackle castable for {X=2}{R}{R}{R}{R}{R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 20 - 10,
        "5X = 10 damage at X=2");
}

#[test]
fn crackle_with_power_divides_damage_among_two_targets() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::crackle_with_power());
    // X=1 → 5 damage, split 3 (target 0) / 2 (target 1) by the even-split
    // front-loaded policy: opp player takes 3, bear takes 2 and dies.
    g.players[0].mana_pool.add(Color::Red, 5);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: Some(1),
    }).expect("Crackle castable for {X=1}{R}{R}{R}{R}{R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 20 - 3, "player gets 3 of the 5 (front-loaded)");
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear dies to its 2");
}

#[test]
fn mentors_guidance_draws_one_without_qualifier() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::mentors_guidance());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mentor's Guidance castable");
    drain_stack(&mut g);
    // No qualifying permanent → no copy. Net: -1 spell + 1 draw = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1, "scry 1, draw 1");
}

#[test]
fn mentors_guidance_copies_with_a_wizard() {
    let mut g = two_player_game();
    // A Wizard you control makes the cast-trigger copy the spell.
    let _wiz = g.add_card_to_battlefield(0, catalog::burrog_befuddler());
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::mentors_guidance());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mentor's Guidance castable");
    drain_stack(&mut g);
    // Copy + original each draw 1 → net -1 spell + 2 draws.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "copy doubles the draw");
}

#[test]
fn dragonsguard_elite_magecraft_adds_counter_and_pumps_x_equal_to_power() {
    let mut g = two_player_game();
    let dge = g.add_card_to_battlefield(0, catalog::dragonsguard_elite());
    g.clear_sickness(dge);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);

    let d = g.battlefield.iter().find(|c| c.id == dge).unwrap();
    assert_eq!(d.counter_count(CounterType::PlusOnePlusOne), 1,
        "Magecraft adds a +1/+1 counter");
    // 2/2 + 1 counter = 3/3.
    assert_eq!(d.power(), 3);
    assert_eq!(d.toughness(), 3);

    // Activate {3}{G}: +X/+X EOT — at 3 power, that's +3/+3 → 6/6.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dge, ability_index: 0, target: None, x_value: None }).expect("{3}{G}: +X/+X");
    drain_stack(&mut g);

    let d2 = g.battlefield.iter().find(|c| c.id == dge).unwrap();
    assert_eq!(d2.power(), 6, "Dragonsguard Elite: 3 + 3 = 6");
    assert_eq!(d2.toughness(), 6);
}

#[test]
fn quintorius_makes_a_spirit_when_a_card_leaves_your_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::quintorius_field_historian());
    g.dispatch_triggers_for_events(&[crate::game::types::GameEvent::CardLeftGraveyard {
        player: 0, card_id: crate::card::CardId(999),
    }]);
    drain_stack(&mut g);
    let spirit = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Spirit")
        .expect("3/2 Spirit minted on gy-leave");
    assert_eq!((spirit.power(), spirit.toughness()), (3, 2));
}

#[test]
fn quintorius_anthem_pumps_spirits_not_himself() {
    let mut g = two_player_game();
    let qid = g.add_card_to_battlefield(0, catalog::quintorius_field_historian());
    let mascot = g.add_card_to_battlefield(0, catalog::spirit_mascot());

    // Spirit Mascot (2/2 Spirit) gets +1/+0 → 3/2.
    let mascot_card = g.compute_battlefield().into_iter()
        .find(|c| c.id == mascot).expect("Spirit Mascot on battlefield");
    assert_eq!((mascot_card.power, mascot_card.toughness), (3, 2));

    // Quintorius is an Elephant Cleric, not a Spirit → unaffected (2/4).
    let q_card = g.compute_battlefield().into_iter()
        .find(|c| c.id == qid).expect("Quintorius on battlefield");
    assert_eq!((q_card.power, q_card.toughness), (2, 4));
}

#[test]
fn quintorius_anthem_expires_when_he_leaves_battlefield() {
    let mut g = two_player_game();
    let qid = g.add_card_to_battlefield(0, catalog::quintorius_field_historian());
    let mascot = g.add_card_to_battlefield(0, catalog::spirit_mascot());

    let before = g.compute_battlefield().into_iter()
        .find(|c| c.id == mascot).unwrap();
    assert_eq!(before.power, 3);

    // Lethal damage to Quintorius (4 toughness → 4 damage kills him).
    g.battlefield_find_mut(qid).unwrap().damage = 4;
    let _ = g.check_state_based_actions();

    let after = g.compute_battlefield().into_iter()
        .find(|c| c.id == mascot).unwrap();
    assert_eq!(after.power, 2, "anthem evaporates without Quintorius");
}

#[test]
fn galvanic_iteration_copies_target_instant() {
    let mut g = two_player_game();
    // Seed cards: a Lightning Bolt as the original instant, Galvanic Iteration
    // as the copy spell.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let gi = g.add_card_to_hand(0, catalog::galvanic_iteration());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::Blue, 1);

    // Cast Bolt targeting the opponent.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("bolt casts");
    // Now cast Galvanic Iteration targeting the Bolt on the stack.
    let bolt_target = g.stack.iter().find_map(|s| match s {
        StackItem::Spell { card, .. } if card.definition.name == "Lightning Bolt" => Some(card.id),
        _ => None,
    }).expect("bolt on stack");
    g.perform_action(GameAction::CastSpell {
        card_id: gi,
        target: Some(Target::Permanent(bolt_target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("galvanic iteration casts");
    drain_stack(&mut g);

    // Opponent took 3 (original Bolt) + 3 (Galvanic Iteration copy) = 6 damage.
    assert_eq!(g.players[1].life, 20 - 6, "Galvanic Iteration copied the Bolt");
}

#[test]
fn expressive_iteration_exiles_top_three_and_grants_may_play() {
    // Push (modern_decks): Expressive Iteration now exiles the top 3
    // cards of your library (via `Effect::Move → Exile`) and grants
    // MayPlay::EndOfThisTurn on `Selector::LastMoved` so the caster
    // can play any of the 3 cards from exile this turn.
    let mut g = two_player_game();
    let top1 = g.add_card_to_library(0, catalog::plains());
    let top2 = g.add_card_to_library(0, catalog::plains());
    let top3 = g.add_card_to_library(0, catalog::plains());
    // Push another card so the library has more than 3.
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::plains());
    let initial_lib = g.players[0].library.len();
    let id = g.add_card_to_hand(0, catalog::expressive_iteration());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("EI castable");
    drain_stack(&mut g);

    // 3 cards moved from library to exile.
    assert_eq!(g.players[0].library.len(), initial_lib - 3);
    // The exiled cards are the top-3 (last-pushed-to-library by
    // `add_card_to_library` is the top — `Vec::push` semantics).
    let exiled_ids: Vec<_> = g.exile.iter().map(|c| c.id).collect();
    assert!(exiled_ids.contains(&top1));
    assert!(exiled_ids.contains(&top2));
    assert!(exiled_ids.contains(&top3));
    // All three should carry the MayPlay permission for player 0.
    for tid in &[top1, top2, top3] {
        let exiled = g.exile.iter().find(|c| c.id == *tid).unwrap();
        assert!(
            exiled.may_play_until.is_some_and(|p| p.player == 0),
            "card {:?} should have player-0 MayPlay permission",
            tid
        );
    }
}

#[test]
fn magma_opus_etb_deals_four_taps_creates_elemental_draws_two() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::plains());
    }
    let initial_hand = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::magma_opus());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(7);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Magma Opus castable for {7}{U}{R}");
    drain_stack(&mut g);

    // 4 damage destroyed the 2/2 bear via SBA.
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "bear died to 4 dmg");
    // Elemental token minted.
    let elem = g.battlefield.iter().find(|c|
        c.is_token && c.definition.name == "Elemental"
    ).expect("Elemental token minted");
    assert_eq!(elem.power(), 3, "elemental_token() is a 3/3 (sos definition)");
    // initial_hand: +1 for Magma Opus, -1 cast, +2 drawn = +2 net
    assert_eq!(g.players[0].hand.len(), initial_hand + 2,
        "drew 2 cards from Magma Opus");
}

#[test]
fn reckless_amplimancer_doubles_power_and_toughness() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::reckless_amplimancer());
    g.clear_sickness(id);
    // {4}{G}: double its 2/2 to 4/4.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None })
        .expect("Reckless Amplimancer activates {4}{G}");
    drain_stack(&mut g);

    let amp = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!((amp.power(), amp.toughness()), (4, 4), "2/2 doubled to 4/4");
}

#[test]
fn eyetwitch_brood_grows_when_another_pest_dies() {
    use crate::card::{CardDefinition, CardType, CounterType, CreatureType, Subtypes};
    let mut g = two_player_game();
    let brood = g.add_card_to_battlefield(0, catalog::eyetwitch_brood());
    // Manually add a Pest creature to the battlefield via add_card_to_battlefield
    // with a small Pest-typed definition (mirrors how tend_the_pests mints).
    let pest_def = CardDefinition {
        name: "Pest",
        cost: crate::mana::ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: crate::effect::Effect::Noop,
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    };
    let pest_id = g.add_card_to_battlefield(0, pest_def);
    g.clear_sickness(pest_id);
    // Kill the Pest with a Lightning Bolt to fire the death event.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(pest_id)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);

    let b = g.battlefield.iter().find(|c| c.id == brood).unwrap();
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1,
        "Eyetwitch Brood got a +1/+1 counter from another Pest dying");
}

#[test]
fn first_day_of_class_buffs_creatures_entering_this_turn() {
    use crate::card::{CounterType, Keyword};
    let mut g = two_player_game();
    // A creature already on the battlefield is NOT affected (only creatures
    // that enter *after* the spell resolves).
    let pre = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Library cards so FDoC's Learn (discard-to-draw fallback) doesn't deck us.
    for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::first_day_of_class());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("FDOC castable");
    drain_stack(&mut g);

    // Still player 0's main phase: cast a creature. It gets a +1/+1 counter
    // and haste as it enters.
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear castable");
    drain_stack(&mut g);

    let entered = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(entered.counter_count(CounterType::PlusOnePlusOne), 1, "entering creature gets a +1/+1 counter");
    assert!(entered.has_keyword(&Keyword::Haste), "entering creature gains haste");
    // The pre-existing bear is untouched.
    let old = g.battlefield.iter().find(|c| c.id == pre).unwrap();
    assert_eq!(old.counter_count(CounterType::PlusOnePlusOne), 0, "pre-existing creature unaffected");
}

/// Draconic Intervention — exile an I/S from your graveyard (X = its MV),
/// deal X to each non-Dragon creature; a creature that would die is exiled
/// instead. Dragons are untouched.
#[test]
fn draconic_intervention_burns_non_dragons_and_exiles_the_dead() {
    use crate::card::{CardDefinition, CardType, CreatureType, Subtypes};
    let dragon_def = |name: &'static str| CardDefinition {
        name, card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 2, toughness: 2, ..Default::default()
    };
    let mut g = two_player_game();
    // A 2-MV instant in the graveyard → X = 2.
    g.add_card_to_graveyard(0, catalog::lightning_helix());
    let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 non-Dragon → dies to 2
    let dragon = g.add_card_to_battlefield(0, dragon_def("Wyrm"));
    let di = g.add_card_to_hand(0, catalog::draconic_intervention());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: di, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Draconic Intervention castable");
    drain_stack(&mut g);

    // The 2/2 non-Dragon took 2 (lethal) → exiled instead of dying.
    assert!(g.exile.iter().any(|c| c.id == small), "lethally-damaged non-Dragon is exiled");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == small), "it did NOT go to the graveyard");
    // The Dragon is untouched.
    assert!(g.battlefield.iter().any(|c| c.id == dragon), "Dragon takes no damage");
    // Draconic Intervention exiles itself on resolve.
    assert!(g.exile.iter().any(|c| c.id == di), "Draconic Intervention exiles itself");
}

/// Fervent Mastery (regular cast) tutors up to three cards to hand (three
/// sequential searches), then discards three at random. Net: three cards
/// leave the library.
#[test]
fn fervent_mastery_tutors_three_cards_from_library() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::grizzly_bears());
    let b = g.add_card_to_library(0, catalog::grizzly_bears());
    let c = g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); } // padding
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(a)),
        DecisionAnswer::Search(Some(b)),
        DecisionAnswer::Search(Some(c)),
    ]));
    let lib_before = g.players[0].library.len();
    let id = g.add_card_to_hand(0, catalog::fervent_mastery());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fervent Mastery castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].library.len(), lib_before - 3, "three cards tutored out of the library");
    // The three searched cards are no longer in the library (they went to hand,
    // some may then be discarded at random — either way they left the library).
    for id in [a, b, c] {
        assert!(!g.players[0].library.iter().any(|c| c.id == id), "tutored card left the library");
    }
}

#[test]
fn verdant_mastery_fetches_basic_for_you_and_opponent() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let island = g.add_card_to_library(1, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
        DecisionAnswer::Search(Some(island)),
    ]));
    let id = g.add_card_to_hand(0, catalog::verdant_mastery());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Verdant Mastery castable");
    drain_stack(&mut g);

    // You should now have a Forest in play.
    assert!(g.battlefield.iter().any(|c| c.id == forest && c.controller == 0),
        "you fetched Forest");
    // Opponent fetched an Island tapped.
    assert!(g.battlefield.iter().any(|c| c.id == island && c.controller == 1),
        "opponent fetched Island");
}

#[test]
fn rip_apart_modes_kill_creature_or_artifact() {
    // Mode 0: 3 damage kills a 2/2.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::rip_apart());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Rip Apart mode 0 castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Rip Apart mode 0 killed the bear");

    // Mode 1: destroy target artifact.
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::rip_apart());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(stone)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Rip Apart mode 1 castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "Rip Apart mode 1 destroyed the Mind Stone");
}
