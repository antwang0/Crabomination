//! Functionality tests for `catalog::sets::decks::recent175`.

use crate::card::CounterType;
use crate::catalog;
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Outpace Oblivion's ETB deals 5 to a creature (kills a Grizzly Bears).
#[test]
fn outpace_oblivion_etb_burns_a_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ench = g.add_card_to_battlefield(0, catalog::outpace_oblivion());
    g.fire_self_etb_triggers(ench, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "5 damage killed the 2/2");
}

/// The sacrifice ability deals 2 to each player who isn't at max speed only.
#[test]
fn outpace_oblivion_sac_spares_max_speed_players() {
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(0, catalog::outpace_oblivion());
    g.players[0].speed = 4; // max — spared
    g.players[1].speed = 2; // below max — takes 2
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ench, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("sac ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0, "max-speed controller was spared");
    assert_eq!(g.players[1].life, l1 - 2, "below-max opponent took 2");
}

/// Sabotage Strategist debuffs a creature that attacks its controller.
#[test]
fn sabotage_strategist_debuffs_attacker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::sabotage_strategist()); // defender's
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(attacker);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }]))
    .expect("attack the Strategist's controller");
    drain_stack(&mut g);
    // 2/2 becomes 1/2 until end of turn.
    let p = g.computed_permanent(attacker).unwrap();
    assert_eq!((p.power, p.toughness), (1, 2), "attacker got -1/-0");
}

/// Magmakin Artillerist burns each opponent for the number of cards discarded
/// in a single resolution (batched CR 701.9 discard event).
#[test]
fn magmakin_artillerist_burns_on_batched_discard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::magmakin_artillerist());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let opp = g.players[1].life;
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(
            &crate::effect::Effect::Discard {
                who: crate::effect::Selector::You,
                amount: crate::effect::Value::Const(2),
                random: false,
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "two cards discarded → 2 damage to the opponent");
}

/// The exhaust ability adds three +1/+1 counters.
#[test]
fn sabotage_strategist_exhaust_grows() {
    let mut g = two_player_game();
    let strat = g.add_card_to_battlefield(0, catalog::sabotage_strategist());
    g.clear_sickness(strat);
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: strat, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("exhaust");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(strat).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Waxen Shapethief enters as a copy of an artifact/creature you control.
#[test]
fn waxen_shapethief_copies_your_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // a 2/2 to copy
    let thief = g.add_card_to_hand(0, catalog::waxen_shapethief());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: thief, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Waxen Shapethief");
    drain_stack(&mut g);
    // enters_as_copy resolves on entry; recompute the board.
    let cp = g.computed_permanent(thief).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "copied the Grizzly Bears");
}

/// Quag Feast mills two then destroys the target only if its MV fits the
/// now-larger graveyard.
#[test]
fn quag_feast_destroys_when_graveyard_is_big_enough() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    // Seed one card so mill-2 pushes the graveyard to 3 (≥ 2).
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::quag_feast());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(spell, Some(Target::Permanent(foe)), vec![], None, None).expect("cast Quag Feast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "MV 2 ≤ 2 milled cards → destroyed");
}

/// Plow Through mode 1 destroys a Vehicle.
#[test]
fn plow_through_destroys_a_vehicle() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(1, catalog::skybox_ferry()); // a Vehicle
    let spell = g.add_card_to_hand(0, catalog::plow_through());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Mode 1 = destroy target Vehicle (slot 0).
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(veh)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("cast Plow Through, destroy mode");
    drain_stack(&mut g);
    assert!(g.battlefield_find(veh).is_none(), "Vehicle destroyed");
}

/// Explosive Getaway blinks a target and deals 4 to each creature.
#[test]
fn explosive_getaway_blinks_and_wipes() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let saved = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2, exiled → spared
    let doomed = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, eats 4
    let spell = g.add_card_to_hand(0, catalog::explosive_getaway());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(spell, Some(Target::Permanent(saved)), vec![], None, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(doomed).is_none(), "unexiled creature took 4 and died");
    assert!(g.exile.iter().any(|c| c.id == saved) || g.battlefield_find(saved).is_some(),
        "the blinked creature was spared (in exile or already returned)");
}

/// Lightwheel Enhancements pumps and grants vigilance, and seeds speed.
#[test]
fn lightwheel_enhancements_pumps_and_seeds_speed() {
    use crate::card::Keyword;
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::lightwheel_enhancements());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(aura, Some(Target::Permanent(bear)), vec![], None, None).expect("cast Aura");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "granted vigilance");
    assert_eq!(g.players[0].speed, 1, "Start your engines! seeded speed 1");
}

/// Thopter Fabricator mints a Thopter on your second draw each turn (once).
#[test]
fn thopter_fabricator_mints_on_second_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::thopter_fabricator());
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    g.players[0].cards_drawn_this_turn = 0;
    let mut ev = vec![];
    g.draw_one(0, &mut ev); // first draw — no token
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count(), 0);
    let mut ev2 = vec![];
    g.draw_one(0, &mut ev2); // second draw — one Thopter
    g.dispatch_triggers_for_events(&ev2);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count(), 1,
        "second draw mints a Thopter");
}

/// Coalstoke Gearhulk reanimates a small creature with a finality counter and
/// grants it haste/menace/deathtouch; it's exiled at the next end step.
#[test]
fn coalstoke_gearhulk_reanimates_then_exiles() {
    use crate::card::{CounterType, Keyword};
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears()); // MV 2, opponent's gy
    let hulk = g.add_card_to_battlefield(0, catalog::coalstoke_gearhulk());
    g.fire_self_etb_triggers(hulk, 0);
    drain_stack(&mut g);
    let cp = g.computed_permanent(dead).expect("reanimated onto the battlefield");
    assert_eq!(cp.controller, 0, "under my control");
    assert!(cp.keywords.contains(&Keyword::Haste), "gained haste");
    assert_eq!(g.battlefield_find(dead).unwrap().counter_count(CounterType::Finality), 1);
    // At my next end step it's exiled.
    g.step = TurnStep::End;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_none(), "exiled at the next end step");
    assert!(g.exile.iter().any(|c| c.id == dead), "moved to exile");
}

/// March of the World Ooze makes your team base 6/6 Oozes and mints an Elephant
/// when an opponent casts on your turn.
#[test]
fn march_of_the_world_ooze_anthem_and_token() {
    use crate::card::CreatureType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::march_of_the_world_ooze());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "base 6/6");
    assert!(cp.subtypes.creature_types.contains(&CreatureType::Ooze), "now an Ooze");
    // Opponent casts on your turn → Elephant.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
    g.active_player_idx = 0; // your turn
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.cast_spell(bolt, Some(Target::Player(0)), vec![], None, None).expect("opponent casts on your turn");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Elephant" && c.controller == 0).count(), 1,
        "minted a 3/3 Elephant");
}

/// Possession Engine steals a creature while it stays on the battlefield.
#[test]
fn possession_engine_steals_a_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let engine = g.add_card_to_battlefield(0, catalog::possession_engine());
    g.fire_self_etb_triggers(engine, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 0, "now under my control");
    // Sacrifice the engine → control reverts.
    g.remove_to_graveyard_with_triggers(engine);
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 1, "control reverts when the Vehicle leaves");
}

/// Oildeep Gearhulk makes an opponent discard a chosen card, then draw.
#[test]
fn oildeep_gearhulk_coercive_discard_then_draw() {
    let mut g = two_player_game();
    let hulk = g.add_card_to_battlefield(0, catalog::oildeep_gearhulk());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_library(1, catalog::forest());
    let hand_before = g.players[1].hand.len();
    g.fire_self_etb_triggers(hulk, 0);
    drain_stack(&mut g);
    // Discard one, draw one → net hand size unchanged, but a card left the hand
    // to the graveyard and a fresh card was drawn.
    assert_eq!(g.players[1].hand.len(), hand_before, "discarded one, drew one");
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "the chosen card was discarded");
}

/// Repurposing Bay sacrifices a MV-1 artifact to fetch a MV-2 artifact.
#[test]
fn repurposing_bay_fetches_mana_value_plus_one() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bay = g.add_card_to_battlefield(0, catalog::repurposing_bay());
    let fodder = g.add_card_to_battlefield(0, catalog::springleaf_drum()); // MV 1 artifact
    let orni = g.add_card_to_library(0, catalog::ornithopter_of_paradise()); // MV 2 — fetchable
    g.clear_sickness(bay);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(orni))]));
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bay, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).expect("sac + fetch");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == orni && c.controller == 0),
        "fetched the MV+1 artifact onto the battlefield");
    assert!(g.battlefield_find(fodder).is_none(), "the MV-1 fodder artifact was sacrificed");
}
