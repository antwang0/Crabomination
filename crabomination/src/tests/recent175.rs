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
