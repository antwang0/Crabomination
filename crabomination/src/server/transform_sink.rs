//! The bot's sink for a "{cost}: Transform this" activation — the Incubator
//! token's (CR 701.53). No generator built one, so every incubated token
//! stayed an inert artifact and Brimaz, Blight of Oreskos's whole engine was
//! dead in self-play (1 win in 60 four-seat pods). Only a noncreature face
//! that would survive as a creature is flipped, so a counterless Incubator
//! is never turned into a 0/0 that dies at once.

use crate::card::CounterType;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::GameAction;

/// Pods: the first "exile this, then return it transformed" activation of a
/// creature whose back face is a Saga (Urabrask // The Great Work) that the
/// engine accepts — its gate (three instants and sorceries this turn) is the
/// moment the Saga pays. Commander games only, so two-player play is
/// unchanged.
pub(super) fn pick_flip_to_saga(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() {
        return None;
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_creature()) {
        let Some(back) = card.definition.back_face.as_deref() else { continue };
        if !back.subtypes.enchantment_subtypes.contains(&crate::card::EnchantmentSubtype::Saga) {
            continue;
        }
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if !matches!(ab.effect, Effect::ExileSelfReturnTransformed) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// The ability transforms its own source.
pub(super) fn ability_transforms_self(e: &Effect) -> bool {
    matches!(e, Effect::Transform { what: Selector::This })
}

/// The first "{cost}: transform this" activation `seat` can pay for whose
/// back face is a creature with positive toughness once its counters count.
pub(super) fn pick_transform_self(state: &GameState, seat: usize) -> Option<GameAction> {
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        if card.definition.is_creature() {
            if let Some(action) = pick_exile_x_transform(state, seat, card) {
                return Some(action);
            }
            continue;
        }
        let Some(back) = card.definition.back_face.as_deref() else { continue };
        let toughness = back.toughness + card.counter_count(CounterType::PlusOnePlusOne) as i32
            - card.counter_count(CounterType::MinusOneMinusOne) as i32;
        if !back.is_creature() || toughness <= 0 {
            continue;
        }
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if ab.sac_cost || ab.exhaust || !ability_transforms_self(&ab.effect) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// A creature's "{X}…, Exile X creature cards from your graveyard: Transform"
/// (Ludevic, Necrogenius): the largest payable X, since each card is a
/// counter on the back face.
fn pick_exile_x_transform(state: &GameState, seat: usize, card: &crate::card::CardInstance) -> Option<GameAction> {
    let back = card.definition.back_face.as_deref()?;
    if !back.is_creature() {
        return None;
    }
    for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
        if !ab.exile_other_x || ab.sac_cost || ab.exhaust || !ability_transforms_self(&ab.effect) {
            continue;
        }
        let Some((filter, _)) = ab.exile_other_filter.as_ref() else { continue };
        let fodder = state.players[seat]
            .graveyard
            .iter()
            .filter(|c| state.evaluate_requirement_on_card(filter, c, seat))
            .count() as u32;
        for x in (1..=fodder).rev() {
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: Some(x),
                mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// Urabrask flips into The Great Work in a pod once three instants and
    /// sorceries were cast this turn, and not before.
    #[test]
    fn urabrask_flips_after_three_spells() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        let ura = g.add_card_to_battlefield(0, crate::catalog::urabrask());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Red, 1);
        assert!(pick_flip_to_saga(&g, 0).is_none(), "no spells yet");
        g.players[0].instants_or_sorceries_cast_this_turn = 3;
        let a = pick_flip_to_saga(&g, 0).expect("flip");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, .. } if card_id == ura));
    }
}

