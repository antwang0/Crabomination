//! Radha, Coalition Warlord's deck (`decks::cmdr_01wr2y_a`).

use crabomination::card::CardId;
use crabomination::catalog;
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, mode: Option<usize>) {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on battlefield");
    (c.power, c.toughness)
}

/// Veteran's Voice taps Radha for its cost; Radha's domain trigger (three
/// basic types: +3/+3) and the Voice (+2/+1, never the tapped host) both land
/// on the other creature.
#[test]
fn veterans_voice_taps_radha_for_both_pumps() {
    let mut g = pod(3);
    for land in [catalog::forest(), catalog::mountain(), catalog::island()] {
        g.add_card_to_battlefield(0, land);
    }
    let radha = g.add_card_to_battlefield(0, catalog::radha_coalition_warlord());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let voice = g.add_card_to_hand(0, catalog::veterans_voice());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, voice, Some(Target::Permanent(radha)), None);
    activate(&mut g, voice, Some(Target::Permanent(bears))).expect("tap Radha");
    assert!(g.battlefield_find(radha).is_some_and(|c| c.tapped));
    assert_eq!(pt(&g, bears), (7, 6));
    assert_eq!(pt(&g, radha), (3, 3));
}

/// CR 302.6 — "Tap an untapped creature you control" isn't {T}: a
/// summoning-sick Bramblesnap taps itself for +1/+1.
#[test]
fn cr_302_6_bramblesnap_taps_itself_while_summoning_sick() {
    let mut g = pod(2);
    let snap = g.add_card_to_battlefield(0, catalog::bramblesnap());
    activate(&mut g, snap, None).expect("tap itself");
    assert!(g.battlefield_find(snap).is_some_and(|c| c.tapped));
    assert_eq!(pt(&g, snap), (2, 2));
    assert!(activate(&mut g, snap, None).is_err(), "no untapped creature left");
}

/// Encounter the Unknown's second mode: 2 damage to each creature and each
/// player in a four-seat pod.
#[test]
fn encounter_the_unknown_shoot_to_kill_hits_every_seat() {
    let mut g = pod(4);
    let bears = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::encounter_the_unknown());
    g.players[0].mana_pool.add(Color::Red, 3);
    let before: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    cast(&mut g, spell, None, Some(1));
    assert!(g.players.iter().zip(before).all(|(p, b)| p.life == b - 2));
    assert!(g.battlefield_find(bears).is_none());
}
