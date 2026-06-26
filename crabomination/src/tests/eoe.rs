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

/// Exalted Sunborn doubles tokens you create.
#[test]
fn exalted_sunborn_doubles_tokens() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::exalted_sunborn());
    // Knight Luminary's ETB normally makes one Human Soldier → doubled to two.
    let lum = g.add_card_to_hand(0, catalog::knight_luminary());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: lum, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Knight Luminary");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Human Soldier").count();
    assert_eq!(soldiers, 2, "token doubler made two Human Soldiers");
}

/// Memorial Team Leader pumps your other creatures only on your turn.
#[test]
fn memorial_team_leader_anthem_only_on_your_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::memorial_team_leader());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+0 on your turn");
    g.active_player_idx = 1;
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "no bonus on the opponent's turn");
}

/// Kavaron Skywarden grows at your end step while Void is active.
#[test]
fn kavaron_skywarden_grows_on_void_end_step() {
    let mut g = two_player_game();
    let skywarden = g.add_card_to_battlefield(0, catalog::kavaron_skywarden());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    kill(&mut g, fodder); // Void on
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(skywarden).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "Void end step added a +1/+1 counter"
    );
}

/// Emergency Eject destroys a nonland permanent and gives its controller a Lander.
#[test]
fn emergency_eject_gives_controller_a_lander() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let eject = g.add_card_to_hand(0, catalog::emergency_eject());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: eject, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Emergency Eject");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "victim destroyed");
    let landers = g.battlefield.iter()
        .filter(|c| c.definition.name == "Lander" && c.controller == 1).count();
    assert_eq!(landers, 1, "the victim's controller got the Lander");
}

/// Edge Rover gives every player a Lander when it dies.
#[test]
fn edge_rover_each_player_gets_a_lander() {
    let mut g = two_player_game();
    let rover = g.add_card_to_battlefield(0, catalog::edge_rover());
    kill(&mut g, rover);
    drain_stack(&mut g);
    for p in 0..2 {
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.name == "Lander" && c.controller == p).count(),
            1, "player {p} got a Lander"
        );
    }
}

/// Cloudsculpt Technician gets +1/+0 only while you control an artifact.
#[test]
fn cloudsculpt_technician_pumps_with_an_artifact() {
    let mut g = two_player_game();
    let tech = g.add_card_to_battlefield(0, catalog::cloudsculpt_technician());
    assert_eq!(g.computed_permanent(tech).unwrap().power, 2, "no artifact → base power");
    g.add_card_to_battlefield(0, catalog::memory_guardian()); // an artifact
    assert_eq!(g.computed_permanent(tech).unwrap().power, 3, "controlling an artifact → +1/+0");
}

/// Brightspear Zealot grows after you've cast two spells this turn.
#[test]
fn brightspear_zealot_grows_after_two_spells() {
    let mut g = two_player_game();
    let zealot = g.add_card_to_battlefield(0, catalog::brightspear_zealot());
    assert_eq!(g.computed_permanent(zealot).unwrap().power, 2);
    g.players[0].spells_cast_this_turn = 2;
    assert_eq!(g.computed_permanent(zealot).unwrap().power, 4, "+2/+0 with two spells cast");
}

/// Dual-Sun Technique draws only when the buffed creature has a +1/+1 counter.
#[test]
fn dual_sun_technique_draws_with_a_counter() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
    let tech = g.add_card_to_hand(0, catalog::dual_sun_technique());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: tech, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dual-Sun Technique");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike));
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "drew because the target had a counter");
}

/// Hymn of the Faller draws an extra card when Void is active.
#[test]
fn hymn_of_the_faller_void_draws_extra() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); }
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    kill(&mut g, fodder); // Void on
    let hymn = g.add_card_to_hand(0, catalog::hymn_of_the_faller());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Default AutoDecider keeps the surveilled card on top.
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: hymn, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hymn of the Faller");
    drain_stack(&mut g);
    // -1 (spell) + 2 drawn (base + Void).
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "Void granted the extra draw");
}
