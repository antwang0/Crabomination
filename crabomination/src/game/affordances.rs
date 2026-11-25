//! Affordance / legality probes: which actions a seat could legally take
//! *right now* (castable hand cards, activatable permanents, legal
//! attackers/blockers, alt-cast modes). Each dry-runs `perform_action`
//! against a `GameState` clone — see `affordance_probe_template`.
//! Split out of `game/mod.rs` (no behavior change).

use super::*;

/// One activated ability's probe data: `(is_mana_ability, optional effect to
/// auto-target)`. The mana flag lets the probe loop skip mana abilities
/// without re-walking the effect tree.
type AbilityProbe = (bool, Option<Effect>);
/// A loyalty ability to probe: its effect when it targets, and whether it costs −X.
type LoyaltyProbe = (Option<Effect>, bool);

/// PERF `(-138)`: how many affordance probes write the resolution-scratch
/// fields of `GameState` — the census behind the `CowBox<ResolutionScratch>`
/// group (a probe that never writes the group clones it as one `Arc` bump).
///
/// `SAME_ALL` counts probes the whole group would serve; `SAME_COLL` those a
/// narrower group of only the collection fields would.
///
/// ```text
/// CRAB_SCRATCH_CENSUS=1 bot_ladder --a gang --b gang --games 6 --threads 1 \
///   --seed 1 --decks cube
/// ```
///
/// ⚠ A census, not a gate. The hash maps and `IdSet` are fingerprinted by
/// `len()` (unstable `Debug` order), so a same-count key swap reads as
/// unchanged — a bias towards the device.
///
/// ⚠ It also measures the wrong quantity (PERF `(-143)`): a `CowBox` group is
/// priced by mutable reaches, not value changes. A `clear()` of an empty
/// `Vec`, a `mem::take`, an equal store all read as "unchanged" here yet
/// deep-copy through `Arc::make_mut`. Count reaches for a proposed group with
/// `cg_edges.py --callers make_mut` instead.
pub mod scratch_census {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    pub static PROBES: AtomicU64 = AtomicU64::new(0);
    pub static SAME_ALL: AtomicU64 = AtomicU64::new(0);
    pub static SAME_COLL: AtomicU64 = AtomicU64::new(0);

    pub fn on() -> bool {
        static LEVEL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *LEVEL.get_or_init(|| match std::env::var("CRAB_SCRATCH_CENSUS") {
            Ok(v) => !v.is_empty() && v != "0",
            _ => false,
        })
    }

    pub(crate) fn tick(same_all: bool, same_coll: bool) {
        PROBES.fetch_add(1, Relaxed);
        if same_all {
            SAME_ALL.fetch_add(1, Relaxed);
        }
        if same_coll {
            SAME_COLL.fetch_add(1, Relaxed);
        }
    }

    /// `(probes, same_all, same_coll)` for a caller that wants to print.
    pub fn snapshot() -> (u64, u64, u64) {
        (PROBES.load(Relaxed), SAME_ALL.load(Relaxed), SAME_COLL.load(Relaxed))
    }
}

/// What one dry run of an action found ([`GameState::probe_on`]).
pub(crate) enum Probe {
    /// The engine took it; the state it left.
    Accepted(GameState),
    /// It returned `Ok` only by suspending a bot seat into a replay modal;
    /// the suspended state.
    Suspended(GameState),
    Rejected,
}

impl GameState {

    /// `(whole group, collections only)` fingerprints of the resolution-scratch
    /// field group — see [`scratch_census`].
    fn resolution_scratch_fingerprint(&self) -> (u64, u64) {
        use std::fmt::Write as _;
        use std::hash::{Hash, Hasher};
        fn h(s: &str) -> u64 {
            let mut hasher = crate::fxhash::FxHasher::default();
            s.hash(&mut hasher);
            hasher.finish()
        }
        let mut s = String::with_capacity(4096);
        // Hash-ordered containers are read by length (unstable `Debug` order).
        let _ = write!(
            s,
            "{}|{}|{}|",
            self.scratch.cards_discarded_per_player_this_resolution.len(),
            self.scratch.nonland_cards_discarded_per_player_this_resolution.len(),
            self.scratch.players_sacrificed_this_resolution.len(),
        );
        let _ = write!(s, "{}|", self.scratch.names_this_resolution.len());
        let _ = write!(s, "{:?}|", self.scratch.activation_mana_colors_scratch);
        let _ = write!(s, "{:?}|", self.target_slots_scratch);
        let _ = write!(s, "{:?}|", self.scratch.last_created_tokens);
        let _ = write!(s, "{:?}|", self.scratch.last_moved_cards);
        let _ = write!(s, "{:?}|", self.scratch.proliferated_this_resolution);
        let _ = write!(s, "{:?}|", self.scratch.discarded_card_ids_this_resolution);
        let _ = write!(s, "{:?}|", self.scratch.exiled_card_ids_this_resolution);
        let _ = write!(s, "{:?}|", self.scratch.destroyed_this_resolution);
        let _ = write!(s, "{:?}|", self.scratch.resolution_targets);
        let _ = write!(s, "{:?}|", self.scratch.cost_sacrificed_batch);
        let _ = write!(s, "{:?}|", self.scratch.damaged_this_resolution);
        let _ = write!(s, "{:?}|", self.scratch.cards_sacrificed_this_resolution);
        let _ = write!(s, "{:?}|", self.scratch.chosen_creature_types_scratch);
        let _ = write!(s, "{:?}|", self.scratch.pending_cast_sacrifices);
        let _ = write!(s, "{:?}|", self.scratch.pending_cast_discards);
        let _ = write!(s, "{:?}|", self.scratch.pending_spree_modes);
        let _ = write!(s, "{:?}|", self.scratch.pending_prepare_copies);
        let _ = write!(s, "{:?}|", self.scratch.pending_ability_exile_other);
        let _ = write!(s, "{:?}|", self.scratch.pending_ability_sac_any);
        let _ = write!(s, "{:?}|", self.scratch.resolution_answer_log);
        let _ = write!(s, "{:?}|", self.scratch.pending_cost_events);
        let _ = write!(s, "{:?}|", self.pending_permanent_deaths);
        let _ = write!(s, "{:?}|", self.scratch.pending_control_changes);
        let _ = write!(s, "{:?}|", self.scratch.named_card_this_resolution);
        let _ = write!(s, "{:?}|", self.scratch.resolving_source);
        let _ = write!(s, "{:?}|", self.scratch.suppress_extra_target_prompts);
        let _ = write!(s, "{:?}|", self.scratch.haunt_pending);
        let _ = write!(s, "{:?}|", self.suspend_signal);
        let _ = write!(s, "{:?}|", self.scratch.stashed_resolution_answer);
        let _ = write!(s, "{:?}|", self.scratch.resolving_spell_snapshot);
        let coll = h(&s);
        let _ = write!(s, "{:?}|", self.sacrificed_power);
        let _ = write!(s, "{:?}|", self.revealed_for_cost);
        let _ = write!(s, "{:?}|", self.sacrificed_total_power);
        let _ = write!(s, "{:?}|", self.sacrificed_count);
        let _ = write!(s, "{:?}|", self.sacrificed_toughness);
        let _ = write!(s, "{:?}|", self.sacrificed_mana_value);
        let _ = write!(s, "{:?}|", self.exiled_for_cost_mana_value);
        let _ = write!(s, "{:?}|", self.prevention_source_color_scratch);
        let _ = write!(s, "{:?}|", self.sacrificed_was_artifact);
        let _ = write!(s, "{:?}|", self.sacrificed_was_outlaw);
        let _ = write!(s, "{:?}|", self.sacrificed_was_vehicle);
        let _ = write!(s, "{:?}|", self.sacrificed_card);
        let _ = write!(s, "{:?}|", self.last_discarded_was_multicolored);
        let _ = write!(s, "{:?}|", self.last_discarded_card_types);
        let _ = write!(s, "{:?}|", self.last_discarded_mana_value);
        let _ = write!(s, "{:?}|", self.last_revealed_from_hand);
        let _ = write!(s, "{:?}|", self.cost_discarded_mana_value);
        let _ = write!(s, "{:?}|", self.tapped_for_cost_power);
        let _ = write!(s, "{:?}|", self.scratch.treasure_mana_spent_to_activate);
        let _ = write!(s, "{:?}|", self.scratch.times_paid_this_effect);
        let _ = write!(s, "{:?}|", self.scratch.discarded_for_cost_card);
        let _ = write!(s, "{:?}|", self.trigger_event_amount_scratch);
        let _ = write!(s, "{:?}|", self.trigger_event_player_scratch);
        let _ = write!(s, "{:?}|", self.last_created_token);
        let _ = write!(s, "{:?}|", self.last_die_roll);
        let _ = write!(s, "{:?}|", self.last_name_sticker_vowels);
        let _ = write!(s, "{:?}|", self.extra_cast_reduction);
        let _ = write!(s, "{:?}|", self.cast_paid_uncounterable);
        let _ = write!(s, "{:?}|", self.cast_kick_count);
        let _ = write!(s, "{:?}|", self.cards_discarded_this_resolution);
        let _ = write!(s, "{:?}|", self.cards_drawn_this_resolution);
        let _ = write!(s, "{:?}|", self.energy_paid_this_resolution);
        let _ = write!(s, "{:?}|", self.permanents_returned_this_resolution);
        let _ = write!(s, "{:?}|", self.permanents_tapped_this_resolution);
        let _ = write!(s, "{:?}|", self.cards_revealed_this_resolution);
        let _ = write!(s, "{:?}|", self.scratch.surveil_kept_top_this_resolution);
        let _ = write!(s, "{:?}|", self.creature_cards_discarded_this_resolution);
        let _ = write!(s, "{:?}|", self.greatest_discarded_mv_this_resolution);
        let _ = write!(s, "{:?}|", self.shuffle_resolving_spell_into_library);
        let _ = write!(s, "{:?}|", self.return_resolving_spell_to_hand);
        let _ = write!(s, "{:?}|", self.exile_resolving_spell);
        let _ = write!(s, "{:?}|", self.resolving_spell_library_from_top);
        let _ = write!(s, "{:?}|", self.resolving_spell_to_battlefield_transformed);
        let _ = write!(s, "{:?}|", self.cipher_encode_pending);
        let _ = write!(s, "{:?}|", self.permanents_destroyed_this_resolution);
        let _ = write!(s, "{:?}|", self.excess_damage_this_resolution);
        let _ = write!(s, "{:?}|", self.creatures_died_this_resolution);
        let _ = write!(s, "{:?}|", self.resolution_depth);
        let _ = write!(s, "{:?}|", self.damage_dealt_this_resolution);
        let _ = write!(s, "{:?}|", self.countered_spell_mana_spent);
        let _ = write!(s, "{:?}|", self.countered_spell_mana_value);
        let _ = write!(s, "{:?}|", self.chosen_number_this_resolution);
        let _ = write!(s, "{:?}|", self.resolution_causer);
        let _ = write!(s, "{:?}|", self.nonland_cards_exiled_this_effect);
        let _ = write!(s, "{:?}|", self.countered_spell_controller);
        let _ = write!(s, "{:?}|", self.counters_removed_this_effect);
        let _ = write!(s, "{:?}|", self.counters_removed_as_cost);
        let _ = write!(s, "{:?}|", self.chosen_creature_type_scratch);
        let _ = write!(s, "{:?}|", self.pending_cast_face);
        let _ = write!(s, "{:?}|", self.pending_cast_spend_float);
        let _ = write!(s, "{:?}|", self.pending_landcycle_pick);
        let _ = write!(s, "{:?}|", self.pending_ability_sac_other);
        let _ = write!(s, "{:?}|", self.pending_ability_tap_other);
        let _ = write!(s, "{:?}|", self.source_name_scratch);
        let _ = write!(s, "{:?}|", self.token_minting_source);
        let _ = write!(s, "{:?}|", self.resolving_lki_source);
        let _ = write!(s, "{:?}|", self.resolving_lki_subject);
        let _ = write!(s, "{:?}|", self.resolving_spell_lifelink_seat);
        let _ = write!(s, "{:?}|", self.resolving_spell_caster);
        let _ = write!(s, "{:?}|", self.resolving_spell_deathtouch_seat);
        let _ = write!(s, "{:?}|", self.exile_resolving_spell_with_countdown);
        (h(&s), coll)
    }
    /// Dry-run an action on a clone and return whether the engine would
    /// accept it; `self` is not modified. Lets bots filter illegal actions
    /// without re-implementing every rule (Teferi, Damping Sphere, hexproof,
    /// summoning sickness, …). Cost: one clone plus one `perform_action`.
    pub fn would_accept(&self, action: GameAction) -> bool {
        // Not for pending decisions: the cloned decider drops scripted state,
        // so a legal answer can be spuriously rejected; bots submit those
        // directly. No transactional checkpoint — the probe is discarded.
        self.accept(action).is_some()
    }

    /// [`would_accept`](Self::would_accept) returning the resulting state —
    /// the `&self` form of [`accept_on`](Self::accept_on).
    pub(crate) fn accept(&self, action: GameAction) -> Option<GameState> {
        Self::accept_on(self, action)
    }

    /// Whether `probe` came back `Ok` only because the action *suspended*
    /// into a replay modal — cost pick, {X} pick, float-spend confirm —
    /// rather than completing.
    ///
    /// Applies only to bots (`!manual_mana`); for a hand-paying seat a
    /// suspend really means "you can begin casting this". A bot that read
    /// such an `Ok` committed an unpayable cast whose replay rolled back
    /// forever — a deadlock that dropped 14 % of cube games. This closes the
    /// whole class, including modals added later.
    fn suspended_without_completing(&self, probe: &GameState, acting: usize) -> bool {
        if self.players.get(acting).is_some_and(|p| p.manual_mana) {
            return false;
        }
        probe.pending_decision.as_ref().is_some_and(|pd| pd.resume.is_action_replay())
    }

    /// Whether `seat` could pay `{n}` right now — floating mana plus what
    /// auto-tapping their untapped sources would add. Probed on a clone so
    /// nothing is spent; drives the "can't attack/block unless you pay"
    /// affordance and blocker-legality gates.
    pub(crate) fn could_pay_generic(&self, seat: usize, n: u32) -> bool {
        n == 0 || self.could_pay_cost(seat, &crate::mana::cost(&[crate::mana::generic(n)]))
    }

    /// The general form of [`could_pay_generic`](Self::could_pay_generic):
    /// whether `seat` could pay an arbitrary cost right now. Probed on a clone,
    /// so nothing is spent.
    pub(crate) fn could_pay_cost(&self, seat: usize, cost: &crate::mana::ManaCost) -> bool {
        cost.cmc() == 0
            || crate::game::pay_census::in_probe(|| {
                self.affordance_probe_template().try_pay_with_auto_tap(seat, cost).is_ok()
            })
    }

    /// A clone of `self` used as a reusable dry-run template by the
    /// affordance probes ([`would_accept_on`](Self::would_accept_on)) and the
    /// bot's candidate sweep (`server::bot`).
    ///
    /// Libraries are kept: they are [`CowBox`](crate::cow::CowBox)es, so
    /// stripping them (as an older version did) only cost a `PlayerData`
    /// unshare per player.
    pub(crate) fn affordance_probe_template(&self) -> GameState {
        self.clone()
    }

    /// Dry-run `action` against a prebuilt [`affordance_probe_template`].
    /// Equivalent to [`would_accept`](Self::would_accept) for cast / activate
    /// / play-land actions; a sweep builds the template once and clones it
    /// per candidate.
    ///
    /// [`affordance_probe_template`]: Self::affordance_probe_template
    pub(crate) fn would_accept_on(template: &GameState, action: GameAction) -> bool {
        Self::accept_on(template, action).is_some()
    }

    /// [`would_accept_on`](Self::would_accept_on) that hands back the state the
    /// accepted action produced, instead of dropping it.
    ///
    /// The probe runs the action to completion, so a caller that owns its
    /// state adopts the result instead of running the action twice (the
    /// second run was 15.9 % of the profile at the fiftieth pass).
    ///
    /// The result is what [`perform_action`](Self::perform_action) would
    /// leave (same clone, `perform_action_inner`, trailing
    /// `clear_stale_target_suppression`) minus the checkpoint: adopt it only
    /// where a rejected action would be abandoned, not restored.
    pub(crate) fn accept_on(template: &GameState, action: GameAction) -> Option<GameState> {
        match Self::probe_on(template, action) {
            Probe::Accepted(g) => Some(g),
            Probe::Suspended(_) | Probe::Rejected => None,
        }
    }

    /// [`accept_on`](Self::accept_on) that tells a refusal from a bot's
    /// suspension into a replay modal (a cost pick, a sacrifice to pay), and
    /// hands back the suspended state. A caller asking "does some answer
    /// complete it" reads the suspension off this one probe. It used to
    /// re-run every refused action to find out: 12 % of a policy-played
    /// game (`decision_menu::accepts`, 2026-11-25).
    pub(crate) fn probe_on(template: &GameState, action: GameAction) -> Probe {
        crate::game::pay_census::in_probe(|| {
            let mut probe = template.clone();
            let acting = template.priority.player_with_priority;
            let before =
                scratch_census::on().then(|| template.resolution_scratch_fingerprint());
            let manual = template.players.get(acting).is_some_and(|p| p.manual_mana);
            let retry = manual.then(|| action.clone());
            let mut result = probe.perform_action_inner(action);
            // CR 601.2g — a hand-paying seat's cast returns `ManualTapRequired`
            // when several source sets could pay: a suspension, not a refusal.
            // Re-run with the engine tapping to learn whether it can pay at
            // all (else Grizzly Bears over two Forests and a Mountain read
            // uncastable).
            if let (Err(crate::game::GameError::ManualTapRequired { .. }), Some(action)) = (&result, retry) {
                probe = template.clone();
                probe.players[acting].manual_mana = false;
                result = probe.perform_action_inner(action);
                probe.players[acting].manual_mana = true;
            }
            let mut ok = result.is_ok();
            // An "up to N targets" cast suspends for its next optional slot;
            // declining is always legal, so the probe declines and reads the
            // result (Fireball was otherwise never accepted for a bot).
            if ok
                && !template.players.get(acting).is_some_and(|p| p.manual_mana)
                && probe.pending_decision.as_ref().is_some_and(|pd| {
                    matches!(
                        pd.resume,
                        crate::game::types::ResumeContext::CastExtraTargetPick { .. }
                            | crate::game::types::ResumeContext::LoyaltyExtraTargetPick { .. }
                    )
                        && matches!(pd.decision, crate::decision::Decision::ChooseTarget { optional: true, .. })
                })
            {
                ok = probe
                    .perform_action_inner(GameAction::SubmitDecision(crate::decision::DecisionAnswer::DeclineTarget))
                    .is_ok();
            }
            if let Some((all, coll)) = before {
                let (a2, c2) = probe.resolution_scratch_fingerprint();
                scratch_census::tick(all == a2, coll == c2);
            }
            if !ok {
                return Probe::Rejected;
            }
            probe.clear_stale_target_suppression();
            if template.suspended_without_completing(&probe, acting) {
                return Probe::Suspended(probe);
            }
            Probe::Accepted(probe)
        })
    }

    /// CardIds in `caster`'s hand they could begin casting (or play, for
    /// lands) **right now**. Drives the client's "castable" hand-card
    /// highlight.
    ///
    /// Authoritative: every candidate is dry-run through [`would_accept`]
    /// (timing, land drops, mana, taxes, targets), mirroring the bot's
    /// candidates in `server::bot`. Modal spells probe every mode until one
    /// accepts (default-mode-only hid Quandrix Charm). X is probed at 0 only,
    /// so an X-only-castable card may be omitted — fine for a hint.
    ///
    /// Empty unless `caster` holds priority, which also skips the clones.
    ///
    /// [`would_accept`]: Self::would_accept
    pub fn castable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.castable_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// [`castable_hand_cards`] against a prebuilt probe template, so a sweep
    /// shares one template; the caller owns the priority short-circuit.
    ///
    /// [`castable_hand_cards`]: Self::castable_hand_cards
    fn castable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.castable_hand_cards_with_taps_on(template, caster).into_iter().map(|(id, _)| id).collect()
    }

    /// [`castable_hand_cards_on`](Self::castable_hand_cards_on), each card
    /// paired with the permanents the probe cast tapped or sacrificed to pay
    /// (the client highlights them on hover).
    pub(crate) fn castable_hand_cards_with_taps_on(
        &self,
        template: &GameState,
        caster: usize,
    ) -> Vec<(CardId, Vec<CardId>)> {
        // Snapshot what each probe needs up front so the immutable borrow
        // of `self.players[caster].hand` is released before the cloning
        // probes run. The effect is cloned only for targeted non-lands.
        let hand: Vec<(CardId, bool, Option<_>, usize)> = self.players[caster]
            .hand
            .iter()
            .map(|c| {
                let is_land = c.definition.is_land();
                let needs_target = !is_land && c.definition.effect.requires_target();
                let mode_count = match &c.definition.effect {
                    crate::effect::Effect::ChooseMode(m) => m.len(),
                    _ => 0,
                };
                (c.id, is_land, needs_target.then(|| c.definition.effect.clone()), mode_count)
            })
            .collect();

        let mut out = Vec::new();
        for (id, is_land, targeted_effect, mode_count) in &hand {
            let accepted = if *is_land {
                Self::would_accept_on(template, GameAction::PlayLand(*id)).then(Vec::new)
            } else {
                // A modal spell is castable if ANY mode is (each mode has
                // its own target pool); everything else probes the single
                // default shape. Bounded by the printed mode count.
                let modes: Vec<Option<usize>> = match mode_count {
                    0 => vec![None],
                    n => (0..*n).map(Some).collect(),
                };
                modes.into_iter().find_map(|mode| {
                    // Auto-pick targets as the bot does; with no legal target
                    // `target: None` is correctly rejected (CR 601.2c).
                    let (target, additional_targets) = match targeted_effect {
                        Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, mode),
                        None => (None, Vec::new()),
                    };
                    let after = Self::accept_on(template, GameAction::CastSpell {
                        card_id: *id,
                        target,
                        additional_targets,
                        mode,
                        x_value: None,
                    })?;
                    Some(Self::sources_spent(template, &after, caster))
                })
            };
            if let Some(taps) = accepted {
                out.push((*id, taps));
            }
        }
        out
    }

    /// The permanents `caster` controlled untapped in `before` that `after`
    /// has tapped or no longer has (a Treasure sacrificed for its mana).
    fn sources_spent(before: &GameState, after: &GameState, caster: usize) -> Vec<CardId> {
        before
            .battlefield
            .iter()
            .filter(|c| c.controller == caster && !c.tapped)
            .filter(|c| after.battlefield.iter().find(|a| a.id == c.id).is_none_or(|a| a.tapped))
            .map(|c| c.id)
            .collect()
    }

    /// CardIds of permanents `seat` controls with at least one **non-mana**
    /// activated ability they could activate **right now** — dry-run through
    /// [`would_accept`]. Drives the client's activatable highlight and its
    /// priority-stop heuristic. Empty off-priority.
    ///
    /// Mana abilities are excluded: they don't use the stack and are
    /// auto-tapped on demand, so a mana dork is no reason to hold priority.
    ///
    /// [`would_accept`]: Self::would_accept
    pub fn activatable_permanents(&self, seat: usize) -> Vec<CardId> {
        if self.player_with_priority() != seat {
            return Vec::new();
        }
        self.activatable_permanents_on(&self.affordance_probe_template(), seat)
    }

    /// [`activatable_permanents`] against a prebuilt probe template; the
    /// caller owns the priority short-circuit.
    ///
    /// [`activatable_permanents`]: Self::activatable_permanents
    fn activatable_permanents_on(&self, template: &GameState, seat: usize) -> Vec<CardId> {
        self.activatable_abilities_on(template, seat, false).into_iter().map(|(id, _)| id).collect()
    }

    /// `(permanent, ability index)` for each non-mana activated ability
    /// `seat` could activate right now, by the same dry run as
    /// [`activatable_permanents`] — every ability with `every`, else only the
    /// first per permanent (probes are the cost). Printed abilities first,
    /// then static-granted ones (Debtor's Pulpit, Urza's Saga) at the indices
    /// `activate_ability` resolves them from.
    ///
    /// [`activatable_permanents`]: Self::activatable_permanents
    fn activatable_abilities_on(&self, template: &GameState, seat: usize, every: bool) -> Vec<(CardId, usize)> {
        // Snapshot (id, [ability probes]) so the borrow of `self.battlefield`
        // is released before the cloning probes run.
        let scan = self.grant_scan();
        let perms: Vec<(CardId, Vec<AbilityProbe>)> = self
            .battlefield
            .iter()
            .filter_map(|c| {
                let abilities: Vec<&crate::effect::ActivatedAbility> =
                    c.definition.activated_abilities.iter().chain(self.granted_abilities_with(c.id, &scan)).collect();
                // Seat's own permanents, plus others' permanents carrying an
                // `opponents_only` (Detention Vortex) or `any_player` (Damping
                // Engine) ability the seat may activate.
                let reachable = c.controller == seat
                    || abilities.iter().any(|a| a.any_player)
                    || (!self.same_team(c.controller, seat) && abilities.iter().any(|a| a.opponents_only));
                (!abilities.is_empty() && reachable).then_some((c, abilities))
            })
            .map(|(c, abilities)| {
                let owns = c.controller == seat;
                let effs = abilities
                    .into_iter()
                    .map(|a| {
                        // Only surface abilities the seat is actually allowed to
                        // use: own permanents' non-opponents_only abilities, or
                        // opponents' opponents_only abilities.
                        let usable =
                            a.any_player || if owns { !a.opponents_only } else { a.opponents_only };
                        let targeted = (usable && a.effect.requires_target()).then(|| a.effect.clone());
                        (is_mana_ability_effect(&a.effect) || !usable, targeted)
                    })
                    .collect();
                (c.id, effs)
            })
            .collect();

        let mut out = Vec::new();
        for (id, ability_effects) in &perms {
            let live = ability_effects.iter().enumerate().filter(|(idx, (is_mana, targeted))| {
                // Skip mana abilities — they don't use the stack and aren't a
                // meaningful instant-speed play (see method doc).
                if *is_mana {
                    return false;
                }
                let target = match targeted {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, seat, None).0,
                    None => None,
                };
                Self::would_accept_on(template, GameAction::ActivateAbility {
                    card_id: *id,
                    ability_index: *idx,
                    target,
                    additional_targets: Vec::new(),
                    x_value: None, mode: None,
                })
            });
            out.extend(live.map(|(idx, _)| (*id, idx)).take(if every { usize::MAX } else { 1 }));
        }
        out
    }

    /// `(planeswalker, loyalty ability index)` for each loyalty ability
    /// `seat` could activate right now — its main phase on an empty stack,
    /// an activation left this turn, the loyalty to pay (CR 606.3) — by the
    /// same dry run, over printed and granted abilities
    /// (`effective_loyalty_abilities`); −X probed at X = 0. Feeds the
    /// client's outline and inspector (loyalty abilities aren't
    /// `activated_abilities`).
    fn activatable_loyalty_on(&self, template: &GameState, seat: usize) -> Vec<(CardId, usize)> {
        let walkers: Vec<(CardId, Vec<LoyaltyProbe>)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && self.computed_has_card_type(c, crate::card::CardType::Planeswalker))
            .map(|c| {
                let abilities = super::effective_loyalty_abilities(c, &self.battlefield)
                    .into_iter()
                    .map(|a| (a.effect.requires_target().then_some(a.effect), a.x_cost))
                    .collect();
                (c.id, abilities)
            })
            .collect();
        let mut out = Vec::new();
        for (id, abilities) in &walkers {
            for (idx, (targeted, x_cost)) in abilities.iter().enumerate() {
                let target = targeted.as_ref().and_then(|eff| self.auto_targets_for_effect_all_slots(eff, seat, None).0);
                let action = GameAction::ActivateLoyaltyAbility {
                    card_id: *id,
                    ability_index: idx,
                    target,
                    x_value: x_cost.then_some(0),
                };
                if Self::would_accept_on(template, action) {
                    out.push((*id, idx));
                }
            }
        }
        out
    }

    /// Creatures `seat` controls that could be declared as attackers right
    /// now — only meaningful during `seat`'s Declare Attackers step while it
    /// holds priority. Drives the client's legal-attacker highlight.
    pub fn legal_attackers(&self, seat: usize) -> Vec<CardId> {
        if self.step != crate::TurnStep::DeclareAttackers
            || self.attack_declarer() != seat
            || self.player_with_priority() != seat
        {
            return Vec::new();
        }
        // The declaration is always over the *active player's* creatures, even
        // when a `combat_chooser` (Master Warcraft) submits it.
        let owner = self.active_player_idx;
        // Per-candidate computed reads below — share one layer gather.
        self.with_frozen_layers(|g| g.legal_attackers_inner(owner))
    }

    fn legal_attackers_inner(&self, seat: usize) -> Vec<CardId> {
        use crate::card::Keyword;
        self.battlefield
            .iter()
            // Like `can_attack()` but defers the Defender check to the
            // kws block below, which honors the ignore-defender exception.
            .filter(|c| {
                c.controller == seat
                    // CR 506.2 / 613.1d — a crewed Vehicle, an animated land
                    // or a stationed Spacecraft is a creature by its computed
                    // type line (the layers are frozen for this walk).
                    && (self.computed_is_creature(c)
                        || self.computed_permanent(c.id).is_some_and(|cp| {
                            cp.card_types().contains(&crate::card::CardType::Creature)
                        }))
                    && !c.tapped
                    && !c.has_keyword(&Keyword::CantAttack)
            })
            .filter(|c| {
                // Honor layer-granted Defender / can't-attack and the
                // per-defender attack restriction (Dandân) so the client's
                // highlight matches what `declare_attackers` will accept.
                let cp = self.computed_permanent(c.id);
                let kws: &[Keyword] = match &cp {
                    Some(cp) => cp.keywords(),
                    None => &c.definition.keywords,
                };
                if (kws.contains(&Keyword::Defender) && !self.ignores_defender_for_attack(c))
                    || kws.contains(&Keyword::CantAttack)
                {
                    return false;
                }
                // CR 302.6 / 702.10 — haste by the computed keywords, as
                // `declare_attackers` reads it (Samut's layer-granted haste).
                if c.summoning_sick && !kws.contains(&Keyword::Haste) && !self.attack_as_though_haste(seat, None) {
                    return false;
                }
                kws.iter().all(|kw| match kw {
                    Keyword::CantAttackUnlessDefenderIsMonarch => {
                        self.monarch.is_some_and(|m| m != seat && self.players[m].is_alive())
                    }
                    Keyword::CanAttackOnlyIfDefenderControls(req) => {
                        // Legal if at least one alive opponent's board satisfies
                        // the filter (they could be chosen as the defender).
                        (0..self.players.len()).any(|d| {
                            d != seat
                                && self.players[d].is_alive()
                                && self.battlefield.iter().any(|p| {
                                    p.controller == d
                                        && self.evaluate_requirement_on_card(req, p, d)
                                })
                        })
                    }
                    Keyword::CanAttackOnlyIfYouControl(req) => self
                        .battlefield
                        .iter()
                        .any(|p| p.controller == seat && self.evaluate_requirement_on_card(req, p, seat)),
                    Keyword::CantAttackOrBlockUnlessEvenCounters => {
                        c.counter_sum() % 2 == 0
                    }
                    Keyword::CantAttackOrBlockUnlessHandSizeAtMost(n) => {
                        self.players[seat].hand.len() as u32 <= *n
                    }
                    Keyword::CantAttackOrBlockUnlessAHandIsEmpty => self.some_hand_is_empty(),
                    Keyword::CantAttackOrBlockUnlessDelirium => self.delirium_active(seat),
                    Keyword::CantAttackUnlessLandTypeOnBattlefield(lt) => self
                        .battlefield
                        .iter()
                        .any(|p| self.permanent_has_land_type(p, *lt)),
                    // CR 508.1g — the whole pay-gate family is legal only if the
                    // seat can produce the tax (pool + auto-tappable sources).
                    Keyword::CantAttackOrBlockUnlessPay(_)
                    | Keyword::CantAttackUnlessPay(_)
                    | Keyword::CantAttackUnlessPayPerCounter(_)
                    | Keyword::CantAttackOrBlockUnlessPayPerCounter(_)
                    | Keyword::CantAttackOrBlockUnlessPayPerCardInEnchanterHand
                    | Keyword::CantAttackOrBlockUnlessPayPerPermanent(_) => self.could_pay_generic(
                        seat,
                        self.attack_block_keyword_tax(c.id, kws, true),
                    ),
                    // CR 508.1g — Hollow Warrior needs a spare untapped match
                    // that isn't the attacker itself.
                    Keyword::AttackBlockCostTapAnother(f) => self.battlefield.iter().any(|p| {
                        p.controller == seat
                            && !p.tapped
                            && p.id != c.id
                            && self.evaluate_requirement_on_card(f, p, seat)
                    }),
                    // CR 508.1g — Leviathan needs N sacrificeable matches
                    // besides itself before the attack is even offered.
                    Keyword::AttackCostSacrifice(f, n) => {
                        self.battlefield
                            .iter()
                            .filter(|p| {
                                p.controller == seat
                                    && p.id != c.id
                                    && self.evaluate_requirement_on_card(f, p, seat)
                            })
                            .count()
                            >= *n as usize
                    }
                    // CR 508.1a — Branded Brawlers / Veteran Brawlers can only
                    // swing into a tapped-out defender.
                    Keyword::CantAttackIfDefenderHasUntappedLand => {
                        (0..self.players.len()).any(|d| {
                            d != seat
                                && self.players[d].is_alive()
                                && !self.battlefield.iter().any(|p| {
                                    p.controller == d && p.definition.is_land() && !p.tapped
                                })
                        })
                    }
                    // CR 508.1a — Vow of Duty: some defender besides the
                    // Aura's controller must remain.
                    Keyword::CantAttackAuraController | Keyword::CantAttackPlayer(_) | Keyword::CantAttackOwner => (0..self.players.len()).any(|d| {
                        !self.same_team(seat, d)
                            && self.players[d].is_alive()
                            && self.attacker_target_block(seat, c.id, kws, Some(d)).is_none()
                    }),
                    Keyword::CantAttackOrBlockUnlessCreatureDiedThisTurn => {
                        self.players[seat].creatures_died_this_turn > 0
                    }
                    Keyword::CantAttackOrBlockUnlessDescend(n) => {
                        self.descend_count(seat) >= *n as usize
                    }
                    Keyword::CantAttackOrBlockUnlessCityBlessing => {
                        self.players[seat].city_blessing
                    }
                    Keyword::CantAttackOrBlockUnlessYouControlCount {
                        filter, min, block_only, exclude_self, ..
                    } => {
                        *block_only
                            || self
                                .battlefield
                                .iter()
                                .filter(|p| {
                                    p.controller == seat
                                        && !(*exclude_self && p.id == c.id)
                                        && self.evaluate_requirement_on_card(filter, p, seat)
                                })
                                .count() as u32
                                >= *min
                    }
                    _ => true,
                })
            })
            .map(|c| c.id)
            .collect()
    }

    /// Creatures `seat` controls that could legally block at least one of the
    /// currently-declared attackers — only meaningful during the Declare
    /// Blockers step with attackers on the board. Drives the client's
    /// legal-blocker highlight.
    pub fn legal_blockers(&self, seat: usize) -> Vec<CardId> {
        if self.step != crate::TurnStep::DeclareBlockers
            || self.attacking().is_empty()
            || !self.may_declare_blocks(seat)
        {
            return Vec::new();
        }
        // Blockers come from the defending players — with a `combat_chooser`
        // that isn't the declaring seat's own board.
        let active = self.active_player_idx;
        // Share one layer gather across the scan, and compute each
        // attacker's view once instead of per blocker candidate.
        self.with_frozen_layers(|g| {
            let attackers = g.computed_attackers();
            g.battlefield
                .iter()
                .filter(|c| c.controller != active && c.can_block())
                .filter(|c| g.can_block_any_computed_attacker(c.id, &attackers))
                .map(|c| c.id)
                .collect()
        })
    }

    /// Which of the declared attackers each of `seat`'s legal blockers may
    /// actually block (`legal_blockers` only answers "can block anything"),
    /// so the client can grey out illegal drops. Empty outside Declare
    /// Blockers.
    pub fn legal_block_targets(&self, seat: usize) -> Vec<(CardId, Vec<CardId>)> {
        let blockers = self.legal_blockers(seat);
        if blockers.is_empty() {
            return Vec::new();
        }
        let attacker_ids: Vec<CardId> = self.attacking().iter().map(|a| a.attacker).collect();
        self.with_frozen_layers(|g| {
            blockers
                .into_iter()
                .map(|b| {
                    let targets = attacker_ids
                        .iter()
                        .copied()
                        .filter(|a| g.blocker_can_block_attacker(b, *a))
                        .collect();
                    (b, targets)
                })
                .collect()
        })
    }

    /// Hand cards the viewer could cast *with their Kicker paid* right now
    /// (CR 702.32), by a `CastSpellKicked` dry-run. Feeds the client's "pay
    /// kicker?" affordance. Empty off-priority.
    pub fn kickable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.kickable_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// [`kickable_hand_cards`] against a prebuilt probe template; the caller
    /// owns the priority short-circuit.
    ///
    /// [`kickable_hand_cards`]: Self::kickable_hand_cards
    fn kickable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        // CR 702.175 — a board-granted offspring cost (Zinnia) is a kicker
        // too; one board walk decides whether any hand creature can have one.
        let grants_offspring = self.battlefield.iter().any(|c| {
            c.controller == caster
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::CreatureSpellsGainOffspring { .. })
                })
        });
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| {
                c.definition.has_kicker().is_some()
                    || c.definition.kicker_action_cost.is_some()
                    || (grants_offspring && self.granted_offspring_cost(caster, &c.definition).is_some())
            })
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            // Use the kicked target set (the broader `If(SpellWasKicked, …)`
            // branch) so a kicked Tear Asunder reports castable at a creature.
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => {
                        self.auto_targets_for_effect_all_slots_kicked(eff, caster, None, true, None)
                    }
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if Self::would_accept_on(template, GameAction::CastSpellKicked {
                card_id: *id,
                target,
                additional_targets,
                mode: None,
                x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// CR 702.33b — for each hand card with "Kicker {A} and/or {B}", the
    /// option subsets the caster could actually pay for right now (dry-run per
    /// subset). Lets the client offer one menu entry per payable combination.
    fn kicker_option_sets_on(
        &self,
        template: &GameState,
        caster: usize,
    ) -> Vec<(CardId, Vec<Vec<u8>>)> {
        // CR 702.175 — a granted offspring paid beside the card's own kicker
        // or offspring rides the same action as one more option.
        let hand: Vec<(CardId, Vec<u8>, bool, crate::effect::Effect)> = self.players[caster]
            .hand
            .iter()
            .filter_map(|c| {
                let mut opts: Vec<u8> = (0..c.definition.kicker_options.len() as u8).collect();
                if self.granted_offspring_beside_own(caster, &c.definition).is_some() {
                    opts.push(crate::card::GRANTED_OFFSPRING_OPTION);
                }
                (!opts.is_empty()).then(|| {
                    (c.id, opts, c.definition.effect.requires_target(), c.definition.effect.clone())
                })
            })
            .collect();
        let mut out = Vec::new();
        for (id, opts, needs_target, effect) in &hand {
            let (target, additional_targets) = if *needs_target {
                self.auto_targets_for_effect_all_slots_kicked(effect, caster, None, true, None)
            } else {
                (None, Vec::new())
            };
            // Non-empty subsets of `opts`, in mask order.
            let sets: Vec<Vec<u8>> = (1u32..(1 << opts.len()))
                .map(|mask| {
                    opts.iter().enumerate().filter(|(i, _)| mask & (1 << i) != 0).map(|(_, &o)| o).collect()
                })
                .filter(|kickers: &Vec<u8>| {
                    Self::would_accept_on(template, GameAction::CastSpellKickers {
                        card_id: *id,
                        kickers: kickers.clone(),
                        target: target.clone(),
                        additional_targets: additional_targets.clone(),
                        mode: None,
                        x_value: None,
                    })
                })
                .collect();
            if !sets.is_empty() {
                out.push((*id, sets));
            }
        }
        out
    }

    /// CardIds in the caster's hand with a live Miracle window (CR 702.94) —
    /// revealed as the turn's first draw. A structural filter, no dry-run:
    /// reads the `granted_alt_cast_cost_eot` + `may_play_until` stamp
    /// `maybe_grant_miracle` leaves.
    pub fn miracle_hand_cards(&self, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter(|c| {
                c.cold_any(|k| {
                    k.granted_alt_cast_cost_eot.is_some()
                        && k.may_play_until.as_ref().is_some_and(|p| p.player == caster)
                })
            })
            .map(|c| c.id)
            .collect()
    }

    /// Hand cards a standing static lets `caster` cast without paying their
    /// mana cost right now — Omniscience, Aluren, Conspiracy Unraveler's
    /// collect-evidence swap. Per-card `may_play_until` grants (miracle) are
    /// reported by `miracle_hand_cards` instead.
    fn free_castable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let hand: Vec<(CardId, Option<Effect>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| {
                !c.cold_any(|k| k.may_play_until.is_some())
                    && !c.definition.is_land()
                    && (self.player_casts_hand_spells_free(caster, c)
                        || self.player_casts_cheap_creature_free(&c.definition)
                        || self.player_casts_spells_for_evidence(caster).is_some())
            })
            .map(|c| {
                let eff = &c.definition.effect;
                (c.id, eff.requires_target().then(|| eff.clone()))
            })
            .collect();
        hand.iter()
            .filter(|(id, targeted)| {
                let (target, additional_targets) = match targeted {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                };
                Self::would_accept_on(template, GameAction::CastFromZoneWithoutPaying {
                    card_id: *id,
                    target,
                    additional_targets,
                    mode: None,
                    x_value: None,
                })
            })
            .map(|(id, _)| *id)
            .collect()
    }

    /// Cards **outside** the caster's hand carrying a live `may_play_until`
    /// permission that they could play right now, via
    /// `GameAction::CastFromZoneWithoutPaying`.
    ///
    /// `Effect::GrantMayPlay` stamps the permission wherever the card sits
    /// (exile for Suspend Aggression, a graveyard for Practiced Scrollsmith);
    /// this publishes that set so the UI can offer the play. Hand-resident
    /// permissions have their own lists. Dry-run gated like every affordance.
    pub fn may_play_castable(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.may_play_castable_on(&self.affordance_probe_template(), caster)
    }

    fn may_play_castable_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let candidates: Vec<(CardId, Option<Effect>)> = self
            .exile
            .iter()
            .chain(self.players.iter().flat_map(|p| p.graveyard.iter()))
            .filter(|c| {
                c.cold_any(|k| k.may_play_until.is_some_and(|perm| perm.player == caster))
                    || (c.exiled_with.is_some() && self.linked_exile_counter_cast(caster, c).is_some())
            })
            .map(|c| {
                let eff = &c.definition.effect;
                (c.id, eff.requires_target().then(|| eff.clone()))
            })
            .collect();
        candidates
            .iter()
            .filter(|(id, targeted)| {
                let (target, additional_targets) = match targeted {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                };
                Self::would_accept_on(template, GameAction::CastFromZoneWithoutPaying {
                    card_id: *id,
                    target,
                    additional_targets,
                    mode: None,
                    x_value: None,
                })
            })
            .map(|(id, _)| *id)
            .collect()
    }

    fn may_play_playable_lands_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let candidates: Vec<CardId> = self
            .exile
            .iter()
            .chain(self.players.iter().flat_map(|p| p.graveyard.iter()))
            .filter(|c| {
                c.definition.is_land()
                    && c.cold_any(|k| k.may_play_until.is_some_and(|perm| perm.player == caster))
            })
            .map(|c| c.id)
            .collect();
        candidates
            .into_iter()
            .filter(|id| Self::would_accept_on(template, GameAction::PlayLand(*id)))
            .collect()
    }

    /// CardIds in the caster's hand they could cast right now paying the
    /// optional Buyback cost (CR 702.27). Mirrors `kickable_hand_cards`.
    pub fn buyback_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.buyback_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// CR 702.176 — hand cards with Bargain the caster could cast right now
    /// (probed at `sacrifice: None`, since the Bargain cost is optional).
    /// Mirrors `buyback_hand_cards`.
    fn bargainable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        use crate::card::Keyword;
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.keywords.contains(&Keyword::Bargain))
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if Self::would_accept_on(template, GameAction::CastSpellBargain {
                card_id: *id,
                sacrifice: None,
                target,
                additional_targets,
                mode: None,
                x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// CR 702.157 — hand cards with Squad the caster could cast paying the
    /// squad cost at least once right now (probed at `times: 1`). Mirrors
    /// `bargainable_hand_cards_on`.
    fn squadable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.squad_cost().is_some())
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if Self::would_accept_on(template, GameAction::CastSpellSquad {
                card_id: *id, times: 1, target, additional_targets, mode: None, x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// CR 702.172 — hand Spree cards the caster could cast right now choosing
    /// the single cheapest mode. Offered so the client surfaces the per-mode
    /// cost picker; the actual mode/target choice happens in the UI.
    fn spreeable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let candidates: Vec<(CardId, usize)> = self.players[caster]
            .hand
            .iter()
            .filter_map(|c| match &c.definition.effect {
                crate::effect::Effect::Spree { modes } | crate::effect::Effect::Tiered { modes }
                    if !modes.is_empty() =>
                {
                    // Probe the cheapest single mode (by mana value).
                    let cheapest = modes
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, m)| m.cost.cmc())
                        .map(|(i, _)| i)?;
                    Some((c.id, cheapest))
                }
                _ => None,
            })
            .collect();
        let mut out = Vec::new();
        for (id, mode) in candidates {
            let effect = self.players[caster].hand.iter().find(|c| c.id == id).map(|c| {
                if let crate::effect::Effect::Spree { modes }
                | crate::effect::Effect::Tiered { modes } = &c.definition.effect
                {
                    modes[mode].effect.clone()
                } else {
                    crate::effect::Effect::Noop
                }
            });
            let (target, additional_targets) = match &effect {
                Some(eff) if eff.requires_target() => {
                    self.auto_targets_for_effect_all_slots(eff, caster, None)
                }
                _ => (None, Vec::new()),
            };
            if Self::would_accept_on(template, GameAction::CastSpellSpree {
                card_id: id,
                spree_modes: vec![mode as u8],
                target,
                additional_targets,
                x_value: None,
            }) {
                out.push(id);
            }
        }
        out
    }

    /// CR 702.33c — hand cards with Multikicker the caster could cast paying
    /// the kicker cost at least once right now (probed at `times: 1`).
    fn multikickable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.has_multikicker().is_some())
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if Self::would_accept_on(template, GameAction::CastSpellMultikicked {
                card_id: *id, times: 1, target, additional_targets, mode: None, x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// CR 702.107 — hand cards with Replicate the caster could cast paying the
    /// replicate cost at least once right now (probed at `times: 1`).
    fn replicatable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| {
                c.definition.replicate_cost().is_some()
                    || c.definition.replicate_energy_cost().is_some()
                    || c.definition.replicate_tap_filter().is_some()
                    || self.granted_replicate_cost(caster, c).is_some()
            })
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if Self::would_accept_on(template, GameAction::CastSpellReplicate {
                card_id: *id, times: 1, target, additional_targets, mode: None, x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// CR 702.78 — hand cards with Conspire the caster could cast right now,
    /// tapping the first two eligible untapped creatures that share a color
    /// with the spell, and exiled cards a may-play grant lets them cast
    /// (Rassilon's conspire from exile). The chosen pair is a probe only; the
    /// client re-picks.
    fn conspirable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        use crate::card::Keyword;
        let playable_exile =
            self.exile.iter().filter(|c| c.may_play_until.is_some_and(|m| m.player == caster));
        let hand: Vec<(CardId, Vec<crate::mana::Color>, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .chain(playable_exile)
            .filter(|c| c.definition.keywords.contains(&Keyword::Conspire) || self.spell_has_conspire(caster, c))
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (
                    c.id,
                    c.definition.printed_colors(),
                    needs_target,
                    needs_target.then(|| c.definition.effect.clone()),
                )
            })
            .collect();
        let mut out = Vec::new();
        for (id, spell_colors, needs_target, effect) in &hand {
            // Two untapped creatures the caster controls sharing a color.
            let pair: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == caster
                        && !c.tapped
                        && self.computed_is_creature(c)
                        && self
                            .computed_permanent(c.id)
                            .map(|cp| cp.colors.iter().any(|col| spell_colors.contains(&col)))
                            .unwrap_or(false)
                })
                .map(|c| c.id)
                .take(2)
                .collect();
            if pair.len() < 2 {
                continue;
            }
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if Self::would_accept_on(template, GameAction::CastSpellConspire {
                card_id: *id, conspire_creatures: [pair[0], pair[1]],
                target, additional_targets, mode: None, x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// Untapped permanents `seat` controls that could help pay a
    /// convoke / improvise / waterbend cost on `spell` (CR 702.51 / 702.126 /
    /// 701.67): creatures for convoke, artifacts for improvise, either for
    /// waterbend. Empty when the spell has none of the three.
    pub fn helper_tap_candidates(&self, seat: usize, spell: CardId) -> Vec<CardId> {
        use crate::card::Keyword;
        let Some(card) = self.players[seat].hand.iter().find(|c| c.id == spell) else {
            return Vec::new();
        };
        let convoke = card.definition.keywords.contains(&Keyword::Convoke)
            || self.spell_granted_convoke(seat, card);
        let improvise = card.definition.keywords.contains(&Keyword::Improvise)
            || self.spell_granted_improvise(seat, card);
        let waterbend = card.definition.waterbend.is_some();
        let tap_creatures = card.definition.keywords.contains(&Keyword::TapCreaturesForGeneric);
        if !convoke && !improvise && !waterbend && !tap_creatures {
            return Vec::new();
        }
        self.battlefield
            .iter()
            .filter(|c| {
                c.controller == seat
                    && !c.tapped
                    && ((convoke || waterbend || tap_creatures) && self.computed_is_creature(c)
                        || (improvise || waterbend) && self.computed_has_card_type(c, crate::card::CardType::Artifact))
            })
            .map(|c| c.id)
            .collect()
    }

    /// Hand cards `seat` could cast right now by tapping helpers for convoke /
    /// improvise (CR 702.51 / 702.126), so the client can offer the
    /// "tap creatures to help" picker. Dry-run-probed with every eligible
    /// helper tapped — the picker itself lets the caster tap fewer.
    fn convokable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        use crate::card::Keyword;
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| {
                c.definition.keywords.contains(&Keyword::Convoke)
                    || c.definition.keywords.contains(&Keyword::Improvise)
                    || c.definition.keywords.contains(&Keyword::TapCreaturesForGeneric)
                    || self.spell_granted_convoke(caster, c)
                    || self.spell_granted_improvise(caster, c)
            })
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            let helpers = self.helper_tap_candidates(caster, *id);
            if helpers.is_empty() {
                continue;
            }
            let (target, additional_targets) = match effect {
                Some(eff) if *needs_target => {
                    self.auto_targets_for_effect_all_slots(eff, caster, None)
                }
                _ => (None, Vec::new()),
            };
            if Self::would_accept_on(template, GameAction::CastSpellConvoke {
                card_id: *id,
                target,
                additional_targets,
                mode: None,
                x_value: None,
                convoke_creatures: helpers,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// [`buyback_hand_cards`] against a prebuilt probe template; the caller
    /// owns the priority short-circuit.
    ///
    /// [`buyback_hand_cards`]: Self::buyback_hand_cards
    fn buyback_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.has_buyback().is_some())
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if Self::would_accept_on(template, GameAction::CastSpellBuyback {
                card_id: *id,
                target,
                additional_targets,
                mode: None,
                x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// CardIds in the caster's hand that have Bestow and could be cast as an
    /// Aura on some creature right now (CR 702.103). The auto-target picks a
    /// creature; an empty result means no legal host or the cost is unpayable.
    pub fn bestowable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.bestowable_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// [`bestowable_hand_cards`] against a prebuilt probe template; the caller
    /// owns the priority short-circuit.
    ///
    /// [`bestowable_hand_cards`]: Self::bestowable_hand_cards
    fn bestowable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let candidates: Vec<CardId> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.has_bestow().is_some())
            .map(|c| c.id)
            .collect();
        let mut out = Vec::new();
        for id in candidates {
            // Bestow needs a creature host; pick any legal creature target.
            let host = self
                .battlefield
                .iter()
                .find(|c| {
                    self.computed_is_creature(c)
                        && self
                            .check_target_legality_with_source(
                                &Target::Permanent(c.id),
                                caster,
                                Some(id),
                            )
                            .is_ok()
                })
                .map(|c| c.id);
            let Some(host) = host else { continue };
            if Self::would_accept_on(template, GameAction::CastBestow {
                card_id: id,
                target: Some(Target::Permanent(host)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }) {
                out.push(id);
            }
        }
        out
    }

    /// Hand cards with a `from_hand` mana ability (Spirit Guides), for the
    /// client's pitch affordance. Empty off-priority.
    ///
    /// Mana abilities only: the one-click pitch submits no target, so a
    /// targeted from-hand ability would always be rejected; those go through
    /// `KnownCard.zone_abilities` and `hand_activatable`.
    pub fn pitchable_hand_cards(&self, player: usize) -> Vec<CardId> {
        if self.player_with_priority() != player {
            return Vec::new();
        }
        self.players[player]
            .hand
            .iter()
            .filter(|c| {
                c.definition
                    .activated_abilities
                    .iter()
                    .any(|a| a.from_hand && is_mana_ability_effect(&a.effect))
            })
            .map(|c| c.id)
            .collect()
    }

    /// Hand cards the player could cast right now via their **Dash**
    /// alternative cost (CR 702.110). Empty off-priority.
    pub fn dashable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.dashable_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// [`dashable_hand_cards`] against a prebuilt probe template; the caller
    /// owns the priority short-circuit.
    ///
    /// [`dashable_hand_cards`]: Self::dashable_hand_cards
    fn dashable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.alternative_cost.as_ref().is_some_and(|a| a.dash))
            .map(|c| c.id)
            .filter(|&id| {
                Self::would_accept_on(template, GameAction::CastSpellAlternative {
                    card_id: id,
                    pitch_card: None,
                    target: None,
                    additional_targets: vec![],
                    mode: None,
                    x_value: None,
                })
            })
            .collect()
    }

    /// Cards in `caster`'s hand they could cast for their Blitz cost right
    /// now, against a prebuilt probe template; the caller owns the priority
    /// short-circuit.
    fn blitzable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.alternative_cost.as_ref().is_some_and(|a| a.blitz))
            .map(|c| c.id)
            .filter(|&id| {
                Self::would_accept_on(template, GameAction::CastSpellAlternative {
                    card_id: id,
                    pitch_card: None,
                    target: None,
                    additional_targets: vec![],
                    mode: None,
                    x_value: None,
                })
            })
            .collect()
    }

    /// Cards in `caster`'s hand they could cast for their Warp cost right
    /// now, against a prebuilt probe template; the caller owns the priority
    /// short-circuit.
    fn warpable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.alternative_cost.as_ref().is_some_and(|a| a.warp))
            .map(|c| c.id)
            .filter(|&id| {
                Self::would_accept_on(template, GameAction::CastSpellAlternative {
                    card_id: id,
                    pitch_card: None,
                    target: None,
                    additional_targets: vec![],
                    mode: None,
                    x_value: None,
                })
            })
            .collect()
    }

    /// Cards in `caster`'s hand they could suspend right now (CR 702.62):
    /// `Keyword::Suspend` and the action would be accepted. Surfaced in
    /// `PlayerView.suspendable_hand`.
    pub fn suspendable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.suspendable_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// [`suspendable_hand_cards`] against a prebuilt probe template; the
    /// caller owns the priority short-circuit.
    ///
    /// [`suspendable_hand_cards`]: Self::suspendable_hand_cards
    fn suspendable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        use crate::card::Keyword;
        self.players[caster]
            .hand
            .iter()
            .filter(|c| {
                c.definition.keywords.iter().any(|k| matches!(k, Keyword::Suspend(..)))
            })
            .map(|c| c.id)
            .filter(|&id| Self::would_accept_on(template, GameAction::Suspend { card_id: id }))
            .collect()
    }

    /// Cards in `caster`'s hand they could Foretell right now (CR 702.143):
    /// the card has a `foretell_cost` and paying {2} at sorcery speed is
    /// legal. Surfaced in `PlayerView.foretellable_hand`.
    pub fn foretellable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.foretellable_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// [`foretellable_hand_cards`] against a prebuilt probe template; the
    /// caller owns the priority short-circuit.
    ///
    /// [`foretellable_hand_cards`]: Self::foretellable_hand_cards
    fn foretellable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.foretell_cost.is_some() || self.hand_foretell_grant(caster, c).is_some())
            .map(|c| c.id)
            .filter(|&id| Self::would_accept_on(template, GameAction::Foretell { card_id: id }))
            .collect()
    }

    /// Cards `caster` could Plot right now (CR 702.170): a hand card with a
    /// `plot_cost`, plus their library's top card while a
    /// `MayPlotFromLibraryTop` grant is out (Fblthp). Paying at sorcery speed
    /// must be legal.
    fn plottable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.plot_cost.is_some())
            .map(|c| c.id)
            .chain(
                self.may_plot_from_library_top(caster)
                    .then(|| self.players[caster].library.first().map(|c| c.id))
                    .flatten(),
            )
            .filter(|&id| Self::would_accept_on(template, GameAction::Plot { card_id: id }))
            .collect()
    }

    /// CR 702.170d — plotted cards in `caster`'s exile they can cast for free
    /// right now (a later turn, sorcery speed).
    fn castable_plotted_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.exile
            .iter()
            .filter(|c| c.owner == caster && self.plotted_cards.contains(&c.id))
            .map(|c| c.id)
            .filter(|&id| {
                let (target, additional_targets) = template
                    .auto_targets_for_effect_all_slots(
                        &template
                            .exile
                            .iter()
                            .find(|c| c.id == id)
                            .map(|c| c.definition.effect.clone())
                            .unwrap_or(crate::effect::Effect::Noop),
                        caster,
                        None,
                    );
                Self::would_accept_on(
                    template,
                    GameAction::CastPlotted {
                        card_id: id,
                        target,
                        additional_targets,
                        mode: None,
                        x_value: None,
                    },
                )
            })
            .collect()
    }

    /// Cards in `caster`'s hand with an Adventure half they could cast right
    /// now (CR 715). The probe auto-targets the adventure effect.
    fn adventurable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter_map(|c| {
                let adv = c.definition.has_adventure()?;
                let (target, additional_targets) = if adv.effect.requires_target() {
                    let (t, extras) =
                        template.auto_targets_for_effect_all_slots(&adv.effect, caster, None);
                    t.as_ref()?;
                    (t, extras)
                } else {
                    (None, vec![])
                };
                let id = c.id;
                Self::would_accept_on(
                    template,
                    GameAction::CastAdventure {
                        card_id: id, target, additional_targets, mode: None, x_value: None,
                    },
                )
                .then_some(id)
            })
            .collect()
    }

    /// CR 715.3d — cards in `caster`'s adventure exile they can take off the
    /// adventure right now: a land half is *played*, everything else is cast.
    fn adventure_exile_playable_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.exile
            .iter()
            .filter(|c| c.owner == caster && (c.on_adventure || c.self_exile_castable()))
            .filter_map(|c| {
                let id = c.id;
                if c.definition.is_land() {
                    return Self::would_accept_on(template, GameAction::PlayLand(id))
                        .then_some(id);
                }
                let (target, additional_targets) = if c.definition.effect.requires_target() {
                    let (t, extras) = template.auto_targets_for_effect_all_slots(
                        &c.definition.effect,
                        caster,
                        None,
                    );
                    t.as_ref()?;
                    (t, extras)
                } else {
                    (None, vec![])
                };
                Self::would_accept_on(
                    template,
                    GameAction::CastAdventureCreature {
                        card_id: id, target, additional_targets, mode: None, x_value: None,
                    },
                )
                .then_some(id)
            })
            .collect()
    }

    /// Cards in `caster`'s hand with an Omen half they could cast right now
    /// (CR 702.183). The probe auto-targets the omen effect.
    fn omenable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter_map(|c| {
                let omen = c.definition.has_omen()?;
                let (target, additional_targets) = if omen.effect.requires_target() {
                    let (t, extras) =
                        template.auto_targets_for_effect_all_slots(&omen.effect, caster, None);
                    t.as_ref()?;
                    (t, extras)
                } else {
                    (None, vec![])
                };
                let id = c.id;
                Self::would_accept_on(
                    template,
                    GameAction::CastOmen {
                        card_id: id, target, additional_targets, mode: None, x_value: None,
                    },
                )
                .then_some(id)
            })
            .collect()
    }

    /// Split cards in `caster`'s hand whose right half they could cast right
    /// now (CR 709). The probe auto-targets the right half's effect.
    fn splittable_right_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter_map(|c| {
                let split = c.definition.has_split()?;
                let (target, additional_targets) = if split.right.effect.requires_target() {
                    let (t, extras) = template
                        .auto_targets_for_effect_all_slots(&split.right.effect, caster, None);
                    t.as_ref()?;
                    (t, extras)
                } else {
                    (None, vec![])
                };
                let id = c.id;
                Self::would_accept_on(
                    template,
                    GameAction::CastSplitRight {
                        card_id: id, target, additional_targets, mode: None, x_value: None,
                    },
                )
                .then_some(id)
            })
            .collect()
    }

    /// CR 709.5 — Room doors castable from `seat`'s hand right now.
    fn room_castable_on(&self, template: &GameState, seat: usize) -> Vec<(CardId, u8)> {
        self.players[seat]
            .hand
            .iter()
            .filter(|c| c.definition.room.is_some())
            .flat_map(|c| [(c.id, 0u8), (c.id, 1u8)])
            .filter(|(id, door)| {
                Self::would_accept_on(
                    template,
                    GameAction::CastRoomDoor { card_id: *id, right: *door == 1 },
                )
            })
            .collect()
    }

    /// CR 709.5e — locked Room doors on `seat`'s battlefield permanents whose
    /// unlock cost is payable right now.
    fn room_unlockable_on(&self, template: &GameState, seat: usize) -> Vec<(CardId, u8)> {
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat && c.definition.room.is_some())
            .flat_map(|c| {
                let mask = c.unlocked_doors;
                let id = c.id;
                [(1u8, 0u8), (2, 1)]
                    .into_iter()
                    .filter(move |(bit, _)| mask & bit == 0)
                    .map(move |(_, door)| (id, door))
            })
            .filter(|(id, door)| {
                Self::would_accept_on(
                    template,
                    GameAction::UnlockRoomDoor { card_id: *id, right: *door == 1 },
                )
            })
            .collect()
    }

    /// Compute every from-hand affordance hint for `seat` in one pass.
    ///
    /// The standalone `*_hand_cards` methods each build their own
    /// [`affordance_probe_template`]; the per-seat view needs all of them on
    /// every action, so this builds one template and threads it through the
    /// `_on` variants. All-empty off-priority.
    ///
    /// [`affordance_probe_template`]: Self::affordance_probe_template
    pub fn compute_hand_affordances(&self, seat: usize) -> HandAffordances {
        if self.player_with_priority() != seat {
            return HandAffordances::default();
        }
        let template = self.affordance_probe_template();
        // Every live ability, for the client's inspector; the permanents
        // carrying one fall out of it, in the same order.
        let activatable_abilities = self.activatable_abilities_on(&template, seat, true);
        let mut activatable_permanents: Vec<CardId> = activatable_abilities.iter().map(|(id, _)| *id).collect();
        activatable_permanents.dedup();
        let castable_with_taps = self.castable_hand_cards_with_taps_on(&template, seat);
        HandAffordances {
            castable: castable_with_taps.iter().map(|(id, _)| *id).collect(),
            auto_tap_sources: castable_with_taps,
            // Pitchable is a pure structural filter (no dry-run), so it
            // needs no template and never touches the probe clone.
            pitchable: self.pitchable_hand_cards(seat),
            kickable: self.kickable_hand_cards_on(&template, seat),
            kicker_option_sets: self.kicker_option_sets_on(&template, seat),
            buyback: self.buyback_hand_cards_on(&template, seat),
            bestowable: self.bestowable_hand_cards_on(&template, seat),
            dashable: self.dashable_hand_cards_on(&template, seat),
            blitzable: self.blitzable_hand_cards_on(&template, seat),
            warpable: self.warpable_hand_cards_on(&template, seat),
            suspendable: self.suspendable_hand_cards_on(&template, seat),
            foretellable: self.foretellable_hand_cards_on(&template, seat),
            plottable: self.plottable_hand_cards_on(&template, seat),
            castable_plotted: self.castable_plotted_on(&template, seat),
            adventurable: self.adventurable_hand_cards_on(&template, seat),
            adventure_exile: self.adventure_exile_playable_on(&template, seat),
            omenable: self.omenable_hand_cards_on(&template, seat),
            splittable_right: self.splittable_right_hand_cards_on(&template, seat),
            bargainable: self.bargainable_hand_cards_on(&template, seat),
            squadable: self.squadable_hand_cards_on(&template, seat),
            spreeable: self.spreeable_hand_cards_on(&template, seat),
            replicatable: self.replicatable_hand_cards_on(&template, seat),
            conspirable: self.conspirable_hand_cards_on(&template, seat),
            multikickable: self.multikickable_hand_cards_on(&template, seat),
            convokable: self.convokable_hand_cards_on(&template, seat),
            miracle: self.miracle_hand_cards(seat),
            free_castable: self.free_castable_hand_cards_on(&template, seat),
            may_play_castable: self.may_play_castable_on(&template, seat),
            may_play_lands: self.may_play_playable_lands_on(&template, seat),
            activatable_permanents,
            activatable_abilities,
            activatable_loyalty: self.activatable_loyalty_on(&template, seat),
            hand_activatable: self.hand_activatable_cards(seat),
            morphable: self.morphable_hand_cards_on(&template, seat),
            turn_up_able: self.turn_up_able_permanents_on(&template, seat),
            reinforceable: self.reinforceable_hand_cards_on(&template, seat),
            discard_activatable: self.discard_activatable_hand_cards_on(&template, seat),
            room_castable: self.room_castable_on(&template, seat),
            room_unlockable: self.room_unlockable_on(&template, seat),
            prepare_castable: self.prepare_castable_on(&template, seat),
            back_castable: self.back_castable_hand_cards_on(&template, seat),
            prototypable: self.prototypable_hand_cards_on(&template, seat),
            spliceable: self.spliceable_hand_cards_on(&template, seat),
            castable_command: self.castable_command_on(&template, seat),
        }
    }

    /// CR 903.8 — `seat`'s commanders that `CastFromCommandZone` (regular
    /// cost plus the {2}-per-prior-cast tax) would accept right now, with
    /// targets auto-picked as in [`castable_hand_cards_on`]. Only the seat's
    /// own `commanders` are probed (not a conspiracy or Vanguard), so it is
    /// free outside Commander.
    ///
    /// [`castable_hand_cards_on`]: Self::castable_hand_cards_on
    fn castable_command_on(&self, template: &GameState, seat: usize) -> Vec<CardId> {
        let player = &self.players[seat];
        if player.commanders.is_empty() {
            return Vec::new();
        }
        let candidates: Vec<(CardId, Option<Effect>)> = player
            .command
            .iter()
            .filter(|c| player.commanders.contains(&c.id))
            .map(|c| {
                let eff = &c.definition.effect;
                (c.id, eff.requires_target().then(|| eff.clone()))
            })
            .collect();
        candidates
            .into_iter()
            .filter(|(id, targeted)| {
                let (target, additional_targets) = match targeted {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, seat, None),
                    None => (None, Vec::new()),
                };
                Self::would_accept_on(template, GameAction::CastFromCommandZone {
                    card_id: *id,
                    target,
                    additional_targets,
                    mode: None,
                    x_value: None,
                    alternative: false,
                    pitch_card: None,
                })
            })
            .map(|(id, _)| id)
            .collect()
    }

    /// CR 702.47 — for each castable Arcane spell in hand, the Splice cards in
    /// hand whose quality matches and whose splice cost (mana plus any
    /// `splice_extra_cost`) is payable on top, dry-run per candidate. Hosts
    /// with no affordable splicer are omitted. `pub(crate)` so the bot can
    /// ask for this category alone rather than the whole
    /// [`compute_hand_affordances`](Self::compute_hand_affordances) sweep.
    pub(crate) fn spliceable_hand_cards_on(
        &self,
        template: &GameState,
        seat: usize,
    ) -> Vec<(CardId, Vec<CardId>)> {
        use crate::card::Keyword;
        // `None` quality: "splice onto instant or sorcery" (any such host).
        let splicers: Vec<(CardId, Option<crate::card::SpellSubtype>)> = self.players[seat]
            .hand
            .iter()
            .filter_map(|c| {
                c.definition.keywords.iter().find_map(|k| match k {
                    Keyword::Splice(_, quality) => Some((c.id, Some(*quality))),
                    Keyword::SpliceOntoInstantOrSorcery(_) => Some((c.id, None)),
                    _ => None,
                })
            })
            .collect();
        if splicers.is_empty() {
            return Vec::new();
        }
        let any_instant_or_sorcery = splicers.iter().any(|(_, q)| q.is_none());
        let hosts: Vec<(CardId, Vec<crate::card::SpellSubtype>, bool)> = self.players[seat]
            .hand
            .iter()
            .filter(|c| {
                !c.definition.subtypes.spell_subtypes.is_empty()
                    || (any_instant_or_sorcery && (c.definition.is_instant() || c.definition.is_sorcery()))
            })
            .map(|c| (c.id, c.definition.subtypes.spell_subtypes.clone(), (c.definition.is_instant() || c.definition.is_sorcery())))
            .collect();
        hosts
            .into_iter()
            .filter_map(|(host, subtypes, instant_or_sorcery)| {
                let usable: Vec<CardId> = splicers
                    .iter()
                    .filter(|(sid, quality)| {
                        *sid != host
                            && quality.map_or(instant_or_sorcery, |q| subtypes.contains(&q))
                            && Self::would_accept_on(template, GameAction::CastSpellSpliced {
                                card_id: host,
                                splice_cards: vec![*sid],
                                target: None,
                                additional_targets: vec![],
                                mode: None,
                                x_value: None,
                            })
                    })
                    .map(|(sid, _)| *sid)
                    .collect();
                (!usable.is_empty()).then_some((host, usable))
            })
            .collect()
    }

    /// CR 702.160 — hand cards with a Prototype face castable for the
    /// prototype cost right now (prototype creatures don't target on cast,
    /// so cost and timing gate it).
    fn prototypable_hand_cards_on(&self, template: &GameState, seat: usize) -> Vec<CardId> {
        self.players[seat]
            .hand
            .iter()
            .filter(|c| c.definition.has_prototype().is_some())
            .map(|c| c.id)
            .filter(|&id| {
                Self::would_accept_on(template, GameAction::CastPrototype {
                    card_id: id,
                    target: None,
                    additional_targets: vec![],
                    mode: None,
                    x_value: None,
                })
            })
            .collect()
    }

    /// SOS Prepare — prepared creatures `seat` controls whose prepare spell
    /// is castable right now (dry-run gated). `cast_prepare_spell`
    /// auto-targets a `None` target, so one with no legal target is
    /// reported uncastable.
    fn prepare_castable_on(&self, template: &GameState, seat: usize) -> Vec<CardId> {
        use crate::card::CounterType;
        self.battlefield
            .iter()
            .filter(|c| {
                c.controller == seat
                    && c.definition.prepare_spell.is_some()
                    && c.counter_count(CounterType::Prepared) > 0
            })
            .map(|c| c.id)
            .filter(|&id| {
                Self::would_accept_on(template, GameAction::CastPrepareSpell {
                    creature_id: id,
                    target: None,
                    additional_targets: vec![],
                    mode: None,
                    x_value: None,
                })
            })
            .collect()
    }

    /// MDFC cards whose back face is castable right now — from hand, plus
    /// graveyard cards with `may_cast_back_from_graveyard` (no shipped card
    /// grants it). The front-face sweep never probes the back, so without
    /// this a back-only-affordable card reads dead and `auto_advance` passes.
    fn back_castable_hand_cards_on(&self, template: &GameState, seat: usize) -> Vec<CardId> {
        let from_hand = self.players[seat]
            .hand
            .iter()
            .filter(|c| c.definition.back_face.as_ref().is_some_and(|b| !b.is_land()))
            .map(|c| c.id);
        let from_graveyard = self.players[seat]
            .graveyard
            .iter()
            .filter(|c| {
                c.may_cast_back_from_graveyard
                    && c.definition.back_face.as_ref().is_some_and(|b| !b.is_land())
            })
            .map(|c| c.id);
        from_hand
            .chain(from_graveyard)
            .filter(|&id| {
                Self::would_accept_on(template, GameAction::CastSpellBack {
                    card_id: id,
                    target: None,
                    additional_targets: vec![],
                    mode: None,
                    x_value: None,
                })
            })
            .collect()
    }

    /// CR 702.77 — hand cards `seat` could Reinforce right now: the card carries
    /// `Keyword::Reinforce` and the activation (pay cost, discard, +1/+1 on a
    /// creature) is legal against some creature target. Surfaced in
    /// `PlayerView.reinforceable_hand`.
    fn reinforceable_hand_cards_on(&self, template: &GameState, seat: usize) -> Vec<CardId> {
        let any_creature = self
            .battlefield
            .iter()
            .find(|c| self.computed_is_creature(c))
            .map(|c| c.id);
        let Some(tid) = any_creature else { return vec![] };
        self.players[seat]
            .hand
            .iter()
            .filter(|c| {
                c.definition
                    .keywords
                    .iter()
                    .any(|k| matches!(k, crate::card::Keyword::Reinforce(_, _)))
            })
            .map(|c| c.id)
            .filter(|&id| {
                Self::would_accept_on(
                    template,
                    GameAction::Reinforce {
                        card_id: id,
                        target: crate::game::types::Target::Permanent(tid),
                    },
                )
            })
            .collect()
    }

    /// Hand cards `seat` could activate a `discard_activated` ability on right
    /// now (the cost is payable). Surfaced in `PlayerView.discard_activatable_hand`.
    fn discard_activatable_hand_cards_on(&self, template: &GameState, seat: usize) -> Vec<CardId> {
        self.players[seat]
            .hand
            .iter()
            .filter(|c| c.definition.discard_activated.is_some())
            .map(|c| c.id)
            .filter(|&id| {
                Self::would_accept_on(template, GameAction::ActivateDiscardAbility { card_id: id })
            })
            .collect()
    }

    /// CR 708.5 — face-down permanents `seat` controls whose turn-up cost is
    /// payable right now. Dry-run-gated through `would_accept` (which derives
    /// the cost and enforces the manifested-noncreature rule). Surfaced in
    /// `PlayerView.turn_up_able`.
    fn turn_up_able_permanents_on(&self, template: &GameState, seat: usize) -> Vec<CardId> {
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat && c.face_down && c.face_up_def.is_some())
            .map(|c| c.id)
            .filter(|&id| Self::would_accept_on(template, GameAction::TurnFaceUp { card_id: id }))
            .collect()
    }

    /// Cards in `seat`'s hand they could cast face down via Morph right now
    /// (CR 702.36): the card has Morph/Megamorph and paying {3} at sorcery
    /// speed is legal. Surfaced in `PlayerView.morphable_hand`.
    fn morphable_hand_cards_on(&self, template: &GameState, seat: usize) -> Vec<CardId> {
        self.players[seat]
            .hand
            .iter()
            .filter(|c| c.definition.has_morph_ability())
            .map(|c| c.id)
            .filter(|&id| Self::would_accept_on(template, GameAction::CastFaceDown { card_id: id }))
            .collect()
    }

    /// Hand cards with a `from_hand` activated ability usable now (Talon
    /// Gates, Spirit Guides): its once-per-turn slot is unspent (Forecast,
    /// CR 702.56) and its "activate only …" `condition` holds. A structural
    /// filter, no probe clone — `activate_ability` re-checks cost and zone.
    fn hand_activatable_cards(&self, seat: usize) -> Vec<CardId> {
        self.players[seat]
            .hand
            .iter()
            .filter(|c| {
                c.definition
                    .activated_abilities
                    .iter()
                    .enumerate()
                    .any(|(i, a)| a.from_hand && self.hand_ability_usable_now(seat, c.id, i, a))
            })
            .map(|c| c.id)
            .collect()
    }

    /// Whether the `from_hand` ability at index `i` on hand card `card_id` is
    /// usable right now: its once-per-turn slot is free and its `condition`
    /// (if any) evaluates true against the current game state.
    fn hand_ability_usable_now(
        &self,
        seat: usize,
        card_id: CardId,
        i: usize,
        a: &crate::effect::ActivatedAbility,
    ) -> bool {
        if a.once_per_turn && self.triggered_once_per_turn_used.contains(&(card_id, i)) {
            return false;
        }
        match &a.condition {
            None => true,
            Some(cond) => {
                let ctx = crate::game::effects::EffectContext::for_trigger(card_id, seat, None, 0);
                self.evaluate_predicate(cond, &ctx)
            }
        }
    }

    /// Extra generic mana the caster owes on top of `card`'s printed
    /// cost (Damping Sphere, Chancellor of the Annex). Public so the bot's
    /// affordability check matches the engine; ignoring it made the bot
    /// resubmit a rejected cast forever (a spectate-mode deadlock).
    pub fn extra_cost_for_card_in_hand(&self, caster: usize, card_id: CardId) -> u32 {
        let Some(card) = self.players[caster]
            .hand
            .iter()
            .find(|c| c.id == card_id)
        else {
            return 0;
        };
        crate::game::actions::extra_cost_for_spell(self, caster, card, None)
    }
}

/// True if `effect` is a pure mana ability (CR 605.1a) — it only adds mana
/// and has no other rider. Excluded from `activatable_permanents`.
fn is_mana_ability_effect(effect: &Effect) -> bool {
    match effect {
        Effect::AddMana { .. } => true,
        // CR 605.1a/603.7 — a reflexive "when you do" rider doesn't stop
        // the ability being a mana ability (mirrors `actions::is_mana_ability`).
        Effect::ReflexiveTrigger { .. } => true,
        Effect::Seq(steps) => !steps.is_empty() && steps.iter().all(is_mana_ability_effect),
        // A board-state-conditional whose every branch only adds mana is still
        // a mana ability (Ilysian Caryatid, Raucous Audience). Mirrors
        // `actions::is_mana_ability`.
        Effect::If { then, else_, .. } => {
            is_mana_ability_effect(then) && is_mana_ability_effect(else_)
        }
        _ => false,
    }
}
