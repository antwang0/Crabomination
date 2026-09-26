//! Commander: the Family Matters precon (BLC, Zinnia, `decks::cmdr_zinnia`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = if n == 2 { two_player_game() } else { multi_player_game(n) };
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

fn cast_full(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>, kicked: bool) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    let target = targets.first().cloned();
    let additional_targets = targets.iter().skip(1).cloned().collect();
    let action = if kicked {
        GameAction::CastSpellKicked { card_id: id, target, additional_targets, mode: None, x_value: x }
    } else {
        GameAction::CastSpell { card_id: id, target, additional_targets, mode: None, x_value: x }
    };
    g.perform_action(action).expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_full(g, id, targets, None, false);
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn to_combat_end(g: &mut GameState) {
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn attack(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    let attacks =
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect();
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("attack");
    drain_stack(g);
}

/// CR 702.175 — Zinnia grants offspring {2}: a creature cast kicked makes a
/// 1/1 token copy, and that base-power-1 copy pumps Zinnia. A flickered
/// creature is a new object (CR 400.7) and makes no second copy.
#[test]
fn zinnia_grants_offspring() {
    let mut g = pod(2);
    let z = g.add_card_to_battlefield(0, catalog::zinnia_valleys_voice());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_full(&mut g, bear, &[], None, true);
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 2, "the bear and its offspring");
    let token = named(&g, 0, "Grizzly Bears").into_iter().find(|&id| id != bear).unwrap();
    assert_eq!(g.computed_permanent(token).map(|c| (c.power, c.toughness)), Some((1, 1)));
    assert_eq!(g.computed_permanent(z).unwrap().power, 2, "+1 for the 1/1 copy");
    let cs = g.add_card_to_hand(0, catalog::cloudshift());
    cast(&mut g, cs, &[Target::Permanent(bear)]);
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 2, "no copy from the flicker");
    // Unkicked, no copy.
    let bear2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear2, &[]);
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 3);
}

/// Fortune Teller's Talent: level 2 opens the library top once a spell has
/// been cast this turn (CR 401.6); level 3 takes {2} off that cast.
#[test]
fn fortune_tellers_talent_levels() {
    let mut g = pod(2);
    let ft = g.add_card_to_battlefield(0, catalog::fortune_tellers_talent());
    let top = g.add_card_to_library(0, catalog::craw_wurm());
    g.battlefield_find_mut(ft).unwrap().class_level = 2;
    assert!(!g.library_top_playable(0, top), "no spell cast yet");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]);
    assert!(g.library_top_playable(0, top));
    g.battlefield_find_mut(ft).unwrap().class_level = 3;
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: top,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("a six-drop off the top for four");
    drain_stack(&mut g);
    assert!(g.battlefield_find(top).is_some());
}

/// Arthur and another creature attack: a creature from the top six joins
/// the attack (CR 508.4) and goes back to hand at end of combat.
#[test]
fn arthur_deploys_and_recalls() {
    let mut g = pod(2);
    let arthur = g.add_card_to_battlefield(0, catalog::arthur_marigold_knight());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_library(0, catalog::craw_wurm());
    g.add_card_to_library(0, catalog::island());
    attack(&mut g, &[arthur, bear], 1);
    assert!(g.attacking().iter().any(|a| a.attacker == wurm), "the Wurm attacks");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    to_combat_end(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == wurm), "returned at end of combat");
    assert_eq!(g.players[1].life, 20 - 4 - 2 - 6);
}

/// Shield Broker steals while the shield counter stays (CR 611.2c); the
/// shield spent on damage (CR 122.1c) hands the creature back.
#[test]
fn shield_broker_steals_until_the_shield_breaks() {
    let mut g = pod(2);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let sb = g.add_card_to_hand(0, catalog::shield_broker());
    cast(&mut g, sb, &[Target::Permanent(wurm)]);
    assert_eq!(g.battlefield_find(wurm).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(wurm).unwrap().counter_count(CounterType::Shield), 1);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(wurm)]);
    let w = g.battlefield_find(wurm).expect("the shield prevented the damage");
    assert_eq!(w.counter_count(CounterType::Shield), 0);
    assert_eq!(w.controller, 1, "back to its owner");
}

/// Echoing Assault: the 1/1 copy attacks the player its original attacks
/// (CR 508.4), and its creature tokens have menace.
#[test]
fn echoing_assault_copies_an_attacker() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::echoing_assault());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    attack(&mut g, &[wurm], 2);
    let copy = named(&g, 0, "Craw Wurm").into_iter().find(|&id| id != wurm).expect("copy");
    let a = g.attacking().iter().find(|a| a.attacker == copy).map(|a| a.target);
    assert_eq!(a, Some(AttackTarget::Player(2)));
    let cp = g.computed_permanent(copy).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.keywords().contains(&Keyword::Menace));
}

/// Echoing Assault triggers once for each player attacked (CR 603.2, "whenever
/// you attack a player"), each copy of a creature attacking that player.
#[test]
fn echoing_assault_copies_one_attacker_per_player_attacked() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::echoing_assault());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for a in [wurm, bear] {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: wurm, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    let copy_of = |g: &GameState, name: &str, orig: CardId| {
        let id = named(g, 0, name).into_iter().find(|&id| id != orig).expect("a copy");
        g.attacking().iter().find(|a| a.attacker == id).map(|a| a.target)
    };
    assert_eq!(copy_of(&g, "Craw Wurm", wurm), Some(AttackTarget::Player(1)));
    assert_eq!(copy_of(&g, "Grizzly Bears", bear), Some(AttackTarget::Player(2)));
}

/// Combat Celebrant exerted (CR 701.43d): the other creatures untap and an
/// additional combat follows (CR 505.1a).
#[test]
fn combat_celebrant_buys_a_second_combat() {
    let mut g = pod(2);
    let cc = g.add_card_to_battlefield(0, catalog::combat_celebrant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(cc);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackersExerting {
        attacks: vec![
            Attack { attacker: cc, target: AttackTarget::Player(1) },
            Attack { attacker: bear, target: AttackTarget::Player(1) },
        ],
        exert: vec![cc],
    })
    .expect("attack");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
    assert!(g.battlefield_find(cc).unwrap().tapped, "not itself");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    let mut steps = 0;
    while g.step != TurnStep::DeclareAttackers && steps < 10 {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
        steps += 1;
    }
    assert_eq!(g.step, TurnStep::DeclareAttackers, "a second combat");
}

/// Storm of Souls returns every creature card as a 1/1 flying Spirit in
/// addition to its types (CR 205.1b), then exiles itself.
#[test]
fn storm_of_souls_returns_spirits() {
    let mut g = pod(2);
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::storm_of_souls());
    cast(&mut g, s, &[]);
    for id in [wurm, bear] {
        let cp = g.computed_permanent(id).expect("returned");
        assert_eq!((cp.power, cp.toughness), (1, 1));
        assert!(cp.keywords().contains(&Keyword::Flying));
    }
    assert!(g.exile.iter().any(|c| c.id == s));
}

/// Stolen by the Fae with X = 2 bounces a mana-value-2 creature and makes two
/// Faeries.
#[test]
fn stolen_by_the_fae_trades_for_faeries() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::stolen_by_the_fae());
    cast_full(&mut g, s, &[Target::Permanent(bear)], Some(2), false);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
    assert_eq!(named(&g, 0, "Faerie").len(), 2);
}

/// Boss's Chauffeur enters with one plus your other creatures, and dying
/// leaves a Citizen per counter.
#[test]
fn bosss_chauffeur_counts_the_family() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bc = g.add_card_to_hand(0, catalog::bosss_chauffeur());
    cast(&mut g, bc, &[]);
    assert_eq!(g.battlefield_find(bc).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    let s = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, s, &[Target::Permanent(bc)]);
    assert_eq!(named(&g, 0, "Citizen").len(), 3);
}

/// Rose Room Treasurer: the first two alliance triggers a turn make Treasure.
#[test]
fn rose_room_treasurer_banks_two_treasures() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::rose_room_treasurer());
    for _ in 0..3 {
        let b = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast(&mut g, b, &[]);
    }
    assert_eq!(named(&g, 0, "Treasure").len(), 2);
}

/// Jacked Rabbit with X = 5: five counters and a card (ravenous, CR 702.156);
/// attacking, a Rabbit per point of power.
#[test]
fn jacked_rabbit_ravenous_and_rabbits() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let jr = g.add_card_to_hand(0, catalog::jacked_rabbit());
    cast_full(&mut g, jr, &[], Some(5), false);
    assert_eq!(g.battlefield_find(jr).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
    assert_eq!(g.players[0].hand.len(), 1);
    attack(&mut g, &[jr], 1);
    assert_eq!(named(&g, 0, "Rabbit").len(), 6);
}

/// Murmuration: a Storm Crow per spell cast this turn at your end step, and
/// the Birds get +1/+1.
#[test]
fn murmuration_counts_spells() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::murmuration());
    for _ in 0..2 {
        let b = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, b, &[Target::Player(1)]);
    }
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let crows = named(&g, 0, "Storm Crow");
    assert_eq!(crows.len(), 2);
    assert_eq!(g.computed_permanent(crows[0]).map(|c| (c.power, c.toughness)), Some((2, 3)));
}

/// Tetsuko: power or toughness 1 or less can't be blocked (CR 509.1b), read
/// off the attacker's stats as blocks are declared; a 2/2 can be.
#[test]
fn tetsuko_frees_the_small() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::tetsuko_umezawa_fugitive());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_wood());
    attack(&mut g, &[elf, bear], 1);
    assert!(!g.blocker_can_block_attacker(wall, elf));
    assert!(g.blocker_can_block_attacker(wall, bear));
}

/// Rapid Augmenter: a creature that enters without being cast grows it; a
/// base-power-1 creature gains haste.
#[test]
fn rapid_augmenter_rewards_uncast_entries() {
    let mut g = pod(2);
    let ra = g.add_card_to_battlefield(0, catalog::rapid_augmenter());
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    cast(&mut g, elf, &[]);
    assert!(g.computed_permanent(elf).unwrap().keywords().contains(&Keyword::Haste));
    assert_eq!(g.battlefield_find(ra).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "the elf was cast");
    let b = g.add_card_to_hand(0, catalog::raise_the_alarm());
    cast(&mut g, b, &[]);
    assert_eq!(g.battlefield_find(ra).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Devilish Valet doubles its power per alliance trigger.
#[test]
fn devilish_valet_doubles() {
    let mut g = pod(2);
    let dv = g.add_card_to_battlefield(0, catalog::devilish_valet());
    let b = g.add_card_to_hand(0, catalog::raise_the_alarm());
    cast(&mut g, b, &[]);
    assert_eq!(g.computed_permanent(dv).unwrap().power, 4);
}

/// Calamity of Cinders spares the tapped.
#[test]
fn calamity_of_cinders_hits_untapped() {
    let mut g = pod(2);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let c = g.add_card_to_hand(0, catalog::calamity_of_cinders());
    cast(&mut g, c, &[]);
    assert!(g.battlefield_find(wurm).is_none());
    assert!(g.battlefield_find(bear).is_some());
}

/// Agate Instigator's printed offspring, and its alliance ping at each
/// opponent.
#[test]
fn agate_instigator_offspring_pings() {
    let mut g = pod(3);
    let a = g.add_card_to_hand(0, catalog::agate_instigator());
    cast_full(&mut g, a, &[], None, true);
    assert_eq!(named(&g, 0, "Agate Instigator").len(), 2);
    // The copy's entry pings each opponent once.
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.players[2].life, 19);
}

/// Pollywog Prodigy draws off an opponent's cheap noncreature spell.
#[test]
fn pollywog_prodigy_draws_off_cheap_spells() {
    let mut g = pod(2);
    let pp = g.add_card_to_battlefield(0, catalog::pollywog_prodigy());
    g.battlefield_find_mut(pp).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
}
