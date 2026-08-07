//! CR conformance for this run:
//! - CR 702.26d — phasing in or out is not a zone change, so it fires no
//!   enters- or leaves-the-battlefield triggers.
//! - CR 704.5n — an Equipment's host legality reads the *computed* type line,
//!   so an animated land keeps what's bolted to it.
//! - CR 613.1c — a layer-4 animation adds the creature type without taking the
//!   land's own types or mana ability away.
//! - CR 602.5 — City of Solitude's off-turn lock covers activated abilities,
//!   mana abilities included, and the server view surfaces it.

use crabomination::card::{CardDefinition, CardId, CardType, LandType};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn phase_out(g: &mut GameState, id: CardId) {
    let ctx = EffectContext::for_ability(id, 0, None);
    g.resolve_effect(&Effect::PhaseOut { what: Selector::This, until_source_leaves: false }, &ctx)
        .expect("phase out");
}

/// CR 702.26d — phasing out fires no leaves-the-battlefield trigger, and
/// phasing back in fires no enters-the-battlefield trigger.
#[test]
fn cr_702_26d_phasing_fires_no_zone_change_triggers() {
    let mut g = two_player_game();
    // Suleiman's Legacy watches every Djinn/Efreet that *enters*.
    g.add_card_to_battlefield(0, catalog::suleimans_legacy());
    let efreet = ready(&mut g, 0, catalog::rainbow_efreet());
    phase_out(&mut g, efreet);
    drain_stack(&mut g);
    assert!(g.battlefield_find(efreet).is_none(), "phased out");
    g.active_player_idx = 0;
    g.do_phasing();
    drain_stack(&mut g);
    assert!(g.battlefield_find(efreet).is_some(), "phased back in, and survived");
}

/// CR 704.5n — an Equipment stays attached to a land that's been animated into
/// a creature; the legality check is layer-aware, not printed-type-only.
#[test]
fn cr_704_5n_equipment_stays_on_an_animated_land() {
    let mut g = two_player_game();
    let druid = ready(&mut g, 0, catalog::quirion_druid());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: druid,
        ability_index: 0,
        target: Some(Target::Permanent(forest)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("animate the Forest");
    drain_stack(&mut g);
    let sword = g.add_card_to_battlefield(0, catalog::short_bow());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(forest);
    g.check_state_based_actions();
    assert_eq!(
        g.battlefield_find(sword).and_then(|c| c.attached_to),
        Some(forest),
        "the animated land is a legal host"
    );
}

/// CR 613.1c — the layer-4 animation adds Creature; the land keeps its land
/// type and its mana ability.
#[test]
fn cr_613_1c_animated_land_keeps_its_land_types() {
    let mut g = two_player_game();
    let druid = ready(&mut g, 0, catalog::quirion_druid());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: druid,
        ability_index: 0,
        target: Some(Target::Permanent(forest)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(forest).expect("still on the battlefield");
    assert!(cp.card_types.contains(&CardType::Creature) && cp.card_types.contains(&CardType::Land));
    assert!(cp.subtypes.land_types.contains(&LandType::Forest));
    g.clear_sickness(forest);
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("it still taps for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

/// City of Solitude shuts an off-turn seat out of activations too, and the
/// server view says so.
#[test]
fn cr_602_5_city_of_solitude_locks_off_turn_activations() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::city_of_solitude());
    let land = ready(&mut g, 1, catalog::forest());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: land,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "even a mana ability waits for their own turn"
    );
    let view = crabomination::server::view::project(&g, 1);
    let lock = &view.players[1].spell_cast_lock;
    assert!(lock.off_turn_locked && lock.off_turn_abilities_locked);
    assert!(!crabomination::server::view::project(&g, 0).players[0]
        .spell_cast_lock
        .off_turn_abilities_locked);
}
