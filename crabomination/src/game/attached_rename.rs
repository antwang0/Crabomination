//! Psychic Paper — "as this becomes attached to a creature, choose a creature
//! card name and a creature type; equipped creature's name and creature type
//! are the last chosen" (`StaticEffect::AttachedTakesChosenNameAndType`).
//!
//! The host's name and creature type are copiable values rewritten in place
//! (CR 707.9b), recorded as a `WhileSourceAttached` temporary copy so the
//! sweep that ends Assimilation Aegis's copy restores the host the moment
//! the Equipment unattaches or leaves (CR 611.2c).

use super::{GameState, TempCopy};
use crate::card::{CardId, SelectionRequirement};
use crate::effect::{Duration, Effect, Selector};
use crate::game::effects::EffectContext;

impl GameState {
    pub(crate) fn rename_attached_host(&mut self, equipment: CardId, host: CardId) {
        let Some(controller) = self.battlefield_find(equipment).map(|c| c.controller) else { return };
        if !self.battlefield_find(host).is_some_and(|h| self.computed_is_creature(h)) {
            return;
        }
        let ctx = EffectContext::for_ability(equipment, controller, None);
        // The two choices, stamped on the Equipment. A headless seat names
        // nothing (the host keeps its own name, itself a creature card name)
        // and takes its own deck's most common creature type — the Auto
        // decider's blanket answer for "choose a creature type" would strip a
        // Doctor of its tribe.
        let _ = self.resolve_effect(
            &Effect::NameCard { what: Selector::This, restrict_to: Some(SelectionRequirement::Creature) },
            &ctx,
        );
        if self.seat_prompts(controller) || !matches!(self.decider.kind(), crate::decision::DeciderKind::Auto) {
            let _ = self.resolve_effect(&Effect::NameCreatureType { what: Selector::This }, &ctx);
        } else {
            let pick = self.own_most_common_creature_type(controller, host);
            if let Some(p) = self.battlefield.find_by_id_mut(equipment) {
                p.chosen_creature_type = pick;
            }
        }
        let Some(paper) = self.battlefield_find(equipment) else { return };
        let name = paper.named_card.clone();
        let creature_type = paper.chosen_creature_type;
        if name.is_none() && creature_type.is_none() {
            return;
        }
        // A re-attach to the same host replaces the earlier rename.
        if let Some(pos) =
            self.temporary_copies.iter().position(|tc| tc.card == host && tc.source == Some(equipment))
            && let Some(def) = self.temporary_copies[pos].original.clone()
        {
            self.temporary_copies.remove(pos);
            if let Some(h) = self.battlefield.find_by_id_mut(host) {
                h.set_definition(def);
            }
        }
        let Some(h) = self.battlefield.find_by_id_mut(host) else { return };
        let original = h.copiable_definition();
        let mut def = (*original).clone();
        if let Some(n) = name {
            def.name = crate::static_str_serde::intern(n);
        }
        if let Some(t) = creature_type {
            def.subtypes.creature_types = vec![t];
        }
        h.set_copiable_definition(std::sync::Arc::new(def));
        self.temporary_copies.push(TempCopy {
            card: host,
            original_name: original.name.to_string(),
            original: Some(original),
            duration: Duration::WhileSourceAttached,
            source: Some(equipment),
            shapeshifter: false,
            until_turn_of: None,
        });
    }

    /// The creature type most common among the cards `seat` owns (board,
    /// hand, library, graveyard); ties and an empty count go to `host`'s own
    /// first type.
    fn own_most_common_creature_type(&self, seat: usize, host: CardId) -> Option<crate::card::CreatureType> {
        let host_first = self.battlefield_find(host).and_then(|h| h.definition.subtypes.creature_types.first().copied());
        let p = self.players.get(seat)?;
        let mut counts: Vec<(crate::card::CreatureType, usize)> = Vec::new();
        let cards = self
            .battlefield
            .iter()
            .filter(|c| c.owner == seat)
            .chain(p.hand.iter())
            .chain(p.library.iter())
            .chain(p.graveyard.iter());
        for c in cards {
            for &t in &c.definition.subtypes.creature_types {
                match counts.iter_mut().find(|(k, _)| *k == t) {
                    Some((_, n)) => *n += 1,
                    None => counts.push((t, 1)),
                }
            }
        }
        let best = counts.iter().map(|(_, n)| *n).max()?;
        if host_first.is_some_and(|h| counts.iter().any(|(k, n)| *k == h && *n == best)) {
            return host_first;
        }
        counts.iter().find(|(_, n)| *n == best).map(|(k, _)| *k).or(host_first)
    }
}
