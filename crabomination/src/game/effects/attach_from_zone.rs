//! Aura and Equipment cards put onto the battlefield attached to something
//! (CR 303.4f / 301.5c): Mantle of the Ancients, Unfinished Business,
//! Retether, Liberated Livestock, Knickknack Ouphe, Songbirds' Blessing.

use crate::card::{CardId, SelectionRequirement, Zone};
use crate::effect::{PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, Target};
use crate::game::GameState;

impl GameState {
    /// The permanent `card` (an Aura or Equipment card not yet on the
    /// battlefield) would attach to: `host` when it is legal, else — with no
    /// host named — the best legal permanent (yours first, then the greatest
    /// power). `None` when an Aura has nothing to enchant (CR 303.4i: it stays
    /// where it is). An Equipment only ever attaches to a creature.
    fn attach_host_for(
        &self,
        card: CardId,
        host: Option<CardId>,
        creatures_only: bool,
        p: usize,
    ) -> Option<CardId> {
        let def = self.find_card_anywhere(card)?.definition.clone();
        let fits = |id: CardId| -> bool {
            let Some(h) = self.battlefield_find(id) else { return false };
            if def.is_aura() {
                if creatures_only && !h.definition.is_creature() {
                    return false;
                }
                match def.aura_enchant_filter() {
                    Some(f) => self.evaluate_requirement_static(f, &Target::Permanent(id), p, Some(card)),
                    None => h.definition.is_creature(),
                }
            } else {
                h.definition.is_creature()
            }
        };
        if let Some(h) = host {
            return fits(h).then_some(h);
        }
        self.battlefield
            .iter()
            .filter(|c| fits(c.id))
            .max_by_key(|c| (c.controller == p, c.definition.is_creature(), c.power()))
            .map(|c| c.id)
    }

    /// Move `card` onto the battlefield under `p` attached to `host`.
    fn put_attached(&mut self, card: CardId, host: CardId, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        self.move_card_to(card, &ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false }, ctx, events);
        if let Some(c) = self.battlefield_find_mut(card) {
            c.attached_to = Some(host);
            events.push(GameEvent::AttachmentMoved { attachment: card, attached_to: Some(host) });
        }
    }

    /// `Effect::PutOntoBattlefieldAttached` — every `filter` card in the
    /// controller's `zones` (greatest mana value first, at most `max`) goes
    /// onto the battlefield attached to `host`, or to its own best legal host
    /// when `host` is `None`. A card with no legal host stays put.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn put_onto_battlefield_attached(
        &mut self,
        zones: &[Zone],
        filter: &SelectionRequirement,
        host: Option<&Selector>,
        max: Option<&Value>,
        creatures_only: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        let p = ctx.controller;
        let host_id = match host {
            Some(sel) => match self.resolve_selector(sel, ctx).into_iter().find_map(|e| e.as_permanent_id()) {
                Some(h) => Some(h),
                None => return,
            },
            None => None,
        };
        let filter = &filter.resolve_x(ctx.x_value);
        let mut cap = max.map(|v| self.evaluate_value(v, ctx).max(0) as usize).unwrap_or(usize::MAX);
        for zone in zones {
            let pick = |c: &crate::card::CardInstance| {
                self.evaluate_requirement_on_card(filter, c, p).then(|| (c.id, c.definition.cost.cmc()))
            };
            let mut ids: Vec<(CardId, u32)> = match zone {
                Zone::Graveyard => self.players[p].graveyard.iter().filter_map(pick).collect(),
                Zone::Hand => self.players[p].hand.iter().filter_map(pick).collect(),
                _ => continue,
            };
            ids.sort_by_key(|(_, mv)| std::cmp::Reverse(*mv));
            for (id, _) in ids {
                if cap == 0 {
                    return;
                }
                if let Some(h) = self.attach_host_for(id, host_id, creatures_only, p) {
                    self.put_attached(id, h, ctx, events);
                    self.scratch.last_moved_cards.push(id);
                    cap -= 1;
                }
            }
        }
    }

    /// `Effect::RevealTopPutAttached` — reveal the top `count` cards; each
    /// `filter` card among them goes onto the battlefield attached to its best
    /// legal host; the rest go to the bottom in a random order (Knickknack
    /// Ouphe).
    pub(super) fn reveal_top_put_attached(
        &mut self,
        count: &Value,
        filter: &SelectionRequirement,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        let p = ctx.controller;
        let n = self.evaluate_value(count, ctx).max(0) as usize;
        let filter = &filter.resolve_x(ctx.x_value);
        let revealed: Vec<CardId> = self.players[p].library.iter().take(n).map(|c| c.id).collect();
        for &id in &revealed {
            let fits = self.players[p]
                .library
                .iter()
                .find(|c| c.id == id)
                .is_some_and(|c| self.evaluate_requirement_on_card(filter, c, p));
            if fits && let Some(h) = self.attach_host_for(id, None, false, p) {
                self.put_attached(id, h, ctx, events);
            }
        }
        self.bottom_in_random_order(p, &revealed);
    }

    /// `Effect::RevealUntilPutAttachedElseHand` — reveal until a `filter`
    /// card; it goes onto the battlefield attached to its best legal host, or
    /// into the hand with none; the rest go to the bottom in a random order
    /// (Songbirds' Blessing).
    pub(super) fn reveal_until_put_attached_else_hand(
        &mut self,
        filter: &SelectionRequirement,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        let p = ctx.controller;
        let Some(pos) = self.players[p]
            .library
            .iter()
            .position(|c| self.evaluate_requirement_on_card(filter, c, p))
        else {
            let all: Vec<CardId> = self.players[p].library.iter().map(|c| c.id).collect();
            self.bottom_in_random_order(p, &all);
            return;
        };
        let rest: Vec<CardId> = self.players[p].library.iter().take(pos).map(|c| c.id).collect();
        let hit = self.players[p].library[pos].id;
        match self.attach_host_for(hit, None, false, p) {
            Some(h) => self.put_attached(hit, h, ctx, events),
            None => {
                self.move_card_to(hit, &ZoneDest::Hand(PlayerRef::You), ctx, events);
            }
        }
        self.bottom_in_random_order(p, &rest);
    }

    /// CR 401.4 — the `ids` still in `p`'s library go to its bottom in a
    /// random order.
    fn bottom_in_random_order(&mut self, p: usize, ids: &[CardId]) {
        use rand::seq::SliceRandom;
        let mut rest: Vec<CardId> =
            ids.iter().copied().filter(|id| self.players[p].library.iter().any(|c| c.id == *id)).collect();
        rest.shuffle(&mut self.rng.draw());
        for id in rest {
            if let Some(card) = Self::take_card(&mut self.players[p].library, id) {
                self.players[p].library.push(card);
            }
        }
    }
}
