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

/// The ability transforms its own source.
pub(super) fn ability_transforms_self(e: &Effect) -> bool {
    matches!(e, Effect::Transform { what: Selector::This })
}

/// The first "{cost}: transform this" activation `seat` can pay for whose
/// back face is a creature with positive toughness once its counters count.
pub(super) fn pick_transform_self(state: &GameState, seat: usize) -> Option<GameAction> {
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        if card.definition.is_creature() {
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
