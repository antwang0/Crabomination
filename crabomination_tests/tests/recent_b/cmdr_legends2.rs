//! Most-built commanders, second batch (`decks::cmdr_legends2`,
//! COMMANDER_BACKLOG §1 regenerated 2026-09-27).

use crabomination::card::{CardId, Keyword, KeywordSlice};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    advance_to(g, TurnStep::DeclareAttackers);
    let attacks = attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect();
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("attack");
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// CR 202.3f — Reaper King's {2/W}…{2/G} is mana value 10; another Scarecrow
/// entering under its controller destroys target permanent, and gets +1/+1.
#[test]
fn reaper_king_destroys_on_another_scarecrow() {
    let mut g = pod(3);
    assert_eq!(catalog::reaper_king().cost.cmc(), 10);
    g.add_card_to_battlefield(0, catalog::reaper_king());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let guide = g.add_card_to_hand(0, catalog::scarecrow_guide());
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, 0, guide, None).expect("cast the Scarecrow");
    assert!(g.battlefield_find(giant).is_none(), "the Giant was destroyed");
    let base = catalog::scarecrow_guide();
    assert_eq!(pt(&g, guide), (base.power + 1, base.toughness + 1));
}

/// CR 510.2 — Shroofus (a Saproling) connecting for 1 makes one Saproling.
#[test]
fn shroofus_saprolings_make_saprolings_on_combat_damage() {
    let mut g = pod(3);
    let shroofus = g.add_card_to_battlefield(0, catalog::shroofus_sproutsire());
    let life = g.players[1].life;
    attack(&mut g, &[shroofus]);
    advance_to(&mut g, TurnStep::PostCombatMain);
    let saprolings = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Saproling").count();
    assert_eq!(saprolings, 1);
    assert_eq!(g.players[1].life, life - 1);
}

/// Kratos gives every creature haste (CR 702.10b), and at a player's end step
/// deals that player damage per creature of theirs that didn't attack.
#[test]
fn kratos_punishes_creatures_that_stayed_home() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::kratos_god_of_war());
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant());
    assert!(g.computed_permanent(theirs).unwrap().keywords().has_kw(&Keyword::Haste), "anyone's creature");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack(&mut g, &[bear]);
    let life = g.players[0].life;
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "Kratos and the second Bear stayed home");
}

/// Myrel: attacking makes a Soldier per Soldier you control (itself
/// included), and during your turn an opponent can't cast a spell.
#[test]
fn myrel_makes_soldiers_and_silences_opponents() {
    let mut g = pod(3);
    let myrel = g.add_card_to_battlefield(0, catalog::myrel_shield_of_argive());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    assert!(cast(&mut g, 1, bolt, Some(Target::Player(0))).is_err(), "not during Myrel's controller's turn");
    attack(&mut g, &[myrel]);
    let soldiers = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 1);
}

/// Doran: a creature spell with toughness above power costs {1} less, and an
/// attacker gets +X/+X for the gap between its power and toughness.
#[test]
fn doran_discounts_walls_and_pumps_by_the_gap() {
    let mut g = pod(3);
    let doran = g.add_card_to_battlefield(0, catalog::doran_besieged_by_time());
    let turtle = g.add_card_to_hand(0, catalog::horned_turtle());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, 0, turtle, None).expect("{2}{U} for {1}{U}");
    attack(&mut g, &[doran]);
    assert_eq!(pt(&g, doran), (5, 10), "0/5 attacking: +5/+5");
}

/// Gargos: a creature of yours becoming the target of a spell makes Gargos
/// fight a creature you don't control.
#[test]
fn gargos_fights_when_your_creature_is_targeted_by_a_spell() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::gargos_vicious_watcher());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, 0, growth, Some(Target::Permanent(bear))).expect("Giant Growth");
    assert!(g.battlefield_find(giant).is_none(), "Gargos (8 power) fought the Giant");
}

/// CR 702.6c — Syr Gwyn's equip Knight {0}: a Knight is equipped for free, a
/// Bear still pays; an equipped attacker draws a card for 1 life.
#[test]
fn syr_gwyn_equips_knights_for_free_and_draws_on_equipped_attacks() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::syr_gwyn_hero_of_ashvale());
    let knight = g.add_card_to_battlefield(0, catalog::white_knight());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blade = g.add_card_to_battlefield(0, catalog::bonesplitter());
    assert!(g.perform_action(GameAction::Equip { equipment: blade, target: bear }).is_err(), "equip {{1}} unpaid");
    g.perform_action(GameAction::Equip { equipment: blade, target: knight }).expect("equip Knight {0}");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(blade).unwrap().attached_to, Some(knight));
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    attack(&mut g, &[knight]);
    assert_eq!((g.players[0].hand.len(), g.players[0].life), (hand + 1, life - 1));
}

/// CR 118.9 — Jodah, Archmage Eternal lets a spell be cast for WUBRG.
#[test]
fn jodah_archmage_casts_anything_for_wubrg() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::jodah_archmage_eternal());
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: wurm, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast for WUBRG");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_some());
}

/// Raggadragga: a mana creature gets +2/+2 and untaps when it attacks; a
/// spell cast with seven or more mana gives a creature +7/+7 and trample.
#[test]
fn raggadragga_pumps_mana_creatures_and_rewards_big_spells() {
    let mut g = pod(3);
    let boss = g.add_card_to_battlefield(0, catalog::raggadragga_goreguts_boss());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    assert_eq!(pt(&g, elf), (3, 3));
    attack(&mut g, &[elf]);
    assert!(!g.battlefield_find(elf).unwrap().tapped, "untapped as it attacked");
    advance_to(&mut g, TurnStep::PostCombatMain);
    let crusher = g.add_card_to_hand(0, catalog::ulamogs_crusher());
    g.players[0].mana_pool.add_colorless(8);
    cast(&mut g, 0, crusher, None).expect("eight mana into the Crusher");
    let pumped = [boss, elf, crusher].iter().any(|&c| {
        let cp = g.computed_permanent(c).unwrap();
        cp.keywords().has_kw(&Keyword::Trample) && cp.power >= 10
    });
    assert!(pumped, "+7/+7 and trample on a creature");
}

/// Alexios moves to each player at their upkeep, untapped, with a +1/+1
/// counter and haste.
#[test]
fn alexios_changes_hands_every_upkeep() {
    let mut g = pod(3);
    let alexios = g.add_card_to_battlefield(0, catalog::alexios_deimos_of_kosmos());
    advance_to(&mut g, TurnStep::End);
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.active_player_idx, 1);
    let a = g.battlefield_find(alexios).unwrap();
    assert_eq!(a.controller, 1);
    assert_eq!(a.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
    assert!(g.computed_permanent(alexios).unwrap().keywords().has_kw(&Keyword::Haste));
}

/// Narset gives your creatures prowess; attacking exiles a cheap noncreature
/// card from a graveyard to cast a copy.
#[test]
fn narset_recasts_a_graveyard_spell_on_attack() {
    let mut g = pod(3);
    let narset = g.add_card_to_battlefield(0, catalog::narset_enlightened_exile());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords().has_kw(&Keyword::Prowess));
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    attack(&mut g, &[narset]);
    assert!(g.exile.iter().any(|c| c.id == bolt), "the Bolt left the graveyard for exile");
}

/// CR 605.1a — a permanent's printed mana ability counts on the battlefield
/// (Raggadragga's pump, Midnight Arsonist's "without mana abilities").
#[test]
fn cr_605_1a_battlefield_permanents_read_their_mana_abilities() {
    use crabomination::card::SelectionRequirement as R;
    let mut g = pod(3);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let thopter = g.add_card_to_battlefield(1, catalog::ornithopter());
    let without = R::Artifact.and(R::HasManaAbility.negate());
    assert!(!g.evaluate_requirement_static(&without, &Target::Permanent(ring), 0, None), "Sol Ring has one");
    assert!(g.evaluate_requirement_static(&without, &Target::Permanent(thopter), 0, None));
}
