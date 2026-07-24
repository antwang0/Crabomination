//! CR conformance for the modern_decks RNA batch-9 engine work:
//! - CR 601.2f — the total cost of a spell includes cost increases; a
//!   "spells that target this cost {2} more" static (Sphinx of New Prahv)
//!   makes an otherwise-affordable removal spell illegal until paid.
//! - CR 702.2c — any nonzero amount of deathtouch damage is lethal; a static
//!   granting deathtouch to instant/sorcery spells (Pestilent Spirit) routes
//!   the resolving spell's damage through the deathtouch destroy SBA.
//! - CR 508.1a / 509.1a — a creature with defender can't be declared as an
//!   attacker unless something lets it; Scuttlegator's counter-gated
//!   defender-bypass only applies while it carries a +1/+1 counter.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// CR 601.2f — Sphinx of New Prahv adds {2} to opponents' spells that target
/// it, so a bare {R} Lightning Bolt can't be cast at it.
#[test]
fn cr_601_2f_target_tax_makes_spell_unaffordable() {
    let mut g = two_player_game();
    let sphinx = g.add_card_to_battlefield(0, catalog::sphinx_of_new_prahv());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(sphinx)), additional_targets: vec![], mode: None, x_value: None }).is_err(),
        "the {{2}} target tax is part of the total cost"
    );
}

/// CR 702.2c — Pestilent Spirit gives your instant/sorcery spells deathtouch,
/// so a 3-damage Lightning Bolt destroys a 6/4 (nonzero deathtouch is lethal).
#[test]
fn cr_702_2c_spell_deathtouch_is_lethal() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pestilent_spirit());
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(wurm)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(wurm).is_none(), "3 deathtouch damage destroyed the 6/4");
}

/// CR 508.1a / 509.1a — Scuttlegator (defender) can't be declared as an
/// attacker with no counter, but may once it carries a +1/+1 counter.
#[test]
fn cr_508_1a_defender_bypass_gated_on_counter() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::scuttlegator());
    g.clear_sickness(s);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.declare_attackers(vec![Attack { attacker: s, target: AttackTarget::Player(1) }]).is_err(),
        "defender can't attack without a counter"
    );
    g.battlefield_find_mut(s).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(
        g.declare_attackers(vec![Attack { attacker: s, target: AttackTarget::Player(1) }]).is_ok(),
        "a +1/+1 counter enables the defender-bypass"
    );
    // Sanity: the keyword is still Defender (the bypass is a static, not a loss).
    assert!(g.computed_permanent(s).unwrap().keywords.contains(&Keyword::Defender));
}
