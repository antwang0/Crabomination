//! Which effects move a creature's power and toughness, by source, for the
//! client's card inspector.
//!
//! A source's share is what the layer pass loses without it (one pass with
//! every effect, one per source left out), so every layer-7 kind is read as
//! the layer pass reads it. Off the hot path: only view projection calls it,
//! for creatures an effect changes. Shares needn't sum where effects interact
//! (a 7b set under a 7c pump); the remainder prints as "other effects".
//!
//! Until-end-of-turn pumps (`CardInstance::power_bonus`) and lasting ones
//! (`perm_power_bonus`) carry no source; they get a line of their own.

use super::GameState;
use crate::card::{CardId, CardInstance};
use crate::game::layers::{affected_includes, apply_layers_one, ContinuousEffect, Layer};

/// One line of a creature's P/T breakdown.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PtShare {
    /// The source's name, or what kind of untracked pump it is.
    pub label: String,
    pub power: i32,
    pub toughness: i32,
}

impl GameState {
    /// The continuous effects the layer pass would apply now, for
    /// [`pt_shares_with`](Self::pt_shares_with) — gathered once per view.
    pub(crate) fn pt_share_effects(&self) -> Vec<ContinuousEffect> {
        self.gather_continuous_effects()
    }

    /// `card`'s P/T breakdown against `effects`: each source of a layer-7
    /// effect that reaches it, with that source's share, then its untracked
    /// pumps. Empty when nothing but counters moves it.
    pub(crate) fn pt_shares_with(&self, card: &CardInstance, effects: &[ContinuousEffect]) -> Vec<PtShare> {
        let mut out = Vec::new();
        let full = apply_layers_one(card, effects);
        let mut sources: Vec<CardId> = effects
            .iter()
            .filter(|e| e.layer == Layer::L7PowerTough && affected_includes(&e.affected, e.source, card))
            .map(|e| e.source)
            .collect();
        sources.sort();
        sources.dedup();
        for source in sources {
            let without: Vec<ContinuousEffect> = effects.iter().filter(|e| e.source != source).cloned().collect();
            let rest = apply_layers_one(card, &without);
            let (power, toughness) =
                (full.power.saturating_sub(rest.power), full.toughness.saturating_sub(rest.toughness));
            if (power, toughness) == (0, 0) {
                continue;
            }
            let label = if source == card.id {
                "its own ability".to_string()
            } else {
                self.find_card_anywhere(source)
                    .map_or_else(|| "an effect".to_string(), |c| c.definition.name.to_string())
            };
            out.push(PtShare { label, power, toughness });
        }
        if (card.power_bonus, card.toughness_bonus) != (0, 0) {
            out.push(PtShare {
                label: "until end of turn".to_string(),
                power: card.power_bonus,
                toughness: card.toughness_bonus,
            });
        }
        if (card.perm_power_bonus, card.perm_toughness_bonus) != (0, 0) {
            out.push(PtShare {
                label: "a lasting effect".to_string(),
                power: card.perm_power_bonus,
                toughness: card.perm_toughness_bonus,
            });
        }
        out
    }

    /// [`pt_shares_with`](Self::pt_shares_with) for one battlefield
    /// permanent, gathering the effects itself.
    pub fn pt_shares(&self, id: CardId) -> Vec<PtShare> {
        let Some(card) = self.battlefield.iter().find(|c| c.id == id) else { return Vec::new() };
        self.pt_shares_with(card, &self.pt_share_effects())
    }
}
