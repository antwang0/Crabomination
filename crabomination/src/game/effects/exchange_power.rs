//! CR 701.10g — exchanging power between two creatures (Serene Master).

use super::EffectContext;
use crate::card::CardId;
use crate::effect::{Duration, Selector};
use crate::game::GameState;
use crate::game::layers::{AffectedPermanents, ContinuousEffect, Layer, Modification, PtSublayer};

impl GameState {
    /// `Effect::ExchangePower` — each creature gets the other's current power
    /// as a layer-7b set for `duration`. Nothing happens unless both are
    /// distinct creatures on the battlefield (CR 701.10b).
    pub(super) fn exchange_power(&mut self, a: &Selector, b: &Selector, duration: Duration, ctx: &EffectContext) {
        let pick = |s: &Selector| {
            self.resolve_selector(s, ctx)
                .into_iter()
                .find_map(|e| e.as_permanent_id())
                .and_then(|id| self.computed_permanent(id).filter(|cp| cp.card_types().contains(&crate::card::CardType::Creature)).map(|cp| (id, cp.power)))
        };
        let (Some((x, px)), Some((y, py))) = (pick(a), pick(b)) else { return };
        if x == y {
            return;
        }
        let kind = self.effect_duration_for(duration, ctx.controller);
        let source = ctx.source.unwrap_or(CardId(0));
        for (id, power) in [(x, py), (y, px)] {
            let timestamp = self.next_timestamp();
            self.add_continuous_effect(ContinuousEffect {
                timestamp,
                source,
                affected: AffectedPermanents::just(id),
                layer: Layer::L7PowerTough,
                sublayer: Some(PtSublayer::SetValue),
                duration: kind.clone(),
                modification: Modification::SetPower(power),
            });
        }
    }
}
