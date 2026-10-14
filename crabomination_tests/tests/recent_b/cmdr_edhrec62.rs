//! Commander: EDHREC ranks 2001-2500 one-card gaps (`decks::cmdr_edhrec62`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

/// Khorvath's Fury: the bot picks foes among opponents (friend/foe, CR 810
/// Battlebond wording), burning each for their hand size.
#[test]
fn khorvaths_fury_burns_foes_by_hand_size() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::island());
    }
    let life = g.players[1].life;
    let fury = g.add_card_to_hand(0, catalog::khorvaths_fury());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell { card_id: fury, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "the opponent is a foe with three cards");
}

/// Ghostfire Blade's equip costs {1} onto a colorless creature, {3} otherwise.
#[test]
fn ghostfire_blade_is_cheap_on_colorless() {
    let mut g = pod(2);
    let blade = ready(&mut g, 0, catalog::ghostfire_blade());
    let construct = ready(&mut g, 0, catalog::sparring_construct());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: blade, target: construct }).expect("{1} onto the Construct");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(blade).unwrap().attached_to, Some(construct));
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::Equip { equipment: blade, target: bears }).is_err(), "{{3}} for the Bears");
}

/// Istvan takes no damage from creatures (CR 615) and grows when an
/// opponent's creature dies.
#[test]
fn istvan_shrugs_off_creatures_and_feeds() {
    let mut g = pod(2);
    let istvan = ready(&mut g, 0, catalog::istvan_butcher_of_eln());
    let archer = ready(&mut g, 1, catalog::scattershot_archer());
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(istvan), 2, Some(archer), &mut evs);
    assert_eq!(g.battlefield_find(istvan).unwrap().damage, 0, "a creature source is prevented");
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.destroy_permanent(bears, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(istvan).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}
