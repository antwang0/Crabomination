//! Commander: the Scrappy Survivors precon (PIP, Dogmeat, Ever Loyal,
//! `decks::cmdr_dogmeat`), and its primitives: what rode a dying creature
//! (CR 603.10a), its Auras returned and Equipment moved to a new host, Aura
//! copies entering attached (CR 303.4f), a paired loot's mana-value compare,
//! Aura/Equipment-only mana, a host-targeted cost discount, and Treasures
//! for paired mana values.

use crabomination::card::{CardId, CounterType, Keyword, Value};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.turn_number = 3;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..12 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>, more: Vec<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: more,
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn attach(g: &mut GameState, what: CardId, to: CardId) {
    g.battlefield_find_mut(what).unwrap().attached_to = Some(to);
}

fn kill(g: &mut GameState, id: CardId) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![id]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn attack(g: &mut GameState, attackers: &[CardId], defender: usize) {
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

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// CR 603.10a — Gunner Conscript reads what rode it as it died: enchanted
/// and equipped makes two Junk, bare makes none; alive it grows per
/// attachment.
#[test]
fn cr_603_10a_gunner_conscript_counts_its_riders_at_death() {
    let mut g = pod(2);
    let gc = g.add_card_to_battlefield(0, catalog::gunner_conscript());
    let pip = g.add_card_to_battlefield(0, catalog::pip_boy_3000());
    let idol = g.add_card_to_battlefield(0, catalog::idolized());
    attach(&mut g, pip, gc);
    attach(&mut g, idol, gc);
    assert_eq!(pt(&g, gc), (4, 4), "+1/+1 per Aura and Equipment");
    kill(&mut g, gc);
    assert_eq!(named(&g, 0, "Junk").len(), 2, "enchanted and equipped");

    let bare = g.add_card_to_battlefield(0, catalog::gunner_conscript());
    kill(&mut g, bare);
    assert_eq!(named(&g, 0, "Junk").len(), 2, "a bare Conscript makes none");
}

/// CR 603.10a — Cass: the dead creature's Aura returns from the graveyard
/// attached to the new host, and its Equipment moves there.
#[test]
fn cr_603_10a_cass_moves_the_fallen_creatures_gear() {
    let mut g = pod(2);
    let cass = g.add_card_to_battlefield(0, catalog::cass_hand_of_vengeance());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::acquired_mutation());
    let pip = g.add_card_to_battlefield(0, catalog::pip_boy_3000());
    attach(&mut g, aura, bear);
    attach(&mut g, pip, bear);
    kill(&mut g, bear);
    let a = g.battlefield_find(aura).expect("the Aura came back");
    assert_eq!(a.attached_to, Some(cass), "onto the only creature left");
    assert_eq!(g.battlefield_find(pip).unwrap().attached_to, Some(cass));
    assert_eq!(pt(&g, cass), (6, 5), "Acquired Mutation on Cass");
}

/// Strong Back — equipping the enchanted creature is {3} cheaper, and an
/// Aura spell targeting it too; another creature pays full price.
#[test]
fn strong_back_discounts_gear_for_its_host() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let back = g.add_card_to_battlefield(0, catalog::strong_back());
    attach(&mut g, back, bear);
    assert_eq!(pt(&g, bear), (4, 4), "+2/+2 for Strong Back itself");
    let pip = g.add_card_to_battlefield(0, catalog::pip_boy_3000());
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::Equip { equipment: pip, target: other }).is_err(), "equip {{2}}, no mana");
    g.perform_action(GameAction::Equip { equipment: pip, target: bear }).expect("equip {2} - 3 = free");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (6, 6), "+2/+2 per attachment");
    g.players[0].mana_pool.add(Color::Green, 1);
    let friend = g.add_card_to_hand(0, catalog::animal_friend());
    assert!(cast(&mut g, 0, friend, Some(Target::Permanent(other))).is_err(), "{{1}}{{G}} on another creature");
    cast(&mut g, 0, friend, Some(Target::Permanent(bear))).expect("{1}{G} - 3 = {G}");
}

/// Codsworth — its {W}{W} pays for an Aura or Equipment spell, never a
/// creature spell.
#[test]
fn codsworth_mana_is_for_auras_and_equipment() {
    let mut g = pod(2);
    let cw = g.add_card_to_battlefield(0, catalog::codsworth_handy_helper());
    g.clear_sickness(cw);
    activate(&mut g, 0, cw, 0, None, vec![]).expect("{W}{W}");
    let knight = g.add_card_to_hand(0, catalog::savannah_lions());
    assert!(cast(&mut g, 0, knight, None).is_err(), "not for a creature spell");
    let idol = g.add_card_to_hand(0, catalog::idolized());
    cast(&mut g, 0, idol, Some(Target::Permanent(cw))).expect("Idolized off Codsworth's mana");
    assert_eq!(g.battlefield_find(idol).unwrap().attached_to, Some(cw));
}

/// CR 303.4f — Three Dog trades its Aura for a copy on each other attacker.
#[test]
fn cr_303_4f_three_dog_copies_its_aura_onto_the_attackers() {
    let mut g = pod(2);
    let dog = g.add_card_to_battlefield(0, catalog::three_dog_galaxy_news_dj());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::almost_perfect());
    attach(&mut g, aura, dog);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    attack(&mut g, &[dog, a, b], 1);
    assert!(g.battlefield_find(aura).is_none(), "the original was sacrificed");
    let copies = named(&g, 0, "Almost Perfect");
    assert_eq!(copies.len(), 2);
    for (host, copy) in [a, b].iter().zip(copies.iter()) {
        let _ = copy;
        assert_eq!(pt(&g, *host), (9, 10), "a copy on each other attacker");
    }
}

/// Cait — the looter whose discard has the greatest mana value (ties count)
/// earns two counters; losing the compare earns none.
#[test]
fn cait_wins_the_discard_compare() {
    for (mine, theirs, won) in [(5, 0, true), (0, 5, false), (2, 2, true)] {
        let mut g = multi_player_game(2);
        g.active_player_idx = 0;
        g.turn_number = 3;
        g.step = TurnStep::PreCombatMain;
        let card = |mv: i32| match mv {
            5 => catalog::serra_angel(),
            2 => catalog::grizzly_bears(),
            _ => catalog::island(),
        };
        g.add_card_to_library(0, card(mine));
        g.add_card_to_library(1, card(theirs));
        let cait = g.add_card_to_battlefield(0, catalog::cait_cage_brawler());
        attack(&mut g, &[cait], 1);
        let n = g.battlefield_find(cait).unwrap().counter_count(CounterType::PlusOnePlusOne);
        assert_eq!(n == 2, won, "mine {mine} vs theirs {theirs}");
    }
}

/// Vault 21 III — reveal the best five: three 3s and two 2s pay five
/// Treasures; a lone 1 and 5 pay nothing.
#[test]
fn vault_21_pays_for_paired_mana_values() {
    let mut g = pod(2);
    let saga = g.add_card_to_battlefield(0, catalog::vault_21_house_gambit());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::armory_paladin());
    }
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    g.add_card_to_hand(0, catalog::sol_ring());
    g.add_card_to_hand(0, catalog::island());
    let ctx = EffectContext::for_ability(saga, 0, None);
    g.resolve_effect(&Effect::TreasurePerPairedManaValueInHand { max: 5 }, &ctx).expect("III");
    assert_eq!(named(&g, 0, "Treasure").len(), 5);
}

/// Almost Perfect — base 9/10 and indestructible.
#[test]
fn almost_perfect_sets_base_pt() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    let ap = g.add_card_to_hand(0, catalog::almost_perfect());
    cast(&mut g, 0, ap, Some(Target::Permanent(bear))).expect("Almost Perfect");
    assert_eq!(pt(&g, bear), (9, 10));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Indestructible));
}

/// Mister Gutsy — Aura spells grow it; dying, a Junk per counter it had.
#[test]
fn mister_gutsy_junks_its_counters() {
    let mut g = pod(2);
    let mg = g.add_card_to_battlefield(0, catalog::mister_gutsy());
    flood(&mut g, 0);
    for _ in 0..2 {
        let idol = g.add_card_to_hand(0, catalog::idolized());
        cast(&mut g, 0, idol, Some(Target::Permanent(mg))).expect("Idolized");
    }
    assert_eq!(g.battlefield_find(mg).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    kill(&mut g, mg);
    assert_eq!(named(&g, 0, "Junk").len(), 2);
}

/// Dogmeat — entering, mill five and take back an Equipment; an equipped
/// attacker makes a Junk token.
#[test]
fn dogmeat_digs_and_junks() {
    let mut g = pod(2);
    let pip = g.add_card_to_graveyard(0, catalog::pip_boy_3000());
    flood(&mut g, 0);
    let dog = g.add_card_to_hand(0, catalog::dogmeat_ever_loyal());
    cast(&mut g, 0, dog, None).expect("Dogmeat");
    assert!(g.players[0].hand.iter().any(|c| c.id == pip), "Pip-Boy back to hand");
    assert_eq!(g.players[0].graveyard.len(), 5, "five milled");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let jj = g.add_card_to_battlefield(0, catalog::junk_jet());
    attach(&mut g, jj, bear);
    attack(&mut g, &[bear, dog], 1);
    assert_eq!(named(&g, 0, "Junk").len(), 1, "only the equipped attacker");
}

/// Commander Sofia Daguerre — destroys a legendary permanent; its
/// controller gets a Junk token.
#[test]
fn sofia_crash_landing_pays_the_victim_junk() {
    let mut g = pod(2);
    let legend = g.add_card_to_battlefield(1, catalog::ian_the_reckless());
    flood(&mut g, 0);
    let sofia = g.add_card_to_hand(0, catalog::commander_sofia_daguerre());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([]));
    cast(&mut g, 0, sofia, None).expect("Sofia");
    assert!(g.battlefield_find(legend).is_none());
    assert_eq!(named(&g, 1, "Junk").len(), 1);
}

/// Pre-War Formalwear — reanimates a small creature wearing it.
#[test]
fn pre_war_formalwear_returns_a_dressed_creature() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    let pwf = g.add_card_to_hand(0, catalog::pre_war_formalwear());
    cast(&mut g, 0, pwf, None).expect("Formalwear");
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(g.battlefield_find(pwf).unwrap().attached_to, Some(bear));
    assert_eq!(pt(&g, bear), (4, 4));
}

/// Preston Garvey — each combat, a Settlement on a land of yours taps for
/// any color.
#[test]
fn preston_garvey_founds_a_settlement() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::preston_garvey_minuteman());
    let waste = g.add_card_to_battlefield(0, catalog::wastes());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let s = named(&g, 0, "Settlement");
    assert_eq!(s.len(), 1);
    assert_eq!(g.battlefield_find(s[0]).unwrap().attached_to, Some(waste));
    activate(&mut g, 0, waste, 1, None, vec![]).expect("the Settlement's granted any-color ability");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Animal Friend — the attacking host makes a Squirrel with a counter per
/// other attachment on it.
#[test]
fn animal_friend_squirrel_counts_other_gear() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let af = g.add_card_to_battlefield(0, catalog::animal_friend());
    let pip = g.add_card_to_battlefield(0, catalog::pip_boy_3000());
    attach(&mut g, af, bear);
    attach(&mut g, pip, bear);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Mode(1)]));
    attack(&mut g, &[bear], 1);
    let sq = named(&g, 0, "Squirrel");
    assert_eq!(sq.len(), 1);
    assert_eq!(g.battlefield_find(sq[0]).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "Pip-Boy, not Animal Friend");
    let _ = Value::ONE;
}
