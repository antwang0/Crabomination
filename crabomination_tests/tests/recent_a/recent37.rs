//! Functionality tests for `catalog::sets::decks::recent37` — the Enchantress
//! draw cycle, black board wipes, and white/equipment value.

use crabomination::card::{CardType, Supertype};
use crabomination::catalog;
use crabomination::mana::Color;
use crabomination::game::two_player_game;
use crabomination::game::*;

fn cast(g: &mut GameState, id: CardId) {
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("spell castable");
    drain_stack(g);
}

#[test]
fn mesa_enchantress_draws_on_enchantment_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mesa_enchantress());
    g.add_card_to_library(0, catalog::island()); // something to draw
    let aura = g.add_card_to_hand(0, catalog::pacifism());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Pacifism");
    drain_stack(&mut g);
    // Cast the aura (−1 hand) then drew a card (+1) → net unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before, "enchantress replaced the cast aura");
}

#[test]
fn femeref_enchantress_draws_when_enchantment_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::femeref_enchantress());
    g.add_card_to_library(0, catalog::island());
    let ench = g.add_card_to_battlefield(0, catalog::pacifism());
    let hand_before = g.players[0].hand.len();
    g.remove_from_battlefield_to_graveyard_raw(ench);
    // The enchantment now sits in the graveyard; fire the put-into-graveyard event.
    g.dispatch_triggers_for_events(&[GameEvent::CardPutIntoGraveyard {
        player: 0, card_id: ench, is_land: false,
    }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew when an enchantment hit the graveyard");
}

#[test]
fn eidolon_of_blossoms_draws_on_its_own_entry() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let eid = g.add_card_to_battlefield(0, catalog::eidolon_of_blossoms());
    let hand_before = g.players[0].hand.len();
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: eid }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "constellation fires on its own entry");
}

#[test]
fn mutilate_scales_with_swamps() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::swamp()); }
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let mut0 = g.add_card_to_hand(0, catalog::mutilate());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, mut0);
    // -3/-3 (three Swamps) kills the 2/2.
    assert!(g.battlefield_find(foe).is_none(), "three Swamps → -3/-3 wipe");
}

#[test]
fn golden_demise_minus_two() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gd = g.add_card_to_hand(0, catalog::golden_demise());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, gd);
    assert!(g.battlefield_find(foe).is_none(), "-2/-2 kills a 2/2");
}

#[test]
fn yahennis_expertise_minus_three() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let ye = g.add_card_to_hand(0, catalog::yahennis_expertise());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, ye);
    // 4/4 → 1/1, survives; assert it took the shrink (toughness now 1).
    assert_eq!(g.computed_permanent(foe).unwrap().toughness, 1, "-3/-3 applied");
}

#[test]
fn sword_of_the_animist_is_legendary() {
    assert!(catalog::sword_of_the_animist().supertypes.contains(&Supertype::Legendary));
}

#[test]
fn dawn_of_hope_makes_a_soldier() {
    let mut g = two_player_game();
    let dawn = g.add_card_to_battlefield(0, catalog::dawn_of_hope());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: dawn, ability_index: 0, target: None, additional_targets: Vec::new(),
        x_value: None,
    }).expect("{3}{W}: make a Soldier");
    drain_stack(&mut g);
    let sol = g.battlefield.iter().find(|c| c.definition.name == "Soldier")
        .expect("Soldier minted");
    assert!(sol.definition.keywords.contains(&crabomination::card::Keyword::Lifelink));
    assert!(sol.definition.card_types.contains(&CardType::Creature));
}

/// Golden Demise wipes everything without the city's blessing, but spares your
/// creatures once you have it.
#[test]
fn golden_demise_spares_your_board_with_city_blessing() {
    // No blessing: -2/-2 hits all creatures; both 2/2s die.
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gd = g.add_card_to_hand(0, catalog::golden_demise());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, gd);
    assert!(!g.battlefield.iter().any(|c| c.id == mine), "your creature dies without the blessing");
    assert!(!g.battlefield.iter().any(|c| c.id == theirs), "their creature dies too");

    // With the blessing: only opponents' creatures take -2/-2.
    let mut g = two_player_game();
    g.players[0].city_blessing = true;
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gd = g.add_card_to_hand(0, catalog::golden_demise());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, gd);
    assert!(g.battlefield.iter().any(|c| c.id == mine), "your creature is spared with the blessing");
    assert!(!g.battlefield.iter().any(|c| c.id == theirs), "their creature still dies");
}

/// Yahenni's Expertise wipes the board (-3/-3) and lets you free-cast a spell
/// with mana value 3 or less from your hand.
#[test]
fn yahennis_expertise_wipes_then_free_casts() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 dies to -3/-3
    let free = g.add_card_to_hand(0, catalog::grizzly_bears()); // MV 2 → castable free
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![free])]));
    let ye = g.add_card_to_hand(0, catalog::yahennis_expertise());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, ye);
    assert!(!g.battlefield.iter().any(|c| c.id == theirs), "board wiped by -3/-3");
    assert!(g.battlefield_find(free).is_some(), "MV≤3 hand card free-cast");
}
