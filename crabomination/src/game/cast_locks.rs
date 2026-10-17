//! CR 101.2 / 601.2 — the "can't cast" locks a spell meets whatever zone it is
//! cast from. The hand path asks them around its timing gate; the zone paths
//! (a commander from the command zone) share these, so a lock can't be walked
//! around by casting from somewhere other than a hand.

use super::GameState;
use super::actions::cast_static;
use crate::card::CardInstance;
use crate::game::GameError;

impl GameState {
    /// The spell-name locks (Meddling Mage, Ashiok's Erasure, Circu, Academic
    /// Probation): `true` when `p` can't cast a spell named `name`.
    pub(crate) fn spell_name_locked(&self, p: usize, name: &str) -> bool {
        // Meddling Mage — spells with the chosen name can't be cast.
        if self.battlefield.iter().any(|c| {
            c.named_card
                .as_deref()
                .is_some_and(|n| crate::card::name_has(name, n))
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::NamedSpellCantBeCast)
                })
        }) {
            return true;
        }
        // Ashiok's Erasure — an opponent of the caster controls a permanent
        // whose `OpponentsCantCastNamed` static locks this spell's name (the
        // card exiled by the Erasure). The lock lives as long as that
        // permanent stays on the battlefield.
        if self.battlefield.iter().any(|c| {
            c.named_card
                .as_deref()
                .is_some_and(|n| crate::card::name_has(name, n))
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        sa.effect,
                        crate::effect::StaticEffect::OpponentsCantCastNamed
                    )
                })
                && !self.same_team(c.controller, p)
        }) {
            return true;
        }
        // Circu, Dimir Lobotomist — an opponent of the caster controls a
        // permanent, and a card exiled with it shares this spell's name.
        if self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| {
                matches!(
                    sa.effect,
                    crate::effect::StaticEffect::OpponentsCantCastNamesExiledWithSource
                )
            }) && !self.same_team(c.controller, p)
                && self
                    .exile
                    .iter()
                    .any(|e| e.exiled_with == Some(c.id) && e.definition.name == name)
        }) {
            return true;
        }
        // Academic Probation mode 0 — an opponent of the caster named this
        // spell ("Opponents can't cast spells with the chosen name until your
        // next turn"); the lock lives on the naming player until their turn.
        if self.players.iter().enumerate().any(|(i, pl)| {
            !self.same_team(i, p)
                && pl
                    .opponents_cant_cast_named
                    .iter()
                    .any(|n| crate::card::name_has(name, n))
        }) {
            return true;
        }
        false
    }

    /// The player and board locks on `card` itself (Ranger-Captain of Eos,
    /// Cease-Fire, Living Breakthrough, City in a Bottle, Llawan, Codie, Gaddock
    /// Teeg, Nikya, Hymn of the Wilds, Hand to Hand, Grid Monitor). `Some(e)`
    /// rejects. `cost_statics` is `cast_cost_scan`'s mask (`u32::MAX` asks
    /// every gated block).
    pub(crate) fn cast_lock_for(
        &self,
        p: usize,
        card: &CardInstance,
        cost_statics: u32,
    ) -> Option<GameError> {
        // Ranger-Captain of Eos lock — this player can't cast noncreature
        // spells for the rest of the turn.
        if self.players[p].cant_cast_noncreature_this_turn && !card.definition.is_creature() {
            return Some(GameError::CantCastNoncreature);
        }

        // Cease-Fire — a turn-scoped, filtered cast lock.
        if !self.players[p].cant_cast_matching_this_turn.is_empty() {
            let locks = self.players[p].cant_cast_matching_this_turn.clone();
            if locks
                .iter()
                .any(|f| self.evaluate_requirement_on_card(f, card, p))
            {
                return Some(GameError::CantCastNoncreature);
            }
        }

        // Living Breakthrough — an opponent locked this spell's mana value
        // until their next turn.
        if self.players.iter().enumerate().any(|(i, pl)| {
            !pl.opponents_cant_cast_mv.is_empty()
                && !self.same_team(i, p)
                && pl
                    .opponents_cant_cast_mv
                    .contains(&card.definition.cost.cmc())
        }) {
            return Some(GameError::SpellNameLocked);
        }

        // City in a Bottle — a symmetric play-lock binds every seat.
        if self.play_locked_for_all(p, card) {
            return Some(GameError::SpellNameLocked);
        }

        // Llawan — an opponent's static locks a whole class of spells.
        if self.opponent_locks_cast_of(p, card) {
            return Some(GameError::CantCastNoncreature);
        }

        // Codie lock — this player can't cast permanent spells.
        if card.definition.is_permanent() && self.player_cant_cast_permanent_spells(p) {
            return Some(GameError::CantCastPermanentSpells);
        }

        // Gaddock Teeg lock — high-mana-value / {X} noncreature spells can't be cast.
        debug_assert!(
            cost_statics & cast_static::NONCREATURE_LOCK != 0
                || !self.noncreature_spell_cast_locked(&card.definition),
            "cast_cost_scan missed a noncreature-cast lock",
        );
        if cost_statics & cast_static::NONCREATURE_LOCK != 0
            && self.noncreature_spell_cast_locked(&card.definition)
        {
            return Some(GameError::CantCastNoncreature);
        }

        // Nikya of the Old Ways — its controller can't cast noncreature spells.
        if !card.definition.is_creature() && self.player_cant_cast_noncreature_spells(p) {
            return Some(GameError::CantCastNoncreature);
        }

        // Hymn of the Wilds — its controller can't cast instants or sorceries.
        if (card.definition.is_instant() || card.definition.is_sorcery())
            && self.player_cant_cast_instants_or_sorceries(p)
        {
            return Some(GameError::CantCastNoncreature);
        }

        // Hand to Hand — nobody casts instants during combat.
        if card.definition.is_instant() && self.combat_spell_lock_active() {
            return Some(GameError::CantCastNoncreature);
        }

        // Grid Monitor — its controller can't cast creature spells.
        if card.definition.is_creature() && self.player_cant_cast_creature_spells(p) {
            return Some(GameError::CantCastNoncreature);
        }
        None
    }
}
