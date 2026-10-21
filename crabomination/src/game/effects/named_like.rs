//! `Effect::BinExiledWithSourceNamedLike` — Jester's Scepter's "put a card
//! exiled with this artifact into its owner's graveyard", the card picked to
//! share a name with the spell the ability counters.

use crate::card::CardId;
use crate::effect::{Selector, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::{GameEvent, GameState};

impl GameState {
    pub(super) fn bin_exiled_with_source_named_like(
        &mut self,
        what: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        let Some(source) = ctx.source else { return };
        let name = self
            .resolve_selector(what, ctx)
            .into_iter()
            .find_map(|e| e.as_card_id())
            .and_then(|id| self.find_card_anywhere(id))
            .map(|c| c.definition.name);
        let Some(name) = name else { return };
        let pick: Option<CardId> = self
            .exile
            .iter()
            .find(|c| c.exiled_with == Some(source) && c.definition.name == name)
            .map(|c| c.id);
        if let Some(id) = pick {
            self.move_card_to(id, &ZoneDest::Graveyard, ctx, events);
        }
    }
}
