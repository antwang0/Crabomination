//! Commander: The Earth King and The Muckslinger Gang's gaps
//! (`decks::cmdr_edhrec53`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn fill_pool(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
    g.priority.player_with_priority = 0;
}

fn cast(g: &mut GameState, id: CardId) -> Result<(), String> {
    fill_pool(g);
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn on_battlefield(g: &GameState, id: CardId) -> bool {
    g.battlefield.iter().any(|c| c.id == id)
}

/// Soul Immolation: CR 701.68a + 601.2b — blight X as a cost, X up to the
/// greatest toughness among your creatures. The counters go on the cheapest
/// creature (the 2/2), X is the 6/6's toughness, and every opponent and
/// their creatures take 6.
#[test]
fn soul_immolation_blights_x_and_burns_the_table() {
    let mut g = pod(3);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let dreadmaw = ready(&mut g, 0, catalog::colossal_dreadmaw());
    let their_giant = ready(&mut g, 1, catalog::hill_giant());
    let life = g.players[1].life;
    let spell = g.add_card_to_hand(0, catalog::soul_immolation());
    cast(&mut g, spell).expect("cast");
    assert!(!on_battlefield(&g, bears), "the 2/2 took six -1/-1 counters");
    assert!(on_battlefield(&g, dreadmaw), "the 6/6 only set X");
    assert!(!on_battlefield(&g, their_giant));
    assert_eq!(g.players[1].life, life - 6);
    assert_eq!(g.players[2].life, life - 6);
    assert_eq!(g.players[0].life, life, "not the caster");
}

/// The Muckslinger Gang: other creatures get +3/+3 only while they're not
/// attacking or blocking (CR 611.3a — the static re-reads each check).
#[test]
fn the_muckslinger_gang_pumps_creatures_at_home() {
    let mut g = pod(2);
    let gang = ready(&mut g, 0, catalog::the_muckslinger_gang());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bears).unwrap().power, 5);
    assert_eq!(g.computed_permanent(gang).unwrap().power, 3, "not itself");
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 2, "attacking: no bonus");
}

/// Old-Growth Troll dies and comes back as an Aura on a Forest you control
/// (CR 303.4f — put onto the battlefield, not cast), granting the Forest its
/// two abilities; the token ability makes a tapped 4/4 Troll.
#[test]
fn old_growth_troll_returns_as_an_aura() {
    let mut g = pod(2);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let troll = ready(&mut g, 0, catalog::old_growth_troll());
    let mut evs = Vec::new();
    g.destroy_permanent(troll, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let aura = g.battlefield.iter().find(|c| c.id == troll).expect("back as an Aura");
    assert_eq!(aura.attached_to, Some(forest));
    assert!(!g.computed_permanent(troll).unwrap().def.is_creature(), "an enchantment now");
    assert_eq!(g.granted_abilities_for(forest).len(), 2);
    fill_pool(&mut g);
    let index = catalog::forest().activated_abilities.len() + 1;
    g.perform_action(GameAction::ActivateAbility { card_id: forest, ability_index: index, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("sacrifice for a Troll");
    drain_stack(&mut g);
    assert!(!on_battlefield(&g, forest));
    let token = g.battlefield.iter().find(|c| c.definition.name == "Troll Warrior").expect("the token");
    assert!(token.tapped);
    assert!(!on_battlefield(&g, troll), "the Aura lost its host");
}

/// Sandman is as big as your land count and climbs back out of the graveyard
/// with a land card.
#[test]
fn sandman_counts_lands_and_returns_with_one() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let sandman = ready(&mut g, 0, catalog::sandman_shifting_scoundrel());
    assert_eq!(g.computed_permanent(sandman).unwrap().power, 3);
    let mut evs = Vec::new();
    g.destroy_permanent(sandman, false, &mut evs);
    drain_stack(&mut g);
    let land = g.add_card_to_graveyard(0, catalog::forest());
    fill_pool(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sandman,
        ability_index: 0,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("graveyard ability");
    drain_stack(&mut g);
    let s = g.battlefield.iter().find(|c| c.id == sandman).expect("back");
    assert!(s.tapped);
    assert!(g.battlefield.iter().any(|c| c.id == land && c.tapped));
    assert_eq!(g.computed_permanent(sandman).unwrap().power, 4);
}

/// Roiling Canopy: a Forest entering with five other Forests out pumps a
/// creature (CR 603.4 intervening if); with four others it doesn't.
#[test]
fn roiling_canopy_pumps_on_the_sixth_forest() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::roiling_canopy());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let fifth = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(fifth)).expect("land drop");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 2, "only four others");
    g.players[0].lands_played_this_turn = 0;
    let sixth = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(sixth)).expect("land drop");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 5);
}
