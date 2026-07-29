//! CR conformance for the modern_decks JOU run:
//! - CR 509.1d–f — a "can't block unless its controller pays {N}" cost, and
//!   its CR 508.1g attack-side sibling.
//! - CR 607.2a / 607.2d — linked abilities: each source's "the exiled cards"
//!   and "the chosen color" read only its own linked data.
//! - CR 616.1 — two applicable prevention effects both apply, in sequence.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// Attach `aura` (cast by seat 0) to `host`.
fn enchant(g: &mut GameState, aura: crabomination::card::CardDefinition, host: CardId) {
    let cost = aura.cost.cmc();
    let id = g.add_card_to_hand(0, aura);
    g.players[0].mana_pool.add(Color::White, cost);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(host)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("enchant");
    drain_stack(g);
}

/// CR 508.1g — an attacker under a pay-to-attack restriction can't be
/// declared without the mana, and the declaration consumes it.
#[test]
fn cr_508_1g_pay_to_attack_consumes_the_mana() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    enchant(&mut g, catalog::oppressive_rays(), bear);
    assert!(g
        .computed_permanent(bear)
        .unwrap()
        .keywords
        .contains(&Keyword::CantAttackOrBlockUnlessPay(3)));

    g.step = TurnStep::DeclareAttackers;
    let swing = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
    assert!(g.declare_attackers(swing.clone()).is_err(), "no mana, no attack");
    g.players[0].mana_pool.add_colorless(3);
    g.declare_attackers(swing).expect("paid the {3}");
    assert_eq!(g.players[0].mana_pool.total(), 0, "the {{3}} was spent");
}

/// CR 509.1d–f — the same restriction gates blocking, charged to the
/// blocker's controller.
#[test]
fn cr_509_1d_pay_to_block_is_charged_to_the_blocker() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::oreskos_swiftclaw());
    g.clear_sickness(attacker);
    enchant(&mut g, catalog::oppressive_rays(), blocker);

    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("swing");
    g.step = TurnStep::DeclareBlockers;
    assert!(g.declare_blockers(vec![(blocker, attacker)]).is_err(), "unpaid");
    g.players[1].mana_pool.add_colorless(3);
    g.declare_blockers(vec![(blocker, attacker)]).expect("paid the {3}");
    assert_eq!(g.players[1].mana_pool.total(), 0);
}

/// CR 607.2a — "the exiled cards" is linked to the ability that exiled them,
/// so a second Banisher Priest leaving play returns only its own prisoner.
#[test]
fn cr_607_2a_linked_exile_returns_only_its_own_card() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::oreskos_swiftclaw());

    let p1 = g.add_card_to_battlefield(0, catalog::banisher_priest());
    g.fire_self_etb_triggers(p1, 0);
    drain_stack(&mut g);
    let p2 = g.add_card_to_battlefield(0, catalog::banisher_priest());
    g.fire_self_etb_triggers(p2, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());

    let mut evs = Vec::new();
    g.destroy_permanent(p1, false, &mut evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_some(), "the first Priest's prisoner came back");
    assert!(g.battlefield_find(b).is_none(), "the second Priest still holds its own");
}

/// CR 607.2d — "the chosen color" is linked to the permanent that chose it,
/// so two Halls anthem two different colors for their own controllers.
#[test]
fn cr_607_2d_each_source_reads_its_own_chosen_color() {
    let mut g = main_phase();
    for (seat, color) in [(0usize, Color::Green), (1usize, Color::White)] {
        let hall = g.add_card_to_battlefield(seat, catalog::hall_of_triumph());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(color)]));
        g.fire_self_etb_triggers(hall, seat);
        drain_stack(&mut g);
    }
    let mine_green = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 green
    let mine_white = g.add_card_to_battlefield(0, catalog::oreskos_swiftclaw()); // 3/1 white
    let theirs_white = g.add_card_to_battlefield(1, catalog::oreskos_swiftclaw());
    assert_eq!(
        g.computed_permanent(mine_green).map(|c| (c.power, c.toughness)),
        Some((3, 3)),
        "my Hall chose green"
    );
    assert_eq!(
        g.computed_permanent(mine_white).map(|c| (c.power, c.toughness)),
        Some((3, 1)),
        "the opponent's white Hall doesn't reach my board"
    );
    assert_eq!(
        g.computed_permanent(theirs_white).map(|c| (c.power, c.toughness)),
        Some((4, 2)),
        "their Hall chose white"
    );
}

/// CR 616.1 — a source-restricted prevention effect only applies to that
/// source's damage; unrelated damage lands in full.
#[test]
fn cr_616_1_source_scoped_shield_ignores_other_sources() {
    let mut g = main_phase();
    let fort = g.add_card_to_battlefield(0, catalog::stonewise_fortifier()); // 2/2
    let shielded = g.add_card_to_battlefield(1, catalog::hydra_broodmaster());
    let other = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fort,
        ability_index: 0,
        target: Some(Target::Permanent(shielded)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("shield against the Hydra");
    drain_stack(&mut g);

    let mut evs = Vec::new();
    assert_eq!(
        g.apply_prevention_shields(
            crabomination::game::effects::EntityRef::Permanent(fort),
            5,
            Some(shielded),
            &mut evs,
        ),
        0,
        "the Hydra's damage is fully prevented"
    );
    assert_eq!(
        g.apply_prevention_shields(
            crabomination::game::effects::EntityRef::Permanent(fort),
            5,
            Some(other),
            &mut evs,
        ),
        5,
        "an unrelated source is untouched"
    );
}

/// CR 616.1f — the shield pass repeats until no applicable effect is left, so
/// two stacked "prevent the next N damage" shields soak N+M in one event.
#[test]
fn cr_616_1f_stacked_shields_both_apply_to_one_event() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        bear,
        0,
        Some(Target::Permanent(bear)),
    );
    for n in [2, 3] {
        g.resolve_effect(
            &crabomination::effect::Effect::PreventNextDamage {
                target: crabomination::effect::Selector::Target(0),
                amount: crabomination::card::Value::Const(n),
            },
            &ctx,
        )
        .expect("shield up");
    }
    let mut evs = Vec::new();
    assert_eq!(
        g.apply_prevention_shields(
            crabomination::game::effects::EntityRef::Permanent(bear),
            6,
            None,
            &mut evs,
        ),
        1,
        "2 + 3 prevented, 1 through"
    );
    assert!(g.prevention_shields.is_empty(), "both shields were spent");
}
