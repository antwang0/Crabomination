//! Commander: the top 600's next EDHREC gaps
//! (`decks::cmdr_edhrec17`).

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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_as(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_as(g, 0, id, targets)
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn perm(id: CardId) -> Target {
    Target::Permanent(id)
}

fn tokens_named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.is_token && c.definition.name == name).count()
}

fn declare(g: &mut GameState, seat: usize, attacks: &[(CardId, usize)]) -> Result<(), String> {
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.iter().map(|&(a, d)| Attack { attacker: a, target: AttackTarget::Player(d) }).collect(),
    ))
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn on_bf(g: &GameState, id: CardId) -> bool {
    g.battlefield_find(id).is_some()
}


/// CR 707.10 — Storm King's Thunder copies the next instant or sorcery X
/// times, its own X read as it resolved (not the storm count).
#[test]
fn cr_707_10_storm_kings_thunder_copies_x_times() {
    let mut g = pod(2);
    let skt = g.add_card_to_hand(0, catalog::storm_kings_thunder());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: skt, target: None, additional_targets: vec![], mode: None, x_value: Some(2) })
        .expect("thunder");
    drain_stack(&mut g);
    let life = g.players[1].life;
    let b = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, b, &[Target::Player(1)]).expect("bolt");
    assert_eq!(g.players[1].life, life - 9, "the Bolt and two copies");
}

/// Release the Gremlins: X artifacts destroyed, X Gremlins made.
#[test]
fn release_the_gremlins_trades_artifacts_for_gremlins() {
    let mut g = pod(2);
    let a = ready(&mut g, 1, catalog::sol_ring());
    let b = ready(&mut g, 1, catalog::sol_ring());
    let rg = g.add_card_to_hand(0, catalog::release_the_gremlins());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: rg, target: Some(perm(a)), additional_targets: vec![perm(b)], mode: None, x_value: Some(2) })
        .expect("gremlins");
    drain_stack(&mut g);
    assert!(!on_bf(&g, a) && !on_bf(&g, b));
    assert_eq!(tokens_named(&g, "Gremlin"), 2);
}

/// Game of Chaos ends with the stakes moved between the two players; the
/// total life is conserved whatever the flips did.
#[test]
fn game_of_chaos_moves_life_between_the_two() {
    let mut g = pod(2);
    let (a, b) = (g.players[0].life, g.players[1].life);
    let gc = g.add_card_to_hand(0, catalog::game_of_chaos());
    cast(&mut g, gc, &[Target::Player(1)]).expect("chaos");
    assert_ne!(g.players[0].life, a, "at least one flip happened");
    assert_eq!(g.players[0].life + g.players[1].life, a + b);
}

/// Molten Birth makes two Elementals; Creepy Doll is indestructible.
#[test]
fn yusri_coin_cards() {
    let mut g = pod(2);
    let mb = g.add_card_to_hand(0, catalog::molten_birth());
    cast(&mut g, mb, &[]).expect("birth");
    assert_eq!(tokens_named(&g, "Elemental"), 2);
    let d = ready(&mut g, 0, catalog::creepy_doll());
    assert!(g.computed_permanent(d).unwrap().keywords().contains(&Keyword::Indestructible));
}

/// Thran Portal: the chosen basic type adds its mana ability, and tapping it
/// costs 1 life.
#[test]
fn thran_portal_taps_for_life() {
    let mut g = pod(2);
    let tp = g.add_card_to_hand(0, catalog::thran_portal());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(tp)).expect("land");
    drain_stack(&mut g);
    g.battlefield_find_mut(tp).unwrap().tapped = false;
    let cp = g.computed_permanent(tp).unwrap();
    assert!(cp.subtypes().land_types.len() >= 2, "Gate plus the chosen basic type");
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tp,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap");
    assert_eq!(g.players[0].life, life - 1);
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Glaive of the Guildpact scales with Gates.
#[test]
fn glaive_counts_gates() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::thran_portal());
    let gl = ready(&mut g, 0, catalog::glaive_of_the_guildpact());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: gl, target: bear }).expect("equip");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3);
    assert!(cp.keywords().contains(&Keyword::Menace));
}

/// CR 702.167c — Altar of the Wretched crafts a Bonemass as big as the
/// creature exiled to make it.
#[test]
fn cr_702_167c_wretched_bonemass_is_the_crafted_power() {
    let mut g = pod(2);
    let altar = ready(&mut g, 0, catalog::altar_of_the_wretched());
    g.add_card_to_graveyard(0, catalog::relic_golem());
    activate(&mut g, altar, 0, None).expect("craft");
    let cp = g.computed_permanent(altar).expect("the Bonemass");
    assert_eq!((cp.power, cp.toughness), (6, 6));
}

/// Wretched Bonemass has each listed keyword an exiled card used to craft it
/// has: a Serra Angel lends flying and vigilance, nothing lends trample.
#[test]
fn wretched_bonemass_has_its_crafted_cards_keywords() {
    let mut g = pod(2);
    let altar = ready(&mut g, 0, catalog::altar_of_the_wretched());
    g.add_card_to_graveyard(0, catalog::serra_angel());
    activate(&mut g, altar, 0, None).expect("craft");
    let cp = g.computed_permanent(altar).expect("the Bonemass");
    assert_eq!(cp.def.name, "Wretched Bonemass");
    let kws = cp.keywords();
    assert!(kws.contains(&Keyword::Flying) && kws.contains(&Keyword::Vigilance));
    assert!(!kws.contains(&Keyword::Trample));
}

/// Death-Priest of Myrkul pumps Zombies and buys a Skeleton after a death.
#[test]
fn death_priest_of_myrkul() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::death_priest_of_myrkul());
    let z = ready(&mut g, 0, catalog::tattered_mummy());
    assert_eq!(g.computed_permanent(z).unwrap().power, 2);
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[perm(b)]).expect("bolt");
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(tokens_named(&g, "Skeleton"), 1);
}

/// Tuya Bearclaw borrows the biggest other power; Ulvenwald Bear's morbid.
#[test]
fn duskana_bears() {
    let mut g = pod(2);
    let t = ready(&mut g, 0, catalog::tuya_bearclaw());
    ready(&mut g, 0, catalog::relic_golem());
    declare(&mut g, 0, &[(t, 1)]).ok();
    assert_eq!(g.computed_permanent(t).unwrap().power, 8);
    let mut g = pod(2);
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[perm(b)]).expect("bolt");
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let ub = g.add_card_to_hand(0, catalog::ulvenwald_bear());
    cast(&mut g, ub, &[perm(mine)]).expect("bear");
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Wild Pair finds a creature with the same total power and toughness.
#[test]
fn wild_pair_finds_a_twin() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::tattered_mummy()); // 1/2: total 3
    g.add_card_to_library(0, catalog::fume_spitter()); // 1/1: total 2
    g.add_card_to_library(0, catalog::grizzly_bears()); // 2/2: total 4
    ready(&mut g, 0, catalog::wild_pair());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, &[]).expect("bear");
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2, "the library's 2/2 joined it");
}

/// Ammit Eternal shrinks per opposing spell; Splendid Agony splits counters.
#[test]
fn hapatra_counters() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::ammit_eternal());
    let b = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, b, &[Target::Player(0)]).expect("bolt");
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
    let x = ready(&mut g, 1, catalog::grizzly_bears());
    let y = ready(&mut g, 1, catalog::grizzly_bears());
    let sa = g.add_card_to_hand(0, catalog::splendid_agony());
    cast(&mut g, sa, &[perm(x), perm(y)]).expect("agony");
    let total: u32 = [x, y].iter().map(|id| g.battlefield_find(*id).map_or(0, |c| c.counter_count(CounterType::MinusOneMinusOne))).sum();
    assert_eq!(total, 2);
}

/// Lethal Sting blights 1 as its cost.
#[test]
fn lethal_sting_costs_a_counter() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::relic_golem());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let ls = g.add_card_to_hand(0, catalog::lethal_sting());
    cast(&mut g, ls, &[perm(theirs)]).expect("sting");
    assert!(!on_bf(&g, theirs));
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
}

/// The Cauldron of Eternity sends your dead to the library bottom.
#[test]
fn cauldron_of_eternity_recycles_the_dead() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_cauldron_of_eternity());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let b = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, b, &[perm(bear)]).expect("bolt");
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bear));
}

/// Braid of Fire: cumulative upkeep adds {R} per age counter.
#[test]
fn cr_702_24_braid_of_fire_pays_in_mana() {
    let mut g = pod(2);
    let braid = ready(&mut g, 0, catalog::braid_of_fire());
    g.step = TurnStep::Upkeep;
    g.process_cumulative_upkeep();
    g.process_cumulative_upkeep();
    assert!(on_bf(&g, braid));
    assert_eq!(g.players[0].mana_pool.total(), 3, "1 then 2");
}

/// Into the Fire's first mode hits every creature.
#[test]
fn into_the_fire_sweeps() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    let f = g.add_card_to_hand(0, catalog::into_the_fire());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: f, target: None, additional_targets: vec![], mode: Some(0), x_value: None })
        .expect("fire");
    drain_stack(&mut g);
    assert!(!on_bf(&g, a) && !on_bf(&g, b));
}

/// Embiggen counts the type line: a Grizzly Bears (Creature — Bear) gets +2.
#[test]
fn embiggen_counts_types() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let e = g.add_card_to_hand(0, catalog::embiggen());
    cast(&mut g, e, &[perm(bear)]).expect("embiggen");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
}

/// CR 608.2d — Hand of Vecna: "equipped creature or a creature you control
/// named Vecna" is one creature, chosen on resolution — never both.
#[test]
fn hand_of_vecna_pumps_one_of_the_host_and_vecna() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let vecna = ready(&mut g, 0, crabomination::card::CardDefinition {
        name: "Vecna",
        card_types: vec![crabomination::card::CardType::Creature],
        power: 8,
        toughness: 8,
        ..Default::default()
    });
    let h = ready(&mut g, 0, catalog::hand_of_vecna());
    g.battlefield_find_mut(h).unwrap().attached_to = Some(bear);
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let pumped = [(bear, 2), (vecna, 8)]
        .iter()
        .filter(|(id, base)| g.computed_permanent(*id).unwrap().power == base + 3)
        .count();
    assert_eq!(pumped, 1);
}

/// Hand of Vecna pumps by hand size at your combat; Wild Defiance answers a
/// targeting instant.
#[test]
fn benton_pumps() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let h = ready(&mut g, 0, catalog::hand_of_vecna());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    let life = g.players[0].life;
    activate(&mut g, h, 0, Some(perm(bear))).expect("equip for life");
    assert_eq!(g.players[0].life, life - 3);
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5);

    let mut g = pod(2);
    ready(&mut g, 0, catalog::wild_defiance());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let e = g.add_card_to_hand(0, catalog::embiggen());
    cast(&mut g, e, &[perm(bear)]).expect("embiggen");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 7, "2 + 2 + 3");
}

