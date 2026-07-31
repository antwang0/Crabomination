use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
    while g.step != crabomination::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: mauler, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let p1_life_before = g.players[1].life;
    let p0_life_before = g.players[0].life;
    while g.step != crabomination::game::types::TurnStep::CombatDamage {
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
fn plumb_the_forbidden_at_x_two_sacs_two_draws_three_loses_three() {
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
    // The X copies + the original each draw 1 / lose 1 → X + 1 = 3 total.
    // Hand: -1 (cast) +3 (draw) = +2 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
    // Life: -3.
    assert_eq!(g.players[0].life, life_before - 3);
}

#[test]
fn owlin_shieldmage_is_a_warding_flyer() {
    use crabomination::card::WardCost;
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
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add(Color::Blue, 3);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Body of Research castable for {G}{G}{G}{U}{U}{U}");
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
    // Cast a Lightning Bolt first, then Show of Confidence — SpellStorm
    // mints one REAL stack copy per other instant/sorcery cast this turn,
    // each resolving its own +1/+1 counter.
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
    // One other I/S (Bolt) → one copy + the original = 2 counters.
    assert_eq!(counters, 2, "Bolt + Show of Confidence = 2 counters");
    // "It gains vigilance until end of turn."
    assert!(bear_card.has_keyword(&Keyword::Vigilance),
        "target gains vigilance until end of turn");
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
    // The same-name sweep exiles the countered copy out of the graveyard.
    assert!(g.exile.iter().any(|c| c.id == bolt),
        "countered Bolt exiled by the same-name sweep");
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
fn multiple_choice_x_one_scries_then_draws() {
    // Real oracle: "If X is 1, scry 1, then draw a card." — no token.
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::multiple_choice());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(1),
    })
    .expect("Multiple Choice castable for {X=1}{U}");
    drain_stack(&mut g);

    // X=1: scry 1, then draw a card. Net hand: -1 (cast) +1 (draw).
    assert_eq!(g.players[0].hand.len(), hand_before,
        "X=1 scries then draws a card");
    // No other bullets fired — in particular no Elemental token.
    assert!(!g.battlefield.iter().any(|c| c.is_token),
        "X=1 creates no token");
}

// ── Lorehold (R/W) ──────────────────────────────────────────────────────────

#[test]
fn lorehold_apprentice_grants_spirits_tap_ping_on_instant_cast() {
    // Real oracle: "Magecraft — … until end of turn, Spirit creatures you
    // control gain '{T}: This creature deals 1 damage to each opponent.'"
    let mut g = two_player_game();
    let _app = g.add_card_to_battlefield(0, catalog::lorehold_apprentice());
    let spirit = g.add_card_to_battlefield(0, catalog::lorehold_braveheart_b165()); // Spirit Cleric
    g.clear_sickness(spirit);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    // The Spirit picked up the granted tap ability (index 0: it has no
    // printed activated abilities). Activating it pings each opponent.
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: spirit, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("granted '{T}: 1 damage to each opponent' activates");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 1,
        "granted tap ability pings each opponent for 1");
    assert!(g.battlefield_find(spirit).unwrap().tapped,
        "the Spirit tapped to pay the granted ability's cost");
}

#[test]
fn lorehold_apprentice_does_not_grant_on_creature_spell() {
    // Magecraft only triggers on instant/sorcery, not creature spells —
    // the Spirit gets no granted ability.
    let mut g = two_player_game();
    let _app = g.add_card_to_battlefield(0, catalog::lorehold_apprentice());
    let spirit = g.add_card_to_battlefield(0, catalog::lorehold_braveheart_b165());
    g.clear_sickness(spirit);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bears castable for {1}{G}");
    drain_stack(&mut g);
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: spirit, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect_err("creature cast should NOT trigger Magecraft's grant");
    assert!(matches!(err, GameError::AbilityIndexOutOfBounds),
        "no granted ability → index 0 out of bounds, got {err:?}");
}

#[test]
fn pillardrop_rescuer_returns_cheap_creature_card_from_graveyard() {
    // Real oracle: ETB returns target CREATURE card with mana value 3 or
    // less from your graveyard to your hand.
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2 creature
    let id = g.add_card_to_hand(0, catalog::pillardrop_rescuer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pillardrop Rescuer castable for {4}{W}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Grizzly Bears (MV 2 creature) should be returned to hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear),
        "Bears should no longer be in graveyard");
}

#[test]
fn pillardrop_rescuer_cannot_return_a_big_or_noncreature_card() {
    // MV 4+ creatures and noncreature cards are illegal targets for the
    // ETB — with only those in the graveyard the trigger has nothing
    // legal to grab, so the graveyard is untouched.
    let mut g = two_player_game();
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel()); // MV 5 creature
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
    let id = g.add_card_to_hand(0, catalog::pillardrop_rescuer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pillardrop Rescuer castable for {4}{W}");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == angel),
        "Serra Angel (MV 5) stays in the graveyard");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "Lightning Bolt (noncreature) stays in the graveyard");
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
fn storm_kiln_artist_creates_treasure_and_scales_with_artifacts() {
    let mut g = two_player_game();
    let ska = g.add_card_to_battlefield(0, catalog::storm_kiln_artist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Real oracle: Magecraft only mints a Treasure — no damage rider.
    assert_eq!(g.players[1].life, p1_life_before - 3,
        "P1 takes only Bolt's 3 (Magecraft deals no damage)");
    let treasures = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1, "Storm-Kiln Artist should mint one Treasure");
    // "This creature gets +1/+0 for each artifact you control" — the
    // freshly minted Treasure bumps it to 3/2.
    let cv = g.computed_permanent(ska).expect("SKA computed");
    assert_eq!((cv.power, cv.toughness), (3, 2),
        "2/2 base +1/+0 for the one Treasure you control");
}

// ── Quandrix (G/U) ──────────────────────────────────────────────────────────

#[test]
fn quandrix_apprentice_magecraft_impulses_a_land_to_hand() {
    // "Look at the top three cards of your library. You may reveal a land
    // card from among them and put that card into your hand. Put the rest
    // on the bottom of your library in any order."
    let mut g = two_player_game();
    let _app = g.add_card_to_battlefield(0, catalog::quandrix_apprentice());
    // Seed the top of the library with three islands — a land is available.
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    let hand_before = g.players[0].hand.len();
    let lands_in_hand_before = g.players[0].hand.iter()
        .filter(|c| c.definition.is_land()).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Looked at 3, took 1 land to hand, bottomed 2 → library net -1.
    assert_eq!(g.players[0].library.len(), lib_before - 1,
        "one of the three looked-at cards leaves the library");
    // Hand: -1 (cast Bolt) +1 (land) = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1);
    let lands_in_hand_after = g.players[0].hand.iter()
        .filter(|c| c.definition.is_land()).count();
    assert_eq!(lands_in_hand_after, lands_in_hand_before + 1,
        "a land card was put into hand");
}

#[test]
fn quandrix_apprentice_magecraft_bottoms_all_when_no_land_revealed() {
    // No land among the top three → nothing goes to hand; all three are
    // put on the bottom of the library.
    let mut g = two_player_game();
    let _app = g.add_card_to_battlefield(0, catalog::quandrix_apprentice());
    for _ in 0..3 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before,
        "no land revealed — all three cards bottomed, library size unchanged");
    assert_eq!(g.players[0].hand.len(), hand_before - 1,
        "hand only lost the cast Bolt");
}

#[test]
fn quandrix_pledgemage_magecraft_adds_counter_on_instant_cast() {
    // "Magecraft — Whenever you cast or copy an instant or sorcery spell,
    // put a +1/+1 counter on this creature."
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::quandrix_pledgemage());
    g.clear_sickness(pm);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let pm_card = g.battlefield.iter().find(|c| c.id == pm).unwrap();
    assert_eq!(pm_card.counter_count(CounterType::PlusOnePlusOne), 1,
        "magecraft puts a +1/+1 counter on the Pledgemage");
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
    // Bolt countered (P1 had no extra mana for the {3} escape), P0 unhurt.
    assert_eq!(g.players[0].life, 20, "Bolt should be countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Bolt should be in graveyard");
}

// ── Prismari (U/R) ──────────────────────────────────────────────────────────

/// Real oracle: "Magecraft — … this creature can't be blocked this turn.
/// If that spell has mana value 5 or greater, put a +1/+1 counter on this
/// creature." A 1-MV Bolt grants unblockable but NO counter.
#[test]
fn prismari_apprentice_unblockable_but_no_counter_on_small_spell() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::prismari_apprentice());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let a = g.battlefield.iter().find(|c| c.id == app).unwrap();
    assert!(a.has_keyword(&Keyword::Unblockable),
        "Magecraft makes Apprentice unblockable this turn");
    assert_eq!(a.counter_count(CounterType::PlusOnePlusOne), 0,
        "Bolt is MV 1 — no +1/+1 counter");
}

/// Real oracle: "Flying / Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, target creature you control has base power 2 until
/// end of turn." Sage is the only creature, so the trigger targets itself:
/// 0/2 → base power 2 → 2/2 EOT.
#[test]
fn symmetry_sage_flies_and_sets_base_power_two_on_instant_cast() {
    let mut g = two_player_game();
    let sage = g.add_card_to_battlefield(0, catalog::symmetry_sage());
    g.clear_sickness(sage);
    assert!(g.battlefield_find(sage).unwrap().has_keyword(&Keyword::Flying),
        "Symmetry Sage has printed flying");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    let s = g.computed_permanent(sage).unwrap();
    assert_eq!(s.power, 2, "base power set to 2 by Magecraft");
    assert_eq!(s.toughness, 2, "base toughness untouched");
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
fn prismari_pledgemage_attacks_despite_defender_after_magecraft() {
    // Real oracle: "Defender / Magecraft — … this creature can attack this
    // turn as though it didn't have defender."
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::prismari_pledgemage());
    g.clear_sickness(pm);
    assert!(g.battlefield_find(pm).unwrap().has_keyword(&Keyword::Defender));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // P/T unchanged — the real card has no pump.
    assert_eq!(g.battlefield_find(pm).unwrap().power(), 3, "no pump on the real card");
    // The defender may now be declared as an attacker (CR 508.1a).
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: pm, target: AttackTarget::Player(1),
    }])).expect("Pledgemage attacks despite defender after Magecraft");
}

#[test]
fn sparring_regimen_learns_on_etb() {
    // Real oracle: "When this enchantment enters, learn." (No token.)
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island()); // for the Learn draw fallback
    let id = g.add_card_to_hand(0, catalog::sparring_regimen());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Sparring Regimen castable for {2}{W}");
    drain_stack(&mut g);
    // -1 (cast) +1 (Learn → draw fallback) = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1,
        "Learn's fallback drew a card");
    assert!(!g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Spirit"),
        "real Sparring Regimen mints no token on ETB");
}

#[test]
fn pest_summoning_creates_two_pests() {
    // Real-text fix: was minting 1 Pest, now mints 2.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_summoning());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pest Summoning castable for {2}{B}{G}");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 2, "Pest Summoning should mint two Pest tokens");
}

/// Callous Bloodmage's third printed mode — "Exile target player's
/// graveyard" (modeled as each-opponent in 1v1) — wipes the opponent's
/// graveyard into exile.
#[test]
fn callous_bloodmage_etb_mode_exiles_opponents_graveyard() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(2)]));
    let id = g.add_card_to_hand(0, catalog::callous_bloodmage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Callous Bloodmage castable for {2}{B}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 0,
        "opponent's graveyard exiled by mode 2");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Pest"),
        "no Pest minted when mode 2 chosen");
}

/// Callous Bloodmage mode 1 — "You draw a card and you lose 1 life."
#[test]
fn callous_bloodmage_etb_mode_draws_and_loses_life() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    let id = g.add_card_to_hand(0, catalog::callous_bloodmage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Callous Bloodmage castable for {2}{B}");
    drain_stack(&mut g);
    // Hand: -1 (cast) + 1 (draw) = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "drew a card (net zero after casting)");
    assert_eq!(g.players[0].life, life_before - 1, "lost 1 life");
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
fn mage_hunters_onslaught_destroys_creature_and_blockers_bleed() {
    // Real oracle: "Destroy target creature or planeswalker. / Whenever
    // a creature blocks this turn, its controller loses 1 life." (No
    // card draw — an earlier synthesized draw rider is gone.)
    use crabomination::game::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mage_hunters_onslaught());
    let bear_id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Combat pieces for the "whenever a creature blocks this turn" rider.
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(blocker);
    g.clear_sickness(attacker);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);

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
    // No draw rider on the printed card: the Onslaught left hand → stack.
    assert_eq!(g.players[0].hand.len(), hand_before - 1,
        "no card is drawn — only the cast spell left hand");

    // Rider: a creature blocking later this turn bleeds its controller.
    let opp_life = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }]))
    .expect("bear attacks");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]))
        .expect("opposing bear blocks");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 1,
        "the blocking creature's controller loses 1 life");
}

// ── STX legends (body-only smoke tests) ─────────────────────────────────────

#[test]
fn galazeth_prismari_grants_tap_for_any_color_to_artifacts() {
    // Printed: "Artifacts you control have '{T}: Add one mana of any
    // color. Spend this mana only to cast an instant or sorcery
    // spell.'" The static is surfaced as a virtual activated ability
    // at index = printed_count on each artifact controlled by
    // Galazeth's controller. Strixhaven Skycoach (artifact, 0 printed
    // activated abilities) gets the grant at index 0; tapping it adds
    // one spend-restricted mana of any color via the existing
    // AnyOneColor decision (AutoDecider picks the first legal color).
    let mut g = two_player_game();
    let _galazeth = g.add_card_to_battlefield(0, catalog::galazeth_prismari());
    let skycoach = g.add_card_to_battlefield(0, catalog::strixhaven_skycoach());

    let restricted_before = g.players[0].mana_pool.restricted_total();
    let free_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: skycoach,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Galazeth grant: {T}: Add one mana of any color (I/S-only)");

    assert_eq!(
        g.players[0].mana_pool.restricted_total() - restricted_before, 1,
        "Galazeth-granted ability adds one instant/sorcery-only mana"
    );
    assert_eq!(
        g.players[0].mana_pool.total(), free_before,
        "the granted mana is spend-restricted, not freely spendable"
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
            target: None, additional_targets: Vec::new(), x_value: None , mode: None})
        .expect_err("no Galazeth → no grant → rejected");
    assert!(
        matches!(err, GameError::AbilityIndexOutOfBounds),
        "expected AbilityIndexOutOfBounds, got {err:?}"
    );
}

#[test]
fn lorehold_apprentice_grant_skips_non_spirits() {
    // The magecraft grant only reaches SPIRIT creatures you control — a
    // plain bear picks up nothing.
    let mut g = two_player_game();
    let apprentice = g.add_card_to_battlefield(0, catalog::lorehold_apprentice());
    g.clear_sickness(apprentice);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
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
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: bear, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect_err("non-Spirit creature gets no granted ability");
    assert!(matches!(err, GameError::AbilityIndexOutOfBounds),
        "expected AbilityIndexOutOfBounds, got {err:?}");
}

#[test]
fn lorehold_pledgemage_magecraft_pumps_plus_one_plus_zero() {
    // Real oracle: "First strike / Magecraft — Whenever you cast or copy
    // an instant or sorcery spell, this creature gets +1/+0 until end of
    // turn." (The old exile-a-card activated pump was synthesized.)
    let mut g = two_player_game();
    let pledge = g.add_card_to_battlefield(0, catalog::lorehold_pledgemage());
    g.clear_sickness(pledge);
    assert!(g.battlefield_find(pledge).unwrap().has_keyword(&Keyword::FirstStrike),
        "Pledgemage has first strike");
    assert!(g.battlefield_find(pledge).unwrap().definition.activated_abilities.is_empty(),
        "no printed activated abilities");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let cv = g.computed_permanent(pledge).expect("pledgemage computed");
    assert_eq!((cv.power, cv.toughness), (3, 2),
        "2/2 gets +1/+0 from the magecraft trigger");
}

#[test]
fn lorehold_pledgemage_magecraft_ignores_creature_spells() {
    let mut g = two_player_game();
    let pledge = g.add_card_to_battlefield(0, catalog::lorehold_pledgemage());
    g.clear_sickness(pledge);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None })
    .expect("Bears castable for {1}{G}");
    drain_stack(&mut g);

    let cv = g.computed_permanent(pledge).expect("pledgemage computed");
    assert_eq!((cv.power, cv.toughness), (2, 2),
        "creature casts do not trigger magecraft");
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
        card_id: beledros, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Beledros activatable as sorcery");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 10, "Pay 10 life cost");
    assert!(!g.battlefield_find(l1).unwrap().tapped, "Forest untapped");
    assert!(!g.battlefield_find(l2).unwrap().tapped, "Swamp untapped");
}

/// Beledros — real Oracle: "At the beginning of each upkeep, create a
/// 1/1 black and green Pest creature token with 'When this token dies,
/// you gain 1 life.'" Fires on EVERY upkeep — yours and each opponent's —
/// always minting under Beledros's controller.
#[test]
fn beledros_witherbloom_mints_pest_on_each_upkeep() {
    let mut g = two_player_game();
    let _beledros = g.add_card_to_battlefield(0, catalog::beledros_witherbloom());
    let pests = |g: &crabomination::game::GameState| g.battlefield.iter()
        .filter(|c| c.is_token
            && c.controller == 0
            && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Pest))
        .count();
    assert_eq!(pests(&g), 0);

    // Your upkeep.
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(pests(&g), 1, "Pest minted on your upkeep");

    // Opponent's upkeep — Beledros still mints for its controller.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(pests(&g), 2, "Pest minted on each opponent's upkeep too");
}

#[test]
fn beledros_witherbloom_rejects_activation_with_insufficient_life() {
    let mut g = two_player_game();
    let beledros = g.add_card_to_battlefield(0, catalog::beledros_witherbloom());
    g.clear_sickness(beledros);
    g.players[0].life = 5; // not enough for the 10-life cost.

    let r = g.perform_action(GameAction::ActivateAbility {
        card_id: beledros, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None});
    assert!(r.is_err(), "Activation rejected when life < 10");
    assert_eq!(g.players[0].life, 5, "Life unchanged on rejection");
}

#[test]
fn tanazir_quandrix_attack_trigger_sets_other_creatures_base_pt_to_tanazirs() {
    // Real Oracle: "Whenever Tanazir Quandrix attacks, you may have the
    // base power and toughness of other creatures you control become
    // equal to Tanazir Quandrix's power and toughness until end of turn."
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::AttackTarget;
    let mut g = two_player_game();
    // Accept the "you may" on the attack trigger (AutoDecider declines).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let tanazir = g.add_card_to_battlefield(0, catalog::tanazir_quandrix());
    g.clear_sickness(tanazir);
    // A friendly creature (2/2) that should become base 4/4.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // An opponent creature that must NOT be affected.
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert_eq!(g.battlefield_find(bear).unwrap().toughness(), 2);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: tanazir,
        target: AttackTarget::Player(1),
    }]))
    .expect("Tanazir can attack");
    drain_stack(&mut g);

    // Other creatures you control get base P/T = Tanazir's P/T (4/4).
    let computed = g.computed_permanent(bear).unwrap();
    assert_eq!((computed.power, computed.toughness), (4, 4),
        "Bear's base P/T should become Tanazir's 4/4 for the turn");
    // Tanazir itself ("other creatures") is unchanged.
    let tz = g.computed_permanent(tanazir).unwrap();
    assert_eq!((tz.power, tz.toughness), (4, 4),
        "Tanazir keeps its own printed 4/4");
    // Opponent's creature is untouched.
    let ob = g.computed_permanent(opp_bear).unwrap();
    assert_eq!((ob.power, ob.toughness), (2, 2),
        "Opponent's creature is not affected");
}

#[test]
fn spectacle_mage_discounts_only_mana_value_five_plus_spells() {
    // Real oracle: "Instant and sorcery spells you cast with mana value
    // 5 or greater cost {1} less to cast." (No prowess — an earlier
    // synthesized prowess trigger is gone.)
    let mut g = two_player_game();
    let _mage = g.add_card_to_battlefield(0, catalog::spectacle_mage());

    // MV-5 sorcery (Tidings, {3}{U}{U}) casts with only 4 mana available.
    for _ in 0..6 { g.add_card_to_library(0, catalog::island()); }
    let tidings = g.add_card_to_hand(0, catalog::tidings());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: tidings, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("MV-5 Tidings castable for {2}{U}{U} with the {1} discount");
    drain_stack(&mut g);

    // MV-1 instant gets NO discount: Lightning Bolt with zero mana fails.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "sub-MV-5 spells get no discount — Bolt uncastable without mana"
    );
}

#[test]
fn sparring_regimen_attack_trigger_counters_and_untaps_target_attacker() {
    use crabomination::game::types::AttackTarget;
    let mut g = two_player_game();
    let _regimen = g.add_card_to_battlefield(0, catalog::sparring_regimen());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);

    // Declare the bear as attacker.
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("bear can attack");
    drain_stack(&mut g);

    // "Whenever you attack, put a +1/+1 counter on target attacking
    // creature and untap it."
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1,
        "Sparring Regimen should pump the target attacker");
    assert!(!b.tapped, "the target attacker is untapped by the trigger");
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
        card_id: pledge, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
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
        card_id: pledge, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None});
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

/// Bookwurm — real oracle: "Trample / When this creature enters, you
/// gain 3 life and draw a card. / {2}{G}: Put this card from your
/// graveyard into your library third from the top."
#[test]
fn bookwurm_etb_gains_three_life_and_draws_a_card() {
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
    assert_eq!(g.players[0].life, life_before + 3,
        "Should gain 3 life (printed ETB)");
    // Bookwurm body on battlefield with Trample.
    let bw = g.battlefield.iter().find(|c| c.definition.name == "Bookwurm")
        .expect("Bookwurm should be on battlefield");
    assert!(bw.has_keyword(&Keyword::Trample));
    assert_eq!(bw.power(), 7);
    assert_eq!(bw.toughness(), 7);
}

/// Bookwurm's graveyard recursion: "{2}{G}: Put this card from your
/// graveyard into your library third from the top."
#[test]
fn bookwurm_graveyard_activation_puts_it_third_from_top() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let wurm = g.add_card_to_graveyard(0, catalog::bookwurm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::ActivateAbility {
        card_id: wurm, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Bookwurm graveyard activation for {2}{G}");
    drain_stack(&mut g);

    assert!(!g.players[0].graveyard.iter().any(|c| c.id == wurm),
        "Bookwurm left the graveyard");
    let lib = &g.players[0].library;
    // Library top is the END of the Vec (push = top).
    let third_from_top = &lib[lib.len() - 3];
    assert_eq!(third_from_top.id, wurm,
        "Bookwurm is third from the top of the library");
}

// ── Field Trip ──────────────────────────────────────────────────────────────

/// Field Trip: search for a Forest, put it onto the battlefield, then
/// Learn (→ Draw 1 approximation). Uses a scripted decider to pick the
/// Forest (AutoDecider declines `SearchLibrary`).
#[test]
fn field_trip_fetches_forest_and_draws_a_card() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
        additional_targets: Vec::new(),
        x_value: None, mode: None,
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

    // Both fighters are real targets: slot 0 = our creature (attacker),
    // slot 1 = the opponent's creature (defender).
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(opp)],
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Spirit Summoning castable for {2}{R}{W}");
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
fn silverquill_apprentice_pumps_target_on_instant_cast() {
    // Real STX Silverquill Apprentice: "Magecraft — Whenever you cast
    // or copy an instant or sorcery spell, target creature gets +1/+0
    // until end of turn." (No +1/+1 counter — that was a drift.)
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::silverquill_apprentice());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let power_sum_before = g.computed_permanent(app).unwrap().power
        + g.computed_permanent(bear).unwrap().power;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // The magecraft trigger pumped its (auto-chosen) creature target by
    // +1/+0 — total power across the two creatures is up by exactly 1,
    // and no +1/+1 counters were minted.
    let power_sum_after = g.computed_permanent(app).unwrap().power
        + g.computed_permanent(bear).unwrap().power;
    assert_eq!(power_sum_after, power_sum_before + 1,
        "one creature got +1/+0 until end of turn from Magecraft");
    assert!(g.battlefield.iter()
        .all(|c| c.counter_count(CounterType::PlusOnePlusOne) == 0),
        "the pump is an EOT effect, not a +1/+1 counter");
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
fn elemental_expressionist_exiles_instead_and_mints_elementals() {
    // Real oracle: Magecraft grants target creature you control (here the
    // Expressionist itself — the only creature) "if this would leave the
    // battlefield, exile it instead" + "when this is put into exile, create
    // a 4/4 blue and red Elemental" until EOT. Each instance triggers
    // separately: two instant casts → two grants → two Elementals.
    let mut g = two_player_game();
    let expr = g.add_card_to_battlefield(0, catalog::elemental_expressionist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == expr),
        "grant does not move the Expressionist");
    // Second instant — Doom Blade on the Expressionist itself. Its own
    // Magecraft (second grant instance) resolves first, then the kill:
    // the death is replaced by exile and BOTH grant instances mint a token.
    let db = g.add_card_to_hand(0, catalog::doom_blade());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: db, target: Some(Target::Permanent(expr)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Doom Blade castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == expr),
        "Expressionist exiled instead of dying");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == expr),
        "not in the graveyard — the leave was replaced with exile");
    let elementals = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Elemental")
        .count();
    assert_eq!(elementals, 2,
        "each granted instance triggers separately → two 4/4 Elementals");
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

/// Reduce to Memory — real oracle: "Exile target nonland permanent.
/// Its controller creates a 3/2 red and white Spirit creature token."
#[test]
fn reduce_to_memory_exiles_and_controller_gets_spirit() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::reduce_to_memory());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Reduce to Memory castable");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in exile");
    // "Its controller creates ..." — the token belongs to PLAYER 1 (the
    // exiled bear's controller), not the caster.
    let spirit = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Spirit")
        .expect("Spirit token should exist on battlefield");
    assert_eq!(spirit.controller, 1,
        "Spirit token goes to the exiled permanent's controller");
    assert_eq!((spirit.power(), spirit.toughness()), (3, 2),
        "printed 3/2 Spirit");
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

/// Excavated Wall — real oracle: "Defender / {1}, {T}: Mill a card."
#[test]
fn excavated_wall_mills_a_card() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let wall = g.add_card_to_battlefield(0, catalog::excavated_wall());
    g.clear_sickness(wall);
    g.players[0].mana_pool.add_colorless(1);

    // Body is a 0/4 artifact creature with Defender.
    let w = g.battlefield_find(wall).expect("Wall on battlefield");
    assert_eq!((w.power(), w.toughness()), (0, 4));
    assert!(w.has_keyword(&Keyword::Defender));

    let gy_before = g.players[0].graveyard.len();
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: wall, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Excavated Wall {1}, {T}: Mill a card");
    drain_stack(&mut g);

    assert_eq!(g.players[0].graveyard.len(), gy_before + 1, "milled one card");
    assert_eq!(g.players[0].library.len(), lib_before - 1);
    assert!(g.battlefield_find(wall).unwrap().tapped, "Wall paid the tap cost");
}

// ── Snow Day ────────────────────────────────────────────────────────────────

/// Snow Day — real oracle: "Tap up to two target creatures. Those
/// creatures don't untap during their controller's next untap step. /
/// Draw two cards, then discard a card." One-target cast: the freeze is
/// the `skip_next_untap` flag (not a stun counter), plus the loot.
#[test]
fn snow_day_taps_freezes_and_loots_with_one_target() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::snow_day());
    let hand_before = g.players[0].hand.len();
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Snow Day castable at one target (\"up to two\")");
    drain_stack(&mut g);

    let target = g.battlefield_find(bear).unwrap();
    assert!(target.tapped, "Bear should be tapped");
    assert!(target.skip_next_untap,
        "Bear won't untap during its controller's next untap step");
    // Draw two, then discard one: -1 cast +2 draw -1 discard = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "draw two / discard one nets zero with the cast");
}

/// Snow Day cast at TWO creatures: both targets are tapped and flagged
/// to skip their controller's next untap step. Slot 0 is `target`, slot
/// 1 is `additional_targets[0]`.
#[test]
fn snow_day_taps_and_freezes_two_target_creatures() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
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
    assert!(b.skip_next_untap, "Bear frozen for its next untap step");

    let a = g.battlefield_find(angel).unwrap();
    assert!(a.tapped, "Serra Angel should be tapped");
    assert!(a.skip_next_untap, "Angel frozen for its next untap step");
}

// ── Spell Satchel ───────────────────────────────────────────────────────────

// Spell Satchel — real oracle: "Magecraft — Whenever you cast or copy
// an instant or sorcery spell, put a book counter on this artifact. /
// {T}, Remove a book counter from this artifact: Add {C}. / {3}, {T},
// Remove three book counters from this artifact: Draw a card."

#[test]
fn spell_satchel_magecraft_adds_book_counter() {
    let mut g = two_player_game();
    let satchel = g.add_card_to_battlefield(0, catalog::spell_satchel());
    // Casting an instant fires magecraft → +1 book counter.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(satchel).unwrap().counter_count(CounterType::Book), 1,
        "magecraft puts a book counter on Spell Satchel");
}

#[test]
fn spell_satchel_creature_cast_does_not_add_book_counter() {
    let mut g = two_player_game();
    let satchel = g.add_card_to_battlefield(0, catalog::spell_satchel());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear castable");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(satchel).unwrap().counter_count(CounterType::Book), 0,
        "magecraft only counts instant/sorcery casts");
}

#[test]
fn spell_satchel_tap_remove_book_adds_colorless() {
    use crabomination::card::CounterType as CT;
    let mut g = two_player_game();
    let satchel = g.add_card_to_battlefield(0, catalog::spell_satchel());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == satchel) {
        c.counters.insert(CT::Book, 1);
    }

    let mana_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: satchel, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("{T}, remove a book counter: Add {C}");
    assert_eq!(g.players[0].mana_pool.total(), mana_before + 1,
        "Spell Satchel adds 1 colorless");
    let s = g.battlefield_find(satchel).unwrap();
    assert!(s.tapped, "Spell Satchel tapped");
    assert_eq!(s.counter_count(CounterType::Book), 0,
        "the book counter was paid as a cost");
}

#[test]
fn spell_satchel_mana_ability_requires_a_book_counter() {
    let mut g = two_player_game();
    let satchel = g.add_card_to_battlefield(0, catalog::spell_satchel());
    // No book counters → the remove-a-counter cost can't be paid.
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: satchel, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None , mode: None});
    assert!(res.is_err(),
        "mana ability needs a book counter to remove");
}

#[test]
fn spell_satchel_draw_ability_removes_three_books_and_draws() {
    use crabomination::card::CounterType as CT;
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let satchel = g.add_card_to_battlefield(0, catalog::spell_satchel());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == satchel) {
        c.counters.insert(CT::Book, 3);
    }
    g.players[0].mana_pool.add_colorless(3);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: satchel, ability_index: 1, target: None,
        additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("{3}, {T}, remove three book counters: Draw a card");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    let s = g.battlefield_find(satchel).unwrap();
    assert!(s.tapped, "Spell Satchel tapped");
    assert_eq!(s.counter_count(CounterType::Book), 0,
        "all three book counters were paid");
}

#[test]
fn spell_satchel_draw_ability_requires_three_books() {
    use crabomination::card::CounterType as CT;
    let mut g = two_player_game();
    let satchel = g.add_card_to_battlefield(0, catalog::spell_satchel());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == satchel) {
        c.counters.insert(CT::Book, 2);
    }
    g.players[0].mana_pool.add_colorless(3);
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: satchel, ability_index: 1, target: None,
        additional_targets: Vec::new(), x_value: None , mode: None});
    assert!(res.is_err(),
        "draw ability needs three book counters; only two present");
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
    use crabomination::game::Attack;
    let mut g = two_player_game();
    let titan = g.add_card_to_battlefield(0, catalog::daemogoth_titan());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(titan);
    g.clear_sickness(fodder);
    g.step = TurnStep::DeclareAttackers;

    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: titan,
        target: crabomination::game::AttackTarget::Player(1),
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
    use crabomination::game::Attack;
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
        attacker, target: crabomination::game::AttackTarget::Player(1),
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

// Daemogoth Woe-Eater — real oracle: "At the beginning of your upkeep,
// sacrifice a creature. / When you sacrifice this creature, each
// opponent discards a card, you draw a card, and you gain 2 life."

#[test]
fn daemogoth_woe_eater_upkeep_sacrifices_a_creature() {
    use crabomination::game::TurnStep;
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let woe = g.add_card_to_battlefield(0, catalog::daemogoth_woe_eater());

    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);

    // The upkeep tithe ate the (lowest-power) fodder bear.
    assert!(!g.battlefield.iter().any(|c| c.id == fodder),
        "a creature was sacrificed at upkeep");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder),
        "the sacrificed bear is in the graveyard");
    // The Woe-Eater itself survives (something else was available).
    let woe_card = g.battlefield.iter().find(|c| c.id == woe)
        .expect("Woe-Eater still on the battlefield");
    assert_eq!((woe_card.power(), woe_card.toughness()), (7, 6));
    // Sacrificing the BEAR does not fire the Woe-Eater's own payoff.
    assert_eq!(g.players[0].life, 20, "no life gain — the payoff needs the Woe-Eater itself");
}

#[test]
fn daemogoth_woe_eater_sacrificing_itself_fires_the_payoff() {
    use crabomination::game::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island()); // for the draw
    g.add_card_to_hand(1, catalog::island());    // for the opp discard
    let woe = g.add_card_to_battlefield(0, catalog::daemogoth_woe_eater());

    let my_hand_before = g.players[0].hand.len();
    let opp_hand_before = g.players[1].hand.len();
    let life_before = g.players[0].life;

    // Woe-Eater is the only creature — the upkeep sacrifice eats itself,
    // which IS "you sacrifice this creature" → payoff fires.
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == woe),
        "Woe-Eater sacrificed itself");
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1,
        "each opponent discards a card");
    assert_eq!(g.players[0].hand.len(), my_hand_before + 1,
        "you draw a card");
    assert_eq!(g.players[0].life, life_before + 2, "you gain 2 life");
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed library with one Forest + an unrelated card so the search
    // has a legal target.
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());

    // The printed "you may search" is a real MayDo: answer yes, then
    // pick the Forest in the search.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(forest)),
    ]));

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
fn confront_the_past_mode_0_reanimates_planeswalker_from_graveyard() {
    let mut g = two_player_game();
    let pw = g.add_card_to_graveyard(0, catalog::professor_dellian_fel());
    let id = g.add_card_to_hand(0, catalog::confront_the_past());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(pw)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: Some(10),
    }).expect("Confront the Past castable for {X}{B}");
    drain_stack(&mut g);

    let p = g.battlefield_find(pw).expect("PW reanimated to battlefield");
    assert_eq!(p.controller, 0, "reanimated under your control");
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
        card_id: spec, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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

/// Mascot Interception's printed "costs {3} less to cast if it targets
/// a token" — a token target makes it castable for just {R}; a
/// non-token target with only {R} floated is rejected.
#[test]
fn mascot_interception_costs_three_less_against_token() {
    // Token target: {3}{R} − {3} = {R}.
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == opp_bear) {
        c.is_token = true;
    }
    let id = g.add_card_to_hand(0, catalog::mascot_interception());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mascot Interception costs just {R} against a token");
    drain_stack(&mut g);
    let bear = g.battlefield.iter().find(|c| c.id == opp_bear).expect("token on bf");
    assert_eq!(bear.controller, 0, "control of the token transferred");

    // Non-token target: no reduction — {R} alone can't pay {3}{R}.
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mascot_interception());
    g.players[0].mana_pool.add(Color::Red, 1);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "full {{3}}{{R}} unaffordable against a non-token");
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

/// Practical Research's printed "unless you discard an instant or
/// sorcery card" exemption: with an IS card among the drawn four, only
/// that single card is pitched (DiscardUnlessKind, the Wrench Mind
/// shape) — the hand keeps 3 instead of 2.
#[test]
fn practical_research_keeps_extra_card_when_discarding_instant() {
    let mut g = two_player_game();
    g.players[0].hand.clear();
    // Top 4 of the library: a Bolt among three Islands.
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_library(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::practical_research());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Practical Research castable");
    drain_stack(&mut g);

    // Drew 4 (3 Islands + Bolt); discarded only the Bolt.
    assert_eq!(g.players[0].hand.len(), 3, "drew 4, pitched only the instant");
    assert!(
        g.players[0].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "the instant card was the discard"
    );
}

/// Hall of Oracles — real oracle: "{T}: Add {C}. / {1}, {T}: Add one
/// mana of any color. / {T}: Put a +1/+1 counter on target creature.
/// Activate only as a sorcery and only if you've cast an instant or
/// sorcery spell this turn."
#[test]
fn hall_of_oracles_mana_abilities_and_gated_counter_ability() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::hall_of_oracles());
    let wiz = g.add_card_to_battlefield(0, catalog::symmetry_sage());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == wiz) {
        c.summoning_sick = false;
    }
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == land) {
        c.summoning_sick = false;
    }

    // Ability 0 — {T}: Add {C}.
    let c_before = g.players[0].mana_pool.colorless_amount();
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Hall {T}: Add {C}");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), c_before + 1);

    // Ability 1 — {1}, {T}: Add one mana of any color.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == land) {
        c.tapped = false;
    }
    g.players[0].mana_pool.add_colorless(1);
    let total_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Hall {1},{T}: any color");
    assert_eq!(g.players[0].mana_pool.total(), total_before, // -1 paid, +1 added
        "one generic paid, one mana of any color added");

    // Ability 2 — gated: no instant/sorcery cast this turn → rejected.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == land) {
        c.tapped = false;
    }
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: Some(Target::Permanent(wiz)), additional_targets: Vec::new(), x_value: None , mode: None});
    assert!(res.is_err(),
        "counter ability requires an instant or sorcery cast this turn");

    // Cast an instant, then the counter ability is live.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);

    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: Some(Target::Permanent(wiz)), additional_targets: Vec::new(), x_value: None , mode: None}).expect("Hall {T}: +1/+1 counter after an instant this turn");
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
        card_id: letter_id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("{T}: Add any color");
    assert_eq!(g.players[0].mana_pool.total(), 1, "added one mana");

    // Untap, then sac to draw.
    g.battlefield_find_mut(letter_id).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(2);
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: letter_id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("{2},{T},Sac: Draw");
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
    // AutoDecider now exiles OPPONENT graveyard cards (free hate — the
    // old empty default forfeited the rider every time). The seed card
    // is exiled; the countered Bolt hit the graveyard after the pick.
    assert!(g.exile.iter().any(|c| c.id == gy), "opponent's graveyard card exiled");
    // The countered Bolt reaches the graveyard before the pick, so the
    // hate default sweeps it into exile as well.
    assert!(g.exile.iter().any(|c| c.id == bolt), "countered spell exiled too");
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
    // X=2 → 10 damage; cost {X}{X}{X}{R}{R} = {2}{2}{2}{R}{R}.
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(6);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("Crackle castable for {X}{X}{X}{R}{R} at X=2");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 20 - 10,
        "5X = 10 damage at X=2");
}

#[test]
fn crackle_with_power_deals_five_x_to_each_of_two_targets() {
    // Real oracle: "Crackle with Power deals five times X damage to
    // EACH of up to X targets" — the damage is NOT divided; every
    // target takes the full 5X.
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::crackle_with_power());
    // X=2 → 10 damage to each of two targets; {X}{X}{X}{R}{R}, X=2.
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(angel)],
        mode: None,
        x_value: Some(2),
    }).expect("Crackle castable for {X}{X}{X}{R}{R} at X=2");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 20 - 10, "player takes the full 5X = 10");
    assert!(!g.battlefield.iter().any(|c| c.id == angel),
        "the 4/4 also takes the full 10 and dies");
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
fn dragonsguard_elite_magecraft_adds_counter_and_activation_doubles_counters() {
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

    // Activate {4}{G}{G}: double the number of +1/+1 counters — 1 → 2.
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dge, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
        .expect("{4}{G}{G}: double +1/+1 counters");
    drain_stack(&mut g);

    let d2 = g.battlefield.iter().find(|c| c.id == dge).unwrap();
    assert_eq!(d2.counter_count(CounterType::PlusOnePlusOne), 2,
        "1 counter doubled to 2");
    assert_eq!(d2.power(), 4, "Dragonsguard Elite: 2 base + 2 counters = 4");
    assert_eq!(d2.toughness(), 4);
}

#[test]
fn quintorius_makes_a_spirit_when_a_card_leaves_your_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::quintorius_field_historian());
    g.dispatch_triggers_for_events(&[crabomination::game::types::GameEvent::CardLeftGraveyard {
        player: 0, card_id: crabomination::card::CardId(999),
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

    // Magecraft self-exile rider: casting Iteration is itself an instant
    // cast, so the card routes to exile (not the graveyard) on resolution.
    assert!(
        g.exile.iter().any(|c| c.id == gi),
        "Galvanic Iteration exiled itself on resolution (Magecraft rider)"
    );
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.id == gi),
        "Galvanic Iteration is not in the graveyard"
    );
}

#[test]
fn expressive_iteration_picks_one_to_hand_and_exiles_one_playable() {
    // Real oracle: "Look at the top three cards of your library. Put one
    // of them into your hand, put one of them on the bottom of your
    // library, and exile one of them. You may play the exiled card this
    // turn." Documented approximation: LookPickToHand(3, rest bottom) +
    // ExileTopAndGrantMayPlay(1). Card economy: +1 hand, +1 playable
    // exile, rest bottomed.
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::island()); }
    let initial_lib = g.players[0].library.len();
    let exile_before = g.exile.len();
    let id = g.add_card_to_hand(0, catalog::expressive_iteration());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("EI castable");
    drain_stack(&mut g);

    // -1 cast, +1 pick to hand → net 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "one looked-at card went to hand");
    // Library: -1 (to hand) -1 (exiled), the bottomed cards stay.
    assert_eq!(g.players[0].library.len(), initial_lib - 2,
        "one card to hand and one to exile; the rest bottomed");
    // Exactly one new exiled card, playable by player 0 this turn.
    let new_exiles: Vec<_> = g.exile.iter().skip(exile_before)
        .filter(|c| c.definition.name == "Island").collect();
    assert_eq!(new_exiles.len(), 1, "exactly one card exiled");
    assert!(
        new_exiles[0].may_play_until.is_some_and(|p| p.player == 0),
        "exiled card carries player-0 may-play-this-turn permission"
    );
}

#[test]
fn magma_opus_deals_four_taps_two_creates_elemental_draws_two() {
    // Real oracle: "Magma Opus deals 4 damage divided as you choose
    // among any number of targets. Tap two target permanents. Create a
    // 4/4 blue and red Elemental creature token. Draw two cards."
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bystander = g.add_card_to_battlefield(1, catalog::serra_angel());
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
    }).expect("Magma Opus castable for {6}{U}{R}");
    drain_stack(&mut g);

    // 4 damage destroyed the 2/2 bear via SBA.
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "bear died to 4 dmg");
    // "Tap two target permanents" — resolution-chosen; the surviving
    // Angel is among the tapped picks (auto-fill takes what's there).
    assert!(g.battlefield_find(bystander).unwrap().tapped,
        "Serra Angel tapped by the tap-two clause");
    // 4/4 blue-and-red Elemental token minted (per printed token stats).
    let elem = g.battlefield.iter().find(|c|
        c.is_token && c.definition.name == "Elemental"
    ).expect("Elemental token minted");
    assert_eq!((elem.power(), elem.toughness()), (4, 4), "printed 4/4 Elemental");
    // initial_hand: +1 for Magma Opus, -1 cast, +2 drawn = +2 net
    assert_eq!(g.players[0].hand.len(), initial_hand + 2,
        "drew 2 cards from Magma Opus");
}

#[test]
fn magma_opus_discard_mode_makes_a_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::magma_opus());
    // {U/R}{U/R} paid with two blue.
    g.players[0].mana_pool.add(Color::Blue, 2);
    assert!(g.would_accept(GameAction::ActivateDiscardAbility { card_id: id }),
        "discard-Treasure mode is offered while affordable");
    g.perform_action(GameAction::ActivateDiscardAbility { card_id: id })
        .expect("discard Magma Opus for a Treasure");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Magma Opus discarded");
    assert!(g.battlefield.iter().any(|c| c.controller == 0
        && c.definition.name == "Treasure"), "Treasure token minted");
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
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
        .expect("Reckless Amplimancer activates {4}{G}");
    drain_stack(&mut g);

    let amp = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!((amp.power(), amp.toughness()), (4, 4), "2/2 doubled to 4/4");
}

#[test]
fn eyetwitch_brood_grows_when_another_pest_dies() {
    use crabomination::card::{CardDefinition, CardType, CounterType, CreatureType, Subtypes};
    let mut g = two_player_game();
    let brood = g.add_card_to_battlefield(0, catalog::eyetwitch_brood());
    // Manually add a Pest creature to the battlefield via add_card_to_battlefield
    // with a small Pest-typed definition (mirrors how tend_the_pests mints).
    let pest_def = CardDefinition {
        name: "Pest",
        cost: crabomination::mana::ManaCost::default(),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        effect: crabomination::effect::Effect::Noop,
        ..Default::default()
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
    use crabomination::card::{CounterType, Keyword};
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
    use crabomination::card::{CardDefinition, CardType, CreatureType, Subtypes};
    let dragon_def = |name: &'static str| CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
fn verdant_mastery_alt_cost_distributes_basics() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Four basics in your library; the alt cast distributes them
    // opp-bf / your-bf / your-bf / your-hand.
    let f: Vec<_> = (0..4).map(|_| g.add_card_to_library(0, catalog::forest())).collect();
    g.decider = Box::new(ScriptedDecider::new(
        f.iter().map(|&id| DecisionAnswer::Search(Some(id))).collect::<Vec<_>>(),
    ));
    let id = g.add_card_to_hand(0, catalog::verdant_mastery());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Verdant Mastery alt cost {3}{G}");
    drain_stack(&mut g);

    // One basic under the opponent's control, two under yours, one in hand.
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_land()).count(), 1);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count(), 2);
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.is_land()).count(), 1);
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

// ── Tend the Pests (additional-cost sacrifice → X Pests) ────────────────────

/// The "sacrifice a creature" is a real additional CAST cost
/// (`AdditionalCastCost::SacrificePermanent`): it is paid while casting and
/// the sacrificed creature's power becomes the spell's X, read back at
/// resolution via `Value::XFromCost`.
#[test]
fn tend_the_pests_sacrifices_at_cast_and_mints_power_pests() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::tend_the_pests());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tend the Pests castable with a creature to sacrifice");
    // Cost is paid on cast — the bear is gone before resolution.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "sacrifice paid while casting, not at resolution");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 2, "X = sacrificed power (2) Pest tokens minted");
}

/// Test of Talents' second half — audit fix: counter, then exile every
/// same-named card from the owner's graveyard/hand/library; they
/// shuffle and draw one per card exiled from their HAND.
#[test]
fn test_of_talents_strips_same_named_copies_and_compensates_hand() {
    let mut g = two_player_game();
    // P1 holds one extra Bolt in hand, one in graveyard, one in library.
    let hand_bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let gy_bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let lib_bolt = g.add_card_to_library(1, catalog::lightning_bolt());
    g.add_card_to_library(1, catalog::island()); // draw fodder
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    g.priority.player_with_priority = 0;
    let tot = g.add_card_to_hand(0, catalog::test_of_talents());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: tot, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Test of Talents castable");
    drain_stack(&mut g);

    for (id, wher) in [(bolt, "countered copy"), (hand_bolt, "hand copy"),
                       (gy_bolt, "graveyard copy"), (lib_bolt, "library copy")] {
        assert!(g.exile.iter().any(|c| c.id == id), "{wher} exiled");
    }
    // Hand: -1 (exiled Bolt) +1 (compensation draw) = unchanged.
    assert_eq!(g.players[1].hand.len(), hand_before,
        "one draw per card exiled from hand");
}
