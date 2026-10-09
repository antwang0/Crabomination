//! "Search your library for an Aura card with mana value less than or equal
//! to that Aura and with a different name than each Aura you control, put
//! that card onto the battlefield attached to [this], then shuffle" (Light-
//! Paws, Emperor's Voice).

use super::{EffectContext, PickValue};
use crate::card::{CardId, EnchantmentSubtype};
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, Target};

impl GameState {
    /// `Effect::SearchAuraAttachToSourceCappedBy` — the candidates are the
    /// library's Auras whose mana value is at most `cap`'s, whose name no Aura
    /// the controller controls shares, and which can legally enchant the
    /// source. Headless: the costliest. Searching shuffles even on a miss.
    pub(super) fn search_aura_attach_to_source_capped_by(
        &mut self,
        cap: &Selector,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let Some(host) = ctx.source.filter(|&s| self.battlefield_find(s).is_some()) else { return Ok(()) };
        let Some(max) = self
            .resolve_selector(cap, ctx)
            .into_iter()
            .find_map(|e| e.as_card_id())
            .and_then(|id| self.find_card_anywhere(id))
            .map(|c| c.definition.cost.cmc())
        else {
            return Ok(());
        };
        let is_aura = |c: &crate::card::CardInstance| {
            c.definition.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Aura)
        };
        let names: Vec<&str> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == p && is_aura(c))
            .map(|c| c.definition.name)
            .collect();
        let mut candidates: Vec<(CardId, String, u32)> = self.players[p]
            .library
            .iter()
            .filter(|c| is_aura(c) && c.definition.cost.cmc() <= max && !names.contains(&c.definition.name))
            .filter(|c| {
                c.definition.effect.target_filter_for_slot(0).is_none_or(|f| {
                    self.evaluate_requirement_static(f, &Target::Permanent(host), p, Some(c.id))
                })
            })
            .map(|c| (c.id, c.definition.name.to_string(), c.definition.cost.cmc()))
            .collect();
        // Stable: library order breaks ties.
        candidates.sort_by_key(|&(_, _, mv)| std::cmp::Reverse(mv));
        let auto: Vec<CardId> = candidates.first().map(|&(id, _, _)| id).into_iter().collect();
        let offer: Vec<(CardId, String)> = candidates.into_iter().map(|(id, name, _)| (id, name)).collect();
        let pick = if offer.is_empty() {
            Vec::new()
        } else {
            let Some(pick) = self.choose_up_to_cards(
                p,
                "Put an Aura onto the battlefield attached to this?".into(),
                host,
                offer,
                1,
                PickValue::Gain,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            pick
        };
        if let Some(&aura) = pick.first() {
            self.move_card_to(aura, &ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false }, ctx, events);
            if let Some(c) = self.battlefield_find_mut(aura) {
                c.attached_to = Some(host);
            }
        }
        self.shuffle_library(p, events);
        Ok(())
    }
}
