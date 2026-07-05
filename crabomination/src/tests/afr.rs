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

// ── AFR venture batch ───────────────────────────────────────────────────────

/// Nadaar ventures on ETB and grants other creatures +1/+1 once completed.
#[test]
fn nadaar_ventures_and_anthems_after_completion() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let nadaar = g.add_card_to_hand(0, catalog::nadaar_selfless_paladin());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: nadaar, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Nadaar");
    drain_stack(&mut g);
    assert!(g.players[0].dungeon.is_some(), "ETB ventured");
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!((b.power, b.toughness), (2, 2), "no anthem before completion");
    g.players[0].dungeons_completed = 1;
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!((b.power, b.toughness), (3, 3), "other creatures +1/+1 once completed");
    let n = g.battlefield.iter().find(|c| c.definition.name == "Nadaar, Selfless Paladin").unwrap().id;
    let n = g.computed_permanent(n).unwrap();
    assert_eq!((n.power, n.toughness), (3, 3), "Nadaar itself unbuffed");
}

/// Gloom Stalker gains double strike only once a dungeon is completed.
#[test]
fn gloom_stalker_double_strike_gate() {
    let mut g = two_player_game();
    let gs = g.add_card_to_battlefield(0, catalog::gloom_stalker());
    assert!(!g.computed_permanent(gs).unwrap().keywords.contains(&crate::card::Keyword::DoubleStrike));
    g.players[0].dungeons_completed = 1;
    assert!(g.computed_permanent(gs).unwrap().keywords.contains(&crate::card::Keyword::DoubleStrike));
}

/// Dungeon Map's second ability ventures at sorcery speed for {3}, {T}.
#[test]
fn dungeon_map_ventures() {
    let mut g = two_player_game();
    g.step = crate::game::TurnStep::PreCombatMain;
    let map = g.add_card_to_battlefield(0, catalog::dungeon_map());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: map, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("venture via map");
    drain_stack(&mut g);
    assert!(g.players[0].dungeon.is_some(), "ventured");
}

/// Triumphant Adventurer has first strike only on its controller's turn.
#[test]
fn triumphant_adventurer_first_strike_on_your_turn() {
    let mut g = two_player_game();
    let ta = g.add_card_to_battlefield(0, catalog::triumphant_adventurer());
    assert!(g.computed_permanent(ta).unwrap().keywords.contains(&crate::card::Keyword::FirstStrike),
        "active player 0 owns it - first strike on");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(ta).unwrap().keywords.contains(&crate::card::Keyword::FirstStrike),
        "off-turn - first strike off");
}

/// Yuan-Ti Malison is unblockable while attacking alone.
#[test]
fn yuan_ti_malison_unblockable_alone() {
    let mut g = two_player_game();
    let snake = g.add_card_to_battlefield(0, catalog::yuan_ti_malison());
    let wall = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(snake);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: snake, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(!g.blocker_can_block_attacker(wall, snake), "unblockable while alone");
}

/// Precipitous Drop gives -2/-2, deepening to -5/-5 once a dungeon is done.
#[test]
fn precipitous_drop_scales_with_completion() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::wurmcoil_engine()); // 6/6
    let drop = g.add_card_to_hand(0, catalog::precipitous_drop());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: drop, target: Some(crate::game::types::Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast the aura");
    drain_stack(&mut g);
    assert!(g.players[0].dungeon.is_some(), "aura ETB ventured");
    let c = g.computed_permanent(big).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "-2/-2 before completion");
    g.players[0].dungeons_completed = 1;
    let c = g.computed_permanent(big).unwrap();
    assert_eq!((c.power, c.toughness), (1, 1), "-5/-5 once completed");
}

/// Bar the Gate counters a creature spell and ventures; rejects noncreature.
#[test]
fn bar_the_gate_counters_creature_spells() {
    let mut g = two_player_game();
    g.step = crate::game::TurnStep::PreCombatMain;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let gate = g.add_card_to_hand(1, catalog::bar_the_gate());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    g.players[1].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: gate, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("counter the bear");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear countered");
    assert!(g.players[1].dungeon.is_some(), "and ventured");
}

/// Radiant Solar ventures when itself or another nontoken creature enters.
#[test]
fn radiant_solar_ventures_on_nontoken_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::radiant_solar());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    drain_stack(&mut g);
    assert!(g.players[0].dungeon.is_some(), "ventured off the bear's ETB");
}

/// Fates' Reversal returns a graveyard creature and ventures.
#[test]
fn fates_reversal_returns_and_ventures() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::fates_reversal());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(crate::game::types::Target::Permanent(dead)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "creature back to hand");
    assert!(g.players[0].dungeon.is_some(), "ventured");
}

/// Delver's Torch ventures when the equipped creature attacks.
#[test]
fn delvers_torch_ventures_on_equipped_attack() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let torch = g.add_card_to_battlefield(0, catalog::delvers_torch());
    g.clear_sickness(bear);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: torch, target: bear })
        .expect("equip");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.players[0].dungeon.is_some(), "ventured off the equipped attack");
}

/// Ranger's Hawk taps another creature as its venture activation cost.
#[test]
fn rangers_hawk_taps_helper_to_venture() {
    let mut g = two_player_game();
    g.step = crate::game::TurnStep::PreCombatMain;
    let hawk = g.add_card_to_battlefield(0, catalog::rangers_hawk());
    let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(hawk);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hawk, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("venture");
    drain_stack(&mut g);
    assert!(g.players[0].dungeon.is_some(), "ventured");
    assert!(g.battlefield_find(helper).unwrap().tapped, "helper tapped as cost");
}

/// Shessra's end-step draw asks for 2 life only after a creature died.
#[test]
fn shessra_pays_two_life_to_draw_after_a_death() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::shessra_deaths_whisper());
    g.add_card_to_library(0, catalog::grizzly_bears());
    // No death: the trigger's intervening-if fails, no draw, no life paid.
    let life = g.players[0].life;
    g.fire_step_triggers(crate::TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "no death, no prompt");
    // A creature dies, then the end step fires the pay-2-draw.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&crate::effect::Effect::Destroy {
        what: crate::effect::Selector::EachPermanent(
            crate::card::SelectionRequirement::HasCreatureType(crate::card::CreatureType::Bear),
        ),
    }, &ctx).unwrap();
    let _ = bear;
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(crate::TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "paid 2 life");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Ellywick +1 ventures; the -7 emblem anthems once a dungeon is completed.
#[test]
fn ellywick_ventures_and_emblem_anthems() {
    let mut g = two_player_game();
    g.step = crate::game::TurnStep::PreCombatMain;
    let elly = g.add_card_to_battlefield(0, catalog::ellywick_tumblestrum());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: elly, ability_index: 0, target: None, x_value: None,
    }).expect("+1");
    drain_stack(&mut g);
    assert!(g.players[0].dungeon.is_some(), "+1 ventured");
    // Force the ultimate: top up loyalty and reset the per-turn use.
    g.battlefield_find_mut(elly).unwrap().add_counters(crate::card::CounterType::Loyalty, 8);
    g.battlefield_find_mut(elly).unwrap().loyalty_uses_this_turn = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: elly, ability_index: 2, target: None, x_value: None,
    }).expect("-7");
    drain_stack(&mut g);
    assert_eq!(g.players[0].emblems.len(), 1, "emblem minted");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].dungeons_completed = 1;
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 4), "+2/+2 from the emblem");
    assert!(b.keywords.contains(&crate::card::Keyword::Trample), "trample granted");
}
