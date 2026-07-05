//! CR 309 / 701.49 — dungeons, venture, and the AFR venture cards
//! (`catalog::sets::decks::afr`).

use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;

fn advance_to(g: &mut GameState, step: crate::game::TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

fn venture_ctx(source: crate::card::CardId) -> crate::game::effects::EffectContext {
    let mut ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(source);
    ctx
}

/// Resolve one Venture and route its events through the trigger dispatcher
/// (the real action paths do this; `resolve_effect` alone doesn't).
fn venture(g: &mut GameState, ctx: &crate::game::effects::EffectContext) {
    let events = g.resolve_effect(&crate::effect::Effect::Venture, ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

/// Venturing with no dungeon enters room 1 of the chosen dungeon and
/// resolves its ability (Lost Mine's Cave Entrance = Scry 1).
#[test]
fn cr_701_49_first_venture_enters_the_first_room() {
    let mut g = two_player_game();
    let gargoyle = g.add_card_to_hand(0, catalog::cloister_gargoyle());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: gargoyle, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cloister Gargoyle");
    drain_stack(&mut g);
    let (name, room) = g.players[0].dungeon.clone().expect("in a dungeon");
    assert_eq!(name, "Lost Mine of Phandelver", "auto pick = Lost Mine");
    assert_eq!(room, 0, "in the Cave Entrance");
}

/// Walking Lost Mine to the end completes the dungeon: the final room draws
/// a card, the tally bumps, and the player leaves the dungeon.
#[test]
fn cr_701_49d_completing_lost_mine() {
    let mut g = two_player_game();
    let sword = g.add_card_to_battlefield(0, catalog::shortcut_seeker());
    g.add_card_to_library(0, catalog::grizzly_bears()); // Temple's draw
    let ctx = venture_ctx(sword);
    // Cave Entrance → Goblin Lair (Mode 0) → Storeroom (Mode 0) → Temple.
    for _ in 0..4 {
        venture(&mut g, &ctx);
    }
    assert_eq!(g.players[0].dungeons_completed, 1, "dungeon completed");
    assert!(g.players[0].dungeon.is_none(), "left the dungeon");
    // Goblin Lair minted a 1/1 Goblin along the way.
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Goblin"),
        "Goblin Lair minted its token");
}

/// The venture branch choice is honored (Mine Tunnels mints a Treasure).
#[test]
fn cr_701_49_branch_choice_is_honored() {
    let mut g = two_player_game();
    let sword = g.add_card_to_battlefield(0, catalog::shortcut_seeker());
    let ctx = venture_ctx(sword);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Mode(0), // enter Lost Mine
        DecisionAnswer::Mode(1), // Cave Entrance → Mine Tunnels
    ]));
    venture(&mut g, &ctx);
    venture(&mut g, &ctx);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "Mine Tunnels minted a Treasure");
    assert_eq!(g.players[0].dungeon.as_ref().unwrap().1, 2, "in Mine Tunnels");
}

/// Shortcut Seeker ventures on combat damage to a player.
#[test]
fn shortcut_seeker_ventures_on_combat_damage() {
    let mut g = two_player_game();
    let seeker = g.add_card_to_battlefield(0, catalog::shortcut_seeker());
    g.clear_sickness(seeker);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: seeker, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert!(g.players[0].dungeon.is_some(), "ventured into a dungeon");
}

/// Cloister Gargoyle is 0/4 until a dungeon is completed, then 3/4 flying.
#[test]
fn cloister_gargoyle_pumps_after_a_completed_dungeon() {
    let mut g = two_player_game();
    let garg = g.add_card_to_battlefield(0, catalog::cloister_gargoyle());
    let before = g.computed_permanent(garg).unwrap();
    assert_eq!((before.power, before.toughness), (0, 4));
    g.players[0].dungeons_completed = 1;
    let after = g.computed_permanent(garg).unwrap();
    assert_eq!((after.power, after.toughness), (3, 4), "+3/+0 once completed");
    assert!(after.keywords.contains(&crate::card::Keyword::Flying), "gains flying");
}

/// Dungeon Crawler returns from the graveyard when you complete a dungeon.
#[test]
fn dungeon_crawler_returns_on_dungeon_completion() {
    let mut g = two_player_game();
    let crawler = g.add_card_to_graveyard(0, catalog::dungeon_crawler());
    let sword = g.add_card_to_battlefield(0, catalog::shortcut_seeker());
    g.add_card_to_library(0, catalog::grizzly_bears()); // Temple's draw
    let ctx = venture_ctx(sword);
    // Start one room before the end; the next venture completes the dungeon.
    g.players[0].dungeon = Some(("Lost Mine of Phandelver".into(), 5));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    venture(&mut g, &ctx);
    assert_eq!(g.players[0].dungeons_completed, 1);
    assert!(g.players[0].hand.iter().any(|c| c.id == crawler),
        "Dungeon Crawler returned to hand on completion");
}

/// Tomb of Annihilation's final room mints The Atropal.
#[test]
fn tomb_of_annihilation_mints_the_atropal() {
    let mut g = two_player_game();
    let sword = g.add_card_to_battlefield(0, catalog::shortcut_seeker());
    let ctx = venture_ctx(sword);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Mode(2), // enter Tomb of Annihilation
        DecisionAnswer::Mode(1), // Trapped Entry → Oubliette
    ]));
    // Trapped Entry → Oubliette → Cradle of the Death God.
    for _ in 0..3 {
        venture(&mut g, &ctx);
    }
    assert_eq!(g.players[0].dungeons_completed, 1, "Tomb completed");
    let atropal = g.battlefield.iter().find(|c| c.definition.name == "The Atropal")
        .expect("The Atropal minted");
    assert_eq!((atropal.power(), atropal.toughness()), (4, 4));
    assert!(atropal.definition.keywords.contains(&crate::card::Keyword::Deathtouch));
}

/// CR 509.4 — Flash Foliage mints a Saproling already blocking the targeted
/// attacker (and is castable only after blockers are declared).
#[test]
fn cr_509_4_flash_foliage_blocks_the_attacker() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let foliage = g.add_card_to_hand(1, catalog::flash_foliage());
    g.add_card_to_library(1, catalog::grizzly_bears()); // the rider draw
    g.players[1].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[1].mana_pool.add_colorless(2);
    // Before blockers: the cast is rejected.
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: foliage, target: Some(crate::game::types::Target::Permanent(attacker)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "cast gated until blockers are declared");
    g.priority.player_with_priority = 0;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    g.players[1].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[1].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: foliage, target: Some(crate::game::types::Target::Permanent(attacker)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable after blockers declared");
    drain_stack(&mut g);
    let sap = g.battlefield.iter().find(|c| c.definition.name == "Saproling")
        .expect("Saproling minted");
    assert_eq!(g.block_map.get(&sap.id), Some(&attacker), "token is blocking the attacker");
    assert!(g.blocked_attackers().contains(&attacker), "attacker is blocked");
    // The bear is blocked by a 1/1: combat kills the Saproling, no damage to P1.
    let life = g.players[1].life;
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life, "blocked attacker deals no player damage");
}
