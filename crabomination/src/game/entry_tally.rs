//! The per-turn entry tallies for a permanent reaching the battlefield
//! without a zone-move effect: a resolving permanent spell (CR 608.3) and a
//! persist / undying return. The token and effect-move paths count their own
//! entrants; these two only emitted `PermanentEntered`, whose funnel stamps
//! the creature log but not the counters, so Celebration ("two or more
//! nonland permanents entered the battlefield under your control this turn")
//! saw tokens only — two creature spells never turned Goddric on.

use super::GameState;

impl GameState {
    /// Count `card_id` (already on the battlefield) toward its controller's
    /// nonland-permanent and artifact entry tallies.
    pub(crate) fn tally_permanent_entry(&mut self, card_id: crate::card::CardId) {
        let Some(c) = self.battlefield_find(card_id) else { return };
        let (p, land, artifact) = (c.controller, c.definition.is_land(), self.computed_has_card_type(c, crate::card::CardType::Artifact));
        let food = c.definition.subtypes.artifact_subtypes.contains(&crate::card::ArtifactSubtype::Food);
        let kinds = entry_kinds(&c.definition);
        if artifact {
            self.players[p].artifacts_entered_this_turn += 1;
        }
        if food {
            self.players[p].foods_entered_this_turn += 1;
        }
        if !land {
            self.players[p].nonland_permanents_entered_this_turn += 1;
        }
        self.apply_kind_entry(p, kinds);
    }

    /// The subtype-keyed entry tallies every entry route shares (a spell
    /// resolving, a zone move, a token): Mounts / Vehicles and Equipment /
    /// Spacecraft. One helper, so a route can't count one and miss another
    /// (a resolving Vehicle spell missed Cloudspire Coordinator's count).
    pub(crate) fn tally_kind_entry(&mut self, p: usize, def: &crate::card::CardDefinition) {
        self.apply_kind_entry(p, entry_kinds(def));
    }

    fn apply_kind_entry(&mut self, p: usize, (mount_or_vehicle, equipment_or_spacecraft): (bool, bool)) {
        if mount_or_vehicle {
            self.players[p].mounts_vehicles_entered_this_turn += 1;
        }
        if equipment_or_spacecraft {
            self.players[p].equipment_spacecraft_entered_this_turn += 1;
        }
    }
}

/// (Mount or Vehicle, Equipment or Spacecraft) — off the printed definition.
fn entry_kinds(def: &crate::card::CardDefinition) -> (bool, bool) {
    use crate::card::ArtifactSubtype as A;
    (
        def.has_creature_type(crate::card::CreatureType::Mount) || def.is_vehicle(),
        def.subtypes.artifact_subtypes.iter().any(|t| matches!(t, A::Equipment | A::Spacecraft)),
    )
}
