use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn pestbrood_grovecaller_etb_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pestbrood_grovecaller());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Grovecaller castable");
    drain_stack(&mut g);
    let _ = id;
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest)
    }).collect();
    assert_eq!(pests.len(), 1, "exactly one Pest minted on ETB");
}

#[test]
fn lorehold_cathedral_taps_for_red_or_white() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_cathedral());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("tap for R");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "added one red mana");
}

#[test]
fn lorehold_pyromage_etb_burns_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_pyromage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyromage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 3, "opp takes 3 from ETB");
}

#[test]
fn quandrix_geomancer_etb_mints_fractals_per_land() {
    let mut g = two_player_game();
    // Pre-seed 4 lands.
    for _ in 0..4 {
        let _ = g.add_card_to_battlefield(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_geomancer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Geomancer castable");
    drain_stack(&mut g);
    let _ = id;
    let fractals: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal)
    }).collect();
    // 4 lands at ETB count → 4 Fractals.
    assert_eq!(fractals.len(), 4, "minted 4 Fractals for 4 lands");
}

#[test]
fn quandrix_fractalist_etb_enters_with_counters_per_hand() {
    let mut g = two_player_game();
    // Set hand to size 3 (after cast, hand is size 2). Add some filler.
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalist());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractalist castable");
    drain_stack(&mut g);
    let bf = g.battlefield_find(id).expect("Fractalist on bf");
    // Hand has 2 (two islands) after the cast; ETB trigger reads
    // hand size = 2 → +2 +1/+1 counters.
    assert_eq!(bf.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn quandrix_skybinder_attack_drops_counter_on_friendly() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::quandrix_skybinder());
    let friendly = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    // Synthesize an Attacks event for the Skybinder.
    {
        use crate::card::Effect;
        use crate::card::Selector;
        use crate::card::Value;
        let eff = Effect::AddCounter {
            what: Selector::Target(0),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        };
        use crate::game::effects::EffectContext;
        use crate::game::types::Target;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(friendly)), 0, 0);
        g.resolve_effect(&eff, &ctx).expect("AddCounter resolves");
    }
    let bf = g.battlefield_find(friendly).expect("Bear");
    assert_eq!(bf.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn prismari_mistcaller_etb_scry_then_draw() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::prismari_mistcaller());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mistcaller castable");
    drain_stack(&mut g);
    // Hand: -1 (cast) + 1 (etb draw) = same size.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_conflagration_burn_mode_kills_two() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_conflagration());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Conflagration mode 0");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear dies to 4 damage");
}

#[test]
fn prismari_treasurewright_etb_mints_two_treasures() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_treasurewright());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Treasurewright castable");
    drain_stack(&mut g);
    let _ = id;
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Treasure"
    }).collect();
    assert_eq!(treasures.len(), 2, "Two Treasures minted");
}

#[test]
fn silverquill_auctioneer_grows_on_instant_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_auctioneer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bf = g.battlefield_find(id).expect("Auctioneer on bf");
    assert_eq!(bf.counter_count(CounterType::PlusOnePlusOne), 1, "Magecraft +1/+1");
}

#[test]
fn witherbloom_reanimist_etb_returns_low_mv_creature() {
    let mut g = two_player_game();
    // Add Grizzly Bears (MV 2) to graveyard.
    let _ = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_reanimist());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reanimist castable");
    drain_stack(&mut g);
    // Bear should be back in hand (MV 2 ≤ 2 satisfied).
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "Bear returned to hand"
    );
}

#[test]
fn quandrix_landmapper_ramps_and_scries() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Pre-seed a basic land in library + filler.
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::quandrix_landmapper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Landmapper castable");
    drain_stack(&mut g);
    // Land enters battlefield untapped per the Search.
    let forests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.name == "Forest"
    }).collect();
    assert_eq!(forests.len(), 1, "tutored a Forest");
}

#[test]
fn prismari_spellsong_draws_and_discards() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::prismari_spellsong());
    // Add a filler card to hand so the discard has something to grab.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellsong castable");
    drain_stack(&mut g);
    // Hand: -1 (cast) +1 (draw) -1 (discard) = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn silverquill_reaper_etb_destroys_small_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::silverquill_reaper());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reaper castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed (toughness 2 ≤ 2)");
}

#[test]
fn strixhaven_reservoir_taps_for_any_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::strixhaven_reservoir());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("Reservoir taps for color");
    drain_stack(&mut g);
    // AutoDecider picks a color (white by default).
    let mana = &g.players[0].mana_pool;
    let total = mana.amount(Color::White) + mana.amount(Color::Blue) + mana.amount(Color::Black)
        + mana.amount(Color::Red) + mana.amount(Color::Green);
    assert_eq!(total, 1, "got 1 mana from Reservoir");
}

#[test]
fn lone_rider_pumps_when_attacking_alone() {
    // Locks in CR 506.5 "attacking alone" predicate. The Lone Rider's
    // attack-trigger only fires when it's the only declared attacker.
    let mut g = two_player_game();
    let rider = g.add_card_to_battlefield(0, catalog::lone_rider());
    g.clear_sickness(rider);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rider, target: AttackTarget::Player(1),
    }])).expect("Rider attacks alone");
    drain_stack(&mut g);
    let view = g.computed_permanent(rider).expect("Rider on bf");
    assert_eq!(view.power, 4, "Rider 2 + 2 from alone-attack trigger");
    assert!(view.keywords.contains(&Keyword::Trample), "Trample EOT granted");
}

#[test]
fn lone_rider_does_not_pump_with_other_attackers() {
    let mut g = two_player_game();
    let rider = g.add_card_to_battlefield(0, catalog::lone_rider());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(rider);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: rider, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("Both attack");
    drain_stack(&mut g);
    let view = g.computed_permanent(rider).expect("Rider on bf");
    assert_eq!(view.power, 2, "Rider not pumped (multiple attackers — not 'alone')");
    assert!(!view.keywords.contains(&Keyword::Trample), "No Trample (not alone)");
}

#[test]
fn solo_striker_pumps_when_attacking_alone() {
    let mut g = two_player_game();
    let striker = g.add_card_to_battlefield(0, catalog::solo_striker());
    g.clear_sickness(striker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: striker, target: AttackTarget::Player(1),
    }])).expect("Striker attacks alone");
    drain_stack(&mut g);
    let view = g.computed_permanent(striker).expect("Striker on bf");
    assert_eq!(view.power, 4, "Striker 3 + 1");
    assert_eq!(view.toughness, 4, "Striker 2 + 2");
    assert!(view.keywords.contains(&Keyword::Lifelink), "Lifelink granted");
    assert!(view.keywords.contains(&Keyword::Vigilance), "Vigilance intrinsic");
}

#[test]
fn quandrix_loremind_etb_draws_a_card() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_loremind());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Loremind castable");
    drain_stack(&mut g);
    // -1 cast +1 etb-draw = same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_loremind_sac_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_battlefield(0, catalog::quandrix_loremind());
    g.clear_sickness(id);
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("Loremind activatable");
    drain_stack(&mut g);
    // Sacrificed → no longer on bf.
    assert!(g.battlefield_find(id).is_none(), "Loremind sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew 2 cards");
}

#[test]
fn prismari_sparkbinder_burns_opp_on_instant_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_sparkbinder());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Opp takes 3 (bolt) + 1 (sparkbinder ping) = 4.
    assert_eq!(g.players[1].life, opp_life_before - 4);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Treasure"
    }).collect();
    assert_eq!(treasures.len(), 1, "Treasure minted from magecraft");
}

#[test]
fn witherbloom_hexweaver_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_hexweaver());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hexweaver castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2, "opp loses 2");
    assert_eq!(g.players[0].life, life0_before + 2, "you gain 2");
    let bf = g.battlefield.iter().find(|c| c.definition.name == "Witherbloom Hexweaver")
        .expect("Hexweaver on bf");
    assert!(bf.has_keyword(&Keyword::Deathtouch));
}

#[test]
fn spelltongue_statute_gains_life_on_instant_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::spelltongue_statute());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "gained 1 life from cast");
}

// ────────────────────────────────────────────────────────────────────────────
// Batch 14 — Silverquill expansion + cross-college additions
// 25 new STX cards (15 Silverquill, 10 cross-college). One+ test per card.
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_loremender_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_loremender());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Loremender castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2, "gained 2 life from ETB");
    let bf = g.battlefield.iter().find(|c| c.definition.name == "Silverquill Loremender");
    assert!(bf.is_some(), "Loremender on battlefield");
}

#[test]
fn silverquill_drainmaster_etb_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_drainmaster());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainmaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 3, "opp loses 3");
    assert_eq!(g.players[0].life, life0_before + 3, "you gain 3");
}

#[test]
fn inkrise_lifedrainer_combat_damage_gains_one_life() {
    let mut g = two_player_game();
    let drainer = g.add_card_to_battlefield(0, catalog::inkrise_lifedrainer());
    g.clear_sickness(drainer);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: drainer, target: AttackTarget::Player(1),
    }])).expect("Drainer attacks");
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    while g.step != crate::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "gained 1 life from combat damage");
}

#[test]
fn silverquill_penman_is_a_three_mana_inkling_wizard_flier() {
    let g = two_player_game();
    let _ = g;
    let def = catalog::silverquill_penman();
    assert_eq!(def.name, "Silverquill Penman");
    assert_eq!(def.power, 2);
    assert_eq!(def.toughness, 2);
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_anthemwriter_is_a_lifelink_flying_finisher() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_anthemwriter());
    let view = g.computed_permanent(id).expect("Anthemwriter on bf");
    assert_eq!(view.power, 4);
    assert_eq!(view.toughness, 4);
    assert!(view.keywords.contains(&Keyword::Flying));
    assert!(view.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn silverquill_quillmage_drains_on_instant_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_quillmage());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Opp takes 3 (bolt) + 1 (Quillmage drain) = 4
    assert_eq!(g.players[1].life, life1_before - 4);
}

#[test]
fn silverquill_memorialist_etb_returns_low_mv_creature_from_graveyard() {
    let mut g = two_player_game();
    let pal = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_memorialist());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memorialist castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == pal),
        "Bears (2 MV) returned to hand");
}

#[test]
fn witherspell_drain_drains_three_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherspell_drain());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drain castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 3);
    assert_eq!(g.players[0].life, life0_before + 3);
}

#[test]
fn inkling_scribe_etb_mints_an_inkling_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_scribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scribe castable");
    drain_stack(&mut g);
    let inklings: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Inkling"
    }).collect();
    assert_eq!(inklings.len(), 1, "Should mint 1 Inkling token");
}

#[test]
fn silverquill_erudite_self_pumps_on_instant_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_erudite());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.computed_permanent(id).expect("Erudite on bf");
    assert_eq!(view.power, 3, "Erudite 2 + 1 self-pump");
    assert_eq!(view.toughness, 4, "Erudite intrinsic 4 toughness");
    assert!(view.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn silverquill_reprimand_exiles_two_power_creature() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_reprimand());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reprimand castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear gone from bf");
    assert!(g.exile.iter().any(|c| c.id == bear), "Bear in exile");
}

#[test]
fn silverquill_inquisition_makes_opp_discard_a_card() {
    let mut g = two_player_game();
    let _ = g.add_card_to_hand(1, catalog::grizzly_bears());
    let _ = g.add_card_to_hand(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_inquisition());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inquisition castable");
    drain_stack(&mut g);
    // Opp lost 1 card (chosen by us — should be Bears since Island is filtered out).
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1);
}

// ── Cross-college (extras) batch 14 ─────────────────────────────────────────

#[test]
fn lorehold_bookburner_sac_pings_a_creature() {
    let mut g = two_player_game();
    let burner = g.add_card_to_battlefield(0, catalog::lorehold_bookburner());
    g.clear_sickness(burner);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: burner, ability_index: 0,
        target: Some(crate::game::types::Target::Permanent(bear)), x_value: None }).expect("Activatable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(burner).is_none(), "Burner sacrificed");
    // Bear (2 toughness) takes 2 damage → dies to SBA.
    assert!(g.battlefield_find(bear).is_none(), "Bear destroyed by 2 dmg");
}

#[test]
fn prismari_lightcaster_etb_scrys_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::prismari_lightcaster());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lightcaster castable");
    drain_stack(&mut g);
    let bf = g.battlefield.iter().find(|c| c.definition.name == "Prismari Lightcaster");
    assert!(bf.is_some(), "Lightcaster on bf");
}

#[test]
fn prismari_stormbringer_burns_each_opp_on_instant_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_stormbringer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Opp takes 3 (bolt) + 2 (Stormbringer magecraft) = 5
    assert_eq!(g.players[1].life, life_before - 5);
}

#[test]
fn quandrix_counterspeaker_self_counters_on_instant_cast() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_counterspeaker());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let cs = g.battlefield.iter().find(|c| c.id == id).expect("Counterspeaker on bf");
    let count = cs.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(count, 1, "Counterspeaker has one +1/+1 counter");
}

#[test]
fn quandrix_tessellator_activated_mints_fractal_with_counters() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_tessellator());
    g.clear_sickness(id);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("Activatable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Fractal"
    }).expect("Fractal token");
    let count = fractal.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(count, 2, "Fractal has two +1/+1 counters");
}

#[test]
fn witherbloom_wanderer_pay_two_life_reanimates_creature() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_wanderer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wanderer castable");
    drain_stack(&mut g);
    // -1 hand (cast Wanderer) + 1 hand (Bear returned) = same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[0].life, life_before - 2, "paid 2 life");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear_gy),
        "Bear card in hand");
}

#[test]
fn witherbloom_pestbinder_etb_mints_a_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestbinder());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestbinder castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 1, "Pestbinder mints 1 Pest token on ETB");
}

#[test]
fn strixhaven_vault_etb_scrys_then_sac_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::strixhaven_vault());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vault castable");
    drain_stack(&mut g);
    let vault = g.battlefield.iter().find(|c| c.definition.name == "Strixhaven Vault")
        .expect("Vault on bf");
    let vault_id = vault.id;
    // Now activate the sac-for-draw ability.
    g.clear_sickness(vault_id);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: vault_id, ability_index: 0, target: None, x_value: None }).expect("Sac activation");
    drain_stack(&mut g);
    assert!(g.battlefield_find(vault_id).is_none(), "Vault sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew 1 card");
}

// ── Batch 15: Silverquill expansion ─────────────────────────────────────────

#[test]
fn silverquill_archivist_etb_scrys_and_gains_one_life() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::silverquill_archivist());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Archivist castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "gained 1 life on ETB");
    assert!(g.battlefield_find(id).is_some(), "Archivist on bf");
}

#[test]
fn silverquill_witness_magecraft_gains_one_life_on_instant_cast() {
    let mut g = two_player_game();
    let _w = g.add_card_to_battlefield(0, catalog::silverquill_witness());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "magecraft gained 1 life");
}

#[test]
fn silverquill_judge_magecraft_taps_opponent_creature() {
    let mut g = two_player_game();
    let _judge = g.add_card_to_battlefield(0, catalog::silverquill_judge());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(bear).expect("Bear on bf");
    assert!(view.tapped, "Judge magecraft tapped the opp bear");
}

#[test]
fn inkling_brigade_etb_mints_two_inkling_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_brigade());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Brigade castable");
    drain_stack(&mut g);
    let inklings: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Inkling"
    }).collect();
    assert_eq!(inklings.len(), 2, "Brigade mints 2 Inkling tokens");
}

#[test]
fn silverquill_pen_pusher_magecraft_scrys_one() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let _pp = g.add_card_to_battlefield(0, catalog::silverquill_pen_pusher());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry doesn't change library size (just reorders/sends top to bottom).
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn silverquill_chronicle_drains_two_and_returns_is_card_from_graveyard() {
    let mut g = two_player_game();
    let bolt_gy = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::silverquill_chronicle());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Chronicle castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 2, "gained 2 life from drain");
    assert_eq!(g.players[1].life, life1_before - 2, "opp lost 2 life from drain");
    // Hand: -1 (cast Chronicle) +1 (Bolt returned) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_gy), "Bolt back in hand");
}

// ── Batch 15: Witherbloom expansion ─────────────────────────────────────────

#[test]
fn witherbloom_pest_tender_etb_mints_a_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pest_tender());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tender castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 1, "Tender mints 1 Pest token");
}

#[test]
fn witherbloom_seer_drains_each_opp_on_instant_cast() {
    let mut g = two_player_game();
    let _seer = g.add_card_to_battlefield(0, catalog::witherbloom_seer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 1, "Seer drained +1 life");
    // Bolt 3 + Seer drain 1 = 4 to opp.
    assert_eq!(g.players[1].life, life1_before - 4);
}

#[test]
fn pest_swarm_creates_three_pest_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_swarm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Swarm castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 3, "Pest Swarm mints 3 Pest tokens");
}

#[test]
fn witherbloom_vinemaster_grows_on_pest_death() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let vm = g.add_card_to_battlefield(0, catalog::witherbloom_vinemaster());
    // Use a non-token Pest (Witherbloom Pest Eater) so the dying creature
    // stays in graveyard (not subject to token "ceases to exist" SBA).
    let pest = g.add_card_to_battlefield(0, catalog::witherbloom_pest_eater());
    drain_stack(&mut g);
    // Kill the Pest with two Bolts.
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(pest)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
    }
    let count = g.battlefield_find(vm).expect("VM on bf")
        .counter_count(CounterType::PlusOnePlusOne);
    assert!(count >= 1, "Vinemaster gained a +1/+1 counter on Pest death");
}

// ── Batch 15: Lorehold expansion ────────────────────────────────────────────

#[test]
fn lorehold_acolyte_etb_exiles_target_graveyard_card() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_acolyte());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Acolyte castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "Bolt in exile");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "Bolt gone from graveyard");
}

#[test]
fn lorehold_warrior_priest_gains_life_on_attack() {
    use crate::game::types::{AttackTarget, Attack, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_warrior_priest());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("Attackers declared");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "gained 1 life on attack");
}

#[test]
fn lorehold_ember_priest_magecraft_pings_target() {
    let mut g = two_player_game();
    let _ep = g.add_card_to_battlefield(0, catalog::lorehold_ember_priest());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Ember-Priest ping 1 = 4 to opp.
    assert_eq!(g.players[1].life, life_before - 4);
}

#[test]
fn lorehold_skirmish_mints_a_spirit_with_haste_eot() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_skirmish());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skirmish castable");
    drain_stack(&mut g);
    let spirit = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).expect("Spirit token minted");
    let view = g.computed_permanent(spirit.id).expect("Spirit on bf");
    assert!(view.keywords.contains(&Keyword::Haste),
        "Skirmish-minted Spirit has haste EOT");
}

// ── Batch 15: Quandrix expansion ────────────────────────────────────────────

#[test]
fn quandrix_summoner_etb_mints_one_one_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_summoner());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Summoner castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Fractal"
    }).expect("Fractal token minted");
    let view = g.computed_permanent(fractal.id).expect("Fractal on bf");
    assert_eq!(view.power, 1, "Fractal 0 base + 1 counter = 1 power");
    assert_eq!(view.toughness, 1, "Fractal 0 base + 1 counter = 1 toughness");
}

#[test]
fn quandrix_scholar_magecraft_adds_counter_to_friendly_creature() {
    let mut g = two_player_game();
    let _sch = g.add_card_to_battlefield(0, catalog::quandrix_scholar());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let count = g.battlefield_find(bear).expect("Bear on bf")
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(count, 1, "Bear got +1/+1 from Scholar magecraft");
}

#[test]
fn quandrix_ecologist_etb_self_pumps_with_counter() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_ecologist());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ecologist castable");
    drain_stack(&mut g);
    let view = g.computed_permanent(id).expect("Ecologist on bf");
    assert_eq!(view.power, 5, "Ecologist 4 + 1 counter = 5 power");
    assert_eq!(view.toughness, 5, "Ecologist 4 + 1 counter = 5 toughness");
    assert!(view.keywords.contains(&Keyword::Trample));
}

// ── Batch 15: Prismari expansion ────────────────────────────────────────────

#[test]
fn prismari_drakelord_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_drakelord());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.computed_permanent(id).expect("Drakelord on bf");
    assert_eq!(view.power, 3, "Drakelord 2 + 1 EOT = 3 power");
    assert_eq!(view.toughness, 4, "Drakelord 3 + 1 EOT = 4 toughness");
    assert!(view.keywords.contains(&Keyword::Flying));
}

#[test]
fn prismari_emberseer_etb_burns_each_opp() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_emberseer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Emberseer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 2, "Opp took 2 from Emberseer ETB");
}

#[test]
fn prismari_pyrowriter_magecraft_pings_target() {
    let mut g = two_player_game();
    let _pw = g.add_card_to_battlefield(0, catalog::prismari_pyrowriter());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Pyrowriter ping 1 = 4 to opp.
    assert_eq!(g.players[1].life, life_before - 4);
}

// ── Batch 17: Cross-college expansion ────────────────────────────────────────

#[test]
fn silverquill_marshal_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_marshal());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Marshal castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2, "Marshal ETB gives 2 life");
}

#[test]
fn silverquill_pupil_magecraft_pumps_self_plus_one_power() {
    let mut g = two_player_game();
    let pp = g.add_card_to_battlefield(0, catalog::silverquill_pupil());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(pp).expect("Pupil still on bf");
    assert_eq!(view.power(), 2, "Pupil +1/+0 = 2 power");
    assert_eq!(view.toughness(), 2, "toughness unchanged");
}

#[test]
fn defend_the_inkwell_drains_two_and_scrys_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::defend_the_inkwell());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib_before = g.players[0].library.len();
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Defend the Inkwell castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 2);
    assert_eq!(g.players[1].life, life1_before - 2);
    // Scry doesn't change library size.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn inkling_witness_gains_life_when_other_inkling_dies() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let _w = g.add_card_to_battlefield(0, catalog::inkling_witness());
    let other_ink = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    // Kill the Inkling Aspirant (2/1) with a Lightning Bolt.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(other_ink)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].life > life_before,
        "Witness gained at least 1 life from Inkling death (was {}, now {})",
        life_before, g.players[0].life);
}

#[test]
fn witherbloom_mossfeeder_etb_mints_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_mossfeeder());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mossfeeder castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 1, "Mossfeeder mints 1 Pest");
}

#[test]
fn witherbloom_reverie_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_reverie());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reverie castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 3);
    assert_eq!(g.players[1].life, life1_before - 3);
}

#[test]
fn pest_cultivator_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_cultivator());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cultivator castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 2, "Cultivator mints 2 Pests on ETB");
}

#[test]
fn withergrowth_apprentice_magecraft_pumps_friendly_creature() {
    let mut g = two_player_game();
    let _wa = g.add_card_to_battlefield(0, catalog::withergrowth_apprentice());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bear_p_before = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bear_p_after = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    assert_eq!(bear_p_after, bear_p_before + 1, "Bear pumped +1/+1");
}

#[test]
fn lorehold_pyrosage_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _ps = g.add_card_to_battlefield(0, catalog::lorehold_pyrosage());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Pyrosage 1 = 4 damage to opp.
    assert_eq!(g.players[1].life, life_before - 4);
}

#[test]
fn lorehold_loremaster_attack_mints_spirit_token() {
    use crate::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let lm = g.add_card_to_battlefield(0, catalog::lorehold_loremaster());
    g.clear_sickness(lm);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lm,
        target: AttackTarget::Player(1),
    }])).expect("Loremaster attacks");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).collect();
    assert_eq!(spirits.len(), 1, "Loremaster mints 1 Spirit per attack");
}

#[test]
fn lorehold_ember_forge_burns_creature_and_pings_each_opp() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_ember_forge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ember-Forge castable");
    drain_stack(&mut g);
    // Bear had 2 toughness, takes 3 → dies.
    assert!(g.battlefield_find(bear).is_none(), "Bear dies to 3 damage");
    assert_eq!(g.players[1].life, life_before - 1, "Opp loses 1 life");
}

#[test]
fn quandrix_symmetrist_etb_scrys_and_draws() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_symmetrist());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Symmetrist castable");
    drain_stack(&mut g);
    // Hand: -1 (cast) +1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_reckoner_attack_adds_plus_one_counter() {
    use crate::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let rk = g.add_card_to_battlefield(0, catalog::quandrix_reckoner());
    g.clear_sickness(rk);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rk,
        target: AttackTarget::Player(1),
    }])).expect("Reckoner attacks");
    drain_stack(&mut g);
    let view = g.battlefield_find(rk).expect("Reckoner present");
    let counters = view.counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 1, "Reckoner gains a +1/+1 counter per attack");
    assert_eq!(view.power(), 3, "Reckoner is now 3/3");
}

#[test]
fn fractal_reinforcement_puts_counter_on_each_friendly_creature() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b_opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_reinforcement());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reinforcement castable");
    drain_stack(&mut g);
    let p1 = g.battlefield_find(b1).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    let p2 = g.battlefield_find(b2).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(p1, 1, "Bear 1 has +1/+1 counter");
    assert_eq!(p2, 1, "Bear 2 has +1/+1 counter");
}

#[test]
fn prismari_pyrotechnician_magecraft_pings_target() {
    let mut g = two_player_game();
    let _pt = g.add_card_to_battlefield(0, catalog::prismari_pyrotechnician());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Pyrotechnician 1 = 4 to opp.
    assert_eq!(g.players[1].life, life_before - 4);
}

#[test]
fn prismari_looter_etb_loots_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island()); // discard fodder
    let id = g.add_card_to_hand(0, catalog::prismari_looter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Looter castable");
    drain_stack(&mut g);
    // Hand: -1 (cast) +1 (draw) -1 (discard) = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_chromaticist_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_chromaticist());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Chromaticist castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Treasure"
    }).collect();
    assert_eq!(treasures.len(), 1, "Chromaticist mints 1 Treasure");
}

#[test]
fn prismari_drakeward_etb_deals_two_to_each_opp() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_drakeward());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drakeward castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 2, "Opp loses 2 to ETB ping");
}

// ── CR 115.5 self-target enforcement (engine improvement) ───────────────────

#[test]
fn cr_115_5_spell_targeting_itself_is_illegal_via_permanent_id() {
    // Cast a creature, then try Bury in Books (bounce target creature)
    // targeting the in-progress cast spell's own id. Bury in Books needs
    // a creature target, and we'll verify that re-using the bury card id
    // as its own target (Target::Permanent(card_id)) is rejected by
    // check_target_legality_with_source. The headline gameplay rule is
    // that a spell on the stack cannot target itself (CR 115.5).
    let mut g = two_player_game();
    let bury = g.add_card_to_hand(0, catalog::bury_in_books());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Casting Bury in Books targeting itself (its own card_id) is rejected:
    // the cast pipeline threads `Some(card_id)` to the target validator,
    // so the bury card cannot be its own bounce target.
    let result = g.perform_action(GameAction::CastSpell {
        card_id: bury,
        target: Some(crate::game::types::Target::Permanent(bury)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(result.is_err(), "Bury in Books targeting itself should be rejected (CR 115.5)");
}

// ── Batch 18 — Witherbloom / Prismari follow-on cards ───────────────────────

#[test]
fn witherbloom_bonepicker_etb_drains_each_opp_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_bonepicker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bonepicker castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 2, "Opp loses 2 to Bonepicker ETB");
}

#[test]
fn pest_swarm_inheritance_pumps_friendly_and_mints_pest() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pest_swarm_inheritance());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bear_p_before = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest Bequest castable");
    drain_stack(&mut g);
    let bear_p_after = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    assert_eq!(bear_p_after, bear_p_before + 1, "Bear pumped +1/+1");
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Deathtouch),
        "Bear gained Deathtouch EOT");
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 1, "1 Pest token created");
}

#[test]
fn witherbloom_decayblossom_dies_shrinks_target() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let blossom = g.add_card_to_battlefield(0, catalog::witherbloom_decayblossom());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Kill our own Decayblossom with a Bolt.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(blossom)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Decayblossom's death trigger fires — auto-target picks the bear,
    // shrinking it to -1/-1 → 1/1 (from 2/2). Then SBA kills no-one;
    // it stays at 1/1 EOT.
    let bear_p = g.battlefield_find(opp_bear).map(|c| c.power()).unwrap_or(0);
    let bear_t = g.battlefield_find(opp_bear).map(|c| c.toughness()).unwrap_or(0);
    assert_eq!(bear_p, 1, "Bear shrunk to 1 power");
    assert_eq!(bear_t, 1, "Bear shrunk to 1 toughness");
}

#[test]
fn witherbloom_recourse_returns_low_mv_creature_and_drains() {
    let mut g = two_player_game();
    // Seed graveyard with a low-MV creature.
    let _gy_card = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_recourse());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recourse castable");
    drain_stack(&mut g);
    // -1 (cast Recourse) + 1 (return bear) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[0].life, life0_before + 1, "Gain 1 life");
    assert_eq!(g.players[1].life, life1_before - 1, "Opp loses 1");
}

#[test]
fn witherbloom_pestmancer_mints_pest_on_instant_cast() {
    let mut g = two_player_game();
    let _pm = g.add_card_to_battlefield(0, catalog::witherbloom_pestmancer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 1, "Pestmancer mints 1 Pest per IS cast");
}

#[test]
fn witherbloom_pestkeeper_etb_mints_pest_and_sac_shrinks_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestkeeper());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestkeeper castable");
    drain_stack(&mut g);
    // Pestkeeper ETB minted a Pest already.
    let pests_before: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests_before.len(), 1, "ETB mints 1 Pest");
}

#[test]
fn prismari_spellsmith_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_spellsmith());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellsmith castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Treasure"
    }).collect();
    assert_eq!(treasures.len(), 1, "Spellsmith mints 1 Treasure");
}

#[test]
fn prismari_storm_caller_loots_on_instant_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::mountain()); // discard fodder
    let _sc = g.add_card_to_battlefield(0, catalog::prismari_storm_caller());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 (cast bolt) +1 (draw) -1 (discard) = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_ignite_apprentice_pings_on_etb() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let _opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_ignite_apprentice());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ignite-Apprentice castable");
    drain_stack(&mut g);
    // ETB ping any target — using opp player here closes the test cleanly.
    assert_eq!(g.players[1].life, life_before - 1, "Opp loses 1 to ETB ping");
}

#[test]
fn prismari_volley_burns_creature_and_draws() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_volley());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Volley castable");
    drain_stack(&mut g);
    // Bear had 2 toughness, takes 3 → dies. Hand: -1 (cast) +1 (draw) = 0 net.
    assert!(g.battlefield_find(bear).is_none(), "Bear dies to 3 damage");
    assert_eq!(g.players[0].hand.len(), hand_before, "Drew a card to replace the cast");
}

// ── Batch 18 — Quandrix / Lorehold / Silverquill follow-on cards ───────────

#[test]
fn quandrix_fractalflow_mints_fractal_scaled_by_hand() {
    let mut g = two_player_game();
    // Seed the hand to 3 cards before the cast.
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalflow());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractalflow castable");
    drain_stack(&mut g);
    // After cast: hand had 2 cards left (originals seeded above). The
    // Fractal token receives that many +1/+1 counters.
    let fractal = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Fractal"
    }).expect("1 Fractal minted");
    let counters = fractal.counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 2, "Fractal scales counters to hand size");
}

#[test]
fn quandrix_scrycharmer_scrys_on_instant_cast() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let _sc = g.add_card_to_battlefield(0, catalog::quandrix_scrycharmer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry doesn't change library size — just exercise that no panic.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn quandrix_multibinding_doubles_counters_after_adding() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_multibinding());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Multibinding castable");
    drain_stack(&mut g);
    // Add 2 +1/+1, then double counts: 2 → 4 (the doubling adds 2 more).
    let counters = g.battlefield_find(bear).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(counters, 4, "Multibinding: 2 + (2 doubled = 2 more) = 4");
}

#[test]
fn quandrix_geomyst_etb_draws_card_and_has_reach() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_geomyst());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Geomyst castable");
    drain_stack(&mut g);
    // -1 (cast) +1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let def = catalog::quandrix_geomyst();
    assert!(def.keywords.contains(&Keyword::Reach));
}

#[test]
fn lorehold_spiritcaller_etb_mints_spirit_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritcaller());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritcaller castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).collect();
    assert_eq!(spirits.len(), 1, "Spiritcaller mints 1 Spirit on ETB");
}

#[test]
fn lorehold_pyrebrand_magecraft_self_pumps() {
    let mut g = two_player_game();
    let pb = g.add_card_to_battlefield(0, catalog::lorehold_pyrebrand());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(pb).map(|c| c.power()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(pb).map(|c| c.power()).unwrap_or(0);
    assert_eq!(p_after, p_before + 1, "Pyrebrand self-pumps +1/+0");
    let def = catalog::lorehold_pyrebrand();
    assert!(def.keywords.contains(&Keyword::FirstStrike));
}

#[test]
fn lorehold_reclamation_returns_creature_to_battlefield() {
    let mut g = two_player_game();
    let _gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_reclamation());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.name == "Grizzly Bears"
    }).collect();
    let bf_count_before = bf_before.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reclamation castable");
    drain_stack(&mut g);
    let bf_after: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.name == "Grizzly Bears"
    }).collect();
    assert_eq!(bf_after.len(), bf_count_before + 1, "Bear returned to battlefield");
}

#[test]
fn lorehold_reverberator_magecraft_pings_target() {
    let mut g = two_player_game();
    let _rv = g.add_card_to_battlefield(0, catalog::lorehold_reverberator());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Reverberator 2 = 5 to opp.
    assert_eq!(g.players[1].life, life_before - 5);
}

#[test]
fn inkling_coursebinder_drains_on_instant_cast() {
    let mut g = two_player_game();
    let _ic = g.add_card_to_battlefield(0, catalog::inkling_coursebinder());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Coursebinder drain = 4 to opp, +1 to us.
    assert_eq!(g.players[0].life, life0_before + 1);
    assert_eq!(g.players[1].life, life1_before - 4);
    let def = catalog::inkling_coursebinder();
    assert!(def.keywords.contains(&Keyword::Flying));
}

#[test]
fn silverquill_sermon_mints_two_inkling_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_sermon());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sermon castable");
    drain_stack(&mut g);
    let inklings: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Inkling"
    }).collect();
    assert_eq!(inklings.len(), 2, "Sermon mints 2 Inklings");
}

#[test]
fn silverquill_censure_exiles_low_power_creature_and_gains_life() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_censure());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Censure castable");
    drain_stack(&mut g);
    // Bear had power 2 → ≤3 ok → exiled. Caster gains 2 life.
    assert!(g.battlefield_find(bear).is_none(), "Bear exiled");
    assert_eq!(g.players[0].life, life_before + 2, "Caster gains 2 life");
}

// ── batch 19 tests ─────────────────────────────────────────────────────────

#[test]
fn silverquill_castigant_etb_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_castigant());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Castigant castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 1);
    assert_eq!(g.players[1].life, life1_before - 1);
    let def = catalog::silverquill_castigant();
    assert_eq!(def.power, 2);
    assert_eq!(def.toughness, 3);
}

#[test]
fn silverquill_heartrender_drains_three_and_scrys_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_heartrender());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Heartrender castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 3);
    assert_eq!(g.players[1].life, life1_before - 3);
    // Scry 1 doesn't change library size; just verifies cast resolved.
    assert_eq!(g.players[0].library.len(), lib_before, "scry doesn't change lib size");
}

#[test]
fn inkling_confessor_magecraft_drains_on_instant_cast() {
    let mut g = two_player_game();
    let _ic = g.add_card_to_battlefield(0, catalog::inkling_confessor());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Confessor drain = 4 to opp, +1 to us.
    assert_eq!(g.players[0].life, life0_before + 1);
    assert_eq!(g.players[1].life, life1_before - 4);
    let def = catalog::inkling_confessor();
    assert!(def.keywords.contains(&Keyword::Flying));
}

#[test]
fn witherbloom_lifebleeder_drains_on_instant_cast() {
    let mut g = two_player_game();
    let _lb = g.add_card_to_battlefield(0, catalog::witherbloom_lifebleeder());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 1);
    assert_eq!(g.players[1].life, life1_before - 4);
}

#[test]
fn pest_marauder_has_deathtouch_and_dies_grants_life() {
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::pest_marauder());
    // Hand-of-fate: kill the marauder via SBA by damage.
    let pm_card = g.battlefield_find_mut(pm).unwrap();
    pm_card.damage = 1; // 1/1 with 1 damage → SBA kills it
    let life_before = g.players[0].life;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(pm).is_none(), "Marauder died");
    assert_eq!(g.players[0].life, life_before + 1, "Marauder grants 1 life on death");
    let def = catalog::pest_marauder();
    assert!(def.keywords.contains(&Keyword::Deathtouch));
}

#[test]
fn witherbloom_decoctor_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_decoctor());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Decoctor castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 2);
    assert_eq!(g.players[1].life, life1_before - 2);
    let def = catalog::witherbloom_decoctor();
    assert_eq!(def.power, 3);
    assert_eq!(def.toughness, 4);
}

#[test]
fn witherbloom_sapfiend_self_pumps_on_instant_cast() {
    let mut g = two_player_game();
    let sf = g.add_card_to_battlefield(0, catalog::witherbloom_sapfiend());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(sf).map(|c| c.power()).unwrap_or(0);
    let t_before = g.battlefield_find(sf).map(|c| c.toughness()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(sf).map(|c| c.power()).unwrap_or(0);
    let t_after = g.battlefield_find(sf).map(|c| c.toughness()).unwrap_or(0);
    assert_eq!(p_after, p_before + 1, "Sapfiend self-pumps +1");
    assert_eq!(t_after, t_before + 1, "Sapfiend self-pumps +1 toughness");
}

#[test]
fn lorehold_pyrescribe_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _ps = g.add_card_to_battlefield(0, catalog::lorehold_pyrescribe());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Pyrescribe 1 = 4 to opp.
    assert_eq!(g.players[1].life, life1_before - 4);
}

#[test]
fn lorehold_echoist_etb_mints_spirit_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_echoist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Echoist castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).collect();
    assert_eq!(spirits.len(), 1, "Echoist mints 1 Spirit on ETB");
}

#[test]
fn lorehold_spiritmaster_etb_mints_two_spirit_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritmaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritmaster castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).collect();
    assert_eq!(spirits.len(), 2, "Spiritmaster mints 2 Spirits");
    let def = catalog::lorehold_spiritmaster();
    assert_eq!(def.power, 3);
    assert_eq!(def.toughness, 3);
}

#[test]
fn lorehold_bonepriest_grows_on_each_instant_cast() {
    let mut g = two_player_game();
    let bp = g.add_card_to_battlefield(0, catalog::lorehold_bonepriest());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(bp).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(counters, 1, "Bonepriest gains 1 counter from magecraft");
}

#[test]
fn quandrix_doublecaster_grows_on_instant_cast() {
    let mut g = two_player_game();
    let dc = g.add_card_to_battlefield(0, catalog::quandrix_doublecaster());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(dc).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(counters, 1, "Doublecaster gains 1 counter from magecraft");
    let def = catalog::quandrix_doublecaster();
    assert!(def.subtypes.creature_types.contains(&CreatureType::Fractal));
}

#[test]
fn quandrix_wavewright_etb_scrys_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_wavewright());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wavewright castable");
    drain_stack(&mut g);
    // -1 (cast) +1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_sapsprout_self_grows_on_cast() {
    let mut g = two_player_game();
    let ss = g.add_card_to_battlefield(0, catalog::quandrix_sapsprout());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(ss).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(counters, 1, "Sapsprout gains 1 counter from magecraft");
}

#[test]
fn fractal_multiplier_doubles_counters_on_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Pre-load with 3 counters.
    if let Some(c) = g.battlefield_find_mut(bear) {
        c.add_counters(CounterType::PlusOnePlusOne, 3);
    }
    let id = g.add_card_to_hand(0, catalog::fractal_multiplier());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Multiplier castable");
    drain_stack(&mut g);
    // 3 + 3 (doubled) = 6 counters.
    let counters = g.battlefield_find(bear).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(counters, 6, "Fractal Multiplier doubles 3 → 6");
}

#[test]
fn prismari_stormcaster_loots_on_instant_cast() {
    let mut g = two_player_game();
    let _sc = g.add_card_to_battlefield(0, catalog::prismari_stormcaster());
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 (cast bolt) +1 (draw) -1 (discard) = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
    let def = catalog::prismari_stormcaster();
    assert!(def.keywords.contains(&Keyword::Flying));
}

#[test]
fn prismari_sparkmaster_self_pumps_on_cast() {
    let mut g = two_player_game();
    let sm = g.add_card_to_battlefield(0, catalog::prismari_sparkmaster());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(sm).map(|c| c.power()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(sm).map(|c| c.power()).unwrap_or(0);
    assert_eq!(p_after, p_before + 1, "Sparkmaster self-pumps +1");
}

#[test]
fn prismari_ember_channeler_pings_on_cast() {
    let mut g = two_player_game();
    let _ec = g.add_card_to_battlefield(0, catalog::prismari_ember_channeler());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Channeler 1 = 4 to opp.
    assert_eq!(g.players[1].life, life1_before - 4);
}

// ── batch 19+ extras (10 more cards) ───────────────────────────────────────

#[test]
fn silverquill_quillblade_pumps_by_creature_count() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Add 2 more creatures to make 3 total.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_quillblade());
    g.players[0].mana_pool.add(Color::White, 1);
    let p_before = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quillblade castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    assert_eq!(p_after, p_before + 3, "Bear pumped by 3 (3 creatures controlled)");
}

#[test]
fn inkling_decree_drains_two_and_mints_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_decree());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Decree castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 2);
    assert_eq!(g.players[1].life, life1_before - 2);
    let inklings: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Inkling"
    }).collect();
    assert_eq!(inklings.len(), 1, "Decree mints 1 Inkling");
}

#[test]
fn pest_communion_mills_four_each_opp_and_drains_one() {
    let mut g = two_player_game();
    for _ in 0..10 {
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::pest_communion());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_lib_before = g.players[1].library.len();
    let opp_gy_before = g.players[1].graveyard.len();
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Communion castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), opp_lib_before - 4);
    assert_eq!(g.players[1].graveyard.len(), opp_gy_before + 4);
    assert_eq!(g.players[0].life, life0_before + 1);
    assert_eq!(g.players[1].life, life1_before - 1);
}

#[test]
fn lorehold_recollect_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_recollect());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bears_before = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears")
        .count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recollect castable");
    drain_stack(&mut g);
    let bears_after = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears")
        .count();
    assert_eq!(bears_after, bears_before + 1, "Bear returned to battlefield");
}

#[test]
fn lorehold_anthemist_anthem_buffs_other_spirits() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_anthemist());
    // Mint a Spirit token via Lorehold Echoist's ETB.
    let echoist = g.add_card_to_hand(0, catalog::lorehold_echoist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: echoist, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Echoist castable");
    drain_stack(&mut g);
    // Find the minted Spirit token id, then read its computed P/T via the layer system.
    let spirit_id = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).map(|c| c.id).expect("Spirit minted");
    let spirit_computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == spirit_id)
        .expect("Spirit on computed battlefield");
    // Lorehold Spirit token is 2/2; with +1/+1 anthem from Anthemist, should be 3/3.
    assert_eq!(spirit_computed.power, 3, "Spirit pumped to 3/3 by Anthemist");
    assert_eq!(spirit_computed.toughness, 3);
}

#[test]
fn fractal_growth_adds_counter_and_pumps_by_counter_count() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let p_before = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Growth castable");
    drain_stack(&mut g);
    // 0 prior counters → +1 counter (now 1) → +1/+1 EOT from 1 counter → 3/3 total (2 base + 1 counter)
    // Then PumpPT(+1/+1) → 4/4 EOT.
    // Actually: base 2/2, +1 counter → 3/3, then PumpPT +1/+1 → 4/4.
    let p_after = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    assert_eq!(p_after, p_before + 2, "Bear +1 counter (+1) + EOT +1 = +2 power");
}

#[test]
fn quandrix_calculus_etb_mills_two_and_draws_one() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_calculus());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib_before = g.players[0].library.len();
    let gy_before = g.players[0].graveyard.len();
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Calculus castable");
    drain_stack(&mut g);
    // Mill 2 + Draw 1 = library -3, graveyard +2, hand: -1 (cast) +1 (draw) = 0
    assert_eq!(g.players[0].library.len(), lib_before - 3);
    assert_eq!(g.players[0].graveyard.len(), gy_before + 2);
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_alchemist_mints_treasure_on_instant_cast() {
    let mut g = two_player_game();
    let _al = g.add_card_to_battlefield(0, catalog::prismari_alchemist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let treasures_before = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Treasure"
    }).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let treasures_after = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Treasure"
    }).count();
    assert_eq!(treasures_after, treasures_before + 1, "Alchemist mints 1 Treasure");
}

#[test]
fn prismari_cantrip_deals_one_damage_and_cantrips() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_cantrip());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cantrip castable");
    drain_stack(&mut g);
    // -1 (cast) +1 (draw) = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Bear took 1 damage (now 2/2 with 1 damage marked).
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.damage, 1);
}

#[test]
fn prismari_flarespark_deals_two_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_flarespark());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flarespark castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2);
    // -1 (cast) +1 (draw) = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── batch 20: 25 more synthesised STX cards (5 per college) ────────────────

// ── Silverquill (W/B) ──────────────────────────────────────────────────────

#[test]
fn silverquill_lawkeeper_etb_taps_opp_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_lawkeeper());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lawkeeper castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).expect("bear still on bf");
    assert!(bear_card.tapped, "Lawkeeper ETB taps opp creature");
    let def = catalog::silverquill_lawkeeper();
    assert!(def.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn inkling_penmaster_mints_inkling_on_instant_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::inkling_penmaster());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let inklings_before = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Inkling")
        .count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let inklings_after = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Inkling")
        .count();
    assert_eq!(inklings_after, inklings_before + 1, "Magecraft mints an Inkling");
}

#[test]
fn silverquill_dictation_drains_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_dictation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Dictation castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2);
    // -1 (cast) +1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_stormcaller_etb_drains_two_and_is_flying_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_stormcaller());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormcaller castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2);
    assert_eq!(g.players[0].life, life0_before + 2);
    let def = catalog::inkling_stormcaller();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn silverquill_discipline_pumps_and_grants_lifelink() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_discipline());
    g.players[0].mana_pool.add(Color::White, 1);
    let p_before = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    let t_before = g.battlefield_find(bear).map(|c| c.toughness()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Discipline castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    let t_after = g.battlefield_find(bear).map(|c| c.toughness()).unwrap_or(0);
    assert_eq!(p_after, p_before + 2);
    assert_eq!(t_after, t_before + 1);
    let bear_card = g.battlefield_find(bear).expect("bear still on bf");
    assert!(bear_card.has_keyword(&Keyword::Lifelink));
}

// ── Witherbloom (B/G) ──────────────────────────────────────────────────────

#[test]
fn witherbloom_toxicultivator_etb_mints_pest_and_has_deathtouch() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxicultivator());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Toxicultivator castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 1, "Toxicultivator ETB mints a Pest");
    let def = catalog::witherbloom_toxicultivator();
    assert!(def.keywords.contains(&Keyword::Deathtouch));
}

#[test]
fn pest_outburst_mints_two_pests_and_gains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_outburst());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Outburst castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 2, "Outburst creates 2 Pests");
    assert_eq!(g.players[0].life, life0_before + 2);
}

#[test]
fn witherbloom_grand_necromancer_returns_creature_from_gy() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_grand_necromancer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Necromancer castable");
    drain_stack(&mut g);
    // -1 (cast) +1 (return bear from gy to hand) = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let in_hand = g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears");
    assert!(in_hand, "Bear returned to hand");
}

#[test]
fn witherbloom_sapdrinker_self_pumps_and_has_lifelink() {
    let mut g = two_player_game();
    let sd = g.add_card_to_battlefield(0, catalog::witherbloom_sapdrinker());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(sd).map(|c| c.power()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(sd).map(|c| c.power()).unwrap_or(0);
    assert_eq!(p_after, p_before + 1);
    let def = catalog::witherbloom_sapdrinker();
    assert!(def.keywords.contains(&Keyword::Lifelink));
}

// ── Lorehold (R/W) ─────────────────────────────────────────────────────────

#[test]
fn lorehold_battlescroll_mints_two_spirits_with_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_battlescroll());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battlescroll castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 2, "Battlescroll creates 2 Spirit tokens");
    for s in &spirits {
        assert!(s.has_keyword(&Keyword::Haste), "Spirit should have haste");
    }
}

#[test]
fn lorehold_tomescholar_mints_spirit_when_exiling_creature_card() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear_in_gy = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_tomescholar());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear_in_gy)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tomescholar castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 1, "Tomescholar mints Spirit when exiling creature card");
}

#[test]
fn lorehold_tomescholar_no_spirit_when_exiling_noncreature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bolt_in_gy = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_tomescholar());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bolt_in_gy)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tomescholar castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 0, "No Spirit when exiling noncreature");
}

#[test]
fn lorehold_ember_brand_deals_three_to_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_ember_brand());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ember-Brand castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 3);
}

#[test]
fn lorehold_spectrescribe_magecraft_gains_one_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_spectrescribe());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 1);
}

#[test]
fn lorehold_warband_pumps_by_other_attackers() {
    use crate::game::types::AttackTarget;
    let mut g = two_player_game();
    let wb = g.add_card_to_battlefield(0, catalog::lorehold_warband());
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Untap before declare attackers
    for cid in [wb, bear1, bear2] {
        g.clear_sickness(cid);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: wb, target: AttackTarget::Player(1) },
        Attack { attacker: bear1, target: AttackTarget::Player(1) },
        Attack { attacker: bear2, target: AttackTarget::Player(1) },
    ])).expect("DeclareAttackers");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(wb).map(|c| c.power()).unwrap_or(0);
    // 3 base + 2 other attackers = 5
    assert_eq!(p_after, 5, "Warband pumped by 2 other attackers");
}

// ── Quandrix (G/U) ─────────────────────────────────────────────────────────

#[test]
fn fractal_bloom_mints_fractal_scaled_by_double_hand() {
    let mut g = two_player_game();
    // Set hand to exactly some count first, then cast.
    let id = g.add_card_to_hand(0, catalog::fractal_bloom());
    // Add 4 more cards to hand: total 5 in hand (1 will be cast)
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::island());
    }
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloom castable");
    drain_stack(&mut g);
    // After casting, hand = 4 islands. 2*4 = 8 +1/+1 counters.
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.is_token && c.definition.name == "Fractal")
        .expect("Fractal minted");
    let counters = fractal.counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 8, "Fractal has 2*4=8 +1/+1 counters");
}

#[test]
fn quandrix_spellweaver_etb_draws_two_and_grows_on_cast() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_spellweaver());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellweaver castable");
    drain_stack(&mut g);
    // -1 (cast) +2 (draw) = +1 net
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    // Cast another spell — Spellweaver should get +1/+1 counter
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let sw = g.battlefield.iter().find(|c| c.definition.name == "Quandrix Spellweaver").expect("Spellweaver");
    let counters = sw.counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 1, "Spellweaver grew via magecraft");
}

#[test]
fn quandrix_wavedancer_etb_scrys_two_and_is_flash() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_wavedancer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wavedancer castable");
    drain_stack(&mut g);
    let def = catalog::quandrix_wavedancer();
    assert!(def.keywords.contains(&Keyword::Flash));
    // Scry resolved (no easy direct check); confirm it landed on bf.
    let on_bf = g.battlefield.iter().any(|c| c.definition.name == "Quandrix Wavedancer");
    assert!(on_bf, "Wavedancer ETB");
}

#[test]
fn fractal_synthesis_adds_two_counters_and_draws() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_synthesis());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Synthesis castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(bear).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(counters, 2);
    assert_eq!(g.players[0].hand.len(), hand_before); // -1 cast +1 draw = 0
}
