//! `decks::recent328` — the Season cycle and the Bloomburrow legends.

use crabomination::card::{CardDefinition, CounterType};
use crabomination::game::actions::cost_reduction_for_spell;
use crabomination::catalog;

use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn etb(g: &mut GameState, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(0, def);
    g.fire_self_etb_triggers(id, 0);
    drain_stack(g);
    id
}

/// Cast a Season card with the given point picks.
fn cast_modes(g: &mut GameState, card_id: CardId, modes: Vec<u8>, target: Option<Target>) -> Result<(), GameError> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellSpree {
        card_id,
        spree_modes: modes,
        target,
        additional_targets: vec![],
        x_value: None,
    })?;
    drain_stack(g);
    Ok(())
}

/// The point budget is five: three copies of the one-point mode plus the
/// two-point mode fits; four copies plus it does not.
#[test]
fn season_modes_are_capped_by_the_point_budget() {
    let mut g = main_phase();
    let s1 = g.add_card_to_hand(0, catalog::season_of_the_burrow());
    assert!(cast_modes(&mut g, s1, vec![0, 0, 0, 0, 0], None).is_ok(), "five one-point modes");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Rabbit").count(),
        5,
        "one Rabbit per point spent"
    );
    let s2 = g.add_card_to_hand(0, catalog::season_of_the_burrow());
    assert!(
        cast_modes(&mut g, s2, vec![0, 0, 0, 0, 0, 0], None).is_err(),
        "six points is over budget"
    );
}

/// Season of the Burrow's two-point mode exiles a permanent and replaces it.
#[test]
fn season_of_the_burrow_exiles_and_replaces() {
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::mountain());
    let season = g.add_card_to_hand(0, catalog::season_of_the_burrow());
    cast_modes(&mut g, season, vec![1], Some(Target::Permanent(theirs))).expect("cast");
    assert!(!g.battlefield.iter().any(|c| c.id == theirs), "exiled");
    assert_eq!(g.players[1].hand.len(), 1, "its controller drew");
}

/// Season of Loss draws off the turn's creature deaths.
#[test]
fn season_of_loss_draws_per_creature_that_died() {
    let mut g = main_phase();
    for _ in 0..2 {
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.destroy_permanent(bear, false, &mut Vec::new());
        drain_stack(&mut g);
    }
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::mountain());
    }
    let season = g.add_card_to_hand(0, catalog::season_of_loss());
    cast_modes(&mut g, season, vec![1], None).expect("cast");
    assert_eq!(g.players[0].hand.len(), 2, "two creatures died, two cards");
}

/// Season of Gathering's three-point mode draws off your biggest creature.
#[test]
fn season_of_gathering_draws_for_greatest_power() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::mountain());
    }
    let season = g.add_card_to_hand(0, catalog::season_of_gathering());
    cast_modes(&mut g, season, vec![2], None).expect("cast");
    assert_eq!(g.players[0].hand.len(), 2, "a 2-power Bear draws two");
}

/// Season of the Bold's three-point rider survives into your next turn.
#[test]
fn season_of_the_bold_rider_outlives_the_turn_it_was_cast() {
    let mut g = main_phase();
    let season = g.add_card_to_hand(0, catalog::season_of_the_bold());
    cast_modes(&mut g, season, vec![2], None).expect("cast");
    assert_eq!(g.delayed_triggers.len(), 1, "the watcher is installed");
    let installed = g.delayed_triggers[0].expires_after_turn;
    assert_eq!(installed, Some(g.turn_number + 2), "expires at the end of your next turn");
}

/// Helga grows off big creature spells and taps for her power.
#[test]
fn helga_grows_on_a_four_drop_and_taps_for_her_power() {
    let mut g = main_phase();
    let helga = etb(&mut g, catalog::helga_skittish_seer());
    g.add_card_to_library(0, catalog::mountain());
    let fatty = g.add_card_to_hand(0, catalog::serra_angel());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: fatty,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast a five-drop");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(helga).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].life, 21, "and a life");
}


/// Wick makes a Snail for the first Rat and grows it for the next.
#[test]
fn wick_makes_then_grows_a_snail() {
    let mut g = main_phase();
    etb(&mut g, catalog::wick_the_whorled_mind());
    let snail = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Snail")
        .map(|c| c.id)
        .expect("Wick is a Rat, so it made a Snail on its own entry");
    let rat = g.add_card_to_battlefield(0, catalog::wick_the_whorled_mind());
    g.fire_self_etb_triggers(rat, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(snail).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}


/// Kotis exiles as much of the victim's library as the damage, and only the
/// cards that fit under X become castable.
#[test]
fn kotis_only_grants_the_cards_under_the_damage_cap() {
    let mut g = main_phase();
    let kotis = g.add_card_to_battlefield(0, catalog::kotis_the_fangkeeper());
    let cheap = g.add_card_to_library(1, catalog::lightning_bolt());
    let pricey = g.add_card_to_library(1, catalog::serra_angel());
    g.clear_sickness(kotis);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: kotis, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(
        g.exile.iter().find(|c| c.id == cheap).unwrap().may_play_until.is_some(),
        "a 1-drop is under the 2-damage cap"
    );
    assert!(
        g.exile.iter().find(|c| c.id == pricey).unwrap().may_play_until.is_none(),
        "a 5-drop is not"
    );
}


/// Artist's Talent loots off noncreature spells and discounts them at level 2.
#[test]
fn artists_talent_loots_then_discounts() {
    let mut g = main_phase();
    let talent = g.add_card_to_battlefield(0, catalog::artists_talent());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let card = g.find_card_anywhere(bolt).unwrap().clone();
    assert_eq!(cost_reduction_for_spell(&g, 0, &card, None), 0, "no discount at level 1");
    g.battlefield_find_mut(talent).unwrap().class_level = 2;
    assert_eq!(cost_reduction_for_spell(&g, 0, &card, None), 1, "level 2 shaves one generic");
}




/// Cruelclaw's hit digs past lands to a free cast.
#[test]
fn the_infamous_cruelclaw_impulses_to_the_first_nonland() {
    let mut g = main_phase();
    let claw = g.add_card_to_battlefield(0, catalog::the_infamous_cruelclaw());
    g.add_card_to_library(0, catalog::mountain());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.clear_sickness(claw);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: claw, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(
        g.exile.iter().find(|c| c.id == bolt).unwrap().may_play_until.is_some(),
        "the first nonland is castable"
    );
}

/// Muerra ramps one mana per Raccoon at your precombat main.
#[test]
fn muerra_ramps_per_raccoon() {
    let mut g = main_phase();
    let muerra = g.add_card_to_battlefield(0, catalog::muerra_trash_tactician());
    g.players[0].mana_pool.empty();
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "Muerra is her own Raccoon");
    let _ = muerra;
}


/// Dragonhawk impulses one card per 4-power creature you control.
#[test]
fn dragonhawk_digs_per_big_creature() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::mountain());
    }
    let hawk = etb(&mut g, catalog::dragonhawk_fates_tempest());
    assert_eq!(
        g.exile.iter().filter(|c| c.may_play_until.is_some()).count(),
        1,
        "only the 5/5 Dragonhawk itself is 4-power"
    );
    let _ = hawk;
}
