//! "Earthquake deals X damage to each creature without flying and each
//! player" also deals X to its caster. The X sizer poured the whole pool into
//! X, so a seat at 11 life with twelve mana cast Earthquake for 11 and drew
//! the game (CR 104.4a) where X = 10 killed both opponents and won. Any X
//! spell (or X creature's ETB) whose X damage or life loss reaches its own
//! caster is capped at the caster's life − 1, halved per X doubler.

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
    // A creature's own ETB reads its X too (Exocrine: "it deals X damage to
    // each player").
    let etb = def.triggered_abilities.iter().filter(|t| {
        t.event.kind == crate::effect::EventKind::EntersBattlefield
            && t.event.scope == crate::effect::EventScope::SelfSource
    });
    std::iter::once(&def.effect).chain(etb.map(|t| &t.effect)).any(effect_hits_caster)
}

fn effect_hits_caster(effect: &Effect) -> bool {
    effect.any_nested(&|e| match e {
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

/// The most X `seat` may declare for `def` without killing itself — halved
/// per "double X" cast trigger of its own (Unbound Flourishing doubled an
/// Exocrine's X from 6 to 12 at 5 life, an all-lose pod, seed 4400324).
pub(super) fn self_harm_x_cap(state: &GameState, seat: usize, def: &CardDefinition) -> Option<u32> {
    if !x_hits_caster(def) {
        return None;
    }
    let doublers = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat)
        .flat_map(|c| c.definition.triggered_abilities.iter())
        .filter(|t| t.effect.any_nested(&|e| matches!(e, Effect::DoubleXOfSpell { .. })))
        .count()
        .min(16) as u32;
    Some(((state.effective_life(seat) - 1).max(0) as u32) >> doublers)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CR 104.4a — Exocrine's ETB deals X to each player, its caster
    /// included; under Unbound Flourishing the X doubles, so at 5 life the
    /// cap is 2 (an X of 6 doubled to 12 drew a four-seat pod, seed 4400324).
    #[test]
    fn an_etb_x_and_an_x_doubler_cap_the_caster_alive() {
        let exocrine = crate::catalog::exocrine();
        assert!(x_hits_caster(&exocrine));
        let mut g = crate::game::multi_player_game(3);
        g.players[0].life = 5;
        assert_eq!(self_harm_x_cap(&g, 0, &exocrine), Some(4));
        g.add_card_to_battlefield(0, crate::catalog::unbound_flourishing());
        assert_eq!(self_harm_x_cap(&g, 0, &exocrine), Some(2));
    }
}
