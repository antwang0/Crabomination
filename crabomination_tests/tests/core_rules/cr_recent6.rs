//! CR conformance for combat keyword rules exercised by the recent296–298
//! Ravnica batches: CR 702.2 (Deathtouch — including *granted* deathtouch via
//! Corpse Blockade), CR 702.4 (Double Strike — combat damage in both steps),
//! and CR 702.111 / 509.1c (Menace — can't be blocked by exactly one creature).

use crabomination::card::{CardDefinition, CardType, Keyword, Subtypes};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameState, TurnStep};

fn body_kw(name: &'static str, p: i32, t: i32, keywords: Vec<Keyword>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        power: p,
        toughness: t,
        keywords,
        subtypes: Subtypes::default(),
        ..Default::default()
    }
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// CR 702.2b — any nonzero combat damage from a source with deathtouch is
/// lethal. Corpse Blockade *grants itself* deathtouch (1 power), so its single
/// point of combat damage destroys a blocked 4/4.
#[test]
fn cr_702_2b_granted_deathtouch_is_lethal() {
    use crabomination::effect::{Duration, Effect, Selector};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    // Player 1 attacks with a 4/4; player 0 blocks with Corpse Blockade.
    g.active_player_idx = 1;
    let big = g.add_card_to_battlefield(1, body_kw("Bruiser", 3, 4, vec![])); // ground 3/4
    g.clear_sickness(big);
    let blockade = g.add_card_to_battlefield(0, catalog::corpse_blockade()); // 1/4 Defender
    // Grant deathtouch directly (Corpse Blockade's own sac ability does this).
    let ctx = EffectContext::for_ability(blockade, 0, None);
    g.resolve_effect(
        &Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Deathtouch,
            duration: Duration::EndOfTurn,
        },
        &ctx,
    ).unwrap();
    assert!(g.computed_permanent(blockade).unwrap().keywords.contains(&Keyword::Deathtouch));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: big, target: AttackTarget::Player(0),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blockade, big)])).expect("block");
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert!(g.battlefield_find(big).is_none(), "1 deathtouch damage killed the 3/4");
    assert!(g.battlefield_find(blockade).is_some(), "the 1/4 Blockade survived 3 damage");
}

/// CR 702.4b — a creature with double strike assigns combat damage in both the
/// first-strike step and the regular step. An unblocked 3/3 double striker
/// deals 6 to the defending player.
#[test]
fn cr_702_4b_double_strike_hits_twice() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body_kw("Twinblade", 3, 3, vec![Keyword::DoubleStrike]));
    g.clear_sickness(atk);
    let start = g.players[1].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, start - 6, "double strike dealt 3 twice");
}

/// CR 702.111b / 509.1c — a creature with menace can't be blocked except by two
/// or more creatures. A lone blocker assignment is illegal.
#[test]
fn cr_702_111b_menace_rejects_a_lone_blocker() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body_kw("Marauder", 2, 2, vec![Keyword::Menace]));
    g.clear_sickness(atk);
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    // One blocker is rejected...
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(b1, atk)])).is_err(),
        "menace can't be blocked by exactly one creature");
    // ...two is legal.
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, atk), (b2, atk)])).expect("double block");
}
