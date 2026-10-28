//! Star Trek Commander: the Federation Fleet precon's new cards
//! (`decks::trek_federation`).

use crabomination::card::{CardId, CardType, CounterType, CreatureType, Keyword, Supertype};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, GameError, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn draw(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        let mut ev = Vec::new();
        g.draw_one(seat, &mut ev);
        g.dispatch_triggers_for_events(&ev);
        drain_stack(g);
    }
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn counters(g: &GameState, id: CardId, kind: CounterType) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(kind))
}

fn pass_to(g: &mut GameState, step: TurnStep) {
    for _ in 0..200 {
        if g.step == step {
            return;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    panic!("never reached {step:?}");
}

/// Seek out (CR 207.2c ability word; CR 121.2 counts draws per turn) fires on
/// the second draw only: not the first, not the third, and not on a third
/// draw for a seek-out creature that arrived after the second.
#[test]
fn seek_out_fires_on_the_second_draw_only() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::dr_pulaski_cynical_clinician());
    let life = g.players[0].life;
    draw(&mut g, 0, 1);
    assert_eq!(g.players[0].life, life);
    draw(&mut g, 0, 1);
    assert_eq!(g.players[0].life, life + 4);
    draw(&mut g, 0, 1);
    assert_eq!(g.players[0].life, life + 4, "the third draw is not a second draw");
    g.add_card_to_battlefield(0, catalog::dr_pulaski_cynical_clinician());
    draw(&mut g, 0, 1);
    assert_eq!(g.players[0].life, life + 4, "a late arrival sees no second draw");
    draw(&mut g, 1, 2);
    assert_eq!(g.players[0].life, life + 4, "an opponent's second draw is not yours");
}

/// Picard draws for another historic permanent with mana value 3+ (CR 700.6)
/// and not for a cheap one or a plain creature.
#[test]
fn picard_draws_for_big_historic_permanents() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::jean_luc_picard_luminary());
    let mut enter = |g: &mut GameState, def| {
        let id = g.add_card_to_hand(0, def);
        flood(g, 0);
        cast(g, id, None);
        g.players[0].hand.len()
    };
    let hand = g.players[0].hand.len();
    assert_eq!(enter(&mut g, catalog::hedron_archive()), hand + 1, "a 4-mana artifact draws");
    assert_eq!(enter(&mut g, catalog::sol_ring()), hand + 1, "a 1-mana artifact");
    assert_eq!(enter(&mut g, catalog::hill_giant()), hand + 1, "a nonhistoric creature");
    assert_eq!(enter(&mut g, catalog::worf_son_of_mogh()), hand + 2, "a 3-mana legend draws");
}

/// Picard's seek out: +1/+1 on another creature and two charge counters on a
/// Spacecraft.
#[test]
fn picard_seek_out_charges_a_spacecraft() {
    let mut g = pod(2);
    let picard = g.add_card_to_battlefield(0, catalog::jean_luc_picard_luminary());
    let ship = g.add_card_to_battlefield(0, catalog::u_s_s_enterprise_c_time_tossed());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    draw(&mut g, 0, 2);
    assert_eq!(counters(&g, ship, CounterType::Charge), 2);
    assert_eq!(counters(&g, bear, CounterType::PlusOnePlusOne), 1);
    assert_eq!(counters(&g, picard, CounterType::PlusOnePlusOne), 0, "\"other\" creature");
    assert!(!g.computed_permanent(ship).unwrap().card_types().contains(&CardType::Creature));
    g.battlefield_find_mut(ship).unwrap().add_counters(CounterType::Charge, 1);
    let c = g.computed_permanent(ship).unwrap();
    assert!(c.card_types().contains(&CardType::Creature), "3+ charge counters: a 6/6 flier");
    assert_eq!((c.power, c.toughness), (6, 6));
}

/// Janeway: two Officers connecting together are one batch (one draw, one
/// token).
#[test]
fn janeway_triggers_once_per_combat_damage_batch() {
    let mut g = pod(2);
    let jane = ready(&mut g, 0, catalog::captain_janeway_endgame());
    let riker = ready(&mut g, 0, catalog::commander_william_riker());
    let hand = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: jane, target: AttackTarget::Player(1) },
        Attack { attacker: riker, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    let officers = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Officer")
        .count();
    assert_eq!(officers, 1);
}

/// Cmdr. Troi on an opponent's turn: they cast Divination free and Troi's
/// controller gets a copy; with nothing to cast, Troi draws instead.
#[test]
fn troi_copies_the_free_spell_or_draws() {
    let mut g = pod(2);
    let troi = ready(&mut g, 0, catalog::cmdr_troi_outgoing_empath());
    g.add_card_to_hand(1, catalog::divination());
    g.active_player_idx = 1;
    let (mine, theirs) = (g.players[0].hand.len(), g.players[1].hand.len());
    g.priority.player_with_priority = 0;
    activate(&mut g, troi, 0, Some(Target::Player(1))).expect("activate");
    assert_eq!(g.players[1].hand.len(), theirs - 1 + 2, "they cast Divination for free");
    assert_eq!(g.players[0].hand.len(), mine + 2, "the copy is Troi's controller's");
    assert_eq!(g.players[1].mana_pool.total(), 0);

    g.battlefield_find_mut(troi).unwrap().tapped = false;
    let mine = g.players[0].hand.len();
    g.players[1].hand.clear();
    g.priority.player_with_priority = 0;
    activate(&mut g, troi, 0, Some(Target::Player(1))).expect("activate");
    assert_eq!(g.players[0].hand.len(), mine + 1, "no spell cast: Troi draws");
}

/// Troi needs an opponent whose turn it is: on your own turn there is no
/// legal target.
#[test]
fn troi_has_no_target_on_your_own_turn() {
    let mut g = pod(2);
    let troi = ready(&mut g, 0, catalog::cmdr_troi_outgoing_empath());
    assert!(activate(&mut g, troi, 0, Some(Target::Player(1))).is_err());
}

/// Kes draws one more card each end step and is sacrificed at nine time
/// counters (after drawing the nine).
#[test]
fn kes_draws_more_each_end_step_and_leaves_at_nine() {
    let mut g = pod(2);
    for _ in 0..20 {
        g.add_card_to_library(0, catalog::island());
    }
    let kes = g.add_card_to_battlefield(0, catalog::kes_evolving_telepath());
    let hand = g.players[0].hand.len();
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(counters(&g, kes, CounterType::Time), 1);
    assert_eq!(g.players[0].hand.len(), hand + 1);

    let mut g = pod(2);
    for _ in 0..20 {
        g.add_card_to_library(0, catalog::island());
    }
    let kes = g.add_card_to_battlefield(0, catalog::kes_evolving_telepath());
    g.battlefield_find_mut(kes).unwrap().add_counters(CounterType::Time, 8);
    let hand = g.players[0].hand.len();
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 9);
    assert!(g.battlefield_find(kes).is_none(), "nine time counters: sacrificed");
}

/// Enterprise-E's arrival destroys tapped creatures only — both sides'.
#[test]
fn enterprise_e_destroys_tapped_creatures() {
    let mut g = pod(3);
    let tapped_mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tapped_theirs = g.add_card_to_battlefield(2, catalog::hill_giant());
    let untapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(tapped_mine).unwrap().tapped = true;
    g.battlefield_find_mut(tapped_theirs).unwrap().tapped = true;
    let ship = g.add_card_to_hand(0, catalog::u_s_s_enterprise_e_sovereign());
    flood(&mut g, 0);
    cast(&mut g, ship, None);
    assert!(g.battlefield_find(tapped_mine).is_none());
    assert!(g.battlefield_find(tapped_theirs).is_none());
    assert!(g.battlefield_find(untapped).is_some());
    assert!(g.battlefield_find(ship).is_some());
}

/// Harry Kim grows only for a creature with greater power than his, and
/// spends two +1/+1 counters from among your creatures to draw.
#[test]
fn harry_kim_grows_then_spends_counters_from_among_creatures() {
    let mut g = pod(2);
    let kim = ready(&mut g, 0, catalog::ensign_harry_kim_operations());
    let lions = g.add_card_to_hand(0, catalog::savannah_lions());
    flood(&mut g, 0);
    cast(&mut g, lions, None);
    assert_eq!(counters(&g, kim, CounterType::PlusOnePlusOne), 1, "power 2 beats his 1");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None);
    assert_eq!(counters(&g, kim, CounterType::PlusOnePlusOne), 1, "power 2 ties his 2");
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    activate(&mut g, kim, 0, None).expect("activate");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(counters(&g, kim, CounterType::PlusOnePlusOne) + counters(&g, bear, CounterType::PlusOnePlusOne), 0);
    assert!(activate(&mut g, kim, 0, None).is_err(), "no counters left to remove");
}

/// Guinan counts only creatures that entered this turn, and draws when it
/// placed two or more counters.
#[test]
fn guinan_rewards_creatures_that_entered_this_turn() {
    let mut g = pod(2);
    let guinan = ready(&mut g, 0, catalog::guinan_ancient_advisor());
    let old = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(old).unwrap().entered_turn = None;
    let a = g.add_card_to_battlefield(0, catalog::savannah_lions());
    for id in [guinan, a] {
        g.battlefield_find_mut(id).unwrap().entered_turn = None;
    }
    g.battlefield_find_mut(a).unwrap().entered_turn = Some(g.turn_number);
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    activate(&mut g, guinan, 0, None).expect("activate");
    assert_eq!(counters(&g, a, CounterType::PlusOnePlusOne), 1);
    assert_eq!(counters(&g, old, CounterType::PlusOnePlusOne), 0);
    assert_eq!(g.players[0].hand.len(), hand, "one counter: no draw");
    let b = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.battlefield_find_mut(b).unwrap().entered_turn = Some(g.turn_number);
    g.battlefield_find_mut(guinan).unwrap().tapped = false;
    activate(&mut g, guinan, 0, None).expect("activate");
    assert_eq!(counters(&g, b, CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].hand.len(), hand + 1, "two counters: draw");
}

/// Reg Barclay's copy is a hasty, nonlegendary Illusion artifact (CR 707.9b)
/// and is exiled at the next end step.
#[test]
fn reg_barclay_copies_a_legend_as_a_temporary_illusion() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::reg_barclay_fidgety_fantasist());
    let worf = g.add_card_to_battlefield(0, catalog::worf_son_of_mogh());
    g.add_card_to_hand(0, catalog::divination());
    let div = g.players[0].hand.last().unwrap().id;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: div,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    // Divination resolves, the seek-out trigger targets Worf.
    for _ in 0..8 {
        if g.stack.is_empty() {
            break;
        }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    drain_stack(&mut g);
    let copy = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.definition.name == "Worf, Son of Mogh")
        .map(|c| c.id)
        .expect("a token Worf");
    assert!(g.battlefield_find(worf).is_some(), "the legend rule spares the original");
    let c = g.computed_permanent(copy).unwrap();
    assert!(!c.supertypes().contains(&Supertype::Legendary));
    assert!(c.card_types().contains(&CardType::Artifact));
    assert!(c.subtypes().creature_types.contains(&CreatureType::Illusion));
    assert!(c.keywords().contains(&Keyword::Haste));
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(copy).is_none(), "exiled at the next end step");
}

/// Temporal Tribulation blinks two of your permanents back at the next end
/// step, and casts again from the graveyard by flashback.
#[test]
fn temporal_tribulation_blinks_two_permanents_until_end_step() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::sol_ring());
    let spell = g.add_card_to_hand(0, catalog::temporal_tribulation());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    let back = g.battlefield.iter().filter(|c| c.owner == 0 && !c.is_token).count();
    assert_eq!(back, 2, "both return at the end step");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == spell));
}

/// Ezri Dax enters as a copy of a creature that died this turn, keeping her
/// name and becoming a legendary Trill.
#[test]
fn ezri_dax_copies_a_creature_that_died_this_turn() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.perform_action(GameAction::PassPriority).ok();
    let mut ev = Vec::new();
    g.destroy_permanent(giant, false, &mut ev);
    drain_stack(&mut g);
    let ezri = g.add_card_to_hand(0, catalog::ezri_dax_newly_joined());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    flood(&mut g, 0);
    cast(&mut g, ezri, None);
    let c = g.computed_permanent(ezri).expect("Ezri on the battlefield");
    assert_eq!(c.def.name, "Ezri Dax, Newly Joined");
    assert_eq!((c.power, c.toughness), (3, 3), "Hill Giant's body");
    assert!(c.supertypes().contains(&Supertype::Legendary));
    assert!(c.subtypes().creature_types.contains(&CreatureType::Trill));
}
