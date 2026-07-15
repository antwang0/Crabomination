//! Functionality tests for `catalog::sets::decks::recent91` (Izzet spellslinger
//! legends + `Effect::ReturnRandomFromGraveyard` and
//! `SelectionRequirement::SharesNameWithControllerGraveyardCard`).

use crabomination::catalog;
use crabomination::card::{CounterType, CreatureType};
use crabomination::game::two_player_game;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

/// Cast Lightning Bolt from `seat` at `at`'s face.
fn bolt_face(g: &mut GameState, seat: usize, at: usize) {
    let bolt = g.add_card_to_hand(seat, catalog::lightning_bolt());
    g.players[seat].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = seat;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(at)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(g);
}

/// Draw one card for `seat`, firing its CardDrawn triggers.
fn draw(g: &mut GameState, seat: usize) {
    let mut ev = vec![];
    g.draw_one(seat, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

#[test]
fn kykar_mints_spirit_on_noncreature_and_sacs_for_red() {
    let mut g = two_player_game();
    let kykar = g.add_card_to_battlefield(0, catalog::kykar_winds_fury());
    bolt_face(&mut g, 0, 1);
    let spirit = g.battlefield.iter().find(|c| {
        c.definition.subtypes.creature_types.contains(&CreatureType::Spirit)
    }).map(|c| c.id).expect("Spirit token minted on noncreature cast");
    // Sacrifice the Spirit for one red mana.
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: kykar, ability_index: 0, target: None, additional_targets: Vec::new(),
        x_value: None,
    }).expect("sac Spirit for red");
    drain_stack(&mut g);
    assert!(g.battlefield_find(spirit).is_none(), "Spirit sacrificed");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "added one red");
}

#[test]
fn nivmizzet_parun_pings_on_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nivmizzet_parun());
    g.add_card_to_library(0, catalog::forest());
    g.players[1].life = 20;
    draw(&mut g, 0);
    assert_eq!(g.players[1].life, 19, "drawing dealt 1 to the opponent");
}

#[test]
fn nivmizzet_parun_draws_when_any_player_casts_is() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nivmizzet_parun());
    g.add_card_to_library(0, catalog::forest());
    let before = g.players[0].hand.len();
    // Opponent casts an instant — Niv's controller (P0) draws.
    bolt_face(&mut g, 1, 0);
    assert!(g.players[0].hand.len() > before, "P0 drew off the opponent's I/S cast");
}

#[test]
fn locust_god_mints_insect_on_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_locust_god());
    g.add_card_to_library(0, catalog::forest());
    draw(&mut g, 0);
    assert!(
        g.battlefield.iter().any(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Insect)),
        "drawing minted an Insect token",
    );
}

#[test]
fn veyran_pumps_on_instant_cast() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::veyran_voice_of_duality());
    bolt_face(&mut g, 0, 1);
    let cp = g.computed_permanent(v).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "magecraft plus one plus one");
}

#[test]
fn charmbreaker_returns_random_is_at_upkeep_and_pumps() {
    let mut g = two_player_game();
    let cb = g.add_card_to_battlefield(0, catalog::charmbreaker_devils());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "the lone I/S returned to hand");
    // Casting an I/S pumps Charmbreaker +4/+0.
    bolt_face(&mut g, 0, 1);
    let cp = g.computed_permanent(cb).unwrap();
    assert_eq!(cp.power, 8, "cast an I/S gave plus four power");
}

#[test]
fn pyromancer_ascension_counters_on_name_match_then_copies() {
    let mut g = two_player_game();
    let asc = g.add_card_to_battlefield(0, catalog::pyromancer_ascension());
    // A Bolt already in the graveyard so a cast Bolt shares its name.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    bolt_face(&mut g, 0, 1);
    assert_eq!(
        g.battlefield_find(asc).unwrap().counter_count(CounterType::Quest),
        1,
        "name-matched cast added a quest counter",
    );
    // Preload two counters, then a cast copies the spell (P1 takes 3 + 3).
    g.battlefield_find_mut(asc).unwrap().add_counters(CounterType::Quest, 2);
    g.players[1].life = 20;
    bolt_face(&mut g, 0, 1);
    assert_eq!(g.players[1].life, 14, "spell was copied for six total to face");
}

#[test]
fn izzet_guildmage_copies_low_mv_instant() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::izzet_guildmage());
    for c in g.battlefield.iter_mut() { c.summoning_sick = false; }
    g.players[0].life = 20;
    g.players[1].life = 20;
    // Put a Bolt on the stack targeting P1's face.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt on stack");
    // Activate the copy ability targeting the bolt on the stack.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Permanent(bolt)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("copy the bolt");
    drain_stack(&mut g);
    // The bolt + its copy each deal 3; the copy may pick a new target, so
    // assert on the total damage dealt across both players.
    let dealt = (20 - g.players[0].life) + (20 - g.players[1].life);
    assert_eq!(dealt, 6, "the ability copied the bolt — two instances of 3 resolved");
}
