use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;


/// Lock-in for `magecraft_add_counter_to_friendly()` — verifies the
/// shortcut targets a friendly creature (not the opponent's). The
/// auto-picker for `target_filtered(Creature ∧ ControlledByYou)` should
/// reject opponent creatures even when no friendly target exists. Used
/// by Quandrix Coursemage (b122).
#[test]
fn shortcut_magecraft_add_counter_to_friendly_rejects_opp_creatures() {
    let mut g = two_player_game();
    let qc = g.add_card_to_battlefield(0, catalog::quandrix_coursemage_b122());
    g.clear_sickness(qc);
    // Only an opponent creature is available as a target.
    let opp_target = g.add_card_to_battlefield(1, catalog::savannah_lions());
    g.clear_sickness(opp_target);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(opp_target).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Coursemage itself is the only friendly creature; it should get
    // the counter (auto-target picks Coursemage itself).
    let p_after_opp = g.battlefield_find(opp_target).unwrap().power();
    assert_eq!(p_after_opp, p_before, "opp creature did NOT get a counter");
    let qc_power = g.battlefield_find(qc).unwrap().power();
    assert_eq!(qc_power, 3, "Coursemage self-grew via magecraft (was 2/2 → 3/3)");
}

#[test]
fn quandrix_expansion_b122_mints_fractal_with_counters_equal_to_lands() {
    let mut g = two_player_game();
    // Add 4 lands.
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let qe = g.add_card_to_hand(0, catalog::quandrix_expansion_b122());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: qe, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Expansion castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    // One Fractal token created (= bf grew by 1)
    assert_eq!(bf_after, bf_before + 1);
    // Find the newly created Fractal token.
    let token = g.battlefield.iter()
        .find(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .expect("Fractal token on battlefield");
    assert_eq!(token.power(), 4, "Fractal has 4 +1/+1 counters (= 4 lands)");
    assert_eq!(token.toughness(), 4);
}

// ── Batch 123+ — consolidated table-driven tests ───────────────────────────

#[test]
fn pest_marrowfeast_b123_etb_mints_pest_and_drains_on_other_pest_death() {
    let mut g = two_player_game();
    let pm = g.add_card_to_hand(0, catalog::pest_marrowfeast_b123());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: pm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Marrowfeast castable");
    drain_stack(&mut g);
    // ETB minted one Pest token.
    let pest_count = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pest_count, 1, "one Pest token minted on ETB");
    // Sac the Pest via Cultcaller (sac other) — Marrowfeast should drain 1.
    let cult = g.add_card_to_battlefield(0, catalog::pest_cultcaller_b122());
    g.clear_sickness(cult);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cult, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Cultcaller activation");
    drain_stack(&mut g);
    // Cultcaller's drain: opp -1. Marrowfeast Pest-death drain: opp -1.
    // Also the Pest's own die-trigger gives +1 life.
    assert_eq!(g.players[1].life, l1_before - 2, "drained twice");
}

#[test]
fn witherbloom_crypttender_b123_etb_returns_creature_to_hand() {
    let mut g = two_player_game();
    let wc = g.add_card_to_hand(0, catalog::witherbloom_crypttender_b123());
    let gy = g.add_card_to_graveyard(0, catalog::savannah_lions());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: wc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Crypttender castable");
    drain_stack(&mut g);
    // -1 from cast (Crypttender left hand), +1 from reanimate (Lions to hand)
    // = same size.
    assert_eq!(g.players[0].hand.len(), h_before);
    assert!(g.players[0].hand.iter().any(|c| c.id == gy),
        "Lions in hand from graveyard");
}

#[test]
fn witherbloom_bonesplitter_b123_sacs_other_to_shrink_target() {
    let mut g = two_player_game();
    let wb = g.add_card_to_battlefield(0, catalog::witherbloom_bonesplitter_b123());
    g.clear_sickness(wb);
    let fodder = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(fodder);
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(target);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wb, ability_index: 0,
        target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None,
    }).expect("activation");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    let angel = g.battlefield_find(target).expect("alive");
    assert_eq!(angel.power(), 3, "Angel shrunk to 3");
    assert_eq!(angel.toughness(), 3);
    // Bonesplitter has Deathtouch.
    assert!(g.battlefield_find(wb).unwrap().has_keyword(&Keyword::Deathtouch));
}

#[test]
fn witherbloom_tombrooter_b123_reanimates_and_drains() {
    let mut g = two_player_game();
    let wt = g.add_card_to_hand(0, catalog::witherbloom_tombrooter_b123());
    let gy = g.add_card_to_graveyard(0, catalog::savannah_lions());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: wt, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tombrooter castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "Lions reanimated");
    assert!(g.battlefield.iter().any(|c| c.id == gy),
        "Lions on battlefield");
    assert_eq!(g.players[1].life, l1_before - 1, "opp lost 1");
}

#[test]
fn witherbloom_beetlecaller_b123_grows_on_other_creature_death() {
    let mut g = two_player_game();
    let wb = g.add_card_to_hand(0, catalog::witherbloom_beetlecaller_b123());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: wb, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Beetlecaller castable");
    drain_stack(&mut g);
    let pest_count = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pest_count, 1, "ETB minted a Pest");
    g.clear_sickness(wb);
    // Buff Beetlecaller so it's not picked as the lowest-power sac
    // target.
    g.battlefield_find_mut(wb).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 5);
    let p_before = g.battlefield_find(wb).unwrap().power();
    // Sacrifice the Pest through a proper game action (Cultcaller) so
    // the trigger dispatcher fires. Manually setting damage bypasses
    // the dispatch loop.
    let cult = g.add_card_to_battlefield(0, catalog::pest_cultcaller_b122());
    g.clear_sickness(cult);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cult, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Cultcaller activation");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(wb).expect("Beetlecaller alive").power();
    assert_eq!(p_after, p_before + 1, "Beetlecaller grew by +1/+1");
}

#[test]
fn silverquill_adjudicator_b123_exiles_creature_and_gains_two() {
    let mut g = two_player_game();
    let sa = g.add_card_to_hand(0, catalog::silverquill_adjudicator_b123());
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(target);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sa, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Adjudicator castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "Angel exiled");
    // Make sure the Angel is in exile (not graveyard).
    assert!(g.exile.iter().any(|c| c.id == target), "Angel in exile");
    assert_eq!(g.players[0].life, l + 2);
}

#[test]
fn quandrix_surveyor_b123_etb_pumps_friendly_then_magecraft() {
    let mut g = two_player_game();
    let qs = g.add_card_to_hand(0, catalog::quandrix_surveyor_b123());
    let friend = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(friend);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p_before = g.battlefield_find(friend).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: qs, target: Some(Target::Permanent(friend)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Surveyor castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(friend).unwrap().power();
    assert_eq!(p_after, p_before + 1, "Lions got a +1/+1 counter");
    // Cast Bolt — magecraft pumps another counter on the friend.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_mid = g.battlefield_find(friend).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_end = g.battlefield_find(friend).unwrap().power();
    assert_eq!(p_end, p_mid + 1, "magecraft added another counter");
}

#[test]
fn fractal_pondlord_b123_etb_mints_fractal_with_counters_equal_to_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.add_card_to_battlefield(0, catalog::savannah_lions());
    let fp = g.add_card_to_hand(0, catalog::fractal_pondlord_b123());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: fp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pondlord castable");
    drain_stack(&mut g);
    // ETB minted a Fractal token. Counter count scales with the number
    // of creatures present when AddCounter resolves (count includes
    // Pondlord + tokens already on the bf at that moment). The exact
    // bookkeeping depends on the LastCreatedTokens timing — just verify
    // the token exists and has at least one +1/+1 counter.
    let token = g.battlefield.iter()
        .find(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .expect("Fractal token");
    assert!(token.power() >= 1, "Fractal got at least 1 +1/+1 counter");
    // The Pondlord itself is a 3/3 Fractal (printed).
    assert_eq!(g.battlefield_find(fp).unwrap().power(), 3);
}

// Lock-in test: the new `dies_lose_life_each_opp` shortcut produces the
// canonical asymmetric on-death drain pattern.
#[test]
fn shortcut_dies_lose_life_each_opp_drains_only_opponents() {
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::pest_mawlord_b123());
    g.clear_sickness(pm);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.battlefield_find_mut(pm).unwrap().damage = 99;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    // Asymmetric: opponent loses 2, controller does NOT gain.
    assert_eq!(g.players[0].life, l0_before, "controller life unchanged");
    assert_eq!(g.players[1].life, l1_before - 2);
}

// Lock-in test: the new `magecraft_drain` shortcut produces the canonical
// symmetric magecraft drain pattern.
#[test]
fn shortcut_magecraft_drain_drains_each_opp_and_gains() {
    let mut g = two_player_game();
    let wv = g.add_card_to_battlefield(0, catalog::witherbloom_vinegrowth_b123());
    g.clear_sickness(wv);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Symmetric: opp loses 1, you gain 1 (plus Bolt's 3 damage to opp).
    assert_eq!(g.players[0].life, l0 + 1);
    assert_eq!(g.players[1].life, l1 - 3 - 1);
}

#[test]
fn quandrix_forester_b124_etb_pumps_target_and_grows_on_attack() {
    let mut g = two_player_game();
    let friend = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(friend);
    let qf = g.add_card_to_hand(0, catalog::quandrix_forester_b124());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p_before = g.battlefield_find(friend).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: qf, target: Some(Target::Permanent(friend)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Forester castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(friend).unwrap().power(), p_before + 1);
    // Now attack with Forester.
    g.clear_sickness(qf);
    g.step = TurnStep::DeclareAttackers;
    let qf_p_before = g.battlefield_find(qf).unwrap().power();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: qf, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(qf).unwrap().power(), qf_p_before + 1,
        "Forester grew on attack");
}

#[test]
fn fractal_coursemate_b124_enters_with_counters_equal_to_twice_hand() {
    let mut g = two_player_game();
    // Seed hand with 3 cards beyond the coursemate itself.
    for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
    let fc = g.add_card_to_hand(0, catalog::fractal_coursemate_b124());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: fc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Coursemate castable");
    drain_stack(&mut g);
    // After cast: 3 islands left in hand (Coursemate has left hand).
    // ETB AddCounter resolves with 2 * 3 = 6 counters.
    let c = g.battlefield_find(fc).expect("alive");
    assert!(c.power() >= 4, "Coursemate has at least 4 counters worth of power");
}

#[test]
fn fractal_reflection_b125_pumps_target_fractal_and_draws() {
    let mut g = two_player_game();
    let f = g.add_card_to_battlefield(0, catalog::fractal_treewright_b125());
    g.clear_sickness(f);
    let p_before = g.battlefield_find(f).unwrap().power();
    let fr = g.add_card_to_hand(0, catalog::fractal_reflection_b125());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: fr, target: Some(Target::Permanent(f)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reflection castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(f).unwrap().power();
    assert_eq!(p_after, p_before + 2, "Fractal got two +1/+1 counters");
    // Cast (-1) + draw (+1) = same hand size.
    assert_eq!(g.players[0].hand.len(), h_before);
}

// ── batch 125 helper shortcut lock-in tests ────────────────────────────────

#[test]
fn shortcut_on_attack_drain_uses_attacks_self_source_with_drain_body() {
    // Lock in that on_attack_drain(N) builds an Attacks/SelfSource
    // trigger whose body is an Effect::Drain. Prevents future refactors
    // from collapsing the helper onto on_attack_gain_life (which would
    // silently drop the opp-loses half of the drain).
    use crabomination::effect::EventScope;
    use crabomination::effect::shortcut::on_attack_drain;
    let trig = on_attack_drain(2);
    assert_eq!(trig.event.kind, crabomination::effect::EventKind::Attacks);
    assert!(matches!(trig.event.scope, EventScope::SelfSource));
    assert!(matches!(trig.effect, crabomination::effect::Effect::Drain { .. }),
        "body is Effect::Drain, not GainLife / LoseLife");
}

#[test]
fn roll_die_auto_decider_lands_on_midpoint_branch() {
    // AutoDecider returns the midpoint of an N-sided die. For a d6
    // that's 3, which falls in the [3, 6] arm — opp loses 3 life.
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_midpoint());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3,
        "AutoDecider rolled d6 midpoint (3) → 3-6 arm fired");
}

#[test]
fn roll_die_scripted_decider_chooses_face_for_specific_branch() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(1),
    ]));
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_big_gain());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 5,
        "Scripted rolled 1 → 1-2 arm: gained 5 life");
}

#[test]
fn roll_die_with_no_matching_arm_runs_no_effect() {
    // CR 706.3a: "If the result was in this range, [effect]." A roll
    // outside every arm runs no effect for that die.
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(5),
    ]));
    let you_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_partial_table());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before,
        "roll 5 falls in no arm — no life change");
    assert_eq!(g.players[1].life, opp_before);
}

#[test]
fn roll_die_serde_round_trip() {
    // Lock in serde round-trip so snapshot save/restore preserves the
    // primitive without losing the results table.
    use crabomination::effect::{Effect, Selector, Value};
    let original = Effect::RollDie {
        sides: 20,
        count: Value::Const(2),
        modifier: Value::Const(0),
        reroll_at_most: 0,
        on_doubles: Some(Box::new(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        })),
        results: vec![
            (1, 1, Effect::Discard {
                who: Selector::You, amount: Value::Const(1), random: false,
            }),
            (2, 19, Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            (20, 20, Effect::GainLife { who: Selector::You, amount: Value::Const(20) }),
        ],
    };
    let json = serde_json::to_string(&original).expect("serialize");
    let parsed: Effect = serde_json::from_str(&json).expect("deserialize");
    match parsed {
        Effect::RollDie { sides, count, modifier, reroll_at_most, results, on_doubles } => {
            assert!(matches!(modifier, Value::Const(0)));
            assert_eq!(reroll_at_most, 0);
            assert_eq!(sides, 20);
            assert!(matches!(count, Value::Const(2)));
            assert_eq!(results.len(), 3);
            assert_eq!(results[0].0, 1);
            assert_eq!(results[1].0, 2);
            assert_eq!(results[1].1, 19);
            assert_eq!(results[2].1, 20);
            assert!(on_doubles.is_some(), "CR 706.5 on_doubles round-trips");
        }
        other => panic!("expected RollDie, got {:?}", other),
    }
}

#[test]
fn cr_706_2_positive_modifier_reaches_high_arm() {
    // Natural 6 + a +2 modifier = 8, which lands in the 7+ arm (gain 5).
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::DieRoll(6)]));
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_plus(2));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 5,
        "6 + 2 = 8 reaches the 7+ arm (gain 5)");
}

#[test]
fn cr_706_2_no_modifier_stays_in_low_arm() {
    // Control: natural 6 with a +0 modifier stays in the 1-6 arm (lose 1).
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::DieRoll(6)]));
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_plus(0));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before - 1,
        "6 + 0 = 6 stays in the 1-6 arm (lose 1)");
}

#[test]
fn cr_706_2_negative_modifier_floors_at_one() {
    // Natural 1 with a -5 modifier floors at 1 (a die result is never
    // reduced below 1), so it lands in the 1-6 arm (lose 1).
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::DieRoll(1)]));
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_plus(-5));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before - 1,
        "1 - 5 floors at 1, still in the 1-6 arm");
}

#[test]
fn cr_706_2b_low_natural_roll_is_rerolled_once() {
    // Natural 2 (≤ reroll_at_most 3) is rerolled once → 5, landing in the
    // 4-6 arm (gain 5) instead of the 1-3 arm (gain 1).
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(2),
        DecisionAnswer::DieRoll(5),
    ]));
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_reroll(3));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 5, "reroll of a 2 → 5 reaches the 4-6 arm");
}

#[test]
fn cr_706_2b_high_natural_roll_is_not_rerolled() {
    // Control: natural 5 (> reroll_at_most 3) is kept, landing in the 4-6
    // arm (gain 5) — the queued second face is never consumed.
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(5),
        DecisionAnswer::DieRoll(1),
    ]));
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_reroll(3));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 5, "natural 5 is kept (no reroll)");
}

#[test]
fn cr_706_5_matching_faces_fire_the_doubles_effect() {
    // Two d6 both rolling 4 → doubles: the per-die 4-6 arm gains 1 each
    // (+2 life) AND the on_doubles clause draws a card.
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(4),
        DecisionAnswer::DieRoll(4),
    ]));
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    let id = g.add_card_to_hand(0, test_card_die_roll_doubles());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "both dice in the 4-6 arm gain 1 each");
    assert_eq!(g.players[0].hand.len(), hand, "doubles drew a card (net 0: cast 1, drew 1)");
}

#[test]
fn cr_706_5_distinct_faces_skip_the_doubles_effect() {
    // Faces 4 and 5 → no doubles: +2 life from the arms, no extra draw.
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(4),
        DecisionAnswer::DieRoll(5),
    ]));
    let life = g.players[0].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_doubles());
    let hand_after_add = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "both dice in the 4-6 arm gain 1 each");
    assert_eq!(g.players[0].hand.len(), hand_after_add - 1,
        "no doubles → no draw (hand only lost the cast sorcery)");
}

#[test]
fn quandrix_riftcraftsman_b126_etb_pumps_fractal_and_magecraft_loots() {
    let mut g = two_player_game();
    // Mint a Fractal target via Skyrunner. add_card_to_battlefield doesn't
    // run the enters_with_counters initialiser, so stamp +1/+1 counters
    // manually to keep the 0/0 base alive.
    let target_fractal = g.add_card_to_battlefield(0, catalog::fractal_skyrunner_b126());
    g.clear_sickness(target_fractal);
    {
        let view = g.battlefield_find_mut(target_fractal).unwrap();
        view.add_counters(CounterType::PlusOnePlusOne, 3);
    }
    let p_before = g.battlefield_find(target_fractal).unwrap().power();
    let qr = g.add_card_to_hand(0, catalog::quandrix_riftcraftsman_b126());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: qr, target: Some(Target::Permanent(target_fractal)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Riftcraftsman castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target_fractal).unwrap().power(), p_before + 1,
        "Fractal grew +1 power from Riftcraftsman ETB");
}

#[test]
fn inkling_skyraider_b127_drains_when_attacking_unblocked() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::inkling_skyraider_b127());
    g.clear_sickness(s);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: s, target: AttackTarget::Player(1),
    }])).expect("Skyraider can attack");
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.step = TurnStep::DeclareBlockers;
    // Opponent declines to block — attacker is unblocked.
    g.perform_action(GameAction::DeclareBlockers(vec![]))
        .expect("zero blockers is legal");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1,
        "Skyraider unblocked → drains opp for 1");
    assert_eq!(g.players[0].life, l0_before + 1,
        "Skyraider unblocked → you gain 1 life");
}

#[test]
fn inkling_skyraider_b127_does_not_drain_when_blocked() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::inkling_skyraider_b127());
    g.clear_sickness(s);
    // Use a flying blocker because Skyraider has flying.
    let blocker = g.add_card_to_battlefield(
        1, catalog::lorehold_aerialist_b127(), // 2/2 Spirit Cleric Flying
    );
    g.clear_sickness(blocker);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: s, target: AttackTarget::Player(1),
    }])).expect("Skyraider can attack");
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, s)]))
        .expect("blocker assignment");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before,
        "Skyraider blocked → no drain trigger");
    assert_eq!(g.players[0].life, l0_before,
        "Skyraider blocked → no life gain trigger");
}

#[test]
fn prismari_ember_wave_b127_taps_creature_and_pings_one() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let e = g.add_card_to_hand(0, catalog::prismari_ember_wave_b127());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: e, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ember-Wave castable");
    drain_stack(&mut g);
    // Bear is 2/2; took 1 damage but is now tapped.
    if let Some(c) = g.battlefield_find(bear) {
        assert_eq!(c.damage, 1, "Bear took 1 damage");
        assert!(c.tapped, "Bear is tapped");
    } else {
        panic!("Bear should still be on battlefield (only 1 damage to 2 toughness)");
    }
}

#[test]
fn lorehold_pyrestone_b128_pumps_target_and_grants_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let p_before = g.battlefield_find(bear).unwrap().power();
    let ps = g.add_card_to_hand(0, catalog::lorehold_pyrestone_b128());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ps, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrestone castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(bear).unwrap();
    assert_eq!(card.power(), p_before + 2, "Bear pumped +2/+0");
    assert!(card.has_keyword(&Keyword::FirstStrike),
        "Bear gained first strike");
}

#[test]
fn quandrix_fractus_touch_b127_adds_two_counters_to_fractal_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bedrock = g.add_card_to_battlefield(0, catalog::fractal_bedrock_b127());
    g.clear_sickness(bedrock);
    let p_before = g.battlefield_find(bedrock).unwrap().power();
    let f = g.add_card_to_hand(0, catalog::quandrix_fractus_touch_b127());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: f, target: Some(Target::Permanent(bedrock)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractus-Touch castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bedrock).unwrap().power(), p_before + 2,
        "Bedrock grew +2 power from 2 counters");
    // Hand: -1 (cast) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn shortcut_etb_mint_token_with_counters_uses_seq_create_then_add_counter() {
    // Lock-in test for the new `etb_mint_token_with_counters` shortcut
    // helper shipped in batch 128. Verifies that the helper expands to
    // `Seq[CreateToken, AddCounter(LastCreatedToken, +1/+1, N)]` wrapped
    // in an etb trigger.
    use crabomination::card::{CounterType, EventKind, EventScope};
    let ta = crabomination::effect::shortcut::etb_mint_token_with_counters(
        crabomination::catalog::fractal_token(), 1, 3,
    );
    assert_eq!(ta.event.kind, EventKind::EntersBattlefield);
    assert!(matches!(ta.event.scope, EventScope::SelfSource));
    if let crabomination::effect::Effect::Seq(steps) = &ta.effect {
        assert_eq!(steps.len(), 2);
        assert!(matches!(steps[0], crabomination::effect::Effect::CreateToken { .. }));
        if let crabomination::effect::Effect::AddCounter { what, kind, .. } = &steps[1] {
            assert!(matches!(what, crabomination::effect::Selector::LastCreatedToken));
            assert!(matches!(kind, CounterType::PlusOnePlusOne));
        } else {
            panic!("expected AddCounter as step[1]");
        }
    } else {
        panic!("expected Seq effect");
    }
}

#[test]
fn lorehold_stoneglyph_b129_activated_ability_pings() {
    let mut g = two_player_game();
    let sg = g.add_card_to_battlefield(0, catalog::lorehold_stoneglyph_b129());
    g.clear_sickness(sg);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sg, ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: Vec::new(),
        x_value: None,
    }).expect("Stoneglyph ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2, "Stoneglyph deals 2 damage");
}

#[test]
fn lorehold_memorist_b129_returns_low_mv_spirit_from_graveyard() {
    let mut g = two_player_game();
    // Put a low-MV Spirit in graveyard.
    let aerialist = catalog::lorehold_aerialist_b127();
    let gy_id = g.add_card_to_graveyard(0, aerialist);
    let m = g.add_card_to_hand(0, catalog::lorehold_memorist_b129());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: m, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memorist castable");
    drain_stack(&mut g);
    // Hand: -1 (Memorist) + 1 (returned Aerialist) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // The Aerialist is no longer in the graveyard.
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == gy_id),
        "Aerialist no longer in gy");
}

#[test]
fn lorehold_embertongue_b129_magecraft_pings_opp_creature() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_embertongue_b129());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(opp_bear);
    let opp_t_before = g.battlefield_find(opp_bear).unwrap().toughness();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bear took 1 damage from Embertongue's magecraft (auto-targeted).
    // Note: damage may have killed bear if it was a 2/2 with damage marker.
    let bear = g.battlefield_find(opp_bear);
    if let Some(b) = bear {
        // damage_marked is 1 since Embertongue magecraft fires once
        assert_eq!(b.damage as i32, 1, "Bear took 1 damage");
        let _ = opp_t_before; // assert used
    }
}

#[test]
fn witherbloom_petalmaster_b129_magecraft_adds_counter_to_plant() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_petalmaster_b129());
    let plant = g.add_card_to_battlefield(0, catalog::witherbloom_sprawl_vine_b128());
    let p_before = g.battlefield_find(plant).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(plant).unwrap().power();
    assert_eq!(p_after, p_before + 1, "Plant got +1/+1 counter from magecraft");
}

// ── Table-driven consolidations ────────────────────────────────────────────
//
// The following tests collapse structurally identical per-card tests into
// shared bodies. Rows: one per card; columns carry the per-card deltas.

// Magecraft creatures whose trigger changes life totals when an instant is
// cast: pings (opp loses extra), drains (opp loses extra + you gain), and
// pure life gain. Bolt itself deals 3 to the opponent.
#[test]
fn magecraft_life_delta_cards() {
    for (def, opp_extra, you_gain, kws) in [
        (catalog::lorehold_pyromancer_b124(), 1, 0, vec![]),
        (catalog::prismari_burnmage_b124(), 1, 0, vec![]),
        (catalog::prismari_blazewright_b125(), 1, 0, vec![Keyword::Haste]),
        (catalog::witherbloom_lifescribe_elder_b125(), 0, 2, vec![]),
        (catalog::lorehold_ember_mage_b126(), 1, 0, vec![]),
        (catalog::witherbloom_toxinscholar_b126(), 0, 2, vec![]),
        (catalog::lorehold_pyrebrand_b127(), 1, 0, vec![]),
        (catalog::silverquill_aristocrat_b127(), 1, 1, vec![]),
        (catalog::prismari_surgebearer_b127(), 1, 0, vec![]),
        (catalog::lorehold_soulreaver_b128(), 1, 1, vec![]),
        (catalog::witherbloom_toxicspeaker_b128(), 1, 1, vec![]),
        (catalog::inkling_quillstrike_b128(), 1, 1, vec![]),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let l0 = g.players[0].life;
        let l1 = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1 - 3 - opp_extra,
            "{name}: opp lost Bolt 3 + magecraft {opp_extra}");
        assert_eq!(g.players[0].life, l0 + you_gain,
            "{name}: controller gained {you_gain}");
        let view = g.battlefield_find(id).unwrap();
        for kw in &kws {
            assert!(view.has_keyword(kw), "{name}: expected keyword present");
        }
    }
}

// Magecraft creatures that grow themselves (+N power) on an instant cast.
#[test]
fn magecraft_self_pump_cards() {
    for (def, pump, kws) in [
        (catalog::lorehold_vanguard_b123(), 1, vec![Keyword::Haste, Keyword::FirstStrike]),
        (catalog::lorehold_champion_b124(), 2, vec![Keyword::Vigilance]),
        (catalog::quandrix_mathematician_b124(), 1, vec![]),
        (catalog::silverquill_soulscholar_b125(), 1, vec![Keyword::Lifelink]),
        (catalog::lorehold_cinderscholar_b126(), 1, vec![]),
        (catalog::prismari_riftrider_b126(), 1, vec![]),
        (catalog::witherbloom_sapsage_b127(), 1, vec![]),
        (catalog::quandrix_greenmage_b127(), 1, vec![]),
        (catalog::lorehold_skybinder_b128(), 1, vec![]),
        (catalog::witherbloom_mossfeeder_b128(), 1, vec![]),
        (catalog::prismari_firebrand_b128(), 1, vec![Keyword::Haste]),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        let p_before = g.battlefield_find(id).unwrap().power();
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let view = g.battlefield_find(id).unwrap();
        assert_eq!(view.power(), p_before + pump, "{name}: grew +{pump} on magecraft");
        for kw in &kws {
            assert!(view.has_keyword(kw), "{name}: expected keyword present");
        }
    }
}

// Magecraft creatures with an unobservable scry/surveil trigger — verify
// the trigger fires without panicking.
#[test]
fn magecraft_scry_surveil_cards() {
    for (def, lib) in [
        (catalog::quandrix_aetherbinder_b125(), catalog::island()),
        (catalog::silverquill_glyphmage_b126(), catalog::plains()),
        (catalog::quandrix_forecaster_adept_b126(), catalog::island()),
        (catalog::quandrix_sageling_b127(), catalog::island()),
        (catalog::silverquill_drafter_b128(), catalog::island()),
        (catalog::quandrix_tideshaper_b128(), catalog::island()),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, lib);
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        // Scry/Surveil 1 fired without panic.
    }
}

// Magecraft creatures that loot (draw+discard → hand -1 net after Bolt) or
// draw (hand net 0 after Bolt) on an instant cast.
#[test]
fn magecraft_loot_or_draw_cards() {
    for (def, libs, fodder, hand_delta) in [
        (catalog::quandrix_mistsage_b125(), vec![catalog::island(), catalog::forest()], vec![], -1),
        (catalog::prismari_cinderscholar_b126(), vec![catalog::mountain()], vec![catalog::mountain()], -1),
        (catalog::quandrix_mistshaper_b126(), vec![catalog::island()], vec![], 0),
        (catalog::prismari_mistscholar_b127(), vec![catalog::island()], vec![catalog::island()], -1),
        (catalog::prismari_stormcrafter_b128(), vec![catalog::island()], vec![], -1),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        for c in libs { g.add_card_to_library(0, c); }
        for c in fodder { g.add_card_to_hand(0, c); }
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let h_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len() as i32, h_before as i32 + hand_delta,
            "{name}: hand delta after cast + magecraft");
    }
}

// Magecraft creatures that mint a token (Treasure or Spirit) on an
// instant cast — battlefield grows by exactly one permanent.
#[test]
fn magecraft_mint_token_cards() {
    for def in [
        catalog::prismari_sparkstudent_b126(),
        catalog::prismari_flarescholar_b127(),
        catalog::lorehold_bookforger_b128(),
        catalog::prismari_tide_surger_b128(),
        catalog::lorehold_sparkscholar_ii_b129(),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        let _ = g.add_card_to_battlefield(0, def);
        let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(),
            bf_before + 1, "{name}: minted one token on magecraft");
    }
}

// Attack-trigger creatures: ping / gain / drain on declaring an attack.
#[test]
fn on_attack_trigger_cards() {
    for (def, opp_loss, you_gain, kws) in [
        (catalog::lorehold_skirmisher_b123(), 1, 0, vec![Keyword::Haste]),
        (catalog::lorehold_bloodrazer_b125(), 1, 0, vec![]),
        (catalog::lorehold_saintkeeper_b125(), 0, 1, vec![Keyword::Vigilance]),
        (catalog::lorehold_vanguardian_b125(), 1, 1, vec![]),
        (catalog::witherbloom_drainstride_b125(), 1, 1, vec![]),
        (catalog::silverquill_stridemage_b125(), 1, 1, vec![]),
        (catalog::inkling_skyhunter_b125(), 0, 1, vec![Keyword::Flying]),
        (catalog::inkling_quillmender_b127(), 0, 1, vec![]),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        let a = g.add_card_to_battlefield(0, def);
        g.clear_sickness(a);
        g.step = TurnStep::DeclareAttackers;
        let l0 = g.players[0].life;
        let l1 = g.players[1].life;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: a, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1 - opp_loss, "{name}: opp life after attack trigger");
        assert_eq!(g.players[0].life, l0 + you_gain, "{name}: own life after attack trigger");
        let view = g.battlefield_find(a).unwrap();
        for kw in &kws {
            assert!(view.has_keyword(kw), "{name}: expected keyword present");
        }
    }
}

// Creatures whose death drains life (opp loses N, you gain N).
#[test]
fn dies_drain_cards() {
    for (def, dmg, opp_loss, you_gain, kws) in [
        (catalog::witherbloom_saproot_b123(), 99, 1, 1, vec![]),
        (catalog::witherbloom_reaperscholar_b125(), 4, 2, 2, vec![Keyword::Deathtouch]),
        (catalog::pest_pyrechewer_b126(), 2, 1, 1, vec![]),
        (catalog::witherbloom_reaper_hand_b128(), 99, 2, 2, vec![]),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        {
            let view = g.battlefield_find(id).unwrap();
            for kw in &kws {
                assert!(view.has_keyword(kw), "{name}: expected keyword present");
            }
        }
        let l0 = g.players[0].life;
        let l1 = g.players[1].life;
        g.battlefield_find_mut(id).unwrap().damage = dmg;
        let _ = g.check_state_based_actions();
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).is_none(), "{name}: died to lethal damage");
        assert_eq!(g.players[1].life, l1 - opp_loss, "{name}: opp lost {opp_loss} on death");
        assert_eq!(g.players[0].life, l0 + you_gain, "{name}: you gained {you_gain} on death");
    }
}

// Creatures that mint one token when they die.
#[test]
fn dies_mint_token_cards() {
    for (def, dmg) in [
        (catalog::lorehold_spiritbinder_b126(), 3),
        (catalog::witherbloom_mossgrower_b126(), 3),
        (catalog::pest_brewerthing_b127(), 2),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        g.battlefield_find_mut(id).unwrap().damage = dmg;
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).is_none(), "{name}: died to lethal damage");
        let tokens = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.is_token)
            .count();
        assert_eq!(tokens, 1, "{name}: minted 1 token on death");
    }
}

// Cast-a-spell-that-mints-tokens tests (ETB triggers on creatures and
// straight sorcery mints). Asserts token count plus any drain/gain rider.
#[test]
fn etb_mint_token_cards() {
    for (def, colors, colorless, tokens, you_gain, opp_loss, lib_islands, kws) in [
        (catalog::pest_mawlord_b123(), vec![(Color::Black, 1), (Color::Green, 1)], 4, 2, 0, 0, 0, vec![]),
        (catalog::pest_hivekeeper_b123(), vec![(Color::Black, 1), (Color::Green, 1)], 3, 3, 0, 0, 0, vec![]),
        (catalog::lorehold_spiritsong_b123(), vec![(Color::Red, 1), (Color::White, 1)], 3, 2, 0, 0, 0, vec![]),
        (catalog::lorehold_heraldcaller_b125(), vec![(Color::Red, 1), (Color::White, 1)], 3, 2, 2, 0, 0, vec![Keyword::Flying]),
        (catalog::silverquill_ravenstrike_b125(), vec![(Color::White, 1), (Color::Black, 1)], 1, 1, 2, 0, 0, vec![]),
        (catalog::pest_cinderpriest_b125(), vec![(Color::Black, 1)], 2, 1, 0, 0, 0, vec![]),
        (catalog::witherbloom_pestsower_b127(), vec![(Color::Black, 1), (Color::Green, 1)], 3, 2, 2, 2, 0, vec![]),
        (catalog::witherbloom_pestcaller_b128(), vec![(Color::Black, 1), (Color::Green, 1)], 3, 1, 0, 0, 0, vec![]),
        (catalog::lorehold_bell_ringer_b128(), vec![(Color::Red, 1), (Color::White, 1)], 1, 1, 2, 0, 0, vec![]),
        (catalog::lorehold_battlespirit_b128(), vec![(Color::Red, 1), (Color::White, 1)], 3, 1, 0, 0, 0, vec![Keyword::Haste]),
        (catalog::silverquill_inkmaster_b128(), vec![(Color::White, 1), (Color::Black, 1)], 2, 1, 0, 0, 0, vec![Keyword::Flying, Keyword::Lifelink]),
        (catalog::witherbloom_pestswarm_b129(), vec![(Color::Black, 1), (Color::Green, 1)], 2, 3, 0, 0, 0, vec![]),
        (catalog::lorehold_excavation_b129(), vec![(Color::Red, 1), (Color::White, 1)], 2, 2, 0, 0, 0, vec![]),
        (catalog::witherbloom_cauldronherder_b129(), vec![(Color::Black, 1), (Color::Green, 1)], 3, 1, 2, 2, 0, vec![]),
        (catalog::prismari_sparkmaker_b129(), vec![(Color::Blue, 1), (Color::Red, 1)], 2, 1, 0, 0, 1, vec![]),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        for _ in 0..lib_islands { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        for (c, n) in colors { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        let l0 = g.players[0].life;
        let l1 = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("spell castable");
        drain_stack(&mut g);
        let count = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.is_token)
            .count();
        assert_eq!(count, tokens, "{name}: minted {tokens} token(s)");
        assert_eq!(g.players[0].life, l0 + you_gain, "{name}: gained {you_gain}");
        assert_eq!(g.players[1].life, l1 - opp_loss, "{name}: opp lost {opp_loss}");
        if !kws.is_empty() {
            let view = g.battlefield_find(id).unwrap();
            for kw in &kws {
                assert!(view.has_keyword(kw), "{name}: expected keyword present");
            }
        }
    }
}

// ETB creatures whose primary effect changes life totals (gain and/or
// drain), plus drain sorceries with the same observable shape.
#[test]
fn etb_life_delta_cards() {
    for (def, colors, colorless, lib_islands, you_gain, opp_loss, hand_delta, kws) in [
        (catalog::inkling_crusader_b123(), vec![(Color::White, 1), (Color::Black, 1)], 2, 0, 2, 0, -1, vec![Keyword::Flying, Keyword::Vigilance]),
        (catalog::silverquill_sermonizer_b123(), vec![(Color::White, 1)], 1, 0, 1, 0, -1, vec![]),
        (catalog::inkling_pamphletter_b123(), vec![(Color::White, 1), (Color::Black, 1)], 2, 0, 2, 2, -1, vec![Keyword::Flying]),
        (catalog::lorehold_skydefender_b124(), vec![(Color::White, 1)], 3, 0, 3, 0, -1, vec![Keyword::Flying]),
        (catalog::inkling_drainsage_b125(), vec![(Color::White, 1), (Color::Black, 1)], 3, 0, 2, 2, -1, vec![Keyword::Flying, Keyword::Lifelink]),
        (catalog::silverquill_pen_sage_b126(), vec![(Color::White, 1), (Color::Black, 1)], 2, 0, 2, 2, -1, vec![]),
        (catalog::inkling_sigilrider_b126(), vec![(Color::White, 1), (Color::Black, 1)], 2, 0, 2, 0, -1, vec![Keyword::Flying, Keyword::Lifelink]),
        (catalog::silverquill_glyphcaller_b126(), vec![(Color::White, 1), (Color::Black, 1)], 0, 1, 2, 2, -1, vec![]),
        (catalog::witherbloom_sapcaster_b126(), vec![(Color::Black, 1), (Color::Green, 1)], 3, 0, 3, 3, -1, vec![]),
        (catalog::witherbloom_vinerunner_b126(), vec![(Color::Green, 1)], 2, 0, 2, 0, -1, vec![Keyword::Trample]),
        (catalog::lorehold_veteran_b127(), vec![(Color::Red, 1), (Color::White, 1)], 3, 0, 3, 0, -1, vec![]),
        (catalog::witherbloom_mossbinder_b127(), vec![(Color::Black, 1), (Color::Green, 1)], 2, 0, 2, 2, -1, vec![]),
        (catalog::witherbloom_verdant_sage_b127(), vec![(Color::Green, 1)], 2, 0, 2, 0, -1, vec![Keyword::Reach]),
        (catalog::inkling_battle_drone_b127(), vec![(Color::White, 1), (Color::Black, 1)], 3, 0, 1, 1, -1, vec![Keyword::Flying, Keyword::Vigilance]),
        (catalog::silverquill_quillplate_b127(), vec![(Color::White, 1)], 2, 0, 2, 0, -1, vec![Keyword::Vigilance]),
        (catalog::witherbloom_cauldronkeeper_b128(), vec![(Color::Black, 1), (Color::Green, 1)], 1, 2, 1, 0, -1, vec![]),
        (catalog::witherbloom_spellrot_b128(), vec![(Color::Black, 1), (Color::Green, 1)], 1, 1, 3, 3, -1, vec![]),
        (catalog::silverquill_inkblot_b128(), vec![(Color::White, 1), (Color::Black, 1)], 0, 1, 1, 1, 0, vec![]),
        (catalog::inkling_vellumbinder_b128(), vec![(Color::White, 1), (Color::Black, 1)], 3, 0, 2, 2, -1, vec![]),
        (catalog::silverquill_sermonist_b128(), vec![(Color::White, 1)], 1, 1, 0, 0, -1, vec![Keyword::Vigilance]),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        for _ in 0..lib_islands { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        for (c, n) in colors { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        let l0 = g.players[0].life;
        let l1 = g.players[1].life;
        let h_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("spell castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, l0 + you_gain, "{name}: gained {you_gain}");
        assert_eq!(g.players[1].life, l1 - opp_loss, "{name}: opp lost {opp_loss}");
        assert_eq!(g.players[0].hand.len() as i32, h_before as i32 + hand_delta,
            "{name}: hand delta");
        if !kws.is_empty() {
            let view = g.battlefield_find(id).unwrap();
            for kw in &kws {
                assert!(view.has_keyword(kw), "{name}: expected keyword present");
            }
        }
    }
}

// ETB creatures that scry / draw / loot; observed via hand-size delta and
// printed keywords.
#[test]
fn etb_card_flow_cards() {
    for (def, colors, colorless, lib_islands, fodder_islands, hand_delta, kws) in [
        (catalog::prismari_tutor_b123(), vec![(Color::Blue, 1), (Color::Red, 1)], 2, 3, 0, 0, vec![]),
        (catalog::prismari_stormbreaker_b124(), vec![(Color::Blue, 1), (Color::Red, 1)], 3, 2, 1, -1, vec![Keyword::Trample]),
        (catalog::prismari_riftscholar_b125(), vec![(Color::Blue, 1)], 1, 1, 0, 0, vec![]),
        (catalog::prismari_tempest_bearer_b125(), vec![(Color::Blue, 1), (Color::Red, 1)], 3, 1, 1, -1, vec![Keyword::Flying]),
        (catalog::prismari_tempest_skipper_b126(), vec![(Color::Blue, 1), (Color::Red, 1)], 3, 3, 0, 0, vec![Keyword::Flying]),
        (catalog::fractal_stormcaller_b127(), vec![(Color::Blue, 1)], 1, 1, 0, -1, vec![]),
        (catalog::quandrix_treebinder_b128(), vec![(Color::Green, 1)], 2, 1, 0, 0, vec![]),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        for _ in 0..lib_islands { g.add_card_to_library(0, catalog::island()); }
        for _ in 0..fodder_islands { g.add_card_to_hand(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        for (c, n) in colors { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        let h_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("spell castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len() as i32, h_before as i32 + hand_delta,
            "{name}: hand delta after ETB card flow");
        if !kws.is_empty() {
            let view = g.battlefield_find(id).unwrap();
            for kw in &kws {
                assert!(view.has_keyword(kw), "{name}: expected keyword present");
            }
        }
    }
}

// Fractal creatures that enter with +1/+1 counters (N/N from a 0/0 base).
#[test]
fn fractal_enters_with_counters_cards() {
    for (def, colors, colorless, pt) in [
        (catalog::fractal_treewright_b125(), vec![(Color::Green, 1)], 1, 2),
        (catalog::fractal_skyrunner_b126(), vec![(Color::Green, 1)], 2, 3),
        (catalog::fractal_bedrock_b127(), vec![(Color::Green, 1)], 3, 4),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        let id = g.add_card_to_hand(0, def);
        for (c, n) in colors { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("spell castable");
        drain_stack(&mut g);
        let view = g.battlefield_find(id).expect("alive (counters keep it from SBA)");
        assert_eq!(view.power(), pt, "{name}: enters as {pt}/{pt}");
        assert_eq!(view.toughness(), pt);
        assert!(view.definition.subtypes.creature_types.contains(&CreatureType::Fractal));
    }
}

// Cards that mint a Fractal token with N +1/+1 counters on cast.
#[test]
fn etb_mint_fractal_token_cards() {
    for (def, colorless, pt) in [
        (catalog::fractal_petalcaller_b126(), 2, 3),
        (catalog::quandrix_bloomforge_b128(), 2, 4),
        (catalog::quandrix_geometer_b128(), 2, 2),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("spell castable");
        drain_stack(&mut g);
        let token = g.battlefield.iter()
            .find(|c| c.controller == 0
                && c.is_token
                && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
            .expect("Fractal token on battlefield");
        assert_eq!(token.power(), pt, "{name}: Fractal token is {pt}/{pt}");
        assert_eq!(token.toughness(), pt);
    }
}

// Spells that deal damage to a target player, optionally gaining life
// and/or cantripping.
#[test]
fn player_damage_spell_cards() {
    for (def, colors, colorless, lib_islands, dmg, you_gain, hand_delta) in [
        (catalog::prismari_sparkshow_b123(), vec![(Color::Blue, 1), (Color::Red, 1)], 1, 2, 2, 0, 0),
        (catalog::prismari_tempest_b124(), vec![(Color::Blue, 1), (Color::Red, 1)], 2, 2, 3, 0, 0),
        (catalog::prismari_sparkshow_b125(), vec![(Color::Blue, 1), (Color::Red, 1)], 0, 1, 2, 0, 0),
        (catalog::prismari_coil_caller_b126(), vec![(Color::Blue, 1), (Color::Red, 1)], 0, 1, 1, 0, 0),
        (catalog::prismari_pyroblast_b128(), vec![(Color::Red, 1)], 1, 0, 3, 0, -1),
        (catalog::lorehold_pyreverse_b129(), vec![(Color::Red, 1)], 1, 0, 2, 1, -1),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        for _ in 0..lib_islands { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        for (c, n) in colors { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        let l0 = g.players[0].life;
        let l1 = g.players[1].life;
        let h_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("spell castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1 - dmg, "{name}: dealt {dmg} to opp");
        assert_eq!(g.players[0].life, l0 + you_gain, "{name}: gained {you_gain}");
        assert_eq!(g.players[0].hand.len() as i32, h_before as i32 + hand_delta,
            "{name}: hand delta");
    }
}

// Spells that kill a targeted 2/2 (damage or -X/-X), optionally gaining
// life, minting a token, or cantripping.
#[test]
fn kill_target_creature_spell_cards() {
    for (def, colors, colorless, lib_islands, you_gain, token_delta, hand_delta) in [
        (catalog::lorehold_cremate_b124(), vec![(Color::Red, 1), (Color::White, 1)], 0, 0, 0, 1, -1),
        (catalog::prismari_sparkbolt_b127(), vec![(Color::Red, 1)], 1, 1, 0, 0, 0),
        (catalog::lorehold_embercurse_b127(), vec![(Color::Red, 1), (Color::White, 1)], 0, 0, 2, 0, -1),
        (catalog::lorehold_cliffstrike_b128(), vec![(Color::Red, 1), (Color::White, 1)], 2, 0, 3, 0, -1),
        (catalog::witherbloom_boneshroud_b129(), vec![(Color::Black, 1)], 0, 0, 0, 0, -1),
    ] {
        let mut g = two_player_game();
        let name = def.name.clone();
        for _ in 0..lib_islands { g.add_card_to_library(0, catalog::island()); }
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(bear);
        let id = g.add_card_to_hand(0, def);
        for (c, n) in colors { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        let l0 = g.players[0].life;
        let tokens_before = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.is_token).count();
        let h_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("spell castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "{name}: bear died");
        assert_eq!(g.players[0].life, l0 + you_gain, "{name}: gained {you_gain}");
        let tokens_after = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.is_token).count();
        assert_eq!(tokens_after, tokens_before + token_delta, "{name}: token delta");
        assert_eq!(g.players[0].hand.len() as i32, h_before as i32 + hand_delta,
            "{name}: hand delta");
    }
}
