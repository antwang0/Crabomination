//! Commander: Azami's, Kolodin's and Sokka and Suki's last gaps
//! (`decks::cmdr_edhrec45`).

use crabomination::card::{CardId, Keyword};
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

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
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

/// Cyclone Summoner cast from hand bounces everything but Giants, Wizards and
/// lands — its own Wizards stay.
#[test]
fn cyclone_summoner_spares_giants_wizards_and_lands() {
    let mut g = pod(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let tracer = g.add_card_to_battlefield(0, catalog::sigil_tracer());
    let land = g.add_card_to_battlefield(2, catalog::forest());
    let summoner = g.add_card_to_hand(0, catalog::cyclone_summoner());
    cast(&mut g, summoner, None).expect("summoner");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "to its owner's hand");
    assert!(g.battlefield_find(tracer).is_some());
    assert!(g.battlefield_find(land).is_some());
    assert!(g.battlefield_find(summoner).is_some());
}

/// Sigil Tracer taps two Wizards to copy an instant (CR 707.10).
#[test]
fn cr_707_10_sigil_tracer_copies_with_two_wizards() {
    let mut g = pod(2);
    let tracer = g.add_card_to_battlefield(0, catalog::sigil_tracer());
    let mage = g.add_card_to_battlefield(0, catalog::vedalken_aethermage());
    let shock = g.add_card_to_hand(0, catalog::shock());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: shock,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("shock");
    let life = g.players[1].life;
    activate(&mut g, tracer, 0, Some(Target::Permanent(shock))).expect("copy");
    assert!(g.battlefield_find(tracer).unwrap().tapped && g.battlefield_find(mage).unwrap().tapped);
    assert_eq!(g.players[1].life, life - 4, "the Shock and its copy");
}

/// Siege Modification makes the Vehicle it enchants a creature with +3/+0
/// and first strike; Depala adds +1/+1 to it.
#[test]
fn siege_modification_and_depala_field_a_vehicle() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::depala_pilot_exemplar());
    let boat = g.add_card_to_battlefield(0, catalog::consulate_dreadnought());
    assert!(!g.computed_permanent(boat).unwrap().card_types().contains(&crabomination::card::CardType::Creature));
    let siege = g.add_card_to_hand(0, catalog::siege_modification());
    cast(&mut g, siege, Some(Target::Permanent(boat))).expect("siege");
    let cp = g.computed_permanent(boat).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (7 + 3 + 1, 11 + 1));
    assert!(cp.keywords().contains(&Keyword::FirstStrike));
}

/// Sokka and Suki: an Equipment entering makes an Ally, and that Ally entering
/// picks up the Equipment; Trusty Boomerang taps a creature and comes home.
#[test]
fn sokka_and_suki_arm_the_allies() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::sokka_and_suki());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let rang = g.add_card_to_hand(0, catalog::trusty_boomerang());
    cast(&mut g, rang, None).expect("boomerang");
    let ally = g.battlefield.iter().find(|c| c.definition.name == "Ally").map(|c| c.id).expect("an Ally token");
    assert_eq!(g.battlefield_find(rang).unwrap().attached_to, Some(ally));
    g.clear_sickness(ally);
    activate(&mut g, ally, 0, Some(Target::Permanent(bear))).expect("boomerang throw");
    assert!(g.battlefield_find(bear).unwrap().tapped);
    assert!(g.players[0].hand.iter().any(|c| c.id == rang), "back to hand");
}
