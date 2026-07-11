//! Functionality tests for WOE wave 11 — completing cards previously deferred
//! in TODO.md (`catalog::sets::decks::recent138` plus in-place completions of
//! Experimental Confectioner and Torch the Tower).

use crate::card::{CardInstance, Keyword};
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, mode: Option<usize>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, idx: usize) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// A Tale for the Ages gives +2/+2 to an enchanted creature you control, but not
/// to an unenchanted one.
#[test]
fn a_tale_for_the_ages_anthems_enchanted() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let enchanted = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    // Hang a Royal Role (+1/+1) so `enchanted` is enchanted.
    let rt = g.add_card_to_hand(0, catalog::royal_treatment());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, rt, Some(Target::Permanent(enchanted)), None);
    g.add_card_to_battlefield(0, catalog::a_tale_for_the_ages());
    // enchanted: 2/2 base + Royal Role +1/+1 + anthem +2/+2 = 5/5.
    let cp = g.computed_permanent(enchanted).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "anthem hits the enchanted creature");
    // plain: no Aura → no anthem.
    let cp = g.computed_permanent(plain).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "unenchanted creature unaffected");
}

/// Break the Spell draws when destroying an enchantment you control; not when
/// destroying an opponent's nontoken enchantment.
#[test]
fn break_the_spell_conditional_draw() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let mine = g.add_card_to_battlefield(0, catalog::a_tale_for_the_ages());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::break_the_spell());
    let before = g.players[0].hand.len(); // includes the spell itself
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, spell, Some(Target::Permanent(mine)), None);
    assert!(g.battlefield_find(mine).is_none(), "own enchantment destroyed");
    // -1 for the spell leaving hand, +1 for the draw → net even.
    assert_eq!(g.players[0].hand.len(), before, "drew a card for own enchantment");

    // Opponent's nontoken enchantment → no draw.
    let theirs = g.add_card_to_battlefield(1, catalog::a_tale_for_the_ages());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::break_the_spell());
    let before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, spell, Some(Target::Permanent(theirs)), None);
    assert!(g.battlefield_find(theirs).is_none(), "opponent enchantment destroyed");
    assert_eq!(g.players[0].hand.len(), before - 1, "no draw (not yours, not a token)");
}

/// Moment of Valor mode 2 destroys a creature with power 4 or greater.
#[test]
fn moment_of_valor_destroys_big() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let spell = g.add_card_to_hand(0, catalog::moment_of_valor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, Some(Target::Permanent(angel)), Some(1));
    assert!(g.battlefield_find(angel).is_none(), "power-4 creature destroyed");
}

/// Moment of Valor mode 1 untaps a creature and gives it +1/+0.
#[test]
fn moment_of_valor_untap_and_pump() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::moment_of_valor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, Some(Target::Permanent(bear)), Some(0));
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2), "+1/+0");
    assert!(cp.keywords.contains(&Keyword::Indestructible), "gained indestructible");
}

/// Gruff Triplets makes two token copies of itself on entry (three total).
#[test]
fn gruff_triplets_self_copies() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::gruff_triplets());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id, None, None);
    let count = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Gruff Triplets")
        .count();
    assert_eq!(count, 3, "original plus two token copies");
}

/// Howling Galefang has haste only while you own an exiled Adventure card.
#[test]
fn howling_galefang_haste_from_exiled_adventure() {
    let mut g = two_player_game();
    let gale = g.add_card_to_battlefield(0, catalog::howling_galefang());
    assert!(
        !g.computed_permanent(gale).unwrap().keywords.contains(&Keyword::Haste),
        "no haste without an exiled Adventure",
    );
    let mut ex = CardInstance::new(g.next_id(), catalog::minecart_daredevil(), 0);
    ex.on_adventure = true;
    g.exile.push(ex);
    assert!(
        g.computed_permanent(gale).unwrap().keywords.contains(&Keyword::Haste),
        "haste while an owned Adventure waits in exile",
    );
}

/// Experimental Confectioner turns a sacrificed Food into a Rat.
#[test]
fn experimental_confectioner_food_to_rat() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::experimental_confectioner());
    let food = g.add_token_to_battlefield(0, &crate::game::effects::food_token());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, food, 0); // {2}, {T}, Sac: gain 3 life
    assert!(
        !g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"),
        "Food sacrificed",
    );
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Rat"),
        "sacrificing a Food made a Rat",
    );
}

/// Bargained Torch the Tower deals 3, scries, and exiles a lethally-damaged
/// target instead of letting it die to the graveyard.
#[test]
fn torch_the_tower_bargained_exiles_target() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let fodder = g.add_token_to_battlefield(0, &crate::game::effects::food_token());
    let id = g.add_card_to_hand(0, catalog::torch_the_tower());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpellBargain {
        card_id: id,
        sacrifice: Some(fodder),
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Torch the Tower bargained");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "3 damage killed the 2/2");
    assert!(
        g.exile.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "lethally-damaged target exiled, not buried",
    );
    assert!(
        !g.players[1].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "not in the graveyard",
    );
}
