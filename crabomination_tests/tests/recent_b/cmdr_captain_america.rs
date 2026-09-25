//! Commander: the Avengers Assemble precon (MSC, Captain America, Team
//! Leader, `decks::cmdr_captain_america`), and its primitives: "the first
//! time that creature has become tapped this turn", a gated flash grant, an
//! "another source" damage bonus, kept single-color mana, and "a player
//! attacked you during their last turn".

use crabomination::card::{CardId, CounterType, Keyword, Value};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..12 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_as(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    cast_as(g, 0, id, &[]).expect("castable");
    id
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

fn put(g: &mut GameState, seat: usize, id: CardId, n: i32) {
    let ctx = EffectContext::for_spell(seat, None, 0, 0);
    let evs = g
        .resolve_effect(
            &Effect::AddCounter {
                what: Selector::ExactObjects(vec![id]),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(n),
            },
            &ctx,
        )
        .expect("counters");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn damage(g: &mut GameState, seat: usize, source: Option<CardId>, to: Selector, n: i32) {
    let mut ctx = EffectContext::for_spell(seat, None, 0, 0);
    ctx.source = source;
    let evs = g.resolve_effect(&Effect::DealDamage { to, amount: Value::Const(n) }, &ctx).expect("damage");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn declare(g: &mut GameState, attacks: &[(CardId, usize)]) {
    for (a, _) in attacks {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.iter().map(|&(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

// ── The commander ────────────────────────────────────────────────────────────

/// Captain America, Team Leader: another Hero entering gains vigilance and
/// haste, and both get a +1/+1 counter.
#[test]
fn team_leader_rallies_an_entering_hero() {
    let mut g = pod(2);
    let cap = g.add_card_to_battlefield(0, catalog::captain_america_team_leader());
    let hero = cast(&mut g, catalog::patriot_shield_wielder());
    assert_eq!((plus(&g, hero), plus(&g, cap)), (1, 1));
    let kws = g.computed_permanent(hero).unwrap().keywords().to_vec();
    assert!(kws.contains(&Keyword::Haste) && kws.contains(&Keyword::Vigilance));
    cast(&mut g, catalog::grizzly_bears());
    assert_eq!(plus(&g, cap), 1, "not a Hero");
}

// ── Engine primitives ────────────────────────────────────────────────────────

/// Captain America, Living Legend untaps a creature of yours the first time
/// it becomes tapped during your turn — not the second time.
#[test]
fn living_legend_untaps_the_first_tap_only() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::captain_america_living_legend());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tap = |g: &mut GameState| {
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let evs = g.resolve_effect(&Effect::Tap { what: Selector::ExactObjects(vec![bear]) }, &ctx).expect("tap");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    };
    tap(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "the first tap is undone");
    tap(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "the second stays");
}

/// Captain Mar-Vell grants flash only once an opponent has cast a spell this
/// turn.
#[test]
fn mar_vell_flash_after_an_opponents_spell() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::captain_mar_vell_space_born());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(cast_as(&mut g, 0, bear, &[]).is_err(), "no flash yet");
    let theirs = g.add_card_to_hand(1, catalog::sol_ring());
    cast_as(&mut g, 1, theirs, &[]).expect("their spell");
    cast_as(&mut g, 0, bear, &[]).expect("flash now");
}

/// Quicksilver grants flash while he's tapped.
#[test]
fn quicksilver_flash_while_tapped() {
    let mut g = pod(2);
    let qs = g.add_card_to_battlefield(0, catalog::quicksilver_speedster());
    g.active_player_idx = 1;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(cast_as(&mut g, 0, bear, &[]).is_err());
    g.battlefield_find_mut(qs).unwrap().tapped = true;
    cast_as(&mut g, 0, bear, &[]).expect("flash while tapped");
}

/// Thor adds 1 to damage your other sources deal to opponents — not his own.
#[test]
fn thor_boosts_other_sources_only() {
    let mut g = pod(2);
    let thor = g.add_card_to_battlefield(0, catalog::thor_asgards_avenger());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    damage(&mut g, 0, Some(bear), Selector::Player(crabomination::effect::PlayerRef::EachOpponent), 2);
    assert_eq!(g.players[1].life, 17);
    damage(&mut g, 0, Some(thor), Selector::Player(crabomination::effect::PlayerRef::EachOpponent), 2);
    assert_eq!(g.players[1].life, 15);
}

/// Photon's combat damage becomes one color of mana kept through the turn.
#[test]
fn photon_keeps_its_mana() {
    let mut g = pod(2);
    let photon = g.add_card_to_battlefield(0, catalog::photon_mighty_marvel());
    declare(&mut g, &[(photon, 1)]);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].mana_pool.total(), 2);
    assert_eq!(g.players[0].kept_mana_this_turn.total(), 2);
}

/// Avenge costs {2} less after a player attacked you during their last
/// turn, and gains a life per creature destroyed.
#[test]
fn avenge_discount_and_lifegain() {
    let mut g = pod(3);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let av = g.add_card_to_hand(0, catalog::avenge());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    let no_discount = g.perform_action(GameAction::CastSpell {
        card_id: av,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(no_discount.is_err(), "four mana isn't six");
    g.players[1].attacked_players_this_turn.push(0);
    g.perform_action(GameAction::CastSpell {
        card_id: av,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("discounted to four");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 22);
}

// ── Cards ────────────────────────────────────────────────────────────────────

/// Captain Marvel copies +1/+1 counters you put on another non-Kree creature.
#[test]
fn captain_marvel_copies_your_counters() {
    let mut g = pod(2);
    let cm = g.add_card_to_battlefield(0, catalog::captain_marvel_apex_avenger());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let kree = g.add_card_to_battlefield(0, catalog::captain_mar_vell_space_born());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 4]));
    put(&mut g, 0, bear, 2);
    assert_eq!(plus(&g, cm), 2);
    put(&mut g, 0, kree, 1);
    assert_eq!(plus(&g, cm), 2, "a Kree's counters aren't copied");
    put(&mut g, 1, bear, 1);
    assert_eq!(plus(&g, cm), 2, "an opponent's counters aren't yours");
}

/// Shang-Chi's tenth counter draws five and gains 5, once.
#[test]
fn shang_chi_tenth_counter() {
    let mut g = pod(2);
    let sc = g.add_card_to_battlefield(0, catalog::shang_chi_and_the_ten_rings());
    put(&mut g, 0, sc, 8);
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    put(&mut g, 0, sc, 3);
    assert_eq!((g.players[0].hand.len(), g.players[0].life), (hand + 5, life + 5));
    put(&mut g, 0, sc, 1);
    assert_eq!(g.players[0].hand.len(), hand + 5, "past ten");
}

/// Jocasta returns from your graveyard tapped and attacking when you attack
/// with your commander.
#[test]
fn jocasta_follows_the_commander_into_battle() {
    let mut g = pod(2);
    let cmdr = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmdr,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast commander");
    drain_stack(&mut g);
    let jocasta = g.add_card_to_graveyard(0, catalog::jocasta_automaton_avenger());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 2]));
    declare(&mut g, &[(cmdr, 1)]);
    let j = g.battlefield_find(jocasta).expect("returned");
    assert!(j.tapped);
    assert!(g.attacking.iter().any(|a| a.attacker == jocasta));
}

/// She-Hulk: a Hero of yours becoming blocked gets a counter per blocker.
#[test]
fn she_hulk_counts_blockers() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::she_hulk_wallbreaker());
    let hero = g.add_card_to_battlefield(0, catalog::patriot_shield_wielder());
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    declare(&mut g, &[(hero, 1)]);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, hero), (b2, hero)])).expect("double block");
    drain_stack(&mut g);
    assert_eq!(plus(&g, hero), 2);
}

/// Vision triggers only on a spell cast outside its caster's turn.
#[test]
fn vision_watches_off_turn_spells() {
    let mut g = pod(2);
    let vision = g.add_card_to_battlefield(0, catalog::vision_synthezoid_avenger());
    cast(&mut g, catalog::sol_ring());
    assert_eq!(plus(&g, vision), 0, "your own turn");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[Target::Player(0)]).expect("an instant on your turn");
    let grew = plus(&g, vision) == 1;
    let phased = g.battlefield_find(vision).is_none();
    assert!(grew || phased, "one mode or the other");
}

/// Hulkbuster Armor makes the equipped creature a 9/9 flier.
#[test]
fn hulkbuster_sets_base_nine_nine() {
    let mut g = pod(2);
    let armor = g.add_card_to_battlefield(0, catalog::hulkbuster_armor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::Attach { what: Selector::ExactObjects(vec![armor]), to: Selector::ExactObjects(vec![bear]) },
        &ctx,
    )
    .expect("attach");
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (9, 9));
    assert!(c.keywords().contains(&Keyword::Flying));
}

/// Rescue bounces an artifact of yours and grows.
#[test]
fn rescue_bounces_an_artifact_and_grows() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let rescue = cast(&mut g, catalog::rescue_pepper_potts());
    assert!(g.players[0].hand.iter().any(|c| c.id == ring));
    assert_eq!(plus(&g, rescue), 1);
}

/// Hawkeye draws when an opponent's creature he damaged this turn dies.
#[test]
fn hawkeye_draws_for_his_kill() {
    let mut g = pod(2);
    let hawk = g.add_card_to_battlefield(0, catalog::hawkeye_avenging_archer());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    damage(&mut g, 0, Some(hawk), Selector::ExactObjects(vec![bear]), 2);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Love on the Battlefield wants exactly two attackers.
#[test]
fn love_on_the_battlefield_needs_a_pair() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::love_on_the_battlefield());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    declare(&mut g, &[(a, 1), (b, 1)]);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert!(g.computed_permanent(a).unwrap().keywords().contains(&Keyword::FirstStrike));
}

/// Black Widow grows and draws on an opponent's second draw each turn.
#[test]
fn black_widow_watches_second_draws() {
    let mut g = pod(3);
    let bw = g.add_card_to_battlefield(0, catalog::black_widow_agile_avenger());
    let hand = g.players[0].hand.len();
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Draw { who: Selector::You, amount: Value::Const(2) }, &ctx).expect("draw");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!((plus(&g, bw), g.players[0].hand.len()), (1, hand + 1));
}
