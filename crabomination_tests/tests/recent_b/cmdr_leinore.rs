//! Commander: the Coven Counters precon (MIC, Leinore, `decks::cmdr_leinore`).

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_by(g, 0, id, targets, None)
}

fn activate(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

fn kill(g: &mut GameState, id: CardId) {
    let mut evs = Vec::new();
    g.destroy_permanent(id, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::plains());
    }
}

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

fn power(g: &GameState, id: CardId) -> i32 {
    g.computed_permanent(id).map_or(0, |cp| cp.power)
}

/// Leinore (coven): the counter lands either way; the draw needs three
/// different powers as the trigger resolves (CR 603.4 is only the Wardens'
/// shape — Leinore's "then if" is a resolution check).
#[test]
fn leinore_counters_and_draws_only_with_coven() {
    let mut g = main_phase(2);
    library(&mut g, 0, 4);
    let leinore = g.add_card_to_battlefield(0, catalog::leinore_autumn_sovereign());
    let hand = g.players[0].hand.len();
    fire(&mut g, TurnStep::BeginCombat);
    assert_eq!(counters(&g, leinore), 1, "the only creature takes the counter");
    assert_eq!(g.players[0].hand.len(), hand, "one power on the board: no draw");

    g.add_card_to_battlefield(0, catalog::hill_giant());
    g.add_card_to_battlefield(0, catalog::craw_wurm());
    fire(&mut g, TurnStep::BeginCombat);
    assert_eq!(g.players[0].hand.len(), hand + 1, "powers 1 / 3 / 6, one of them +1: coven draws");
}

/// Celestial Judgment: one creature survives per power; the caster keeps its
/// own of a shared power and an opponent keeps only its weakest of another.
#[test]
fn celestial_judgment_keeps_one_per_power() {
    let mut g = main_phase(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs_two = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant_a = g.add_card_to_battlefield(1, catalog::hill_giant());
    let giant_b = g.add_card_to_battlefield(1, catalog::hill_giant());
    let judgment = g.add_card_to_hand(0, catalog::celestial_judgment());
    cast_at(&mut g, judgment, &[]).expect("cast");
    assert!(g.battlefield_find(mine).is_some(), "the caster keeps its own 2-power creature");
    assert!(g.battlefield_find(theirs_two).is_none(), "the other 2-power creature is destroyed");
    let giants = [giant_a, giant_b].iter().filter(|id| g.battlefield_find(**id).is_some()).count();
    assert_eq!(giants, 1, "exactly one 3-power creature survives");
}

/// Curse of Conformity: the enchanted player's nonlegendary creatures are 3/3
/// with no creature types (layers 4 and 7b); the caster's are untouched.
#[test]
fn curse_of_conformity_resets_the_cursed_players_creatures() {
    let mut g = main_phase(3);
    let vanguard = g.add_card_to_battlefield(1, catalog::elite_vanguard());
    let bystander = g.add_card_to_battlefield(2, catalog::elite_vanguard());
    let legend = g.add_card_to_battlefield(1, catalog::kyler_sigardian_emissary());
    let curse = g.add_card_to_hand(0, catalog::curse_of_conformity());
    cast_at(&mut g, curse, &[Target::Player(1)]).expect("cast");
    let cp = g.computed_permanent(vanguard).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(!cp.subtypes().creature_types.contains(&CreatureType::Human), "lost its creature types");
    assert_eq!(power(&g, bystander), 2, "another player's creatures are not cursed");
    assert_eq!(power(&g, legend), 2, "legendary creatures are exempt");
}

/// Curse of Clinging Webs: a nontoken creature of the enchanted player dying
/// is exiled and the curse's controller gets a Spider; a token gives nothing.
#[test]
fn curse_of_clinging_webs_exiles_and_spins() {
    let mut g = main_phase(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let curse = g.add_card_to_hand(0, catalog::curse_of_clinging_webs());
    cast_at(&mut g, curse, &[Target::Player(1)]).expect("cast");
    kill(&mut g, bear);
    assert!(g.exile.iter().any(|c| c.id == bear), "the dead creature is exiled");
    let spiders = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Spider").count();
    assert_eq!(spiders(&g), 1);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spider" && c.controller == 0));
    kill(&mut g, other);
    assert_eq!(spiders(&g), 1, "an uncursed player's creature is not watched");
}

/// Kurbis enters with one counter per mana spent (ruling: {X}{G}{G} at X=3 is
/// five), and a counter buys a damage shield for a countered creature.
#[test]
fn kurbis_counts_mana_spent_and_shields() {
    let mut g = main_phase(2);
    let kurbis = g.add_card_to_hand(0, catalog::kurbis_harvest_celebrant());
    cast_by(&mut g, 0, kurbis, &[], Some(3)).expect("cast");
    assert_eq!(counters(&g, kurbis), 5);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
    activate(&mut g, kurbis, &[Target::Permanent(bear)]).expect("shield");
    assert_eq!(counters(&g, kurbis), 4);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_by(&mut g, 1, bolt, &[Target::Permanent(bear)], None).expect("bolt");
    assert!(g.battlefield_find(bear).is_some(), "the damage was prevented");
}

/// Kyler: another Human entering adds a counter; other Humans get +1/+1 per
/// counter of any kind (ruling) on Kyler.
#[test]
fn kyler_grows_and_pumps_humans() {
    let mut g = main_phase(2);
    let kyler = g.add_card_to_battlefield(0, catalog::kyler_sigardian_emissary());
    let vanguard = g.add_card_to_hand(0, catalog::elite_vanguard());
    cast_at(&mut g, vanguard, &[]).expect("cast");
    assert_eq!(counters(&g, kyler), 1);
    g.battlefield_find_mut(kyler).unwrap().counters.insert(CounterType::Charge, 1);
    assert_eq!(power(&g, vanguard), 4, "2 + one +1/+1 counter + one charge counter");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(power(&g, bear), 2, "not a Human");
}

/// Moorland Rescuer dies: other creature cards with total power up to its
/// last-known power return, and it is exiled.
#[test]
fn moorland_rescuer_returns_up_to_its_power() {
    let mut g = main_phase(2);
    let rescuer = g.add_card_to_battlefield(0, catalog::moorland_rescuer());
    for def in [catalog::hill_giant(), catalog::grizzly_bears(), catalog::llanowar_elves()] {
        g.add_card_to_graveyard(0, def);
    }
    kill(&mut g, rescuer);
    let back: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0).map(|c| c.definition.name).collect();
    assert_eq!(back.len(), 2, "3 + 1 fits under 4: {back:?}");
    assert!(back.contains(&"Hill Giant") && back.contains(&"Llanowar Elves"));
    assert!(g.exile.iter().any(|c| c.id == rescuer), "the Rescuer is exiled");
}

/// Ruinous Intrusion reads the exiled permanent's mana value.
#[test]
fn ruinous_intrusion_counts_the_exiled_mana_value() {
    let mut g = main_phase(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let _ = angel;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::ruinous_intrusion());
    cast_at(&mut g, spell, &[Target::Permanent(ring), Target::Permanent(bear)]).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == ring));
    assert_eq!(counters(&g, bear), 1, "Sol Ring's mana value is 1");
}

/// Sigardian Zealot: one creature per power gets +X/+X and vigilance.
#[test]
fn sigardian_zealot_pumps_one_per_power() {
    let mut g = main_phase(2);
    let zealot = g.add_card_to_battlefield(0, catalog::sigardian_zealot());
    let elf_a = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let elf_b = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    fire(&mut g, TurnStep::BeginCombat);
    assert_eq!(power(&g, zealot), 6, "the Zealot's own power is one of the three");
    assert_eq!(power(&g, elf_a) + power(&g, elf_b), 1 + 4, "only one of the two 1-power Elves");
    let vigilant = [elf_a, elf_b]
        .iter()
        .filter(|id| g.computed_permanent(**id).unwrap().keywords().contains(&Keyword::Vigilance))
        .count();
    assert_eq!(vigilant, 1);
}

/// Wall of Mourning exiles one card per opponent (CR 102.2 at four seats) and
/// returns one at a coven end step.
#[test]
fn wall_of_mourning_banks_a_card_per_opponent() {
    let mut g = main_phase(4);
    library(&mut g, 0, 5);
    let wall = g.add_card_to_hand(0, catalog::wall_of_mourning());
    cast_at(&mut g, wall, &[]).expect("cast");
    let banked = |g: &GameState| g.exile.iter().filter(|c| c.exiled_with == Some(wall)).count();
    assert_eq!(banked(&g), 3);
    let hand = g.players[0].hand.len();
    fire(&mut g, TurnStep::End);
    assert_eq!(g.players[0].hand.len(), hand, "no coven");
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    fire(&mut g, TurnStep::End);
    assert_eq!((banked(&g), g.players[0].hand.len()), (2, hand + 1), "powers 0 / 1 / 2");
}

/// Heronblade Elite taps for its power in one color, and grows with Humans.
#[test]
fn heronblade_elite_taps_for_its_power() {
    let mut g = main_phase(2);
    let elite = g.add_card_to_battlefield(0, catalog::heronblade_elite());
    g.battlefield_find_mut(elite).unwrap().counters.insert(CounterType::PlusOnePlusOne, 2);
    g.clear_sickness(elite);
    let before = g.players[0].mana_pool.total();
    activate(&mut g, elite, &[]).expect("tap");
    assert_eq!(g.players[0].mana_pool.total(), before + 3);
}

/// Sigarda: its controller has hexproof, so an opponent's burn can't aim at them.
#[test]
fn sigarda_gives_you_hexproof() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::sigarda_herons_grace());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    assert!(cast_by(&mut g, 1, bolt, &[Target::Player(0)], None).is_err());
    let vanguard = g.add_card_to_battlefield(0, catalog::elite_vanguard());
    assert!(g.computed_permanent(vanguard).unwrap().keywords().contains(&Keyword::Hexproof));
}

/// Celebrate the Harvest fetches one basic per distinct power among your
/// creatures, as it resolves.
#[test]
fn celebrate_the_harvest_counts_distinct_powers() {
    let mut g = main_phase(2);
    library(&mut g, 0, 5);
    for def in [catalog::llanowar_elves(), catalog::llanowar_elves(), catalog::hill_giant()] {
        g.add_card_to_battlefield(0, def);
    }
    let spell = g.add_card_to_hand(0, catalog::celebrate_the_harvest());
    cast_at(&mut g, spell, &[]).expect("cast");
    let lands = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    assert_eq!(lands, 2, "powers 1 and 3");
}
