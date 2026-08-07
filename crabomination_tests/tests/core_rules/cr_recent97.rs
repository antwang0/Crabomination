//! CR conformance for this run's engine work:
//! - CR 509.1b — evasion abilities are cumulative, and one gained after a
//!   legal block was declared doesn't undo that block.
//! - CR 614.12 — an enters-the-battlefield replacement from a *general*
//!   static doesn't apply to its own source (Orb of Dreams), while a
//!   self-scoped one does.
//! - CR 610.4a — a permanent phased out "until [source] leaves" doesn't
//!   phase in from the untap step's turn-based action.

use crabomination::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, StaticAbility,
    Subtypes,
};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector, StaticEffect};
use crabomination::game::types::{Attack, AttackTarget, GameAction};
use crabomination::game::*;
use crabomination::mana::{cost, generic};

fn body(name: &'static str, keywords: Vec<Keyword>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: p,
        toughness: t,
        keywords,
        ..Default::default()
    }
}

/// "Permanents enter tapped" — the Orb of Dreams shape.
fn orb_of_dreams() -> CardDefinition {
    CardDefinition {
        name: "Orb Test",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Permanents enter tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::EachPermanent(R::Permanent),
            },
        }],
        ..Default::default()
    }
}

/// CR 509.1b — flying and shadow together can only be blocked by a creature
/// with both; each evasion ability is a separate restriction.
#[test]
fn cr_509_1b_evasion_abilities_are_cumulative() {
    for (blocker_kws, legal) in [
        (vec![Keyword::Flying], false),
        (vec![Keyword::Shadow], false),
        (vec![Keyword::Flying, Keyword::Shadow], true),
    ] {
        let mut g = two_player_game();
        let attacker =
            g.add_card_to_battlefield(0, body("Evader", vec![Keyword::Flying, Keyword::Shadow], 2, 2));
        g.clear_sickness(attacker);
        let blocker = g.add_card_to_battlefield(1, body("Blocker", blocker_kws.clone(), 2, 2));
        g.clear_sickness(blocker);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.priority.player_with_priority = 1;
        let ok = g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).is_ok();
        assert_eq!(ok, legal, "{blocker_kws:?}");
    }
}

/// CR 509.1b — an evasion ability gained after blockers are declared doesn't
/// unmake the block.
#[test]
fn cr_509_1b_evasion_gained_after_blocks_doesnt_unblock() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, body("Grounded", vec![], 2, 2));
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, body("Blocker", vec![], 2, 2));
    g.clear_sickness(blocker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");

    let ctx = crabomination::game::effects::EffectContext::for_ability(attacker, 0, None);
    g.resolve_effect(
        &Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Flying,
            duration: crabomination::effect::Duration::EndOfTurn,
        },
        &ctx,
    )
    .expect("grant flying");
    assert!(g.computed_permanent(attacker).unwrap().keywords.contains(&Keyword::Flying));
    assert!(g.blocks(blocker, attacker), "the declared block stands");
}

/// CR 614.12 — a general "permanents enter tapped" static doesn't tap its own
/// source, but a self-scoped one does.
#[test]
fn cr_614_12_general_enters_tapped_spares_its_own_source() {
    let mut g = two_player_game();
    let orb = g.add_card_to_battlefield(0, orb_of_dreams());
    g.fire_self_etb_triggers(orb, 0);
    assert!(!g.battlefield_find(orb).unwrap().tapped, "Orb of Dreams enters untapped");
    let later = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.fire_self_etb_triggers(later, 0);
    assert!(g.battlefield_find(later).unwrap().tapped, "but everything after it is tapped");

    let mut g = two_player_game();
    let selfish = g.add_card_to_battlefield(0, catalog::bad_river());
    g.fire_self_etb_triggers(selfish, 0);
    assert!(g.battlefield_find(selfish).unwrap().tapped, "a `This`-scoped one does apply");
}

/// CR 610.4a — a permanent phased out "until [source] leaves" stays out
/// through its controller's untap step.
#[test]
fn cr_610_4a_linked_phase_out_skips_the_untap_step() {
    let mut g = two_player_game();
    let anchor = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hidden = g.add_card_to_battlefield(0, catalog::serra_angel());
    let ctx = crabomination::game::effects::EffectContext::for_ability(anchor, 0, None);
    g.resolve_effect(
        &Effect::PhaseOut {
            what: Selector::EachPermanent(R::HasKeyword(Keyword::Flying)),
            until_source_leaves: true,
        },
        &ctx,
    )
    .expect("phase out");
    assert!(g.battlefield_find(hidden).is_none());
    g.active_player_idx = 0;
    g.do_phasing();
    assert!(
        g.phased_out.iter().any(|c| c.id == hidden),
        "the untap step's turn-based action doesn't bring it back"
    );
    // An unlinked phase-out does come back on the same path.
    let plain = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.resolve_effect(
        &Effect::PhaseOut {
            what: Selector::EachPermanent(R::HasKeyword(Keyword::Flying)),
            until_source_leaves: false,
        },
        &ctx,
    )
    .expect("phase out");
    g.do_phasing();
    assert!(g.battlefield_find(plain).is_some(), "the unlinked one phases back in");
}
