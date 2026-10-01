//! Cards that name a commander (`decks::cmdr_familiars`, COMMANDER_BACKLOG
//! §3): the partner familiars, Lozhan, the Backgrounds, Astarion's Thirst and
//! Mines of Moria.

use crabomination::card::{CardId, CounterType, Keyword, KeywordSlice};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    g
}

/// A creature on the battlefield that is `seat`'s commander (CR 903.3).
fn commander(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.players[seat].commanders.push(id);
    g.clear_sickness(id);
    id
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

fn attack(g: &mut GameState, attacker: CardId, seat: usize) {
    advance_to(g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(seat) }]))
        .expect("attack");
    drain_stack(g);
}

fn lives(g: &GameState) -> Vec<i32> {
    g.players.iter().map(|p| p.life).collect()
}

/// CR 903.3 / 510.2 — Kediss: a commander's combat damage to one opponent
/// is dealt again, by the commander, to each other opponent.
#[test]
fn kediss_splashes_commander_damage_to_each_other_opponent() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::kediss_emberclaw_familiar());
    let bear = commander(&mut g, 0, catalog::grizzly_bears());
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(plain);
    attack(&mut g, bear, 1);
    advance_to(&mut g, TurnStep::EndCombat);
    drain_stack(&mut g);
    assert_eq!(lives(&g), vec![20, 18, 18], "2 to the defender, 2 to the other opponent");
}

/// CR 601.2f — Esior: an opponent's spell targeting your commander costs {3}
/// more; one aimed elsewhere doesn't.
#[test]
fn esior_taxes_spells_targeting_your_commander() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::esior_wardwing_familiar());
    let cmdr = commander(&mut g, 0, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let cast = |g: &mut GameState, t| {
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(t)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(cast(&mut g, cmdr).is_err(), "{{R}} alone can't pay the tax");
    cast(&mut g, other).expect("a non-commander target is untaxed");
}

/// Anara: commanders you control are indestructible during your turn only.
#[test]
fn anara_shields_your_commanders_on_your_turn() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::anara_wolvid_familiar());
    let cmdr = commander(&mut g, 0, catalog::grizzly_bears());
    let has = |g: &GameState| g.computed_permanent(cmdr).unwrap().keywords().has_kw(&Keyword::Indestructible);
    assert!(has(&g), "your turn");
    g.active_player_idx = 1;
    assert!(!has(&g), "an opponent's turn");
}

/// Agent of the Iron Throne: your commander gains the drain trigger; a
/// creature of yours dying makes each opponent lose 1.
#[test]
fn agent_of_the_iron_throne_drains_on_your_deaths() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::agent_of_the_iron_throne());
    commander(&mut g, 0, catalog::grizzly_bears());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(fodder)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(lives(&g), vec![20, 19, 19]);
}

/// Inspiring Leader: while you control a commander creature you own, your
/// creature tokens get +2/+2.
#[test]
fn inspiring_leader_pumps_tokens_while_your_commander_is_out() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::inspiring_leader());
    let token = g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    let _ = token;
    let spirit = {
        let mut t = crabomination::card::TokenDefinition {
            name: "Spirit".into(),
            card_types: vec![crabomination::card::CardType::Creature],
            power: 1,
            toughness: 1,
            ..Default::default()
        };
        t.colors = vec![Color::White];
        g.add_token_to_battlefield(0, &t)
    };
    assert_eq!(g.computed_permanent(spirit).unwrap().power, 1, "no commander yet");
    commander(&mut g, 0, catalog::grizzly_bears());
    let cp = g.computed_permanent(spirit).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Cast a Grizzly Bears for `seat` — from the command zone as its commander
/// when `commander`, else from hand.
fn cast_bears(g: &mut GameState, seat: usize, commander: bool) -> CardId {
    g.priority.player_with_priority = seat;
    g.players[seat].mana_pool.add(Color::Green, 2);
    let id = if commander {
        let id = g.seat_commanders(seat, vec![catalog::grizzly_bears()])[0];
        g.perform_action(GameAction::CastFromCommandZone {
            card_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
            alternative: false,
            pitch_card: None,
        })
        .map(|_| id)
    } else {
        let id = g.add_card_to_hand(seat, catalog::grizzly_bears());
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .map(|_| id)
    }
    .expect("cast bears");
    drain_stack(g);
    id
}

/// CR 614.1c / 614.12 — Master Chef: the commander enters with an extra
/// +1/+1 counter; other creatures get one only while that commander is out.
#[test]
fn master_chef_counters_the_commander_then_the_rest() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::master_chef());
    let before = cast_bears(&mut g, 0, false);
    assert_eq!(g.battlefield_find(before).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "no commander out");
    let cmdr = cast_bears(&mut g, 0, true);
    assert_eq!(g.battlefield_find(cmdr).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "its own rider only");
    let after = cast_bears(&mut g, 0, false);
    assert_eq!(g.battlefield_find(after).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    g.active_player_idx = 1;
    let theirs = cast_bears(&mut g, 1, false);
    assert_eq!(g.battlefield_find(theirs).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "yours only");
}

/// CR 702.16 / 608.2d — Noble Heritage: as the commander enters each player
/// may put two counters on a creature of theirs; each opponent who does gives
/// you protection from them until your next turn.
#[test]
fn noble_heritage_trades_counters_for_protection() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::noble_heritage());
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    // Seat 0's commander is the next card id: yes + pick it, yes + seat 1's
    // bear, then seat 2 declines.
    let cmdr = CardId(b2.0 + 1);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Cards(vec![cmdr]),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Cards(vec![b1]),
        DecisionAnswer::Bool(false),
    ]));
    assert_eq!(cast_bears(&mut g, 0, true), cmdr);
    let pp = |g: &GameState, id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!((pp(&g, cmdr), pp(&g, b1), pp(&g, b2)), (2, 2, 0));
    assert_eq!(g.players[0].protected_from_seats_until_next_turn, 1 << 1, "only from seat 1, who took the deal");
}

/// Tavern Brawler: at your upkeep your commander exiles your top card and gets
/// +X/+0 for its mana value.
#[test]
fn tavern_brawler_pumps_by_the_exiled_cards_mana_value() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::tavern_brawler());
    let cmdr = commander(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::Untap;
    let evs = g.advance_step(Vec::new()).unwrap_or_default();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.step, TurnStep::Upkeep);
    assert_eq!(g.computed_permanent(cmdr).unwrap().power, 4, "Grizzly Bears exiled: +2/+0");
    assert!(g.exile.iter().any(|c| c.owner == 0 && c.may_play_until.is_some()), "playable this turn");
}

/// Far Traveler: at your end step the commander blinks a tapped creature
/// you control (its counters are gone — a new object, CR 400.7).
#[test]
fn far_traveler_blinks_a_tapped_creature_at_end_step() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::far_traveler());
    commander(&mut g, 0, catalog::grizzly_bears());
    let tired = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.battlefield_find_mut(tired).unwrap().tapped = true;
    g.battlefield_find_mut(tired).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    let giant = g.battlefield.iter().find(|c| c.definition.name == "Hill Giant").expect("back");
    assert!(!giant.tapped, "returned untapped");
    assert_eq!(giant.counter_count(CounterType::PlusOnePlusOne), 0, "a new object");
}

/// Guild Artisan: attacking a player no opponent out-lives makes two
/// Treasures; attacking a lower-life opponent doesn't.
#[test]
fn guild_artisan_rewards_attacking_the_life_leader() {
    let treasures = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count();
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::guild_artisan());
    let cmdr = commander(&mut g, 0, catalog::grizzly_bears());
    g.players[2].life = 25;
    attack(&mut g, cmdr, 1);
    assert_eq!(treasures(&g), 0, "seat 2 has more life than seat 1");

    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::guild_artisan());
    let cmdr = commander(&mut g, 0, catalog::grizzly_bears());
    g.players[0].life = 30; // your own life doesn't count
    g.players[2].life = 15;
    attack(&mut g, cmdr, 1);
    assert_eq!(treasures(&g), 2);
}

/// Astarion's Thirst: X +1/+1 counters on your commander, X = the exiled
/// creature's power.
#[test]
fn astarions_thirst_feeds_your_commander() {
    let mut g = pod(3);
    let cmdr = commander(&mut g, 0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::astarions_thirst());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == giant));
    assert_eq!(g.battlefield_find(cmdr).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Lozhan: casting a Dragon spell deals its mana value to a target that
/// isn't a commander.
#[test]
fn lozhan_burns_on_a_dragon_spell() {
    let mut g = pod(3);
    g.players[0].hostile_player_targets = true;
    g.add_card_to_battlefield(0, catalog::lozhan_dragons_legacy());
    let dragon = g.add_card_to_hand(0, catalog::shivan_dragon());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    let before: i32 = lives(&g).iter().sum();
    g.perform_action(GameAction::CastSpell {
        card_id: dragon,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Shivan Dragon");
    drain_stack(&mut g);
    assert_eq!(lives(&g).iter().sum::<i32>(), before - 6, "six to a player");
    assert_eq!(g.players[0].life, 20, "not the caster");
}

/// Mines of Moria enters tapped unless you control a legendary creature.
#[test]
fn mines_of_moria_enters_tapped_without_a_legend() {
    let mut g = pod(3);
    let mines = g.add_card_to_hand(0, catalog::mines_of_moria());
    g.perform_action(GameAction::PlayLand(mines)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mines).unwrap().tapped);
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::lozhan_dragons_legacy());
    let mines = g.add_card_to_hand(0, catalog::mines_of_moria());
    g.perform_action(GameAction::PlayLand(mines)).expect("play");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(mines).unwrap().tapped);
}
