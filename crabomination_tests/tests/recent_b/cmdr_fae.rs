//! Commander: the Fae Dominion precon (WOC, Tegwyll, `decks::cmdr_fae`) and
//! the primitives it needed.

use crabomination::card::{AdditionalCastCost, CardDefinition, CardType, Keyword, SelectionRequirement as R};
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

/// CR 601.2b — "you may cast this spell as though it had flash by tapping
/// three untapped creatures you control with flying": off-turn it needs the
/// three fliers and taps them; in your main phase it costs nothing extra.
#[test]
fn cr_601_2b_flash_by_tapping_fliers() {
    let spell = || CardDefinition {
        name: "Test Scouring",
        card_types: vec![CardType::Sorcery],
        flash_additional_cost: Some(AdditionalCastCost::TapPermanents {
            filter: R::Creature.and(R::HasKeyword(Keyword::Flying)),
            count: 3,
        }),
        effect: Effect::Noop,
        ..Default::default()
    };
    let cast = |g: &mut GameState, id| {
        g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
    };
    let mut g = pod(2);
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    let fliers: Vec<_> = (0..2).map(|_| g.add_card_to_battlefield(0, catalog::serra_angel())).collect();
    let s = g.add_card_to_hand(0, spell());
    assert!(cast(&mut g, s).is_err(), "two fliers can't pay");
    let third = g.add_card_to_battlefield(0, catalog::serra_angel());
    cast(&mut g, s).expect("three fliers pay");
    for id in fliers.iter().chain([&third]) {
        assert!(g.battlefield_find(*id).unwrap().tapped);
    }

    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    let s = g.add_card_to_hand(0, spell());
    cast(&mut g, s).expect("sorcery timing needs nothing extra");
    assert!(!g.battlefield_find(angel).unwrap().tapped);
}
