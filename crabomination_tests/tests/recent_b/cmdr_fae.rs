//! Commander: the Fae Dominion precon (WOC, Tegwyll, `decks::cmdr_fae`) and
//! the primitives it needed.

use crabomination::card::SelectionRequirement as R;
use crabomination::catalog;
use crabomination::effect::{Duration, Effect, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn run(g: &mut GameState, seat: usize, e: &Effect) {
    let ctx = EffectContext::for_spell(seat, None, 0, 0);
    g.resolve_effect(e, &ctx).expect("resolve");
    drain_stack(g);
}

/// CR 701.38 — goad lasts until the goader's next turn; "goaded for the rest
/// of the game" survives it, while a plain goad beside it lapses.
#[test]
fn cr_701_38_a_goad_for_the_game_outlives_the_goaders_turn() {
    let mut g = pod(3);
    let forever = g.add_card_to_battlefield(1, catalog::serra_angel());
    let plain = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    run(&mut g, 0, &Effect::GoadForTheGame { what: Selector::EachPermanent(R::HasName("Serra Angel".into())) });
    run(&mut g, 0, &Effect::Goad { what: Selector::EachPermanent(R::HasName("Grizzly Bears".into())) });
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(forever).unwrap().goaded_by.contains(&0), "still goaded");
    assert!(g.battlefield_find(plain).unwrap().goaded_by.is_empty(), "a plain goad lapsed");
}

/// CR 508.1a — "they can't attack you or planeswalkers you control": the
/// named seat is refused as a defender, another opponent isn't.
#[test]
fn cr_508_1a_a_creature_that_cant_attack_you_attacks_someone_else() {
    let mut g = pod(3);
    g.active_player_idx = 1;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    run(
        &mut g,
        0,
        &Effect::GrantCantAttackYou {
            what: Selector::EachPermanent(R::HasName("Grizzly Bears".into())),
            duration: Duration::EndOfTurn,
        },
    );
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let at = |p| GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(p) }]);
    assert!(g.perform_action(at(0)).is_err(), "can't attack the granting player");
    g.perform_action(at(2)).expect("another opponent is fine");
}
