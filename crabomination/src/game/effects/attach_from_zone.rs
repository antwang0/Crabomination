//! Aura and Equipment cards put onto the battlefield attached to something
//! (CR 303.4f / 301.5c): Mantle of the Ancients, Unfinished Business,
//! Retether, Liberated Livestock, Knickknack Ouphe, Songbirds' Blessing.

use crate::card::{CardId, SelectionRequirement, Zone};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, Target};
use crate::game::GameState;

impl GameState {
    /// CR 303.4f — the permanent `card` (an Aura or Equipment card not yet on
    /// the battlefield) attaches to: `host` when the effect names one and it
    /// is legal, else the legal permanent `p` — the player putting it there —
    /// chooses (headless: yours first, then the greatest power). Inner `None`
    /// when an Aura has nothing to enchant (CR 303.4i: it stays where it is);
    /// outer `None` is a suspend. An Equipment only ever attaches to a
    /// creature.
    #[allow(clippy::too_many_arguments)]
    fn choose_attach_host(
        &mut self,
        cursor: &mut usize,
        card: CardId,
        host: Option<CardId>,
        creatures_only: bool,
        p: usize,
        effect: &Effect,
        may: bool,
    ) -> Option<Option<CardId>> {
        let hosts = self.attach_hosts_for(card, host, creatures_only, p);
        let asks = self.seat_prompts(p) || !matches!(self.decider.kind(), crate::decision::DeciderKind::Auto);
        if hosts.is_empty() || !asks || (hosts.len() == 1 && !may) {
            return Some(hosts.first().copied());
        }
        let name = self.find_card_anywhere(card).map_or(String::new(), |c| c.definition.name.to_string());
        let legal: Vec<Target> = hosts.into_iter().map(Target::Permanent).collect();
        // `may`: declining keeps the card where it is ("you may put …").
        match self.ask_seat_target_maybe_logged(cursor, p, format!("Attach {name} to"), card, legal, effect, may)? {
            Some(Target::Permanent(h)) => Some(Some(h)),
            _ => Some(None),
        }
    }

    /// The legal hosts for [`choose_attach_host`](Self::choose_attach_host),
    /// the headless pick first.
    fn attach_hosts_for(&self, card: CardId, host: Option<CardId>, creatures_only: bool, p: usize) -> Vec<CardId> {
        let Some(def) = self.find_card_anywhere(card).map(|c| c.definition.clone()) else { return Vec::new() };
        let fits = |id: CardId| -> bool {
            let Some(h) = self.battlefield_find(id) else { return false };
            if def.is_aura() {
                if creatures_only && !self.computed_is_creature(h) {
                    return false;
                }
                match def.aura_enchant_filter() {
                    Some(f) => self.evaluate_requirement_static(f, &Target::Permanent(id), p, Some(card)),
                    None => self.computed_is_creature(h),
                }
            } else {
                self.computed_is_creature(h)
            }
        };
        if let Some(h) = host {
            return if fits(h) { vec![h] } else { Vec::new() };
        }
        let mut hosts: Vec<&crate::card::CardInstance> = self.battlefield.iter().filter(|c| fits(c.id)).collect();
        // Stable over battlefield order, so ties keep the old pick (the last).
        hosts.reverse();
        hosts.sort_by_key(|c| std::cmp::Reverse((c.controller == p, self.computed_is_creature(c), c.power())));
        hosts.into_iter().map(|c| c.id).collect()
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
        effect: &Effect,
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
        let cap = max.map(|v| self.evaluate_value(v, ctx).max(0) as usize).unwrap_or(usize::MAX);
        // The candidates in the headless order: zone by zone, greatest mana
        // value first.
        let mut cands: Vec<CardId> = Vec::new();
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
            cands.extend(ids.into_iter().map(|(id, _)| id));
        }
        // Every choice is made first, then the cards move (a re-run replays):
        // which cards when the cap binds (Liberated Livestock's "an Aura
        // card"), then each one's host (CR 303.4f).
        let mut cursor = 0;
        if cap < cands.len()
            && (self.seat_prompts(p) || !matches!(self.decider.kind(), crate::decision::DeciderKind::Auto))
        {
            let hosted: Vec<CardId> = cands
                .iter()
                .copied()
                .filter(|&id| !self.attach_hosts_for(id, host_id, creatures_only, p).is_empty())
                .collect();
            let named: Vec<(CardId, String)> = hosted
                .iter()
                .filter_map(|&id| self.find_card_anywhere(id).map(|c| (id, c.definition.name.to_string())))
                .collect();
            let auto: Vec<CardId> = hosted.iter().copied().take(cap).collect();
            let Some(chosen) = self.ask_seat_cards_logged(
                &mut cursor,
                p,
                format!("Choose up to {cap} to put onto the battlefield attached"),
                ctx.source.unwrap_or(CardId(0)),
                named,
                0,
                cap as u32,
                crate::decision::PickValue::Gain,
                effect,
                auto,
            ) else {
                return;
            };
            cands = chosen;
        }
        let mut cap = cap;
        let mut picks: Vec<(CardId, CardId)> = Vec::new();
        for id in cands {
            if cap == 0 {
                break;
            }
            let Some(h) = self.choose_attach_host(&mut cursor, id, host_id, creatures_only, p, effect, false) else {
                return;
            };
            if let Some(h) = h {
                picks.push((id, h));
                cap -= 1;
            }
        }
        self.clear_answer_log();
        for (id, h) in picks {
            self.put_attached(id, h, ctx, events);
            self.scratch.last_moved_cards.push(id);
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
        effect: &Effect,
    ) {
        let p = ctx.controller;
        let n = self.evaluate_value(count, ctx).max(0) as usize;
        let filter = &filter.resolve_x(ctx.x_value);
        let revealed: Vec<CardId> = self.players[p].library.iter().take(n).map(|c| c.id).collect();
        let mut cursor = 0;
        // "You may put any number of" them: which ones first (headless: all
        // that have a host), then each one's host.
        let eligible: Vec<(CardId, String)> = revealed
            .iter()
            .filter_map(|&id| self.players[p].library.iter().find(|c| c.id == id))
            .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
            .filter(|c| !self.attach_hosts_for(c.id, None, false, p).is_empty())
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        let auto: Vec<CardId> = eligible.iter().map(|e| e.0).collect();
        let max = eligible.len() as u32;
        let chosen = if eligible.is_empty() {
            Vec::new()
        } else if let Some(chosen) = self.ask_seat_cards_logged(
            &mut cursor,
            p,
            "Put any number of these onto the battlefield".into(),
            ctx.source.unwrap_or(CardId(0)),
            eligible,
            0,
            max,
            crate::decision::PickValue::Gain,
            effect,
            auto,
        ) {
            chosen
        } else {
            return;
        };
        let mut picks: Vec<(CardId, CardId)> = Vec::new();
        for id in chosen {
            let Some(h) = self.choose_attach_host(&mut cursor, id, None, false, p, effect, false) else { return };
            if let Some(h) = h {
                picks.push((id, h));
            }
        }
        self.clear_answer_log();
        for (id, h) in picks {
            self.put_attached(id, h, ctx, events);
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
        effect: &Effect,
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
        let mut cursor = 0;
        let Some(host) = self.choose_attach_host(&mut cursor, hit, None, false, p, effect, true) else { return };
        self.clear_answer_log();
        match host {
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
