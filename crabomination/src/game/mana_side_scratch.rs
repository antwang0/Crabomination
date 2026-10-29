//! CR 605.3a — a mana ability doesn't use the stack: activated while another
//! spell or ability is resolving (auto-tap paying a "you may pay {1}" in the
//! middle of an effect) or while one is parked on a decision, it is not a new
//! resolution of that object. Its activation resets the cost scratch and its
//! resolution resets the per-resolution scratch, so Ripples of Undeath paid
//! from a Mind Stone lost "milled this way" and returned nothing. The outer
//! resolution's values are saved around it and put back.

use crate::card::CardId;
use crate::game::GameState;

impl GameState {
    /// A printed mana ability activated inside a resolution, or under a
    /// parked decision: the activation must leave that resolution's scratch
    /// as it found it.
    pub(crate) fn mana_ability_inside_a_resolution(&self, card_id: CardId, ability_index: usize) -> bool {
        (self.resolution_depth > 0 || self.pending_decision.is_some())
            && self
                .battlefield_find(card_id)
                .and_then(|c| c.definition.activated_abilities.get(ability_index))
                .is_some_and(|a| crate::game::actions::is_mana_ability(&a.effect))
    }

    /// Run `f`, then restore every field the activation path and the
    /// resolution entry reset (`resolve_effect_into_kind`'s reset block, the
    /// land tap's cost scratch) and the resolution stamps.
    pub(crate) fn keeping_outer_resolution_scratch<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let direct = (
            self.sacrificed_power,
            self.sacrificed_total_power,
            self.sacrificed_count,
            self.sacrificed_toughness,
            self.sacrificed_mana_value,
            self.last_discarded_mana_value,
            self.last_revealed_from_hand,
            self.tapped_for_cost_power,
            self.last_created_token,
            self.chosen_creature_type_scratch,
            self.cards_discarded_this_resolution,
            self.cards_drawn_this_resolution,
            self.energy_paid_this_resolution,
            self.permanents_returned_this_resolution,
            self.permanents_tapped_this_resolution,
            self.cards_revealed_this_resolution,
            self.creature_cards_discarded_this_resolution,
            self.greatest_discarded_mv_this_resolution,
            self.permanents_destroyed_this_resolution,
            self.excess_damage_this_resolution,
            self.creatures_died_this_resolution,
            self.damage_dealt_this_resolution,
            self.countered_spell_mana_spent,
            self.countered_spell_controller,
            self.counters_removed_this_effect,
            self.countered_spell_mana_value,
            self.exiled_for_cost_mana_value,
            self.counters_removed_as_cost,
            self.cost_discarded_mana_value,
            self.resolution_causer,
            self.source_name_scratch,
            self.token_minting_source,
        );
        let cold = (self.tapped_for_cost.clone(), self.cost_exiled_cards.clone(), self.ench_creature_etb_bonus.clone());
        // The scratch group is copy-on-write: hold the old handle and copy the
        // listed fields back only if the activation unshared it.
        let scratch = self.scratch.clone();
        let out = f(self);
        (
            self.sacrificed_power,
            self.sacrificed_total_power,
            self.sacrificed_count,
            self.sacrificed_toughness,
            self.sacrificed_mana_value,
            self.last_discarded_mana_value,
            self.last_revealed_from_hand,
            self.tapped_for_cost_power,
            self.last_created_token,
            self.chosen_creature_type_scratch,
            self.cards_discarded_this_resolution,
            self.cards_drawn_this_resolution,
            self.energy_paid_this_resolution,
            self.permanents_returned_this_resolution,
            self.permanents_tapped_this_resolution,
            self.cards_revealed_this_resolution,
            self.creature_cards_discarded_this_resolution,
            self.greatest_discarded_mv_this_resolution,
            self.permanents_destroyed_this_resolution,
            self.excess_damage_this_resolution,
            self.creatures_died_this_resolution,
            self.damage_dealt_this_resolution,
            self.countered_spell_mana_spent,
            self.countered_spell_controller,
            self.counters_removed_this_effect,
            self.countered_spell_mana_value,
            self.exiled_for_cost_mana_value,
            self.counters_removed_as_cost,
            self.cost_discarded_mana_value,
            self.resolution_causer,
            self.source_name_scratch,
            self.token_minting_source,
        ) = direct;
        if self.tapped_for_cost != cold.0 {
            self.tapped_for_cost = cold.0;
        }
        if self.cost_exiled_cards != cold.1 {
            self.cost_exiled_cards = cold.1;
        }
        if self.ench_creature_etb_bonus != cold.2 {
            self.ench_creature_etb_bonus = cold.2;
        }
        if !self.scratch.shares_with(&scratch) {
            let s = &mut self.scratch;
            s.treasure_mana_spent_to_activate = scratch.treasure_mana_spent_to_activate;
            s.times_paid_this_effect = scratch.times_paid_this_effect;
            s.discarded_for_cost_card = scratch.discarded_for_cost_card;
            s.last_created_tokens = scratch.last_created_tokens.clone();
            s.last_moved_cards = scratch.last_moved_cards.clone();
            s.proliferated_this_resolution = scratch.proliferated_this_resolution.clone();
            s.surveil_kept_top_this_resolution = scratch.surveil_kept_top_this_resolution;
            s.cards_discarded_per_player_this_resolution = scratch.cards_discarded_per_player_this_resolution.clone();
            s.nonland_cards_discarded_per_player_this_resolution =
                scratch.nonland_cards_discarded_per_player_this_resolution.clone();
            s.discarded_card_ids_this_resolution = scratch.discarded_card_ids_this_resolution.clone();
            s.cast_immediately_this_resolution = scratch.cast_immediately_this_resolution;
            s.exiled_card_ids_this_resolution = scratch.exiled_card_ids_this_resolution.clone();
            s.destroyed_this_resolution = scratch.destroyed_this_resolution.clone();
            s.destroyed_controllers_this_resolution = scratch.destroyed_controllers_this_resolution.clone();
            s.damaged_this_resolution = scratch.damaged_this_resolution.clone();
            s.players_sacrificed_this_resolution = scratch.players_sacrificed_this_resolution.clone();
            s.cards_sacrificed_this_resolution = scratch.cards_sacrificed_this_resolution.clone();
            s.first_sacrificed_types = scratch.first_sacrificed_types.clone();
            s.sacrificed_lki_this_resolution = scratch.sacrificed_lki_this_resolution.clone();
            s.named_card_this_resolution = scratch.named_card_this_resolution.clone();
            s.token_replacement_pick = scratch.token_replacement_pick;
            s.mirrormind_active = scratch.mirrormind_active;
            s.names_this_resolution = scratch.names_this_resolution.clone();
            s.cost_sacrificed_batch = scratch.cost_sacrificed_batch.clone();
        }
        out
    }
}
