//! Functionality tests for `catalog::sets::decks::recent135` (WOE wave 8) and
//! the primitives it introduces: `Predicate::CastSpellIsAdventure`,
//! `Effect::CreateTokenAttachedToEach`, and the Young Hero Role toughness gate.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{AttackTarget, Target};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

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

fn kill(g: &mut GameState, id: CardId) {
    let ctl = g.battlefield_find(id).unwrap().controller;
    let ctx = crabomination::game::effects::EffectContext::for_ability(id, ctl, Some(Target::Permanent(id)));
    g.resolve_effect(
        &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &ctx,
    )
    .unwrap();
    g.dispatch_triggers_for_events(&[]);
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn has_token_attached(g: &GameState, host: CardId, name: &str) -> bool {
    g.battlefield.iter().any(|c| c.attached_to == Some(host) && c.definition.name == name)
}

fn tokens_named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

/// Chancellor of Tales copies an Adventure spell (via `CastSpellIsAdventure`),
/// but not a normal instant/sorcery.
#[test]
fn chancellor_copies_adventure_spell() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::chancellor_of_tales());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Puny Snack (Gingerbread Hunter's adventure): -2/-2. Two copies kill a 2/2.
    let gh = g.add_card_to_hand(0, catalog::gingerbread_hunter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastAdventure {
        card_id: gh,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Puny Snack");
    drain_stack(&mut g);
    // Original -2/-2 plus the copy's -2/-2 = -4/-4 → the 2/2 dies.
    assert!(g.battlefield_find(target).is_none(), "adventure copied → -4/-4 killed the bear");
}

/// A normal (non-Adventure) spell does not trigger Chancellor of Tales.
#[test]
fn chancellor_ignores_non_adventure() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::chancellor_of_tales());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::monstrous_rage()); // +2/+0, not an adventure
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(target)));
    // Only one Monster Role (no copy), so no second Role token was minted.
    assert_eq!(tokens_named(&g, "Monster"), 1, "non-adventure spell isn't copied");
}

/// Asinine Antics puts a Cursed Role on each opponent creature
/// (`CreateTokenAttachedToEach`), turning them all into 1/1s.
#[test]
fn asinine_antics_curses_each_opponent_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::asinine_antics());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, None);
    assert!(has_token_attached(&g, a, "Cursed"), "bear cursed");
    assert!(has_token_attached(&g, b, "Cursed"), "angel cursed");
    assert_eq!(g.computed_permanent(b).unwrap().power, 1, "Cursed Role sets the angel to 1/1");
}

/// The Young Hero Role's counter-on-attack only fires while toughness ≤ 3.
#[test]
fn young_hero_gate_blocks_big_creature() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Serra Angel is 4/4 — over the toughness gate.
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    // Attach a Young Hero Role via Embereth Veteran's sac ability.
    let vet = g.add_card_to_battlefield(0, catalog::embereth_veteran());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, vet, 0, Some(Target::Permanent(angel)));
    assert!(has_token_attached(&g, angel, "Young Hero"), "angel has the Role");
    g.clear_sickness(angel);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: angel,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(angel).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        0,
        "toughness 4 > 3 → no Young Hero counter",
    );
}

/// Feed the Cauldron destroys a small creature and, on your turn, makes a Food.
#[test]
fn feed_the_cauldron_makes_food_on_your_turn() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::feed_the_cauldron());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed");
    assert_eq!(tokens_named(&g, "Food"), 1, "your turn → Food created");
}

/// Collector's Vault loots and makes a Treasure.
#[test]
fn collectors_vault_loots_and_treasures() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let vault = g.add_card_to_battlefield(0, catalog::collectors_vault());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.clear_sickness(vault);
    activate(&mut g, vault, 0, None);
    assert_eq!(tokens_named(&g, "Treasure"), 1, "Treasure created");
}

/// Cooped Up stops its host attacking; the activated ability exiles it.
#[test]
fn cooped_up_locks_then_exiles() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::cooped_up());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::CantAttack),
        "enchanted creature can't attack",
    );
    let aura_id = g.battlefield.iter().find(|c| c.definition.name == "Cooped Up").unwrap().id;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, aura_id, 0, None);
    assert!(g.battlefield_find(bear).is_none(), "bear exiled");
}

/// Twisted Sewer-Witch makes a Rat, then a Wicked Role on every Rat it controls.
#[test]
fn twisted_sewer_witch_roles_all_rats() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let witch = g.add_card_to_hand(0, catalog::twisted_sewer_witch());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, witch, None);
    assert_eq!(tokens_named(&g, "Rat"), 1, "one Rat token");
    assert_eq!(tokens_named(&g, "Wicked"), 1, "a Wicked Role on the Rat");
    let rat = g.battlefield.iter().find(|c| c.definition.name == "Rat").unwrap().id;
    assert!(has_token_attached(&g, rat, "Wicked"), "Role attached to the Rat");
}

/// Mintstrosity leaves a Food behind when it dies.
#[test]
fn mintstrosity_dies_to_food() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let mint = g.add_card_to_battlefield(0, catalog::mintstrosity());
    kill(&mut g, mint);
    assert_eq!(tokens_named(&g, "Food"), 1, "death → Food");
}

/// Dream Spoilers only fires on the opponent's turn.
#[test]
fn dream_spoilers_fires_off_turn() {
    let mut g = two_player_game();
    // Opponent's turn (player 1 active); Dream Spoilers controller is player 0.
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::dream_spoilers());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let trick = g.add_card_to_hand(0, catalog::leaping_ambush());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    // Cast an instant during the opponent's turn → trigger targets their bear.
    g.perform_action(GameAction::CastSpell {
        card_id: trick,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast during opponent turn");
    drain_stack(&mut g);
    assert_eq!(
        g.computed_permanent(bear).unwrap().toughness,
        1,
        "opponent's bear got -1/-1 from Dream Spoilers",
    );
}

/// Elvish Archivist draws once when an enchantment enters (once-per-turn gate).
#[test]
fn elvish_archivist_draws_on_enchantment() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::elvish_archivist());
    g.add_card_to_library(0, catalog::grizzly_bears());
    // Cast an enchantment (Hopeful Vigil) so it enters and fires the draw.
    let ench = g.add_card_to_hand(0, catalog::hopeful_vigil());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, ench, None);
    // Cast consumed Hopeful Vigil (−1) but the trigger drew one (+1) → net even.
    assert_eq!(g.players[0].hand.len(), hand_before, "drew for the enchantment entering");
}

/// Eriette's Tempting Apple steals a creature until end of turn.
#[test]
fn tempting_apple_steals_creature() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let apple = g.add_card_to_hand(0, catalog::eriettes_tempting_apple());
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, apple, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "gained control of the bear");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "and it has haste");
}
