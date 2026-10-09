//! "Earthquake deals X damage to each creature without flying and each
//! player" also deals X to its caster. The X sizer poured the whole pool into
//! X, so a seat at 11 life with twelve mana cast Earthquake for 11 and drew
//! the game (CR 104.4a) where X = 10 killed both opponents and won. Any X
//! spell whose X damage or life loss reaches its own caster is capped at the
//! caster's life − 1.

use crate::card::CardDefinition;
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::game::GameState;

/// Does `who` name the caster among the players it covers?
fn covers_caster(who: &Selector) -> bool {
    matches!(who, Selector::Player(PlayerRef::EachPlayer | PlayerRef::You) | Selector::You)
}

/// Is this leaf X damage or X life loss aimed at the caster?
fn leaf_hits_caster(e: &Effect) -> bool {
    match e {
        Effect::DealDamage { to, amount: Value::XFromCost } => covers_caster(to),
        Effect::LoseLife { who, amount: Value::XFromCost } => covers_caster(who),
        _ => false,
    }
}

/// True when `def`'s X damage or life loss reaches its caster: directly
/// (`DealDamage { to: each player }`) or through a per-player loop
/// (`ForEach { each player } { DealDamage { to: TriggerSource } }`).
pub(super) fn x_hits_caster(def: &CardDefinition) -> bool {
    def.effect.any_nested(&|e| match e {
        Effect::ForEach { selector, body } if covers_caster(selector) => body.any_nested(&|inner| {
            matches!(
                inner,
                Effect::DealDamage { to: Selector::TriggerSource, amount: Value::XFromCost }
                    | Effect::LoseLife { who: Selector::TriggerSource, amount: Value::XFromCost }
            )
        }),
        other => leaf_hits_caster(other),
    })
}

/// The most X `seat` may declare for `def` without killing itself.
pub(super) fn self_harm_x_cap(state: &GameState, seat: usize, def: &CardDefinition) -> Option<u32> {
    x_hits_caster(def).then(|| (state.effective_life(seat) - 1).max(0) as u32)
}
