//! Commander: the Prismari Artistry precon (SOC, Rootha,
//! `decks::cmdr_rootha`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_x(g, seat, id, target, None)
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
}

fn declare(g: &mut GameState, seat: usize, attacks: Vec<(CardId, usize)>) -> Result<(), String> {
    g.active_player_idx = seat;
    for (a, _) in &attacks {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, d)| Attack { attacker, target: AttackTarget::Player(d) }).collect(),
    ))
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn prepared(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::Prepared)
}

/// Rootha's X is the greatest instant/sorcery mana value cast this turn, X
/// included (CR 202.3e): Braingeyser for 4 is a 6. No spell, no token.
#[test]
fn rootha_makes_an_x_x_off_the_biggest_spell() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    g.add_card_to_battlefield(0, catalog::rootha_mastering_the_moment());
    step(&mut g, TurnStep::BeginCombat);
    assert!(named(&g, 0, "Elemental").is_empty(), "no spell, no token");
    g.step = TurnStep::PreCombatMain;
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, 0, opt, None).expect("opt");
    let bg = g.add_card_to_hand(0, catalog::braingeyser());
    cast_x(&mut g, 0, bg, Some(Target::Player(0)), Some(4)).expect("braingeyser");
    step(&mut g, TurnStep::BeginCombat);
    let el = named(&g, 0, "Elemental");
    assert_eq!(el.len(), 1);
    assert_eq!(pt(&g, el[0]), (6, 6), "locked at mint time");
    let c = g.computed_permanent(el[0]).unwrap();
    assert!(c.keywords().contains(&Keyword::Flying) && c.keywords().contains(&Keyword::Haste));
}

/// Muddle becomes a copy of a nonlegendary creature of yours until end of
/// turn (CR 707.2) when you cast an instant or sorcery.
#[test]
fn muddle_copies_a_creature_on_a_spell() {
    let mut g = pod(3);
    stock_libraries(&mut g, 5);
    let m = g.add_card_to_battlefield(0, catalog::muddle_the_ever_changing());
    g.add_card_to_battlefield(0, catalog::hill_giant());
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, 0, opt, None).expect("opt");
    let c = g.battlefield_find(m).unwrap();
    assert_eq!(c.definition.name, "Hill Giant");
    assert_eq!(pt(&g, m), (3, 3));
}

/// Inspired Skypainter enters prepared; Maestro's Gift copies a creature of
/// yours with haste.
#[test]
fn inspired_skypainter_prepares_maestros_gift() {
    let mut g = pod(2);
    let sp = g.add_card_to_hand(0, catalog::inspired_skypainter());
    cast(&mut g, 0, sp, None).expect("cast");
    let sp = named(&g, 0, "Inspired Skypainter")[0];
    assert_eq!(prepared(&g, sp), 1);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: sp,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Maestro's Gift");
    drain_stack(&mut g);
    let giants = named(&g, 0, "Hill Giant");
    assert_eq!(giants.len(), 2);
    let copy = giants.into_iter().find(|id| *id != giant).unwrap();
    assert!(g.computed_permanent(copy).unwrap().keywords().contains(&Keyword::Haste));
}

/// Abstract Performance — one pile of four goes to the graveyard, the other
/// comes to you (one spell of it cast free, the rest to hand).
#[test]
fn abstract_performance_splits_eight() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let ap = g.add_card_to_hand(0, catalog::abstract_performance());
    let hand = g.players[0].hand.len() - 1;
    cast(&mut g, 0, ap, None).expect("cast");
    // The face-up pile (four Bears) is the rich one, so the chooser bins it.
    let bears_in_gy = g.players[0].graveyard.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears_in_gy, 4);
    assert_eq!(g.players[0].hand.len(), hand + 4, "the four Islands");
    assert!(g.exile.is_empty());
}

/// Abstract Performance casts a spell from the kept pile for free.
#[test]
fn abstract_performance_casts_one_free() {
    let mut g = pod(2);
    // Top four (face down): Islands. Next four (face up): one Bear and three
    // Islands — too poor to bin, so the chooser bins the face-down pile.
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let ap = g.add_card_to_hand(0, catalog::abstract_performance());
    cast(&mut g, 0, ap, None).expect("cast");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1, "cast free");
    let islands_in_hand = g.players[0].hand.iter().filter(|c| c.definition.name == "Island").count();
    assert_eq!(islands_in_hand, 3);
}

/// Dirgur Focusmage — {1} off instants and sorceries; a 5+ one from hand
/// prepares Braingeyser, which draws X.
#[test]
fn dirgur_focusmage_prepares_braingeyser() {
    let mut g = pod(2);
    stock_libraries(&mut g, 20);
    let df = g.add_card_to_battlefield(0, catalog::dirgur_focusmage());
    let small = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, 0, small, None).expect("opt");
    assert_eq!(prepared(&g, df), 0);
    let bg = g.add_card_to_hand(0, catalog::braingeyser());
    cast_x(&mut g, 0, bg, Some(Target::Player(0)), Some(3)).expect("braingeyser for 3");
    assert_eq!(prepared(&g, df), 1);
    let h = g.players[0].hand.len();
    flood(&mut g, 0);
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: df,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("prepared Braingeyser");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h + 2);
    assert_eq!(prepared(&g, df), 0);
}

/// Leitmotif Composer copies itself on a 5+ spell; {2}{U} makes every
/// Composer unblockable this turn.
#[test]
fn leitmotif_composer_multiplies() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let lc = g.add_card_to_battlefield(0, catalog::leitmotif_composer());
    let bg = g.add_card_to_hand(0, catalog::braingeyser());
    cast_x(&mut g, 0, bg, Some(Target::Player(0)), Some(3)).expect("braingeyser");
    let all = named(&g, 0, "Leitmotif Composer");
    assert_eq!(all.len(), 2);
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lc,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    for id in all {
        assert!(g.computed_permanent(id).unwrap().keywords().contains(&Keyword::Unblockable));
    }
}

/// Furygale Flocking — two 3/3 flyers per opponent with haste, each pair made
/// to attack its own opponent (CR 508.1d).
#[test]
fn furygale_flocking_sends_two_at_each_opponent() {
    let mut g = pod(3);
    let ff = g.add_card_to_hand(0, catalog::furygale_flocking());
    cast(&mut g, 0, ff, None).expect("cast");
    let els = named(&g, 0, "Elemental");
    assert_eq!(els.len(), 4);
    for id in &els {
        let c = g.battlefield_find(*id).unwrap();
        assert!(g.computed_permanent(*id).unwrap().keywords().contains(&Keyword::Haste));
        assert!(matches!(c.chosen_player, Some(1) | Some(2)));
    }
    for p in [1, 2] {
        assert_eq!(els.iter().filter(|id| g.battlefield_find(**id).unwrap().chosen_player == Some(p)).count(), 2);
    }
}

/// Prismari Pianist — one Elemental per instant or sorcery, three for a 5+.
#[test]
fn prismari_pianist_scales_with_mana_value() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    g.add_card_to_battlefield(0, catalog::prismari_pianist());
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, 0, opt, None).expect("opt");
    assert_eq!(named(&g, 0, "Elemental").len(), 1);
    let bg = g.add_card_to_hand(0, catalog::braingeyser());
    cast_x(&mut g, 0, bg, Some(Target::Player(0)), Some(3)).expect("braingeyser");
    assert_eq!(named(&g, 0, "Elemental").len(), 4);
}

/// Renegade Bull — +X/+0 per spell's mana value; attacking recasts a copy of
/// an instant from your graveyard (CR 707.12).
#[test]
fn renegade_bull_pumps_and_recasts() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let bull = g.add_card_to_battlefield(0, catalog::renegade_bull());
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, 0, opt, None).expect("opt");
    assert_eq!(pt(&g, bull), (1, 5));
    // The attack half on a fresh board, a lone Bolt in the graveyard.
    let mut g = pod(2);
    let bull = g.add_card_to_battlefield(0, catalog::renegade_bull());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let l = g.players[1].life;
    declare(&mut g, 0, vec![(bull, 1)]).expect("attack");
    assert!(g.exile.iter().any(|c| c.definition.name == "Lightning Bolt"), "the card is exiled");
    assert_eq!(g.players[1].life, l - 3, "the copy was cast at the opponent");
}

/// Determined Iteration populates with haste, then sacrifices the copy at the
/// next end step; with no creature token it does nothing.
#[test]
fn determined_iteration_populates_a_hasty_copy() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::determined_iteration());
    let ff = g.add_card_to_hand(0, catalog::prismari_pianist());
    cast(&mut g, 0, ff, None).expect("pianist");
    step(&mut g, TurnStep::BeginCombat);
    assert!(named(&g, 0, "Elemental").is_empty());
    let opt = g.add_card_to_hand(0, catalog::opt());
    stock_libraries(&mut g, 5);
    g.step = TurnStep::PreCombatMain;
    cast(&mut g, 0, opt, None).expect("opt");
    step(&mut g, TurnStep::BeginCombat);
    let els = named(&g, 0, "Elemental");
    assert_eq!(els.len(), 2);
    assert!(els.iter().any(|id| g.computed_permanent(*id).unwrap().keywords().contains(&Keyword::Haste)));
    step(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Elemental").len(), 1);
}

/// Mirrorwing Dragon — a spell that targets only it is copied for each other
/// creature its caster controls (CR 707.10).
#[test]
fn mirrorwing_dragon_redirects_onto_the_casters_creatures() {
    let mut g = pod(2);
    let md = g.add_card_to_battlefield(0, catalog::mirrorwing_dragon());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(md))).expect("bolt");
    assert!(named(&g, 1, "Grizzly Bears").is_empty(), "the caster's Bears were bolted");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1, "the Dragon's controller's weren't");
}

/// Plargg and Nassari — each player exiles to a nonland card; the opponent
/// vetoes the biggest; up to two of the rest are cast free.
#[test]
fn plargg_and_nassari_casts_what_the_opponent_allows() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::plargg_and_nassari());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::hill_giant());
    g.add_card_to_library(2, catalog::llanowar_elves());
    // Seat 0's library is its own; the veto names the Hill Giant (MV 4).
    step(&mut g, TurnStep::Upkeep);
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1);
    assert_eq!(named(&g, 0, "Llanowar Elves").len(), 1);
    assert!(named(&g, 0, "Hill Giant").is_empty());
    assert!(g.exile.iter().any(|c| c.definition.name == "Hill Giant"));
    assert!(g.exile.iter().any(|c| c.definition.name == "Island"));
}

/// Redoubled Stormsinger copies each creature token that entered this turn,
/// tapped and attacking; the copies are sacrificed at the next end step.
#[test]
fn redoubled_stormsinger_doubles_fresh_tokens() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let rs = g.add_card_to_battlefield(0, catalog::redoubled_stormsinger());
    g.add_card_to_battlefield(0, catalog::prismari_pianist());
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, 0, opt, None).expect("opt");
    declare(&mut g, 0, vec![(rs, 1)]).expect("attack");
    let els = named(&g, 0, "Elemental");
    assert_eq!(els.len(), 2);
    assert!(els.iter().any(|id| g.attacking.iter().any(|a| a.attacker == *id)), "the copy attacks");
    step(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Elemental").len(), 1);
}

/// Surge to Victory — +X/+0 from the exiled card; a creature connecting casts
/// a copy of it.
#[test]
fn surge_to_victory_pumps_and_copies_on_damage() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let sv = g.add_card_to_hand(0, catalog::surge_to_victory());
    cast(&mut g, 0, sv, Some(Target::Permanent(bolt))).expect("cast");
    assert_eq!(pt(&g, bear), (3, 2));
    let l = g.players[1].life;
    declare(&mut g, 0, vec![(bear, 1)]).expect("attack");
    while g.step != TurnStep::EndCombat && !g.is_game_over() {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, l - 3 - 3, "three combat damage and a copied Bolt");
}

/// Volcanic Salvo costs {X} less (total power) and deals 6 to each of up to
/// two targets.
#[test]
fn volcanic_salvo_hits_two() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(1, catalog::hill_giant());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let vs = g.add_card_to_hand(0, catalog::volcanic_salvo());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: vs,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// The three Izzet duals enter tapped (Turbulent Springs untapped against
/// eight opposing lands).
#[test]
fn izzet_duals_enter_tapped() {
    for f in [catalog::coastal_peak, catalog::molten_tributary, catalog::turbulent_springs] {
        let mut g = pod(2);
        let l = g.add_card_to_hand(0, f());
        g.perform_action(GameAction::PlayLand(l)).expect("land");
        assert!(g.battlefield_find(l).unwrap().tapped);
    }
    let mut g = pod(2);
    for _ in 0..8 {
        g.add_card_to_battlefield(1, catalog::island());
    }
    let l = g.add_card_to_hand(0, catalog::turbulent_springs());
    g.perform_action(GameAction::PlayLand(l)).expect("land");
    assert!(!g.battlefield_find(l).unwrap().tapped);
}

/// Throes of Chaos cascades (CR 702.85) into a cheaper nonland card.
#[test]
fn throes_of_chaos_cascades() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::throes_of_chaos());
    cast(&mut g, 0, t, None).expect("cast");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1);
}
