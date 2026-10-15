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
/// original's target by default).
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
