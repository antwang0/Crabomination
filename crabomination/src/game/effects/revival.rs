//! Revival Trance (FIC, Final Fantasy VI) effects: a random mode, opponents
//! handing back your graveyard, riders on a free-cast creature, and free
//! casts that bill each card's owner.

use rand::RngExt;

use super::EffectContext;
use crate::card::{CardId, SelectionRequirement, Zone};
use crate::decision::{Decision, DecisionAnswer, OptionalKind};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::types::{GameError, GameEvent, StackItem, Target};
use crate::game::GameState;

impl GameState {
    /// `Effect::ChooseModeAtRandom` — one mode, uniformly (Umaro's "choose one
    /// at random"). Not a die roll: nothing that watches rolls sees it.
    pub(super) fn choose_mode_at_random(
        &mut self,
        modes: &[Effect],
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        if modes.is_empty() {
            return Ok(());
        }
        let idx = self.rng.draw().random_range(0..modes.len());
        let mode = &modes[idx];
        // The mode is only known now, so a targeted one takes the
        // auto-picker's target as it resolves.
        if mode.requires_target() {
            let Some(t) = self.auto_target_for_effect_avoiding(mode, ctx.controller, ctx.source) else {
                return Ok(());
            };
            let mut c = ctx.clone();
            c.targets = vec![t];
            return self.run_effect(mode, &c, events);
        }
        self.run_effect(mode, ctx, events)
    }

    /// `Effect::EachOpponentReturnsFromYourGraveyard` — starting with the next
    /// opponent in turn order, each opponent chooses a matching card in your
    /// graveyard nobody chose yet (the weakest is offered first); every
    /// chosen card returns to the battlefield under your control (Rejoin the
    /// Fight).
    pub(super) fn each_opponent_returns_from_your_graveyard(
        &mut self,
        filter: &SelectionRequirement,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let opponents: Vec<usize> = self
            .seats_in_turn_order_from(me)
            .into_iter()
            .filter(|s| self.opponents_of(me).contains(s))
            .collect();
        let mut cursor = 0;
        let mut chosen: Vec<CardId> = Vec::new();
        for opp in opponents {
            let mut legal: Vec<(u32, CardId)> = self.players[me]
                .graveyard
                .iter()
                .filter(|c| !chosen.contains(&c.id) && self.evaluate_requirement_on_card(filter, c, me))
                .map(|c| (c.definition.cost.cmc(), c.id))
                .collect();
            if legal.is_empty() {
                break;
            }
            legal.sort();
            // `legal` is non-empty, so `None` is a suspend: stop and let the
            // resume replay the logged picks (asking the next opponent would
            // overwrite the parked ask — `scripts/audit_stash_in_loop.py`).
            let Some(picked) = self.ask_seat_target_logged(
                &mut cursor,
                opp,
                format!("P{opp}: choose a card in P{me}'s graveyard to return"),
                source,
                legal.into_iter().map(|(_, id)| Target::Permanent(id)).collect(),
                effect,
            ) else {
                return Ok(());
            };
            if let Target::Permanent(id) = picked {
                chosen.push(id);
            }
        }
        self.clear_answer_log();
        let dest = ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false };
        for id in chosen {
            self.move_card_to(id, &dest, ctx, events);
        }
        Ok(())
    }

    /// `Effect::GrantCastSpellRiders` — a creature spell `what` names that is
    /// on the stack gains haste and/or "sacrifice at the beginning of the
    /// next end step" as it resolves (Strago and Relm's free cast).
    pub(super) fn grant_cast_spell_riders(&mut self, what: &Selector, haste: bool, sacrifice_eot: bool, ctx: &EffectContext) {
        let ids: Vec<CardId> = self.resolve_selector(what, ctx).into_iter().filter_map(|e| e.as_card_id()).collect();
        for item in self.stack.iter_mut() {
            if let StackItem::Spell { card, .. } = item
                && ids.contains(&card.id)
                && card.definition.is_creature()
            {
                card.resolve_riders = Some((haste, sacrifice_eot));
            }
        }
    }

    /// `Effect::CastExiledFree` / `CastExiledFreeOwnersLoseLife` — you may
    /// cast any of the exiled cards `what` names without paying their mana
    /// costs (Etali, Primal Storm); with `owners_lose_life`, each spell's owner
    /// then loses life equal to its mana value (Kefka, Dancing Mad). Lands
    /// can't be cast and are skipped.
    pub(super) fn cast_exiled_free(
        &mut self,
        what: &Selector,
        owners_lose_life: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let ids: Vec<CardId> = self.resolve_selector(what, ctx).into_iter().filter_map(|e| e.as_card_id()).collect();
        for cid in ids {
            let Some(card) = self.exile.iter().find(|c| c.id == cid) else { continue };
            if card.definition.is_land() {
                continue;
            }
            let (owner, mv, name, def) =
                (card.owner, card.definition.cost.cmc(), card.definition.name, card.definition.arc());
            let take = match self.decider.kind() {
                crate::decision::DeciderKind::Auto => true,
                _ => matches!(
                    self.decider.decide(&Decision::OptionalTrigger {
                        source: ctx.source.unwrap_or(CardId(0)),
                        description: format!("Cast {name} without paying its mana cost?"),
                        kind: OptionalKind::CastFree,
                    }),
                    DecisionAnswer::Bool(true)
                ),
            };
            if !take {
                continue;
            }
            let target = self.auto_target_for_effect_avoiding(&def.effect, me, Some(cid));
            if let Ok(mut ev) = self.cast_card_for_free(me, cid, Zone::Exile, target, vec![], None, None, false) {
                events.append(&mut ev);
                if !owners_lose_life {
                    continue;
                }
                let lose = Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Seat(owner)),
                    amount: Value::Const(mv as i32),
                };
                self.run_effect(&lose, ctx, events)?;
            }
        }
        Ok(())
    }

    /// `Effect::ExileUntilCastOneTakeOne` (Invasion of Alara). The bot casts
    /// the highest mana value hit it can and keeps the next; a seat with a UI
    /// is asked per hit, then which hit goes to hand.
    pub(super) fn exile_until_cast_one_take_one(
        &mut self,
        count: u32,
        filter: &SelectionRequirement,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        let p = ctx.controller;
        let (mut exiled, mut hits) = (Vec::new(), Vec::new());
        while hits.len() < count as usize {
            let Some(top) = self.players[p].library.first().map(|c| c.id) else { break };
            self.move_card_to(top, &ZoneDest::Exile, ctx, events);
            exiled.push(top);
            if self.exile.iter().find(|c| c.id == top).is_some_and(|c| self.evaluate_requirement_on_card(filter, c, p)) {
                hits.push(top);
            }
        }
        let mv = |g: &Self, id: CardId| g.exile.iter().find(|c| c.id == id).map_or(0, |c| c.definition.cost.cmc());
        hits.sort_by_key(|&id| std::cmp::Reverse(mv(self, id)));
        let auto = matches!(self.decider.kind(), crate::decision::DeciderKind::Auto);
        let mut cast = None;
        for &h in &hits {
            let Some(def) = self.exile.iter().find(|c| c.id == h).map(|c| c.definition.arc()) else { continue };
            let take = auto
                || matches!(
                    self.decider.decide(&Decision::OptionalTrigger {
                        source: ctx.source.unwrap_or(CardId(0)),
                        description: format!("Cast {} without paying its mana cost?", def.name),
                        kind: OptionalKind::CastFree,
                    }),
                    DecisionAnswer::Bool(true)
                );
            if !take {
                continue;
            }
            let (target, more) = self.auto_targets_for_effect_all_slots_sourced(&def.effect, p, None, Some(h));
            if let Ok(mut ev) = self.cast_card_for_free(p, h, Zone::Exile, target, more, None, None, false) {
                events.append(&mut ev);
                cast = Some(h);
                break;
            }
        }
        let left: Vec<CardId> = hits.iter().copied().filter(|&h| Some(h) != cast).collect();
        let keep = match left.as_slice() {
            [] => None,
            [only] => Some(*only),
            [first, second, ..] if !auto => {
                let name = self.exile.iter().find(|c| c.id == *first).map_or("", |c| c.definition.name);
                let first_ok = matches!(
                    self.decider.decide(&Decision::OptionalTrigger {
                        source: ctx.source.unwrap_or(CardId(0)),
                        description: format!("Put {name} into your hand (otherwise the other card)?"),
                        kind: OptionalKind::MayBody,
                    }),
                    DecisionAnswer::Bool(true)
                );
                Some(if first_ok { *first } else { *second })
            }
            [first, ..] => Some(*first),
        };
        if let Some(k) = keep {
            self.move_card_to(k, &ZoneDest::Hand(PlayerRef::You), ctx, events);
        }
        let rest: Vec<CardId> = exiled.into_iter().filter(|id| self.exile.iter().any(|c| c.id == *id)).collect();
        for &id in &rest {
            self.move_card_to(
                id,
                &ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Bottom },
                ctx,
                events,
            );
        }
        self.bottom_in_random_order(p, &rest);
    }
}
