//! Auto-target picker for effects that the engine resolves without explicit
//! user input (ETB triggers, attack triggers, bot-driven casts).

use crate::card::{CardId, CardInstance};
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
        self.auto_target_for_effect_avoiding_set(
            eff,
            controller,
            avoid_source.as_slice(),
        )
    }

    /// Like [`auto_target_for_effect_avoiding`] but with a set of avoided
    /// permanents — used when a doubled trigger batch picks per-copy targets
    /// at push time (CR 603.3d) so the second copy prefers a fresh target.
    pub fn auto_target_for_effect_avoiding_set(
        &self,
        eff: &Effect,
        controller: usize,
        avoid: &[CardId],
    ) -> Option<Target> {
        self.auto_target_for_effect_avoiding_set_x(eff, controller, avoid, 0)
    }

    /// Like [`auto_target_for_effect_avoiding_set`] but concretizes any
    /// `{X}`-from-cost target filter against `x` before picking (an ETB
    /// triggered ability reading the cast's X — Dune Drifter). Callers with
    /// no X pass 0 via the non-`_x` wrappers.
    pub fn auto_target_for_effect_avoiding_set_x(
        &self,
        eff: &Effect,
        controller: usize,
        avoid: &[CardId],
        x: u32,
    ) -> Option<Target> {
        self.auto_target_for_effect_avoiding_set_xc(eff, controller, avoid, x, 0)
    }

    /// Like [`auto_target_for_effect_avoiding_set_x`] but also concretizes
    /// `ManaValueAtMostConverged` against the cast's converge count before
    /// picking (Sundering Archaic's "exile target nonland permanent … with
    /// mana value ≤ the number of colors of mana spent"). Without this the
    /// converge atom evaluates false-for-everything at selection time, and
    /// a resolve-time-only gate lets the picker aim at over-cap permanents
    /// whose exile then fizzles.
    pub fn auto_target_for_effect_avoiding_set_xc(
        &self,
        eff: &Effect,
        controller: usize,
        avoid: &[CardId],
        x: u32,
        converge: u32,
    ) -> Option<Target> {
        // One freeze over the whole candidate walk — see
        // `legal_targets_for_filter`, which the picker mirrors.
        self.with_frozen_layers(|s| {
            s.auto_target_for_effect_avoiding_set_xc_inner(eff, controller, avoid, x, converge)
        })
    }

    fn auto_target_for_effect_avoiding_set_xc_inner(
        &self,
        eff: &Effect,
        controller: usize,
        avoid: &[CardId],
        x: u32,
        converge: u32,
    ) -> Option<Target> {
        let avoid_source = avoid.first().copied();
        // Effects with a bare `Selector::Target(0)` (e.g. Lightning Bolt's
        // "deal 3 damage to any target") have no surfaced primary filter —
        // they accept any legal entity. Fall back to `Any` so the picker
        // walks players + permanents instead of short-circuiting to None.
        let any_filter = crate::card::SelectionRequirement::Any;
        // `primary_target_filter` and `target_filter_for_slot` are two
        // independent hand-written walks over ~1000 variants, and they sit at
        // opposite ends of one target's life: this picker aims with the first,
        // CR 608.2b re-checks with the second at resolution
        // (`continue_trigger_resolution_with_source`). A variant with an arm in
        // the slot walker and none here aimed at `Any` and was then thrown away
        // by its own filter — a silent, total fizzle with nothing logged. That
        // is how creature Haunt died: `HauntCreature` checks `Enchantment`,
        // picked a *player*, and Absolver Thrull's and Orzhov Euthanist's
        // "destroy target enchantment" halves did nothing.
        //
        // So fall back to the checker's own filter before falling back to
        // `Any`, which makes the two agree by construction wherever the
        // primary walker is silent. `Any` stays the last resort for a bare
        // `Selector::Target(0)` ("any target"), which genuinely accepts
        // anything and has no slot filter either.
        //
        // Both calls are the *mode-blind* ones, deliberately.
        // `auto_targets_for_effect_all_slots_kicked` reaches this picker only
        // when the **chosen mode's** slot 0 has no filter, and CR 608.2b's
        // re-check is mode-blind too — so aiming with the same call the
        // checker uses is the whole point. Mode-aware picking already happens
        // one level up, through `target_filter_for_slot_in_mode_kicked`.
        let req_owned = eff
            .primary_target_filter()
            .or_else(|| eff.target_filter_for_slot(0))
            .map(|f| f.resolve_x(x).resolve_converge(converge));
        let req = req_owned.as_ref().unwrap_or(&any_filter);
        // First opponent on a different team. Falls back to the next
        // seat in singleton-team / unknown-team cases so the legacy 1v1
        // pick (`(controller + 1) % n`) is preserved.
        let opp = self
            .first_opponent_of(controller)
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
        // The battlefield form of the same question. Every walk below already
        // holds the permanent, and the plain form re-finds it by id —
        // `evaluate_requirement_static`'s own `battlefield_find` is 1.7 % of
        // simulator instructions on its own (pass 53).
        let gates = crate::game::effects::eval::PrintedGates::default();
        let is_legal_bf = |c: &CardInstance| -> bool {
            self.requirement_on_permanent(req, c, controller, avoid_source, &gates)
                && self.check_target_legality(&Target::Permanent(c.id), controller).is_ok()
        };
        // CR 702.21 — a hostile permanent with a non-trivial Ward gets the
        // targeting spell/ability countered unless the actor pays, and the
        // engine pays wards from a (typically empty) floating pool. Prefer
        // un-warded candidates; warded ones stay as a fallback so effects
        // never fizzle outright.
        let hostile_ward = |cid: CardId| -> bool { self.has_hostile_ward(cid, controller) };

        // CR 601.2c — Flagbearer: an opponent's Flagbearer must be chosen when
        // the slot can take it, so it outranks every other candidate here.
        for fb in self.flagbearer_candidates(controller) {
            let t = Target::Permanent(fb);
            if is_legal(&t) {
                return Some(t);
            }
        }

        if accepts_player {
            let player_primary = Target::Player(primary_player);
            if is_legal(&player_primary) { return Some(player_primary); }
            let player_secondary = Target::Player(secondary_player);
            if is_legal(&player_secondary) { return Some(player_secondary); }
        }

        // Stack walk for counter-class effects (Spell Queller, Mystic
        // Snake, Venser-style ETB / triggered counters whose target is a
        // spell on the stack). The battlefield/graveyard walks below
        // never consider stack objects, so without this pass an
        // auto-targeted "counter target spell" trigger fizzles for lack
        // of a legal target. We prefer the topmost spell *not* cast by
        // the controller (you counter the opponent's spell), falling
        // back to any legal stack spell. Gated on the requirement being
        // exactly `IsSpellOnStack` so it only fires for genuine
        // counter-class targets — effects with a looser filter (e.g. a
        // magecraft "exile a card from your graveyard" with an `Any` /
        // `Nonland` filter) must not grab the just-cast spell sitting on
        // the stack.
        // The gate fires for a bare `IsSpellOnStack` *and* for compound
        // filters that narrow it (`And(IsSpellOnStack, Creature)` — "counter
        // target creature spell"); `is_legal` still applies the full filter,
        // so widening the gate only decides whether to walk the stack.
        fn mentions_spell_on_stack(r: &crate::card::SelectionRequirement) -> bool {
            use crate::card::SelectionRequirement as R;
            match r {
                R::IsSpellOnStack
                | R::SpellTargetsControllerOrControlled
                | R::SpellTargetsCreature => true,
                R::And(a, b) | R::Or(a, b) => {
                    mentions_spell_on_stack(a) || mentions_spell_on_stack(b)
                }
                _ => false,
            }
        }
        if mentions_spell_on_stack(req) {
            use crate::game::types::StackItem;
            // Topmost first: iterate the stack in reverse.
            let mut hostile: Option<Target> = None;
            let mut friendly: Option<Target> = None;
            for si in self.stack.iter().rev() {
                if let StackItem::Spell { card, caster, .. } = si {
                    let t = Target::Permanent(card.id);
                    if is_legal(&t) {
                        if *caster != controller {
                            hostile = Some(t);
                            break;
                        } else if friendly.is_none() {
                            friendly = Some(t);
                        }
                    }
                }
            }
            if let Some(t) = hostile.or(friendly) {
                return Some(t);
            }
        }

        // Graveyard-target effects: walk primary player's graveyard first,
        // then secondary's. Reanimate/Disentomb (friendly) hits the caster's
        // graveyard; Ghost Vacuum (hostile) hits the opp's. Falls through
        // to the battlefield walk below if no graveyard match.
        if prefer_graveyard {
            // The `avoid` set matters here too: a multi-slot trigger fanning
            // out over a graveyard (Celestial Gatekeeper's "up to two target
            // Bird and/or Cleric cards") re-picked the card it already claimed
            // and stalled after one slot.
            for &p in &[primary_player, secondary_player] {
                if let Some(c) = self.players[p]
                    .graveyard
                    .iter()
                    .filter(|c| !avoid.contains(&c.id))
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
        let is_avoided = |cid: CardId| -> bool { avoid.contains(&cid) };
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
                .filter(|c| is_legal_bf(c))
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
        } else {
            // Hostile pick: un-warded first, then the biggest threat.
            // The power term was missing until 2026-08-22, so removal took
            // whichever legal enemy body happened to sit earliest on the
            // board — a recorded game spent Grapple with Death on a 2/2
            // utility creature with a 3/3 and a five-drop beside it. The
            // friendly branch above has always picked its best target;
            // only the hostile side was arbitrary.
            primary_candidates
                .sort_by_cached_key(|c| (hostile_ward(c.0), std::cmp::Reverse(c.1)));
        }
        if let Some(&(cid, _)) = primary_candidates.first() {
            return Some(Target::Permanent(cid));
        }
        for pass_warded in [false, true] {
            if let Some(t) = self
                .battlefield
                .iter()
                .filter(|c| !is_avoided(c.id))
                .filter(|c| pass_warded || !hostile_ward(c.id))
                .find(|c| is_legal_bf(c))
                .map(|c| Target::Permanent(c.id))
            {
                return Some(t);
            }
        }
        // Source-fallback: only the avoided source is a legal candidate.
        // Pick it as a last resort so the trigger doesn't fizzle entirely.
        if let Some(t) = self
            .battlefield
            .iter()
            .filter(|c| c.controller == primary_player)
            .find(|c| is_legal_bf(c))
            .map(|c| Target::Permanent(c.id))
        {
            return Some(t);
        }
        if let Some(t) = self
            .battlefield
            .iter()
            .find(|c| is_legal_bf(c))
            .map(|c| Target::Permanent(c.id))
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
        //
        // **Gated on the effect being able to reach off the board.** The
        // filter language has no zone predicate, so a board-shaped
        // requirement ("target creature", a bare `Not(Player)`) is satisfied
        // by a creature *card* in a graveyard or in exile. Ungated, a
        // "destroy target creature" trigger with no legal battlefield
        // creature aimed at an exiled card and then silently fizzled at
        // resolution — in the training path only, where `wants_ui` is false
        // and no modal is ever posed, so no instrument watched it. Resolving
        // targetless is the right answer there.
        //
        // `may_target_offboard_card` is the effect-shape half (every zone
        // change, not just the reanimation-shaped ones — walk *order* stays
        // `prefers_graveyard_target`'s narrower question) and
        // `mentions_offboard_zone` the filter half, which is the gate the
        // `ChooseCards` modal path already takes. See ENGINE_BACKLOG's "the
        // target enumerator is zone-blind".
        if !eff.may_target_offboard_card() && !req.mentions_offboard_zone() {
            return None;
        }
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

    /// Enumerate every legal slot-0 target for `eff` from the
    /// perspective of `controller`. Used by the UI trigger-target
    /// picker — when a wants_ui controller is about to push a
    /// targeted trigger, the engine surfaces a `Decision::ChooseTarget`
    /// listing all of these so the player can pick.
    ///
    /// Order: players (controller first, then opponents), then
    /// battlefield permanents (in battlefield iteration order), then
    /// each graveyard (controller first), then exile. This matches
    /// the auto-picker's traversal order but accumulates instead of
    /// returning on first hit.
    pub fn enumerate_legal_targets(
        &self,
        eff: &crate::effect::Effect,
        controller: usize,
    ) -> Vec<Target> {
        self.enumerate_legal_targets_with_source(eff, controller, None)
    }

    /// As `enumerate_legal_targets`, but source-relative filter clauses
    /// (`OtherThanSource`, counter-relative MV gates) evaluate against
    /// `source` instead of silently passing — the trigger-queue picker
    /// passes the triggering permanent so an "other target creature"
    /// prompt doesn't offer the source itself.
    pub fn enumerate_legal_targets_with_source(
        &self,
        eff: &crate::effect::Effect,
        controller: usize,
        source: Option<CardId>,
    ) -> Vec<Target> {
        self.enumerate_legal_targets_xc(eff, controller, source, 0, 0)
    }

    /// [`enumerate_legal_targets_with_source`] concretizing the cast's `{X}`
    /// and converge count, exactly as
    /// [`auto_target_for_effect_avoiding_set_xc`] does.
    ///
    /// The two must agree: this list is what a `wants_ui` controller is
    /// offered, and the picker is what everyone else gets. Left unresolved,
    /// `ManaValueAtMostXFromCost` / `ManaValueAtMostConverged` match nothing
    /// at enumeration time, so a UI player was told a targeted ETB had no
    /// legal targets and it resolved as a no-op — Sundering Archaic's
    /// converge exile silently did nothing for the human and worked for the
    /// bot.
    ///
    /// [`enumerate_legal_targets_with_source`]: Self::enumerate_legal_targets_with_source
    /// [`auto_target_for_effect_avoiding_set_xc`]: Self::auto_target_for_effect_avoiding_set_xc
    pub fn enumerate_legal_targets_xc(
        &self,
        eff: &crate::effect::Effect,
        controller: usize,
        source: Option<CardId>,
        x: u32,
        converge: u32,
    ) -> Vec<Target> {
        use crate::card::SelectionRequirement;
        let any_filter = SelectionRequirement::Any;
        let req_owned =
            eff.primary_target_filter().map(|f| f.resolve_x(x).resolve_converge(converge));
        let req = req_owned.as_ref().unwrap_or(&any_filter);
        // Same scope gate as the auto-picker's off-board fallback, so the
        // enumerated set and the picked target can't disagree about which
        // zones the effect reaches.
        let offboard = eff.may_target_offboard_card() || req.mentions_offboard_zone();
        self.legal_targets_for_filter_scoped(
            req,
            eff.accepts_player_target(),
            controller,
            source,
            offboard,
        )
    }

    /// The graveyard / exile cards a `wants_ui` controller may pick for an
    /// off-board slot-0 target ("target card in a graveyard" — Sundering
    /// Archaic's `{2}`), as `(id, name)` pairs for the `ChooseCards` modal.
    ///
    /// The activation path poses that modal and the view layer greys the
    /// ability row out when this comes back empty, so both must agree on what
    /// counts as a candidate — a row that looks live but is rejected on click
    /// reads as a dead button.
    pub fn offboard_target_candidates(
        &self,
        filter: &crate::card::SelectionRequirement,
        controller: usize,
        source: CardId,
    ) -> Vec<(CardId, String)> {
        self.players
            .iter()
            .flat_map(|pl| pl.graveyard.iter())
            .chain(self.exile.iter())
            .filter(|c| {
                c.id != source
                    && self.evaluate_requirement_static(
                        filter,
                        &Target::Permanent(c.id),
                        controller,
                        Some(source),
                    )
            })
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect()
    }

    /// Every object/player in any zone that satisfies `req` and passes the
    /// CR 115.4 legality check. The filter-level core of
    /// `enumerate_legal_targets_with_source`; a caller that already knows the
    /// slot's own filter (CR 115.7c retargeting) uses it directly.
    ///
    /// The scan is `&self` and asks every candidate the same layer-aware
    /// questions — `evaluate_requirement_static`'s layer-4 reads and
    /// `check_target_legality`'s Shroud/Hexproof reads — so one freeze covers
    /// the whole walk. Without it `check_target_legality` opens its own scope
    /// per candidate and each one re-gathers every continuous effect in the
    /// game.
    pub fn legal_targets_for_filter(
        &self,
        req: &crate::card::SelectionRequirement,
        accepts_player: bool,
        controller: usize,
        source: Option<CardId>,
    ) -> Vec<Target> {
        let offboard = req.mentions_offboard_zone();
        self.legal_targets_for_filter_scoped(req, accepts_player, controller, source, offboard)
    }

    /// [`legal_targets_for_filter`](Self::legal_targets_for_filter) with the
    /// off-board walk decided by the caller.
    ///
    /// **The filter language has no zone predicate**, so a board-shaped
    /// requirement ("target creature", a bare `Not(Player)`, `Any`) is
    /// satisfied by a creature *card* in a graveyard or in exile. Walking
    /// those zones unconditionally offered candidates the resolver then
    /// rejects — Cuombajj Witches' "any target" listed every card in every
    /// graveyard — and left every caller to separate the two by hand.
    /// `offboard` is that question asked once, and the auto-picker asks it the
    /// same way (`may_target_offboard_card || mentions_offboard_zone`): the
    /// two walks must agree on scope or the UI path and the training path
    /// target different sets.
    pub fn legal_targets_for_filter_scoped(
        &self,
        req: &crate::card::SelectionRequirement,
        accepts_player: bool,
        controller: usize,
        source: Option<CardId>,
        offboard: bool,
    ) -> Vec<Target> {
        self.with_frozen_layers(|s| {
            s.legal_targets_for_filter_inner(req, accepts_player, controller, source, offboard)
        })
    }

    fn legal_targets_for_filter_inner(
        &self,
        req: &crate::card::SelectionRequirement,
        accepts_player: bool,
        controller: usize,
        source: Option<CardId>,
        offboard: bool,
    ) -> Vec<Target> {
        let is_legal = |t: &Target| -> bool {
            self.evaluate_requirement_static(req, t, controller, source)
                && self.check_target_legality(t, controller).is_ok()
        };
        // See `auto_target_for_effect_avoiding_set_xc_inner` — the
        // battlefield walk hands the permanent over rather than making
        // `evaluate_requirement_static` re-find it.
        let gates = crate::game::effects::eval::PrintedGates::default();
        let is_legal_bf = |c: &CardInstance| -> bool {
            self.requirement_on_permanent(req, c, controller, source, &gates)
                && self.check_target_legality(&Target::Permanent(c.id), controller).is_ok()
        };

        let mut out: Vec<Target> = Vec::new();
        if accepts_player {
            // Caster first, then each other seat in turn order.
            let n = self.players.len();
            for offset in 0..n {
                let seat = (controller + offset) % n;
                let t = Target::Player(seat);
                if is_legal(&t) {
                    out.push(t);
                }
            }
        }
        for c in &self.battlefield {
            if is_legal_bf(c) {
                out.push(Target::Permanent(c.id));
            }
        }
        if !offboard {
            return out;
        }
        // Graveyards: walk controller's first for graveyard-friendly
        // effects (Reanimate, Disentomb), then others.
        let n = self.players.len();
        for offset in 0..n {
            let seat = (controller + offset) % n;
            for c in &self.players[seat].graveyard {
                let t = Target::Permanent(c.id);
                if is_legal(&t) {
                    out.push(t);
                }
            }
        }
        for c in &self.exile {
            let t = Target::Permanent(c.id);
            if is_legal(&t) {
                out.push(t);
            }
        }
        out
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
    /// CR 702.21 — does `cid` belong to someone other than `actor` and
    /// carry a non-trivial Ward? Targeting it gets the spell countered
    /// unless the tax is paid, so both auto-target walks rank warded
    /// hostile permanents below un-warded ones.
    pub(crate) fn has_hostile_ward(&self, cid: CardId, actor: usize) -> bool {
        use crate::card::Keyword;
        let Some(c) = self.battlefield.find_by_id(cid) else { return false };
        if c.controller == actor {
            return false;
        }
        let cp = self.computed_permanent(cid);
        let kws: &[Keyword] = match &cp {
            Some(cp) => cp.keywords(),
            None => &c.definition.keywords,
        };
        kws.iter().any(|k| matches!(k, Keyword::Ward(w)
            if !crate::game::actions::ward_cost_is_trivial(w)))
    }

    pub fn auto_targets_for_effect_all_slots(
        &self,
        eff: &Effect,
        controller: usize,
        mode: Option<usize>,
    ) -> (Option<Target>, Vec<Target>) {
        self.auto_targets_for_effect_all_slots_kicked(eff, controller, mode, false, None)
    }

    /// Source-aware variant: slot filters that read the ability's source
    /// (`ManaValueEqualsCountersOnSource` — Wishing Well's coin count) need it
    /// to resolve. Reflexive "when you do" bodies pass their `ctx.source`.
    pub fn auto_targets_for_effect_all_slots_sourced(
        &self,
        eff: &Effect,
        controller: usize,
        mode: Option<usize>,
        source: Option<CardId>,
    ) -> (Option<Target>, Vec<Target>) {
        self.auto_targets_for_effect_all_slots_kicked(eff, controller, mode, false, source)
    }

    /// Kicker-aware variant (CR 702.32): slot filters resolve to the
    /// `If(SpellWasKicked, …)` branch that matches `kicked`, so a bot
    /// preparing a `CastSpellKicked` aims at the kicked target set (Tear
    /// Asunder's nonland permanent rather than the base artifact/enchant).
    pub fn auto_targets_for_effect_all_slots_kicked(
        &self,
        eff: &Effect,
        controller: usize,
        mode: Option<usize>,
        kicked: bool,
        source: Option<CardId>,
    ) -> (Option<Target>, Vec<Target>) {
        // Slot 0 — if it carries its own numbered `TargetFiltered` filter
        // (Rabid Bite's friendly-creature power source lives in slot 0,
        // inside `Value::PowerOf`), pick it by that filter in the loop
        // below. Otherwise (bare `Target(0)` effects like Lightning Bolt)
        // use the source-aware heuristic picker.
        // The slot-0 lookup answers two questions — "is there a filter" and
        // "which one" — and the loop below asked it a second time for the
        // value. It is a walk of the whole effect tree, so keep the answer.
        let slot0_filter = eff.target_filter_for_slot_in_mode_kicked(0, mode, kicked);
        let mut slot_0 = if slot0_filter.is_some() {
            None
        } else {
            self.auto_target_for_effect_avoiding(eff, controller, None)
        };
        let mut additional = Vec::new();
        let mut already_picked: Vec<CardId> = Vec::new();
        let mut slot: u8 = if slot0_filter.is_some() { 0 } else { 1 };
        // Loop-invariant: the seat does not change between slots.
        let opp = self
            .first_opponent_of(controller)
            .unwrap_or((controller + 1) % self.players.len());
        // Cap at 16 slots — no real card uses more than 4, but cap defensively.
        while slot < 16 {
            // Borrowed, not cloned: the filter lives in `eff`, which outlives
            // this frame, and a `SelectionRequirement` clone deep-copies the
            // whole `And`/`Or` tree.
            let req = match if slot == 0 {
                slot0_filter
            } else {
                eff.target_filter_for_slot_in_mode_kicked(slot, mode, kicked)
            } {
                Some(r) => r,
                None => break,
            };
            let is_legal = |t: &Target| -> bool {
                self.evaluate_requirement_static(req, t, controller, source)
                    && self.check_target_legality(t, controller).is_ok()
            };
            // The battlefield candidate walk below already holds each
            // permanent; hand it over instead of re-finding it by id.
            let gates = crate::game::effects::eval::PrintedGates::default();
            let is_legal_bf = |c: &CardInstance| -> bool {
                self.requirement_on_permanent(req, c, controller, source, &gates)
                    && self.check_target_legality(&Target::Permanent(c.id), controller).is_ok()
            };
            let is_legal_gy = |c: &CardInstance| -> bool {
                self.requirement_on_graveyard_card(req, c, controller, source)
                    && self.check_target_legality(&Target::Permanent(c.id), controller).is_ok()
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
                // Graveyard-preferring effects (reanimate / regrow — Young
                // Necromancer's reflexive return) must not grab a battlefield
                // permanent that happens to match the filter; sweep
                // graveyards first for those.
                if found.is_none() && eff.prefers_graveyard_target() {
                    found = first_legal_graveyard_card(self, is_legal_gy);
                }
                // Battlefield: prefer one not already picked by slot 0 or
                // earlier slots to avoid double-targeting when the filter is
                // permissive.
                //
                // Candidates are ranked by *side* first, which this filtered
                // path did not do until 2026-08-22: it took the first legal
                // permanent in battlefield order, so a bare "destroy target
                // creature" aimed at the caster's own creature whenever that
                // creature happened to sit earlier on the board, and
                // Proctor's Gaze bounced the bot's own body in four recorded
                // games. The unfiltered picker
                // (`auto_target_for_effect_avoiding_set_xc`) already ranked
                // by side and ward; only this branch was blind. Friendly
                // effects overwhelmingly carry a "you control" clause in the
                // filter itself, so `is_legal` — not this ordering — is what
                // keeps a gift on the caster's side.
                if found.is_none() {
                    // Refilled per slot rather than collected per slot: the
                    // buffer outlives the loop, so the common empty case
                    // stops allocating.
                    already_picked.clear();
                    if let Some(Target::Permanent(id)) = slot_0 {
                        already_picked.push(id);
                    }
                    already_picked.extend(additional.iter().filter_map(|t| match t {
                        Target::Permanent(id) => Some(*id),
                        _ => None,
                    }));
                    // Per *slot*, not per effect: Homesickness is
                    // "target player draws two" + "tap and stun target
                    // creature", and the whole-effect classifier reads the
                    // gift and calls the whole spell friendly, which had the
                    // bot stunning its own board.
                    let prefer_friendly = eff.prefers_friendly_target_for_slot(slot, mode);
                    // 0 = wanted side, un-warded; 1 = wanted side, warded;
                    // 2 = the other side (a last resort, see `optional`).
                    let rank = |id: CardId, ctrl: usize| -> u8 {
                        let wanted = (ctrl == controller) == prefer_friendly;
                        match (wanted, self.has_hostile_ward(id, controller)) {
                            (true, false) => 0,
                            (true, true) => 1,
                            (false, _) => 2,
                        }
                    };
                    // CR: a *mandatory* target must be chosen when any legal
                    // one exists, even a self-damaging pick. An "up to N"
                    // slot (`min_targets: 0`) may simply be declined — which
                    // is what makes Proctor's Gaze fetch its land and bounce
                    // nothing when the opponent has no nonland permanent.
                    let optional =
                        slot >= eff.min_targets_in_mode(mode).unwrap_or(u8::MAX);
                    // Within a rank, the biggest body wins: ranking by
                    // side alone left removal taking whichever legal
                    // candidate sat earliest on the board.
                    // A plain loop, not `filter().filter().map().min()`: the
                    // chain is walked per battlefield permanent per slot and
                    // every element pays the adapters' `&mut F` forwarding
                    // (PERF (-78)'s test 1). `min()` keeps the first of equal
                    // keys and the key ends in the id, so ties cannot happen;
                    // `<` reproduces it either way.
                    let mut best: Option<(u8, i32, CardId)> = None;
                    for c in self.battlefield.iter() {
                        if already_picked.contains(&c.id) || !is_legal_bf(c) {
                            continue;
                        }
                        let power = self
                            .computed_permanent(c.id)
                            .map(|cp| cp.power)
                            .unwrap_or(c.definition.power);
                        let key = (rank(c.id, c.controller), -power, c.id);
                        if best.is_none_or(|b| key < b) {
                            best = Some(key);
                        }
                    }
                    match best {
                        // Never spend an optional slot on the wrong side.
                        Some((2, _, _)) if optional => {}
                        Some((_, _, id)) => found = Some(Target::Permanent(id)),
                        None if optional => {}
                        None => {
                            // Mandatory slot: allow reuse of an
                            // already-picked permanent rather than leave a
                            // required target unfilled.
                            found = self
                                .battlefield
                                .iter()
                                .find(|c| is_legal_bf(c))
                                .map(|c| Target::Permanent(c.id));
                        }
                    }
                }
                // Graveyard cards (e.g. an `InGraveyard` reflexive return —
                // Curious Forager's "return target permanent card from your
                // graveyard"). The battlefield walk above can't see them, so
                // sweep every graveyard as a last resort.
                if found.is_none() {
                    found = first_legal_graveyard_card(self, is_legal_gy);
                }
                found
            };
            match pick {
                Some(t) if slot == 0 => slot_0 = Some(t),
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

/// The first graveyard card, in seat order, that `is_legal` accepts.
///
/// Two call sites in `auto_targets_for_effect_all_slots_kicked` wrote this as
/// `players.iter().flat_map(..).map(..).find(..)`; the walk is per graveyard
/// card per slot and the adapter stack costs a `&mut F` forward per element
/// (PERF (-78)).
///
/// Takes the card, not its id: the requirement goes through
/// [`GameState::requirement_on_graveyard_card`], which skips the walker's
/// battlefield miss and re-find for a card the loop already holds (PERF
/// `(-185)`).
fn first_legal_graveyard_card(
    state: &GameState,
    is_legal: impl Fn(&CardInstance) -> bool,
) -> Option<Target> {
    for p in state.players.iter() {
        for c in p.graveyard.iter() {
            if is_legal(c) {
                return Some(Target::Permanent(c.id));
            }
        }
    }
    None
}
