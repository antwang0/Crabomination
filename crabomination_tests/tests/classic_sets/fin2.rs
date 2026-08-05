//! Final Fantasy (FIN) gap closure: Town // Adventure lands and Sidequests.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
}

/// Advance to `step` on seat 0's next turn (games start mid-turn, so the naive
/// walk to Upkeep lands on the opponent's).
fn advance_to_your(g: &mut GameState, step: TurnStep) {
    for p in 0..g.players.len() {
        for i in 0..40 {
            g.players[p].library.push(CardInstance::new(CardId(5000 + (p * 100 + i) as u32), catalog::forest(), p));
        }
    }
    while !(g.step == step && g.active_player_idx == 0) {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast_adventure(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastAdventure {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast adventure");
    drain_stack(g);
}

/// Every new FIN factory is registered under its printed name.
#[test]
fn fin2_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::ishgard_the_holy_see as fn() -> crabomination::card::CardDefinition,
        catalog::jidoor_aristocratic_capital,
        catalog::lindblum_industrial_regency,
        catalog::midgar_city_of_mako,
        catalog::zanarkand_ancient_metropolis,
        catalog::balamb_garden_seed_academy,
        catalog::sidequest_card_collection,
        catalog::sidequest_catch_a_fish,
        catalog::sidequest_hunt_the_mark,
        catalog::sidequest_play_blitzball,
        catalog::sidequest_raise_a_chocobo,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

// ── Town // Adventure lands (CR 715.3d) ─────────────────────────────────────

/// The land half is played out of adventure exile — not cast — and it costs the
/// turn's land drop.
#[test]
fn adventure_land_half_is_played_from_exile() {
    let mut g = two_player_game();
    let ishgard = g.add_card_to_hand(0, catalog::ishgard_the_holy_see());
    g.add_card_to_graveyard(0, catalog::sol_ring());
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::plains());
    }
    g.players[0].mana_pool.add(Color::White, 5);
    let sol_ring = g.players[0].graveyard[0].id;
    g.perform_action(GameAction::CastAdventure {
        card_id: ishgard,
        target: Some(Target::Permanent(sol_ring)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Faith & Grief");
    drain_stack(&mut g);
    assert!(
        g.exile.iter().any(|c| c.id == ishgard && c.on_adventure),
        "Faith & Grief exiled the land half on the adventure"
    );
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Sol Ring"));
    g.perform_action(GameAction::PlayLand(ishgard)).expect("play the land half");
    let land = g.battlefield_find(ishgard).expect("Ishgard is on the battlefield");
    assert!(land.tapped, "Towns enter tapped");
    assert_eq!(g.players[0].lands_played_this_turn, 1);
}

/// The exiled land half is surfaced to the client as a playable affordance.
#[test]
fn adventure_land_half_is_offered_as_an_affordance() {
    let mut g = two_player_game();
    let midgar = g.add_card_to_hand(0, catalog::midgar_city_of_mako());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    g.players[0].mana_pool.add(Color::Black, 3);
    cast_adventure(&mut g, midgar, None);
    assert_eq!(g.compute_hand_affordances(0).adventure_exile, vec![midgar]);
    // A spent land drop takes it back off the list.
    g.players[0].lands_played_this_turn = 1;
    assert!(g.compute_hand_affordances(0).adventure_exile.is_empty());
}

/// Overture mills half the target opponent's library, rounded down.
#[test]
fn overture_mills_half_rounded_down() {
    let mut g = two_player_game();
    let jidoor = g.add_card_to_hand(0, catalog::jidoor_aristocratic_capital());
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    g.players[0].mana_pool.add(Color::Blue, 6);
    let before = g.players[1].library.len();
    cast_adventure(&mut g, jidoor, Some(Target::Player(1)));
    assert_eq!(g.players[1].library.len(), before - before / 2);
}

/// Lasting Fayth's Hero scales with the lands you control.
#[test]
fn lasting_fayth_hero_counts_your_lands() {
    let mut g = two_player_game();
    let zanarkand = g.add_card_to_hand(0, catalog::zanarkand_ancient_metropolis());
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.players[0].mana_pool.add(Color::Green, 6);
    cast_adventure(&mut g, zanarkand, None);
    let hero = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Hero")
        .expect("Hero token");
    assert_eq!(hero.counters.get(&CounterType::PlusOnePlusOne).copied(), Some(6));
}

/// Balamb Garden's transform cost drops {1} per other Town you control.
#[test]
fn balamb_garden_transform_is_discounted_by_towns() {
    let mut g = two_player_game();
    let garden = g.add_card_to_battlefield(0, catalog::balamb_garden_seed_academy());
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::treno_dark_city());
    }
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.battlefield_find_mut(garden).unwrap().tapped = false;
    // {5}{G}{U} less {2} for the two other Towns = {3}{G}{U}.
    g.players[0].mana_pool.add(Color::Green, 4);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: garden, ability_index: 2,
        target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("transform");
    drain_stack(&mut g);
    let bf = g.battlefield_find(garden).expect("still on the battlefield");
    assert_eq!(bf.definition.name, "Balamb Garden, Airborne");
    assert!(bf.definition.keywords.contains(&Keyword::Flying));
}

// ── Sidequests ──────────────────────────────────────────────────────────────

/// Card Collection draws three and discards two on entry, then transforms at
/// your end step once the graveyard holds eight cards.
#[test]
fn sidequest_card_collection_transforms_on_eight_cards() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_card_collection());
    for _ in 0..8 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    advance_to(&mut g, TurnStep::End);
    let bf = g.battlefield_find(quest).expect("still around");
    assert_eq!(bf.definition.name, "Magicked Card");
    assert!(bf.definition.card_types.contains(&CardType::Artifact));
}

/// Under eight cards the quest stays face up.
#[test]
fn sidequest_card_collection_waits_for_the_graveyard() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_card_collection());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Sidequest: Card Collection");
}

/// Catch a Fish's "if you put a card into your hand this way" rider makes the
/// Food and flips the quest.
#[test]
fn sidequest_catch_a_fish_rider_fires_on_a_pick() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_catch_a_fish());
    g.players[0].library.insert(0, CardInstance::new(CardId(900), catalog::grizzly_bears(), 0));
    advance_to_your(&mut g, TurnStep::Upkeep);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"));
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Cooking Campsite");
}

/// No eligible card on top → no Food, no transform.
#[test]
fn sidequest_catch_a_fish_rider_skips_on_a_miss() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_catch_a_fish());
    g.players[0].library.insert(0, CardInstance::new(CardId(901), catalog::lightning_bolt(), 0));
    advance_to_your(&mut g, TurnStep::Upkeep);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Food"));
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Sidequest: Catch a Fish");
}

/// Hunt the Mark makes a Treasure when an opponent's creature died, and flips
/// on the third.
#[test]
fn sidequest_hunt_the_mark_transforms_on_three_treasures() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_hunt_the_mark());
    for _ in 0..2 {
        g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    }
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(victim)));
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Yiazmat, Ultimate Mark");
}

/// Play Blitzball flips at end of combat once a player took 6+ combat damage.
#[test]
fn sidequest_play_blitzball_transforms_on_six_combat_damage() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_play_blitzball());
    g.players[1].combat_damage_taken_this_turn = 6;
    advance_to(&mut g, TurnStep::EndCombat);
    let bf = g.battlefield_find(quest).expect("still around");
    assert_eq!(bf.definition.name, "World Champion, Celestial Weapon");
}

/// Non-combat damage doesn't count toward Blitzball's threshold.
#[test]
fn sidequest_play_blitzball_ignores_noncombat_damage() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_play_blitzball());
    g.players[1].damage_taken_this_turn = 9;
    advance_to(&mut g, TurnStep::EndCombat);
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Sidequest: Play Blitzball");
}

/// Raise a Chocobo makes a Bird on entry and flips at your main phase with four.
#[test]
fn sidequest_raise_a_chocobo_transforms_on_four_birds() {
    let mut g = two_player_game();
    let quest = g.add_card_to_hand(0, catalog::sidequest_raise_a_chocobo());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::birds_of_paradise());
    }
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, quest, None);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Bird"), "ETB Bird token");
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Sidequest: Raise a Chocobo");
    g.add_card_to_battlefield(0, catalog::birds_of_paradise());
    advance_to_your(&mut g, TurnStep::PreCombatMain);
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Black Chocobo");
}
