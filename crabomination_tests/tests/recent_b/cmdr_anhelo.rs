//! Commander: the Maestros Massacre precon (NCC, Anhelo, the Painter,
//! `decks::cmdr_anhelo`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector, Value};
use crabomination::game::effects::EffectContext;
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

fn cast_full(
    g: &mut GameState,
    seat: usize,
    id: CardId,
    target: Option<Target>,
    extra: Vec<Target>,
    mode: Option<usize>,
) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: extra, mode, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_full(g, seat, id, target, vec![], None)
}

fn cast_casualty(g: &mut GameState, id: CardId, sac: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellCasualty {
        card_id: id,
        sacrifice: sac,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn run(g: &mut GameState, effect: Effect, source: CardId) {
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(source);
    let events = g.resolve_effect(&effect, &ctx).expect("effect");
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

fn kill(g: &mut GameState, id: CardId) {
    run(g, Effect::Destroy { what: Selector::ExactObjects(vec![id]) }, id);
}

fn stock(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

/// Anhelo — the first instant or sorcery each turn has casualty 2: the
/// sacrifice copies it.
#[test]
fn anhelo_grants_casualty_to_the_first_spell() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::anhelo_the_painter());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_casualty(&mut g, bolt, bears, Some(Target::Player(1))).expect("casualty bolt");
    assert_eq!(g.players[1].life, 14, "the Bolt and its copy");
    assert!(g.battlefield_find(bears).is_none());
    let second = g.add_card_to_hand(0, catalog::lightning_bolt());
    let more = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(cast_casualty(&mut g, second, more, Some(Target::Player(1))).is_err(), "only the first");
}

/// Audacious Swap — the permanent is shuffled away and its owner plays their
/// new top card.
#[test]
fn audacious_swap_shuffles_and_replaces() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_library(1, catalog::forest());
    let swap = g.add_card_to_hand(0, catalog::audacious_swap());
    cast(&mut g, 0, swap, Some(Target::Permanent(angel))).expect("swap");
    // The Angel joined a two-card library; whichever came off the top was
    // played (the Forest) or cast for free (the Angel again).
    assert_eq!(g.players[1].library.len(), 1);
    let forest_out = !named(&g, 1, "Forest").is_empty();
    let angel_back = g.battlefield_find(angel).is_some();
    assert!(forest_out ^ angel_back, "exactly one came back");
}

/// Body Count — draws per creature of yours that died this turn.
#[test]
fn body_count_draws_per_death() {
    let mut g = pod(2);
    stock(&mut g, 0, 3);
    for _ in 0..2 {
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        kill(&mut g, b);
    }
    let bc = g.add_card_to_hand(0, catalog::body_count());
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, bc, None).expect("body count");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2);
}

/// Cormela — restricted {U}{B}{R}; dying regrows an instant or sorcery.
#[test]
fn cormela_makes_spell_mana_and_regrows() {
    let mut g = pod(2);
    let c = g.add_card_to_battlefield(0, catalog::cormela_glamour_thief());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: c,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("mana");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 3, "instant-and-sorcery-only mana");
    kill(&mut g, c);
    assert!(g.players[0].hand.iter().any(|x| x.id == bolt), "regrew the Bolt");
}

/// Cryptic Pursuit — an instant from hand manifests the top card.
#[test]
fn cryptic_pursuit_manifests() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::cryptic_pursuit());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Player(1))).expect("bolt");
    let fd: Vec<CardId> = g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).map(|c| c.id).collect();
    assert_eq!(fd.len(), 1);
    // The manifested Bolt dies: it's exiled and castable.
    kill(&mut g, fd[0]);
    assert!(g.exile.iter().any(|c| c.id == fd[0]), "exiled as an instant card");
}

/// Dogged Detective — an opponent's second draw lets it come back to hand.
#[test]
fn dogged_detective_returns_on_their_second_draw() {
    let mut g = pod(2);
    let dd = g.add_card_to_graveyard(0, catalog::dogged_detective());
    stock(&mut g, 1, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut ctx = EffectContext::for_spell(1, None, 0, 0);
    ctx.source = Some(dd);
    let ev = g.resolve_effect(&Effect::Draw { who: Selector::You, amount: Value::Const(2) }, &ctx).expect("draw");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dd));
}

/// Double Vision — the first instant each turn is copied.
#[test]
fn double_vision_copies_the_first() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::double_vision());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Player(1))).expect("bolt");
    assert_eq!(g.players[1].life, 14);
    let second = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, second, Some(Target::Player(1))).expect("bolt");
    assert_eq!(g.players[1].life, 11, "only the first is copied");
}

/// Drawn from Dreams — two of the top seven to hand.
#[test]
fn drawn_from_dreams_keeps_two() {
    let mut g = pod(2);
    stock(&mut g, 0, 7);
    let dd = g.add_card_to_hand(0, catalog::drawn_from_dreams());
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, dd, None).expect("dreams");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2);
    assert_eq!(g.players[0].library.len(), 5);
}

/// Flawless Forgery — casts a copy of an opponent's graveyard instant.
#[test]
fn flawless_forgery_casts_their_spell() {
    let mut g = pod(2);
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let ff = g.add_card_to_hand(0, catalog::flawless_forgery());
    cast(&mut g, 0, ff, Some(Target::Permanent(bolt))).expect("forgery");
    assert!(g.exile.iter().any(|c| c.id == bolt));
    assert_eq!(g.players[1].life, 17, "the copy hit the opponent");
}

/// Maestros Charm — the drain mode.
#[test]
fn maestros_charm_drains() {
    let mut g = pod(2);
    let mc = g.add_card_to_hand(0, catalog::maestros_charm());
    cast_full(&mut g, 0, mc, None, vec![], Some(1)).expect("charm");
    assert_eq!(g.players[1].life, 17);
    assert_eq!(g.players[0].life, 23);
}

/// Maestros Confluence — two -3/-3s on one creature and a goad.
#[test]
fn maestros_confluence_shrinks_and_goads() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mc = g.add_card_to_hand(0, catalog::maestros_confluence());
    cast_full(&mut g, 0, mc, Some(Target::Permanent(angel)), vec![Target::Permanent(angel)], None)
        .expect("confluence");
    assert!(g.battlefield_find(angel).is_none(), "-6/-6");
    assert!(g.goaded_by_player(g.battlefield_find(bears).unwrap(), 0));
}

/// Make an Example — the opponent sacrifices one of their two piles.
#[test]
fn make_an_example_takes_a_pile() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    let me = g.add_card_to_hand(0, catalog::make_an_example());
    cast(&mut g, 0, me, None).expect("example");
    let left = named(&g, 1, "Grizzly Bears").len();
    assert!(left < 3, "a nonempty pile went");
}

/// Parnesse — an opponent targeting your permanent pays 4 life or is
/// countered.
#[test]
fn parnesse_taxes_targeting() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::parnesse_the_subtle_brush());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g
        .computed_permanent(bears)
        .unwrap()
        .keywords()
        .iter()
        .any(|k| matches!(k, Keyword::Ward(_))));
}

/// Rekindling Phoenix — dies into an Elemental that brings it back.
#[test]
fn rekindling_phoenix_rises() {
    let mut g = pod(2);
    let rp = g.add_card_to_battlefield(0, catalog::rekindling_phoenix());
    kill(&mut g, rp);
    assert_eq!(named(&g, 0, "Elemental").len(), 1);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(rp).is_some(), "back");
    assert!(named(&g, 0, "Elemental").is_empty());
}

/// Sinister Concierge — dying suspends itself and an opposing creature.
#[test]
fn sinister_concierge_suspends() {
    let mut g = pod(2);
    let sc = g.add_card_to_battlefield(0, catalog::sinister_concierge());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    kill(&mut g, sc);
    for id in [sc, angel] {
        let c = g.exile.iter().find(|c| c.id == id).expect("exiled");
        assert_eq!(c.counter_count(CounterType::Time), 3);
    }
}

/// Skyclave Shade — kicked, it enters with two counters; a land on your
/// turn lets it be cast from the graveyard.
#[test]
fn skyclave_shade_recurs_on_landfall() {
    let mut g = pod(2);
    let ss = g.add_card_to_graveyard(0, catalog::skyclave_shade());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let land = g.add_card_to_hand(0, catalog::swamp());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    let c = g.players[0].graveyard.iter().find(|c| c.id == ss).expect("still in the graveyard");
    assert!(c.may_play_until.is_some(), "castable from the graveyard this turn");
}

/// Smuggler's Buggy — hides a card on entering.
#[test]
fn smugglers_buggy_hides_a_card() {
    let mut g = pod(2);
    stock(&mut g, 0, 4);
    let sb = g.add_card_to_hand(0, catalog::smugglers_buggy());
    cast(&mut g, 0, sb, None).expect("buggy");
    assert!(g.exile.iter().any(|c| c.exiled_with == Some(sb)));
}

/// Spellbinding Soprano — attacking discounts instants and sorceries.
#[test]
fn spellbinding_soprano_discounts() {
    let mut g = pod(2);
    let s = g.add_card_to_battlefield(0, catalog::spellbinding_soprano());
    g.clear_sickness(s);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: s, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].turn_spell_discounts.len(), 1);
}

/// Syrix — a Phoenix of yours dying lets Syrix be cast from the graveyard.
#[test]
fn syrix_returns_when_a_phoenix_dies() {
    let mut g = pod(2);
    let sy = g.add_card_to_graveyard(0, catalog::syrix_carrier_of_the_flame());
    let rp = g.add_card_to_battlefield(0, catalog::rekindling_phoenix());
    kill(&mut g, rp);
    assert!(g.players[0].graveyard.iter().find(|c| c.id == sy).is_some_and(|c| c.may_play_until.is_some()));
}

/// Waste Management — exiles two graveyard cards and makes a Rogue per
/// creature card.
#[test]
fn waste_management_makes_rogues() {
    let mut g = pod(2);
    let a = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(1, catalog::hill_giant());
    let wm = g.add_card_to_hand(0, catalog::waste_management());
    cast_full(&mut g, 0, wm, Some(Target::Permanent(a)), vec![Target::Permanent(b)], None).expect("waste");
    assert_eq!(named(&g, 0, "Rogue").len(), 2);
}

/// Xander's Pact — cast an opponent's exiled top card for life.
#[test]
fn xanders_pact_steals_a_spell() {
    let mut g = pod(2);
    let bolt = g.add_card_to_library(1, catalog::lightning_bolt());
    let xp = g.add_card_to_hand(0, catalog::xanders_pact());
    cast(&mut g, 0, xp, None).expect("pact");
    assert!(g.exile.iter().any(|c| c.id == bolt));
}

/// Zndrsplt's Judgment — you copy your best creature, opponents bounce one.
#[test]
fn zndrsplts_judgment_friend_and_foe() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::serra_angel());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let zj = g.add_card_to_hand(0, catalog::zndrsplts_judgment());
    cast(&mut g, 0, zj, None).expect("judgment");
    assert_eq!(named(&g, 0, "Serra Angel").len(), 2);
    assert!(g.players[1].hand.iter().any(|c| c.id == bears));
}
