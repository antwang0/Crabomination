//! Commander: the Raining Cats and Dogs deck (SLD, Rin and Seri,
//! `decks::cmdr_rinseri`).

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn cast(g: &mut GameState, id: CardId) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate_x(
    g: &mut GameState,
    id: CardId,
    index: usize,
    target: Option<Target>,
    x_value: Option<u32>,
) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    activate_x(g, id, index, target, None)
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn has(g: &GameState, id: CardId, k: &Keyword) -> bool {
    g.computed_permanent(id).expect("on the battlefield").keywords().contains(k)
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn attack_with(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for a in attackers {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn block_and_finish_combat(g: &mut GameState, defender: usize, blocks: Vec<(CardId, CardId)>) {
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(blocks)).expect("blocks");
    drain_stack(g);
    advance_to(g, TurnStep::PostCombatMain);
}

/// A Dog spell makes a Cat, a Cat spell makes a Dog; the tap ability deals
/// damage per Dog and gains life per Cat (Rin and Seri count as both).
#[test]
fn rin_and_seri_trade_cats_for_dogs_and_count_both() {
    let mut g = main_phase(2);
    let rs = g.add_card_to_battlefield(0, catalog::rin_and_seri_inseparable());
    let dog_spell = g.add_card_to_hand(0, catalog::pack_leader());
    cast(&mut g, dog_spell);
    assert_eq!(named(&g, 0, "Cat").len(), 1, "a Dog spell made a Cat");
    let cat_spell = g.add_card_to_hand(0, catalog::skyhunter_strike_force());
    cast(&mut g, cat_spell);
    assert_eq!(named(&g, 0, "Dog").len(), 1, "a Cat spell made a Dog");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear);
    assert_eq!(named(&g, 0, "Cat").len() + named(&g, 0, "Dog").len(), 2, "a Bear makes nothing");
    g.clear_sickness(rs);
    let (life0, life1) = (g.players[0].life, g.players[1].life);
    activate(&mut g, rs, 0, Some(Target::Player(1))).expect("activate");
    // Dogs: Rin and Seri, Pack Leader, the Dog token. Cats: Rin and Seri,
    // Skyhunter Strike Force, the Cat token.
    assert_eq!(g.players[1].life, life1 - 3);
    assert_eq!(g.players[0].life, life0 + 3);
}

/// CR 613.4c — each of Jetmir's tiers switches on at its own creature count.
#[test]
fn jetmir_stacks_its_tiers_by_creature_count() {
    let mut g = main_phase(2);
    let jetmir = g.add_card_to_battlefield(0, catalog::jetmir_nexus_of_revels());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, jetmir), (5, 4), "two creatures: nothing yet");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, jetmir), (6, 4));
    assert!(has(&g, jetmir, &Keyword::Vigilance) && !has(&g, jetmir, &Keyword::Trample));
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    assert_eq!(pt(&g, jetmir), (7, 4));
    assert!(has(&g, jetmir, &Keyword::Trample) && !has(&g, jetmir, &Keyword::DoubleStrike));
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    assert_eq!(pt(&g, jetmir), (8, 4));
    assert!(has(&g, jetmir, &Keyword::DoubleStrike));
}

/// CR 614.1a — Jinnie Fay turns a 1/1 Citizen into a bigger Cat or Dog; the
/// tie between them goes to the Cat's haste.
#[test]
fn cr_614_1a_jinnie_fay_replaces_a_small_token() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::jinnie_fay_jetmirs_second());
    let kitt = g.add_card_to_hand(0, catalog::kitt_kanto_mayhem_diva());
    cast(&mut g, kitt);
    assert!(named(&g, 0, "Citizen").is_empty(), "the Citizen was replaced");
    let cats = named(&g, 0, "Cat");
    assert_eq!(cats.len(), 1);
    assert_eq!(pt(&g, cats[0]), (2, 2));
    assert!(has(&g, cats[0], &Keyword::Haste));
}

/// On another player's turn, Kitt Kanto taps two of your creatures to pump
/// and goad one of that player's creatures (CR 701.15).
#[test]
fn cr_701_15_kitt_kanto_goads_the_active_players_creature() {
    let mut g = main_phase(3);
    let kitt = g.add_card_to_battlefield(0, catalog::kitt_kanto_mayhem_diva());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    advance_to(&mut g, TurnStep::BeginCombat);
    assert!(g.battlefield_find(kitt).unwrap().tapped && g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(pt(&g, theirs), (6, 6));
    assert!(has(&g, theirs, &Keyword::Trample));
    assert!(g.battlefield_find(theirs).unwrap().goaded_by.contains(&0));
}

/// Parley — lands revealed make Citizens (which Phabine gives haste), nonland
/// cards pump your creatures, then everyone draws.
#[test]
fn phabine_parleys_at_the_beginning_of_combat() {
    let mut g = main_phase(3);
    let phabine = g.add_card_to_battlefield(0, catalog::phabine_bosss_confidant());
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(1, catalog::forest());
    g.add_card_to_library(2, catalog::grizzly_bears());
    advance_to(&mut g, TurnStep::BeginCombat);
    let citizens = named(&g, 0, "Citizen");
    assert_eq!(citizens.len(), 2, "two lands revealed");
    assert!(has(&g, citizens[0], &Keyword::Haste));
    assert_eq!(pt(&g, phabine), (4, 7), "one nonland revealed");
    assert_eq!(pt(&g, citizens[0]), (2, 2));
    for p in 0..3 {
        assert_eq!(g.players[p].hand.len(), 1, "seat {p} drew");
    }
}

/// Animal Sanctuary's counter needs one of its six animal types.
#[test]
fn animal_sanctuary_grows_only_its_animals() {
    let mut g = main_phase(2);
    let land = g.add_card_to_battlefield(0, catalog::animal_sanctuary());
    let cat = g.add_card_to_battlefield(0, catalog::feline_sovereign());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(activate(&mut g, land, 1, Some(Target::Permanent(bear))).is_err(), "a Bear isn't an animal here");
    activate(&mut g, land, 1, Some(Target::Permanent(cat))).expect("a Cat is");
    assert_eq!(g.battlefield_find(cat).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Bloodline Pretender grows when another creature of its chosen type enters.
#[test]
fn bloodline_pretender_grows_on_its_chosen_type() {
    let mut g = main_phase(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Cat)]));
    let bp = g.add_card_to_hand(0, catalog::bloodline_pretender());
    cast(&mut g, bp);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear);
    assert_eq!(g.battlefield_find(bp).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    let cat = g.add_card_to_hand(0, catalog::skyhunter_strike_force());
    cast(&mut g, cat);
    assert_eq!(g.battlefield_find(bp).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Feline Sovereign pumps other Cats, and Cats connecting destroy an
/// artifact of the damaged player's.
#[test]
fn feline_sovereign_pumps_cats_and_breaks_an_artifact() {
    let mut g = main_phase(3);
    let sov = g.add_card_to_battlefield(0, catalog::feline_sovereign());
    let cat = g.add_card_to_battlefield(0, catalog::skyhunter_strike_force());
    assert_eq!(pt(&g, cat), (3, 3));
    assert_eq!(pt(&g, sov), (2, 3), "not itself");
    assert!(has(&g, cat, &Keyword::ProtectionFromCreatureType(CreatureType::Dog)));
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let other_ring = g.add_card_to_battlefield(2, catalog::sol_ring());
    attack_with(&mut g, &[cat], 1);
    block_and_finish_combat(&mut g, 1, vec![]);
    assert!(g.battlefield_find(ring).is_none(), "the damaged player's artifact is gone");
    assert!(g.battlefield_find(other_ring).is_some());
}

/// Channel — Greater Tanuki fetches a basic land tapped from hand.
#[test]
fn greater_tanuki_channels_for_a_basic() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::forest());
    let t = g.add_card_to_hand(0, catalog::greater_tanuki());
    activate(&mut g, t, 0, None).expect("channel");
    let forests = named(&g, 0, "Forest");
    assert_eq!(forests.len(), 1);
    assert!(g.battlefield_find(forests[0]).unwrap().tapped);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == t));
}

/// Highcliff Felidar destroys each opponent's greatest-power creature.
#[test]
fn highcliff_felidar_destroys_each_opponents_biggest() {
    let mut g = main_phase(3);
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel());
    let (a1, b1) = (g.add_card_to_battlefield(1, catalog::serra_angel()), g.add_card_to_battlefield(1, catalog::grizzly_bears()));
    let (a2, b2) = (g.add_card_to_battlefield(2, catalog::serra_angel()), g.add_card_to_battlefield(2, catalog::grizzly_bears()));
    let f = g.add_card_to_hand(0, catalog::highcliff_felidar());
    cast(&mut g, f);
    assert!(g.battlefield_find(a1).is_none() && g.battlefield_find(a2).is_none());
    assert!(g.battlefield_find(b1).is_some() && g.battlefield_find(b2).is_some());
    assert!(g.battlefield_find(mine).is_some());
}

/// Komainu Battle Armor connecting goads every creature the damaged player
/// controls.
#[test]
fn cr_701_15_komainu_goads_the_damaged_players_creatures() {
    let mut g = main_phase(3);
    let armor = g.add_card_to_battlefield(0, catalog::komainu_battle_armor());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bystander = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    attack_with(&mut g, &[armor], 1);
    block_and_finish_combat(&mut g, 1, vec![]);
    assert!(g.battlefield_find(theirs).unwrap().goaded_by.contains(&0));
    assert!(g.battlefield_find(bystander).unwrap().goaded_by.is_empty());
}

/// Equipped via reconfigure, the armor's bonus and goad ride the host.
#[test]
fn komainu_reconfigured_rides_its_host() {
    let mut g = main_phase(3);
    let armor = g.add_card_to_battlefield(0, catalog::komainu_battle_armor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Reconfigure { equipment: armor, target: Some(bear) }).expect("reconfigure");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (4, 4));
    assert!(has(&g, bear, &Keyword::Menace));
    attack_with(&mut g, &[bear], 1);
    block_and_finish_combat(&mut g, 1, vec![]);
    assert!(g.battlefield_find(theirs).unwrap().goaded_by.contains(&0));
}

/// CR 613.4b — Mirror Entity sets base P/T X/X and grants every type.
#[test]
fn mirror_entity_sets_base_pt_and_all_types() {
    let mut g = main_phase(2);
    let me = g.add_card_to_battlefield(0, catalog::mirror_entity());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate_x(&mut g, me, 0, None, Some(4)).expect("activate");
    assert_eq!(pt(&g, bear), (4, 4));
    assert_eq!(pt(&g, me), (4, 4));
    assert!(has(&g, bear, &Keyword::Changeling));
}

/// Nacatl War-Pride makes one attacking copy per creature the defending
/// player controls, exiled at the next end step.
#[test]
fn nacatl_war_pride_copies_per_defending_creature() {
    let mut g = main_phase(3);
    let nacatl = g.add_card_to_battlefield(0, catalog::nacatl_war_pride());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let second = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    attack_with(&mut g, &[nacatl], 1);
    let copies: Vec<CardId> = named(&g, 0, "Nacatl War-Pride").into_iter().filter(|&c| c != nacatl).collect();
    assert_eq!(copies.len(), 2, "defending player controls two creatures");
    for c in &copies {
        assert!(g.battlefield_find(*c).unwrap().tapped);
        assert!(g.attacking.iter().any(|a| a.attacker == *c), "attacking");
    }
    // CR 509.1c — each War-Pride, copies too, must be blocked if able.
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![])).is_err(), "must be blocked");
    block_and_finish_combat(&mut g, 1, vec![(blocker, nacatl), (second, copies[0])]);
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Nacatl War-Pride"), vec![nacatl], "the copies were exiled");
}

/// CR 615.1 — Pack Leader's attack shields your Dogs from combat damage.
#[test]
fn cr_615_1_pack_leader_shields_dogs_in_combat() {
    let mut g = main_phase(2);
    let leader = g.add_card_to_battlefield(0, catalog::pack_leader());
    let tanuki = g.add_card_to_battlefield(0, catalog::greater_tanuki());
    assert_eq!(pt(&g, tanuki), (7, 6));
    assert_eq!(pt(&g, leader), (2, 2), "not itself");
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    attack_with(&mut g, &[leader], 1);
    block_and_finish_combat(&mut g, 1, vec![(angel, leader)]);
    let l = g.battlefield_find(leader).expect("Pack Leader survived the Angel");
    assert_eq!(l.damage, 0);
}

/// Showdown of the Skalds I exiles four you may play; II grows a creature per
/// spell you cast that turn.
#[test]
fn showdown_of_the_skalds_impulses_then_grows() {
    let mut g = main_phase(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let saga = g.add_card_to_hand(0, catalog::showdown_of_the_skalds());
    cast(&mut g, saga);
    assert_eq!(g.players[0].library.len(), 1, "four exiled");
    let exiled = g.exile.iter().find(|c| c.owner == 0).map(|c| c.id).expect("an exiled Bear");
    flood(&mut g, 0);
    let pool = g.players[0].mana_pool.total();
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: exiled,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("play it from exile");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), pool - 2, "paying its own cost");
    assert!(g.battlefield_find(exiled).is_some(), "played from exile");
    // Chapter II, on your next turn.
    g.add_card_to_library(1, catalog::plains());
    advance_to(&mut g, TurnStep::End);
    while !(g.active_player_idx == 0 && g.step == TurnStep::PreCombatMain) {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(saga).unwrap().counter_count(CounterType::Lore), 2);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear);
    let grown: u32 = [exiled, bear]
        .iter()
        .map(|&id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(grown, 1, "one spell, one counter");
}

/// Lieutenant — Skyhunter gives your other creatures melee only while you
/// control your commander.
#[test]
fn skyhunter_strike_force_lieutenant_grants_melee() {
    let mut g = main_phase(3);
    let sky = g.add_card_to_battlefield(0, catalog::skyhunter_strike_force());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(has(&g, sky, &Keyword::Melee));
    assert!(!has(&g, bear, &Keyword::Melee), "no commander yet");
    let cmd = g.add_card_to_battlefield(0, catalog::rin_and_seri_inseparable());
    g.players[0].commanders.push(cmd);
    assert!(has(&g, bear, &Keyword::Melee));
}
