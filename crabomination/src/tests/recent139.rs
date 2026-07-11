//! Functionality tests for `catalog::sets::decks::recent139` (WOE wave 12).

use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn rat_count(g: &GameState) -> usize {
    g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Rat").count()
}

/// Misleading Motes sends a creature to its owner's library.
#[test]
fn misleading_motes_bounces_to_library() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::misleading_motes());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, spell, Some(Target::Permanent(enemy)), None);
    assert!(g.battlefield_find(enemy).is_none(), "creature left the battlefield");
    assert!(
        g.players[1].library.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "put into its owner's library",
    );
}

/// Taken by Nightmares exiles a creature.
#[test]
fn taken_by_nightmares_exiles() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::taken_by_nightmares());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, Some(Target::Permanent(enemy)), None);
    assert!(g.battlefield_find(enemy).is_none(), "creature exiled");
    assert!(g.exile.iter().any(|c| c.definition.name == "Grizzly Bears"), "in exile");
}

/// Faerie Fencing gives -X/-X, plus -3/-3 more when you control a Faerie.
#[test]
fn faerie_fencing_faerie_bonus() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // A Faerie you control turns X=1 into a lethal -4/-4 on a 3/3.
    g.add_card_to_battlefield(0, catalog::spellstutter_sprite());
    let target = g.add_card_to_battlefield(1, catalog::centaur_courser()); // 3/3
    let spell = g.add_card_to_hand(0, catalog::faerie_fencing());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1); // X = 1
    cast(&mut g, spell, Some(Target::Permanent(target)), Some(1));
    assert!(g.battlefield_find(target).is_none(), "-1/-1 plus Faerie -3/-3 = -4/-4 killed the 3/3");
}

/// Flick a Coin pings, makes a Treasure, and draws.
#[test]
fn flick_a_coin_ping_treasure_draw() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::flick_a_coin());
    let before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[1].life;
    cast(&mut g, spell, Some(Target::Player(1)), None);
    assert_eq!(g.players[1].life, life - 1, "1 damage to the player");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"), "made a Treasure");
    assert_eq!(g.players[0].hand.len(), before, "-1 spell +1 draw = even");
}

/// Frantic Firebolt scales with instant/sorcery cards in your graveyard.
#[test]
fn frantic_firebolt_scales_with_graveyard() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
    let target = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let spell = g.add_card_to_hand(0, catalog::frantic_firebolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, Some(Target::Permanent(target)), None);
    // 2 + 2 instants = 4 damage → kills the 4/4.
    assert!(g.battlefield_find(target).is_none(), "4 damage killed the 4/4");
}

/// Ogre Chitterlord makes two Rats when it enters.
#[test]
fn ogre_chitterlord_makes_rats() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::ogre_chitterlord());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, id, None, None);
    assert_eq!(rat_count(&g), 2, "two Rats on entry");
}

/// Redcap Gutter-Dweller makes two Rats when it enters.
#[test]
fn redcap_gutter_dweller_makes_rats() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::redcap_gutter_dweller());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id, None, None);
    assert_eq!(rat_count(&g), 2, "two Rats on entry");
}

/// Shatter the Oath destroys a creature and hangs a Wicked Role on your creature.
#[test]
fn shatter_the_oath_destroy_and_role() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let enemy = g.add_card_to_battlefield(1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::shatter_the_oath());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, spell, Some(Target::Permanent(enemy)), None);
    assert!(g.battlefield_find(enemy).is_none(), "target destroyed");
    assert!(
        g.battlefield.iter().any(|c| c.attached_to == Some(mine) && c.definition.name == "Wicked"),
        "Wicked Role attached to your creature",
    );
}

/// Tattered Ratter pumps a Rat that becomes blocked.
#[test]
fn tattered_ratter_pumps_blocked_rat() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tattered_ratter());
    // A 1/1 Rat attacker.
    let attacker = g.add_card_to_battlefield(0, {
        use crate::card::{CardDefinition, CardType, CreatureType, Subtypes};
        CardDefinition {
            name: "Test Rat",
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
            ..Default::default()
        }
    });
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(blocker);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(attacker).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 1), "blocked Rat got +2/+0");
}

/// Redtooth Vanguard returns from the graveyard when an enchantment enters.
#[test]
fn redtooth_vanguard_recurs_on_enchantment() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_graveyard(0, catalog::redtooth_vanguard());
    // Cast an enchantment; the graveyard trigger offers to pay {2}.
    let ench = g.add_card_to_hand(0, catalog::a_tale_for_the_ages());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3); // {1} for the aura + {2} to recur
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, ench, None, None);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Redtooth Vanguard"),
        "Redtooth Vanguard returned to hand",
    );
}
