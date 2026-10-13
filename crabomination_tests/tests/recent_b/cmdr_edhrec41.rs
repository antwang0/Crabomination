//! Commander: Abigale Poet Laureate's, Grist's and Gandalf the Grey's last
//! gaps (`decks::cmdr_edhrec41`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g);
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn p1p1(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// CR 603.3d — Generous Pup shares its counters once a turn; Hagra
/// Constrictor gives menace to each creature with a +1/+1 counter.
#[test]
fn cr_603_3d_generous_pup_shares_once_a_turn() {
    let mut g = pod(2);
    let pup = ready(&mut g, 0, catalog::generous_pup());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let snake = g.add_card_to_hand(0, catalog::hagra_constrictor());
    cast(&mut g, snake, None).expect("snake");
    assert_eq!(p1p1(&g, snake), 2);
    let ward = g.add_card_to_hand(0, catalog::battlegrowth());
    cast(&mut g, ward, Some(Target::Permanent(pup))).expect("first counter");
    assert_eq!(p1p1(&g, bear), 1, "the pup shared");
    assert_eq!(p1p1(&g, snake), 3);
    let again = g.add_card_to_hand(0, catalog::battlegrowth());
    cast(&mut g, again, Some(Target::Permanent(pup))).expect("second counter");
    assert_eq!(p1p1(&g, bear), 1, "only once each turn");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Menace));
    let plain = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(plain).unwrap().keywords().contains(&Keyword::Menace));
}

/// Leader's Talent: level 3 puts a counter on each creature you control per
/// spell; level 2 gains 2 life when one with a counter leaves.
#[test]
fn leaders_talent_levels_up() {
    let mut g = pod(2);
    let talent = g.add_card_to_hand(0, catalog::leaders_talent());
    cast(&mut g, talent, None).expect("talent");
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g);
    act(&mut g, GameAction::ActivateAbility { card_id: talent, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None }).expect("level 2");
    act(&mut g, GameAction::ActivateAbility { card_id: talent, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None }).expect("level 3");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    assert_eq!(p1p1(&g, bear), 1);
    let life = g.players[0].life;
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(bear))).expect("murder");
    assert_eq!(g.players[0].life, life + 2, "it had a counter");
}

/// Grist, Voracious Larva transforms when a creature of yours returns from
/// the graveyard; the Plague Swarm's +1 makes an Insect and mills two.
#[test]
fn grist_transforms_on_a_reanimation() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let grist = ready(&mut g, 0, catalog::grist_voracious_larva());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let raise = g.add_card_to_hand(0, catalog::zombify());
    cast(&mut g, raise, Some(Target::Permanent(bear))).expect("zombify");
    assert!(g.battlefield_find(bear).is_some());
    let gid = g.battlefield.iter().find(|c| c.definition.name == "Grist, the Plague Swarm").map(|c| c.id);
    let gid = gid.unwrap_or(grist);
    assert_eq!(g.battlefield_find(gid).unwrap().definition.name, "Grist, the Plague Swarm");
    let lib = g.players[0].library.len();
    act(&mut g, GameAction::ActivateLoyaltyAbility { card_id: gid, ability_index: 0, target: None, x_value: None }).expect("+1");
    assert_eq!(named(&g, 0, "Insect").len(), 1);
    assert_eq!(g.players[0].library.len(), lib - 2);
}

/// CR 603.2c — Amzu: once a turn, cards leaving your graveyard make an
/// Insect with counters equal to the greatest mana value among them.
#[test]
fn cr_603_2c_amzu_counts_the_biggest_card() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::amzu_swarms_hunger());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let raise = g.add_card_to_hand(0, catalog::raise_dead());
    cast(&mut g, raise, Some(Target::Permanent(wurm))).expect("raise dead");
    let insects = named(&g, 0, "Insect");
    assert_eq!(insects.len(), 1);
    assert_eq!(p1p1(&g, insects[0]), 6, "Craw Wurm's mana value");
    let again = g.add_card_to_hand(0, catalog::raise_dead());
    cast(&mut g, again, Some(Target::Permanent(bear))).expect("raise dead");
    assert_eq!(named(&g, 0, "Insect").len(), 1, "only once each turn");
}

/// Zask: another Insect dying goes to the bottom of its library, then two
/// mill.
#[test]
fn zask_recycles_dead_insects() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    ready(&mut g, 0, catalog::zask_skittering_swarmlord());
    let amzu = ready(&mut g, 0, catalog::amzu_swarms_hunger());
    let lib = g.players[0].library.len();
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(amzu))).expect("murder");
    assert!(g.players[0].library.iter().any(|c| c.id == amzu), "on the bottom of the library");
    assert_eq!(g.players[0].library.len(), lib + 1 - 2);
}

/// CR 700.2 — Gandalf the Grey picks a mode that hasn't been chosen each
/// time: the second instant can't pick the same one again.
#[test]
fn cr_700_2_gandalf_never_repeats_a_mode() {
    let mut g = pod(2);
    let gandalf = ready(&mut g, 0, catalog::gandalf_the_grey());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    let picks = g.battlefield_find(gandalf).map(|c| c.modes_chosen.clone()).expect("still here");
    assert_eq!(picks.len(), 1);
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt2, Some(Target::Player(1))).expect("bolt");
    if let Some(c) = g.battlefield_find(gandalf) {
        assert_eq!(c.modes_chosen.len(), 2);
        assert_ne!(c.modes_chosen[0], c.modes_chosen[1]);
    }
}

/// Glamdring: first strike, +1/+0 per instant and sorcery in your graveyard;
/// Glamdring, Foe-hammer makes instants cheaper by the bearer's power.
#[test]
fn glamdring_scales_and_foe_hammer_discounts() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sword = ready(&mut g, 0, catalog::glamdring());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::divination());
    flood(&mut g);
    act(&mut g, GameAction::Equip { equipment: sword, target: bear }).expect("equip");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4);
    assert!(cp.keywords().contains(&Keyword::FirstStrike));
    let hammer = ready(&mut g, 0, catalog::glamdring_foe_hammer());
    let other = ready(&mut g, 0, catalog::craw_wurm());
    flood(&mut g);
    act(&mut g, GameAction::Equip { equipment: hammer, target: other }).expect("equip");
    // A 6-power bearer takes all of Divination's {2}: {U} alone casts it.
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Blue, 1);
    let div = g.add_card_to_hand(0, catalog::divination());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: div, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{U} is enough");
}
