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
