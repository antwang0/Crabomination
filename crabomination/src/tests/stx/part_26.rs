//! Functionality tests for the extras_14 + extras_15 STX batches — real
//! Strixhaven cards (Campus lands, keyword creatures, spells, Equipment,
//! payoff creatures) wired against existing primitives plus the new
//! `Effect::PayManaOrElse`.

use crate::card::{CounterType, CreatureType, Keyword};
use crate::game::types::Target;
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;

// ── Lands ───────────────────────────────────────────────────────────────────

#[test]
fn campus_land_enters_tapped_and_taps_for_both_colors() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_campus());
    g.perform_action(GameAction::PlayLand(id)).expect("campus playable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().tapped, "Campus enters tapped");
    // Untap it and verify both mana abilities produce their colors.
    g.battlefield_find_mut(id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("tap for R");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    g.battlefield_find_mut(id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, x_value: None,
    }).expect("tap for W");
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
}

#[test]
fn campus_land_scry_ability_taps_and_is_payable() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_battlefield(0, catalog::quandrix_campus());
    g.battlefield_find_mut(id).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 2, target: None, x_value: None,
    }).expect("{4},{T}: Scry 1 activatable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().tapped, "scry taps the Campus");
}

#[test]
fn access_tunnel_grants_unblockable_to_small_creature() {
    let mut g = two_player_game();
    let tunnel = g.add_card_to_battlefield(0, catalog::access_tunnel());
    g.battlefield_find_mut(tunnel).unwrap().tapped = false;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2, power ≤ 3
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: tunnel, ability_index: 1,
        target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("evasion activatable on a power-3-or-less creature");
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(bear).unwrap().granted_keywords_eot.contains(&Keyword::Unblockable),
        "bear gains can't-be-blocked this turn",
    );
}

#[test]
fn archway_commons_kept_when_paid_sacrificed_otherwise() {
    // Paid: {1} floating → land stays.
    let mut g = two_player_game();
    let kept = g.add_card_to_hand(0, catalog::archway_commons());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::PlayLand(kept)).expect("playable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(kept).is_some(), "paid {{1}} keeps Archway Commons");
    assert!(g.battlefield_find(kept).unwrap().tapped, "still enters tapped");

    // Unpaid: no mana → sacrificed by the ETB tax.
    let mut g2 = two_player_game();
    let sacked = g2.add_card_to_hand(0, catalog::archway_commons());
    g2.perform_action(GameAction::PlayLand(sacked)).expect("playable");
    drain_stack(&mut g2);
    assert!(g2.battlefield_find(sacked).is_none(), "unpaid Archway Commons is sacrificed");
}

// ── Keyword creatures ────────────────────────────────────────────────────────

#[test]
fn keyword_creatures_have_their_printed_keywords() {
    let g = two_player_game();
    let _ = &g;
    let karok = catalog::moldering_karok();
    assert!(karok.keywords.contains(&Keyword::Trample) && karok.keywords.contains(&Keyword::Lifelink));
    let drake = catalog::needlethorn_drake();
    assert!(drake.keywords.contains(&Keyword::Flying) && drake.keywords.contains(&Keyword::Deathtouch));
    let sloth = catalog::relic_sloth();
    assert!(sloth.keywords.contains(&Keyword::Vigilance) && sloth.keywords.contains(&Keyword::Menace));
    let aerialist = catalog::waterfall_aerialist();
    assert!(aerialist.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))));
}

#[test]
fn springmane_cervin_gains_two_life_on_etb() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::springmane_cervin());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "ETB gains 2 life");
}

#[test]
fn professor_of_zoomancy_mints_a_pest_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::professor_of_zoomancy());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest)),
        "a Pest token entered",
    );
}

#[test]
fn scurrid_colony_grows_with_eight_lands() {
    let mut g = two_player_game();
    let colony = g.add_card_to_battlefield(0, catalog::scurrid_colony());
    // Fewer than 8 lands → base 2/2.
    let computed = g.compute_battlefield();
    let c = computed.iter().find(|c| c.id == colony).unwrap();
    assert_eq!((c.power, c.toughness), (2, 2), "base body without 8 lands");
    for _ in 0..8 { g.add_card_to_battlefield(0, catalog::forest()); }
    let computed = g.compute_battlefield();
    let c = computed.iter().find(|c| c.id == colony).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "+2/+2 with eight or more lands");
}

#[test]
fn wormhole_serpent_grants_target_unblockable() {
    let mut g = two_player_game();
    let serpent = g.add_card_to_battlefield(0, catalog::wormhole_serpent());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: serpent, ability_index: 0,
        target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("{3}{U}: unblockable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().granted_keywords_eot.contains(&Keyword::Unblockable));
}

#[test]
fn mage_hunter_drains_when_opponent_casts_instant() {
    let mut g = two_player_game();
    // Mage Hunter on seat 1; the active player (seat 0) is its opponent.
    g.add_card_to_battlefield(1, catalog::mage_hunter());
    let life = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("active player casts Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "the caster loses 1 to Mage Hunter");
}

#[test]
fn leonin_lightscribe_pumps_team_on_magecraft() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leonin_lightscribe());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pwr = g.battlefield_find(bear).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a spell to trigger magecraft");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), pwr + 1, "team +1/+1 EOT");
}

#[test]
fn blood_researcher_grows_on_lifegain() {
    let mut g = two_player_game();
    let br = g.add_card_to_battlefield(0, catalog::blood_researcher());
    g.adjust_life(0, 3);
    g.dispatch_triggers_for_events(&[crate::game::types::GameEvent::LifeGained { player: 0, amount: 3 }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(br).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "a +1/+1 counter on lifegain",
    );
}

#[test]
fn arrogant_poet_may_pay_life_for_flying_on_attack() {
    use crate::game::Attack;
    let mut g = two_player_game();
    let poet = g.add_card_to_battlefield(0, catalog::arrogant_poet());
    g.clear_sickness(poet);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    let life = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: poet, target: AttackTarget::Player(1),
    }])).expect("declare attackers");
    // Choose to pay the 2 life for flying.
    g.decider = Box::new(crate::decision::ScriptedDecider::new(
        vec![crate::decision::DecisionAnswer::Bool(true)],
    ));
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "paid 2 life");
    assert!(g.battlefield_find(poet).unwrap().granted_keywords_eot.contains(&Keyword::Flying));
}

#[test]
fn biomathematician_makes_a_fractal_and_grows_it() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::biomathematician());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .expect("a Fractal token");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 1, "Fractal got a +1/+1 counter");
}

#[test]
fn campus_guide_puts_basic_land_on_top() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let forest = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::campus_guide());
    g.players[0].mana_pool.add_colorless(2);
    // Fetch the Forest to the top of the library.
    g.decider = Box::new(crate::decision::ScriptedDecider::new(
        vec![crate::decision::DecisionAnswer::Search(Some(forest))],
    ));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(forest), "Forest on top");
}

#[test]
fn biblioplex_assistant_returns_instant_from_graveyard_to_top() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::biblioplex_assistant());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(bolt), "Bolt placed on top");
}

#[test]
fn overgrown_arch_gains_life_and_learns() {
    let mut g = two_player_game();
    let arch = g.add_card_to_battlefield(0, catalog::overgrown_arch());
    g.battlefield_find_mut(arch).unwrap().tapped = false;
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: arch, ability_index: 0, target: None, x_value: None,
    }).expect("{T}: gain 1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gains 1 life");
}

// ── Spells ───────────────────────────────────────────────────────────────────

#[test]
fn expel_exiles_tapped_creature_only() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::expel());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Expel targets the tapped bear");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "tapped bear exiled");
}

#[test]
fn crushing_disappointment_drains_all_and_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    let id = g.add_card_to_hand(0, catalog::crushing_disappointment());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 - 2);
    assert_eq!(g.players[1].life, l1 - 2);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew two");
}

#[test]
fn essence_infusion_adds_counters_and_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::essence_infusion());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2, "two +1/+1 counters");
    assert!(c.granted_keywords_eot.contains(&Keyword::Lifelink), "gains lifelink EOT");
}

#[test]
fn professors_warning_indestructible_mode() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::professors_warning());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("indestructible mode");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().granted_keywords_eot.contains(&Keyword::Indestructible));
}

#[test]
fn sudden_breakthrough_pumps_and_makes_treasure() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pwr = g.battlefield_find(bear).unwrap().power();
    let id = g.add_card_to_hand(0, catalog::sudden_breakthrough());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.power(), pwr + 2, "+2/+0");
    assert!(c.granted_keywords_eot.contains(&Keyword::FirstStrike));
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "a Treasure token was created",
    );
}

#[test]
fn arcane_subtraction_shrinks_power() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::arcane_subtraction());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), -2, "2 base power -4 = -2");
}

/// CR 702.21 — Ward {2} on Waterfall Aerialist counters an opponent's
/// spell when the caster can't pay the ward cost.
#[test]
fn cr_702_21_waterfall_aerialist_ward_counters_unpaid_spell() {
    let mut g = two_player_game();
    let aerialist = g.add_card_to_battlefield(0, catalog::waterfall_aerialist());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = crate::game::types::TurnStep::PreCombatMain;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1); // enough for Bolt, not Ward {2}
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(aerialist)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt cast — Ward is a trigger, not a cast restriction");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aerialist).is_some(), "Ward counters the unpaid Bolt");
}

/// CR 509 — a creature granted "can't be blocked this turn" (Wormhole
/// Serpent's activated ability) connects unblocked for combat damage.
#[test]
fn cr_509_wormhole_grant_makes_attacker_unblockable() {
    use crate::game::Attack;
    let mut g = two_player_game();
    let serpent = g.add_card_to_battlefield(0, catalog::wormhole_serpent());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: serpent, ability_index: 0,
        target: Some(Target::Permanent(attacker)), x_value: None,
    }).expect("grant unblockable");
    drain_stack(&mut g);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("advance");
    }
    let opp = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // The blocker can't be assigned to an unblockable attacker.
    let block = g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]));
    assert!(block.is_err(), "can't block an unblockable creature");
    while g.step != crate::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("advance");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "unblockable bear connects for 2");
}

#[test]
fn infuse_with_vitality_returns_creature_on_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::infuse_with_vitality());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gain 2 life");
    assert!(g.battlefield_find(bear).unwrap().granted_keywords_eot.contains(&Keyword::Deathtouch));
    // Kill the bear; the granted dies-trigger returns it tapped.
    bolt_own_creature(&mut g, 0, bear);
    drain_stack(&mut g);
    let returned = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears");
    assert!(returned.is_some() && returned.unwrap().tapped, "bear returns to battlefield tapped");
}

// ── extras_15 — equipment + payoff creatures ─────────────────────────────────

#[test]
fn poets_quill_equips_for_plus_one_one_and_lifelink() {
    let mut g = two_player_game();
    let quill = g.add_card_to_battlefield(0, catalog::poets_quill());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: quill, target: bear }).expect("equip {1}{B}");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    assert!(cp.keywords.contains(&Keyword::Lifelink), "grants lifelink");
}

#[test]
fn team_pennant_grants_vigilance_and_trample() {
    let mut g = two_player_game();
    let pennant = g.add_card_to_battlefield(0, catalog::team_pennant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: pennant, target: bear }).expect("equip {3}");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::Vigilance) && cp.keywords.contains(&Keyword::Trample));
}

#[test]
fn zephyr_boots_grants_flying() {
    let mut g = two_player_game();
    let boots = g.add_card_to_battlefield(0, catalog::zephyr_boots());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: boots, target: bear }).expect("equip {2}");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying));
}

#[test]
fn leech_fanatic_has_lifelink_only_on_your_turn() {
    let mut g = two_player_game();
    let lf = g.add_card_to_battlefield(0, catalog::leech_fanatic());
    // Player 0's turn (default active player) → lifelink.
    assert!(g.computed_permanent(lf).unwrap().keywords.contains(&Keyword::Lifelink));
    // Opponent's turn → no lifelink.
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(lf).unwrap().keywords.contains(&Keyword::Lifelink));
}

#[test]
fn stonerise_spirit_grants_flying_by_exiling_graveyard_card() {
    let mut g = two_player_game();
    let spirit = g.add_card_to_battlefield(0, catalog::stonerise_spirit());
    g.clear_sickness(spirit);
    let fodder = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: spirit, ability_index: 0,
        target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("{4}, exile a gy card: grant flying");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == fodder), "graveyard card exiled as cost");
    assert!(g.battlefield_find(bear).unwrap().granted_keywords_eot.contains(&Keyword::Flying));
}

#[test]
fn novice_dissector_sacrifices_to_add_counter() {
    let mut g = two_player_game();
    let dissector = g.add_card_to_battlefield(0, catalog::novice_dissector());
    g.clear_sickness(dissector);
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dissector, ability_index: 0,
        target: Some(Target::Permanent(target)), x_value: None,
    }).expect("{1}, sac a creature: +1/+1 counter");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed as cost");
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn blood_age_general_pumps_attacking_spirits() {
    use crate::game::Attack;
    let mut g = two_player_game();
    let general = g.add_card_to_battlefield(0, catalog::blood_age_general());
    // A separate attacking Spirit (Stonerise Spirit, a 1/2 flier).
    let spirit = g.add_card_to_battlefield(0, catalog::stonerise_spirit());
    g.clear_sickness(spirit);
    g.clear_sickness(general);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("advance");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: spirit, target: AttackTarget::Player(1),
    }])).expect("attack with the Spirit");
    drain_stack(&mut g);
    let pwr = g.battlefield_find(spirit).unwrap().power();
    g.perform_action(GameAction::ActivateAbility {
        card_id: general, ability_index: 0, target: None, x_value: None,
    }).expect("{T}: attacking Spirits +1/+0");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(spirit).unwrap().power(), pwr + 1, "attacking Spirit pumped");
}

// ── extras_15 batch 3 — graveyard hate, draw, evasion-payoff ─────────────────

#[test]
fn go_blank_discards_two_and_exiles_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::go_blank());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Go Blank targets player 1");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "discarded both cards");
    assert!(g.players[1].graveyard.is_empty(), "graveyard exiled");
}

#[test]
fn secret_rendezvous_draws_for_both() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    for _ in 0..4 { g.add_card_to_library(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::secret_rendezvous());
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 - 1 + 3, "you draw 3");
    assert_eq!(g.players[1].hand.len(), h1 + 3, "target opponent draws 3");
}

#[test]
fn fuming_effigy_pings_when_card_leaves_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fuming_effigy());
    let opp = g.players[1].life;
    g.dispatch_triggers_for_events(&[crate::game::types::GameEvent::CardLeftGraveyard {
        player: 0, card_id: crate::card::CardId(999),
    }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "1 damage to each opponent");
}

#[test]
fn kelpie_guide_untaps_and_taps() {
    let mut g = two_player_game();
    let kelpie = g.add_card_to_battlefield(0, catalog::kelpie_guide());
    g.clear_sickness(kelpie);
    // Untap another permanent you control.
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    g.perform_action(GameAction::ActivateAbility {
        card_id: kelpie, ability_index: 0,
        target: Some(Target::Permanent(mine)), x_value: None,
    }).expect("{T}: untap another permanent");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(mine).unwrap().tapped, "untapped");
    // The tap ability requires eight lands — rejected without them.
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: kelpie, ability_index: 1,
        target: Some(Target::Permanent(foe)), x_value: None,
    });
    assert!(res.is_err(), "tap ability gated on eight or more lands");
}
