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

    /// `Effect::CastExiledFreeOwnersLoseLife` — you may cast any of the
    /// exiled cards `what` names without paying their mana costs; each
    /// spell's owner then loses life equal to its mana value (Kefka, Dancing
    /// Mad). Lands can't be cast and are skipped.
    pub(super) fn cast_exiled_free_owners_lose_life(
        &mut self,
        what: &Selector,
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
                let lose = Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Seat(owner)),
                    amount: Value::Const(mv as i32),
                };
                self.run_effect(&lose, ctx, events)?;
            }
        }
        Ok(())
    }
}
