//! Functionality tests for `catalog::sets::decks::recent159` (MKM).

use crate::card::Keyword;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

/// Fanatical Strength pumps +3/+3 and grants trample.
#[test]
fn fanatical_strength_pumps_and_tramples() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fanatical_strength());
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Fanatical Strength");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5), "+3/+3");
    assert!(c.keywords.contains(&Keyword::Trample), "gained trample");
}

/// Festerleech's activated pump only fires once each turn.
#[test]
fn festerleech_pump_once_per_turn() {
    let mut g = two_player_game();
    let fl = g.add_card_to_battlefield(0, catalog::festerleech());
    g.clear_sickness(fl);
    fill_mana(&mut g);
    for _ in 0..2 {
        let _ = g.perform_action(GameAction::ActivateAbility {
            card_id: fl, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        });
        drain_stack(&mut g);
    }
    assert_eq!(g.computed_permanent(fl).unwrap().power, 3, "one activation only: 1 + 2");
}

/// Cornered Crook sacrifices an artifact to deal 3 damage.
#[test]
fn cornered_crook_sac_for_damage() {
    let mut g = two_player_game();
    g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    g.move_card_to_battlefield_for_test(0, catalog::cornered_crook());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "sacrificed the Treasure and dealt 3");
}

/// Crime Novelist grows and ramps when you sacrifice an artifact.
#[test]
fn crime_novelist_grows_on_artifact_sacrifice() {
    let mut g = two_player_game();
    let cn = g.add_card_to_battlefield(0, catalog::crime_novelist());
    let treasure = g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    let mut evs = g.remove_to_graveyard_with_triggers(treasure);
    evs.push(GameEvent::PermanentSacrificed { card_id: treasure, who: 0 });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(cn).unwrap().power, 2, "got a +1/+1 counter");
}

/// Absolving Lammasu clears suspicion on entry and suspects on death.
#[test]
fn absolving_lammasu_clears_then_suspects() {
    let mut g = two_player_game();
    // A friendly creature is suspected; ETB clears it.
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mine).unwrap().suspected = true;
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let lam = g.add_card_to_battlefield(0, catalog::absolving_lammasu());
    g.fire_self_etb_triggers(lam, 0);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(mine).unwrap().suspected, "ETB cleared suspicion");
    // Death suspects an opponent's creature.
    let life = g.players[0].life;
    let mut evs = g.remove_to_graveyard_with_triggers(lam);
    evs.push(GameEvent::CreatureDied { card_id: lam });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "gained 3 life");
    assert!(g.battlefield_find(foe).unwrap().suspected, "suspected the opponent's creature");
}
