//! Functionality tests for `catalog::sets::decks::recent50`.

use crate::catalog;
use crate::game::effects::EffectContext;
use crate::game::two_player_game;
use crate::game::*;

#[test]
fn enigma_drake_power_tracks_graveyard_spells() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature — not counted
    let drake = g.add_card_to_battlefield(0, catalog::enigma_drake());
    let cp = g.computed_permanent(drake).unwrap();
    assert_eq!(cp.power, 2, "two instants in the graveyard");
    assert_eq!(cp.toughness, 4);
}

#[test]
fn niblis_of_frost_taps_and_locks_on_spellcast() {
    let mut g = two_player_game();
    let niblis = g.add_card_to_battlefield(0, catalog::niblis_of_frost());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let trig = catalog::niblis_of_frost().triggered_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_trigger(niblis, 0, None, 0);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&trig, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "opponent's creature tapped");
    assert!(g.battlefield_find(foe).unwrap().skip_next_untap, "and locked down for its next untap");
}

#[test]
fn wavesifter_investigates_twice() {
    let mut g = two_player_game();
    let ws = g.add_card_to_battlefield(0, catalog::wavesifter());
    g.fire_self_etb_triggers(ws, 0);
    drain_stack(&mut g);
    let clues = g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == 0).count();
    assert_eq!(clues, 2, "two Clue tokens");
}
