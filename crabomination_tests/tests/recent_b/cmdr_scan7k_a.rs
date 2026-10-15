//! Commander: one-card gaps of the 2026-10-15 full-list EDHREC scan
//! (`decks::cmdr_scan7k_a`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(g: &mut GameState, id: CardId) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: index, target, additional_targets: vec![], x_value: None, mode: None })
        .expect("activate");
    drain_stack(g);
}

fn step_to(g: &mut GameState, seat: usize, step: TurnStep) {
    while g.active_player_idx != seat || g.step != step {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

fn named_count(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn loyalty(g: &mut GameState, pw: CardId, index: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: pw, ability_index: index, target, x_value: None })
        .expect("loyalty ability");
    drain_stack(g);
}

/// Elspeth, Knight-Errant's −8 emblem makes your lands and creatures
/// indestructible, not an opponent's.
#[test]
fn elspeth_knight_errant_emblem_grants_indestructible() {
    let mut g = pod(2);
    let pw = ready(&mut g, 0, catalog::elspeth_knight_errant());
    g.battlefield_find_mut(pw).unwrap().add_counters(CounterType::Loyalty, 6);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let land = ready(&mut g, 0, catalog::forest());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    loyalty(&mut g, pw, 2, None);
    for id in [bear, land] {
        assert!(g.permanent_has_keyword(id, &Keyword::Indestructible));
    }
    assert!(!g.permanent_has_keyword(theirs, &Keyword::Indestructible));
}

/// Nighthawk triggers on another Hero entering, not on a non-Hero.
#[test]
fn nighthawk_pumps_when_a_hero_enters() {
    let mut g = pod(2);
    let hawk = ready(&mut g, 0, catalog::nighthawk_dark_defender());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    let elves = g.add_card_to_hand(0, catalog::llanowar_elves());
    cast(&mut g, elves);
    assert_eq!(pt(&g, bear), (2, 2), "an Elf is no Hero");
    let cap = g.add_card_to_hand(0, catalog::captain_america_super_soldier());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    cast(&mut g, cap);
    let total: i32 = [hawk, bear, cap].iter().map(|id| pt(&g, *id).0).sum();
    assert_eq!(total, 2 + 2 + 3 + 1, "one +1/+1 for the Hero");
}

/// Training Regimen counters a creature at your beginning of combat, and the
/// countered creature has trample.
#[test]
fn training_regimen_counters_at_combat_and_grants_trample() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::training_regimen());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(!g.permanent_has_keyword(bear, &Keyword::Trample));
    step_to(&mut g, 0, TurnStep::BeginCombat);
    assert_eq!(plus(&g, bear), 1);
    assert!(g.permanent_has_keyword(bear, &Keyword::Trample));
}

/// Heroes for Hire makes three Treasures and turns one into an impulse draw.
#[test]
fn heroes_for_hire_sacrifices_treasure_to_impulse() {
    let mut g = pod(2);
    let top = g.add_card_to_library(0, catalog::island());
    let hfh = g.add_card_to_hand(0, catalog::heroes_for_hire());
    cast(&mut g, hfh);
    assert_eq!(named_count(&g, 0, "Treasure"), 3);
    activate(&mut g, hfh, 0, None);
    assert_eq!(named_count(&g, 0, "Treasure"), 2);
    assert!(g.exile.iter().any(|c| c.id == top), "the top card is exiled to play");
}

/// Greatsword of Tyr: the equipped attacker gets a counter and taps a
/// creature the defending player controls.
#[test]
fn greatsword_of_tyr_counters_and_taps_a_defender() {
    let mut g = pod(3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sword = ready(&mut g, 0, catalog::greatsword_of_tyr());
    flood(&mut g);
    g.perform_action(GameAction::Equip { equipment: sword, target: bear }).expect("equip {W}");
    let blocker = ready(&mut g, 2, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(blocker))]));
    declare(&mut g, vec![(bear, 2)]);
    assert_eq!(plus(&g, bear), 1);
    assert!(g.battlefield_find(blocker).unwrap().tapped);
}

/// Livaan pumps by the noncreature spell's mana value and ignores creatures.
#[test]
fn livaan_pumps_by_the_spells_mana_value() {
    let mut g = pod(2);
    let liv = ready(&mut g, 0, catalog::livaan_cultist_of_tiamat());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear);
    assert_eq!(pt(&g, liv).0 + pt(&g, bear).0, 1 + 2, "a creature spell: no trigger");
    let div = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, div);
    assert_eq!(pt(&g, liv).0 + pt(&g, bear).0, 1 + 2 + 3, "+X/+0, X = 3");
}

/// The Black Arrow's 1 damage destroys a Dragon, whatever its toughness.
#[test]
fn the_black_arrow_slays_a_dragon() {
    let mut g = pod(2);
    let dragon = ready(&mut g, 1, catalog::shivan_dragon());
    ready(&mut g, 1, catalog::leyline_of_sanctity()); // the Dragon is the one hostile target
    let arrow = g.add_card_to_hand(0, catalog::the_black_arrow());
    cast(&mut g, arrow);
    assert!(g.battlefield_find(dragon).is_none());
}

/// Captain America shields his controller from an opponent's targeting
/// while he keeps his shield counter.
#[test]
fn captain_america_gives_hexproof_while_shielded() {
    let mut g = pod(2);
    let cap = g.add_card_to_hand(0, catalog::captain_america_super_soldier());
    cast(&mut g, cap);
    assert_eq!(g.battlefield_find(cap).unwrap().counter_count(CounterType::Shield), 1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "you have hexproof");
}

/// Meager Meal counters a creature and gains a target player 2 life, then
/// Gollum is castable from exile.
#[test]
fn gollum_meager_meal_counters_and_gains_life() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let gollum = g.add_card_to_hand(0, catalog::gollum_silent_slinker());
    flood(&mut g);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastAdventure {
        card_id: gollum,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Player(0)],
        mode: None,
        x_value: None,
    })
    .expect("adventure");
    drain_stack(&mut g);
    assert_eq!(plus(&g, bear), 1);
    assert_eq!(g.players[0].life, life + 2);
    assert!(g.exile.iter().any(|c| c.id == gollum));
}

/// Origin of Thor's chapter III has your creature deal its power to each
/// opponent.
#[test]
fn origin_of_thor_chapter_three_hits_each_opponent() {
    let mut g = pod(3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let saga = ready(&mut g, 0, catalog::origin_of_thor());
    for seat in 0..3 {
        for _ in 0..3 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    let before = [g.players[1].life, g.players[2].life];
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::PostCombatMain);
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    assert_eq!([g.players[1].life, g.players[2].life], [before[0] - 2, before[1] - 2]);
}

/// Triskaidekaphobia: a player at exactly 13 loses at your upkeep; the rest
/// gain 1.
#[test]
fn triskaidekaphobia_kills_at_thirteen() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::triskaidekaphobia());
    for seat in 0..3 {
        for _ in 0..3 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g.players[1].life = 13;
    g.players[2].life = 14;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    step_to(&mut g, 1, TurnStep::Upkeep);
    step_to(&mut g, 0, TurnStep::Draw);
    assert!(g.players[1].eliminated);
    assert_eq!(g.players[2].life, 15);
}

/// CR 903 pods: Avatar of Growth costs {1} less per opponent, and each
/// player fetches two basics.
#[test]
fn avatar_of_growth_discounts_per_opponent_and_ramps_everyone() {
    let mut g = pod(4);
    for seat in 0..4 {
        for _ in 0..3 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    let avatar = g.add_card_to_hand(0, catalog::avatar_of_growth());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: avatar, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{4}{G}{G} less three");
    drain_stack(&mut g);
    for seat in 0..4 {
        assert_eq!(named_count(&g, seat, "Forest"), 2, "seat {seat}");
    }
}

/// Dragonrage adds {R} per attacker and grants each a firebreathing ability.
#[test]
fn dragonrage_adds_red_per_attacker() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(a, 1), (b, 1)]);
    let rage = g.add_card_to_hand(0, catalog::dragonrage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: rage, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 2);
    activate(&mut g, a, 0, None);
    assert_eq!(pt(&g, a), (3, 2), "{{R}}: +1/+0 until end of turn");
}

/// Thrull Wizard's tax is "{B} or {3}": three colorless saves the black
/// spell, nothing at all doesn't.
#[test]
fn thrull_wizard_taxes_black_spells_b_or_three() {
    for (floating, paid) in [(3, true), (0, false)] {
        let mut g = pod(2);
        let wiz = ready(&mut g, 0, catalog::thrull_wizard());
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        let kill = g.add_card_to_hand(1, catalog::doom_blade());
        g.players[1].mana_pool.add(Color::Black, 2);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: kill, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("Doom Blade");
        g.players[1].mana_pool.add_colorless(floating);
        g.players[0].mana_pool.add(Color::Black, 2);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: wiz, ability_index: 0, target: Some(Target::Permanent(kill)), additional_targets: vec![], x_value: None, mode: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bear).is_none(), paid, "{floating} floating");
    }
}

// ── decks::cmdr_scan7k_b ─────────────────────────────────────────────────────

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

/// Armaggon's entry destroys up to three creatures — the opponents'.
#[test]
fn armaggon_destroys_up_to_three() {
    let mut g = pod(3);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs: Vec<CardId> = (0..3).map(|i| ready(&mut g, 1 + i % 2, catalog::grizzly_bears())).collect();
    let shark = g.add_card_to_hand(0, catalog::armaggon_future_shark());
    cast(&mut g, shark);
    assert!(g.battlefield_find(mine).is_some());
    assert!(theirs.iter().all(|id| g.battlefield_find(*id).is_none()));
}

/// Shark Shredder's hit puts a creature card from the damaged player's
/// graveyard onto the battlefield under its controller, tapped and attacking.
#[test]
fn shark_shredder_steals_from_the_graveyard_it_hit() {
    let mut g = pod(3);
    let shark = ready(&mut g, 0, catalog::shark_shredder_killer_clone());
    let corpse = g.add_card_to_graveyard(2, catalog::serra_angel());
    declare(&mut g, vec![(shark, 2)]);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat && g.battlefield_find(corpse).is_none() {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    let angel = g.battlefield_find(corpse).expect("reanimated");
    assert_eq!(angel.controller, 0);
    assert!(angel.tapped);
    assert!(g.attacking().iter().any(|a| a.attacker == corpse && a.target == AttackTarget::Player(2)));
}

/// Songbird discards for flying.
#[test]
fn songbird_discards_for_flying() {
    let mut g = pod(2);
    let bird = ready(&mut g, 0, catalog::songbird_sonic_screamer());
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, bird, 0, None);
    assert!(g.permanent_has_keyword(bird, &Keyword::Flying));
    assert!(g.players[0].hand.is_empty());
}

/// Unesh's entry splits the top four between hand and graveyard, and Sphinx
/// spells cost {2} less after it.
#[test]
fn unesh_splits_four_and_discounts_sphinxes() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    let unesh = g.add_card_to_hand(0, catalog::unesh_criosphinx_sovereign());
    cast(&mut g, unesh);
    assert_eq!(g.players[0].hand.len() + g.players[0].graveyard.len(), 4);
    assert_eq!(g.players[0].library.len(), 2);
    let jwar = g.add_card_to_hand(0, catalog::sphinx_of_jwar_isle());
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: jwar, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{4}{U}{U} less two");
}

/// Teferi, Timebender's −3: 2 life and two cards.
#[test]
fn teferi_timebender_minus_three_draws_two() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let pw = ready(&mut g, 0, catalog::teferi_timebender());
    g.battlefield_find_mut(pw).unwrap().add_counters(CounterType::Loyalty, 5);
    let life = g.players[0].life;
    loyalty(&mut g, pw, 1, None);
    assert_eq!(g.players[0].life, life + 2);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Cartouche of Knowledge draws as it enters and gives flying and +1/+1.
#[test]
fn cartouche_of_knowledge_draws_and_grants_flying() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::cartouche_of_knowledge());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("aura");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (3, 3));
    assert!(g.permanent_has_keyword(bear, &Keyword::Flying));
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Reptil's {3} form: a 3/5 Dinosaur Hero with vigilance and reach this turn.
#[test]
fn reptil_becomes_a_brontosaurus() {
    let mut g = pod(2);
    let reptil = ready(&mut g, 0, catalog::reptil_dinomorpher());
    activate(&mut g, reptil, 0, None);
    assert_eq!(pt(&g, reptil), (3, 5));
    assert!(g.permanent_has_keyword(reptil, &Keyword::Reach));
    assert!(g.permanent_has_keyword(reptil, &Keyword::Vigilance));
    let c = g.computed_permanent(reptil).unwrap();
    assert!(c.subtypes().creature_types.contains(&crabomination::card::CreatureType::Dinosaur));
    assert!(!c.subtypes().creature_types.contains(&crabomination::card::CreatureType::Human), "becomes, not in addition");
}

/// Roaring Primadox returns one of your creatures at your upkeep.
#[test]
fn roaring_primadox_bounces_at_upkeep() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    library(&mut g, 1, 2);
    ready(&mut g, 0, catalog::roaring_primadox());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::Draw);
    let bf = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf, 1, "one of the two went home");
    let _ = bear;
}

/// Spider-Man, Brooklyn Visionary cast for its web-slinging cost bounces a
/// tapped creature and fetches a tapped basic.
#[test]
fn spider_man_brooklyn_visionary_web_slings_and_ramps() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::forest());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spidey = g.add_card_to_hand(0, catalog::spider_man_brooklyn_visionary());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: spidey, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("web-slinging {2}{G}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    let forest = g.battlefield.iter().find(|c| c.definition.name == "Forest").expect("fetched");
    assert!(forest.tapped);
}

/// That's Rough Buddy puts two counters once a creature of yours has left
/// the battlefield this turn, one otherwise.
#[test]
fn thats_rough_buddy_counts_a_departed_creature() {
    for left in [false, true] {
        let mut g = pod(2);
        library(&mut g, 0, 2);
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        if left {
            let other = ready(&mut g, 0, catalog::llanowar_elves());
            let mut evs = Vec::new();
            g.sacrifice_one(other, 0, &mut evs);
        }
        let buddy = g.add_card_to_hand(0, catalog::thats_rough_buddy());
        flood(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: buddy, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(plus(&g, bear), if left { 2 } else { 1 });
    }
}

/// Paramecia Coloniex dying exiles itself to put a creature card on top of
/// your library.
#[test]
fn paramecia_coloniex_stacks_a_creature_on_top() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    let worm = ready(&mut g, 0, catalog::paramecia_coloniex());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut evs = Vec::new();
    g.sacrifice_one(worm, 0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == worm));
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(angel), "on top");
}

// ── decks::cmdr_scan7k_c ─────────────────────────────────────────────────────

fn kill(g: &mut GameState, id: CardId) {
    let owner = g.battlefield_find(id).unwrap().controller;
    let mut evs = Vec::new();
    g.sacrifice_one(id, owner, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// Fall of Gil-galad III: your creature gains "dies: draw two" and fights; it
/// loses the fight and you draw.
#[test]
fn fall_of_gil_galad_chapter_three_fights_with_a_death_draw() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    library(&mut g, 1, 6);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let big = ready(&mut g, 1, catalog::serra_angel());
    let saga = ready(&mut g, 0, catalog::fall_of_gil_galad());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    let hand = g.players[0].hand.len();
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    assert!(g.battlefield_find(bear).is_none(), "the bear lost the fight");
    assert_eq!(g.battlefield_find(big).unwrap().damage, 2);
    assert_eq!(g.players[0].hand.len(), hand + 1 + 2, "the draw step and the granted draw two");
}

/// The Huntsman's Redemption III: up to two creatures get +2/+2 and trample.
#[test]
fn the_huntsmans_redemption_chapter_three_pumps_two() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    library(&mut g, 1, 6);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    let saga = ready(&mut g, 0, catalog::the_huntsmans_redemption());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    for id in [a, b] {
        assert_eq!(pt(&g, id), (4, 4));
        assert!(g.permanent_has_keyword(id, &Keyword::Trample));
    }
}

/// Hall of Tagsin's {4} ability makes a tapped Powerstone.
#[test]
fn hall_of_tagsin_makes_a_tapped_powerstone() {
    let mut g = pod(2);
    let hall = ready(&mut g, 0, catalog::hall_of_tagsin());
    activate(&mut g, hall, 2, None);
    let stone = g.battlefield.iter().find(|c| c.definition.name == "Powerstone").expect("a Powerstone");
    assert!(stone.tapped);
}

/// Clay Revenant returns from the graveyard to hand for {2}{B}.
#[test]
fn clay_revenant_returns_from_the_graveyard() {
    let mut g = pod(2);
    let golem = g.add_card_to_graveyard(0, catalog::clay_revenant());
    activate(&mut g, golem, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == golem));
}

/// Cartouche of Strength's entry fight kills the opponent's bear.
#[test]
fn cartouche_of_strength_fights_on_entry() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let angel = ready(&mut g, 0, catalog::serra_angel());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::cartouche_of_strength());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(angel)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("aura");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none());
    assert_eq!(pt(&g, angel), (5, 5));
    assert!(g.permanent_has_keyword(angel, &Keyword::Trample));
}

/// Most Wanted pays two Treasures when its creature dies.
#[test]
fn most_wanted_leaves_two_treasures() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::most_wanted());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("aura");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (4, 3));
    kill(&mut g, bear);
    assert_eq!(named_count(&g, 0, "Treasure"), 2);
}

/// Supportive Parents taps two creatures for one mana of any color.
#[test]
fn supportive_parents_taps_two_for_mana() {
    let mut g = pod(2);
    let parents = ready(&mut g, 0, catalog::supportive_parents());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: parents, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("tap two");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1);
    assert!(g.battlefield_find(parents).unwrap().tapped && g.battlefield_find(bear).unwrap().tapped);
}

/// Hansk hands its opponent three Walkers each upkeep and draws when one dies.
#[test]
fn hansk_gives_walkers_and_draws_on_their_deaths() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    library(&mut g, 1, 4);
    ready(&mut g, 0, catalog::hansk_slayer_zealot());
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::Draw);
    let walkers: Vec<CardId> = g.battlefield.iter().filter(|c| c.definition.name == "Walker").map(|c| c.id).collect();
    assert_eq!(walkers.len(), 3);
    assert!(walkers.iter().all(|id| g.battlefield_find(*id).unwrap().controller == 1));
    let hand = g.players[0].hand.len();
    kill(&mut g, walkers[0]);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Vicious Shadows burns a player for their hand size when a creature dies.
#[test]
fn vicious_shadows_burns_for_hand_size() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::vicious_shadows());
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::island());
    }
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    kill(&mut g, bear);
    assert_eq!(g.players[1].life, life - 4);
}

/// Thor Odinson's two prowess instances: +2/+2 per noncreature spell.
#[test]
fn thor_odinson_has_double_prowess() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let thor = ready(&mut g, 0, catalog::thor_odinson());
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt);
    assert_eq!(pt(&g, thor), (6, 6));
}

/// Monica Rambeau transforms into Photon, whose noncreature spells grow the
/// rest of the team.
#[test]
fn monica_rambeau_transforms_into_photon() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let monica = ready(&mut g, 0, catalog::monica_rambeau());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, monica, 0, None);
    assert!(g.battlefield_find(monica).unwrap().transformed);
    assert!(g.permanent_has_keyword(monica, &Keyword::Hexproof));
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt);
    assert_eq!(plus(&g, bear), 1);
    assert_eq!(plus(&g, monica), 0, "each *other* creature");
}

/// Unflinching Hortimancer grows on life gain; Way of the Mentor makes a
/// five-loyalty Jace and adds loyalty on life gain.
#[test]
fn sundial_life_gain_grows_hortimancer_and_jace() {
    let mut g = pod(2);
    let hort = ready(&mut g, 0, catalog::unflinching_hortimancer());
    let way = g.add_card_to_hand(0, catalog::way_of_the_mentor());
    cast(&mut g, way);
    let jace = g.battlefield.iter().find(|c| c.definition.is_planeswalker()).map(|c| c.id).expect("a Jace token");
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 5);
    let salve = g.add_card_to_hand(0, catalog::healing_salve());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: salve, target: Some(Target::Player(0)), additional_targets: vec![], mode: Some(0), x_value: None,
    })
    .expect("salve");
    drain_stack(&mut g);
    assert_eq!(plus(&g, hort), 1);
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 6);
}

/// Donatello, Mutant Mechanic turns a Sol Ring into a 3/3 Robot; when it
/// dies its counters move on.
#[test]
fn donatello_mutant_mechanic_animates_and_salvages_counters() {
    let mut g = pod(2);
    let don = ready(&mut g, 0, catalog::donatello_mutant_mechanic());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    activate(&mut g, don, 0, Some(Target::Permanent(ring)));
    assert_eq!(pt(&g, ring), (3, 3));
    assert!(g.computed_permanent(ring).unwrap().subtypes().creature_types.contains(&crabomination::card::CreatureType::Robot));
    kill(&mut g, ring);
    assert_eq!(plus(&g, don), 3, "the counters moved to Donatello");
}

/// Donatello, Gadget Master's hit copies an artifact.
#[test]
fn donatello_gadget_master_copies_an_artifact() {
    let mut g = pod(2);
    let don = ready(&mut g, 0, catalog::donatello_gadget_master());
    ready(&mut g, 0, catalog::sol_ring());
    declare(&mut g, vec![(don, 1)]);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(named_count(&g, 0, "Sol Ring"), 2);
}

// ── decks::cmdr_scan7k_d ─────────────────────────────────────────────────────

fn attach(g: &mut GameState, aura: CardId, host: CardId) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(host)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("aura");
    drain_stack(g);
}

/// Invisibility: only a Wall may block the enchanted creature.
#[test]
fn invisibility_admits_only_walls() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::invisibility());
    attach(&mut g, aura, bear);
    let other = ready(&mut g, 1, catalog::grizzly_bears());
    let wall = ready(&mut g, 1, catalog::wall_of_shards());
    declare(&mut g, vec![(bear, 1)]);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(other, bear)])).is_err(), "a Bear can't block");
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, bear)])).expect("a Wall can");
}

/// Stonebrow pumps each attacking trampler — Khenra Charioteer gives the
/// team trample.
#[test]
fn stonebrow_pumps_tramplers_from_khenra() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::stonebrow_krosan_hero());
    ready(&mut g, 0, catalog::khenra_charioteer());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(g.permanent_has_keyword(bear, &Keyword::Trample));
    declare(&mut g, vec![(bear, 1)]);
    assert_eq!(pt(&g, bear), (4, 4));
}

/// Overwhelming Remorse costs {1} less per creature card in your graveyard.
#[test]
fn overwhelming_remorse_discounts_by_graveyard_creatures() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::lurching_rotbeast());
    }
    let theirs = ready(&mut g, 1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::overwhelming_remorse());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(theirs)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("{4}{B} less four");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == theirs));
}

/// Dragon Egg hatches a firebreathing flier.
#[test]
fn dragon_egg_hatches_a_firebreathing_dragon() {
    let mut g = pod(2);
    let egg = ready(&mut g, 0, catalog::dragon_egg());
    kill(&mut g, egg);
    let dragon = g.battlefield.iter().find(|c| c.definition.name == "Dragon").map(|c| c.id).expect("hatched");
    g.clear_sickness(dragon);
    activate(&mut g, dragon, 0, None);
    assert_eq!(pt(&g, dragon), (3, 2));
    assert!(g.permanent_has_keyword(dragon, &Keyword::Flying));
}

/// Daghatar moves a +1/+1 counter between two creatures.
#[test]
fn daghatar_moves_a_counter() {
    let mut g = pod(2);
    let dag = g.add_card_to_hand(0, catalog::daghatar_the_adamant());
    cast(&mut g, dag);
    assert_eq!(plus(&g, dag), 4);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dag, ability_index: 0, target: Some(Target::Permanent(dag)),
        additional_targets: vec![Target::Permanent(bear)], x_value: None, mode: None,
    })
    .expect("move");
    drain_stack(&mut g);
    assert_eq!((plus(&g, dag), plus(&g, bear)), (3, 1));
}

/// Wall of Shards' cumulative upkeep feeds an opponent one life per age
/// counter.
#[test]
fn wall_of_shards_upkeep_gives_an_opponent_life() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    library(&mut g, 1, 4);
    let wall = ready(&mut g, 0, catalog::wall_of_shards());
    let life = g.players[1].life;
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::Draw);
    assert!(g.battlefield_find(wall).is_some(), "paid");
    assert_eq!(g.players[1].life, life + 1);
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::Draw);
    assert_eq!(g.players[1].life, life + 1 + 2, "two age counters");
}

/// Refuse to Yield untaps and pumps +2/+7.
#[test]
fn refuse_to_yield_untaps_and_pumps() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::refuse_to_yield());
    attach(&mut g, spell, bear);
    assert_eq!(pt(&g, bear), (4, 9));
    assert!(!g.battlefield_find(bear).unwrap().tapped);
}

/// Lord of Tresserhorn's entry: 2 life, two creatures, two cards to an
/// opponent.
#[test]
fn lord_of_tresserhorn_pays_its_entry() {
    let mut g = pod(2);
    library(&mut g, 1, 3);
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::grizzly_bears());
    let lord = g.add_card_to_hand(0, catalog::lord_of_tresserhorn());
    let life = g.players[0].life;
    cast(&mut g, lord);
    assert_eq!(g.players[0].life, life - 2);
    assert_eq!(named_count(&g, 0, "Grizzly Bears"), 0);
    assert!(g.battlefield_find(lord).is_some(), "the Lord keeps itself when two others can go");
    assert_eq!(g.players[1].hand.len(), 2);
}

/// Defabricate exiles a countered artifact spell.
#[test]
fn defabricate_exiles_an_artifact_spell() {
    let mut g = pod(2);
    let ring = g.add_card_to_hand(1, catalog::sol_ring());
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: ring, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("ring");
    let spell = g.add_card_to_hand(0, catalog::defabricate());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(ring)), additional_targets: vec![], mode: Some(0), x_value: None,
    })
    .expect("counter");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == ring));
}

/// Frondland Felidar gives a vigilant creature "{1}, {T}: tap target
/// creature."
#[test]
fn frondland_felidar_grants_a_tapper() {
    let mut g = pod(2);
    let cat = ready(&mut g, 0, catalog::frondland_felidar());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    activate(&mut g, cat, 0, Some(Target::Permanent(theirs)));
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}

/// Old Man Willow is as big as your land count; Dauntless Dourbark counts
/// Forests and Treefolk and tramples beside another Treefolk.
#[test]
fn old_man_willow_and_dourbark_size_themselves() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::forest());
    }
    let willow = ready(&mut g, 0, catalog::old_man_willow());
    let bark = ready(&mut g, 0, catalog::dauntless_dourbark());
    assert_eq!(pt(&g, willow), (3, 3));
    assert_eq!(pt(&g, bark), (5, 5), "three Forests and two Treefolk");
    assert!(g.permanent_has_keyword(bark, &Keyword::Trample));
}

// ── decks::cmdr_scan7k_e ─────────────────────────────────────────────────────

/// The Kingpin's attack trigger: pay 2 life and a 1/5 assigns 5.
#[test]
fn kingpin_of_crime_makes_walls_hit_by_toughness() {
    let mut g = pod(2);
    let king = ready(&mut g, 0, catalog::the_kingpin_of_crime());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[0].life;
    let foe = g.players[1].life;
    declare(&mut g, vec![(king, 1)]);
    assert_eq!(g.players[0].life, life - 2);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, foe - 5);
}

/// Liliana the Repentant mills two as another creature enters; Dark Matter
/// Manipulator gets +2/+0 at seven graveyard cards.
#[test]
fn liliana_mills_and_dark_matter_grows() {
    let mut g = pod(2);
    library(&mut g, 0, 10);
    ready(&mut g, 0, catalog::liliana_the_repentant());
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    let dmm = g.add_card_to_hand(0, catalog::dark_matter_manipulator());
    cast(&mut g, dmm);
    assert_eq!(g.players[0].graveyard.len(), 2 + 3 + 2, "two seeded, its three, Liliana's two");
    assert_eq!(pt(&g, dmm), (3, 2));
}

/// Liliana's exhaust returns a creature card and grows her, once.
#[test]
fn liliana_exhaust_reanimates_once() {
    let mut g = pod(2);
    let lili = ready(&mut g, 0, catalog::liliana_the_repentant());
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    activate(&mut g, lili, 0, Some(Target::Permanent(angel)));
    assert!(g.battlefield_find(angel).is_some());
    assert_eq!(plus(&g, lili), 1);
    flood(&mut g);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: lili, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).is_err(), "exhaust: once");
}

/// Aang, A Lot to Learn grows when another creature of yours dies and has
/// vigilance once a Lesson is in your graveyard.
#[test]
fn aang_a_lot_to_learn_grows_and_learns() {
    let mut g = pod(2);
    let aang = ready(&mut g, 0, catalog::aang_a_lot_to_learn());
    assert!(!g.permanent_has_keyword(aang, &Keyword::Vigilance));
    g.add_card_to_graveyard(0, catalog::sokkas_sword_training());
    assert!(g.permanent_has_keyword(aang, &Keyword::Vigilance));
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    kill(&mut g, bear);
    assert_eq!(plus(&g, aang), 1);
}

/// Pathmaker Initiate makes a 2-power creature unblockable this turn.
#[test]
fn pathmaker_initiate_unblocks_a_small_creature() {
    let mut g = pod(2);
    let initiate = ready(&mut g, 0, catalog::pathmaker_initiate());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, initiate, 0, Some(Target::Permanent(bear)));
    assert!(g.permanent_has_keyword(bear, &Keyword::Unblockable));
}

/// Wildfire Eternal unblocked: cast a sorcery from hand for free.
#[test]
fn wildfire_eternal_casts_free_when_unblocked() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    let eternal = ready(&mut g, 0, catalog::wildfire_eternal());
    let div = g.add_card_to_hand(0, catalog::divination());
    declare(&mut g, vec![(eternal, 1)]);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert!(g.players[0].graveyard.iter().any(|c| c.id == div), "Divination was cast");
    assert_eq!(g.players[0].mana_pool.total(), 0, "for free");
}

/// Twisted Embrace kills an opposing creature and pumps its host.
#[test]
fn twisted_embrace_kills_and_pumps() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::serra_angel());
    let aura = g.add_card_to_hand(0, catalog::twisted_embrace());
    attach(&mut g, aura, bear);
    assert!(g.battlefield_find(theirs).is_none());
    assert_eq!(pt(&g, bear), (3, 3));
}

/// Leech Gauntlet reconfigured onto a creature gives it lifelink and stops
/// being a creature.
#[test]
fn leech_gauntlet_reconfigures_for_lifelink() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let leech = ready(&mut g, 0, catalog::leech_gauntlet());
    flood(&mut g);
    g.perform_action(GameAction::Equip { equipment: leech, target: bear }).expect("reconfigure {4}");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(bear, &Keyword::Lifelink));
    assert!(!g.computed_permanent(leech).unwrap().card_types().contains(&crabomination::card::CardType::Creature));
}

/// Loyal Tutor stacks a planeswalker on top.
#[test]
fn loyal_tutor_stacks_a_planeswalker() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let pw = g.add_card_to_library(0, catalog::worzel_the_protector());
    let tutor = g.add_card_to_hand(0, catalog::loyal_tutor());
    cast(&mut g, tutor);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(pw));
}

/// Worzel's Cat adds loyalty to each planeswalker; Worzel can be a commander.
#[test]
fn worzel_cat_adds_loyalty() {
    let mut g = pod(2);
    assert!(catalog::worzel_the_protector().can_be_commander);
    let pw = ready(&mut g, 0, catalog::worzel_the_protector());
    loyalty(&mut g, pw, 0, None);
    let before = g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty);
    let cat = g.battlefield.iter().find(|c| c.definition.name == "Cat").map(|c| c.id).expect("a Cat");
    g.clear_sickness(cat);
    activate(&mut g, cat, 0, None);
    assert_eq!(g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty), before + 1);
}

/// Photon Blast Barrage with X = 2 is three pings (the copies keep the
/// original's target while it isn't yet lethal).
#[test]
fn photon_blast_barrage_copies_x_times() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::photon_blast_barrage());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(angel)), additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(angel).unwrap().damage, 3);
}

/// CR 702.40a / 608.2b — bug fix: once the shots already aimed at a creature
/// are lethal, a storm-style copy defaults to another opposing creature, so
/// X = 2 kills three 1/1s instead of one 1/1 and two fizzles.
#[test]
fn photon_blast_barrage_copies_spread_past_a_lethal_target() {
    let mut g = pod(3);
    let elves: Vec<CardId> = [1, 1, 2].iter().map(|&s| ready(&mut g, s, catalog::llanowar_elves())).collect();
    let spell = g.add_card_to_hand(0, catalog::photon_blast_barrage());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(elves[0])), additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    for e in elves {
        assert!(g.battlefield_find(e).is_none(), "every 1/1 dies");
    }
}

// ── decks::cmdr_scan7k_f ─────────────────────────────────────────────────────

/// Ezio grows each attack; at two counters he has first strike and is an
/// Assassin.
#[test]
fn ezio_becomes_an_assassin_at_two_counters() {
    let mut g = pod(2);
    let ezio = ready(&mut g, 0, catalog::ezio_brash_novice());
    declare(&mut g, vec![(ezio, 1)]);
    assert!(!g.permanent_has_keyword(ezio, &Keyword::FirstStrike));
    g.battlefield_find_mut(ezio).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.permanent_has_keyword(ezio, &Keyword::FirstStrike));
    assert!(g.computed_permanent(ezio).unwrap().subtypes().creature_types.contains(&crabomination::card::CreatureType::Assassin));
}

/// Origin of the Hidden Ones III: each attacking Assassin brings another,
/// tapped and attacking.
#[test]
fn origin_of_the_hidden_ones_chapter_three_multiplies_assassins() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    library(&mut g, 1, 6);
    let saga = ready(&mut g, 0, catalog::origin_of_the_hidden_ones());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    let ezio = ready(&mut g, 0, catalog::ezio_brash_novice());
    g.battlefield_find_mut(ezio).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    g.clear_sickness(ezio);
    declare(&mut g, vec![(ezio, 1)]);
    assert!(g.attacking().iter().any(|a| a.attacker != ezio), "a fresh Assassin joined the attack");
}

/// Thanos grows on any other creature's death; Visions of Villainy costs
/// {1}{B} beside him.
#[test]
fn thanos_grows_and_visions_discount() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let thanos = ready(&mut g, 0, catalog::thanos_deaths_consort());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    kill(&mut g, bear);
    assert_eq!(plus(&g, thanos), 1);
    let visions = g.add_card_to_hand(0, catalog::visions_of_villainy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: visions, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{1}{B} with a Villain");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Villainous Wrath drains by the opponent's creature count, then wipes.
#[test]
fn villainous_wrath_drains_then_wipes() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 1, catalog::grizzly_bears());
    }
    let wrath = g.add_card_to_hand(0, catalog::villainous_wrath());
    let life = g.players[1].life;
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: wrath, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("wrath");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
    assert!(g.battlefield.iter().all(|c| !c.definition.is_creature()));
}

/// Lake-town's sacrifice puts two counters on a Human.
#[test]
fn lake_town_grows_a_human() {
    let mut g = pod(2);
    let town = ready(&mut g, 0, catalog::lake_town());
    let sokka = ready(&mut g, 0, catalog::sokka_wolf_coves_protector());
    activate(&mut g, town, 2, Some(Target::Permanent(sokka)));
    assert_eq!(plus(&g, sokka), 2);
    assert!(g.battlefield_find(town).is_none());
}

/// Sanctum of Calm Waters draws a card per Shrine and discards one.
#[test]
fn sanctum_of_calm_waters_draws_per_shrine() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    library(&mut g, 1, 6);
    ready(&mut g, 0, catalog::sanctum_of_calm_waters());
    ready(&mut g, 0, catalog::go_shintai_of_lost_wisdom());
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1 + 2 - 1, "draw step, two Shrines, one discard");
}

/// King's Assassin kills a tapped creature, but only before attacks on your
/// turn.
#[test]
fn kings_assassin_kills_a_tapped_creature() {
    let mut g = pod(2);
    let assassin = ready(&mut g, 0, catalog::kings_assassin());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    activate(&mut g, assassin, 0, Some(Target::Permanent(theirs)));
    assert!(g.battlefield_find(theirs).is_none());
}

/// Tainted Adversary paid twice: two counters and four decayed Zombies.
#[test]
fn tainted_adversary_pays_repeatedly() {
    let mut g = pod(2);
    let adv = g.add_card_to_hand(0, catalog::tainted_adversary());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.players[0].mana_pool.add_colorless(5);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true), DecisionAnswer::Bool(false)]));
    g.perform_action(GameAction::CastSpell { card_id: adv, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(plus(&g, adv), 2);
    assert_eq!(named_count(&g, 0, "Zombie"), 4);
}

/// Mockingbird grows when she survives damage.
#[test]
fn mockingbird_grows_on_damage() {
    let mut g = pod(2);
    let bird = ready(&mut g, 0, catalog::mockingbird_bobbi_morse());
    let shock = g.add_card_to_hand(0, catalog::shock());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: shock, target: Some(Target::Permanent(bird)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("shock");
    drain_stack(&mut g);
    assert_eq!(plus(&g, bird), 1);
}

/// Gandalf, White Rider pumps the team on each spell; dying, he goes fifth
/// from the top.
#[test]
fn gandalf_white_rider_pumps_and_returns_to_the_library() {
    let mut g = pod(2);
    library(&mut g, 0, 8);
    let gandalf = ready(&mut g, 0, catalog::gandalf_white_rider());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let wisps = g.add_card_to_hand(0, catalog::niveous_wisps());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: wisps, target: Some(Target::Permanent(theirs)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("wisps");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (3, 2));
    assert!(g.battlefield_find(theirs).unwrap().tapped);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    kill(&mut g, gandalf);
    assert_eq!(g.players[0].library.get(4).map(|c| c.id), Some(gandalf));
}

/// Gideon's Avenger grows when an opposing creature taps.
#[test]
fn gideons_avenger_grows_on_opposing_taps() {
    let mut g = pod(2);
    let avenger = ready(&mut g, 0, catalog::gideons_avenger());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let wisps = g.add_card_to_hand(0, catalog::niveous_wisps());
    library(&mut g, 0, 2);
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: wisps, target: Some(Target::Permanent(theirs)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("wisps");
    drain_stack(&mut g);
    assert_eq!(plus(&g, avenger), 1);
}

// ── decks::cmdr_scan7k_g ─────────────────────────────────────────────────────

/// Guan Yu's 1,000-Li March kills only the tapped creatures.
#[test]
fn guan_yus_march_destroys_tapped_creatures() {
    let mut g = pod(2);
    let tapped = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    let untapped = ready(&mut g, 1, catalog::grizzly_bears());
    let march = g.add_card_to_hand(0, catalog::guan_yus_1_000_li_march());
    cast(&mut g, march);
    assert!(g.battlefield_find(tapped).is_none());
    assert!(g.battlefield_find(untapped).is_some());
}

/// Quintessential Katana jumps onto an entering Ninja.
#[test]
fn quintessential_katana_attaches_to_an_entering_ninja() {
    let mut g = pod(2);
    let katana = ready(&mut g, 0, catalog::quintessential_katana());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ninja = g.add_card_to_hand(0, catalog::ninja_of_the_deep_hours());
    cast(&mut g, ninja);
    assert_eq!(g.battlefield_find(katana).unwrap().attached_to, Some(ninja));
    assert_eq!(pt(&g, ninja), (3, 3));
}

/// Leonardo's Technique returns two cheap creature cards.
#[test]
fn leonardos_technique_returns_two() {
    let mut g = pod(2);
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let spell = g.add_card_to_hand(0, catalog::leonardos_technique());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(a)), additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some());
}

/// Stature can't be blocked at 1 power; her power-up grows her past it.
#[test]
fn stature_is_unblockable_while_small() {
    let mut g = pod(2);
    let stature = ready(&mut g, 0, catalog::stature_size_shifter());
    assert!(g.permanent_has_keyword(stature, &Keyword::Unblockable));
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: stature, ability_index: 0, target: None, additional_targets: vec![], x_value: Some(2), mode: None,
    })
    .expect("power-up");
    drain_stack(&mut g);
    assert_eq!(plus(&g, stature), 2);
    assert!(!g.permanent_has_keyword(stature, &Keyword::Unblockable));
}

/// Secret Identity's Reveal: a 3/4 Hero with flying and vigilance.
#[test]
fn secret_identity_reveals_a_hero() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::secret_identity());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("reveal");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (3, 4));
    assert!(g.permanent_has_keyword(bear, &Keyword::Flying) && g.permanent_has_keyword(bear, &Keyword::Vigilance));
}

/// Katara grows per Lesson in the graveyard; Scarring Memories makes an
/// opponent sacrifice, discard and lose 3.
#[test]
fn katara_counts_lessons_and_scarring_memories_punishes() {
    let mut g = pod(2);
    let katara = ready(&mut g, 0, catalog::katara_seeking_revenge());
    g.add_card_to_graveyard(0, catalog::scarring_memories());
    assert_eq!(pt(&g, katara), (4, 4));
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::scarring_memories());
    let life = g.players[1].life;
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[1].hand.is_empty());
    assert_eq!(g.players[1].life, life - 3);
    assert_eq!(pt(&g, katara), (5, 5));
}

/// Minion Missile pays with a discard, destroys, and burns the controller.
#[test]
fn minion_missile_destroys_and_burns() {
    let mut g = pod(2);
    g.add_card_to_hand(0, catalog::island());
    let theirs = ready(&mut g, 1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::minion_missile());
    let life = g.players[1].life;
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(theirs)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none());
    assert_eq!(g.players[1].life, life - 2);
}

/// Shadow Sliver gives every Sliver shadow.
#[test]
fn shadow_sliver_shades_all_slivers() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::shadow_sliver());
    assert!(g.permanent_has_keyword(mine, &Keyword::Shadow));
}

/// Reality Heist with four artifacts costs {1}{U}{U} and takes two artifacts.
#[test]
fn reality_heist_takes_two_artifacts() {
    let mut g = pod(2);
    for _ in 0..4 {
        ready(&mut g, 0, catalog::sol_ring());
    }
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_library(0, catalog::sol_ring());
    g.add_card_to_library(0, catalog::mind_stone());
    let heist = g.add_card_to_hand(0, catalog::reality_heist());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: heist, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("affinity");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// The Blackstaff animates an artifact while it stays tapped.
#[test]
fn blackstaff_animates_while_tapped() {
    let mut g = pod(2);
    let staff = ready(&mut g, 0, catalog::the_blackstaff_of_waterdeep());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    activate(&mut g, staff, 0, Some(Target::Permanent(ring)));
    assert_eq!(pt(&g, ring), (4, 4));
}

/// Massacre Girl, Most Wanted pings an opponent when your creature dies and
/// grows from it.
#[test]
fn massacre_girl_pings_and_grows() {
    let mut g = pod(2);
    let mg = ready(&mut g, 0, catalog::massacre_girl_most_wanted());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let life = g.players[1].life;
    kill(&mut g, bear);
    assert_eq!(g.players[1].life, life - 1);
    assert_eq!(plus(&g, mg), 1);
}

/// Northampton Farm stores a creature and brings it back.
#[test]
fn northampton_farm_stores_and_returns_a_creature() {
    let mut g = pod(2);
    let farm = ready(&mut g, 0, catalog::northampton_farm());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, farm, 1, Some(Target::Permanent(bear)));
    assert!(g.exile.iter().any(|c| c.id == bear));
    g.battlefield_find_mut(farm).unwrap().tapped = false;
    activate(&mut g, farm, 2, None);
    assert!(g.battlefield_find(bear).is_some());
}

// ── decks::cmdr_scan7k_h ─────────────────────────────────────────────────────

/// Iron Man Armor attaches as it enters and flies its wearer; unattached, it
/// suits up as a flier as big as your artifact count.
#[test]
fn iron_man_armor_attaches_and_suits_up() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let armor = g.add_card_to_hand(0, catalog::iron_man_armor());
    cast(&mut g, armor);
    assert_eq!(g.battlefield_find(armor).unwrap().attached_to, Some(bear));
    assert_eq!(pt(&g, bear), (4, 3));
    let spare = ready(&mut g, 0, catalog::iron_man_armor());
    ready(&mut g, 0, catalog::sol_ring());
    activate(&mut g, spare, 0, None);
    assert_eq!(pt(&g, spare), (3, 3), "three artifacts");
    assert!(g.permanent_has_keyword(spare, &Keyword::Flying));
}

/// Ajani Goldmane's −1 counters the team and grants vigilance.
#[test]
fn ajani_goldmane_minus_one_counters_the_team() {
    let mut g = pod(2);
    let pw = ready(&mut g, 0, catalog::ajani_goldmane());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    loyalty(&mut g, pw, 1, None);
    assert_eq!(plus(&g, bear), 1);
    assert!(g.permanent_has_keyword(bear, &Keyword::Vigilance));
}

/// Saved by the Shell costs {G} beside a Turtle.
#[test]
fn saved_by_the_shell_discounts_with_a_turtle() {
    let mut g = pod(2);
    let turtle = ready(&mut g, 0, catalog::donatello_gadget_master());
    let spell = g.add_card_to_hand(0, catalog::saved_by_the_shell());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(turtle)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("{G} with a Turtle");
    drain_stack(&mut g);
    assert_eq!(plus(&g, turtle), 1);
    assert!(g.permanent_has_keyword(turtle, &Keyword::Indestructible));
}

/// Avatar of Burgeoning Echoes' landfall makes a Jace with two loyalty.
#[test]
fn avatar_of_burgeoning_echoes_empowers_jace_on_landfall() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::avatar_of_burgeoning_echoes());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    let jace = g.battlefield.iter().find(|c| c.definition.is_planeswalker()).expect("a Jace token");
    assert_eq!(jace.counter_count(CounterType::Loyalty), 2);
}

/// Fall of the Impostor III exiles the opponent's biggest creature.
#[test]
fn fall_of_the_impostor_exiles_the_biggest() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    library(&mut g, 1, 6);
    let small = ready(&mut g, 1, catalog::grizzly_bears());
    let big = ready(&mut g, 1, catalog::serra_angel());
    let saga = ready(&mut g, 0, catalog::fall_of_the_impostor());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    assert!(g.battlefield_find(big).is_none());
    assert!(g.battlefield_find(small).is_some());
}

/// Devout Invocation turns each tapped creature into a 4/4 Angel.
#[test]
fn devout_invocation_makes_an_angel_per_tapped_creature() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::devout_invocation());
    cast(&mut g, spell);
    assert_eq!(named_count(&g, 0, "Angel"), 3);
}

/// Gimli fights on the third death of the turn and is indestructible from
/// the second.
#[test]
fn gimli_fights_on_the_third_death() {
    let mut g = pod(2);
    let gimli = ready(&mut g, 0, catalog::gimli_mournful_avenger());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    for _ in 0..3 {
        let bear = ready(&mut g, 0, catalog::llanowar_elves());
        kill(&mut g, bear);
    }
    assert_eq!(plus(&g, gimli), 3);
    assert!(g.permanent_has_keyword(gimli, &Keyword::Indestructible));
    assert!(g.battlefield_find(theirs).is_none(), "the third resolution's fight");
}

// ── decks::cmdr_scan7k_i ─────────────────────────────────────────────────────

fn no_blocks_through_combat(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

fn cast_at(g: &mut GameState, id: CardId, target: Option<Target>, additional_targets: Vec<Target>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets, mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

/// Slinza: a Beast costs {2} less and enters with an extra counter (Woodland
/// Liege draws for it); its power-4 entry lets Slinza fight for {1}{R/G}.
#[test]
fn slinza_discounts_counters_and_fights_a_big_entry() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    ready(&mut g, 0, catalog::slinza_the_spiked_stampede());
    ready(&mut g, 0, catalog::woodland_liege());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let baloth = g.add_card_to_hand(0, catalog::enormous_baloth());
    let hand = g.players[0].hand.len();
    flood(&mut g);
    let before = g.players[0].mana_pool.total();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Target(Target::Permanent(theirs))]));
    g.perform_action(GameAction::CastSpell { card_id: baloth, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(before - g.players[0].mana_pool.total(), 5 + 2, "{{4}}{{G}} for the Baloth, {{1}}{{R/G}} for the fight");
    assert_eq!(plus(&g, baloth), 1, "the Beast's extra counter");
    assert_eq!(g.players[0].hand.len(), hand, "the Liege replaced the cast Beast");
    assert!(g.battlefield_find(theirs).is_none(), "Slinza fought the Bears");
}

/// Zagras gives your other creatures deathtouch, and a creature of yours that
/// hits a planeswalker destroys it.
#[test]
fn zagras_destroys_a_damaged_planeswalker() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::zagras_thief_of_heartbeats());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(g.permanent_has_keyword(bear, &Keyword::Deathtouch));
    let pw = ready(&mut g, 1, catalog::ajani_goldmane());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Planeswalker(pw) }]))
        .expect("attack");
    drain_stack(&mut g);
    no_blocks_through_combat(&mut g);
    assert!(g.battlefield_find(pw).is_none(), "two damage left Ajani at 2; the trigger destroyed him");
}

/// Power of Fire's host taps to ping any target.
#[test]
fn power_of_fire_grants_a_pinger() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::power_of_fire());
    attach(&mut g, aura, bear);
    let life = g.players[1].life;
    activate(&mut g, bear, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 1);
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Feroz: +2 makes two flying Birds, 0 draws and drops a cheap permanent, −7
/// makes the Ban that taxes creature spells.
#[test]
fn feroz_birds_drop_and_ban() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let pw = ready(&mut g, 0, catalog::feroz_ulgrothas_warden());
    loyalty(&mut g, pw, 0, None);
    assert_eq!(named_count(&g, 0, "Bird"), 2);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    g.battlefield_find_mut(pw).unwrap().loyalty_uses_this_turn = 0;
    loyalty(&mut g, pw, 1, None);
    assert!(g.battlefield_find(bear).is_some(), "the Bears were put onto the battlefield");
    g.battlefield_find_mut(pw).unwrap().loyalty_uses_this_turn = 0;
    g.battlefield_find_mut(pw).unwrap().add_counters(CounterType::Loyalty, 7);
    loyalty(&mut g, pw, 2, None);
    assert_eq!(named_count(&g, 0, "Feroz's Ban"), 1);
    let elves = g.add_card_to_hand(0, catalog::llanowar_elves());
    flood(&mut g);
    let before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::CastSpell { card_id: elves, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    assert_eq!(before - g.players[0].mana_pool.total(), 3, "{{G}} plus the Ban's {{2}}");
}

/// Oath of Ajani counters the team and discounts a planeswalker by {1}.
#[test]
fn oath_of_ajani_counters_and_discounts_walkers() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let oath = g.add_card_to_hand(0, catalog::oath_of_ajani());
    cast(&mut g, oath);
    assert_eq!(plus(&g, bear), 1);
    let ajani = g.add_card_to_hand(0, catalog::ajani_goldmane());
    flood(&mut g);
    let before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::CastSpell { card_id: ajani, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    assert_eq!(before - g.players[0].mana_pool.total(), 3);
}

/// The Spot exiles a permanent and a graveyard card; dying, he goes to the
/// bottom of the library and both cards return to their owners' hands.
#[test]
fn the_spot_exiles_and_returns_on_death() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let angel = g.add_card_to_graveyard(1, catalog::serra_angel());
    let spot = g.add_card_to_hand(0, catalog::the_spot_living_portal());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(bear)),
        DecisionAnswer::Target(Target::Permanent(angel)),
    ]));
    cast(&mut g, spot);
    assert!(g.exile.iter().any(|c| c.id == bear) && g.exile.iter().any(|c| c.id == angel));
    kill(&mut g, spot);
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(spot), "on the bottom");
    for id in [bear, angel] {
        assert!(g.players[1].hand.iter().any(|c| c.id == id), "back in its owner's hand");
    }
}

/// The Spot's Portal bottoms a creature; the 2 life is waived with a Villain.
#[test]
fn the_spots_portal_bottoms_and_charges_without_a_villain() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let portal = g.add_card_to_hand(0, catalog::the_spots_portal());
    let life = g.players[0].life;
    cast_at(&mut g, portal, Some(Target::Permanent(bear)), vec![]);
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(bear));
    assert_eq!(g.players[0].life, life - 2);
    ready(&mut g, 0, catalog::the_spot_living_portal());
    let other = ready(&mut g, 1, catalog::grizzly_bears());
    let portal = g.add_card_to_hand(0, catalog::the_spots_portal());
    cast_at(&mut g, portal, Some(Target::Permanent(other)), vec![]);
    assert_eq!(g.players[0].life, life - 2, "a Villain waives the life");
}

/// Justice grows when another of your nonland permanents is bounced; Spider-Man
/// 2099's entry bounces a creature and he draws on combat damage.
#[test]
fn justice_grows_on_a_bounce_and_spider_man_2099_draws() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let justice = ready(&mut g, 0, catalog::justice_vance_astrovik());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let unsummon = g.add_card_to_hand(0, catalog::unsummon());
    cast_at(&mut g, unsummon, Some(Target::Permanent(bear)), vec![]);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert_eq!(plus(&g, justice), 1);
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let spidey = g.add_card_to_hand(0, catalog::spider_man_2099_miguel_ohara());
    cast(&mut g, spidey);
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs), "Spider-Man's entry bounced the Bears");
    assert_eq!(plus(&g, justice), 1, "not a permanent you control");
    g.clear_sickness(spidey);
    let hand = g.players[0].hand.len();
    declare(&mut g, vec![(spidey, 1), (justice, 1)]);
    no_blocks_through_combat(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "one draw for the batch");
}

/// Agent 13 investigates only when a creature attacks alone.
#[test]
fn agent_13_investigates_on_a_lone_attack() {
    let mut g = pod(3);
    let agent = ready(&mut g, 0, catalog::agent_13_sharon_carter());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(agent, 1), (bear, 2)]);
    assert_eq!(named_count(&g, 0, "Clue"), 0, "two attackers");
    let mut g = pod(3);
    ready(&mut g, 0, catalog::agent_13_sharon_carter());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(bear, 2)]);
    assert_eq!(named_count(&g, 0, "Clue"), 1);
}

/// Political Triumph's fourth plan counter sacrifices it for a draw and a
/// counter on each creature.
#[test]
fn political_triumph_pays_off_on_the_fourth_entry() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    let plan = ready(&mut g, 0, catalog::political_triumph());
    let mut bears = Vec::new();
    for _ in 0..4 {
        let b = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast(&mut g, b);
        bears.push(b);
    }
    assert!(g.battlefield_find(plan).is_none());
    assert!(bears.iter().all(|b| plus(&g, *b) == 1));
    assert_eq!(g.players[0].hand.len(), 1, "drew one");
}

/// Fire Nation Sentinels grow the team on an opposing nontoken death; Fire
/// Nation Occupation makes a Soldier on entry and on an off-turn cast.
#[test]
fn fire_nation_sentinels_and_occupation() {
    let mut g = pod(2);
    let sentinels = ready(&mut g, 0, catalog::fire_nation_sentinels());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    kill(&mut g, theirs);
    assert_eq!(plus(&g, sentinels), 1);
    let occ = g.add_card_to_hand(0, catalog::fire_nation_occupation());
    cast(&mut g, occ);
    assert_eq!(named_count(&g, 0, "Soldier"), 1);
    library(&mut g, 0, 2);
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt);
    assert_eq!(named_count(&g, 0, "Soldier"), 1, "not on your own turn");
    g.active_player_idx = 1;
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt);
    assert_eq!(named_count(&g, 0, "Soldier"), 2);
}

/// Bard the Bowman: the second draw of a turn counters a creature and gives it
/// lifelink.
#[test]
fn bard_the_bowman_rewards_the_second_draw() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let bard = ready(&mut g, 0, catalog::bard_the_bowman());
    let div = g.add_card_to_hand(0, catalog::divination());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bard))]));
    cast(&mut g, div);
    assert_eq!(plus(&g, bard), 1);
    assert!(g.permanent_has_keyword(bard, &Keyword::Lifelink));
}

/// Raff taps two creatures to draw off an instant, and pumps the team.
#[test]
fn raff_taps_two_to_draw_and_pumps() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let raff = ready(&mut g, 0, catalog::raff_weatherlight_stalwart());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let opt = g.add_card_to_hand(0, catalog::opt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    cast(&mut g, opt);
    assert!(g.battlefield_find(raff).unwrap().tapped && g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "Opt's draw and Raff's");
    activate(&mut g, raff, 0, None);
    assert_eq!(pt(&g, bear), (3, 3));
    assert!(g.permanent_has_keyword(bear, &Keyword::Vigilance));
}

/// Leering Emblem pumps its creature on each spell.
#[test]
fn leering_emblem_pumps_on_cast() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let emblem = ready(&mut g, 0, catalog::leering_emblem());
    g.battlefield_find_mut(emblem).unwrap().attached_to = Some(bear);
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt);
    assert_eq!(pt(&g, bear), (4, 4));
}

/// Vance's Blasting Cannons: the upkeep exile is castable when nonland; the
/// third spell of a turn transforms it into Spitfire Bastion.
#[test]
fn vances_blasting_cannons_impulses_and_transforms() {
    let mut g = pod(2);
    library(&mut g, 1, 3);
    library(&mut g, 0, 3);
    let top = g.add_card_to_library(0, catalog::opt());
    g.players[0].library.rotate_right(1);
    assert_eq!(g.players[0].library[0].id, top);
    let cannons = ready(&mut g, 0, catalog::vances_blasting_cannons());
    step_to(&mut g, 1, TurnStep::Upkeep);
    step_to(&mut g, 0, TurnStep::Upkeep);
    assert!(g.exile.iter().any(|c| c.id == top && c.may_play_until.is_some()));
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    for _ in 0..3 {
        let thopter = g.add_card_to_hand(0, catalog::ornithopter());
        cast(&mut g, thopter);
    }
    assert_eq!(g.battlefield_find(cannons).unwrap().definition.name, "Spitfire Bastion");
}

// ── decks::cmdr_scan7k_j ─────────────────────────────────────────────────────

/// Tura makes a Soldier per instant; Stonybrook Schoolmaster makes a Merfolk
/// Wizard when it becomes tapped.
#[test]
fn tura_and_stonybrook_make_tokens() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    ready(&mut g, 0, catalog::tura_kennerud_skyknight());
    let school = ready(&mut g, 0, catalog::stonybrook_schoolmaster());
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt);
    assert_eq!(named_count(&g, 0, "Soldier"), 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    declare(&mut g, vec![(school, 1)]);
    assert_eq!(named_count(&g, 0, "Merfolk Wizard"), 1);
}

/// Esgaroth Garrison recruits on entry and counts your creatures for power.
#[test]
fn esgaroth_garrison_recruits_and_counts() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let top = g.add_card_to_library(0, catalog::opt());
    g.players[0].library.rotate_right(1);
    assert_eq!(g.players[0].library[0].id, top);
    let garrison = g.add_card_to_hand(0, catalog::esgaroth_garrison());
    cast(&mut g, garrison);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "drew Opt and discarded it");
    assert_eq!(named_count(&g, 0, "Human Soldier"), 1, "a nonland discard recruits");
    assert_eq!(pt(&g, garrison), (2, 5));
}

/// Dissenter's Deliverance kills an artifact; Beneath the Sands fetches a
/// tapped basic.
#[test]
fn jolrael_cyclers_destroy_and_ramp() {
    let mut g = pod(2);
    let rock = ready(&mut g, 1, catalog::network_terminal());
    let dd = g.add_card_to_hand(0, catalog::dissenters_deliverance());
    cast_at(&mut g, dd, Some(Target::Permanent(rock)), vec![]);
    assert!(g.battlefield_find(rock).is_none());
    let forest = g.add_card_to_library(0, catalog::forest());
    let sands = g.add_card_to_hand(0, catalog::beneath_the_sands());
    cast(&mut g, sands);
    assert!(g.battlefield_find(forest).is_some_and(|c| c.tapped));
}

/// Bebop gives Rhinos menace; Rocksteady stops Boars being blocked by more
/// than one creature.
#[test]
fn bebop_and_rocksteady_share_their_evasion() {
    let mut g = pod(2);
    let bebop = ready(&mut g, 0, catalog::bebop_warthog_warrior());
    let rock = ready(&mut g, 0, catalog::rocksteady_crash_courser());
    assert!(g.permanent_has_keyword(rock, &Keyword::Menace));
    assert!(g.permanent_has_keyword(bebop, &Keyword::CantBeBlockedByMoreThanOne));
}

/// Baleful Ammit shrinks your own creature; Fell Beast drains by its counters.
#[test]
fn baleful_ammit_and_fell_beast() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ammit = g.add_card_to_hand(0, catalog::baleful_ammit());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    cast(&mut g, ammit);
    assert_eq!(pt(&g, bear), (1, 1));
    let beast = ready(&mut g, 0, catalog::fell_beast_of_mordor());
    g.battlefield_find_mut(beast).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    declare(&mut g, vec![(beast, 1)]);
    assert_eq!((g.players[0].life, g.players[1].life), (mine + 2, theirs - 2));
}

/// The Destined Black Mage pings each opponent per noncreature spell.
#[test]
fn destined_black_mage_pings_each_opponent() {
    let mut g = pod(3);
    library(&mut g, 0, 2);
    ready(&mut g, 0, catalog::the_destined_black_mage());
    let lives = (g.players[1].life, g.players[2].life);
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt);
    assert_eq!((g.players[1].life, g.players[2].life), (lives.0 - 1, lives.1 - 1));
}

/// Stalactite Dagger makes a changeling and makes its bearer every type.
#[test]
fn stalactite_dagger_shapeshifts_its_bearer() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let dagger = g.add_card_to_hand(0, catalog::stalactite_dagger());
    cast(&mut g, dagger);
    assert_eq!(named_count(&g, 0, "Shapeshifter"), 1);
    g.battlefield_find_mut(dagger).unwrap().attached_to = Some(bear);
    assert_eq!(pt(&g, bear), (3, 3));
    assert!(g.computed_permanent(bear).is_some_and(|c| c.keywords().contains(&Keyword::Changeling)));
}

/// Loki steals the lone creature a spell of yours targets, untapped and hasty
/// on your turn; Bionic Blow pumps and punches.
#[test]
fn loki_steals_and_bionic_blow_punches() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::loki_god_of_lies());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    cast_at(&mut g, growth, Some(Target::Permanent(theirs)), vec![]);
    let c = g.battlefield_find(theirs).unwrap();
    assert_eq!(c.controller, 0);
    assert!(!c.tapped);
    assert!(g.permanent_has_keyword(theirs, &Keyword::Haste));
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let blow = g.add_card_to_hand(0, catalog::bionic_blow());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: blow,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(giant)],
        mode: None,
        x_value: Some(1),
    })
    .expect("blow");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none(), "3 power killed the Hill Giant");
}

/// Network Terminal loots by tapping another artifact.
#[test]
fn network_terminal_loots_off_another_artifact() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let term = ready(&mut g, 0, catalog::network_terminal());
    let other = ready(&mut g, 0, catalog::ornithopter());
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, term, 1, None);
    assert!(g.battlefield_find(other).unwrap().tapped);
    assert_eq!(g.players[0].graveyard.len(), 1, "drew, then discarded");
}

/// Urza's Command: a tapped Powerstone and a Construct that counts artifacts.
#[test]
fn urzas_command_makes_a_powerstone_and_a_construct() {
    let mut g = pod(2);
    let cmd = g.add_card_to_hand(0, catalog::urzas_command());
    flood(&mut g);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: cmd, spree_modes: vec![1, 2], target: None, additional_targets: vec![], x_value: None,
    })
    .expect("command");
    drain_stack(&mut g);
    let stone = g.battlefield.iter().find(|c| c.definition.name == "Powerstone").unwrap();
    assert!(stone.tapped);
    let construct = g.battlefield.iter().find(|c| c.definition.name == "Construct").map(|c| c.id).unwrap();
    assert_eq!(pt(&g, construct), (2, 2), "the Powerstone and itself");
    ready(&mut g, 0, catalog::ornithopter());
    assert_eq!(pt(&g, construct), (3, 3), "counted live");
}

/// Ajani Goldmane's −6 Avatar tracks your life total as it changes.
#[test]
fn ajani_goldmane_avatar_tracks_life() {
    let mut g = pod(2);
    let pw = ready(&mut g, 0, catalog::ajani_goldmane());
    g.battlefield_find_mut(pw).unwrap().add_counters(CounterType::Loyalty, 6);
    loyalty(&mut g, pw, 2, None);
    let avatar = g.battlefield.iter().find(|c| c.definition.name == "Avatar").map(|c| c.id).unwrap();
    let life = g.players[0].life;
    assert_eq!(pt(&g, avatar), (life, life));
    g.players[0].life -= 7;
    assert_eq!(pt(&g, avatar), (life - 7, life - 7));
}

/// Astonishing Spider-Man refills a discarded hand; Spider-Sense counters an
/// instant.
#[test]
fn astonishing_spider_man_and_spider_sense() {
    let mut g = pod(2);
    library(&mut g, 0, 5);
    let spidey = ready(&mut g, 0, catalog::astonishing_spider_man());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    declare(&mut g, vec![(spidey, 1)]);
    assert_eq!(g.players[0].graveyard.len(), 2);
    assert_eq!(g.players[0].hand.len(), 2);
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let opt = g.add_card_to_hand(0, catalog::opt());
    let sense = g.add_card_to_hand(0, catalog::spider_sense());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: opt, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("opt");
    g.perform_action(GameAction::CastSpell {
        card_id: sense, target: Some(Target::Permanent(opt)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("sense");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), 2, "Opt never resolved");
}

/// Centaur Omenreader discounts creatures by {2} only while tapped.
#[test]
fn centaur_omenreader_discounts_while_tapped() {
    let mut g = pod(2);
    let centaur = ready(&mut g, 0, catalog::centaur_omenreader());
    let spent = |g: &mut GameState| {
        let giant = g.add_card_to_hand(0, catalog::hill_giant());
        flood(g);
        let before = g.players[0].mana_pool.total();
        g.perform_action(GameAction::CastSpell { card_id: giant, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("cast");
        drain_stack(g);
        before - g.players[0].mana_pool.total()
    };
    assert_eq!(spent(&mut g), 4);
    g.battlefield_find_mut(centaur).unwrap().tapped = true;
    assert_eq!(spent(&mut g), 2);
}

/// Leaf-Crowned Elder casts a kin creature off the top for free.
#[test]
fn leaf_crowned_elder_casts_a_kin_for_free() {
    let mut g = pod(2);
    library(&mut g, 1, 3);
    library(&mut g, 0, 3);
    let kin = g.add_card_to_library(0, catalog::leaf_crowned_elder());
    g.players[0].library.rotate_right(1);
    ready(&mut g, 0, catalog::leaf_crowned_elder());
    step_to(&mut g, 1, TurnStep::Upkeep);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![kin])]));
    step_to(&mut g, 0, TurnStep::Upkeep);
    assert!(g.battlefield_find(kin).is_some(), "the Treefolk Shaman shares a type");
}

/// Elder of Laurels pumps per creature; Swashbuckler's Whip grants reach and
/// a tapper.
#[test]
fn elder_of_laurels_and_swashbucklers_whip() {
    let mut g = pod(2);
    let elder = ready(&mut g, 0, catalog::elder_of_laurels());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, elder, 0, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (4, 4));
    let whip = ready(&mut g, 0, catalog::swashbucklers_whip());
    g.battlefield_find_mut(whip).unwrap().attached_to = Some(elder);
    assert!(g.permanent_has_keyword(elder, &Keyword::Reach));
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    activate(&mut g, elder, 1, Some(Target::Permanent(theirs)));
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}

/// CR 604.3 / 613.4a — a token whose printed text says its power and
/// toughness *are* a count reads it live (a characteristic-defining ability),
/// not as a snapshot from when it was made: Kalonian Twingrove's Treefolk
/// grows with the next Forest.
#[test]
fn cr_604_3_kalonian_twingrove_token_counts_forests_live() {
    let mut g = pod(2);
    for _ in 0..2 {
        ready(&mut g, 0, catalog::forest());
    }
    let grove = g.add_card_to_hand(0, catalog::kalonian_twingrove());
    cast(&mut g, grove);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Treefolk Warrior").map(|c| c.id).unwrap();
    assert_eq!(pt(&g, token), (2, 2));
    ready(&mut g, 0, catalog::forest());
    assert_eq!(pt(&g, token), (3, 3));
}

// ── decks::cmdr_scan7k_k ─────────────────────────────────────────────────────

/// The Fantastic Four: the entry and each mana-value-4 spell pick a mode not
/// yet chosen this turn, so three triggers make three different choices;
/// Wall Off discounts per opposing creature.
#[test]
fn fantastic_four_rotates_modes_and_wall_off_discounts() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let life = g.players[1].life;
    let ff = g.add_card_to_hand(0, catalog::the_fantastic_four());
    cast(&mut g, ff);
    for _ in 0..2 {
        let giant = g.add_card_to_hand(0, catalog::hill_giant());
        cast(&mut g, giant);
    }
    let counters: u32 = g.battlefield.iter().filter(|c| c.controller == 0).map(|c| plus(&g, c.id)).sum();
    let used = [
        named_count(&g, 0, "Wall"),
        usize::from(g.players[1].life == life - 3),
        (counters / 2) as usize,
        3 - g.players[0].library.len(),
    ];
    assert!(used.iter().all(|&n| n <= 1), "no mode twice in a turn: {used:?}");
    assert_eq!(used.iter().sum::<usize>(), 3, "three triggers, three modes: {used:?}");
    for _ in 0..3 {
        ready(&mut g, 1, catalog::grizzly_bears());
    }
    let off = g.add_card_to_hand(0, catalog::wall_off());
    flood(&mut g);
    let before = g.players[0].mana_pool.total();
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell { card_id: off, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(before - g.players[0].mana_pool.total(), 1, "{{3}}{{W}} minus three");
    assert_eq!(g.players[0].life, life + 4);
}

/// Hedron Matrix adds its bearer's mana value; Solid Footing makes a vigilant
/// creature hit with its toughness.
#[test]
fn hedron_matrix_and_solid_footing() {
    let mut g = pod(2);
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let matrix = ready(&mut g, 0, catalog::hedron_matrix());
    g.battlefield_find_mut(matrix).unwrap().attached_to = Some(giant);
    assert_eq!(pt(&g, giant), (3 + 4, 3 + 4));
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let footing = g.add_card_to_hand(0, catalog::solid_footing());
    attach(&mut g, footing, bear);
    assert_eq!(pt(&g, bear), (3, 3));
    assert!(!g.permanent_has_keyword(bear, &Keyword::AssignsCombatDamageByToughness), "no vigilance");
    let angel = ready(&mut g, 0, catalog::serra_angel());
    let footing = g.add_card_to_hand(0, catalog::solid_footing());
    attach(&mut g, footing, angel);
    assert!(g.permanent_has_keyword(angel, &Keyword::AssignsCombatDamageByToughness));
}

/// Lost Isle Calling gathers a verse per scry and cashes them in — CR 608.2h,
/// the exile-itself cost leaves its verse count readable as last-known
/// information; Council's Deliberation comes back from the graveyard on a
/// scry.
#[test]
fn cr_608_2h_lost_isle_calling_and_councils_deliberation() {
    let mut g = pod(2);
    library(&mut g, 0, 8);
    let isle = ready(&mut g, 0, catalog::lost_isle_calling());
    ready(&mut g, 0, catalog::island());
    let council = g.add_card_to_graveyard(0, catalog::councils_deliberation());
    let top = g.players[0].library[0].id;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::ScryOrder { kept_top: vec![top], bottom: vec![] },
        DecisionAnswer::Bool(true),
    ]));
    let opt = g.add_card_to_hand(0, catalog::opt());
    let hand = g.players[0].hand.len();
    cast(&mut g, opt);
    assert_eq!(g.battlefield_find(isle).unwrap().counter_count(CounterType::Verse), 1);
    assert!(g.exile.iter().any(|c| c.id == council), "exiled from the graveyard");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "Opt's draw and the Deliberation's");
    g.battlefield_find_mut(isle).unwrap().add_counters(CounterType::Verse, 2);
    let hand = g.players[0].hand.len();
    activate(&mut g, isle, 0, None);
    assert_eq!(g.players[0].hand.len(), hand + 3);
}

/// The Astonishing Ant-Man grows on draws and splits into Insects; Colony
/// Commander makes an Insect on the first counter each turn.
#[test]
fn ant_men_grow_and_make_insects() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let ant = ready(&mut g, 0, catalog::the_astonishing_ant_man());
    let div = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, div);
    assert_eq!(plus(&g, ant), 2);
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ant, ability_index: 0, target: None, additional_targets: vec![], x_value: Some(2), mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(plus(&g, ant), 0);
    assert_eq!(named_count(&g, 0, "Insect"), 2);
    let mut g = pod(2);
    ready(&mut g, 0, catalog::ant_man_colony_commander());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..2 {
        let growth = g.add_card_to_hand(0, catalog::battlegrowth());
        cast_at(&mut g, growth, Some(Target::Permanent(bear)), vec![]);
    }
    assert_eq!(plus(&g, bear), 2);
    assert_eq!(named_count(&g, 0, "Insect"), 1, "only once each turn");
}

/// Invocation of Saint Traft's creature brings an attacking Angel, exiled at
/// end of combat; Dorothea is sacrificed after attacking.
#[test]
fn invocation_of_saint_traft_and_dorothea() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let inv = g.add_card_to_hand(0, catalog::invocation_of_saint_traft());
    attach(&mut g, inv, bear);
    let dot = ready(&mut g, 0, catalog::dorothea_vengeful_victim());
    declare(&mut g, vec![(bear, 1), (dot, 1)]);
    assert_eq!(named_count(&g, 0, "Angel"), 1);
    no_blocks_through_combat(&mut g);
    let evs = g.advance_step(Vec::new()).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(named_count(&g, 0, "Angel"), 0, "exiled at end of combat");
    assert!(g.battlefield_find(dot).is_none(), "Dorothea was sacrificed");
}

/// Jennifer Walters transforms into She-Hulk; Storm gives your targeted
/// creature flying.
#[test]
fn jennifer_walters_and_storm_windrider() {
    let mut g = pod(2);
    let jen = ready(&mut g, 0, catalog::jennifer_walters());
    activate(&mut g, jen, 0, None);
    assert_eq!(g.battlefield_find(jen).unwrap().definition.name, "The Sensational She-Hulk");
    assert_eq!(pt(&g, jen), (6, 6));
    ready(&mut g, 0, catalog::storm_windrider());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    cast_at(&mut g, growth, Some(Target::Permanent(bear)), vec![]);
    assert!(g.permanent_has_keyword(bear, &Keyword::Flying));
}

/// Niambi bounces your creature for life; Linvala's sacrifice protects the
/// team.
#[test]
fn niambi_bounces_for_life_and_linvala_protects() {
    let mut g = pod(2);
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let niambi = g.add_card_to_hand(0, catalog::niambi_esteemed_speaker());
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(giant))]));
    cast(&mut g, niambi);
    assert!(g.players[0].hand.iter().any(|c| c.id == giant));
    assert_eq!(g.players[0].life, life + 4);
    let linvala = ready(&mut g, 0, catalog::linvala_shield_of_sea_gate());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    activate(&mut g, linvala, 0, None);
    assert!(g.permanent_has_keyword(bear, &Keyword::Indestructible));
}

// ── decks::cmdr_oct15_a ──────────────────────────────────────────────────────

/// Invasion of Fiora: "choose one or both" — picking only the nonlegendary
/// wipe spares the legends.
#[test]
fn invasion_of_fiora_one_mode_spares_the_legends() {
    let mut g = pod(2);
    let legend = ready(&mut g, 1, catalog::isamaru_hound_of_konda());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let inv = g.add_card_to_hand(0, catalog::invasion_of_fiora());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![1])]));
    cast(&mut g, inv);
    assert!(g.battlefield_find(legend).is_some(), "legendary survives");
    assert!(g.battlefield_find(bear).is_none(), "nonlegendary destroyed");
}

fn marchesa(g: &mut GameState) -> CardId {
    let back = *catalog::invasion_of_fiora().back_face.expect("back face");
    ready(g, 0, back)
}

/// Marchesa, Resolute Monarch — CR 603.4: the upkeep draw is an intervening
/// "if" on not having been dealt combat damage since your last turn.
#[test]
fn marchesa_draws_unless_combat_damaged_since_your_turn() {
    let mut g = pod(2);
    library(&mut g, 0, 5);
    library(&mut g, 1, 5);
    marchesa(&mut g);
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    let hand = g.players[0].hand.len();
    step_to(&mut g, 0, TurnStep::Draw);
    assert_eq!(g.players[0].hand.len(), hand + 2, "undamaged: the upkeep draw and the draw step's");
    assert_eq!(g.players[0].life, 19);

    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    step_to(&mut g, 1, TurnStep::End);
    assert_eq!(g.players[0].life, 17, "the Bear connected");
    let hand = g.players[0].hand.len();
    step_to(&mut g, 0, TurnStep::Draw);
    assert_eq!(g.players[0].hand.len(), hand + 1, "damaged: the draw step's only");
}

/// Invasion of Segovia: two trampling Krakens; Caetus untaps up to four
/// creatures at your end step.
#[test]
fn invasion_of_segovia_krakens_and_caetus_untaps() {
    let mut g = pod(2);
    let inv = g.add_card_to_hand(0, catalog::invasion_of_segovia());
    cast(&mut g, inv);
    let krakens: Vec<CardId> = g.battlefield.iter().filter(|c| c.definition.name == "Kraken").map(|c| c.id).collect();
    assert_eq!(krakens.len(), 2);
    assert!(g.permanent_has_keyword(krakens[0], &Keyword::Trample));
    let caetus = ready(&mut g, 0, *catalog::invasion_of_segovia().back_face.unwrap());
    for &k in &krakens {
        g.battlefield_find_mut(k).unwrap().tapped = true;
    }
    g.battlefield_find_mut(caetus).unwrap().tapped = true;
    step_to(&mut g, 0, TurnStep::End);
    assert!(krakens.iter().chain([&caetus]).all(|&k| !g.battlefield_find(k).unwrap().tapped), "untapped at end step");
}

/// Gathering Place: its {G}/{W} abilities need a basic land or the turn it
/// entered; {C} always works.
#[test]
fn gathering_place_colors_need_a_basic_or_a_fresh_entry() {
    let mut g = pod(2);
    let gp = g.add_card_to_battlefield(0, catalog::gathering_place());
    g.battlefield_find_mut(gp).unwrap().entered_turn = None;
    let green = GameAction::ActivateAbility { card_id: gp, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None };
    assert!(g.perform_action(green.clone()).is_err(), "no basic, not fresh: no {{G}}");
    g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(green).expect("a basic land unlocks {G}");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

/// In Bolas's Clutches steals a permanent and makes it legendary.
#[test]
fn in_bolass_clutches_steals_and_makes_legendary() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::in_bolass_clutches());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
    assert!(g.computed_permanent(bear).unwrap().supertypes().contains(&crabomination::card::Supertype::Legendary));
}

/// Roku's Mastery scries 2 only at X >= 4 (the scripted scry bottoms both).
#[test]
fn rokus_mastery_scries_at_four() {
    for (x, scried) in [(3, false), (4, true)] {
        let mut g = pod(2);
        library(&mut g, 0, 4);
        let top = g.players[0].library[0].id;
        let wurm = ready(&mut g, 1, catalog::craw_wurm());
        let spell = g.add_card_to_hand(0, catalog::rokus_mastery());
        flood(&mut g);
        let bottom: Vec<CardId> = g.players[0].library.iter().take(2).map(|c| c.id).collect();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder { kept_top: vec![], bottom }]));
        g.perform_action(GameAction::CastSpell { card_id: spell, target: Some(Target::Permanent(wurm)), additional_targets: vec![], mode: None, x_value: Some(x) })
            .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(wurm).map(|c| c.damage), if x >= 4 { None } else { Some(x) });
        assert_eq!(g.players[0].library[0].id != top, scried, "X = {x}");
    }
}

/// Pet Avengers' power-up: a counter and a 3/2 vigilant Hero, once.
#[test]
fn pet_avengers_power_up_makes_a_hero() {
    let mut g = pod(2);
    let pa = ready(&mut g, 0, catalog::pet_avengers());
    g.battlefield_find_mut(pa).unwrap().entered_turn = None;
    activate(&mut g, pa, 0, None);
    assert_eq!(plus(&g, pa), 1);
    assert_eq!(named_count(&g, 0, "Hero"), 1);
}

/// Pelia bounces an opponent's creature; Happy Hogan tucks one second from
/// the top (yes) or to the bottom.
#[test]
fn pelia_bounces_and_happy_hogan_tucks() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let pelia = g.add_card_to_hand(0, catalog::pelia_immortal_innovator());
    cast(&mut g, pelia);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bounced to its owner's hand");

    let mut g = pod(2);
    library(&mut g, 1, 3);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let hogan = g.add_card_to_hand(0, catalog::happy_hogan_bodyguard());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, hogan);
    assert_eq!(g.players[1].library.get(1).map(|c| c.id), Some(bear), "second from the top");
}

/// Forebear's Blade moves to another creature you control when its bearer dies.
#[test]
fn forebears_blade_follows_to_a_new_bearer() {
    let mut g = pod(2);
    let first = ready(&mut g, 0, catalog::grizzly_bears());
    let heir = ready(&mut g, 0, catalog::craw_wurm());
    let blade = g.add_card_to_battlefield(0, catalog::forebears_blade());
    g.battlefield_find_mut(blade).unwrap().attached_to = Some(first);
    assert_eq!(pt(&g, first), (5, 2));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(first)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(first).is_none());
    assert_eq!(g.battlefield_find(blade).unwrap().attached_to, Some(heir));
    assert_eq!(pt(&g, heir), (9, 4));
}

/// Rune of Flight relays flying through an Equipment to its creature.
#[test]
fn rune_of_flight_on_equipment_flies_the_bearer() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let blade = g.add_card_to_battlefield(0, catalog::forebears_blade());
    g.battlefield_find_mut(blade).unwrap().attached_to = Some(bear);
    let rune = g.add_card_to_hand(0, catalog::rune_of_flight());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: rune, target: Some(Target::Permanent(blade)), additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(bear, &Keyword::Flying));
    assert_eq!(g.players[0].hand.len(), 1, "drew on entry");
}

/// Orchard Warden gains a Treefolk's toughness; Ulvenwald Observer draws
/// when a toughness-4 creature dies, not a 2/2.
#[test]
fn colfenor_treefolk_pair() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    ready(&mut g, 0, catalog::orchard_warden());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let observer = g.add_card_to_hand(0, catalog::ulvenwald_observer());
    cast(&mut g, observer);
    assert_eq!(g.players[0].life, 26, "gained the Observer's 6 toughness");
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    for id in [bear, observer] {
        let bolt = g.add_card_to_hand(0, catalog::murder());
        flood(&mut g);
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(id)), additional_targets: vec![], mode: None, x_value: None })
            .expect("murder");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1, "only the 6/6's death draws");
}

/// True Polymorph turns one creature into a copy of another.
#[test]
fn true_polymorph_copies_the_second_target() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let tp = g.add_card_to_hand(0, catalog::true_polymorph());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: tp, target: Some(Target::Permanent(bear)), additional_targets: vec![Target::Permanent(wurm)], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (6, 4));
}

/// Iceman and Firestar: a blue spell taps a creature.
#[test]
fn iceman_taps_on_a_blue_spell() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    ready(&mut g, 0, catalog::iceman_and_firestar());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let forge = g.add_card_to_hand(0, catalog::futurist_forge());
    cast(&mut g, forge);
    assert!(g.battlefield_find(wurm).unwrap().tapped);
}

/// The Royal Scions' −8: draw four, then the reflexive trigger deals damage
/// equal to the hand size.
#[test]
fn royal_scions_ultimate_burns_for_hand_size() {
    let mut g = pod(3);
    library(&mut g, 0, 6);
    let pw = ready(&mut g, 0, catalog::the_royal_scions());
    g.battlefield_find_mut(pw).unwrap().add_counters(CounterType::Loyalty, 5);
    loyalty(&mut g, pw, 2, None);
    assert_eq!(g.players[0].hand.len(), 4);
    assert_eq!(g.players[0].life, 20);
    assert_eq!(g.players[1].life + g.players[2].life, 36, "four damage to an opponent");
}

/// Katerina: the chosen letter (scripted "G") makes a Cat off Grizzly Bears,
/// not off Llanowar Elves.
#[test]
fn katerina_makes_a_cat_off_the_chosen_letter() {
    let mut g = pod(2);
    let kat = g.add_card_to_hand(0, catalog::katerina_of_myras_marvels());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(6)]));
    cast(&mut g, kat);
    for (def, cats) in [(catalog::llanowar_elves(), 0), (catalog::grizzly_bears(), 1)] {
        let id = g.add_card_to_hand(0, def);
        cast(&mut g, id);
        assert_eq!(named_count(&g, 0, "Cat"), cats);
    }
}

/// Headless, Katerina names the commonest initial among her controller's spells.
#[test]
fn katerina_headless_pick_is_the_commonest_initial() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    g.add_card_to_library(0, catalog::grizzly_bears());
    let kat = g.add_card_to_hand(0, catalog::katerina_of_myras_marvels());
    cast(&mut g, kat);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(named_count(&g, 0, "Cat"), 1, "L for Lightning");
}

/// Safe Passage prevents a Bolt to you and to your creature.
#[test]
fn safe_passage_shields_you_and_your_creatures() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sp = g.add_card_to_hand(0, catalog::safe_passage());
    cast(&mut g, sp);
    for t in [Target::Permanent(bear), Target::Player(0)] {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        flood(&mut g);
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(t), additional_targets: vec![], mode: None, x_value: None })
            .expect("bolt");
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(g.players[0].life, 20);
}

/// Dwalin hones each Equipment you control; each hone counter is +1/+0.
#[test]
fn dwalin_hone_counters_pump_the_bearer() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let blade = g.add_card_to_battlefield(0, catalog::forebears_blade());
    g.battlefield_find_mut(blade).unwrap().attached_to = Some(bear);
    let dwalin = g.add_card_to_hand(0, catalog::dwalin_weaponmaster());
    cast(&mut g, dwalin);
    assert_eq!(g.battlefield_find(blade).unwrap().counter_count(CounterType::Hone), 1);
    assert_eq!(pt(&g, bear), (6, 2), "2 + 3 Blade + 1 hone");
}

/// Thorin: with an enduring story (three legendaries), your artifacts and
/// creatures have ward {1}.
#[test]
fn thorin_grants_ward_with_an_enduring_story() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let thorin = g.add_card_to_hand(0, catalog::thorin_oakenshield());
    cast(&mut g, thorin);
    let ward = || Keyword::Ward(crabomination::card::WardCost::generic(1));
    assert!(!g.permanent_has_keyword(bear, &ward()), "no story yet");
    ready(&mut g, 0, catalog::isamaru_hound_of_konda());
    ready(&mut g, 0, catalog::dwalin_weaponmaster());
    g.check_state_based_actions();
    assert!(g.permanent_has_keyword(bear, &ward()));
}

/// Quake taps a creature whenever you cast a noncreature spell.
#[test]
fn quake_taps_on_a_noncreature_spell() {
    let mut g = pod(3);
    library(&mut g, 0, 2);
    ready(&mut g, 0, catalog::quake_agent_of_shield());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let forge = g.add_card_to_hand(0, catalog::futurist_forge());
    cast(&mut g, forge);
    assert!(g.battlefield_find(wurm).unwrap().tapped);
}

/// Denzilore Fatehold counters the team on a scry.
#[test]
fn denzilore_counters_on_scry() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    let den = ready(&mut g, 0, catalog::denzilore_fatehold());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let spell = g.add_card_to_hand(0, catalog::rokus_mastery());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: spell, target: Some(Target::Permanent(wurm)), additional_targets: vec![], mode: None, x_value: Some(4) })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!((plus(&g, den), plus(&g, bear)), (1, 1));
}

/// Gorging Vulture gains a life per creature card milled.
#[test]
fn gorging_vulture_counts_milled_creatures() {
    let mut g = pod(2);
    for def in [catalog::grizzly_bears(), catalog::island(), catalog::craw_wurm(), catalog::island()] {
        g.add_card_to_library(0, def);
    }
    let v = g.add_card_to_hand(0, catalog::gorging_vulture());
    cast(&mut g, v);
    assert_eq!(g.players[0].life, 22);
    assert_eq!(g.players[0].graveyard.len(), 4);
}

/// Psionic Pulse counters a noncreature spell and exiles the Dragon on an adventure.
#[test]
fn sapphire_dragon_psionic_pulse_counters() {
    let mut g = pod(2);
    let forge = g.add_card_to_hand(1, catalog::futurist_forge());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: forge, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("forge");
    let dragon = g.add_card_to_hand(0, catalog::sapphire_dragon());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastAdventure { card_id: dragon, target: Some(Target::Permanent(forge)), additional_targets: vec![], mode: None, x_value: None })
        .expect("pulse");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != forge), "countered");
    assert!(g.exile.iter().any(|c| c.id == dragon), "on an adventure");
}

/// Shortcut to Mushrooms counters a creature at end step after a permanent
/// of yours left the battlefield.
#[test]
fn shortcut_to_mushrooms_counters_after_a_departure() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    library(&mut g, 1, 3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sm = g.add_card_to_hand(0, catalog::shortcut_to_mushrooms());
    cast(&mut g, sm);
    step_to(&mut g, 0, TurnStep::End);
    assert_eq!(plus(&g, bear), 0, "nothing left: no counter");
    let forge = ready(&mut g, 0, catalog::futurist_forge());
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    activate(&mut g, forge, 0, None);
    step_to(&mut g, 0, TurnStep::End);
    assert!(plus(&g, bear) >= 1, "the Forge left");
}

/// Restoration Specialist returns an artifact and an enchantment.
#[test]
fn restoration_specialist_returns_both_halves() {
    let mut g = pod(2);
    let rs = ready(&mut g, 0, catalog::restoration_specialist());
    let art = g.add_card_to_graveyard(0, catalog::futurist_forge());
    let ench = g.add_card_to_graveyard(0, catalog::shortcut_to_mushrooms());
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rs, ability_index: 0, target: Some(Target::Permanent(art)), additional_targets: vec![Target::Permanent(ench)], x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let hand: Vec<CardId> = g.players[0].hand.iter().map(|c| c.id).collect();
    assert!(hand.contains(&art) && hand.contains(&ench), "{hand:?}");
}

/// Meddle moves a single-creature-target spell to another creature, and
/// leaves a spell aimed at a player alone (the 2004-10-04 ruling).
#[test]
fn meddle_moves_a_creature_target_only() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("bolt");
    let meddle = g.add_card_to_hand(0, catalog::meddle());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: meddle, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None }).expect("meddle");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "the Bear is spared");
    assert_eq!(g.battlefield_find(wurm).unwrap().damage, 3, "the Bolt hit the Wurm");

    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None }).expect("bolt");
    let meddle = g.add_card_to_hand(0, catalog::meddle());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: meddle, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None }).expect("meddle");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 17, "a player target isn't Meddle's");
}

/// Mizzium Meddler pulls a removal spell onto itself.
#[test]
fn mizzium_meddler_redirects_to_itself() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("bolt");
    let mm = g.add_card_to_hand(0, catalog::mizzium_meddler());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: mm, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("meddler");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(g.battlefield_find(mm).unwrap().damage, 3);
}

/// The auto-targeter fills both of Restoration Specialist's optional slots.
#[test]
fn restoration_specialist_auto_targets_both() {
    let mut g = pod(2);
    let rs = ready(&mut g, 0, catalog::restoration_specialist());
    let art = g.add_card_to_graveyard(0, catalog::futurist_forge());
    let ench = g.add_card_to_graveyard(0, catalog::shortcut_to_mushrooms());
    let def = catalog::restoration_specialist();
    let (t0, extra) = g.auto_targets_for_effect_all_slots_x(&def.activated_abilities[0].effect, 0, None, false, Some(rs), None);
    assert_eq!(t0, Some(Target::Permanent(art)));
    assert_eq!(extra, vec![Target::Permanent(ench)]);
}
