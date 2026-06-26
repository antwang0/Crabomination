//! Edge of Eternities — Warp (cast cheap, exile at next end step, recast from
//! exile), Void (a nonland permanent left the battlefield or a spell was warped
//! this turn), Lander tokens, and assorted card behaviors.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::types::TurnStep;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Sacrifice a battlefield permanent, firing LTB / dies triggers (CR 701.16).
fn kill(g: &mut GameState, id: CardId) {
    use crate::game::types::Target;
    let ctl = g.battlefield_find(id).unwrap().controller;
    let ctx = crate::game::effects::EffectContext::for_ability(id, ctl, Some(Target::Permanent(id)));
    g.resolve_effect(
        &crate::effect::Effect::SacrificePermanent { what: crate::effect::Selector::Target(0) },
        &ctx,
    )
    .unwrap();
    drain_stack(g);
}

/// Warp: cast Bygone Colossus for its {3} warp cost. It enters as a 9/9, and at
/// the next end step it's exiled with a `WhileExiled` may-play so it can be
/// recast from exile.
#[test]
fn warp_casts_cheap_then_exiles_at_end_step_and_grants_recast() {
    let mut g = two_player_game();
    let colossus = g.add_card_to_hand(0, catalog::bygone_colossus());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3); // the warp cost, not the {9} face
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: colossus, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("warp-cast Bygone Colossus for {3}");
    drain_stack(&mut g);
    let c = g.battlefield_find(colossus).expect("Colossus entered");
    assert_eq!((c.definition.power, c.definition.toughness), (9, 9));
    assert!(g.players[0].warped_spell_this_turn, "warping a spell satisfies Void");

    // At the next end step the warp delayed trigger exiles it.
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(colossus).is_none(), "warped permanent left the battlefield");
    let exiled = g.exile.iter().find(|c| c.id == colossus).expect("exiled by warp");
    assert!(exiled.may_play_until.is_some(), "recastable from exile");
}

/// Void inactive: Decode Transmissions draws two and you lose 2 life.
#[test]
fn void_inactive_decode_transmissions_self_loses_life() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let decode = g.add_card_to_hand(0, catalog::decode_transmissions());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: decode, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Decode Transmissions");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 - 2, "you lose 2 life with Void off");
    assert_eq!(g.players[1].life, life1, "opponent untouched");
}

/// Void active (a creature died this turn): Decode Transmissions instead makes
/// each opponent lose 2 life.
#[test]
fn void_active_decode_transmissions_opponent_loses_life() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    // Make a nonland permanent leave the battlefield this turn.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    kill(&mut g, bear);
    drain_stack(&mut g);
    assert!(g.nonland_permanent_left_bf_this_turn, "Void condition latched");

    let decode = g.add_card_to_hand(0, catalog::decode_transmissions());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: decode, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Decode Transmissions");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0, "you keep your life with Void on");
    assert_eq!(g.players[1].life, life1 - 2, "opponent loses 2 with Void on");
}

/// Lander: Biomechan Engineer's ETB mints a Lander; sacrificing it for {2} fetches
/// a basic land onto the battlefield tapped.
#[test]
fn lander_token_fetches_a_basic_land() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let plains = g.add_card_to_library(0, catalog::plains());
    let eng = g.add_card_to_hand(0, catalog::biomechan_engineer());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: eng, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Biomechan Engineer");
    drain_stack(&mut g);
    let lander = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Lander")
        .expect("ETB created a Lander").id;
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: lander, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("crack the Lander");
    drain_stack(&mut g);
    let p = g.battlefield_find(plains).expect("Plains fetched to battlefield");
    assert!(p.tapped, "fetched land enters tapped");
    assert!(g.battlefield_find(lander).is_none(), "Lander was sacrificed");
}

/// Drix Fatemaker's static gives trample to your creatures with a +1/+1 counter.
#[test]
fn drix_fatemaker_grants_trample_to_countered_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::drix_fatemaker());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // No counter yet → no trample.
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample));
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample),
        "a +1/+1 counter turns on the trample static"
    );
}

/// Broodguard Elite leaves with X counters and dumps them on a target creature.
#[test]
fn broodguard_elite_moves_counters_on_leave() {
    let mut g = two_player_game();
    let brood = g.add_card_to_battlefield(0, catalog::broodguard_elite());
    g.battlefield_find_mut(brood).unwrap().counters.insert(CounterType::PlusOnePlusOne, 3);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    kill(&mut g, brood);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3,
        "the leaving Broodguard moved its counters to the bear"
    );
}

/// Cosmic Epiphany draws one card per instant/sorcery in your graveyard.
#[test]
fn cosmic_epiphany_draws_per_instant_sorcery_in_graveyard() {
    let mut g = two_player_game();
    let id1 = g.next_id(); g.players[0].graveyard.push(crate::card::CardInstance::new(id1, catalog::lightning_bolt(), 0));
    let id2 = g.next_id(); g.players[0].graveyard.push(crate::card::CardInstance::new(id2, catalog::day_of_judgment(), 0));
    let id3 = g.next_id(); g.players[0].graveyard.push(crate::card::CardInstance::new(id3, catalog::grizzly_bears(), 0));
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let epiphany = g.add_card_to_hand(0, catalog::cosmic_epiphany());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: epiphany, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cosmic Epiphany");
    drain_stack(&mut g);
    // -1 (the spell left hand) + 2 drawn.
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 2, "drew 2 (one per I/S in gy)");
}

/// Beyond the Quiet exiles every creature.
#[test]
fn beyond_the_quiet_exiles_all_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let quiet = g.add_card_to_hand(0, catalog::beyond_the_quiet());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: quiet, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Beyond the Quiet");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert!(g.exile.iter().any(|c| c.id == a) && g.exile.iter().any(|c| c.id == b));
}

/// Perimeter Patrol grows whenever an artifact you control enters.
#[test]
fn perimeter_patrol_pumps_on_artifact_etb() {
    let mut g = two_player_game();
    let patrol = g.add_card_to_battlefield(0, catalog::perimeter_patrol());
    assert_eq!(g.computed_permanent(patrol).unwrap().power, 3);
    let art = g.add_card_to_hand(0, catalog::memory_guardian());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: art, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Memory Guardian");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(patrol).unwrap().power, 4, "+1/+0 from an artifact entering");
}
