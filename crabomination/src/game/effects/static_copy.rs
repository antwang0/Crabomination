//! CR 707.2 / 614.1c — "creatures you control enter as a copy of X", the
//! static form of `enters_as_copy` (Infinite Reflection).

use crate::card::CardId;
use crate::effect::{Effect, Selector, StaticEffect};
use crate::game::effects::EffectContext;
use crate::game::types::GameEvent;
use crate::game::{GameState, Target};

impl GameState {
    /// CR 707.2 — every card in every graveyard matching `filter`, as
    /// `(id, name, printed power)`: the copy sources of an `EntersAsCopy`
    /// with `from_graveyards` (Body Double's "any creature card in a
    /// graveyard"). Evaluated against the card, so a type filter reads its
    /// printed characteristics.
    pub(crate) fn graveyard_copy_candidates(
        &self,
        filter: &crate::card::SelectionRequirement,
        controller: usize,
    ) -> Vec<(CardId, String, i32)> {
        self.players
            .iter()
            .flat_map(|p| p.graveyard.iter())
            .filter(|c| self.evaluate_requirement_on_card(filter, c, controller))
            .map(|c| (c.id, c.definition.name.to_string(), c.definition.power))
            .collect()
    }

    /// Apply a `StaticEffect::CreaturesEnterAsCopyOf` that `controller`'s
    /// permanents hold over the entering `card_id`. With several, the last one
    /// found is applied last and wins (CR 616.1 — the controller's order).
    pub(crate) fn apply_static_enters_as_copy(
        &mut self,
        card_id: CardId,
        controller: usize,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        let mut specs: Vec<(CardId, Selector)> = Vec::new();
        for src in self.battlefield.iter() {
            if src.controller != controller || src.id == card_id {
                continue;
            }
            for sa in src.definition.static_abilities.iter() {
                if let StaticEffect::CreaturesEnterAsCopyOf { filter, of } = &sa.effect
                    && self.evaluate_requirement_static(
                        filter,
                        &Target::Permanent(card_id),
                        controller,
                        Some(src.id),
                    )
                {
                    specs.push((src.id, of.clone()));
                }
            }
        }
        let mut model = None;
        for (src, of) in specs {
            let ctx = EffectContext::for_trigger(src, controller, None, 0);
            if let Some(id) = self
                .resolve_selector(&of, &ctx)
                .into_iter()
                .filter_map(|e| e.as_permanent_id())
                .find(|&id| id != card_id)
            {
                model = Some(id);
            }
        }
        let Some(model) = model else { return false };
        let ctx = EffectContext::for_trigger(card_id, controller, Some(Target::Permanent(model)), 0);
        if let Ok(evs) = self.resolve_effect_driven(
            &Effect::BecomeCopyOf {
                what: Selector::This,
                source: Selector::Target(0),
                extra_creature_types: vec![],
                keep_own_triggered: false,
                keep_own_activated: false,
                keep_name: false,
            },
            &ctx,
        ) {
            events.extend(evs);
        }
        true
    }

    /// Mystic Reflection's armed copy (CR 707.2, 614.12): an entering creature
    /// or planeswalker becomes a copy of the model this turn, in the first
    /// batch only. A model that left is copied as it last existed (CR 608.2h).
    pub(crate) fn apply_next_entries_copy(
        &mut self,
        card_id: CardId,
        controller: usize,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        let Some(armed) = self.next_entries_copy.clone() else { return false };
        if armed.turn != self.turn_number || armed.model == card_id {
            return false;
        }
        let Some(c) = self.battlefield.find_by_id(card_id) else { return false };
        if !(self.computed_is_creature(c) || self.computed_has_card_type(c, crate::card::CardType::Planeswalker)) {
            return false;
        }
        if self.battlefield.find_by_id(armed.model).is_none() {
            let Some(lki) = armed.lki else { return false };
            self.stamp_copy(card_id, &lki, &[], false, false, false);
            if let Some(n) = self.next_entries_copy.as_mut() {
                n.applied = true;
            }
            return true;
        }
        let ctx = EffectContext::for_trigger(card_id, controller, Some(Target::Permanent(armed.model)), 0);
        if let Ok(evs) = self.resolve_effect_driven(
            &Effect::BecomeCopyOf {
                what: Selector::This,
                source: Selector::Target(0),
                extra_creature_types: vec![],
                keep_own_triggered: false,
                keep_own_activated: false,
                keep_name: false,
            },
            &ctx,
        ) {
            events.extend(evs);
        }
        if let Some(n) = self.next_entries_copy.as_mut() {
            n.applied = true;
        }
        true
    }

    /// CR 707.2 — `cid` (a permanent) becomes a copy of `src_def`, the
    /// copiable values `Effect::BecomeCopyOf` read; instance state stays.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn stamp_copy(
        &mut self,
        cid: CardId,
        src_def: &std::sync::Arc<crate::card::CardDefinition>,
        extra_creature_types: &[crate::card::CreatureType],
        keep_own_triggered: bool,
        keep_own_activated: bool,
        keep_name: bool,
    ) {
        // "Except it has this ability" keeps the copier's
        // *printed* abilities — the definition before its
        // first copy — not whatever it last copied. Reading
        // the current definition re-appended the kept
        // abilities on every re-copy, so an Aurora Shifter
        // copying an Aurora Shifter doubled its trigger list
        // each turn until the process ran out of memory.
        let printed = self
            .temporary_copies
            .iter()
            .find(|t| t.card == cid)
            .and_then(|t| t.original.clone());
        if let Some(c) = self.battlefield.find_by_id_mut(cid) {
            let mut new_def = (**src_def).clone();
            for t in extra_creature_types {
                if !new_def.subtypes.creature_types.contains(t) {
                    new_def.subtypes.creature_types.push(*t);
                }
            }
            let own = printed.unwrap_or_else(|| c.definition.arc());
            if keep_own_triggered {
                for t in &own.triggered_abilities {
                    if !new_def.triggered_abilities.contains(t) {
                        new_def.triggered_abilities.push(t.clone());
                    }
                }
            }
            if keep_own_activated {
                for a in &own.activated_abilities {
                    if !new_def.activated_abilities.contains(a) {
                        new_def.activated_abilities.push(a.clone());
                    }
                }
            }
            if keep_name {
                new_def.name = own.name;
            }
            // CR 708.10 — a face-down permanent that becomes a
            // copy keeps the face-down characteristics; only its
            // copiable values (what it turns up as) change.
            if c.face_down {
                c.face_up_def = Some(std::sync::Arc::new(new_def));
                return;
            }
            // CR 123.6c — the copier's own name stickers stay
            // on over its new copiable values.
            let original = c.copiable_definition();
            c.set_copiable_definition(std::sync::Arc::new(new_def));
            // CR 400.7 — in its next zone the object is its
            // printed card again; `revert_copy_on_leave`
            // restores this (a dead Clone is a Clone in the
            // graveyard, so Vizier's embalm stays available).
            self.temporary_copies.push(crate::game::TempCopy {
    until_turn_of: None,
    shapeshifter: false,
                card: cid,
                original_name: original.name.to_string(),
                original: Some(original),
                duration: crate::effect::Duration::Permanent,
                source: None,
            });
        }
    }

    /// CR 606.3 override — `seat` holds an emblem letting them activate
    /// loyalty abilities at instant speed (Teferi, Temporal Archmage).
    pub(crate) fn loyalty_at_instant_speed(&self, seat: usize) -> bool {
        self.players.get(seat).is_some_and(|p| {
            p.emblems.iter().any(|e| {
                e.statics
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::LoyaltyAbilitiesAtInstantSpeed))
            })
        })
    }
}
