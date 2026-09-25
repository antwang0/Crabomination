//! Commander: The Hosts of Mordor precon (LTC, Sauron, Lord of the Rings,
//! `decks::cmdr_sauron`).

use crabomination::card::{CardId, CardType, CounterType, CreatureType, Keyword};
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

fn activate_x(g: &mut GameState, seat: usize, id: CardId, index: usize, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// Seat 0's Army, if any, and its power.
fn army_power(g: &GameState) -> Option<i32> {
    let army = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)
    })?;
    Some(pt(g, army.id).0)
}

fn attack(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn combat_damage(g: &mut GameState) {
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

fn kill(g: &mut GameState, id: CardId) {
    let mut evs = Vec::new();
    g.destroy_permanent(id, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn controller_of(g: &GameState, id: CardId) -> Option<usize> {
    g.battlefield_find(id).map(|c| c.controller)
}

/// Sauron — the cast trigger amasses five, mills five and brings a creature
/// back; an opponent's commander dying tempts you.
#[test]
fn sauron_musters_and_reanimates() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let giant = g.add_card_to_library(0, catalog::hill_giant());
    let s = g.add_card_to_hand(0, catalog::sauron_lord_of_the_rings());
    cast(&mut g, 0, s, None).expect("sauron");
    assert_eq!(army_power(&g), Some(5));
    assert_eq!(controller_of(&g, giant), Some(0), "milled, then returned");
    assert!(g.battlefield_find(s).is_some());
    // An opponent's commander dies: the Ring tempts you.
    let cmdr = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[1].commanders.push(cmdr);
    let before = g.players[0].ring_temptations;
    kill(&mut g, cmdr);
    assert_eq!(g.players[0].ring_temptations, before + 1);
}

/// Cavern-Hoard Dragon — cheaper per artifact of the most-artifacted
/// opponent; a Treasure per artifact the damaged player controls.
#[test]
fn cavern_hoard_dragon_loots_artifacts() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::ornithopter());
    }
    let d = g.add_card_to_hand(0, catalog::cavern_hoard_dragon());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: d, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{4}{R}{R} after three artifacts");
    drain_stack(&mut g);
    attack(&mut g, &[d], 1);
    combat_damage(&mut g);
    assert_eq!(named(&g, 0, "Treasure").len(), 3);
}

/// Corsairs of Umbar — combat damage amasses three; {2}{U} makes an Orc
/// unblockable.
#[test]
fn corsairs_amass_on_damage() {
    let mut g = pod(2);
    let c = g.add_card_to_battlefield(0, catalog::corsairs_of_umbar());
    attack(&mut g, &[c], 1);
    combat_damage(&mut g);
    assert_eq!(army_power(&g), Some(3));
    let army = g.battlefield.iter().find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Army)).unwrap().id;
    flood(&mut g, 0);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: c,
        ability_index: 0,
        target: Some(Target::Permanent(army)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("unblockable");
    drain_stack(&mut g);
    assert!(g.computed_permanent(army).unwrap().keywords().contains(&Keyword::Unblockable), "the Army is an Orc");
}

/// Fiery Inscription — tempts on entry, then each instant or sorcery burns
/// every opponent for 2.
#[test]
fn fiery_inscription_burns_on_spells() {
    let mut g = pod(3);
    let f = g.add_card_to_hand(0, catalog::fiery_inscription());
    cast(&mut g, 0, f, None).expect("inscription");
    assert_eq!(g.players[0].ring_temptations, 1);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let d = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, 0, d, None).expect("divination");
    assert_eq!((g.players[1].life, g.players[2].life), (18, 18));
}

/// Grishnákh — amass two, then steal a creature with power at most the
/// Army's: untapped and hasty, back at end of turn.
#[test]
fn grishnakh_steals_within_the_armys_power() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.battlefield_find_mut(bears).unwrap().tapped = true;
    let gr = g.add_card_to_hand(0, catalog::grishnakh_brash_instigator());
    cast(&mut g, 0, gr, None).expect("grishnakh");
    assert_eq!(army_power(&g), Some(2));
    assert_eq!(controller_of(&g, bears), Some(0), "power 2 ≤ the 2/2 Army");
    assert_eq!(controller_of(&g, giant), Some(1), "power 3 is too big");
    let b = g.battlefield_find(bears).unwrap();
    assert!(!b.tapped);
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Haste));
}

/// Gríma — its hit makes the player dig to an instant or sorcery you cast
/// free; the misses and an uncast find go to the bottom.
#[test]
fn grima_casts_the_found_spell() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    let div = g.add_card_to_library(1, catalog::divination());
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::island());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let gr = g.add_card_to_battlefield(0, catalog::grima_sarumans_footman());
    let hand = g.players[0].hand.len();
    attack(&mut g, &[gr], 1);
    combat_damage(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2, "Divination cast free for you");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == div), "an opponent's card goes to their graveyard");
    assert_eq!(g.players[1].library.len(), 5, "the two misses are back on the bottom");
}

/// In the Darkness Bind Them — a Wraith and a temptation per early chapter;
/// chapter IV steals a creature from each opponent.
#[test]
fn in_the_darkness_bind_them_chapters() {
    let mut g = pod(3);
    let saga = g.add_card_to_hand(0, catalog::in_the_darkness_bind_them());
    cast(&mut g, 0, saga, None).expect("saga");
    assert_eq!(named(&g, 0, "Wraith").len(), 1);
    assert_eq!(g.players[0].ring_temptations, 1);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::hill_giant());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    g.saga_advance(saga);
    drain_stack(&mut g);
    assert_eq!((controller_of(&g, a), controller_of(&g, b)), (Some(0), Some(0)));
}

/// Knollspine Dragon — swaps a hand for a card per damage dealt to the
/// opponent this turn.
#[test]
fn knollspine_dragon_refills() {
    let mut g = pod(2);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::island());
    g.players[1].damage_taken_this_turn = 5;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.players[1].life = 15;
    let k = g.add_card_to_hand(0, catalog::knollspine_dragon());
    cast(&mut g, 0, k, Some(Target::Player(1))).expect("knollspine");
    assert_eq!(g.players[0].hand.len(), 5, "one card discarded, five drawn");
}

/// Lidless Gaze — each player's top card is yours to play, with any mana.
#[test]
fn lidless_gaze_steals_the_tops() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let theirs = g.add_card_to_library(1, catalog::grizzly_bears());
    let lg = g.add_card_to_hand(0, catalog::lidless_gaze());
    cast(&mut g, 0, lg, None).expect("gaze");
    assert!(g.exile.iter().any(|c| c.id == theirs));
    // {1}{G} with only red: mana of any type.
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Red, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: theirs,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the opponent's bears");
    drain_stack(&mut g);
    assert_eq!(controller_of(&g, theirs), Some(0));
}

/// Lord of the Nazgûl — spells make Wraiths, and Wraiths have protection
/// from Ring-bearers.
#[test]
fn lord_of_the_nazgul_raises_wraiths() {
    let mut g = pod(2);
    let lord = g.add_card_to_battlefield(0, catalog::lord_of_the_nazgul());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let d = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, 0, d, None).expect("divination");
    let wraith = named(&g, 0, "Wraith")[0];
    let bearer = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[1].ring_bearer = Some(bearer);
    for id in [lord, wraith] {
        let kws = g.computed_permanent(id).unwrap().keywords().to_vec();
        assert!(kws.iter().any(|k| matches!(k, Keyword::ProtectionFromMatching(_))), "{kws:?}");
    }
    assert!(g.is_a_ring_bearer(bearer));
}

/// Lord of the Nazgûl — nine Wraiths are 9/9s.
#[test]
fn nine_wraiths_are_nine_nine() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::lord_of_the_nazgul());
    let d = g.add_card_to_hand(0, catalog::divination());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    for _ in 0..7 {
        let d = g.add_card_to_hand(0, catalog::shock());
        cast(&mut g, 0, d, Some(Target::Player(1))).expect("shock");
    }
    cast(&mut g, 0, d, None).expect("divination");
    let wraiths = named(&g, 0, "Wraith");
    assert_eq!(wraiths.len(), 8, "eight tokens + the Lord");
    assert_eq!(pt(&g, wraiths[0]), (9, 9));
}

/// Monstrosity of the Lake — {5} taps and stuns the other side.
#[test]
fn monstrosity_of_the_lake_stuns() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::monstrosity_of_the_lake());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    cast(&mut g, 0, m, None).expect("monstrosity");
    let b = g.battlefield_find(bears).unwrap();
    assert!(b.tapped);
    assert_eq!(b.counter_count(CounterType::Stun), 1);
}

/// Moria Scavenger — discarding a creature draws and amasses; anything else
/// just draws.
#[test]
fn moria_scavenger_loots_orcs() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let m = g.add_card_to_battlefield(0, catalog::moria_scavenger());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.clear_sickness(m);
    activate_x(&mut g, 0, m, 0, None).expect("creature discard");
    assert_eq!(army_power(&g), Some(1));
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"));
    g.battlefield_find_mut(m).unwrap().tapped = false;
    activate_x(&mut g, 0, m, 1, None).expect("plain discard");
    assert_eq!(army_power(&g), Some(1));
}

/// Orcish Siegemaster — tramples its Orcs and attacks for the biggest power.
#[test]
fn orcish_siegemaster_hits_hard() {
    let mut g = pod(2);
    let s = g.add_card_to_battlefield(0, catalog::orcish_siegemaster());
    let scav = g.add_card_to_battlefield(0, catalog::moria_scavenger());
    g.add_card_to_battlefield(0, catalog::hill_giant());
    assert!(g.computed_permanent(scav).unwrap().keywords().contains(&Keyword::Trample));
    attack(&mut g, &[s], 1);
    assert_eq!(pt(&g, s), (3, 5));
}

/// Rampaging War Mammoth — cycling for X destroys up to X artifacts.
#[test]
fn rampaging_war_mammoth_cycles_into_artifact_removal() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(1, catalog::sol_ring());
    let b = g.add_card_to_battlefield(1, catalog::mind_stone());
    let c = g.add_card_to_battlefield(1, catalog::ornithopter());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let left = |g: &GameState| [a, b, c].iter().filter(|&&id| g.battlefield_find(id).is_some()).count();
    // X = 0 destroys nothing; X = 2 destroys up to two.
    let m = g.add_card_to_hand(0, catalog::rampaging_war_mammoth());
    flood(&mut g, 0);
    g.perform_action(GameAction::Cycle { card_id: m, x_value: Some(0) }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(left(&g), 3);
    let m = g.add_card_to_hand(0, catalog::rampaging_war_mammoth());
    flood(&mut g, 0);
    g.perform_action(GameAction::Cycle { card_id: m, x_value: Some(2) }).expect("cycle");
    drain_stack(&mut g);
    assert!((1..=2).contains(&(3 - left(&g))), "one or two destroyed");
}

/// Relic of Sauron — two mana of {U}/{B}/{R}; {3}, {T}: loot two.
#[test]
fn relic_of_sauron_taps_for_two() {
    let mut g = pod(2);
    let relic = g.add_card_to_battlefield(0, catalog::relic_of_sauron());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::island());
    activate_x(&mut g, 0, relic, 1, None).expect("loot");
    assert_eq!(g.players[0].hand.len(), 2, "one + two − one");
}

/// Revenge of Ravens — each attacker drains its controller for you.
#[test]
fn revenge_of_ravens_taxes_attackers() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::revenge_of_ravens());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::hill_giant());
    attack(&mut g, &[a, b], 1);
    assert_eq!((g.players[0].life, g.players[1].life), (18, 22));
}

/// Saruman — a noncreature spell amasses its mana value; Orcs have ward.
#[test]
fn saruman_amasses_per_spell() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::saruman_the_white_hand());
    let fi = g.add_card_to_hand(0, catalog::fiery_inscription());
    cast(&mut g, 0, fi, None).expect("inscription");
    assert_eq!(army_power(&g), Some(3));
    let army = g.battlefield.iter().find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Army)).unwrap().id;
    assert!(g.computed_permanent(army).unwrap().keywords().iter().any(|k| matches!(k, Keyword::Ward(_))));
}

/// Shelob — an opponent's dead creature is exiled with her; she eats one for
/// counters and a card, or puts one onto the battlefield for X.
#[test]
fn shelob_weaves_the_dead() {
    let mut g = pod(2);
    let shelob = g.add_card_to_battlefield(0, catalog::shelob_dread_weaver());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    for id in [bears, giant] {
        kill(&mut g, id);
    }
    assert!(g.exile.iter().any(|c| c.id == bears) && g.exile.iter().any(|c| c.id == giant));
    activate_x(&mut g, 0, shelob, 1, Some(4)).expect("X = 4, Hill Giant");
    assert_eq!(controller_of(&g, giant), Some(0));
    assert!(g.battlefield_find(giant).unwrap().tapped);
    let hand = g.players[0].hand.len();
    activate_x(&mut g, 0, shelob, 0, None).expect("feed");
    assert_eq!(pt(&g, shelob), (5, 5));
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears));
    assert!(activate_x(&mut g, 0, shelob, 0, None).is_err(), "nothing left to feed");
}

/// Subjugate the Hobbits — every small noncommander creature changes sides.
#[test]
fn subjugate_the_hobbits_takes_the_small() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cmdr = g.add_card_to_battlefield(1, catalog::goblin_guide());
    g.players[1].commanders.push(cmdr);
    let big = g.add_card_to_battlefield(1, catalog::serra_angel());
    let s = g.add_card_to_hand(0, catalog::subjugate_the_hobbits());
    cast(&mut g, 0, s, None).expect("subjugate");
    assert_eq!(
        (controller_of(&g, bears), controller_of(&g, cmdr), controller_of(&g, big)),
        (Some(0), Some(1), Some(1))
    );
}

/// Summons of Saruman — amass X, mill X, cast a cheap enough milled spell
/// free.
#[test]
fn summons_of_saruman_casts_from_the_mill() {
    let mut g = pod(2);
    // Top to bottom: Consider and an Island are milled, then three more.
    let consider = g.add_card_to_library(0, catalog::consider());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let s = g.add_card_to_hand(0, catalog::summons_of_saruman());
    let hand = g.players[0].hand.len();
    cast_x(&mut g, 0, s, None, Some(2)).expect("summons");
    assert_eq!(army_power(&g), Some(2));
    assert_eq!(g.players[0].hand.len(), hand, "Summons left, Consider (mana value 1) drew one");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == consider), "cast, then to the graveyard");
}

/// The Balrog of Moria — its death exiles itself and a creature per
/// opponent; cycling makes two Treasures.
#[test]
fn the_balrog_takes_them_with_it() {
    let mut g = pod(3);
    let balrog = g.add_card_to_battlefield(0, catalog::the_balrog_of_moria());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::hill_giant());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    kill(&mut g, balrog);
    assert!(g.exile.iter().any(|c| c.id == balrog));
    assert!(g.exile.iter().any(|c| c.id == a) && g.exile.iter().any(|c| c.id == b));
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let m = g.add_card_to_hand(0, catalog::the_balrog_of_moria());
    flood(&mut g, 0);
    g.perform_action(GameAction::Cycle { card_id: m, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Treasure").len(), 2);
}

/// The Mouth of Sauron — the target mills three; amass per instant and
/// sorcery in their graveyard.
#[test]
fn the_mouth_of_sauron_counts_spells() {
    let mut g = pod(2);
    g.add_card_to_graveyard(1, catalog::shock());
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::divination());
    g.add_card_to_library(1, catalog::lightning_bolt());
    let m = g.add_card_to_hand(0, catalog::the_mouth_of_sauron());
    cast(&mut g, 0, m, Some(Target::Player(1))).expect("mouth");
    assert_eq!(g.players[1].graveyard.len(), 4);
    assert_eq!(army_power(&g), Some(3));
}

/// Too Greedily, Too Deep — steal a graveyard creature and let it hit every
/// other creature for its power.
#[test]
fn too_greedily_too_deep_reanimates_and_blasts() {
    let mut g = pod(2);
    let angel = g.add_card_to_graveyard(1, catalog::hill_giant());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let t = g.add_card_to_hand(0, catalog::too_greedily_too_deep());
    cast(&mut g, 0, t, Some(Target::Permanent(angel))).expect("too greedily");
    assert_eq!(controller_of(&g, angel), Some(0));
    assert!(g.battlefield_find(bears).is_none());
    assert_eq!(g.battlefield_find(wurm).unwrap().damage, 3, "Hill Giant hits for 3");
}

/// Treason of Isengard — a spell back on top, and two Orcs.
#[test]
fn treason_of_isengard_restacks() {
    let mut g = pod(2);
    let shock = g.add_card_to_graveyard(0, catalog::shock());
    let t = g.add_card_to_hand(0, catalog::treason_of_isengard());
    cast(&mut g, 0, t, Some(Target::Permanent(shock))).expect("treason");
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(shock));
    assert_eq!(army_power(&g), Some(2));
}

/// Wake the Dragon — a 6/6 that takes an artifact from each player it hits.
#[test]
fn wake_the_dragon_hoards() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let w = g.add_card_to_hand(0, catalog::wake_the_dragon());
    cast(&mut g, 0, w, None).expect("wake");
    let dragon = named(&g, 0, "Dragon")[0];
    assert_eq!(pt(&g, dragon), (6, 6));
    assert!(g.computed_permanent(dragon).unwrap().card_types().contains(&CardType::Creature));
    attack(&mut g, &[dragon], 1);
    combat_damage(&mut g);
    assert_eq!(controller_of(&g, ring), Some(0));
}
