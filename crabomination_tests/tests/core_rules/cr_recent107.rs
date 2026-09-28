//! CR 608.2d — "return a [permanent] you control to its owner's hand" is a
//! choice made as the effect resolves, not a target
//! (`Effect::ReturnOneYouControl`). Whitemane Lion, Stonecloaker, Kor
//! Skyfisher, Cavern Harpy, Guildless Commons, Species Gorger, Zell Dincht
//! and Time Wipe declared a target, so the ability could fizzle and the bot
//! named the card when the trigger went on the stack.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

fn cast(g: &mut GameState, id: CardId) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn in_hand(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].hand.iter().any(|c| c.id == id)
}

#[test]
fn cr_608_2d_return_a_creature_you_control_is_not_a_target() {
    for def in [
        catalog::whitemane_lion(),
        catalog::stonecloaker(),
        catalog::kor_skyfisher(),
        catalog::cavern_harpy(),
        catalog::guildless_commons(),
        catalog::species_gorger(),
        catalog::zell_dincht(),
    ] {
        let body = def.triggered_abilities.iter().find(|t| {
            matches!(t.effect, crabomination::effect::Effect::ReturnOneYouControl { .. })
        });
        assert!(body.is_some_and(|t| !t.effect.requires_target()), "{}", def.name);
    }
    assert!(!catalog::time_wipe().effect.requires_target());
}

/// Whitemane Lion returns another creature when there is one (the cheapest),
/// and itself when it is alone.
#[test]
fn cr_608_2d_whitemane_lion_picks_on_resolution() {
    let mut g = main_phase();
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let lion = g.add_card_to_hand(0, catalog::whitemane_lion());
    g.players[0].mana_pool.add(Color::White, 2);
    cast(&mut g, lion);
    assert!(in_hand(&g, 0, bears), "the cheapest other creature");
    assert!(g.battlefield_find(angel).is_some() && g.battlefield_find(lion).is_some());

    let mut g = main_phase();
    let lion = g.add_card_to_hand(0, catalog::whitemane_lion());
    g.players[0].mana_pool.add(Color::White, 2);
    cast(&mut g, lion);
    assert!(in_hand(&g, 0, lion), "alone, it returns itself");
}

/// Guildless Commons returns a tapped land, never itself while another land
/// is there to bounce.
#[test]
fn cr_608_2d_guildless_commons_keeps_itself() {
    let mut g = main_phase();
    let island = g.add_card_to_battlefield(0, catalog::island());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(forest).unwrap().tapped = true;
    let commons = g.add_card_to_hand(0, catalog::guildless_commons());
    g.perform_action(GameAction::PlayLand(commons)).expect("land drop");
    drain_stack(&mut g);
    assert!(in_hand(&g, 0, forest), "the tapped land comes back");
    assert!(g.battlefield_find(commons).is_some() && g.battlefield_find(island).is_some());
}

/// Time Wipe saves the best creature you control, then destroys the rest.
#[test]
fn cr_608_2d_time_wipe_saves_the_best_creature() {
    let mut g = main_phase();
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wipe = g.add_card_to_hand(0, catalog::time_wipe());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, wipe);
    assert!(in_hand(&g, 0, angel));
    assert!(g.battlefield_find(bears).is_none() && g.battlefield_find(theirs).is_none());
}
