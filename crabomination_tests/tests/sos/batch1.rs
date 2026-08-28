#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use crate::prepared_on_battlefield;

// ── New batch: Silverquill / extra White / Black / Red / Green ──────────────

#[test]
fn ascendant_dustspeaker_etb_pumps_other_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::ascendant_dustspeaker());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Dustspeaker castable for {4}{W}");
    drain_stack(&mut g);

    let pumped = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(pumped.counter_count(CounterType::PlusOnePlusOne), 1,
        "Bear should have one +1/+1 counter from Dustspeaker ETB");
    let dust = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert!(dust.definition.keywords.contains(&Keyword::Flying));
}

#[test]
fn shattered_acolyte_sac_destroys_artifact() {
    let mut g = two_player_game();
    let acolyte = g.add_card_to_battlefield(0, catalog::shattered_acolyte());
    g.clear_sickness(acolyte);
    let mind_stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: acolyte,
        ability_index: 0,
        target: Some(Target::Permanent(mind_stone)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Shattered Acolyte sac-and-destroy castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == mind_stone),
        "Mind Stone should be destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == acolyte),
        "Shattered Acolyte should have been sacrificed");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == acolyte),
        "Acolyte should be in its owner's graveyard");
}

#[test]
fn dig_site_inventory_grants_counter_and_vigilance() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::dig_site_inventory());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Dig Site Inventory castable for {W}");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(target.counter_count(CounterType::PlusOnePlusOne), 1);
    let view = g.computed_permanent(bear).unwrap();
    assert!(view.keywords().contains(&Keyword::Vigilance));
}

#[test]
fn group_project_creates_2_2_red_white_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::group_project());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Group Project castable for {1}{W}");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 1);
    let spirit = g.battlefield.iter().find(|c| c.definition.name == "Spirit").unwrap();
    assert_eq!(spirit.definition.power, 2);
    assert_eq!(spirit.definition.toughness, 2);
}

#[test]
fn render_speechless_discards_and_pumps() {
    // Push (modern_decks): now multi-target. Slot 0 = target opponent
    // (reveal + chosen-discard); slot 1 = optional creature gets two
    // +1/+1 counters.
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::render_speechless());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("Render Speechless castable for {2}{W}{B}");
    drain_stack(&mut g);

    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "Opponent should have discarded the nonland card");
    let pumped = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(pumped.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn render_speechless_can_target_opponent_without_creature() {
    // Slot 0 (opp discard) only — no slot 1 = no counter.
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::render_speechless());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Render Speechless castable for {2}{W}{B}");
    drain_stack(&mut g);

    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"));
}

#[test]
fn snooping_page_combat_damage_draws_and_loses_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let page = g.add_card_to_battlefield(0, catalog::snooping_page());
    g.clear_sickness(page);
    let life_before = g.players[0].life;
    let opp_life_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();

    // Attack with the page; auto-resolve combat through to damage.
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: page,
        target: AttackTarget::Player(1),
    }]))
    .expect("Snooping Page can attack");
    drain_stack(&mut g);

    // Drive combat forward via the no-blockers path.
    while !matches!(g.step, TurnStep::PostCombatMain | TurnStep::End) {
        let _ = g.perform_action(GameAction::PassPriority);
    }
    drain_stack(&mut g);

    assert!(g.players[1].life < opp_life_before, "Snooping Page hit the opponent");
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Should draw 1 from combat-damage trigger");
    assert_eq!(g.players[0].life, life_before - 1,
        "Should lose 1 from combat-damage trigger");
}

#[test]
fn zealous_lorecaster_etb_returns_instant_from_graveyard() {
    let mut g = two_player_game();
    let bolt_in_grave = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt_in_grave)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable for {5}{R}");
    drain_stack(&mut g);

    // Lorecaster on bf, bolt back in hand → hand: -1 cast +1 returned = same.
    assert!(g.battlefield.iter().any(|c| c.id == id));
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_in_grave),
        "Bolt should be back in hand");
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn environmental_scientist_etb_searches_basic_to_hand() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    // "You MAY search" — accept the MayDo, then pick the Forest.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(forest)),
    ]));
    let id = g.add_card_to_hand(0, catalog::environmental_scientist());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Environmental Scientist castable for {1}{G}");
    drain_stack(&mut g);

    // Hand: -1 cast +1 forest tutored = same. Library now lacks the forest.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest),
        "Forest should be in hand after search");
}

#[test]
fn pestbrood_sloth_death_creates_two_pest_tokens() {
    let mut g = two_player_game();
    let sloth = g.add_card_to_battlefield(0, catalog::pestbrood_sloth());
    let bf_before = g.battlefield.len();

    // Kill via Murder (sorcery-speed destroy is fine for the test —
    // we just need a dies trigger).
    let murder_id = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: murder_id, target: Some(Target::Permanent(sloth)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Murder castable for {1}{B}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == sloth),
        "Pestbrood Sloth should be in the graveyard");
    let pest_count = g.battlefield.iter().filter(|c| c.definition.name == "Pest").count();
    assert_eq!(pest_count, 2, "Pestbrood Sloth should create two Pest tokens on death");
    // Net battlefield: -1 sloth + 2 pests = +1.
    assert_eq!(g.battlefield.len(), bf_before + 1);
}

#[test]
fn dinas_guidance_searches_creature_to_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bear)),
    ]));
    let id = g.add_card_to_hand(0, catalog::dinas_guidance());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Dina's Guidance castable for {1}{B}{G}");
    drain_stack(&mut g);

    // Hand: -1 cast +1 bears = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Grizzly Bears should be in hand after search");
}

#[test]
fn dinas_guidance_mode_one_sends_creature_to_graveyard() {
    // Mode 1 (search → graveyard) lands the chosen creature in the
    // controller's graveyard, enabling reanimator interactions.
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bear)),
    ]));
    let id = g.add_card_to_hand(0, catalog::dinas_guidance());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let gy_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("Dina's Guidance castable for {1}{B}{G} in mode 1");
    drain_stack(&mut g);

    // Graveyard +1 for the bear; the cast spell also lands in graveyard
    // so gy_after should be gy_before + 2 (bear + Dina's Guidance).
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear),
        "Grizzly Bears should be in graveyard after mode-1 search");
    assert!(g.players[0].graveyard.len() >= gy_before + 2,
        "Graveyard should grow by at least 2 (bear + sorcery)");
    // Bear should NOT be in hand.
    assert!(!g.players[0].hand.iter().any(|c| c.id == bear),
        "Bear should NOT be in hand for mode 1");
}

#[test]
fn pursue_the_past_loots_two_and_gains_two() {
    // Discard+draw chain is gated on `Effect::MayDo`; opt in via Bool(true).
    let mut g = two_player_game();
    // Library: two cards to draw from.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    // Hand: pursue + a swamp to discard.
    let pursue = g.add_card_to_hand(0, catalog::pursue_the_past());
    let _swamp_in_hand = g.add_card_to_hand(0, catalog::swamp());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len(); // 2
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: pursue, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pursue the Past castable for {R}{W}");
    drain_stack(&mut g);

    // Hand: -1 cast (pursue) -1 discard (swamp) +2 draw = net +0 from
    // hand_before (2 → 0 + 2 = 2).
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Graveyard: at least 1 (the spell after resolution); plus the
    // discarded swamp = at least 2 cards.
    assert!(g.players[0].graveyard.len() >= 2,
        "Graveyard should hold the resolved spell and the discarded card");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Pursue the Past"),
        "Pursue the Past should be in graveyard after resolving");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Swamp"),
        "Discarded Swamp should be in graveyard");
    // +2 life.
    assert_eq!(g.players[0].life, life_before + 2);
    // life_gained_this_turn bumped by 2.
    assert!(g.players[0].life_gained_this_turn >= 2);
}

#[test]
fn pursue_the_past_skips_loot_when_declining() {
    // Declining the may-discard: 2 life still gains, no draw/discard.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let pursue = g.add_card_to_hand(0, catalog::pursue_the_past());
    let _swamp = g.add_card_to_hand(0, catalog::swamp());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len(); // 2
    // Default: AutoDecider answers no.

    g.perform_action(GameAction::CastSpell {
        card_id: pursue, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pursue the Past castable");
    drain_stack(&mut g);

    // Hand: -1 cast (pursue) only — no discard, no draw.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
    // Graveyard: just Pursue itself (no discarded swamp).
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Pursue the Past"));
    assert!(!g.players[0].graveyard.iter().any(|c| c.definition.name == "Swamp"),
        "No discard fired");
    assert_eq!(g.players[0].life, life_before + 2,
        "Lifegain still resolves regardless of may-do choice");
}

#[test]
fn efflorescence_pumps_and_grants_trample_indestructible_after_lifegain() {
    let mut g = two_player_game();
    // Prime Infusion via Oracle's Restoration on a friendly creature.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let oracle = g.add_card_to_hand(0, catalog::oracles_restoration());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: oracle, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Oracle's Restoration castable for {G}");
    drain_stack(&mut g);

    // Cast Efflorescence on the bear.
    let id = g.add_card_to_hand(0, catalog::efflorescence());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Efflorescence castable for {2}{G}");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(target.counter_count(CounterType::PlusOnePlusOne), 2);
    let view = g.computed_permanent(bear).unwrap();
    assert!(view.keywords().contains(&Keyword::Trample),
        "Infusion should grant trample after lifegain");
    assert!(view.keywords().contains(&Keyword::Indestructible),
        "Infusion should grant indestructible after lifegain");
}

#[test]
fn efflorescence_only_pumps_without_lifegain() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::efflorescence());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Efflorescence castable for {2}{G}");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(target.counter_count(CounterType::PlusOnePlusOne), 2);
    let view = g.computed_permanent(bear).unwrap();
    assert!(!view.keywords().contains(&Keyword::Trample));
    assert!(!view.keywords().contains(&Keyword::Indestructible));
}

#[test]
fn old_growth_educator_etb_grows_after_lifegain() {
    let mut g = two_player_game();
    // First gain some life this turn to satisfy Infusion.
    let oracle = g.add_card_to_hand(0, catalog::oracles_restoration());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: oracle, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Oracle's Restoration castable for {G}");
    drain_stack(&mut g);
    assert!(g.players[0].life_gained_this_turn >= 1,
        "Life gained tracker should be primed by Oracle's Restoration");

    // Now ETB Old-Growth Educator — Infusion should add 2 +1/+1 counters.
    let id = g.add_card_to_hand(0, catalog::old_growth_educator());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Old-Growth Educator castable for {2}{B}{G}");
    drain_stack(&mut g);

    let educator = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(educator.counter_count(CounterType::PlusOnePlusOne), 2,
        "Infusion should add 2 +1/+1 counters when life was gained this turn");
    assert!(educator.definition.keywords.contains(&Keyword::Reach));
    assert!(educator.definition.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn old_growth_educator_etb_no_counters_without_lifegain() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::old_growth_educator());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Old-Growth Educator castable for {2}{B}{G}");
    drain_stack(&mut g);

    let educator = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(educator.counter_count(CounterType::PlusOnePlusOne), 0,
        "Infusion should be inactive when no life has been gained this turn");
}

#[test]
fn foolish_fate_drains_three_after_lifegain() {
    let mut g = two_player_game();
    // Step 1: gain life to prime Infusion.
    let oracle = g.add_card_to_hand(0, catalog::oracles_restoration());
    let bear_self = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: oracle, target: Some(Target::Permanent(bear_self)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Oracle's Restoration castable for {G}");
    drain_stack(&mut g);

    // Step 2: cast Foolish Fate on opponent's bear.
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::foolish_fate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Foolish Fate castable for {2}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Opponent's bear should be destroyed");
    assert_eq!(g.players[1].life, opp_life_before - 3,
        "Infusion should drain 3 life from the target's controller");
}

#[test]
fn foolish_fate_no_drain_without_lifegain() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::foolish_fate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Foolish Fate castable for {2}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Opponent's bear should be destroyed");
    assert_eq!(g.players[1].life, opp_life_before,
        "Without prior lifegain, no drain should occur");
}

#[test]
fn teachers_pest_attacks_gains_one_life() {
    let mut g = two_player_game();
    let pest = g.add_card_to_battlefield(0, catalog::teachers_pest());
    g.clear_sickness(pest);
    let life_before = g.players[0].life;

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: pest,
        target: AttackTarget::Player(1),
    }]))
    .expect("Teacher's Pest can attack");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 1,
        "Teacher's Pest should grant 1 life on attack");
    let view = g.computed_permanent(pest).unwrap();
    assert!(view.keywords().contains(&Keyword::Menace),
        "Teacher's Pest should have menace");
}

// ── Owlin Historian ─────────────────────────────────────────────────────────

#[test]
fn owlin_historian_etb_surveils_one_and_has_flying() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::owlin_historian());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Owlin Historian castable for {2}{W}");
    drain_stack(&mut g);

    let view = g.computed_permanent(id).expect("Historian on battlefield");
    assert!(view.keywords().contains(&Keyword::Flying),
        "Owlin Historian should have flying");
    assert!(g.players[0].library.len() <= lib_before,
        "Surveil 1 should not grow the library");
}

// ── Inkling Mascot ──────────────────────────────────────────────────────────

// ── Cost of Brilliance ──────────────────────────────────────────────────────

#[test]
fn cost_of_brilliance_draws_two_loses_two_pumps_creature() {
    // Push (modern_decks): Cost of Brilliance is now multi-target —
    // slot 0 = target player draws 2 + loses 2 life, slot 1 = optional
    // creature target gets +1/+1 counter. Caster aims slot 0 at self.
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cost_of_brilliance());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(0)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("Cost of Brilliance castable for {2}{B}");
    drain_stack(&mut g);

    // Hand: -1 cast +2 draw = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Should net +1 hand from drawing 2 minus the cast itself");
    assert_eq!(g.players[0].life, life_before - 2,
        "Should lose 2 life");
    let pumped = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(pumped.counter_count(CounterType::PlusOnePlusOne), 1,
        "Bear should have 1 +1/+1 counter");
}

#[test]
fn cost_of_brilliance_can_target_opponent_for_draw() {
    // The slot 0 draw target can be aimed at an opponent — they draw 2
    // and lose 2 life. The +1/+1 counter half (slot 1) is optional and
    // can be skipped.
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::cost_of_brilliance());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_hand_before = g.players[1].hand.len();
    let opp_life_before = g.players[1].life;
    let caster_hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Cost of Brilliance castable aimed at opp");
    drain_stack(&mut g);

    // Caster hand: -1 cast = -1 net (no draw on caster's side).
    assert_eq!(g.players[0].hand.len(), caster_hand_before - 1);
    // Opp hand: +2 draw.
    assert_eq!(g.players[1].hand.len(), opp_hand_before + 2);
    // Opp life: -2.
    assert_eq!(g.players[1].life, opp_life_before - 2);
}

// ── Mind Roots ──────────────────────────────────────────────────────────────

#[test]
fn mind_roots_makes_opponent_discard_two() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mind_roots());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Mind Roots castable for {1}{B}{G}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].hand.len(), opp_hand_before - 2,
        "Opponent should have discarded 2 cards");
}

#[test]
fn mind_roots_steals_a_discarded_land_to_caster_battlefield() {
    // Push (modern_decks): the "Put up to one land card discarded this way
    // onto the battlefield tapped under your control" rider now wires
    // via `Selector::DiscardedThisResolution` + `Selector::Take(1)`.
    // Seed opp hand with one land + two non-land cards; cast Mind Roots,
    // both are discarded; the land should land on the caster's
    // battlefield tapped.
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let opp_island = g.add_card_to_hand(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::mind_roots());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Mind Roots castable for {1}{B}{G}");
    drain_stack(&mut g);

    // The discarded Island should now be on the caster's battlefield, tapped.
    let stolen = g.battlefield_find(opp_island)
        .expect("opp's Island should be on the battlefield after Mind Roots resolves");
    assert_eq!(stolen.controller, 0,
        "Mind Roots steals the discarded land to the caster's side");
    assert!(stolen.tapped, "Stolen land should be tapped");
    assert_eq!(g.battlefield.len(), bf_before + 1);
}

#[test]
fn mind_roots_does_not_steal_a_nonland_discarded_card() {
    // No land in opp's hand → no land discarded → nothing moves to bf.
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mind_roots());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Mind Roots castable for {1}{B}{G}");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before,
        "No lands discarded → no land-grab — battlefield should be unchanged");
}

// ── Stadium Tidalmage ───────────────────────────────────────────────────────

#[test]
fn stadium_tidalmage_etb_loots_once() {
    // Loot trigger is `Effect::MayDo` — inject `Bool(true)` to exercise
    // the opted-in path.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::stadium_tidalmage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Yes to the may-loot.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Stadium Tidalmage castable for {2}{U}{R}");
    drain_stack(&mut g);

    // After casting: -1 (Tidalmage cast). ETB: draw 1, discard 1.
    // Net hand should be: hand_before(2) - 1 (cast) + 1 (draw) - 1 (discard) = 1.
    assert_eq!(g.players[0].hand.len(), 1, "Net hand size after cast+ETB loot");
    // Discarded card should be in graveyard.
    assert!(!g.players[0].graveyard.is_empty(),
        "Looting should put a card in the graveyard");
}

#[test]
fn stadium_tidalmage_etb_skips_loot_when_declining() {
    // AutoDecider defaults to Bool(false) → ETB loot is skipped.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::stadium_tidalmage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Default AutoDecider says no.

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Stadium Tidalmage castable for {2}{U}{R}");
    drain_stack(&mut g);

    // Hand: started with [Bolt + Tidalmage] = 2, cast Tidalmage → 1 left.
    // No loot fired, so still 1 (unchanged from before declining).
    assert_eq!(g.players[0].hand.len(), 1, "No loot should fire");
    assert!(g.players[0].graveyard.is_empty(),
        "No loot → graveyard stays empty");
}

// ── Pterafractyl ────────────────────────────────────────────────────────────

#[test]
fn pterafractyl_etb_with_x_counters_and_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pterafractyl());
    // Pay {2} for X=2 plus {G}{U}.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("Pterafractyl castable for X=2 {G}{U}");
    drain_stack(&mut g);

    let view = g.computed_permanent(id).expect("Pterafractyl on battlefield");
    assert!(view.keywords().contains(&Keyword::Flying));
    let inst = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(inst.counter_count(CounterType::PlusOnePlusOne), 2,
        "Pterafractyl enters with X=2 +1/+1 counters");
    assert_eq!(g.players[0].life, life_before + 2,
        "Pterafractyl should gain 2 life on ETB");
}

// CR 614.12 — "enters with N counters" replacement now lands BEFORE the
// first state-based-action sweep on the new permanent, so a printed
// 1/0 body (Pterafractyl) survives ETB when X≥1. Verifies the printed
// P/T (X+1)/X exactly at X=1.
#[test]
fn pterafractyl_cr_614_12_zero_toughness_base_survives_etb_via_enters_with() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pterafractyl());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(1),
    })
    .expect("Pterafractyl castable for X=1 {G}{U}");
    drain_stack(&mut g);

    let view = g.computed_permanent(id)
        .expect("Pterafractyl should survive ETB — counters applied before SBA");
    // Printed 1/0 + 1 +1/+1 counter = 2/1 exact (X=1 path).
    assert_eq!(view.power, 2, "X=1: 1/0 + 1 +1/+1 = 2/1");
    assert_eq!(view.toughness, 1);
}

// ── Fractal Mascot ──────────────────────────────────────────────────────────

#[test]
fn fractal_mascot_etb_taps_and_stuns_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_mascot());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Fractal Mascot castable for {4}{G}{U}");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(target.tapped, "Bear should be tapped");
    assert_eq!(target.counter_count(CounterType::Stun), 1,
        "Bear should have 1 stun counter");
    let mascot = g.computed_permanent(id).unwrap();
    assert!(mascot.keywords().contains(&Keyword::Trample));
}

// ── Mind into Matter ────────────────────────────────────────────────────────

#[test]
fn mind_into_matter_draws_x_cards() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::mind_into_matter());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    })
    .expect("Mind into Matter castable for X=3 {G}{U}");
    drain_stack(&mut g);

    // -1 (cast) +3 (draw X=3) -1 (AutoDecider now TAKES the optional
    // "put a permanent onto the battlefield" — declining a free deploy
    // was the old blanket-no bug) = +1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3 - 1);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Island"),
        "the drawn permanent was deployed",
    );
}

#[test]
fn mind_into_matter_optional_permanent_lands_with_scripted_yes() {
    // The "may put a permanent ≤ X from hand onto the battlefield
    // tapped" half rides `Effect::PutFromHandOntoBattlefield` — the
    // controller picks the card via a ChooseCards decision (min 0, so
    // the auto-decider declines). Scripted card pick exercises the
    // paid path.
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mind_into_matter());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    // ScriptedDecider picks the Bear at the ChooseCards prompt.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    })
    .expect("Mind into Matter castable for X=3");
    drain_stack(&mut g);

    // Bear (MV 2 ≤ 3) should be on battlefield, tapped.
    let bear_on_bf = g.battlefield.iter().find(|c| c.id == bear);
    assert!(bear_on_bf.is_some(), "Bear should be on battlefield");
    assert!(bear_on_bf.unwrap().tapped, "Bear should enter tapped");
}

// ── Growth Curve ────────────────────────────────────────────────────────────

#[test]
fn growth_curve_doubles_existing_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Pre-load 2 +1/+1 counters.
    {
        let inst = g.battlefield.iter_mut().find(|c| c.id == bear).unwrap();
        inst.add_counters(CounterType::PlusOnePlusOne, 2);
    }
    let id = g.add_card_to_hand(0, catalog::growth_curve());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Growth Curve castable for {G}{U}");
    drain_stack(&mut g);

    // Pre: 2. +1: 3. Double: 3 + 3 = 6.
    let inst = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(inst.counter_count(CounterType::PlusOnePlusOne), 6,
        "Growth Curve should leave 2*(N+1) counters: starting with 2, ends at 6");
}

#[test]
fn growth_curve_on_creature_with_no_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::growth_curve());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Growth Curve castable for {G}{U}");
    drain_stack(&mut g);

    // Pre: 0. +1: 1. Double: 1 + 1 = 2.
    let inst = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(inst.counter_count(CounterType::PlusOnePlusOne), 2,
        "0 → +1 → double = 2");
}

// ── Quandrix Charm ──────────────────────────────────────────────────────────

#[test]
fn quandrix_charm_mode_2_makes_creature_5_5() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_charm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: Some(2), x_value: None,
    })
    .expect("Quandrix Charm castable for {G}{U}");
    drain_stack(&mut g);

    let view = g.computed_permanent(bear).unwrap();
    assert_eq!(view.power, 5, "Bear should be 5/5 base after mode 2");
    assert_eq!(view.toughness, 5);
}

#[test]
fn quandrix_charm_mode_1_destroys_enchantment() {
    use crabomination::card::CardDefinition;
    
    // Build a vanilla enchantment.
    let ench_def = CardDefinition {
        name: "Test Enchant",
        cost: crabomination::mana::ManaCost::default(),
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    };
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(1, ench_def);
    let id = g.add_card_to_hand(0, catalog::quandrix_charm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(ench)), additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("Quandrix Charm castable for {G}{U} with enchantment target");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == ench),
        "Mode 1 should destroy the enchantment");
}

// ── Vibrant Outburst ────────────────────────────────────────────────────────

#[test]
fn vibrant_outburst_deals_three_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vibrant_outburst());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Vibrant Outburst castable for {U}{R}");
    drain_stack(&mut g);

    // 2/2 bear takes 3 damage and dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should die to 3 damage");
}

/// `auto_targets_for_effect_all_slots` should pick targets for both
/// slots of Vibrant Outburst without manual specification, so a bot
/// drives the multi-target shape end-to-end.
#[test]
fn auto_target_picker_fills_multi_slot_vibrant_outburst() {
    let mut g = two_player_game();
    let _bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vibrant_outburst());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    // Find the spell card definition and ask the picker for all slots.
    let card_def = &g.players[0]
        .hand
        .iter()
        .find(|c| c.id == id)
        .unwrap()
        .definition;
    let eff = &card_def.effect;
    let (slot_0, additional) = g.auto_targets_for_effect_all_slots(eff, 0, None);
    assert!(slot_0.is_some(), "Slot 0 must be picked");
    assert!(
        !additional.is_empty(),
        "Slot 1 (optional creature tap target) must also be picked"
    );
}

#[test]
fn vibrant_outburst_taps_optional_second_target() {
    // Push (modern_decks): slot 1 = optional creature target tap. Two
    // creatures: slot 0 = bear1 (3 dmg, dies); slot 1 = bear2 (taps).
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vibrant_outburst());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear1)),
        additional_targets: vec![Target::Permanent(bear2)],
        mode: None,
        x_value: None,
    })
    .expect("Vibrant Outburst castable");
    drain_stack(&mut g);

    // bear1 dies; bear2 stays but is tapped.
    assert!(!g.battlefield.iter().any(|c| c.id == bear1));
    let bear2_card = g.battlefield.iter().find(|c| c.id == bear2).expect("bear2 alive");
    assert!(bear2_card.tapped, "bear2 should be tapped");
}

// ── Stress Dream ────────────────────────────────────────────────────────────

#[test]
fn stress_dream_kills_creature_and_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::stress_dream());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Stress Dream castable for {3}{U}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should die to 5 damage");
    // Hand: -1 cast +1 draw = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn stress_dream_scrys_two_before_drawing() {
    // Promoted in modern_decks batch 43: the "look at top 2, choose 1
    // to hand, other to bottom" half is now Scry 2 → Draw 1 (was Scry 1
    // → Draw 1). The Scry 2 step lets the auto-decider see both top
    // cards before drawing one.
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let lib_before = g.players[0].library.len();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::stress_dream());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Stress Dream castable for {3}{U}{R}");
    drain_stack(&mut g);
    // Library: -1 from the draw step. Scry 2 reorders but doesn't
    // change library size.
    assert_eq!(g.players[0].library.len(), lib_before - 1);
}

// ── Arcane Omens ────────────────────────────────────────────────────────────

#[test]
fn arcane_omens_discards_x_cards_using_converged_value() {
    // Pay {4}{B} but use a non-monocolor combination — we'll prep
    // the opponent's hand and validate the discard count.
    let mut g = two_player_game();
    for n in [
        catalog::lightning_bolt(),
        catalog::island(),
        catalog::grizzly_bears(),
    ] {
        g.add_card_to_hand(1, n);
    }

    let id = g.add_card_to_hand(0, catalog::arcane_omens());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    let opp_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Arcane Omens castable for {4}{B}");
    drain_stack(&mut g);

    // Mono-black cast → ConvergedValue = 1 → targeted opponent discards 1.
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1);
}

// ── Together as One ─────────────────────────────────────────────────────────

#[test]
fn together_as_one_uses_converged_value_for_each_clause() {
    // Push (modern_decks): now multi-target — slot 0 = target player
    // for the draw, slot 1 = any target for the damage. The
    // ConvergedValue = 0 (mono-colorless cast) zeros all three clauses.
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::together_as_one());
    g.players[0].mana_pool.add_colorless(6);
    let opp_life_before = g.players[1].life;
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(0)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    })
    .expect("Together as One castable for {6}");
    drain_stack(&mut g);

    // ConvergedValue = 0, so no draw, no damage, no life gain.
    assert_eq!(g.players[1].life, opp_life_before);
    assert_eq!(g.players[0].life, life_before);
    // Hand: -1 cast + 0 draw = hand_before - 1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn together_as_one_three_color_cast_deals_three_to_each_clause() {
    // With 3 distinct colors spent, ConvergedValue = 3: opp draws 3,
    // any-target takes 3 damage, you gain 3 life.
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::together_as_one());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let opp_life_before = g.players[1].life;
    let life_before = g.players[0].life;
    let opp_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    })
    .expect("Together as One castable for {6}");
    drain_stack(&mut g);

    // Opp drew 3 cards and took 3 damage (-3 life).
    assert_eq!(g.players[1].hand.len(), opp_hand_before + 3);
    assert_eq!(g.players[1].life, opp_life_before - 3);
    // You gain 3 life.
    assert_eq!(g.players[0].life, life_before + 3);
}

// ── Rancorous Archaic ───────────────────────────────────────────────────────

#[test]
fn archaics_agony_deals_converge_damage_to_target_creature() {
    // Pay {4}{R}: only Red counts as a distinct color among colored
    // pips paid → converge value should be 1.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::archaics_agony());
    // Pay {4}{R}: 4 colorless + 1 red = converge value 1 (only Red).
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Archaic's Agony castable for {4}{R}");
    drain_stack(&mut g);

    // Converge=1 (only Red contributed). Bear has 2 toughness, takes 1
    // damage — should still be alive.
    let bear_card = g.battlefield.iter().find(|c| c.id == bear)
        .expect("Bear survives 1 damage");
    assert_eq!(bear_card.damage, 1,
        "Bear should have 1 damage marker after Converge=1 Archaic's Agony");
}

#[test]
fn rancorous_archaic_etb_with_converge_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::rancorous_archaic());
    // {5} cast — pay all-colorless, ConvergedValue = 0.
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Rancorous Archaic castable for {5}");
    drain_stack(&mut g);

    let view = g.computed_permanent(id).unwrap();
    assert!(view.keywords().contains(&Keyword::Trample));
    assert!(view.keywords().contains(&Keyword::Reach));
    let inst = g.battlefield.iter().find(|c| c.id == id).unwrap();
    // ConvergedValue=0 → 0 counters → 2/2 base body.
    assert_eq!(inst.counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(view.power, 2);
    assert_eq!(view.toughness, 2);
}

/// The converge payment repair, pool half: a converge cast's generic
/// portion drains one-per-distinct-color before going deep. {5} out of
/// {W W W U B G} used to go W,W,W,U,B (converge 3) under the
/// WUBRG-greedy default; a converge spell now takes W,U,B,G then the
/// leftover W — converge 4.
#[test]
fn rancorous_archaic_converge_payment_spreads_pool_colors() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::rancorous_archaic());
    g.players[0].mana_pool.add(Color::White, 3);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Rancorous Archaic castable for {5}");
    drain_stack(&mut g);

    let inst = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(
        inst.counter_count(CounterType::PlusOnePlusOne),
        4,
        "converge drain spreads across W/U/B/G instead of eating the Plains"
    );
}

/// The converge payment repair, auto-tap half: with nothing floating, a
/// converge cast taps toward distinct colors. Three Plains + Island +
/// Swamp + Forest used to tap the first five in battlefield order
/// (converge 3); the repaired tapper takes four distinct basics before
/// doubling up — converge 4.
#[test]
fn rancorous_archaic_auto_tap_prefers_distinct_colors() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::plains());
    }
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::rancorous_archaic());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("auto-tap covers {5} from six lands");
    drain_stack(&mut g);

    let inst = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(
        inst.counter_count(CounterType::PlusOnePlusOne),
        4,
        "auto-tap reaches for W/U/B/G before the second Plains"
    );
}

// ── Wisdom of Ages ──────────────────────────────────────────────────────────

#[test]
fn wisdom_of_ages_returns_all_instants_and_sorceries_from_graveyard() {
    let mut g = two_player_game();
    // Stock graveyard with bolt (instant), island (land), grizzly bears
    // (creature), wrath (sorcery).
    let bolt = g.add_card_to_battlefield(0, catalog::lightning_bolt());
    let isl = g.add_card_to_battlefield(0, catalog::island());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wrath = g.add_card_to_battlefield(0, catalog::wrath_of_god());
    for cid in [bolt, isl, bears, wrath] {
        let idx = g.battlefield.iter().position(|c| c.id == cid).unwrap();
        let card = g.battlefield.remove(idx);
        g.players[0].graveyard.push(card);
    }

    let id = g.add_card_to_hand(0, catalog::wisdom_of_ages());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Wisdom of Ages castable for {4}{U}{U}{U}");
    drain_stack(&mut g);

    // Bolt and Wrath should be back in hand; Island and Grizzly Bears stay in graveyard.
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
    assert!(g.players[0].hand.iter().any(|c| c.id == wrath));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == isl));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bears));
    // Push (modern_decks): the new `Effect::SetNoMaxHandSize` clause
    // clears `Player.max_hand_size` so the cleanup-step CR 514.1
    // enforcement is skipped for the rest of the game.
    assert_eq!(g.players[0].max_hand_size, None,
        "Wisdom of Ages removes the maximum hand size on the caster");
}

#[test]
fn wisdom_of_ages_lets_caster_keep_more_than_seven_cards() {
    // Functional test: cast Wisdom of Ages so the flag flips, then push
    // 10 cards into hand and trigger cleanup — none should be discarded.
    use crabomination::game::TurnStep;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::wisdom_of_ages());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Wisdom of Ages castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].max_hand_size, None);

    // Pile up a 10-card hand and run the cleanup step.
    for _ in 0..10 {
        g.add_card_to_hand(0, catalog::island());
    }
    let hand_before = g.players[0].hand.len();
    g.step = TurnStep::Cleanup;
    g.do_cleanup(&mut Vec::new());
    // No discards — hand size is unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "no cards discarded under the no-max-hand-size flag");
}

#[test]
fn wisdom_of_ages_exiles_itself_after_resolve_via_exile_on_resolve_flag() {
    // Push (modern_decks): the printed "Exile Wisdom of Ages" rider
    // now lands via the new `CardDefinition.exile_on_resolve` flag —
    // the resolved sorcery goes to exile, not graveyard, so it can't
    // be flashbacked/Past-in-Flames-looped.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::wisdom_of_ages());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(4);
    let exile_before = g.exile.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Wisdom of Ages castable for {4}{U}{U}{U}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == id),
        "Wisdom of Ages should land in exile after resolve");
    assert_eq!(g.exile.len(), exile_before + 1,
        "Exile zone gained one card");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == id),
        "Wisdom of Ages should NOT be in graveyard");
}

// ── Rapturous Moment ────────────────────────────────────────────────────────

#[test]
fn rapturous_moment_loots_and_adds_mana() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::rapturous_moment());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Rapturous Moment castable for {4}{U}{R}");
    drain_stack(&mut g);

    // Hand: -1 cast +3 draw -2 discard = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Mana pool should have at least 2U and 3R post-resolution.
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 2);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3);
}

// ── Splatter Technique ──────────────────────────────────────────────────────

#[test]
fn splatter_technique_mode_0_draws_four_mode_1_wipes_creatures() {
    // Mode 0: draw 4.
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::splatter_technique());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Splatter Technique castable in mode 0");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 4);

    // Mode 1: deal 4 to each creature.
    let mut g = two_player_game();
    let bear0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::splatter_technique());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Splatter Technique castable in mode 1");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear0));
    assert!(!g.battlefield.iter().any(|c| c.id == bear1));
}

// ── Arnyn, Deathbloom Botanist ──────────────────────────────────────────────

#[test]
fn arnyn_drains_when_a_one_power_creature_you_control_dies() {
    let mut g = two_player_game();
    let arnyn = g.add_card_to_battlefield(0, catalog::arnyn_deathbloom_botanist());
    let view = g.computed_permanent(arnyn).unwrap();
    assert!(view.keywords().contains(&Keyword::Deathtouch));

    // Place a 1/1 creature you control (mana value goes through bears →
    // we simulate a 1/1 by using a token-style creature definition).
    use crabomination::card::CardDefinition;
    let weak = CardDefinition {
        name: "Weak Creature",
        cost: crabomination::mana::ManaCost::default(),
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        effect: crabomination::effect::Effect::Noop,
        ..Default::default()
    };
    let weak_id = g.add_card_to_battlefield(0, weak);

    let life_before = g.players[0].life;
    let opp_life_before = g.players[1].life;

    // Kill via Murder so the dies-trigger pipeline runs end-to-end.
    let murder_id = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: murder_id, target: Some(Target::Permanent(weak_id)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Murder castable for {1}{B}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == weak_id),
        "Weak creature should be in the graveyard");
    assert_eq!(g.players[1].life, opp_life_before - 2,
        "Arnyn drains opponent for 2");
    assert_eq!(g.players[0].life, life_before + 2,
        "Arnyn gains 2 life");
}

// ── Startled Relic Sloth ────────────────────────────────────────────────────

#[test]
fn startled_relic_sloth_combat_step_exiles_graveyard_card() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Move bear to graveyard.
    let card = g.battlefield.iter().position(|c| c.id == bear).unwrap();
    let bear_card = g.battlefield.remove(card);
    g.players[0].graveyard.push(bear_card);

    let sloth = g.add_card_to_battlefield(0, catalog::startled_relic_sloth());
    let view = g.computed_permanent(sloth).unwrap();
    assert!(view.keywords().contains(&Keyword::Trample));
    assert!(view.keywords().contains(&Keyword::Lifelink));

    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);

    assert!(
        g.exile.iter().any(|c| c.id == bear),
        "Begin-combat trigger should exile a graveyard card"
    );
}

// ── Hardened Academic ───────────────────────────────────────────────────────

#[test]
fn hardened_academic_discard_grants_lifelink_eot() {
    let mut g = two_player_game();
    let academic = g.add_card_to_battlefield(0, catalog::hardened_academic());
    g.add_card_to_hand(0, catalog::island());

    let view = g.computed_permanent(academic).unwrap();
    assert!(view.keywords().contains(&Keyword::Flying));
    assert!(view.keywords().contains(&Keyword::Haste));
    assert!(!view.keywords().contains(&Keyword::Lifelink));

    g.perform_action(GameAction::ActivateAbility {
        card_id: academic,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Discard ability should activate");
    drain_stack(&mut g);

    let view = g.computed_permanent(academic).unwrap();
    assert!(view.keywords().contains(&Keyword::Lifelink),
        "Hardened Academic should have lifelink until EOT after discard activation");
}

// ── Slumbering Trudge ───────────────────────────────────────────────────────

#[test]
fn slumbering_trudge_x_zero_enters_with_three_stun_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::slumbering_trudge());
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(0),
    })
    .expect("Slumbering Trudge castable for X=0");
    drain_stack(&mut g);

    let inst = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(inst.counter_count(CounterType::Stun), 3,
        "X=0 → 3 stun counters via NonNeg(3-X)");
    assert!(inst.tapped, "Slumbering Trudge enters tapped");
}

#[test]
fn slumbering_trudge_x_three_enters_without_stun_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::slumbering_trudge());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3); // X=3

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("Slumbering Trudge castable for X=3");
    drain_stack(&mut g);

    let inst = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(inst.counter_count(CounterType::Stun), 0,
        "X=3 → 0 stun counters (NonNeg(3-3))");
    assert!(!inst.tapped,
        "X=3 (> 2) — the enters-tapped replacement does not apply, Trudge is untapped");
}

// ── Fractal Anomaly ─────────────────────────────────────────────────────────

#[test]
fn fractal_anomaly_tokens_grows_with_cards_drawn() {
    let mut g = two_player_game();
    // Draw two cards "this turn" by setting the counter directly.
    g.players[0].cards_drawn_this_turn = 3;

    let id = g.add_card_to_hand(0, catalog::fractal_anomaly());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Fractal Anomaly castable for {U}");
    drain_stack(&mut g);

    // A new Fractal token should be on the battlefield.
    let token = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Fractal")
        .expect("Fractal token should be on the battlefield");
    assert_eq!(g.battlefield.len(), bf_before + 1);
    assert_eq!(token.counter_count(CounterType::PlusOnePlusOne), 3,
        "Fractal token should have 3 +1/+1 counters (one per card drawn this turn)");
    let view = g.computed_permanent(token.id).unwrap();
    assert_eq!(view.power, 3);
    assert_eq!(view.toughness, 3);
}

#[test]
fn fractal_anomaly_after_draw_effect_uses_live_counter() {
    // Validates that `Player.cards_drawn_this_turn` is incremented on
    // `Effect::Draw`, and that `Value::CardsDrawnThisTurn` reads the
    // live counter at resolution time.
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    // Hand-cast a `Draw 2` spell first so cards_drawn_this_turn
    // increments from 0 to 2.
    let bolt_then_draw = catalog::concentrate(); // {2}{U}{U} draw 3
    let _ = g.add_card_to_hand(0, bolt_then_draw);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    let conc = g.players[0].hand.last().unwrap().id;
    g.perform_action(GameAction::CastSpell {
        card_id: conc, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Concentrate castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].cards_drawn_this_turn, 3,
        "Concentrate should bump cards_drawn_this_turn to 3");

    // Now cast Fractal Anomaly — the token should enter with 3 counters.
    let id = g.add_card_to_hand(0, catalog::fractal_anomaly());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Fractal Anomaly castable for {U}");
    drain_stack(&mut g);

    let token = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Fractal")
        .expect("Fractal token should be on the battlefield");
    assert_eq!(token.counter_count(CounterType::PlusOnePlusOne), 3);
}

#[test]
fn fractal_anomaly_zero_cards_drawn_dies_to_sba() {
    let mut g = two_player_game();
    // 0 cards drawn this turn — the printed "0/0 with 0 counters" dies.
    g.players[0].cards_drawn_this_turn = 0;
    let id = g.add_card_to_hand(0, catalog::fractal_anomaly());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Fractal Anomaly castable for {U}");
    drain_stack(&mut g);

    // The Fractal token should have died to SBA — no token on battlefield.
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Fractal"),
        "0/0 Fractal token with 0 counters should die to SBA");
}

// ── Tenured Concocter ───────────────────────────────────────────────────────

#[test]
fn tenured_concocter_infusion_pumps_self_when_life_gained() {
    // Push (modern_decks): the Infusion "+2/+0 as long as you gained life
    // this turn" self-pump is now wired via the `lifegain_selfpump_for_name`
    // helper table (same pattern as Honor Troll / Ulna Alley Shopkeep).
    let mut g = two_player_game();
    let conc = g.add_card_to_battlefield(0, catalog::tenured_concocter());
    g.clear_sickness(conc);

    // No lifegain: stays at base 4/5.
    let base = g.computed_permanent(conc).unwrap();
    assert_eq!(base.power, 4);
    assert_eq!(base.toughness, 5);

    // With lifegain: 6/5.
    g.players[0].life_gained_this_turn = 3;
    let pumped = g.computed_permanent(conc).unwrap();
    assert_eq!(pumped.power, 6, "Tenured Concocter Infusion: +2/+0 when life gained");
    assert_eq!(pumped.toughness, 5);
}

#[test]
fn tenured_concocter_draws_when_opp_targets_it_with_scripted_yes() {
    // Opp casts Lightning Bolt targeting P0's Tenured Concocter. The
    // BecameTarget trigger fires with caster=P1 (opponent). ScriptedDecider
    // says yes → P0 draws a card.
    let mut g = two_player_game();
    let conc = g.add_card_to_battlefield(0, catalog::tenured_concocter());
    g.clear_sickness(conc);
    // Seed P0's library so the draw has something to pull.
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    // P1 has priority by default (turn 0 has P0 active but we'll just
    // give P1 priority for the cast).
    g.priority.player_with_priority = 1;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(conc)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    assert_eq!(
        g.players[0].hand.len(),
        hand_before + 1,
        "P0 should draw 1 card after Concocter is targeted by opp's Bolt"
    );
}

#[test]
fn tenured_concocter_does_not_trigger_when_owner_self_targets() {
    // P0 casts Lightning Bolt targeting their own Tenured Concocter
    // (an unusual but legal play). The trigger should NOT fire because
    // the caster is not an opponent. Hand-before contains the Bolt;
    // hand-after = hand-before - 1 (Bolt cast) if no draw.
    let mut g = two_player_game();
    let conc = g.add_card_to_battlefield(0, catalog::tenured_concocter());
    g.clear_sickness(conc);
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Scripted yes — but the trigger shouldn't even ask.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(conc)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    // No draw — hand lost just the Bolt (cast).
    assert_eq!(
        g.players[0].hand.len(),
        hand_before - 1,
        "P0 should NOT draw — Concocter targeted by its own controller"
    );
}

#[test]
fn tenured_concocter_does_not_draw_with_auto_decider_no_default() {
    // AutoDecider's MayDo default is false (decline). Verify the
    // trigger fires but the draw is declined when no scripted answer
    // is provided.
    let mut g = two_player_game();
    let conc = g.add_card_to_battlefield(0, catalog::tenured_concocter());
    g.clear_sickness(conc);
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(conc)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    // AutoDecider declines MayDo — no draw.
    assert_eq!(
        g.players[0].hand.len(),
        hand_before,
        "AutoDecider declines the may-draw; hand unchanged"
    );
}

#[test]
fn tenured_concocter_does_not_trigger_when_opp_targets_other_permanent() {
    // Opp targets a different permanent (their own creature or P0's
    // bear) — the BecameTarget event fires for the OTHER permanent,
    // not Concocter. Concocter's trigger checks target == source.id
    // so it should NOT fire here.
    let mut g = two_player_game();
    let conc = g.add_card_to_battlefield(0, catalog::tenured_concocter());
    g.clear_sickness(conc);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand_before = g.players[0].hand.len();

    // Bolt targets P0's bear, not the Concocter.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    // Concocter's BecameTarget trigger checks target == source.id;
    // since opp's Bolt targeted the bear (not Concocter), no trigger.
    assert_eq!(
        g.players[0].hand.len(),
        hand_before,
        "Concocter shouldn't trigger when opp targets a different permanent"
    );
}

#[test]
fn traumatic_critique_x_damage_loots() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::traumatic_critique());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4); // X=4
    let opp_life_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: Some(4),
    })
    .expect("Traumatic Critique castable for X=4 {U}{R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, opp_life_before - 4,
        "Should deal 4 damage with X=4");
    // Hand: -1 cast +2 draw -1 discard = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}


/// The recorded 2026-08-19 soft-lock: a manual-mana player activates
/// Forum of Amity's surveil ({2}{W}{B}, {T}) with an off-color {G}
/// The recorded 2026-08-20 export, "stuck paying for fields of strife": both
/// copies tapped, `{W}{W}` floating, and a float-spend prompt that would not
/// go away.
///
/// The CR 601.2g prompt is posed forty lines before the `{T}` / `{Q}` gate, so
/// an *already tapped* source got the modal anyway. Answering it replayed the
/// activation, which died on `CardIsTapped`; the client re-armed and
/// re-submitted, which re-posed the same modal. Nothing the player could do
/// broke the cycle — the sibling test below covers the other half of that
/// export (the tap surviving a failed payment).
///
/// The prompt is now gated on the self-tap cost actually being payable, so a
/// tapped source is refused up front with the honest error.
#[test]
fn fields_of_strife_surveil_on_a_tapped_source_refuses_instead_of_prompting() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    g.players[0].manual_mana = true;
    let strife = g.add_card_to_battlefield(0, catalog::fields_of_strife());
    // Untapped sources so the float-spend question is live, plus the excess
    // white float that makes it "optional" (one {W} pays the pip, the other
    // would be swept into the generic).
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::plains());
    }
    g.add_card_to_battlefield(0, catalog::mountain());
    for c in g.battlefield.iter_mut() {
        c.tapped = false;
    }
    g.players[0].mana_pool.add(Color::White, 2);

    let surveil = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: strife,
            ability_index: 2, // {2}{R}{W}, {T}: Surveil 1
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
            mode: None,
        })
    };

    // Healthy board: the prompt is a real question and is posed.
    surveil(&mut g).expect("the activation suspends for the float-spend choice");
    assert!(
        matches!(
            g.pending_decision.as_ref().map(|p| &p.decision),
            Some(crabomination::decision::Decision::OptionalTrigger { .. }),
        ),
        "an untapped source with spendable float still asks",
    );
    // (Answering it either completes the activation or bounces for manual
    // tapping — either is fine here; what matters is that it was asked.)
    let _ = g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(true)));

    // Same board, but the land is already tapped — it paid for mana, or an
    // earlier attempt consumed it. The activation can never pay its {T}, so
    // there is nothing to ask about: refuse with the real error.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    g.players[0].manual_mana = true;
    let strife = g.add_card_to_battlefield(0, catalog::fields_of_strife());
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::plains());
    }
    g.add_card_to_battlefield(0, catalog::mountain());
    for c in g.battlefield.iter_mut() {
        c.tapped = false;
    }
    g.players[0].mana_pool.add(Color::White, 2);
    g.battlefield.iter_mut().find(|c| c.id == strife).unwrap().tapped = true;

    assert!(
        matches!(surveil(&mut g), Err(GameError::CardIsTapped(id)) if id == strife),
        "a tapped source is refused up front, not asked about",
    );
    assert!(
        g.pending_decision.is_none(),
        "and no unanswerable prompt is left behind",
    );
    assert_eq!(
        g.players[0].mana_pool.total(),
        2,
        "the refusal costs the player nothing",
    );
}

/// floating. The CR 601.2g float-spend prompt suspends the activation;
/// answering "yes" resumes it, pays the {T} cost, then fails with
/// `ManualTapRequired` — and the tap-cost survived the error, so every
/// retry died on `CardIsTapped` and the game was unrecoverable (no
/// cancel, no continue). The tap must revert on that path; the
/// eager-tapped floating mana stays, by design, for the retry to spend.
#[test]
fn forum_of_amity_float_prompt_failure_releases_the_tap() {
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.players[0].wants_ui = true;
    g.players[0].manual_mana = true;
    let forum = g.add_card_to_battlefield(0, catalog::forum_of_amity());
    g.add_card_to_battlefield(0, catalog::plains());
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::mountain());
    // The stuck game's shape: a floating off-color {G} from a manual tap.
    g.players[0].mana_pool.add(Color::Green, 1);
    let activate = GameAction::ActivateAbility {
        card_id: forum,
        ability_index: 2, // {2}{W}{B}, {T}: Surveil 1
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };
    let r = g.perform_action(activate.clone()).expect("suspends into the float prompt");
    assert!(r.is_empty(), "suspension emits nothing");
    assert!(
        matches!(
            g.pending_decision.as_ref().map(|p| &p.decision),
            Some(crabomination::decision::Decision::OptionalTrigger { .. })
        ),
        "the CR 601.2g float-spend confirmation is posed"
    );

    // The client auto-answers "yes, spend the float". {G} alone cannot
    // cover {2}{W}{B}, so the resume fails wanting manual taps…
    let err = g
        .perform_action(GameAction::SubmitDecision(
            crabomination::decision::DecisionAnswer::Bool(true),
        ))
        .expect_err("the float cannot cover the cost");
    assert!(format!("{err:?}").contains("ManualTapRequired"), "got {err:?}");
    // …and the tap-cost must have been released: this was the soft-lock.
    let tapped = g.battlefield.iter().find(|c| c.id == forum).map(|c| c.tapped);
    assert_eq!(tapped, Some(false), "the {{T}} cost reverts on payment failure");

    // The retry with the cost fully floating completes the activation.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(activate).expect("a covered pool activates without prompting");
    let tapped = g.battlefield.iter().find(|c| c.id == forum).map(|c| c.tapped);
    assert_eq!(tapped, Some(true), "the completed activation pays its tap");
}

/// The same combat-trick window, but reached by *playing* the combat rather
/// than hand-building the board — and paying from untapped lands rather than
/// a pre-filled pool, since mana pools empty between steps (CR 500.4).
///
/// This is the shape a real game presents, and it pins the three things the
/// client's "hold priority here" decision rests on: the declare-blockers step
/// survives the declaration (CR 509.4 restarts the priority round rather than
/// advancing to combat damage), the defender gets a real window in it, and the
/// charm shows up in that seat's `castable_hand` — the affordance list the
/// client reads to decide whether to auto-pass the window.
#[test]
fn quandrix_charm_is_castable_in_the_declare_blockers_window() {
    use crabomination::game::types::{Attack, AttackTarget};
    for blocks in [true, false] {
        let mut g = two_player_game();
        g.players[1].wants_ui = true;
        g.players[1].manual_mana = true;
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(atk);
        g.clear_sickness(blk);
        let charm = g.add_card_to_hand(1, catalog::quandrix_charm());
        // Untapped lands, empty pool — the state a defender actually enters
        // combat in. A pre-floated pool would hide a payment-path failure.
        let forest = g.add_card_to_battlefield(1, catalog::forest());
        let island = g.add_card_to_battlefield(1, catalog::island());
        for c in g.battlefield.iter_mut() {
            c.tapped = false;
        }
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack declared");
        while g.step == TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass to blockers");
        }
        assert_eq!(g.step, TurnStep::DeclareBlockers);

        // Blocking (or explicitly declining) must not end the step.
        let assignments = if blocks { vec![(blk, atk)] } else { vec![] };
        g.perform_action(GameAction::DeclareBlockers(assignments))
            .expect("declaration accepted");
        assert_eq!(
            g.step,
            TurnStep::DeclareBlockers,
            "CR 509.4 — the declaration restarts the priority round, it doesn't              advance to combat damage (blocks={blocks})",
        );

        // CR 509.4 hands the round back to the active player first; the
        // defender's window is one pass later.
        assert_eq!(g.priority.player_with_priority, 0, "active player goes first");
        g.perform_action(GameAction::PassPriority).expect("active player passes");
        assert_eq!(g.step, TurnStep::DeclareBlockers, "still before damage");
        assert_eq!(g.priority.player_with_priority, 1, "the defender's window");

        // What the client reads to decide whether to auto-pass this window.
        assert!(
            g.compute_hand_affordances(1).castable.contains(&charm),
            "the charm must be advertised as castable here, or the client              auto-passes the window (blocks={blocks})",
        );

        // And the cast really goes through, paid by hand from the two lands.
        for land in [forest, island] {
            g.perform_action(GameAction::ActivateAbility {
                card_id: land,
                ability_index: 0,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            })
            .expect("tap a land for its colour");
        }
        g.perform_action(GameAction::CastSpell {
            card_id: charm,
            target: Some(Target::Permanent(blk)),
            additional_targets: vec![],
            mode: Some(2),
            x_value: None,
        })
        .expect("mode 2 is castable before combat damage");
        drain_stack(&mut g);
        let cp = g.computed_permanent(blk).expect("the creature survives the trick");
        assert_eq!((cp.power, cp.toughness), (5, 5), "base 5/5, applied before damage");
    }
}

/// The recorded 2026-08-19 loss: a blocking Fractal needed Quandrix
/// Charm's third mode (base 5/5), but the client's castability hint
/// probed only the default mode — mode 0 wants a spell on the stack and
/// mode 1 an enchantment, so with neither around the charm read as
/// uncastable and the block died with the answer in hand. The engine
/// always accepted the cast; the affordance sweep now probes every mode.
#[test]
fn quandrix_charm_castable_hint_probes_every_mode() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let fractal = g.add_card_to_battlefield(
        0,
        crabomination::card::CardDefinition {
            name: "Fractal",
            card_types: vec![crabomination::card::CardType::Creature],
            power: 3,
            toughness: 3,
            ..Default::default()
        },
    );
    g.battlefield.iter_mut().find(|c| c.id == fractal).unwrap().is_token = true;
    g.step = TurnStep::DeclareBlockers;
    g.active_player_idx = 1;
    g.attacking = vec![crabomination::game::types::Attack {
        attacker: atk,
        target: crabomination::game::types::AttackTarget::Player(0),
    }];
    g.block_map.insert(atk, [fractal].into_iter().collect());
    g.priority.player_with_priority = 0;
    let charm = g.add_card_to_hand(0, catalog::quandrix_charm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    // The affordance layer — what the client's hand highlight reads.
    assert!(
        g.castable_hand_cards(0).contains(&charm),
        "the charm is castable via mode 2 even with modes 0/1 targetless"
    );
    // And the cast itself buffs the blocker.
    g.perform_action(GameAction::CastSpell {
        card_id: charm,
        target: Some(Target::Permanent(fractal)),
        additional_targets: vec![],
        mode: Some(2),
        x_value: None,
    })
    .expect("mode 2 targets the blocking token");
    drain_stack(&mut g);
    let cp = g.computed_permanent(fractal).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "base 5/5 until end of turn");
}

/// Proctor's Gaze — "return **up to one** target nonland permanent to its
/// owner's hand, then search for a basic land" — must fetch the land and
/// bounce *nothing* when the opponent controls no nonland permanent.
///
/// Regression, 2026-08-22, from four recorded games: the bot cast Gaze for
/// the ramp and the slot walk, which had no side preference, handed the
/// optional bounce slot the bot's own creature. Board reconstructed from
/// `replay-1787255919-2` (Deluge Virtuoso): the opponent controlled five
/// lands and nothing else.
#[test]
fn proctors_gaze_declines_the_bounce_rather_than_returning_our_own_body() {
    let mut g = two_player_game();
    let own = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for l in [catalog::mountain(), catalog::forest(), catalog::forest()] {
        g.add_card_to_battlefield(1, l);
    }
    let gaze = catalog::proctors_gaze();
    let (slot0, extra) = g.auto_targets_for_effect_all_slots(&gaze.effect, 0, None);
    assert_eq!(slot0, None, "the optional bounce is declined, not aimed at {own:?}");
    assert!(extra.is_empty());

    // And with an enemy body available it is aimed there, so declining is
    // about the *side*, not about never bouncing at all.
    let opp = g.add_card_to_battlefield(1, catalog::hill_giant());
    let (slot0, _) = g.auto_targets_for_effect_all_slots(&gaze.effect, 0, None);
    assert_eq!(slot0, Some(Target::Permanent(opp)), "bounces the opponent's creature");
}

/// Homesickness — "target player draws two cards; tap up to two target
/// creatures and put a stun counter on each" — must draw for *us* and stun
/// *them*. The slots disagree about whose side they want, which is the
/// point of the test.
///
/// Regression, 2026-08-22, from `replay-1787357896-1` turn 16: the bot cast
/// this and put both stun counters on its own Campus Composer and Fractal
/// Tender. `prefers_friendly_target` answered per *effect* and combined
/// children with `any`, so the friendly "target player draws two" made the
/// whole spell read friendly and the walk preferred the caster's own
/// creatures for the stun slots.
#[test]
fn homesickness_draws_for_us_and_stuns_them() {
    let mut g = two_player_game();
    let own = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let a = g.add_card_to_battlefield(1, catalog::hill_giant());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hs = catalog::homesickness();
    let (slot0, extra) = g.auto_targets_for_effect_all_slots(&hs.effect, 0, None);
    assert_eq!(slot0, Some(Target::Player(0)), "slot 0 is a gift: we draw");
    assert!(
        !extra.contains(&Target::Permanent(own)),
        "slots 1-2 are hostile: our own {own:?} must not be stunned, got {extra:?}"
    );
    for t in [a, b] {
        assert!(extra.contains(&Target::Permanent(t)), "stuns the opponent's {t:?}");
    }
}

/// Regression: the client identifies the cast-time extra-target suspend so it
/// can fold the pick into the cast it is holding for manual mana payment.
/// Without that fold, a `ManualTapRequired` bounce re-arms a cast still
/// missing slot 1 and every mana tap re-poses the prompt.
///
/// It used to identify it by string-comparing the prompt against
/// `EXTRA_CAST_TARGET_PROMPT` — so teaching the prompt to name its slot broke
/// the match silently and Vibrant Outburst started asking for its tap target
/// again after paying. The decision now carries a structural
/// `extra_cast_slot` flag instead, which no prompt rewording can break.
#[test]
fn the_extra_cast_target_prompt_is_flagged_structurally() {
    use crabomination::decision::Decision;
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let tapme = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    // Vibrant Outburst — "3 damage to any target. Tap up to one target
    // creature." Slot 0 is the burn, slot 1 the optional tap.
    let id = g.add_card_to_hand(0, catalog::vibrant_outburst());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Vibrant Outburst castable for {U}{R}");

    match g.pending_decision.as_ref().map(|p| &p.decision) {
        Some(Decision::ChooseTarget { extra_cast_slot, description, .. }) => {
            assert!(
                *extra_cast_slot,
                "the cast-time extra slot is flagged, so the client folds the answer \
                 into the held cast instead of re-prompting after payment",
            );
            assert_ne!(
                description,
                crabomination::decision::EXTRA_CAST_TARGET_PROMPT,
                "and the prompt is free to name its slot without breaking that",
            );
        }
        other => panic!("expected the slot-1 target prompt, got {other:?}"),
    }

    // Every other ChooseTarget stays unflagged.
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Target(
        Target::Permanent(tapme),
    )))
    .expect("slot 1 accepted");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "the 2/2 took 3 damage");
    assert!(
        g.battlefield_find(tapme).is_some_and(|c| c.tapped),
        "and the second target was tapped",
    );
}

/// A multi-target spell's prompts have to say *which part* each slot feeds.
///
/// Together as One is "target player draws X, this deals X damage to any
/// target, and you gain X life". Both picks are made through separate prompts
/// and both used to read the same: the cast-time cursor showed a bare "Click
/// on a target" and the engine's extra-target suspend said "choose an
/// additional target". With a player legal for *both* slots there was no way
/// to tell which half you were aiming (reported Aug 2026).
#[test]
fn together_as_one_names_the_part_each_target_slot_feeds() {
    use crabomination::decision::Decision;
    let def = catalog::together_as_one();
    // The per-slot text distinguishes the two halves.
    assert_eq!(
        def.effect.target_slot_text(0, None).as_deref(),
        Some("draw cards"),
        "slot 0 is the draw half",
    );
    assert_eq!(
        def.effect.target_slot_text(1, None).as_deref(),
        Some("deal damage to target"),
        "slot 1 is the burn half",
    );

    // And the engine's slot-1 prompt carries it instead of the generic text.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::together_as_one());
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Together as One castable for {6}");
    match g.pending_decision.as_ref().map(|p| &p.decision) {
        Some(Decision::ChooseTarget { description, source_name, .. }) => {
            assert_eq!(source_name, "Together as One");
            assert_eq!(
                description, "deal damage to target",
                "the second prompt names the burn half, not \"an additional target\"",
            );
        }
        other => panic!("expected the slot-1 target prompt, got {other:?}"),
    }

    // The cast still resolves correctly: converge 3 → draw 3, 3 damage, 3 life.
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Target(
        Target::Permanent(bear),
    )))
    .expect("slot 1 accepted");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23, "converge 3 → gain 3");
    assert!(g.battlefield_find(bear).is_none(), "the 2/2 took 3 damage");
}
