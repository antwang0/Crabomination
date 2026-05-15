//! Auto-target picker for effects that the engine resolves without explicit
//! user input (ETB triggers, attack triggers, bot-driven casts).

use crate::card::CardId;
use crate::effect::Effect;
use crate::game::{GameState, Target};

impl GameState {
    /// Pick a legal target for an effect that requires one, used when the
    /// engine fires a trigger without explicit user input (ETB, attack trigger,
    /// etc.). Returns `None` if the effect requires no target or no legal
    /// target exists.
    ///
    /// Targets must satisfy *both* the effect's selector requirement AND
    /// targeting legality (Hexproof / Shroud / Protection / player-side
    /// Leyline of Sanctity). Without the legality gate the random bot
    /// happily picks an opponent's Hexproof creature, the cast is
    /// rejected by `cast_spell`, and (in spectate mode) the match
    /// deadlocks — see `debug/deadlock-t10-1777412787-934831200.json`,
    /// where the bot kept aiming Bone Shards at Sylvan Caryatid.
    pub fn auto_target_for_effect(&self, eff: &Effect, controller: usize) -> Option<Target> {
        self.auto_target_for_effect_avoiding(eff, controller, None)
    }

    /// Source-aware auto-target picker. When `avoid_source` is set, the
    /// returned target prefers any *other* legal candidate to the avoided
    /// permanent — falling back to the source only if no other legal pick
    /// exists. Powers Strixhaven's Magecraft/Repartee triggers where the
    /// trigger source is rarely the right pick (a 1/1 utility creature
    /// shouldn't pump itself when a 5/5 attacker is on the board).
    pub fn auto_target_for_effect_avoiding(
        &self,
        eff: &Effect,
        controller: usize,
        avoid_source: Option<CardId>,
    ) -> Option<Target> {
        let req = eff.primary_target_filter()?;
        // First opponent on a different team. Falls back to the next
        // seat in singleton-team / unknown-team cases so the legacy 1v1
        // pick (`(controller + 1) % n`) is preserved.
        let opp = self
            .opponents_of(controller)
            .first()
            .copied()
            .unwrap_or((controller + 1) % self.players.len());
        let prefer_friendly = eff.prefers_friendly_target();
        // `prefers_graveyard_target` is the broader classifier — it covers
        // both reanimate (friendly graveyard) and graveyard hate (Ghost
        // Vacuum exiling target card from a graveyard). We walk graveyards
        // BEFORE the battlefield when this is set, so an `Any`-filtered
        // Move-to-Exile doesn't grab a battlefield permanent.
        let prefer_graveyard = eff.prefers_graveyard_target();
        // Skip Player candidates entirely when the effect operates on
        // permanents/stack — without this, an `Any`-filtered Move (Regrowth)
        // auto-targets the caster as a player and silently fizzles since
        // `Effect::Move` only consumes Permanent / Card entity refs.
        let accepts_player = eff.accepts_player_target();
        let primary_player = if prefer_friendly { controller } else { opp };
        let secondary_player = if prefer_friendly { opp } else { controller };

        // Combined check: requirement match + targetable by `controller`.
        let is_legal = |t: &Target| -> bool {
            self.evaluate_requirement_static(req, t, controller, avoid_source)
                && self.check_target_legality(t, controller).is_ok()
        };

        if accepts_player {
            let player_primary = Target::Player(primary_player);
            if is_legal(&player_primary) { return Some(player_primary); }
            let player_secondary = Target::Player(secondary_player);
            if is_legal(&player_secondary) { return Some(player_secondary); }
        }

        // Graveyard-target effects: walk primary player's graveyard first,
        // then secondary's. Reanimate/Disentomb (friendly) hits the caster's
        // graveyard; Ghost Vacuum (hostile) hits the opp's. Falls through
        // to the battlefield walk below if no graveyard match.
        if prefer_graveyard {
            for &p in &[primary_player, secondary_player] {
                if let Some(c) = self.players[p]
                    .graveyard
                    .iter()
                    .map(|c| Target::Permanent(c.id))
                    .find(|t| is_legal(t))
                {
                    return Some(c);
                }
            }
        }

        // Battlefield: walk preferred-controller permanents first, then
        // any matching permanent. Without the preference, the bot would
        // happily Vines its opponent's bear instead of its own.
        //
        // Source-avoidance pass (see `auto_target_for_effect_avoiding`'s
        // doc comment): when caller asked us to avoid the trigger source,
        // skip the source on the first pass and only fall back to it if
        // no other legal candidate exists.
        let is_avoided = |cid: CardId| -> bool {
            avoid_source.map(|s| s == cid).unwrap_or(false)
        };
        // For friendly pumps (Magecraft / Repartee +1/+1 fan-out, transient
        // PumpPT spells), prefer the highest-power friendly creature so the
        // buff lands on the bot's biggest threat — improves expected value
        // versus the prior "first-in-Vec" pick (which was deterministic but
        // typically picked a 1-drop utility creature). For hostile picks the
        // current first-match heuristic still applies.
        let collect_legal_on_player = |p: usize| -> Vec<(CardId, i32)> {
            self.battlefield
                .iter()
                .filter(|c| c.controller == p)
                .filter(|c| !is_avoided(c.id))
                .filter(|c| is_legal(&Target::Permanent(c.id)))
                .map(|c| {
                    let power = self
                        .computed_permanent(c.id)
                        .map(|cp| cp.power)
                        .unwrap_or(c.definition.power);
                    (c.id, power)
                })
                .collect()
        };
        let mut primary_candidates = collect_legal_on_player(primary_player);
        if prefer_friendly && !primary_candidates.is_empty() {
            // Sort by descending power so the strongest creature wins.
            primary_candidates.sort_by_key(|c| std::cmp::Reverse(c.1));
        }
        if let Some(&(cid, _)) = primary_candidates.first() {
            return Some(Target::Permanent(cid));
        }
        if let Some(t) = self
            .battlefield
            .iter()
            .filter(|c| !is_avoided(c.id))
            .map(|c| Target::Permanent(c.id))
            .find(|t| is_legal(t))
        {
            return Some(t);
        }
        // Source-fallback: only the avoided source is a legal candidate.
        // Pick it as a last resort so the trigger doesn't fizzle entirely.
        if let Some(t) = self
            .battlefield
            .iter()
            .filter(|c| c.controller == primary_player)
            .map(|c| Target::Permanent(c.id))
            .find(|t| is_legal(t))
        {
            return Some(t);
        }
        if let Some(t) = self
            .battlefield
            .iter()
            .map(|c| Target::Permanent(c.id))
            .find(|t| is_legal(t))
        {
            return Some(t);
        }
        // Final fallback: any graveyard, then exile. Reanimate-style spells
        // (Goryo's Vengeance, Animate Dead) hit this path when their target
        // was just lifted off the prefer-graveyard branch (e.g. their
        // controller's graveyard is empty). Hexproof and friends don't
        // apply to graveyard/exile targets, but we still funnel through
        // `is_legal` so any future zone-aware legality rules pick up
        // these zones too.
        for player in &self.players {
            if let Some(c) = player
                .graveyard
                .iter()
                .map(|c| Target::Permanent(c.id))
                .find(|t| is_legal(t))
            {
                return Some(c);
            }
        }
        if let Some(c) = self
            .exile
            .iter()
            .map(|c| Target::Permanent(c.id))
            .find(|t| is_legal(t))
        {
            return Some(c);
        }
        None
    }

    /// Pick legal targets for every slot the effect uses, returning
    /// `(slot 0, Vec<slot 1..>)`.
    ///
    /// Walks the effect tree (via `target_filter_for_slot_in_mode`) and
    /// produces a `Vec<Target>` for `additional_targets`, plus an
    /// `Option<Target>` for slot 0. Each slot is filled with the
    /// best-pick legal target (per `auto_target_for_effect_avoiding`'s
    /// preferences). Slots that fail to find any legal candidate are
    /// skipped — matching the printed "up to N target" semantics where
    /// the spell still resolves with fewer (or zero) targets when no
    /// legal pick exists.
    ///
    /// The slot enumeration stops at the first slot index for which
    /// the effect tree contains no `Selector::TargetFiltered { slot }`
    /// reference. So for Snow Day (slots 0, 1), this returns up to
    /// 2 targets. For Homesickness (slots 0, 1, 2), it returns up to
    /// 3 targets. Vibrant Outburst (slots 0, 1) returns up to 2.
    ///
    /// Used by the bot harness to drive multi-target casts without
    /// surfacing a UI prompt.
    pub fn auto_targets_for_effect_all_slots(
        &self,
        eff: &Effect,
        controller: usize,
        mode: Option<usize>,
    ) -> (Option<Target>, Vec<Target>) {
        // Slot 0 — use the existing source-aware picker.
        let slot_0 = self.auto_target_for_effect_avoiding(eff, controller, None);
        let mut additional = Vec::new();
        let mut slot: u8 = 1;
        // Cap at 16 slots — no real card uses more than 4, but cap defensively.
        while slot < 16 {
            let req = match eff.target_filter_for_slot_in_mode(slot, mode) {
                Some(r) => r.clone(),
                None => break,
            };
            // Use the same hostile/friendly preference heuristics by
            // constructing a small Effect::PumpPT-style probe and calling
            // the picker against that filter. Simpler approach: walk
            // battlefield + players, return first legal.
            let opp = self
                .opponents_of(controller)
                .first()
                .copied()
                .unwrap_or((controller + 1) % self.players.len());
            let is_legal = |t: &Target| -> bool {
                self.evaluate_requirement_static(&req, t, controller, None)
                    && self.check_target_legality(t, controller).is_ok()
            };
            let pick = {
                // Player slots: try controller first (caster-friendly),
                // then opponent.
                let mut found: Option<Target> = None;
                let player_caster = Target::Player(controller);
                let player_opp = Target::Player(opp);
                if is_legal(&player_caster) {
                    found = Some(player_caster);
                } else if is_legal(&player_opp) {
                    found = Some(player_opp);
                }
                // Battlefield: walk all permanents, prefer one not already
                // picked by slot 0 or earlier slots to avoid double-targeting
                // when the filter is permissive.
                if found.is_none() {
                    let already_picked: Vec<CardId> = std::iter::once(slot_0.clone())
                        .chain(additional.iter().cloned().map(Some))
                        .filter_map(|t| match t {
                            Some(Target::Permanent(id)) => Some(id),
                            _ => None,
                        })
                        .collect();
                    if let Some(c) = self
                        .battlefield
                        .iter()
                        .filter(|c| !already_picked.contains(&c.id))
                        .map(|c| Target::Permanent(c.id))
                        .find(|t| is_legal(t))
                    {
                        found = Some(c);
                    } else if let Some(c) = self
                        .battlefield
                        .iter()
                        .map(|c| Target::Permanent(c.id))
                        .find(|t| is_legal(t))
                    {
                        // Fall back to allowing reuse if nothing else
                        // matches — better to have a target than skip
                        // when the spell semantics allow it.
                        found = Some(c);
                    }
                }
                found
            };
            match pick {
                Some(t) => additional.push(t),
                // Slot has a filter but no legal target — stop here.
                // "Up to N target" effects resolve cleanly with whatever
                // targets were filled in.
                None => break,
            }
            slot += 1;
        }
        (slot_0, additional)
    }
}
