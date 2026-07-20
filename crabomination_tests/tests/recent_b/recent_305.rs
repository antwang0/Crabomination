//! Tests for the recent305 Guildpact gap batch.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, GameAction};
use crabomination::mana::Color;

#[test]
fn battering_wurm_blocks_only_by_equal_or_greater_power() {
    let mut g = two_player_game();
    let bw = g.add_card_to_battlefield(0, catalog::battering_wurm());
    let kw = g.computed_permanent(bw).unwrap().keywords;
    assert!(kw.contains(&Keyword::CantBeBlockedByPowerLess));
}

#[test]
fn caustic_rain_exiles_a_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let cr = g.add_card_to_hand(0, catalog::caustic_rain());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: cr, target: Some(Target::Permanent(land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land exiled");
    assert!(g.exile.iter().any(|c| c.id == land), "and it's in exile, not the graveyard");
}

#[test]
fn daggerclaw_imp_flies_and_cant_block() {
    let mut g = two_player_game();
    let di = g.add_card_to_battlefield(0, catalog::daggerclaw_imp());
    let kw = g.computed_permanent(di).unwrap().keywords;
    assert!(kw.contains(&Keyword::Flying) && kw.contains(&Keyword::CantBlock));
}

#[test]
fn dryad_sophisticate_has_nonbasic_landwalk() {
    let mut g = two_player_game();
    let ds = g.add_card_to_battlefield(0, catalog::dryad_sophisticate());
    assert!(g.computed_permanent(ds).unwrap().keywords.iter().any(|k| matches!(
        k,
        Keyword::LandwalkFiltered(_)
    )));
}

#[test]
fn harrier_griffin_taps_on_upkeep() {
    use crabomination::TurnStep;
    let mut g = two_player_game();
    let hg = g.add_card_to_battlefield(0, catalog::harrier_griffin());
    let _ = hg;
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    // Fire the upkeep step trigger.
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "upkeep trigger tapped a creature");
}

#[test]
fn gristleback_sacs_for_life_equal_to_power() {
    let mut g = two_player_game();
    let gb = g.add_card_to_battlefield(0, catalog::gristleback());
    g.clear_sickness(gb);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: gb, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for life");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gb).is_none(), "sacrificed");
    assert_eq!(g.players[0].life, life + 2, "gained life equal to its 2 power");
}

#[test]
fn frazzle_counters_a_nonblue_spell() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // green, nonblue
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear on stack");
    let fz = g.add_card_to_hand(0, catalog::frazzle());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: fz, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("frazzle the bear");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "nonblue spell countered");
}

#[test]
fn abyssal_nocturnus_grows_when_an_opponent_discards() {
    let mut g = two_player_game();
    let an = g.add_card_to_battlefield(0, catalog::abyssal_nocturnus());
    // Opponent discards a card.
    let mut events = Vec::new();
    let victim = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.discard_card(1, victim, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let cp = g.computed_permanent(an).unwrap();
    assert_eq!(cp.power, 4, "+2/+2 on the opponent's discard");
    assert!(cp.keywords.contains(&Keyword::Fear), "and gains fear");
}
