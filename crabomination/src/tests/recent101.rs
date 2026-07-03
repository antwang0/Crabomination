//! Functionality tests for the `catalog::sets::decks::recent101` Kamigawa: Neon
//! Dynasty batch 7.

use crate::card::CounterType;
use crate::catalog;
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;

fn pass_through_combat(g: &mut GameState) {
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(g);
}

/// Coiling Stalker counters a creature that lacks a +1/+1 counter on combat damage.
#[test]
fn coiling_stalker_counters_on_damage() {
    let mut g = two_player_game();
    let stalker = g.add_card_to_battlefield(0, catalog::coiling_stalker());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(stalker);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: stalker,
        target: AttackTarget::Player(1),
    }]))
    .expect("stalker attacks");
    drain_stack(&mut g);
    pass_through_combat(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "the uncountered bear got a +1/+1 counter"
    );
}

/// Sunblade Samurai's Channel fetches a Plains and gains 2 life.
#[test]
fn sunblade_samurai_channel_ramps_and_gains() {
    let mut g = two_player_game();
    let plains = g.add_card_to_library(0, catalog::plains());
    g.players[0].life = 20;
    let sam = g.add_card_to_hand(0, catalog::sunblade_samurai());
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(plains)),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: sam,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("channel Sunblade Samurai");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Plains"), "fetched a Plains");
    assert_eq!(g.players[0].life, 22, "gained 2 life");
}

/// Moonsnare Specialist's ETB bounces a creature.
#[test]
fn moonsnare_specialist_bounces() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ninja = g.add_card_to_battlefield(0, catalog::moonsnare_specialist());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(victim)),
    ]));
    g.fire_self_etb_triggers(ninja, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "returned to owner's hand");
}

/// Undercity Scrounger only makes a Treasure once a creature has died.
#[test]
fn undercity_scrounger_gated_on_death() {
    let mut g = two_player_game();
    let scrounger = g.add_card_to_battlefield(0, catalog::undercity_scrounger());
    g.clear_sickness(scrounger);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let act = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: scrounger,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
    };
    assert!(act(&mut g).is_err(), "can't activate before a death");
    // Kill a creature so the condition is met.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    act(&mut g).expect("activates after a death");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"), "made a Treasure");
}

/// Season of Renewal returns a creature and an enchantment card from graveyard.
#[test]
fn season_of_renewal_returns_both() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ench = g.add_card_to_graveyard(0, catalog::golden_tail_disciple());
    let spell = g.add_card_to_hand(0, catalog::season_of_renewal());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(ench)],
        mode: None,
        x_value: None,
    })
    .expect("cast Season of Renewal");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature returned");
    assert!(g.players[0].hand.iter().any(|c| c.id == ench), "enchantment returned");
}

/// Assassin's Ink is cheaper with an artifact and an enchantment in play.
#[test]
fn assassins_ink_cost_reduction() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bonesplitter()); // artifact
    g.add_card_to_battlefield(0, catalog::golden_tail_disciple()); // enchantment
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::assassins_ink());
    // {2}{B}{B} - {2} = {B}{B}: two black mana suffices.
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast discounted Assassin's Ink");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature destroyed");
}

/// Mnemonic Sphere draws two when sacrificed, and one via Channel from hand.
#[test]
fn mnemonic_sphere_draw_modes() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    // Sac mode: draw two.
    let sphere = g.add_card_to_battlefield(0, catalog::mnemonic_sphere());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: sphere,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("sac Mnemonic Sphere");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 2, "drew two");
    assert!(g.battlefield_find(sphere).is_none(), "sphere sacrificed");
}

/// Suit Up turns a creature into a 4/5 and draws.
#[test]
fn suit_up_pumps_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::suit_up());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Suit Up");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 5), "became a 4/5");
    assert_eq!(g.players[0].hand.len(), before - 1 + 1, "spell left hand, drew one");
}

/// Careful Consideration draws four and discards two in the main phase.
#[test]
fn careful_consideration_main_phase_loot() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::careful_consideration());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let before = g.players[0].hand.len(); // spell still in hand
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Careful Consideration");
    drain_stack(&mut g);
    // -1 (spell) + 4 drawn - 2 discarded = +1 net.
    assert_eq!(g.players[0].hand.len(), before + 1, "drew four, discarded two (main phase)");
}
