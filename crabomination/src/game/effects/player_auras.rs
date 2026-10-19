//! CR 303.4 — Auras attached to players (the Curse family): searching one
//! onto the battlefield enchanting a player, returning one from a graveyard
//! enchanting a player, moving them between players, and Curse of Echoes'
//! table-wide copy offer.

use super::EffectContext;
use crate::card::{CardId, SelectionRequirement};
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// The `filter` permanents attached to `seat` (CR 303.4).
    pub(crate) fn auras_attached_to_player(
        &self,
        seat: usize,
        filter: &SelectionRequirement,
        viewer: usize,
    ) -> Vec<CardId> {
        self.battlefield
            .iter()
            .filter(|c| c.attached_to_player == Some(seat))
            .filter(|c| self.evaluate_requirement_on_card(filter, c, viewer))
            .map(|c| c.id)
            .collect()
    }

    /// Put `card` (already out of its zone) onto the battlefield under
    /// `controller`'s control, enchanting `seat`.
    fn enter_attached_to_player(
        &mut self,
        card: crate::card::CardInstance,
        controller: usize,
        seat: usize,
        events: &mut Vec<GameEvent>,
    ) {
        let id = card.id;
        let dest = ZoneDest::Battlefield { controller: PlayerRef::Seat(controller), tapped: false };
        self.place_card_in_dest(card, controller, &dest, events);
        if let Some(c) = self.battlefield_find_mut(id) {
            c.attached_to = None;
            c.attached_to_player = Some(seat);
            events.push(GameEvent::AttachmentMoved { attachment: id, attached_to: None });
            self.scratch.last_moved_cards.push(id);
        }
    }

    /// `Effect::ReturnToBattlefieldAttachedToPlayer`.
    pub(super) fn return_to_battlefield_attached_to_player(
        &mut self,
        what: &Selector,
        host: &PlayerRef,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(seat) = self.resolve_player(host, ctx) else { return Ok(()) };
        if !self.players[seat].is_alive() {
            return Ok(());
        }
        let ids: Vec<CardId> = self.resolve_selector(what, ctx).into_iter().filter_map(|e| e.as_card_id()).collect();
        for id in ids {
            let Some(owner) = self.players.iter().position(|pl| pl.graveyard.iter().any(|c| c.id == id)) else {
                continue;
            };
            let Some(card) = Self::take_card(&mut self.players[owner].graveyard, id) else { continue };
            self.note_left_graveyard(owner, id, events);
            self.enter_attached_to_player(card, ctx.controller, seat, events);
        }
        Ok(())
    }

    /// `Effect::SearchAttachToPlayer`.
    pub(super) fn search_attach_to_player(
        &mut self,
        filter: &SelectionRequirement,
        host: &PlayerRef,
        distinct_names: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let Some(seat) = self.resolve_player(host, ctx) else { return Ok(()) };
        if !self.players[seat].is_alive() {
            return Ok(());
        }
        let taken: Vec<&'static str> = if distinct_names {
            self.battlefield
                .iter()
                .filter(|c| c.attached_to_player == Some(seat))
                .map(|c| c.definition.name)
                .collect()
        } else {
            Vec::new()
        };
        // CR 701.23 — the searcher finds the card; headless, the priciest.
        let pick = self.players[me]
            .library
            .iter()
            .filter(|c| self.off_battlefield_card_matches(filter, c, me))
            .filter(|c| !taken.contains(&c.definition.name))
            .max_by_key(|c| c.definition.cost.cmc())
            .map(|c| c.id);
        if let Some(id) = pick
            && let Some(card) = Self::take_card(&mut self.players[me].library, id)
        {
            self.enter_attached_to_player(card, me, seat, events);
        }
        self.shuffle_library(me, events);
        Ok(())
    }

    /// `Effect::ReattachPlayerAurasToOpponents`.
    pub(super) fn reattach_player_auras_to_opponents(
        &mut self,
        from: &PlayerRef,
        filter: &SelectionRequirement,
        all: bool,
        then: Option<&Effect>,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let Some(seat) = self.resolve_player(from, ctx) else { return Ok(()) };
        let mut auras = self.auras_attached_to_player(seat, filter, me);
        if !all {
            auras.truncate(1);
        }
        let mut moved = false;
        for aura in auras {
            // CR 303.4d — an Aura can't enchant a player it couldn't; the
            // opponent of choice (headless: the highest life total) can't
            // be the player it already enchants.
            let to = (0..self.players.len())
                .filter(|&p| p != seat && self.players[p].is_alive() && !self.same_team(p, me))
                .max_by_key(|&p| (self.players[p].life, std::cmp::Reverse(p)));
            let Some(to) = to else { break };
            if let Some(c) = self.battlefield_find_mut(aura) {
                c.attached_to_player = Some(to);
                events.push(GameEvent::AttachmentMoved { attachment: aura, attached_to: None });
                moved = true;
            }
        }
        if moved && let Some(then) = then {
            self.run_effect(then, ctx, events)?;
        }
        Ok(())
    }

    /// `Effect::EachOtherPlayerMayCopy`.
    pub(super) fn each_other_player_may_copy(
        &mut self,
        what: &Selector,
        except: &PlayerRef,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        use crate::decision::{Decision, DecisionAnswer, OptionalKind};
        let skip = self.resolve_player(except, ctx);
        let spell = match what {
            Selector::TriggerSource => ctx.trigger_source.and_then(|e| e.as_card_id()),
            _ => self.resolve_selector(what, ctx).into_iter().find_map(|e| e.as_card_id()),
        };
        let Some(spell) = spell else { return Ok(()) };
        let source = ctx.source.unwrap_or(CardId(0));
        for seat in self.seats_in_turn_order_from(self.active_player_idx) {
            if Some(seat) == skip || !self.players[seat].is_alive() {
                continue;
            }
            // A headless seat copies (a free spell of its choosing); a
            // scripted decider answers yes/no.
            let yes = matches!(self.decider.kind(), crate::decision::DeciderKind::Auto)
                || matches!(
                    self.decider.decide(&Decision::OptionalTrigger {
                        source,
                        description: "Copy that spell?".to_string(),
                        kind: OptionalKind::FreeUpside,
                    }),
                    DecisionAnswer::Bool(true)
                );
            if yes {
                self.copy_stack_spell_controlled(spell, 1, true, Some(seat), None, events);
            }
        }
        Ok(())
    }
}
