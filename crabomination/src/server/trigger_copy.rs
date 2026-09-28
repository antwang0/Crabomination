//! Copying one of your own triggered abilities — Strionic Resonator's "{2},
//! {T}: copy target triggered ability you control" — was activated by no bot
//! path: a 6-seat `--card-census` over every deck (seed 3100001) never used it
//! in four decks, because the only window it answers is the bot's own trigger
//! on top of the stack. A pod bot now copies its own value trigger there (a
//! draw, a token, a tutor, counters, life, or damage to opponents). Commander
//! games only, so two-player play is unchanged.

use crate::effect::{Effect, PlayerRef, Selector};
use crate::game::GameState;
use crate::game::types::{GameAction, StackItem, Target};

/// Would a second copy of `e` help its controller? Untargeted value only, so a
/// copy can't be aimed back at the bot's own board.
fn worth_copying(e: &Effect) -> bool {
    match e {
        Effect::Draw { who: Selector::You, .. }
        | Effect::CreateToken { who: PlayerRef::You, .. }
        | Effect::Search { who: PlayerRef::You, .. }
        | Effect::GainLife { who: Selector::You, .. }
        | Effect::AddCounter { what: Selector::This | Selector::EachPermanent(_), .. } => true,
        Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), .. } => true,
        Effect::Seq(v) => v.iter().any(worth_copying),
        Effect::MayDo { body, .. } => worth_copying(body),
        _ => false,
    }
}

/// Copy the bot's own worthwhile trigger on top of the stack with a
/// `CopyAbility` activation, if one is accepted.
pub(super) fn pick_trigger_copy(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() {
        return None;
    }
    let Some(StackItem::Trigger { source, controller, effect, target: None, .. }) = state.stack.last() else {
        return None;
    };
    if *controller != seat || !worth_copying(effect) {
        return None;
    }
    state.battlefield.iter().filter(|c| c.controller == seat).find_map(|c| {
        c.definition.activated_abilities.iter().enumerate().find_map(|(i, ab)| {
            if !matches!(ab.effect, Effect::CopyAbility { .. }) {
                return None;
            }
            let action = GameAction::ActivateAbility {
                card_id: c.id,
                ability_index: i,
                target: Some(Target::Permanent(*source)),
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            state.would_accept(action.clone()).then_some(action)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// Strionic Resonator copies its controller's ETB draw trigger in a pod,
    /// and not without a commander.
    #[test]
    fn strionic_resonator_copies_an_own_draw_trigger() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        for _ in 0..5 {
            g.add_card_to_library(0, crate::catalog::island());
        }
        let resonator = g.add_card_to_battlefield(0, crate::catalog::strionic_resonator());
        let elf = g.add_card_to_hand(0, crate::catalog::elvish_visionary());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast the Visionary");
        while !matches!(g.stack.last(), Some(StackItem::Trigger { .. })) && !g.stack.is_empty() {
            g.perform_action(GameAction::PassPriority).expect("resolve the creature");
        }
        g.priority.player_with_priority = 0;
        assert!(pick_trigger_copy(&g, 0).is_none(), "not a pod");
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        let a = pick_trigger_copy(&g, 0).expect("copy the draw");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, .. } if card_id == resonator));
        let hand = g.players[0].hand.len();
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 2, "the trigger and its copy");
    }
}
