//! Commander: the Upgrades Unleashed precon (NEC, Chishiro, `decks::cmdr_chishiro`).
//! "Modified" is CR 700.9: counters, Equipment, or an Aura its controller controls.

use crabomination::card::{CardId, CounterType, Keyword};
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

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
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
    cast_x(g, id, targets, None)
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

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::plains());
    }
}

fn power(g: &GameState, id: CardId) -> i32 {
    g.computed_permanent(id).expect("on the battlefield").power
}

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).expect("on the battlefield").counter_count(CounterType::PlusOnePlusOne)
}

fn attach(g: &mut GameState, what: CardId, host: CardId) {
    g.battlefield_find_mut(what).unwrap().attached_to = Some(host);
}

fn declare(g: &mut GameState, seat: usize, attacks: Vec<Attack>) -> Result<(), String> {
    for a in &attacks {
        g.clear_sickness(a.attacker);
    }
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(attacks)).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn at(attacker: CardId, p: usize) -> Attack {
    Attack { attacker, target: AttackTarget::Player(p) }
}

fn run_combat_out(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

/// Akki Battle Squad: modified attackers untap (the rest stay tapped) and a
/// combat phase is added — once a turn.
#[test]
fn akki_battle_squad_untaps_the_modified_for_another_combat() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::akki_battle_squad());
    let modded = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(modded).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let plain = g.add_card_to_battlefield(0, catalog::hill_giant());
    declare(&mut g, 0, vec![at(modded, 1), at(plain, 1)]).expect("attack");
    assert!(!g.battlefield_find(modded).unwrap().tapped && g.battlefield_find(plain).unwrap().tapped);
    assert_eq!(g.additional_combat_phases, 1);
    run_combat_out(&mut g);
    declare(&mut g, 0, vec![at(modded, 1)]).expect("second combat");
    assert_eq!(g.additional_combat_phases, 1, "only once each turn");
}

/// Ascendant Acolyte: enters with the +1/+1 counters among your other
/// creatures, then doubles them on your upkeep.
#[test]
fn ascendant_acolyte_counts_then_doubles() {
    let mut g = main_phase(2);
    for n in [2, 1] {
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(b).unwrap().add_counters(CounterType::PlusOnePlusOne, n);
    }
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(theirs).unwrap().add_counters(CounterType::PlusOnePlusOne, 5);
    let aa = g.add_card_to_hand(0, catalog::ascendant_acolyte());
    cast_at(&mut g, aa, &[]).expect("cast");
    assert_eq!(counters(&g, aa), 3, "yours only");
    fire(&mut g, TurnStep::Upkeep);
    assert_eq!(counters(&g, aa), 6);
}

/// Collision of Realms: creatures go home; a player who shuffled a nontoken
/// creature reveals to a creature card, a token-only player doesn't.
#[test]
fn collision_of_realms_rerolls_the_creatures() {
    let mut g = main_phase(3);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.add_card_to_battlefield(2, catalog::sol_ring());
    let cr = g.add_card_to_hand(0, catalog::collision_of_realms());
    cast_at(&mut g, cr, &[]).expect("cast");
    assert!(g.battlefield_find(giant).is_some(), "the only creature card in its library comes right back");
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 2).count(), 1, "no creature, no reveal");
    // A token-only player shuffles nothing that counts: no reveal.
    let mut g = main_phase(2);
    let spare = g.add_card_to_library(1, catalog::grizzly_bears());
    let tok = g.add_card_to_hand(0, catalog::raise_the_alarm());
    cast_at(&mut g, tok, &[]).expect("tokens");
    for c in g.battlefield.iter_mut().filter(|c| c.controller == 0 && c.definition.name == "Soldier") {
        c.controller = 1;
        c.owner = 1;
    }
    let cr = g.add_card_to_hand(0, catalog::collision_of_realms());
    cast_at(&mut g, cr, &[]).expect("cast");
    assert_eq!(named(&g, 1, "Soldier"), 0, "tokens are shuffled away");
    assert!(g.battlefield_find(spare).is_none(), "tokens don't earn a reveal");
}

/// Concord with the Kami: counter, card and Spirit, each gated by its board.
#[test]
fn concord_with_the_kami_modes_by_board() {
    let mut g = main_phase(2);
    library(&mut g, 0, 2);
    g.add_card_to_battlefield(0, catalog::concord_with_the_kami());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    fire(&mut g, TurnStep::End);
    assert_eq!(counters(&g, bear), 2);
    assert_eq!((g.players[0].hand.len(), named(&g, 0, "Spirit")), (hand, 0), "nothing enchanted or equipped");
    let aura = g.add_card_to_battlefield(0, catalog::rancor());
    attach(&mut g, aura, bear);
    let blade = g.add_card_to_battlefield(0, catalog::mage_slayer());
    attach(&mut g, blade, bear);
    fire(&mut g, TurnStep::End);
    assert_eq!((g.players[0].hand.len(), named(&g, 0, "Spirit")), (hand + 1, 1));
}

/// Elemental Mastery: the enchanted creature taps for power-many hasty
/// Elementals, exiled at the next end step.
#[test]
fn elemental_mastery_taps_for_elementals() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.clear_sickness(giant);
    let em = g.add_card_to_hand(0, catalog::elemental_mastery());
    cast_at(&mut g, em, &[Target::Permanent(giant)]).expect("enchant");
    activate(&mut g, giant, 0, None).expect("granted ability");
    assert_eq!(named(&g, 0, "Elemental"), 3);
    fire(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Elemental"), 0);
}

/// Goblin Razerunners: sacrifice a land for a counter; the end step may shoot
/// a player for its counters.
#[test]
fn goblin_razerunners_sacrifices_lands_to_shoot() {
    let mut g = main_phase(2);
    let gr = g.add_card_to_battlefield(0, catalog::goblin_razerunners());
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    activate(&mut g, gr, 0, None).expect("first land");
    activate(&mut g, gr, 0, None).expect("second land");
    assert!(activate(&mut g, gr, 0, None).is_err(), "no land left");
    assert_eq!(counters(&g, gr), 2);
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Target(Target::Player(1))]));
    fire(&mut g, TurnStep::End);
    assert_eq!(g.players[1].life, life - 2);
}

/// Kaima goads each opposing creature one of your Auras enchants — not one
/// enchanted by its own controller's Aura — and grows by one per goad.
#[test]
fn kaima_goads_what_your_auras_enchant() {
    let mut g = main_phase(3);
    let k = g.add_card_to_battlefield(0, catalog::kaima_the_fractured_calm());
    let mine = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let yours = g.add_card_to_battlefield(0, catalog::rancor());
    attach(&mut g, yours, mine);
    let theirs_bear = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(2, catalog::rancor());
    attach(&mut g, theirs, theirs_bear);
    fire(&mut g, TurnStep::End);
    assert!(g.goaders(g.battlefield_find(mine).unwrap()).contains(&0));
    assert!(g.goaders(g.battlefield_find(theirs_bear).unwrap()).is_empty(), "their own Aura");
    assert_eq!(counters(&g, k), 1);
}

/// Kosei with an Aura, Equipment and a counter: combat damage to one opponent
/// draws that many and hits each other opponent for as much; bare, nothing.
#[test]
fn kosei_fully_loaded_spreads_the_damage() {
    let mut g = main_phase(3);
    library(&mut g, 0, 6);
    let k = g.add_card_to_battlefield(0, catalog::kosei_penitent_warlord());
    let aura = g.add_card_to_battlefield(0, catalog::rancor());
    attach(&mut g, aura, k);
    declare(&mut g.clone(), 0, vec![at(k, 1)]).expect("probe");
    let blade = g.add_card_to_battlefield(0, catalog::mage_slayer());
    attach(&mut g, blade, k);
    let mut bare = g.clone();
    g.battlefield_find_mut(k).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let (hand, l2) = (g.players[0].hand.len(), g.players[2].life);
    declare(&mut g, 0, vec![at(k, 1)]).expect("attack");
    run_combat_out(&mut g);
    // Rancor +2 and a counter: 3 power; Mage Slayer adds 3 on attack.
    assert_eq!(g.players[0].hand.len(), hand + 3);
    assert_eq!(g.players[2].life, l2 - 3);
    declare(&mut bare, 0, vec![at(k, 1)]).expect("attack");
    run_combat_out(&mut bare);
    assert_eq!((bare.players[0].hand.len(), bare.players[2].life), (hand, l2), "no counter, no ability");
}

/// Mage Slayer hits whatever the equipped creature attacks — the player, or a
/// planeswalker (not its controller).
#[test]
fn mage_slayer_hits_what_it_attacks() {
    let mut g = main_phase(3);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let ms = g.add_card_to_battlefield(0, catalog::mage_slayer());
    attach(&mut g, ms, giant);
    let life = g.players[2].life;
    declare(&mut g.clone(), 0, vec![at(giant, 2)]).expect("attack");
    let mut p = g.clone();
    declare(&mut p, 0, vec![at(giant, 2)]).expect("attack");
    assert_eq!(p.players[2].life, life - 3);
    let pw = g.add_card_to_battlefield(1, catalog::chandra_torch_of_defiance());
    let loyalty = g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty);
    let l1 = g.players[1].life;
    declare(&mut g, 0, vec![Attack { attacker: giant, target: AttackTarget::Planeswalker(pw) }]).expect("attack");
    assert_eq!(g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty), loyalty - 3);
    assert_eq!(g.players[1].life, l1);
}

/// One with the Kami: the enchanted creature or another modified creature of
/// yours dying makes power-many Spirits; an unmodified one doesn't. The host
/// with a counter fires once, not twice. Regression (CR 603.10a): a creature
/// modified only by its own Aura read unmodified as it died, the Aura having
/// gone to the graveyard before the trigger looked.
#[test]
fn one_with_the_kami_turns_modified_deaths_into_spirits() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.battlefield_find_mut(giant).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let owk = g.add_card_to_hand(0, catalog::one_with_the_kami());
    cast_at(&mut g, owk, &[Target::Permanent(giant)]).expect("enchant");
    let modded = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(modded).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let aura_only = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rancor = g.add_card_to_battlefield(0, catalog::rancor());
    attach(&mut g, rancor, aura_only);
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for (victim, spirits) in [(plain, 0), (modded, 3), (aura_only, 7), (giant, 11)] {
        let m = g.add_card_to_hand(0, catalog::murder());
        cast_at(&mut g, m, &[Target::Permanent(victim)]).expect("kill");
        assert_eq!(named(&g, 0, "Spirit"), spirits);
    }
}

/// Orochi Merge-Keeper taps for {G}{G} only while modified.
#[test]
fn orochi_merge_keeper_doubles_when_modified() {
    let mut g = main_phase(2);
    let o = g.add_card_to_battlefield(0, catalog::orochi_merge_keeper());
    g.clear_sickness(o);
    assert!(activate(&mut g.clone(), o, 1, None).is_err(), "unmodified");
    g.battlefield_find_mut(o).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: o,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("modified");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2);
}

/// Rampant Rejuvenator fetches basics up to its power when it dies.
#[test]
fn rampant_rejuvenator_dies_into_lands() {
    let mut g = main_phase(2);
    let rr = g.add_card_to_hand(0, catalog::rampant_rejuvenator());
    cast_at(&mut g, rr, &[]).expect("cast");
    g.battlefield_find_mut(rr).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let lands = named(&g, 0, "Forest");
    let m = g.add_card_to_hand(0, catalog::murder());
    cast_at(&mut g, m, &[Target::Permanent(rr)]).expect("kill");
    assert_eq!(named(&g, 0, "Forest"), lands + 3, "power 3 at death");
}

/// Silkguard: X counters, then hexproof for your Auras, Equipment and
/// modified creatures — not the unmodified or the opponent's.
#[test]
fn silkguard_counters_then_shrouds_the_modified() {
    let mut g = main_phase(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blade = g.add_card_to_battlefield(0, catalog::mage_slayer());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(theirs).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let s = g.add_card_to_hand(0, catalog::silkguard());
    cast_x(&mut g, s, &[Target::Permanent(a)], Some(1)).expect("cast");
    let hexproof = |g: &GameState, id| g.computed_permanent(id).unwrap().keywords().contains(&Keyword::Hexproof);
    assert_eq!(counters(&g, a), 1);
    assert!(hexproof(&g, a) && hexproof(&g, blade));
    assert!(!hexproof(&g, b) && !hexproof(&g, theirs));
}

/// Smoke Spirits' Aid: a Smoke Blessing on each target; when one's creature
/// dies it pings that creature's controller and you get a Treasure.
#[test]
fn smoke_spirits_aid_blesses_and_pays_out() {
    let mut g = main_phase(2);
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::smoke_spirits_aid());
    cast_x(&mut g, s, &[Target::Permanent(theirs), Target::Permanent(mine)], Some(2)).expect("cast");
    let blessings: Vec<_> =
        g.battlefield.iter().filter(|c| c.definition.name == "Smoke Blessing").filter_map(|c| c.attached_to).collect();
    assert_eq!(blessings.len(), 2);
    assert!(blessings.contains(&theirs) && blessings.contains(&mine));
    let life = g.players[1].life;
    let m = g.add_card_to_hand(0, catalog::murder());
    cast_at(&mut g, m, &[Target::Permanent(theirs)]).expect("kill");
    assert_eq!(g.players[1].life, life - 1);
    assert_eq!(named(&g, 0, "Treasure"), 1);
}

/// Spearbreaker Behemoth lends indestructible only to power 5 or greater.
#[test]
fn spearbreaker_behemoth_protects_the_big() {
    let mut g = main_phase(2);
    let sb = g.add_card_to_battlefield(0, catalog::spearbreaker_behemoth());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(0, catalog::akki_battle_squad());
    assert!(activate(&mut g.clone(), sb, 0, Some(Target::Permanent(bear))).is_err(), "power 2");
    activate(&mut g, sb, 0, Some(Target::Permanent(big))).expect("power 6");
    assert!(g.computed_permanent(big).unwrap().keywords().contains(&Keyword::Indestructible));
}

/// Tanuki Transplanter: it or its host attacking adds {G} per power, kept
/// through the phase change.
#[test]
fn tanuki_transplanter_ramps_on_attack() {
    let mut g = main_phase(2);
    let tt = g.add_card_to_battlefield(0, catalog::tanuki_transplanter());
    let mut solo = g.clone();
    declare(&mut solo, 0, vec![at(tt, 1)]).expect("attack");
    assert_eq!(solo.players[0].mana_pool.amount(Color::Green), 2);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    attach(&mut g, tt, giant);
    declare(&mut g, 0, vec![at(giant, 1)]).expect("attack");
    assert_eq!(power(&g, giant), 3);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3);
    run_combat_out(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3, "kept past combat");
}

/// Unquenchable Fury: attacks hit the defending player for their hand size;
/// the Aura comes back to hand when its creature dies.
#[test]
fn unquenchable_fury_burns_by_hand_and_returns() {
    let mut g = main_phase(3);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let uf = g.add_card_to_hand(0, catalog::unquenchable_fury());
    cast_at(&mut g, uf, &[Target::Permanent(bear)]).expect("enchant");
    for _ in 0..4 {
        g.add_card_to_hand(2, catalog::plains());
    }
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    declare(&mut g, 0, vec![at(bear, 2)]).expect("attack");
    assert_eq!((g.players[1].life, g.players[2].life), (l1, l2 - 4));
    let m = g.add_card_to_hand(0, catalog::murder());
    cast_at(&mut g, m, &[Target::Permanent(bear)]).expect("kill");
    assert!(g.players[0].hand.iter().any(|c| c.id == uf));
}

/// Vastwood Surge fetches two tapped basics; kicked, two counters on each of
/// your creatures.
#[test]
fn vastwood_surge_ramps_and_kicked_grows() {
    let mut g = main_phase(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let vs = g.add_card_to_hand(0, catalog::vastwood_surge());
    cast_at(&mut g, vs, &[]).expect("unkicked");
    let forests: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Forest").collect();
    assert!(forests.len() == 2 && forests.iter().all(|c| c.tapped));
    assert_eq!(counters(&g, bear), 0);
    let vs = g.add_card_to_hand(0, catalog::vastwood_surge());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: vs,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked");
    drain_stack(&mut g);
    assert_eq!(counters(&g, bear), 2);
}
