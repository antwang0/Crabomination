//! Commander: the 2026-10-25 (`01Y6ZRFx`) Spider-Man, Hometown Hero closers
//! (`decks::cmdr_01y6_e`).

use crabomination::card::{CardId, CounterType, Keyword};
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

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
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

/// CR 702.142 — Frenzied Raider grows when a boast ability is activated,
/// not for another activated ability.
#[test]
fn frenzied_raider_grows_on_boast() {
    let mut g = pod(2);
    let raider = ready(&mut g, 0, catalog::frenzied_raider());
    let arni = ready(&mut g, 0, catalog::arni_brokenbrow());
    let noose = ready(&mut g, 0, catalog::trip_noose());
    g.battlefield_find_mut(arni).unwrap().attacked_this_turn = true;
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, noose, Some(Target::Permanent(arni))).expect("noose");
    assert_eq!(g.battlefield_find(raider).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "not a boast");
    activate(&mut g, arni, None).expect("boast");
    assert_eq!(g.battlefield_find(raider).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Skyclave Sentinel attacks only with a +1/+1 counter on it.
#[test]
fn skyclave_sentinel_attacks_with_a_counter() {
    for counter in [false, true] {
        let mut g = pod(2);
        let s = ready(&mut g, 0, catalog::skyclave_sentinel());
        if counter {
            g.battlefield_find_mut(s).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
        }
        g.step = TurnStep::DeclareAttackers;
        let ok = g.declare_attackers(vec![Attack { attacker: s, target: AttackTarget::Player(1) }]).is_ok();
        assert_eq!(ok, counter);
    }
}

/// Imminent Doom fires for a spell of its counter count, then counts up.
#[test]
fn imminent_doom_counts_up() {
    let mut g = pod(2);
    let doom = g.add_card_to_battlefield(0, catalog::imminent_doom());
    g.battlefield_find_mut(doom).unwrap().add_counters(CounterType::Doom, 1);
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 3 - 1, "Bolt and a 1-point doom ping");
    assert_eq!(g.battlefield_find(doom).unwrap().counter_count(CounterType::Doom), 2);
}

/// Redcap Melee costs you a land for hitting a nonred creature only.
#[test]
fn redcap_melee_costs_a_land_against_nonred() {
    for (def, sacrifices) in [(catalog::grizzly_bears(), true), (catalog::hill_giant(), false)] {
        let mut g = pod(2);
        let land = g.add_card_to_battlefield(0, catalog::mountain());
        g.battlefield_find_mut(land).unwrap().tapped = true;
        let t = g.add_card_to_battlefield(1, def);
        let melee = g.add_card_to_hand(0, catalog::redcap_melee());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast(&mut g, melee, Some(Target::Permanent(t)));
        assert!(g.battlefield_find(t).is_none());
        assert_eq!(g.battlefield_find(land).is_none(), sacrifices);
    }
}

/// Blow Your House Down: the Bears can't block, the Wall is destroyed.
#[test]
fn blow_your_house_down_flattens_walls() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_stone());
    let spell = g.add_card_to_hand(0, catalog::blow_your_house_down());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(wall)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(bear, &Keyword::CantBlock));
    assert!(g.battlefield_find(wall).is_none());
}

/// Dwarven Armory works only during an upkeep, for a land.
#[test]
fn dwarven_armory_arms_in_upkeep() {
    let mut g = pod(2);
    let armory = g.add_card_to_battlefield(0, catalog::dwarven_armory());
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    assert!(activate(&mut g, armory, Some(Target::Permanent(bear))).is_err(), "main phase");
    g.step = TurnStep::Upkeep;
    activate(&mut g, armory, Some(Target::Permanent(bear))).expect("upkeep");
    assert!(g.battlefield_find(land).is_none());
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4));
}
