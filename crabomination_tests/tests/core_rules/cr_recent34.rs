//! CR conformance for the modern_decks RNA batch-10 engine work:
//! - CR 118.8 — paying life is a cost event distinct from losing life; Font of
//!   Agonies banks blood counters when you *pay* life, but combat/noncombat
//!   damage (life loss) does not trigger it.
//! - CR 614.16 — a counter-doubling replacement (Doubling Season) applies to the
//!   +1/+1 counters Galloping Lizrog places, so removing 3 places 12 (2× then ×2).
//! - CR 508.1g — an "until your next turn" Propaganda tax (Forbidding Spirit)
//!   sums with a static tax and expires at the taxed player's untap step.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

/// CR 118.8 — Font of Agonies triggers on paying life, not on losing it.
#[test]
fn cr_118_8_pay_life_not_loss_banks_blood() {
    let mut g = two_player_game();
    let font = g.add_card_to_battlefield(0, catalog::font_of_agonies());
    // Losing life to damage is not "paying life": no blood counters.
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 3, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(font).unwrap().counter_count(CounterType::Blood), 0, "damage is not paying life");
    // Paying life as a cost banks that many.
    g.dispatch_triggers_for_events(&[GameEvent::PaidLife { player: 0, amount: 2 }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(font).unwrap().counter_count(CounterType::Blood), 2, "paying 2 life banks 2");
}

/// CR 614.16 — Doubling Season doubles the counters Galloping Lizrog places.
#[test]
fn cr_614_16_doubling_season_doubles_lizrog() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::doubling_season());
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(src).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    let lizrog = g.move_card_to_battlefield_for_test(0, catalog::galloping_lizrog());
    drain_stack(&mut g);
    // 3 removed -> 6 placed -> doubled to 12.
    assert_eq!(g.battlefield_find(lizrog).unwrap().counter_count(CounterType::PlusOnePlusOne), 12, "2x then Doubling Season");
}

/// CR 508.1g — Forbidding Spirit's tax stacks with Propaganda and clears at the
/// taxed player's next untap.
#[test]
fn cr_508_1g_temporary_tax_stacks_and_expires() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::propaganda());
    g.move_card_to_battlefield_for_test(1, catalog::forbidding_spirit());
    drain_stack(&mut g);
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    // Propaganda {2} + Forbidding Spirit {2} = {4}. {2} alone is not enough.
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }])).is_err(), "{{2}} short of the {{4}} tax");
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }])).expect("pay {4}");
    assert_eq!(g.attacking().len(), 1);
    // The temporary half expires at the taxed player's untap.
    g.active_player_idx = 1;
    g.do_untap();
    assert_eq!(g.players[1].attack_tax_until_your_turn, 0, "temporary tax expired");
}
