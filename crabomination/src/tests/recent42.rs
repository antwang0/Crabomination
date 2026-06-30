//! Functionality tests for `catalog::sets::decks::recent42` — charge-counter
//! bombs (`Effect::DestroyEachNonlandWithManaValue`), Gaddock Teeg's cast lock
//! (`StaticEffect::NoncreatureSpellsCantBeCastIf`), and The Tabernacle's granted
//! upkeep tax.

use crate::card::CounterType;
use crate::catalog;
use crate::game::effects::EffectContext;
use crate::game::two_player_game;
use crate::game::*;

fn activate(g: &mut GameState, id: CardId, idx: usize) {
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: idx, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("ability activates");
    drain_stack(g);
}

#[test]
fn ratchet_bomb_blows_up_matching_mana_value() {
    let mut g = two_player_game();
    let bomb = g.add_card_to_battlefield(0, catalog::ratchet_bomb());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // {1}{G} = MV 2
    let one_drop = g.add_card_to_battlefield(1, catalog::llanowar_elves()); // MV 1
    let land = g.add_card_to_battlefield(1, catalog::forest()); // nonland-only filter spares it
    // Tick to two charge counters, then detonate.
    g.battlefield_find_mut(bomb).unwrap().add_counters(CounterType::Charge, 2);
    activate(&mut g, bomb, 1); // {T}, Sacrifice: destroy each nonland with MV == 2
    assert!(g.battlefield_find(bear).is_none(), "MV-2 creature destroyed");
    assert!(g.battlefield_find(one_drop).is_some(), "MV-1 creature spared");
    assert!(g.battlefield_find(land).is_some(), "lands are never hit");
    assert!(g.battlefield_find(bomb).is_none(), "bomb sacrificed itself");
}

#[test]
fn engineered_explosives_enters_with_sunburst_counters() {
    // Two distinct colors of mana → two charge counters (ConvergedValue).
    assert_eq!(
        catalog::engineered_explosives().enters_with_counters.unwrap().0,
        CounterType::Charge
    );
}

#[test]
fn sphere_of_the_suns_taps_for_any_color_off_a_charge_counter() {
    let mut g = two_player_game();
    let sphere = g.add_card_to_battlefield(0, catalog::sphere_of_the_suns());
    g.battlefield_find_mut(sphere).unwrap().add_counters(CounterType::Charge, 3);
    activate(&mut g, sphere, 0);
    assert_eq!(g.players[0].mana_pool.total(), 1, "added one mana");
    assert_eq!(
        g.battlefield_find(sphere).unwrap().counter_count(CounterType::Charge),
        2,
        "spent a charge counter"
    );
}

#[test]
fn gaddock_teeg_locks_expensive_noncreature_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gaddock_teeg());
    assert!(g.noncreature_spell_cast_locked(&catalog::wrath_of_god()), "MV-4 sorcery is locked");
    assert!(g.noncreature_spell_cast_locked(&catalog::engineered_explosives()), "an X-cost artifact is locked");
    assert!(!g.noncreature_spell_cast_locked(&catalog::grizzly_bears()), "creatures are exempt");
    assert!(!g.noncreature_spell_cast_locked(&catalog::lightning_bolt()), "cheap noncreature spells are fine");
}

#[test]
fn tabernacle_grants_every_creature_an_upkeep_tax() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_tabernacle_at_pendrell_vale());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let granted = g.statics_granted_triggers_for(g.battlefield_find(bear).unwrap());
    assert!(!granted.is_empty(), "the bear inherits the Tabernacle's upkeep trigger");
}

#[test]
fn tabernacle_destroys_an_unpaid_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_tabernacle_at_pendrell_vale());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Resolve the granted "pay {1} or destroy" with no mana available.
    let granted = g.statics_granted_triggers_for(g.battlefield_find(bear).unwrap());
    let effect = granted[0].effect.clone();
    let ctx = EffectContext::for_ability(bear, 0, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "unpaid creature is destroyed");
}
