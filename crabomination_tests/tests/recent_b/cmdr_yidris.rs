//! Commander: the Entropic Uprising precon (C16, Yidris,
//! `decks::cmdr_yidris`) and the primitives it needed.

use crabomination::card::{SelectionRequirement as R, CardDefinition, CardType, CounterType, EventKind, EventScope, EventSpec, TriggeredAbility, Value};
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::types::TurnStep;
use crabomination::game::*;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// CR 800.4a — "whenever a player loses the game": each other player's
/// permanent triggers once per departing player.
#[test]
fn cr_800_4a_a_permanent_sees_each_player_leave() {
    let watcher = || CardDefinition {
        name: "Test Watcher",
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerLeftGame, EventScope::SelfSource),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(5) },
        }],
        ..Default::default()
    };
    let mut g = pod(4);
    let mine = g.add_card_to_battlefield(0, watcher());
    let theirs = g.add_card_to_battlefield(3, watcher());
    g.players[1].life = 0;
    g.players[2].life = 0;
    g.check_state_based_actions();
    drain_stack(&mut g);
    for id in [mine, theirs] {
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 10, "two players left");
    }
}

/// CR 723.1 — each of two players controls the other's next turn; the swap
/// takes effect on the turn itself, and the caster controls nobody.
#[test]
fn cr_723_1_two_players_control_each_others_next_turn() {
    let mut g = pod(3);
    let mut ctx = EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
    ctx.targets = vec![Target::Player(1), Target::Player(2)];
    g.resolve_effect(
        &Effect::PlayersControlEachOthersNextTurn { first: PlayerRef::Target(0), second: PlayerRef::Target(1) },
        &ctx,
    )
    .expect("resolve");
    g.apply_pending_player_control(1);
    assert_eq!(g.controlled_by.get(1).copied().flatten(), Some(2), "seat 2 runs seat 1's turn");
    g.apply_pending_player_control(2);
    assert_eq!(g.controlled_by.get(2).copied().flatten(), Some(1), "seat 1 runs seat 2's turn");
    g.apply_pending_player_control(0);
    assert!(g.controlled_by.is_empty(), "the caster's own turn is theirs");
}

/// CR 701.34 — every creature card in the target's graveyard is manifested
/// under the caster: face-down 2/2s; the non-creature card stays.
#[test]
fn cr_701_34_manifest_a_graveyard_under_your_control() {
    let mut g = pod(2);
    g.add_card_to_graveyard(1, catalog::serra_angel());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let mut ctx = EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&Effect::ManifestFromGraveyard { who: PlayerRef::Target(0), filter: R::Creature }, &ctx)
        .expect("resolve");
    let manifested: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).map(|c| c.id).collect();
    assert_eq!(manifested.len(), 2);
    for id in manifested {
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2));
        assert_eq!(g.battlefield_find(id).unwrap().owner, 1, "still the opponent's card");
    }
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}
