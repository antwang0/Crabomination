//! Commander: the Sworn to Darkness precon (C14, Ob Nixilis of the Black
//! Oath, `decks::cmdr_ob_nixilis`) and the primitives it needed.

use crabomination::card::{CardId, CounterType, Keyword, SelectionRequirement as R, StaticAbility, Value};
use crabomination::catalog;
use crabomination::effect::{ActivatedAbility, Effect, PlayerRef, Selector, StaticEffect};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::{Color, b, cost, generic};

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

fn activate(g: &mut GameState, id: CardId, index: usize) -> Result<(), String> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) -> Result<(), String> {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))
}

/// CR 508.1d — "attacks that player this combat if able": the creature must
/// be declared, and only against the opponent stamped on it.
#[test]
fn cr_508_1d_must_attack_chosen_player_binds_attacker_and_defender() {
    let mut g = pod(3);
    let mut def = catalog::grizzly_bears();
    def.keywords.push(Keyword::MustAttackChosenPlayer);
    let bear = g.add_card_to_battlefield(0, def);
    g.clear_sickness(bear);
    g.battlefield_find_mut(bear).unwrap().chosen_player = Some(2);

    let snapshot = g.clone();
    assert!(declare(&mut g, vec![]).is_err(), "it must attack");
    let mut g = snapshot.clone();
    assert!(declare(&mut g, vec![(bear, 1)]).is_err(), "and only the chosen opponent");
    let mut g = snapshot.clone();
    declare(&mut g, vec![(bear, 2)]).expect("attacking the chosen opponent is legal");
}

/// CR 508.1d — with its chosen seat gone (CR 800.4a), nothing binds it.
#[test]
fn cr_508_1d_must_attack_chosen_player_lapses_when_that_player_left() {
    let mut g = pod(3);
    let mut def = catalog::grizzly_bears();
    def.keywords.push(Keyword::MustAttackChosenPlayer);
    let bear = g.add_card_to_battlefield(0, def);
    g.clear_sickness(bear);
    g.battlefield_find_mut(bear).unwrap().chosen_player = Some(2);
    g.players[2].eliminated = true;
    declare(&mut g, vec![]).expect("no live chosen opponent — no requirement");
}

/// CR 114.4 — an emblem's "creatures you control have '…'" works from the
/// command zone: each of its owner's creatures gains the activated ability,
/// and nobody else's does.
#[test]
fn cr_114_4_emblem_grants_an_activated_ability_to_its_owners_creatures() {
    let mut g = pod(3);
    let grant = StaticAbility {
        description: "Creatures you control have \"{1}{B}, Sacrifice this creature: gain X, draw X.\"",
        effect: StaticEffect::GrantActivatedAbility {
            applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            ability: ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                sac_cost: true,
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::SacrificedPower },
                    Effect::Draw { who: Selector::You, amount: Value::SacrificedPower },
                ]),
                ..Default::default()
            },
            condition: None,
        },
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::CreateEmblem { who: PlayerRef::You, name: "Test".into(), triggered: vec![], statics: vec![grant] },
        &ctx,
    )
    .expect("emblem");
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    flood(&mut g, 0);
    activate(&mut g, mine, 0).expect("the emblem's ability is on my creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "sacrificed as the cost");
    assert_eq!(g.players[0].life, life + 2, "X is the sacrificed creature's power");
    assert_eq!(g.players[0].hand.len(), hand + 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    flood(&mut g, 1);
    assert!(activate(&mut g, theirs, 0).is_err(), "not on an opponent's creature");
}

/// CR 119.3 — "you gain life equal to the life lost this way" counts every
/// opponent's loss at a table; `Drain` gains its amount once.
#[test]
fn cr_119_3_drain_life_lost_gains_the_total_lost() {
    let mut g = pod(4);
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let each_opp = || Selector::Player(PlayerRef::EachOpponent);
    g.resolve_effect(&Effect::DrainLifeLost { from: each_opp(), to: Selector::You, amount: Value::Const(2) }, &ctx)
        .expect("drain");
    assert_eq!(g.players[0].life, 26, "three opponents lost 2 each");
    for seat in 1..4 {
        assert_eq!(g.players[seat].life, 18);
    }
    g.players[3].life = 1;
    g.resolve_effect(&Effect::DrainLifeLost { from: each_opp(), to: Selector::You, amount: Value::Const(2) }, &ctx)
        .expect("drain");
    assert_eq!(g.players[0].life, 32, "a seat at 1 still loses 2 (life can go negative)");
}

/// CR 119.3 / 800.4 — Gray Merchant and Kokusho gain every opponent's loss at
/// a table: Gray Merchant's devotion 2 drains three opponents for 6; Kokusho's
/// death, three opponents for 15.
#[test]
fn cr_119_3_gray_merchant_and_kokusho_gain_the_table_total() {
    let mut g = pod(4);
    let gary = g.add_card_to_hand(0, catalog::gray_merchant_of_asphodel());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: gary, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 26);
    assert_eq!(g.players[1].life, 18);
    let kokusho = g.add_card_to_battlefield(0, catalog::kokusho_the_evening_star());
    let ctx = EffectContext::for_spell(1, Some(Target::Permanent(kokusho)), 0, 0);
    g.resolve_effect(&Effect::Destroy { what: Selector::Target(0) }, &ctx).expect("destroy");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 41);
    assert_eq!(g.players[3].life, 13);
}

/// CR 707.10 / 608.2b — a copy of your own Destroy spell defaults to another
/// opposing creature: the original target is gone by the time the copy would
/// destroy it (Reverberate on your own Murder).
#[test]
fn cr_707_10_a_self_copied_destroy_defaults_to_a_new_target() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    let reverb = g.add_card_to_hand(0, catalog::reverberate());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: murder, target: Some(Target::Permanent(angel)), additional_targets: vec![], mode: None, x_value: None })
        .expect("Murder");
    g.perform_action(GameAction::CastSpell { card_id: reverb, target: Some(Target::Permanent(murder)), additional_targets: vec![], mode: None, x_value: None })
        .expect("Reverberate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none());
    assert!(g.battlefield_find(bear).is_none(), "the copy took the Bear");
}

// ── The cards ──

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    g.battlefield_find_mut(id).unwrap().loyalty_uses_this_turn = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: None })
        .expect("loyalty ability");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, extra: Vec<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, g.priority.player_with_priority);
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: extra, mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Morbid (CR 207.2c ability word) — a creature died this turn.
fn morbid(g: &mut GameState) {
    g.players[1].creatures_died_this_turn = 1;
}

/// CR 903.3a — Ob Nixilis can be a commander. +2 drains every opponent at a
/// table (CR 800.4 — "each opponent" is all of them); −2 makes a 5/5 flying
/// Demon for 2 life; −8's emblem hands each of your creatures its ability.
#[test]
fn ob_nixilis_of_the_black_oath_drains_the_table_and_makes_demons() {
    let def = catalog::ob_nixilis_of_the_black_oath();
    assert!(def.can_be_commander, "CR 903.3a — \"can be your commander\"");
    let mut g = pod(4);
    let ob = g.add_card_to_battlefield(0, def);
    let life = g.players[0].life;
    loyalty(&mut g, ob, 0, None);
    assert_eq!(g.players[0].life, life + 3, "gains what the three opponents lost");
    for seat in 1..4 {
        assert_eq!(g.players[seat].life, 19);
    }
    loyalty(&mut g, ob, 1, None);
    let demons = named(&g, 0, "Demon");
    assert_eq!(demons.len(), 1);
    assert_eq!(pt(&g, demons[0]), (5, 5));
    assert!(g.computed_permanent(demons[0]).unwrap().keywords().contains(&Keyword::Flying));
    assert_eq!(g.players[0].life, life + 1, "−2 costs 2 life");

    g.battlefield_find_mut(ob).unwrap().counters.insert(CounterType::Loyalty, 8);
    loyalty(&mut g, ob, 2, None);
    assert_eq!(g.players[0].emblems.len(), 1);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::swamp());
    }
    let hand = g.players[0].hand.len();
    flood(&mut g, 0);
    activate(&mut g, demons[0], 0).expect("the emblem's ability rides the Demon");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 5, "draws the Demon's power");
    assert_eq!(g.players[0].life, life + 6);
}

/// Lieutenant (CR 207.2c) — only while you control your commander: +2/+2 and
/// combat damage to a player makes that player sacrifice a creature.
#[test]
fn demon_of_wailing_agonies_is_a_lieutenant() {
    let mut g = pod(3);
    let demon = g.add_card_to_battlefield(0, catalog::demon_of_wailing_agonies());
    assert_eq!(pt(&g, demon), (4, 4));
    let ob = g.add_card_to_battlefield(0, catalog::ob_nixilis_of_the_black_oath());
    g.players[0].commanders.push(ob);
    assert_eq!(pt(&g, demon), (6, 6));
    let bear = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(demon);
    declare(&mut g, vec![(demon, 2)]).expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 2;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[2].life, 14);
    assert!(g.battlefield_find(bear).is_none(), "the damaged player sacrificed");
    assert!(g.battlefield_find(other).is_some(), "the other opponent didn't");
}

/// {B}: +1/+1; undying (CR 702.93) brings it back with a +1/+1 counter.
#[test]
fn evernight_shade_pumps_and_returns() {
    let mut g = pod(2);
    let shade = g.add_card_to_battlefield(0, catalog::evernight_shade());
    flood(&mut g, 0);
    activate(&mut g, shade, 0).expect("pump");
    drain_stack(&mut g);
    assert_eq!(pt(&g, shade), (2, 2));
    let ctx = EffectContext::for_spell(1, Some(Target::Permanent(shade)), 0, 0);
    g.resolve_effect(&Effect::Destroy { what: Selector::Target(0) }, &ctx).expect("destroy");
    drain_stack(&mut g);
    let back = named(&g, 0, "Evernight Shade");
    assert_eq!(back.len(), 1, "undying returns it");
    assert_eq!(pt(&g, back[0]), (2, 2));
}

/// Sacrifice another creature: two +1/+1 counters. Dies: an X/X Horror, X its
/// last-known power (CR 603.10a).
#[test]
fn flesh_carver_grows_and_leaves_a_horror() {
    let mut g = pod(2);
    let carver = g.add_card_to_battlefield(0, catalog::flesh_carver());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(carver).unwrap().keywords().contains(&Keyword::Intimidate));
    flood(&mut g, 0);
    activate(&mut g, carver, 0).expect("sacrifice another");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none());
    assert_eq!(pt(&g, carver), (4, 4));
    let ctx = EffectContext::for_spell(1, Some(Target::Permanent(carver)), 0, 0);
    g.resolve_effect(&Effect::Destroy { what: Selector::Target(0) }, &ctx).expect("destroy");
    drain_stack(&mut g);
    let horror = named(&g, 0, "Horror");
    assert_eq!(horror.len(), 1);
    assert_eq!(pt(&g, horror[0]), (4, 4));
}

/// Both halves: an edict for you and a chosen opponent, each who sacrificed
/// draws two; then you and a chosen opponent each reanimate one.
#[test]
fn infernal_offering_trades_with_an_opponent() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    for seat in 0..3 {
        for _ in 0..3 {
            g.add_card_to_library(seat, catalog::swamp());
        }
    }
    let offering = g.add_card_to_hand(0, catalog::infernal_offering());
    let hands: Vec<usize> = (0..3).map(|s| g.players[s].hand.len()).collect();
    cast(&mut g, offering, None, vec![], None).expect("cast");
    assert_eq!(g.players[0].hand.len(), hands[0] + 1, "cast one, drew two");
    let drew: Vec<usize> = (1..3).filter(|&s| g.players[s].hand.len() == hands[s] + 2).collect();
    assert_eq!(drew.len(), 1, "exactly one opponent traded");
    // Then both reanimate: every Bear is back.
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1);
    assert_eq!((1..3).map(|s| named(&g, s, "Grizzly Bears").len()).sum::<usize>(), 2);
}

/// Morbid — the cast trigger copies it (CR 707.10), so two creatures die.
#[test]
fn malicious_affliction_copies_itself_with_morbid() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b_ = g.add_card_to_battlefield(1, catalog::serra_angel());
    let black = g.add_card_to_battlefield(1, catalog::ob_nixilis_of_the_black_oath());
    let _ = black;
    let spell = g.add_card_to_hand(0, catalog::malicious_affliction());
    cast(&mut g, spell, Some(Target::Permanent(b_)), vec![], None).expect("cast");
    assert!(g.battlefield_find(b_).is_none());
    assert!(g.battlefield_find(a).is_some(), "no morbid, no copy");
    let c = g.add_card_to_battlefield(1, catalog::serra_angel());
    morbid(&mut g);
    let spell = g.add_card_to_hand(0, catalog::malicious_affliction());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: spell, target: Some(Target::Permanent(c)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    assert_eq!(g.stack.len(), 2, "the morbid cast trigger");
    drain_stack(&mut g);
    assert!(g.battlefield_find(c).is_none());
    assert!(g.battlefield_find(a).is_none(), "the copy took a new target");
}

/// Morbid ETB: −4/−4 to a creature; no death this turn, no trigger.
#[test]
fn morkrut_banshee_needs_morbid() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let banshee = g.add_card_to_hand(0, catalog::morkrut_banshee());
    cast(&mut g, banshee, None, vec![], None).expect("cast");
    assert!(g.battlefield_find(angel).is_some());
    morbid(&mut g);
    let banshee = g.add_card_to_hand(0, catalog::morkrut_banshee());
    cast(&mut g, banshee, None, vec![], None).expect("cast");
    assert!(g.battlefield_find(angel).is_none(), "−4/−4 kills the 4/4");
}

/// {B}: 1 damage to each creature and each player — every seat at a table.
#[test]
fn pestilence_demon_pings_everything() {
    let mut g = pod(3);
    let demon = g.add_card_to_battlefield(0, catalog::pestilence_demon());
    let elf = g.add_card_to_battlefield(2, catalog::llanowar_elves());
    flood(&mut g, 0);
    activate(&mut g, demon, 0).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elf).is_none());
    assert_eq!(g.battlefield_find(demon).unwrap().damage, 1);
    for seat in 0..3 {
        assert_eq!(g.players[seat].life, 19);
    }
}

/// Choose two (CR 700.2) — the default pair: target player loses X, target
/// creature gets −X/−X.
#[test]
fn profane_command_drains_and_shrinks() {
    let mut g = pod(3);
    let angel = g.add_card_to_battlefield(2, catalog::serra_angel());
    let cmd = g.add_card_to_hand(0, catalog::profane_command());
    cast(&mut g, cmd, Some(Target::Player(1)), vec![Target::Permanent(angel)], Some(4)).expect("cast");
    assert_eq!(g.players[1].life, 16);
    assert!(g.battlefield_find(angel).is_none());
}

/// Beginning of combat stamps a live opponent at random; the creature must
/// attack them (CR 508.1d); its combat damage halves their life.
#[test]
fn raving_dead_attacks_a_random_opponent_and_halves_their_life() {
    let mut g = pod(4);
    g.players[3].eliminated = true;
    let dead = g.add_card_to_battlefield(0, catalog::raving_dead());
    g.clear_sickness(dead);
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let q = g.battlefield_find(dead).unwrap().chosen_player.expect("an opponent was chosen");
    assert!(q == 1 || q == 2, "a live opponent, never a departed seat");
    let other = 3 - q;
    let snapshot = g.clone();
    assert!(declare(&mut g, vec![(dead, other)]).is_err());
    let mut g = snapshot;
    declare(&mut g, vec![(dead, q)]).expect("attack the chosen");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = q;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[q].life, 9, "20 − 2 = 18, then half of 18 is lost");
}

/// Morbid, at each end step: destroy a non-Demon creature.
#[test]
fn reaper_from_the_abyss_reaps_non_demons() {
    let mut g = pod(2);
    g.players[0].hostile_player_targets = true;
    g.add_card_to_battlefield(0, catalog::reaper_from_the_abyss());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "no death, no trigger");
    morbid(&mut g);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(named(&g, 0, "Reaper from the Abyss").len(), 1, "never itself");
}

/// Morbid-gated: {T} and tap two untapped creatures for a 5/5 Demon.
#[test]
fn skirsdag_high_priest_needs_a_death() {
    let mut g = pod(2);
    let priest = g.add_card_to_battlefield(0, catalog::skirsdag_high_priest());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b_ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [priest, a, b_] {
        g.clear_sickness(id);
    }
    assert!(activate(&mut g, priest, 0).is_err(), "activate only if a creature died");
    morbid(&mut g);
    activate(&mut g, priest, 0).expect("morbid");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Demon").len(), 1);
    for id in [priest, a, b_] {
        assert!(g.battlefield_find(id).unwrap().tapped);
    }
}

/// Split second; the target player's creatures are 0/2 with no abilities.
#[test]
fn sudden_spoiling_blanks_a_players_creatures() {
    let def = catalog::sudden_spoiling();
    assert!(def.keywords.contains(&Keyword::SplitSecond));
    let mut g = pod(3);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel());
    let spoil = g.add_card_to_hand(0, def);
    cast(&mut g, spoil, Some(Target::Player(1)), vec![], None).expect("cast");
    assert_eq!(pt(&g, angel), (0, 2));
    assert!(!g.computed_permanent(angel).unwrap().keywords().contains(&Keyword::Flying));
    assert_eq!(pt(&g, mine), (4, 4), "only that player's creatures");
}

/// Only during combat on an opponent's turn: X creature cards back, each
/// sacrificed at the next end step.
#[test]
fn wake_the_dead_raises_a_temporary_army() {
    let mut g = pod(2);
    let a = g.add_card_to_graveyard(0, catalog::serra_angel());
    let b_ = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let wake = g.add_card_to_hand(0, catalog::wake_the_dead());
    let targets = || (Some(Target::Permanent(a)), vec![Target::Permanent(b_)]);
    g.step = TurnStep::DeclareAttackers;
    let (t, extra) = targets();
    assert!(cast(&mut g, wake, t, extra, Some(2)).is_err(), "not on your own turn");
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    let (t, extra) = targets();
    cast(&mut g, wake, t, extra, Some(2)).expect("combat on an opponent's turn");
    assert_eq!(named(&g, 0, "Serra Angel").len(), 1);
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(named(&g, 0, "Serra Angel").is_empty() && named(&g, 0, "Grizzly Bears").is_empty());
}

/// Upkeep: sacrifice another creature and each opponent loses its power; with
/// nothing else, tap it and lose 7.
#[test]
fn xathrid_demon_feeds_or_bites() {
    let mut g = pod(3);
    let demon = g.add_card_to_battlefield(0, catalog::xathrid_demon());
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none());
    assert!(g.battlefield_find(demon).is_some(), "never itself");
    assert_eq!((g.players[1].life, g.players[2].life), (16, 16));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(demon).unwrap().tapped);
    assert_eq!(g.players[0].life, 13);
}
