//! `Effect::CastWithoutPayingImmediate` and `Effect::CastCopyForCost`: cast a
//! card (or a copy of it) as an effect resolves — free, at its own cost, or at
//! a printed alternative cost ("you may cast the copy by paying {3} rather than
//! paying its mana cost" — Blue Mage's Cane).

use super::{EffectContext, EntityRef};
use crate::card::{CardId, Zone};
use crate::decision::OptionalKind;
use crate::effect::Selector;
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};
use crate::mana::ManaCost;

impl GameState {
    /// The shared body. `cost_override` replaces the card's mana cost (an
    /// alternative cost, CR 118.9); taxes and discounts still apply.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn cast_immediate(
        &mut self,
        what: &Selector,
        source_zone: &Zone,
        exile_after: &bool,
        pay_own_cost: &bool,
        copy: &bool,
        reduce_generic: &u32,
        cost_override: Option<&ManaCost>,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        // Resolve `what` to a single card in `source_zone`, ask
        // the controller via OptionalTrigger, and on yes hand
        // off to the free-cast helper. The helper auto-targets /
        // auto-modes the card; ScriptedDecider can override.
        let entities = self.resolve_selector(what, ctx);
        // A targeted graveyard/exile card resolves to `Permanent` (the
        // only `Target` card variant); a `LastMoved`/zone selector may
        // yield `Card`. Accept either.
        let card_id = entities.into_iter().find_map(|e| match e {
            EntityRef::Card(id) | EntityRef::Permanent(id) => Some(id),
            _ => None,
        });
        let Some(card_id) = card_id else { return Ok(()); };
        // Confirm the card is actually in the named zone — the
        // selector may have read a stale target.
        if self.find_card_zone(card_id) != Some(*source_zone) {
            return Ok(());
        }
        // A Cage / Magistrate / graveyard lock forbids this cast, and
        // a forbidden cast is a no-op, not an `Err` — `cast_card_for_free`'s
        // doc has the whole story (`cube` / `all` seed 1254) and covers
        // the twelve call sites that go through it. This arm is NOT one
        // of them: a `pay_own_cost` / discounted cast charges real mana,
        // so it calls `cast_card_from_zone_spending` directly below and
        // needs its own check. Asked here rather than at that call
        // because it also keeps the controller from being OFFERED a
        // cast that cannot happen — the prompt is a few lines down.
        if matches!(
            source_zone,
            crate::card::Zone::Graveyard | crate::card::Zone::Library | crate::card::Zone::Exile
        ) {
            let def = self.find_card_anywhere(card_id).map(|c| c.definition.arc());
            if let Some(def) = def
                && self.cast_from_zone_blocked(ctx.controller, &def, *source_zone)
            {
                return Ok(());
            }
        }
        // Lands are played, not cast — skip them silently. This
        // makes `ForEach LastMoved → CastWithoutPayingImmediate`
        // safe to use over a mixed top-of-library exile (e.g.
        // Improvisation Capstone exiles whatever's on top).
        let is_land = self
            .find_card_anywhere(card_id)
            .map(|c| c.definition.card_types.contains(&crate::card::CardType::Land))
            .unwrap_or(false);
        if is_land { return Ok(()); }
        // A free cast is optional; it is declined when it would carry
        // the battlefield past the engine's bound — `CopyCardAndCastFree`
        // has the Surge to Victory / Leitmotif Composer story.
        if self.battlefield.len() as i64 + self.spell_token_estimate(card_id, ctx.controller)
            > crate::recommend::BOARD_GATE as i64
        {
            return Ok(());
        }
        use crate::decision::{Decision, DecisionAnswer};
        let source_for_ask = ctx.source.unwrap_or(CardId(0));
        // Free cast = pure upside, so every non-scripted seat
        // accepts (the blanket "no" killed every ForEach→
        // CastWithoutPayingImmediate card). No suspension here: this
        // arm often runs inside a ForEach over LastMoved, and a
        // stash-and-rerun suspend would re-queue only the inner cast,
        // dropping the loop's remaining offers.
        let yes = match self.decider.kind() {
            crate::decision::DeciderKind::Auto => true,
            _ => matches!(
                self.decider.decide(&Decision::OptionalTrigger {
                    source: source_for_ask,
                    description: if *pay_own_cost {
                        "Cast the copy for its cost?".to_string()
                    } else if *reduce_generic > 0 {
                        format!("Cast for {{{reduce_generic}}} less?")
                    } else {
                        "Cast without paying?".to_string()
                    },
                    kind: OptionalKind::CastFree,
                }),
                DecisionAnswer::Bool(true)
            ),
        };
        if !yes {
            return Ok(());
        }
        // Auto-pick a target for the freshly-cast spell. Targets
        // are picked from the controller's perspective (avoiding
        // the cast card itself).
        let card_def = self
            .find_card_anywhere(card_id)
            .map(|c| c.definition.arc());
        let Some(card_def) = card_def else { return Ok(()); };
        // CR 601.2c — an untargeted spell names no target. The auto-picker
        // guesses one for a mass effect too, and a stale `target` on the
        // stack item made Devastation Tide a "spell with a single target"
        // for Radiant Performer (an 8-seat pod copied it 431 times).
        let auto_target = if card_def.effect.requires_target() {
            self.auto_target_for_effect_avoiding(&card_def.effect, ctx.controller, Some(card_id))
        } else {
            None
        };
        // CR 707.12 — `copy` casts a materialized copy (the original
        // stays put); the copy ceases to exist off the stack.
        let cast_id = if *copy {
            let copy_id = self.next_id();
            let inst = crate::card::CardInstance::new(
                copy_id,
                (*card_def).clone(),
                ctx.controller,
            );
            self.players[ctx.controller].hand.push(inst);
            copy_id
        } else {
            card_id
        };
        // "That copy costs {N} less to cast" — pay the discounted
        // cost up front; an unaffordable discount declines the cast.
        // Mana actually charged, threaded to `finalize_cast` — a
        // `pay_own_cost` / discounted cast here spends real mana, and
        // reporting 0 hides it from Increment, the Opus "five or more
        // mana" shapes and expend. See
        // `cast_card_from_zone_spending`.
        let mut mana_spent = 0u32;
        if *reduce_generic > 0 || *pay_own_cost || cost_override.is_some() {
            let mut discounted = cost_override.cloned().unwrap_or_else(|| card_def.cost.clone());
            // A paid cast is a cast: the caster's taxes and discounts
            // apply (Sproutback Trudge's life-gained discount, a
            // Thalia on the table).
            if let Some(card) = self.find_card_anywhere(cast_id) {
                let zone = if *copy { crate::card::Zone::Hand } else { *source_zone };
                self.add_spell_taxes(ctx.controller, card, auto_target.as_ref(), &mut discounted);
                let less = crate::game::actions::cost_reduction_for_spell_full(
                    self,
                    ctx.controller,
                    card,
                    auto_target.as_ref(),
                    zone == crate::card::Zone::Graveyard,
                    zone == crate::card::Zone::Exile,
                );
                discounted.reduce_generic(less);
                crate::game::actions::apply_colored_cost_statics(self, ctx.controller, card, &mut discounted);
            }
            discounted.reduce_generic(*reduce_generic);
            crate::game::actions::apply_spell_cost_floor(self, &mut discounted);
            let forced_only = self.seat_prompts(ctx.controller);
            match self.try_pay_with_auto_tap_mode(ctx.controller, &discounted, forced_only) {
                Ok(receipt) => {
                    mana_spent = receipt
                        .pool_before
                        .total()
                        .saturating_sub(self.players[ctx.controller].mana_pool.total());
                    self.pay_life_cost(ctx.controller, receipt.side_effects.life_lost);
                }
                Err(_) => {
                    if *copy {
                        self.players[ctx.controller].hand.retain(|c| c.id != cast_id);
                    }
                    return Ok(());
                }
            }
        }
        // Paying can take the card with it — a life payment that
        // drops the controller out of a pod removes every card they
        // own (CR 800.4a; pod seed 25048, Emet-Selch recasting Rite
        // of Replication). The cast then has nothing to cast.
        let cast_zone = if *copy { crate::card::Zone::Hand } else { *source_zone };
        if self.find_card_zone(cast_id) != Some(cast_zone) {
            return Ok(());
        }
        let cast_events = self.cast_card_from_zone_spending(
            ctx.controller,
            cast_id,
            cast_zone,
            auto_target,
            vec![],
            None,
            None,
            *exile_after,
            mana_spent,
        );
        let cast_events = match cast_events {
            Ok(evs) => evs,
            Err(e) => {
                if *copy {
                    // Unmaterialize the rejected copy.
                    self.players[ctx.controller].hand.retain(|c| c.id != cast_id);
                    return Ok(());
                }
                return Err(e);
            }
        };
        if *copy {
            for item in self.stack.iter_mut().rev() {
                if let crate::game::types::StackItem::Spell { card, .. } = item
                    && card.id == cast_id
                {
                    card.is_token = true;
                    break;
                }
            }
        }
        events.extend(cast_events);
        Ok(())
    }
}
