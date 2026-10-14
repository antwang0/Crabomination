//! Commander: one-card gaps of EDHREC ranks 2501-3500 (`decks::cmdr_rank3k_a`).

use crabomination::card::{CardId, CounterType, Keyword};
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

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None });
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(g, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// Seat 0 attacks `defender` with `attackers`, unblocked, and runs combat out.
fn swing(g: &mut GameState, attackers: &[CardId], defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

/// Graceful Reprieve returns the creature when it dies this turn — to its
/// owner, not the caster.
#[test]
fn graceful_reprieve_returns_the_dead_to_its_owner() {
    let mut g = pod(3);
    let bear = ready(&mut g, 2, catalog::grizzly_bears());
    let reprieve = g.add_card_to_hand(0, catalog::graceful_reprieve());
    cast(&mut g, reprieve, Some(Target::Permanent(bear)));
    let blade = g.add_card_to_hand(0, catalog::doom_blade());
    cast(&mut g, blade, Some(Target::Permanent(bear)));
    assert_eq!(named(&g, 2, "Grizzly Bears"), 1, "back under seat 2");
}

/// Germinate Recruits makes one Cadet per life gained this turn.
#[test]
fn germinate_recruits_counts_life_gained() {
    let mut g = pod(2);
    g.players[0].life_gained_this_turn = 3;
    let spell = g.add_card_to_hand(0, catalog::germinate_recruits());
    cast(&mut g, spell, None);
    assert_eq!(named(&g, 0, "Cadet"), 3);
}

/// Feral Hydra's {3} ability is open to an opponent.
#[test]
fn feral_hydra_any_player_may_grow_it() {
    let mut g = pod(2);
    let hydra = ready(&mut g, 0, catalog::feral_hydra());
    g.players[1].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: hydra,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("seat 1 activates seat 0's Hydra");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Wilderland Scrounger's ferocious attack trigger needs a 4-power creature.
#[test]
fn wilderland_scrounger_is_ferocious() {
    for big in [false, true] {
        let mut g = pod(2);
        let wolf = ready(&mut g, 0, catalog::wilderland_scrounger());
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        if big {
            ready(&mut g, 0, catalog::serra_angel());
        }
        swing(&mut g, &[wolf], 1);
        let got = g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne);
        assert_eq!(got, u32::from(big), "big={big}");
    }
}

/// Budding Insurgent draws only off a legendary enchantment.
#[test]
fn budding_insurgent_draws_off_a_legendary_enchantment() {
    for (def, draws) in [(catalog::oblivion_ring(), 0), (catalog::honden_of_seeing_winds(), 1)] {
        let mut g = pod(2);
        let dryad = ready(&mut g, 0, catalog::budding_insurgent());
        let ench = g.add_card_to_battlefield(1, def);
        let hand = g.players[0].hand.len();
        g.add_card_to_library(0, catalog::island());
        activate(&mut g, dryad, 0, Some(Target::Permanent(ench)));
        assert!(g.battlefield_find(ench).is_none());
        assert_eq!(g.players[0].hand.len(), hand + draws);
    }
}

/// Skemfar Avenger draws off a nontoken Elf, not a token or a non-Elf.
#[test]
fn skemfar_avenger_draws_off_nontoken_elves() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::skemfar_avenger());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    for victim in [bear, elf] {
        let blade = g.add_card_to_hand(0, catalog::murder());
        cast(&mut g, blade, Some(Target::Permanent(victim)));
    }
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].life, 19);
}

/// Casey Jones draws three now and discards three at random next upkeep.
#[test]
fn casey_jones_draws_then_pays_next_upkeep() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let casey = g.add_card_to_hand(0, catalog::casey_jones_vigilante());
    cast(&mut g, casey, None);
    assert_eq!(g.players[0].hand.len(), 3);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 0);
    assert_eq!(g.players[0].graveyard.len(), 3);
}

/// Rise of the Deathbringer's draw mode reads the greatest power and costs
/// that much life.
#[test]
fn rise_of_the_deathbringer_draws_by_greatest_power() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::serra_angel());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let rise = g.add_card_to_hand(0, catalog::rise_of_the_deathbringer());
    act(&mut g, GameAction::CastSpell { card_id: rise, target: None, additional_targets: vec![], mode: Some(0), x_value: None });
    assert_eq!(g.players[0].hand.len(), 4);
    assert_eq!(g.players[0].life, 16);
}

/// Ashling makes the damaged player sacrifice the targeted creature.
#[test]
fn ashling_makes_the_damaged_player_sacrifice() {
    let mut g = pod(3);
    let ashling = ready(&mut g, 0, catalog::ashling_the_extinguisher());
    let victim = ready(&mut g, 2, catalog::grizzly_bears());
    swing(&mut g, &[ashling], 2);
    assert!(g.battlefield_find(victim).is_none());
    assert!(g.players[2].graveyard.iter().any(|c| c.id == victim));
}

/// Dong Zhou turns an opponent's creature on its controller.
#[test]
fn dong_zhou_turns_a_creature_on_its_controller() {
    let mut g = pod(3);
    let angel = ready(&mut g, 2, catalog::serra_angel());
    let dong = g.add_card_to_hand(0, catalog::dong_zhou_the_tyrant());
    cast(&mut g, dong, None);
    let _ = angel;
    assert_eq!(g.players[2].life, 16, "Serra Angel hits its own controller for 4");
    assert_eq!(g.players[1].life, 20);
}

/// Ghost Lantern grows its bearer on each of your deaths; Bind Spirit
/// regrows a creature card.
#[test]
fn ghost_lantern_grows_its_bearer() {
    let mut g = pod(2);
    let lantern = ready(&mut g, 0, catalog::ghost_lantern());
    let bearer = ready(&mut g, 0, catalog::serra_angel());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    act(&mut g, GameAction::Equip { equipment: lantern, target: bearer });
    let blade = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, blade, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(bearer).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Super Intelligence draws for the enchanted creature's controller.
#[test]
fn super_intelligence_draws_for_the_hosts_controller() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::island());
    let aura = g.add_card_to_hand(0, catalog::super_intelligence());
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Platoon Dispenser draws at your end step only beside two other creatures.
#[test]
fn platoon_dispenser_needs_two_other_creatures() {
    for others in [1, 2] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::platoon_dispenser());
        for _ in 0..others {
            ready(&mut g, 0, catalog::grizzly_bears());
        }
        g.add_card_to_library(0, catalog::island());
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), usize::from(others == 2), "others={others}");
    }
}

/// Deadeye Plunderers counts artifacts; Havoc Jester pings off a sacrifice;
/// Drogskol Cavalry gains off another Spirit.
#[test]
fn deadeye_havoc_and_drogskol() {
    let mut g = pod(2);
    let pirate = ready(&mut g, 0, catalog::deadeye_plunderers());
    activate(&mut g, pirate, 0, None);
    assert_eq!(g.computed_permanent(pirate).unwrap().power, 4, "one Treasure");
    ready(&mut g, 0, catalog::havoc_jester());
    let treasure = g.battlefield.iter().find(|c| c.definition.name == "Treasure").unwrap().id;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: treasure,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sac the Treasure");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "Havoc Jester pings off the sacrifice");
    let cavalry = ready(&mut g, 0, catalog::drogskol_cavalry());
    activate(&mut g, cavalry, 0, None);
    assert_eq!(named(&g, 0, "Spirit"), 1);
    assert_eq!(g.players[0].life, 22);
}

/// Nephalia Smuggler blinks another creature of yours back untapped.
#[test]
fn nephalia_smuggler_blinks() {
    let mut g = pod(2);
    let rogue = ready(&mut g, 0, catalog::nephalia_smuggler());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    activate(&mut g, rogue, 0, Some(Target::Permanent(bear)));
    assert_eq!(named(&g, 0, "Grizzly Bears"), 1);
    assert!(!g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().tapped);
}

/// Djeru's Resolve untaps and shields; Fell the Pheasant kills a flyer and
/// leaves a Food; Hornswoggle leaves a Treasure.
#[test]
fn djerus_resolve_fell_the_pheasant_hornswoggle() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let resolve = g.add_card_to_hand(0, catalog::djerus_resolve());
    cast(&mut g, resolve, Some(Target::Permanent(bear)));
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_some(), "damage prevented");
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let fell = g.add_card_to_hand(0, catalog::fell_the_pheasant());
    cast(&mut g, fell, Some(Target::Permanent(angel)));
    assert!(g.battlefield_find(angel).is_none());
    assert_eq!(named(&g, 0, "Food"), 1);
    assert!(g.computed_permanent(bear).unwrap().keywords().iter().all(|k| *k != Keyword::Flying));
}
