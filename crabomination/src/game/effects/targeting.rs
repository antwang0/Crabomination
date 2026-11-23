//! Auto-target picker for effects that the engine resolves without explicit
//! user input (ETB triggers, attack triggers, bot-driven casts).

use crate::card::{CardId, CardInstance};
use crate::effect::Effect;
use crate::game::{GameState, Target};

impl GameState {
    /// The seat that aims a trigger: its controller, unless the body is "the
    /// active player may … target …" (Curse of Inertia — the attacking
    /// player picks the permanent of their choice).
    pub(crate) fn trigger_target_chooser(&self, eff: &Effect, controller: usize) -> usize {
        match eff {
            Effect::MayDoBy { who: crate::effect::PlayerRef::ActivePlayer, body, .. }
                if body.requires_target() && self.players.get(self.active_player_idx).is_some_and(|p| p.is_alive()) =>
            {
                self.active_player_idx
            }
            _ => controller,
        }
    }

    /// Pick a legal target for an effect that requires one, when the engine
    /// fires a trigger without user input (ETB, attack trigger, ...). `None` if
    /// the effect takes no target or none is legal.
    ///
    /// Targets must satisfy both the selector requirement AND targeting
    /// legality (Hexproof / Shroud / Protection / Leyline of Sanctity);
    /// otherwise the bot aims at an illegal target, `cast_spell` rejects it,
    /// and a spectated match deadlocks.
    pub fn auto_target_for_effect(&self, eff: &Effect, controller: usize) -> Option<Target> {
        self.auto_target_for_effect_avoiding(eff, controller, None)
    }

    /// [`auto_target_for_effect`](Self::auto_target_for_effect) for an
    /// ability of `source`: a pick that has protection from the source (CR
    /// 702.16b) is passed over for the next, rather than handed to an
    /// activation the engine then rejects while another target was legal.
    pub fn auto_target_for_ability(&self, eff: &Effect, controller: usize, source: CardId) -> Option<Target> {
        let mut t = self.auto_target_for_effect(eff, controller)?;
        // A retry's avoid set leads with the source: its first entry is read
        // as the trigger source, and the protected picks ride behind it. Its
        // fallback hands an avoided one back when nothing else is legal.
        let mut avoid = vec![source];
        loop {
            if !self.ability_target_has_protection(&t, source) {
                return Some(t);
            }
            let Target::Permanent(id) = t else { return None };
            if avoid.contains(&id) || avoid.len() > 8 {
                return None;
            }
            avoid.push(id);
            t = self.auto_target_for_effect_avoiding_set(eff, controller, &avoid)?;
        }
    }

    /// Source-aware auto-target picker: with `avoid_source` set, any *other*
    /// legal candidate beats the avoided permanent, which is the fallback only.
    /// For Magecraft/Repartee triggers, where the source (a 1/1 utility
    /// creature) is rarely the right pump target.
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
    /// `ManaValueAtMostConverged` against the cast's converge count (Sundering
    /// Archaic). Unconcretized, the atom is false for everything at selection
    /// and the picker can aim at over-cap permanents that then fizzle.
    pub fn auto_target_for_effect_avoiding_set_xc(
        &self,
        eff: &Effect,
        controller: usize,
        avoid: &[CardId],
        x: u32,
        converge: u32,
    ) -> Option<Target> {
        if let Some(pick) = self.own_blink_pick(eff, controller, avoid) {
            return pick;
        }
        if let Some((fighter, _)) = self.fight_pair_pick(eff, controller, avoid.first().copied(), None) {
            return fighter;
        }
        // One freeze over the whole candidate walk — see
        // `legal_targets_for_filter`, which the picker mirrors.
        self.with_frozen_layers(|s| {
            s.auto_target_for_effect_avoiding_set_xc_inner(eff, controller, avoid, x, converge)
        })
        .filter(|t| !self.declines_own_side_pick(eff, controller, t) && !self.declines_gift_pick(eff, controller, t))
    }

    /// CR 601.2c — an optional blink of your own permanent ("exile up to one
    /// other target creature you control, then return that card" — Flickering
    /// Hound; the end-step blinkers): `Some(pick)` for such a slot, `None` for
    /// any other. The returned card is a new object (CR 400.7): tokens never
    /// come back (CR 111.7) and counters and attachments are lost, so those
    /// are never blinked. Preference: one with its own enters trigger, then a
    /// tapped one (returns untapped), then any other — except a creature that
    /// would lose its attack (untapped, able to attack, precombat on its
    /// controller's turn). With none, the slot stays empty. The general
    /// own-side pick blinked the biggest creature instead.
    fn own_blink_pick(&self, eff: &Effect, controller: usize, avoid: &[CardId]) -> Option<Option<Target>> {
        use crate::effect::{EventKind, EventScope};
        use crate::game::types::TurnStep;
        let body = match eff {
            Effect::OptionalTargets { body, .. } => &**body,
            other => other,
        };
        if !matches!(body, Effect::ExileAndReturnToOwner { .. }) || !eff.target_slot_optional(0, None) {
            return None;
        }
        let filter = eff.target_filter_for_slot(0)?;
        if !filter.excludes_opponents_side() {
            return None;
        }
        let source = avoid.first().copied();
        let precombat = self.active_player_idx == controller
            && matches!(
                self.step,
                TurnStep::Untap | TurnStep::Upkeep | TurnStep::Draw | TurnStep::PreCombatMain | TurnStep::BeginCombat
            );
        // `None`: never blink it; else its preference (higher first).
        let rank = |c: &CardInstance| -> Option<u8> {
            let attached = self.battlefield.iter().any(|a| a.attached_to == Some(c.id));
            if c.is_token || c.total_counters() > 0 || attached {
                return None;
            }
            let etb = c
                .definition
                .triggered_abilities
                .iter()
                .any(|t| t.event.kind == EventKind::EntersBattlefield && t.event.scope == EventScope::SelfSource);
            // An enters trigger is worth the attack it costs; nothing else is.
            if etb {
                return Some(2);
            }
            let loses_attack = precombat && self.computed_is_creature(c) && !c.tapped && !c.summoning_sick;
            (!loses_attack).then_some(u8::from(c.tapped))
        };
        let pick = self
            .battlefield
            .iter()
            .filter(|c| c.controller == controller && !avoid.contains(&c.id))
            .filter_map(|c| Some((rank(c)?, c)))
            .filter(|(_, c)| {
                let t = Target::Permanent(c.id);
                self.evaluate_requirement_static(filter, &t, controller, source)
                    && self.check_target_legality(&t, controller).is_ok()
            })
            .max_by_key(|(r, c)| (*r, c.definition.cost.cmc(), std::cmp::Reverse(c.id)))
            .map(|(_, c)| Target::Permanent(c.id));
        Some(pick)
    }

    /// The open trigger-target ask (`ResumeContext::TriggerTargetPick`) as
    /// [`own_blink_pick`](Self::own_blink_pick) and
    /// [`fight_pair_pick`](Self::fight_pair_pick) would fill it: `Some(pick)`
    /// (`Some(None)` = leave the slot empty) for an own-side blink slot or
    /// either slot of an optional fight, `None` for any other ask. A prompting
    /// seat — every bot seat in a simulated game (`build_match_template`) —
    /// answers trigger targets itself and never reaches the engine's fill, so
    /// the bot must ask this first.
    pub fn trigger_slot_pick(&self) -> Option<Option<Target>> {
        let pd = self.pending_decision.as_ref()?;
        let crate::game::types::ResumeContext::TriggerTargetPick { pending, picked, .. } = &pd.resume else {
            return None;
        };
        let (eff, source, controller) = (&pending.effect, pending.source, pending.controller);
        match picked.first() {
            None => self
                .own_blink_pick(eff, controller, &[source])
                .or_else(|| self.fight_pair_pick(eff, controller, Some(source), None).map(|(f, _)| f)),
            Some(Target::Permanent(f)) if picked.len() == 1 => self
                .fight_pair_pick(eff, controller, Some(source), Some(*f))
                .map(|(_, rest)| rest.first().cloned()),
            _ => None,
        }
    }

    /// CR 701.14 — "target creature you control fights up to one target
    /// creature an opponent controls" (Yoshimaru, Scrappy Stray): the pair is
    /// chosen together, by the fight's outcome. A kill your fighter survives
    /// beats a trade, a trade is taken only for a creature worth at least as
    /// much as yours, and with neither the optional slot stays empty. `None`
    /// for any other effect. (Slot by slot, a 2/2 fought a 6/4 for nothing.)
    pub(crate) fn fight_pair_pick(
        &self,
        eff: &Effect,
        controller: usize,
        source: Option<CardId>,
        fighter: Option<CardId>,
    ) -> Option<(Option<Target>, Vec<Target>)> {
        use crate::card::{Keyword, KeywordSlice};
        use crate::effect::Selector;
        let body = match eff {
            Effect::OptionalTargets { body, .. } => &**body,
            other => other,
        };
        let Effect::Fight {
            attacker: Selector::TargetFiltered { slot: 0, filter: mine },
            defender: Selector::TargetFiltered { slot: 1, filter: theirs },
        } = body
        else {
            return None;
        };
        if eff.target_slot_optional(0, None) || !eff.target_slot_optional(1, None) {
            return None;
        }
        let legal = |f: &crate::card::SelectionRequirement, c: &CardInstance| {
            let t = Target::Permanent(c.id);
            self.evaluate_requirement_static(f, &t, controller, source) && self.check_target_legality(&t, controller).is_ok()
        };
        // (power, toughness left, deathtouch, indestructible, worth).
        let stats = |c: &CardInstance| {
            let cp = self.computed_permanent(c.id)?;
            let kws = cp.keywords();
            Some((
                cp.power.max(0),
                cp.toughness - c.damage as i32,
                kws.has_kw(&Keyword::Deathtouch),
                kws.has_kw(&Keyword::Indestructible),
                c.definition.cost.cmc() as i32 + cp.power + cp.toughness,
            ))
        };
        let kills = |(p, _, dt, _, _): (i32, i32, bool, bool, i32), (_, t, _, ind, _): (i32, i32, bool, bool, i32)| {
            !ind && p > 0 && (p >= t || dt)
        };
        let fighters: Vec<(&CardInstance, _)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == controller && fighter.is_none_or(|f| f == c.id) && legal(mine, c))
            .filter_map(|c| Some((c, stats(c)?)))
            .collect();
        let defenders: Vec<(&CardInstance, _)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller != controller && legal(theirs, c) && !self.has_hostile_ward(c.id, controller))
            .filter_map(|c| Some((c, stats(c)?)))
            .collect();
        let mut best: Option<((i32, i32), CardId, CardId)> = None;
        for &(f, fs) in &fighters {
            for &(d, ds) in &defenders {
                if !kills(fs, ds) {
                    continue;
                }
                let score = if !kills(ds, fs) {
                    (2, ds.4 - fs.4 / 4)
                } else if ds.4 >= fs.4 {
                    (1, ds.4 - fs.4)
                } else {
                    continue;
                };
                if best.as_ref().is_none_or(|b| score > b.0) {
                    best = Some((score, f.id, d.id));
                }
            }
        }
        Some(match best {
            Some((_, f, d)) => (Some(Target::Permanent(f)), vec![Target::Permanent(d)]),
            // No fight worth having: slot 0 is still required, so name the
            // least valuable fighter and leave the defender out.
            None => (fighters.iter().min_by_key(|(c, s)| (s.4, c.id)).map(|(c, _)| Target::Permanent(c.id)), Vec::new()),
        })
    }

    /// CR 601.2c — the mirror of [`declines_own_side_pick`]: the leading
    /// optional friendly group of a `SlotGroups` ("a +1/+1 counter on each of
    /// any number of target creatures", Filigree Vector) leaves an opponent's
    /// permanent unpicked rather than handing it a counter. A bare optional
    /// slot reaches the picker peeled of its context — a dual-use one (Atomic
    /// Microsizer) or a per-player one (Nils) — so it keeps its pick.
    fn declines_gift_pick(&self, eff: &Effect, controller: usize, t: &Target) -> bool {
        let Effect::SlotGroups(v) = eff else { return false };
        let leads_friendly = v.iter().find(|m| m.requires_target()).is_some_and(|m| {
            matches!(m, Effect::ApplyToTargets { min_targets: 0, effect, .. }
                if effect.prefers_friendly_target() && !effect.permanent_slot_is_hostile(0, None))
        });
        let Target::Permanent(tid) = t else { return false };
        leads_friendly && self.battlefield_find(*tid).is_some_and(|c| !self.same_team(c.controller, controller))
    }

    /// CR 601.2c — "up to one target": a removal-shaped slot that could name an
    /// opponent's permanent is declined when the only legal pick is its own
    /// side's (the walk's last resort, which a mandatory slot must take, had
    /// Aerial Extortionist exile its controller's Mind Stone). A slot that only
    /// names your own ("target land you control") keeps its pick. Your own
    /// graveyard counts as your side (Raven Eagle).
    fn declines_own_side_pick(&self, eff: &Effect, controller: usize, t: &Target) -> bool {
        let Target::Permanent(tid) = t else { return false };
        if !eff.target_slot_optional(0, None)
            || eff.target_filter_for_slot(0).is_none_or(|f| f.excludes_opponents_side())
        {
            return false;
        }
        if let Some(c) = self.battlefield_find(*tid) {
            return eff.permanent_slot_is_hostile(0, None) && self.same_team(c.controller, controller);
        }
        // A spell on the stack: a hostile slot spares your own side's (Subtlety).
        if let Some(caster) = self.stack.iter().find_map(|si| match si {
            crate::game::types::StackItem::Spell { card, caster, .. } if card.id == *tid => Some(*caster),
            _ => None,
        }) {
            return eff.permanent_slot_is_hostile(0, None) && self.same_team(caster, controller);
        }
        // A graveyard card: only an exile is hostile (a return to hand is a
        // regrowth, not a bounce).
        eff.graveyard_slot_is_hostile(0, None)
            && self
                .players
                .iter()
                .position(|p| p.graveyard.iter().any(|c| c.id == *tid))
                .is_some_and(|side| self.same_team(side, controller))
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
        // `primary_target_filter` (this picker) and `target_filter_for_slot`
        // (the CR 608.2b re-check at resolution) are two independent walks
        // over ~1000 variants. A variant with an arm only in the slot walker
        // aimed at `Any` here and was then thrown away by its own filter — a
        // silent fizzle (creature Haunt: Absolver Thrull, Orzhov Euthanist).
        // So fall back to the checker's own filter before `Any`, which makes
        // the two agree by construction; `Any` stays the last resort for a
        // bare `Selector::Target(0)` ("any target").
        //
        // Both calls are deliberately mode-blind: this picker is reached only
        // when the chosen mode's slot 0 has no filter, and the CR 608.2b
        // re-check is mode-blind too. Mode-aware picking happens one level up
        // (`target_filter_for_slot_in_mode_kicked`).
        // A tree naming neither X nor converge is BORROWED rather than
        // rewritten into an identical copy — PERF `(-358)`.
        let req_src = eff.primary_target_filter().or_else(|| eff.target_filter_for_slot(0));
        let req_owned = req_src
            .filter(|f| f.names_x_or_converge())
            .map(|f| f.resolve_x(x).resolve_converge(converge));
        debug_assert!(
            req_owned.is_some()
                || req_src.is_none_or(|f| *f == f.resolve_x(x).resolve_converge(converge)),
            "names_x_or_converge said no but the rewrite moved the filter: {req_src:?}",
        );
        let req = req_owned.as_ref().or(req_src).unwrap_or(&any_filter);
        // The opponent to aim at when the choice is open and nobody makes it,
        // ranked (commander-damage race, lowest life, fewest untapped
        // blockers) rather than positional: `first_opponent_of` doesn't skip
        // the eliminated, and a positional answer made every pod a ring of
        // one-way spells. `default_hostile_opponent` short-circuits to a lone
        // candidate, so a duel is unchanged. Falls back to the next seat in
        // singleton / unknown-team cases.
        let opp = self
            .default_hostile_opponent(controller)
            .or_else(|| self.first_opponent_of(controller))
            .unwrap_or((controller + 1) % self.players.len());
        let prefer_friendly = eff.prefers_friendly_target();
        // `prefers_graveyard_target` covers reanimation and graveyard hate
        // (Ghost Vacuum): walk graveyards BEFORE the battlefield so an
        // `Any`-filtered Move-to-Exile doesn't grab a permanent. A filter
        // naming a permanent (`.and(R::Permanent)`, CR 109.2) never takes a
        // graveyard card (`target_out_of_zone`), so it skips the sweep.
        let prefer_graveyard = eff.prefers_graveyard_target() && !req.names_permanent();
        // Skip players when the effect operates on permanents/stack, or an
        // `Any`-filtered Move (Regrowth) targets the caster and fizzles. A
        // player-only slot-0 filter wins, though: the slot can be a player
        // while the body works on permanents (Mudhole's `CardsInZone { who:
        // Target(0) }`, the "enchant player" Curses). `accepts_player_target`
        // folds that slot read in itself, so this site and the enumerator's
        // can't drift.
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

        // Gathering Auras and Equipment onto "target permanent or player"
        // (Ardenn) only moves Curses onto a player; the headless pick wants
        // a creature, so it skips the player slots.
        let gathers = prefer_friendly
            && accepts_player
            && eff.any_nested(&|e| matches!(e, Effect::AttachAnyNumberTo { .. }));
        // A counter effect on a player wants a player with the right counter.
        let aim = if accepts_player { counter_aim(eff) } else { CounterAim::None };
        let player_ok = |t: &Target| is_legal(t) && !self.counter_player_skip(aim, t, controller);
        if accepts_player && !gathers {
            let player_primary = Target::Player(primary_player);
            if player_ok(&player_primary) { return Some(player_primary); }
            if prefer_friendly {
                let player_secondary = Target::Player(secondary_player);
                if player_ok(&player_secondary) { return Some(player_secondary); }
            } else if let Some(t) = self
                .living_seats()
                .filter(|&p| p != primary_player && p != controller && !self.same_team(p, controller))
                .map(Target::Player)
                .find(|t| player_ok(t))
            {
                return Some(t);
            }
        }

        // Stack walk for counter-class effects (Spell Queller, Mystic Snake):
        // the battlefield/graveyard walks below never see stack objects.
        // Prefers the topmost spell not cast by the controller, then any
        // legal stack spell. Gated on the filter naming `IsSpellOnStack`
        // (bare, or narrowed: `And(IsSpellOnStack, Creature)`), so a looser
        // filter (a magecraft "exile a card from your graveyard" with
        // `Any`) can't grab the just-cast spell; `is_legal` still applies
        // the full filter.
        fn mentions_spell_on_stack(r: &crate::card::SelectionRequirement) -> bool {
            use crate::card::SelectionRequirement as R;
            match r {
                R::IsSpellOnStack
                | R::SpellTargetsControllerOrControlled
                | R::SpellTargetsCreature
                // Radiant Performer's "target spell that targets only a
                // single permanent or player" — its entry trigger found no
                // target at all without the stack walk.
                | R::SpellWithSingleTarget
                | R::SpellWithAnyTarget => true,
                R::And(a, b) | R::Or(a, b) => {
                    mentions_spell_on_stack(a) || mentions_spell_on_stack(b)
                }
                _ => false,
            }
        }
        /// Whether `r` admits some object that isn't a spell on the stack.
        fn admits_off_stack(r: &crate::card::SelectionRequirement) -> bool {
            use crate::card::SelectionRequirement as R;
            match r {
                R::IsSpellOnStack
                | R::SpellTargetsControllerOrControlled
                | R::SpellTargetsCreature
                | R::SpellWithSingleTarget
                | R::SpellWithAnyTarget => false,
                R::Or(a, b) => admits_off_stack(a) || admits_off_stack(b),
                R::And(a, b) => admits_off_stack(a) && admits_off_stack(b),
                _ => true,
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
            // A slot that also takes a permanent ("target spell or nonland
            // permanent" — Press the Enemy, Venser) never settles for its own
            // controller's spell: the battlefield walk after this one picks.
            if let Some(t) = hostile.or(friendly.filter(|_| prefer_friendly || !admits_off_stack(req))) {
                return Some(t);
            }
        }
        // CR 115.1 — "target activated or triggered ability": each stack
        // ability by its own id, topmost first, the side the effect favours
        // first (Stifle an opponent's, Strionic Resonator one of yours).
        if req.mentions_stack_ability() {
            let cands = self.stack_ability_targets(req, controller, avoid_source);
            let side = |t: &Target| {
                let Target::Permanent(aid) = t else { return false };
                let mine = self.stack_ability_pos(*aid, None).is_some_and(|i| {
                    matches!(&self.stack[i], crate::game::types::StackItem::Trigger { controller: c, .. } if *c == controller)
                });
                mine == prefer_friendly
            };
            let legal = |t: &&Target| self.check_target_legality(t, controller).is_ok();
            if let Some(t) = cands.iter().filter(legal).find(|t| side(t)).or_else(|| cands.iter().find(legal)) {
                return Some(t.clone());
            }
        }

        // Graveyard-target effects: Reanimate/Disentomb (friendly) hit the
        // caster's graveyard; Ghost Vacuum (hostile) hits an opponent's
        // (`best_graveyard_card`). Falls through to the battlefield walk
        // below if no graveyard match.
        if prefer_graveyard {
            // The `avoid` set matters here too: a multi-slot trigger fanning
            // out over a graveyard (Celestial Gatekeeper's "up to two target
            // Bird and/or Cleric cards") re-picked the card it already claimed
            // and stalled after one slot.
            let pick = best_graveyard_card(self, controller, opp, prefer_friendly, false, |c| {
                !avoid.contains(&c.id) && is_legal(&Target::Permanent(c.id))
            });
            if pick.is_some() {
                return pick;
            }
        }

        // Battlefield: preferred-controller permanents first, then any
        // match (else the bot Vines its opponent's bear instead of its own).
        //
        // Source avoidance (see `auto_target_for_effect_avoiding`): skip the
        // source on the first pass, fall back to it only if nothing else is
        // legal. It keeps a trigger off its *controller's own* source; an
        // opponent-controlled trigger (a Probing Telepathy copy of Aboleth
        // Spawn's ETB copying Flametongue Kavu) has no reason to spare it.
        let hostile_source = avoid_source.is_some_and(|s| {
            self.battlefield
                .find_by_id(s)
                .is_some_and(|c| c.controller != controller && !self.same_team(c.controller, controller))
        });
        // Doubling its own counters is the point of a source's "double the
        // counters on target creature" (a backed-up Bright-Palm trigger), so
        // the source isn't spared there.
        let own_doubling = doubles_target_counters(eff);
        let is_avoided = |cid: CardId| -> bool {
            avoid.contains(&cid)
                && !((hostile_source || own_doubling) && Some(cid) == avoid_source)
        };
        // Friendly pumps (Magecraft / Repartee fan-outs, PumpPT spells) take
        // the highest-power friendly creature so the buff lands on the
        // biggest threat. Hostile picks keep the first-match heuristic.
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
        // A token copy of a legend you control dies to the legend rule (CR
        // 704.5j), so a token-copy effect takes a nonlegendary permanent
        // first. Copying Venser, Fervent Forger looped (each token's ETB
        // copied the Technique again; two Vensers at a table copied each
        // other to the action cap), so every walk below ranks it so.
        let copies_to_token = eff.any_nested(&|e| {
            matches!(e, Effect::CreateTokenCopyOf { .. } | Effect::CreateTokenCopiesHasteSac { .. })
        });
        let legend_copy = |cid: CardId| {
            copies_to_token
                && self
                    .battlefield
                    .find_by_id(cid)
                    .is_some_and(|c| c.definition.supertypes.contains(&crate::card::Supertype::Legendary))
        };
        let mut primary_candidates = collect_legal_on_player(primary_player);
        if prefer_friendly && !primary_candidates.is_empty() {
            // Sort by descending power so the strongest creature wins — except
            // a Zada, which copies a spell aimed only at it onto every other
            // creature it could target, so it outranks any single body.
            let fans_out =
                |cid: CardId| self.battlefield.find_by_id(cid).is_some_and(copies_to_the_team);
            // "Double the counters on target creature" (Bright-Palm, Tanazir)
            // is worth what the creature already carries, so the most
            // counters outrank the most power.
            // So is "another counter of a kind it has" (Animation Module).
            let ranks_counters = doubles_target_counters(eff) || counter_aim(eff) == CounterAim::AddPresent;
            let counters = |cid: CardId| -> u32 {
                if !ranks_counters {
                    return 0;
                }
                self.battlefield.find_by_id(cid).map_or(0, |c| c.counter_sum())
            };
            // An untap wants something tapped (Garruk Wildspeaker's lands).
            let untaps = eff.any_nested(&|e| matches!(e, Effect::Untap { .. }));
            let already_untapped =
                |cid: CardId| untaps && self.battlefield.find_by_id(cid).is_some_and(|c| !c.tapped);
            primary_candidates.sort_by_cached_key(|c| {
                (
                    legend_copy(c.0),
                    already_untapped(c.0),
                    !fans_out(c.0),
                    std::cmp::Reverse(counters(c.0)),
                    std::cmp::Reverse(c.1),
                )
            });
        } else {
            // Hostile pick: un-warded first, then the biggest threat (without
            // the power term removal took whichever enemy sat earliest on the
            // board).
            // A counter strip wants a permanent that carries counters.
            let strips = primary_candidates.len() > 1 && counter_aim(eff) == CounterAim::Strip;
            let bare = |cid: CardId| strips && self.battlefield.find_by_id(cid).is_some_and(|c| c.counter_sum() == 0);
            primary_candidates
                .sort_by_cached_key(|c| (legend_copy(c.0), hostile_ward(c.0), bare(c.0), std::cmp::Reverse(c.1)));
        }
        if let Some(&(cid, _)) = primary_candidates.first() {
            return Some(Target::Permanent(cid));
        }
        // A hostile pick whose ranked opponent has nothing legal goes to the
        // other opponents next, ranked the same way — the battlefield-order
        // walk below took the caster's own creature for Liliana's −2/−1 while
        // a third seat's sat there. A duel has no other opponent.
        if !prefer_friendly && self.players.len() > 2 {
            let mut others: Vec<(CardId, i32)> = self
                .living_seats()
                .filter(|&p| p != primary_player && p != controller && !self.same_team(p, controller))
                .flat_map(collect_legal_on_player)
                .collect();
            others.sort_by_cached_key(|c| (legend_copy(c.0), hostile_ward(c.0), std::cmp::Reverse(c.1)));
            if let Some(&(cid, _)) = others.first() {
                return Some(Target::Permanent(cid));
            }
        }
        // A hostile "any target" reaches its own controller only once no
        // opponent and no opposing permanent is legal: The Black Arrow's ping
        // went to its caster's face past a hexproof player's Dragon.
        if accepts_player && !gathers && !prefer_friendly {
            let me = Target::Player(controller);
            if player_ok(&me) {
                return Some(me);
            }
        }
        for pass_warded in [false, true] {
            let mut legal = self
                .battlefield
                .iter()
                .filter(|c| !is_avoided(c.id))
                .filter(|c| pass_warded || !hostile_ward(c.id))
                .filter(|c| is_legal_bf(c));
            // A friendly copy whose own side had nothing (Venser's "an
            // opponent controls") still takes a nonlegend first.
            let pick = if copies_to_token { legal.min_by_key(|c| legend_copy(c.id)) } else { legal.next() };
            if let Some(c) = pick {
                return Some(Target::Permanent(c.id));
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
        // Final fallback: any graveyard, then exile (reanimation whose
        // prefer-graveyard branch found nothing). Hexproof doesn't apply
        // there, but `is_legal` still runs so zone-aware legality applies.
        //
        // **Gated on the effect being able to reach off the board.** The
        // filter language has no zone predicate, so a board-shaped
        // requirement ("target creature", a bare `Not(Player)`) is satisfied
        // by a creature *card* in a graveyard or exile; ungated, "destroy
        // target creature" aimed at an exiled card and silently fizzled in
        // the training path. Resolving targetless is right there.
        //
        // `may_target_offboard_card` is the effect-shape half (walk *order*
        // stays `prefers_graveyard_target`'s narrower question) and
        // `mentions_offboard_zone` the filter half, the same gate the
        // `ChooseCards` modal path takes. See ENGINE_BACKLOG's "the target
        // enumerator is zone-blind".
        if !eff.offboard_target_scope(req) {
            return None;
        }
        // The picks `avoid` holds beyond the source stay avoided here too: a
        // fan-out over graveyards (Mysterious Stranger's one card per
        // graveyard) re-picked its slot-0 card from this walk and stopped
        // after one target. The source itself is this last resort's to take
        // (Golgari Thug's dies trigger tops the Thug).
        let avoid = avoid.get(1..).unwrap_or(&[]);
        for player in &self.players {
            if let Some(c) = player
                .graveyard
                .iter()
                .filter(|c| !avoid.contains(&c.id))
                .map(|c| Target::Permanent(c.id))
                .find(|t| is_legal(t))
            {
                return Some(c);
            }
        }
        // Exile only when the filter names it (`target_out_of_zone`).
        if !req.mentions_offboard_zone() {
            return None;
        }
        if let Some(c) = self
            .exile
            .iter()
            .filter(|c| !avoid.contains(&c.id))
            .map(|c| Target::Permanent(c.id))
            .find(|t| is_legal(t))
        {
            return Some(c);
        }
        None
    }

    /// Enumerate every legal slot-0 target for `eff` from `controller`'s
    /// perspective, for the UI trigger-target picker (`Decision::ChooseTarget`
    /// for a wants_ui controller).
    ///
    /// Order: players (controller first), battlefield permanents, each
    /// graveyard (controller first), then exile — the auto-picker's traversal,
    /// accumulated instead of returning on first hit.
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
    /// The two must agree: this is what a `wants_ui` controller is offered, the
    /// picker what everyone else gets. Unresolved, `ManaValueAtMostXFromCost` /
    /// `ManaValueAtMostConverged` match nothing and a UI player's targeted ETB
    /// resolved as a no-op (Sundering Archaic).
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
        // No `.or_else(target_filter_for_slot(0))` here, unlike the picker,
        // and not a gap: `core_rules::target_walkers::the_primary_target_
        // filter_agrees_with_the_slot_walker_on_slot_zero` holds the two
        // walkers to the same filter over the catalog, so the picker's `or`
        // arm is unreachable on shipped cards.
        // Borrowed unless the tree names X or converge — see the picker.
        let req_src = eff.primary_target_filter();
        let req_owned = req_src
            .filter(|f| f.names_x_or_converge())
            .map(|f| f.resolve_x(x).resolve_converge(converge));
        debug_assert!(
            req_owned.is_some()
                || req_src.is_none_or(|f| *f == f.resolve_x(x).resolve_converge(converge)),
            "names_x_or_converge said no but the rewrite moved the filter: {req_src:?}",
        );
        let req = req_owned.as_ref().or(req_src).unwrap_or(&any_filter);
        // Same scope gate as the auto-picker's off-board fallback, so the
        // enumerated set and the picked target can't disagree about which
        // zones the effect reaches.
        let offboard = eff.offboard_target_scope(req);
        self.legal_targets_for_filter_scoped(
            req,
            // See the picker's note: a player-only slot 0 is a player slot
            // whatever the body does with it, and the walker says so.
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
    /// The view layer greys the ability out when this is empty, so it must
    /// agree with the activation path's modal (else a live-looking row is a
    /// dead button).
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
    /// slot's filter (CR 115.7c retargeting) uses it directly.
    ///
    /// Runs under one freeze: every candidate asks the same layer-aware
    /// questions, and without it `check_target_legality` would re-gather every
    /// continuous effect per candidate.
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
    /// requirement ("target creature", `Not(Player)`, `Any`) is satisfied by a
    /// creature *card* in a graveyard or exile (Cuombajj Witches listed every
    /// graveyard card). `offboard` is that question asked once, the same way
    /// the auto-picker asks it (`may_target_offboard_card ||
    /// mentions_offboard_zone`): the two must agree or the UI and training
    /// paths target different sets.
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
        // CR 115.1 — each stack ability is a candidate of its own.
        if req.mentions_stack_ability() {
            out.extend(
                self.stack_ability_targets(req, controller, source)
                    .into_iter()
                    .filter(|t| self.check_target_legality(t, controller).is_ok()),
            );
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
        // Exile only when the filter names it (`target_out_of_zone`).
        if req.mentions_offboard_zone() {
            for c in &self.exile {
                let t = Target::Permanent(c.id);
                if is_legal(&t) {
                    out.push(t);
                }
            }
        }
        out
    }

    /// CR 702.21 — does `cid` belong to someone other than `actor` and
    /// carry a non-trivial Ward? Targeting it gets the spell countered
    /// unless the tax is paid, so both auto-target walks rank warded
    /// hostile permanents below un-warded ones.
    pub(crate) fn has_hostile_ward(&self, cid: CardId, actor: usize) -> bool {
        use crate::card::Keyword;
        let Some(c) = self.battlefield.find_by_id(cid) else { return false };
        if self.same_team(c.controller, actor) {
            return false;
        }
        // Presence-gated like `check_target_legality_inner`'s keyword read:
        // the view is a memo hit once the scope has gathered, and a gather
        // otherwise — skip the gate in the first case, the view in the second.
        if !self.layers_memoized()
            && !self.card_keyword_possible_on(c, |k| matches!(k, Keyword::Ward(_)))
        {
            return false;
        }
        let cp = self.computed_permanent_on(c);
        let kws: &[Keyword] = match &cp {
            Some(cp) => cp.keywords(),
            None => &c.definition.keywords,
        };
        kws.iter().any(|k| matches!(k, Keyword::Ward(w)
            if !crate::game::actions::ward_cost_is_trivial(w)))
    }

    /// Pick legal targets for every slot the effect uses, returning
    /// `(slot 0, Vec<slot 1..>)`, each the best legal pick per
    /// `auto_target_for_effect_avoiding`'s preferences (slots via
    /// `target_filter_for_slot_in_mode`). A slot with no legal candidate is
    /// skipped, matching "up to N target" resolving with fewer. Enumeration
    /// stops at the first slot index with no `Selector::TargetFiltered { slot }`
    /// in the tree (Snow Day: 2, Homesickness: 3). Drives bot multi-target
    /// casts without a UI prompt.
    #[cfg_attr(feature = "trig-census", track_caller)]
    pub fn auto_targets_for_effect_all_slots(
        &self,
        eff: &Effect,
        controller: usize,
        mode: Option<usize>,
    ) -> (Option<Target>, Vec<Target>) {
        // Which of the ~26 call sites the enumerator's 3.6 % of `sealed`
        // actually comes from — see `call_site_census`. `#[track_caller]`
        // rather than a counter per site: the attribute is behind the
        // feature, so the shipped wrapper is the one line it always was.
        #[cfg(feature = "trig-census")]
        call_site_census::note(std::panic::Location::caller());
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
        self.auto_targets_for_effect_all_slots_x(eff, controller, mode, kicked, source, None)
    }

    /// [`Self::auto_targets_for_effect_all_slots_kicked`] with the cast's X
    /// known: a slot filter that names X ("mana value X or less" — Finale of
    /// Promise) is read at that X instead of at no X, where nothing matched.
    pub fn auto_targets_for_effect_all_slots_x(
        &self,
        eff: &Effect,
        controller: usize,
        mode: Option<usize>,
        kicked: bool,
        source: Option<CardId>,
        x: Option<u32>,
    ) -> (Option<Target>, Vec<Target>) {
        // Slot 0 with its own numbered `TargetFiltered` filter (Rabid Bite's
        // friendly power source inside `Value::PowerOf`) is picked by that
        // filter in the loop below; a bare `Target(0)` (Lightning Bolt) uses
        // the source-aware picker. The lookup walks the whole tree, so its
        // answer is kept for the loop.
        if let Some(pick) = self.fight_pair_pick(eff, controller, source, None) {
            return pick;
        }
        let slot0_filter = eff.target_filter_for_slot_in_mode_kicked(0, mode, kicked);
        let mut slot_0 = if slot0_filter.is_some() {
            None
        } else {
            self.auto_target_for_effect_avoiding(eff, controller, None)
        };
        let mut additional = Vec::new();
        let mut already_picked: Vec<CardId> = Vec::new();
        // CR 601.2c — "for each opponent, … target … that player controls"
        // (`ForEachOpponentTarget`): no two picks may share a controller, as
        // the cast validator enforces (else Desecrate Reality was rejected).
        let per_opponent = eff.per_opponent_targets(mode);
        let mut picked_controllers: Vec<usize> = Vec::new();
        let mut slot: u8 = if slot0_filter.is_some() { 0 } else { 1 };
        // Loop-invariant: the seat does not change between slots.
        let opp = self
            .first_opponent_of(controller)
            .unwrap_or((controller + 1) % self.players.len());
        // Asked lazily: most effects meet no player or counter question.
        let strips = std::cell::OnceCell::new();
        let aim = || *strips.get_or_init(|| counter_aim(eff));
        // Cap at 16 slots — no real card uses more than 4, but cap defensively.
        while slot < 16 {
            // "Destroy X target …": no slot past a known X.
            if x.is_some_and(|x| eff.slot_past_x_cap(slot, x)) {
                break;
            }
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
            let req_at_x = x.filter(|_| req.names_x_or_converge()).map(|x| req.resolve_x(x));
            let req = req_at_x.as_ref().unwrap_or(req);
            // CR 601.2c — a cross-slot filter ("another target", "with the same
            // controller") reads the slots already chosen. The cast validator
            // stamps `target_slots_scratch`; this walk passes its picks
            // explicitly, or "another" was answered vacuously and the action
            // then failed validation. Asked once per slot, since almost no
            // filter carries a cross-slot atom.
            let cross = req.mentions_cross_slot();
            let filled: Vec<Option<Target>> = if cross {
                std::iter::once(slot_0.clone()).chain(additional.iter().cloned().map(Some)).collect()
            } else {
                Vec::new()
            };
            let is_legal = |t: &Target| -> bool {
                self.evaluate_requirement_static(req, t, controller, source)
                    && (!cross || self.cross_slot_targets_ok(req, t, &filled))
                    && self.check_target_legality(t, controller).is_ok()
            };
            // The battlefield candidate walk below already holds each
            // permanent; hand it over instead of re-finding it by id.
            let gates = crate::game::effects::eval::PrintedGates::default();
            // This walk evaluated the requirement once per board permanent per
            // slot (PERF `(-300)`), and a filter naming a card type rejects much
            // of the board on that one test: one walk per *slot* answers it for
            // every candidate; the walker still decides what the pre-test keeps.
            let type_gate = self.printed_type_prefilter(req, &gates);
            let is_legal_bf = |c: &CardInstance| -> bool {
                if let Some(t) = &type_gate
                    && crate::game::effects::eval::printed_type_rejects(t, c)
                {
                    // The audit for the pre-test, in the shape `(-296)`'s
                    // narrowed source list uses: a candidate dropped here that
                    // the walker would have kept is a target the enumerator
                    // never offers.
                    debug_assert!(
                        !self.requirement_on_permanent(req, c, controller, source, &gates),
                        "the printed card-type pre-test dropped a candidate the walker keeps: {}",
                        c.definition.name,
                    );
                    return false;
                }
                self.requirement_on_permanent(req, c, controller, source, &gates)
                    && (!cross
                        || self.cross_slot_targets_ok(req, &Target::Permanent(c.id), &filled))
                    && self.check_target_legality(&Target::Permanent(c.id), controller).is_ok()
            };
            // CR 115.3 — one "up to N target cards" names N different
            // cards: a card an earlier slot already took is not a candidate
            // (Ever After filled both slots with one creature card, and the
            // cast was rejected every time).
            let is_legal_gy = |c: &CardInstance| -> bool {
                slot_0 != Some(Target::Permanent(c.id))
                    && !additional.contains(&Target::Permanent(c.id))
                    && self.requirement_on_graveyard_card(req, c, controller, source)
                    && (!cross
                        || self.cross_slot_targets_ok(req, &Target::Permanent(c.id), &filled))
                    && self.check_target_legality(&Target::Permanent(c.id), controller).is_ok()
            };
            let pick = {
                // Player slots: controller first (caster-friendly), then opponent —
                // unless the seat's `hostile_player_targets` flag is set and the
                // slot is hostile (discard, damage, life loss, mill, …:
                // `player_slot_is_hostile`), when the opponent comes first.
                // Caster-first had Arcane Omens discard the caster's own hand, and a
                // bot's search never sees the other player since the candidate list
                // bakes one target.
                let mut found: Option<Target> = None;
                let hostile = self.hostile_player_slot(controller, eff, slot, mode);
                // An optional gift slot (Parnesse's opponent copy) is never
                // filled with an opponent by the auto-picker; asked lazily,
                // only on the opponent fallback.
                let gift = || eff.player_slot_is_gift(slot) && eff.target_slot_optional(slot, mode);
                let (first, second) = if hostile { (opp, controller) } else { (controller, opp) };
                let player_first = Target::Player(first);
                let player_second = Target::Player(second);
                // CR 601.2c — a divide effect's slots are one "any number of
                // targets": a player already named can't be named again, and
                // Forked Bolt / Avacyn's Judgment aimed both halves at the same
                // face, which the cast rejects — no divider was ever cast.
                let distinct = eff
                    .distinct_target_range(mode)
                    .filter(|&(first, _)| slot > first)
                    .map(|(first, _)| first);
                let taken = |t: &Target| {
                    distinct.is_some_and(|first| {
                        (first == 0 && slot_0.as_ref() == Some(t))
                            || additional.iter().skip(usize::from(first.saturating_sub(1))).any(|a| a == t)
                    })
                };
                // A counter strip takes a player's poison only (CR 122.6):
                // the caster's own when it has some, never an opponent's.
                let strip_skip = |t: &Target| matches!(t, Target::Player(_)) && self.counter_player_skip(aim(), t, controller);
                if is_legal(&player_first) && !taken(&player_first) && !strip_skip(&player_first) {
                    found = Some(player_first);
                } else if let Some(t) = hostile
                    .then(|| {
                        // CR 102.2 — at N seats a hostile slot the first
                        // opponent can't take ("any other target") names the
                        // next opponent, not the caster's own face.
                        self.opponents_of(controller)
                            .into_iter()
                            .map(Target::Player)
                            .find(|t| *t != player_first && is_legal(t) && !taken(t) && !strip_skip(t))
                    })
                    .flatten()
                {
                    found = Some(t);
                } else if is_legal(&player_second)
                    && !taken(&player_second)
                    && !strip_skip(&player_second)
                    && (second == controller || !gift())
                    // A hostile slot never falls back onto the caster's own
                    // face: the battlefield walk below ranks the other side
                    // first, and an optional slot may stay empty.
                    && (!hostile || second != controller)
                {
                    found = Some(player_second);
                }
                // Graveyard-preferring effects (reanimate / regrow — Young
                // Necromancer's reflexive return) must not grab a battlefield
                // permanent that happens to match the filter; sweep
                // graveyards first for those.
                if found.is_none() && eff.prefers_graveyard_target() && !req.names_permanent() {
                    found = best_graveyard_card(
                        self,
                        controller,
                        opp,
                        eff.prefers_friendly_target_for_slot(slot, mode),
                        !req.excludes_opponents_side() && eff.target_slot_optional_x(slot, mode, x.unwrap_or(u32::MAX)),
                        is_legal_gy,
                    );
                }
                // A slot that names exile ("target creature card exiled with
                // [this]" — Shelob, Quintorius): the graveyard sweep above
                // never looks there, and the board walk below can't match it.
                // The most valuable legal card.
                if found.is_none() && req.mentions_offboard_zone() {
                    found = self
                        .exile
                        .iter()
                        .filter(|c| is_legal_gy(c))
                        .max_by_key(|c| (c.printed_cmc(), std::cmp::Reverse(c.id.0)))
                        .map(|c| Target::Permanent(c.id));
                }
                // Battlefield: prefer one not already picked by an earlier slot, to
                // avoid double-targeting when the filter is permissive.
                //
                // Ranked by *side* first (as the unfiltered picker
                // `auto_target_for_effect_avoiding_set_xc` is): battlefield order
                // had "destroy target creature" hit the caster's own creature.
                // Friendly effects nearly always say "you control" in the filter, so
                // `is_legal`, not this ordering, keeps a gift on the caster's side.
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
                    if per_opponent {
                        picked_controllers.clear();
                        picked_controllers.extend(
                            already_picked.iter().filter_map(|id| self.battlefield_find(*id)).map(|c| c.controller),
                        );
                    }
                    // Per *slot*, not per effect: Homesickness is
                    // "target player draws two" + "tap and stun target
                    // creature", and the whole-effect classifier reads the
                    // gift and calls the whole spell friendly, which had the
                    // bot stunning its own board.
                    let prefer_friendly = eff.prefers_friendly_target_for_slot(slot, mode);
                    // A filter that rules out a side settles it: "you don't control"
                    // makes every candidate the wanted one (Hate Mirage's token copies
                    // read as a gift and were never cast), and "you control" the other
                    // way (Raubahn's "up to one target Equipment you control" read as a
                    // hostile Attach and was never filled).
                    let side_fixed = req.excludes_your_side() || req.excludes_opponents_side();
                    // 0 = wanted side, un-warded; 1 = wanted side, warded;
                    // 2 = the other side (a last resort, see `optional`).
                    let rank = |id: CardId, ctrl: usize| -> u8 {
                        let wanted = side_fixed || (ctrl == controller) == prefer_friendly;
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
                    // Within a rank, the biggest body wins.
                    // A plain loop, not a `filter().filter().map().min()` chain, which
                    // pays adapter forwarding per permanent per slot (PERF (-78)'s test
                    // 1). The key ends in the id, so ties can't happen.
                    let mut best: Option<(u8, bool, bool, i32, CardId)> = None;
                    for c in self.battlefield.iter() {
                        if already_picked.contains(&c.id)
                            || (per_opponent && picked_controllers.contains(&c.controller))
                            || !is_legal_bf(c)
                        {
                            continue;
                        }
                        let power = self
                            .computed_permanent(c.id)
                            .map(|cp| cp.power)
                            .unwrap_or(c.definition.power);
                        // Zada first among friendly picks (`copies_to_the_team`).
                        let fan = prefer_friendly
                            && c.controller == controller
                            && copies_to_the_team(c);
                        // A counter strip wants a permanent carrying counters.
                        let bare = c.counter_sum() == 0 && aim() != CounterAim::None;
                        let key = (rank(c.id, c.controller), !fan, bare, -power, c.id);
                        if best.is_none_or(|b| key < b) {
                            best = Some(key);
                        }
                    }
                    match best {
                        // Never spend an optional slot on the wrong side.
                        Some((2, ..)) if optional => {}
                        Some((.., id)) => found = Some(Target::Permanent(id)),
                        None if optional => {}
                        // CR 115.3 — one "X target …" names distinct objects:
                        // once they run out, the slot stays empty.
                        None if distinct.is_some() => {}
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
                // Graveyard cards (an `InGraveyard` reflexive return — Curious
                // Forager) as a last resort, gated as the single-slot picker is (CR
                // 109.2): a board slot has no graveyard fallback, or Doom Blade
                // aims at a creature card that already died.
                if found.is_none() && eff.offboard_target_scope(req) {
                    found = best_graveyard_card(
                        self,
                        controller,
                        opp,
                        eff.prefers_friendly_target_for_slot(slot, mode),
                        !req.excludes_opponents_side() && eff.target_slot_optional_x(slot, mode, x.unwrap_or(u32::MAX)),
                        is_legal_gy,
                    );
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

/// The graveyard card a bot names for a graveyard slot. Seats go
/// friendly-first (controller, `opp`, the rest in turn order) or hostile-first
/// (`opp`, the other opponents, controller last) — CR 102.2: at N seats every
/// opponent's graveyard is reachable, not only `opp`'s. Within a seat the
/// richest card wins (creature, then mana value; the earliest on a tie); a
/// hostile body that only reaches its controller's own graveyard (magecraft's
/// "exile a card from your graveyard") spends the poorest instead.
///
/// Takes the card, not its id: the requirement goes through
/// [`GameState::requirement_on_graveyard_card`] at the enumerator's call
/// site, which skips the walker's battlefield miss and re-find (PERF `(-185)`).
fn best_graveyard_card(
    state: &GameState,
    controller: usize,
    opp: usize,
    friendly: bool,
    spare_own: bool,
    is_legal: impl Fn(&CardInstance) -> bool,
) -> Option<Target> {
    let n = state.players.len();
    let others = (1..n)
        .map(|k| (controller + k) % n)
        .filter(|&s| s != opp && state.players[s].is_alive());
    let head = if friendly { [Some(controller), Some(opp)] } else { [Some(opp), None] };
    // An optional unfriendly slot that could name an opponent's card ("exile
    // up to two target cards from a single graveyard") is declined rather
    // than spent on your own.
    let tail = (!friendly && !spare_own).then_some(controller);
    for p in head.into_iter().flatten().chain(others).chain(tail) {
        let poorest = !friendly && p == controller;
        let mut best: Option<((bool, u32), CardId)> = None;
        for c in &state.players[p].graveyard {
            if !is_legal(c) {
                continue;
            }
            let key = (c.definition.is_creature(), c.definition.cost.cmc());
            if best.is_none_or(|(k, _)| if poorest { key < k } else { key > k }) {
                best = Some((key, c.id));
            }
        }
        if let Some((_, id)) = best {
            return Some(Target::Permanent(id));
        }
    }
    None
}


/// Which call site each `auto_targets_for_effect_all_slots` enumeration
/// comes from. Callgrind can't split it — all ~26 sites inline into
/// `bot::cast_candidates` — so `#[track_caller]` on the wrapper gives the
/// line.
///
/// Compile-time gated on `trig-census` so the shipped binary carries none of
/// it (`#[track_caller]` included); then `CRAB_TARGET_SITE_CENSUS=1` at run
/// time, and `bot_ladder` prints the table heaviest first. ⚠ Feature-off
/// builds moved Ir by a few parts per million against the pre-census tip
/// (code layout, not semantics); the debuginfo differs, so compare the
/// counter, not `md5sum`.
#[cfg(feature = "trig-census")]
pub mod call_site_census {
    use std::sync::Mutex;

    /// `file:line -> enumerations`. A `Mutex<Vec<_>>` rather than an atomic
    /// array because the key set is not known until the run: this is a
    /// census, and nothing runs it unless the variable is set.
    static SITES: Mutex<Vec<(&'static str, u32, u64)>> = Mutex::new(Vec::new());

    pub fn on() -> bool {
        static LEVEL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *LEVEL.get_or_init(|| match std::env::var("CRAB_TARGET_SITE_CENSUS") {
            Ok(v) => !v.is_empty() && v != "0",
            _ => false,
        })
    }

    pub(super) fn note(loc: &'static std::panic::Location<'static>) {
        if !on() {
            return;
        }
        let Ok(mut sites) = SITES.lock() else { return };
        match sites.iter_mut().find(|(f, l, _)| *f == loc.file() && *l == loc.line()) {
            Some((_, _, n)) => *n += 1,
            None => sites.push((loc.file(), loc.line(), 1)),
        }
    }

    /// The table, heaviest first.
    pub fn snapshot() -> Vec<(&'static str, u32, u64)> {
        let Ok(sites) = SITES.lock() else { return Vec::new() };
        let mut out = sites.clone();
        out.sort_by_key(|(_, _, n)| std::cmp::Reverse(*n));
        out
    }
}

/// Zada, Hedron Grinder: a spell aimed only at it is copied onto every other
/// creature it could target, so a friendly pick outranks any single body.
fn copies_to_the_team(c: &CardInstance) -> bool {
    c.definition
        .triggered_abilities
        .iter()
        .any(|t| matches!(t.effect, Effect::CopyForEachOtherTargetableCreature | Effect::CopyForChosenOtherTargetableCreature))
}

/// The effect doubles the counters on its target — a friendly target is then
/// picked by the counters it carries.
fn doubles_target_counters(eff: &Effect) -> bool {
    match eff {
        Effect::DoubleCountersOnEach { .. } | Effect::DoubleAllCountersOn { .. } => true,
        Effect::Seq(steps) => steps.first().is_some_and(doubles_target_counters),
        _ => false,
    }
}

impl GameState {
    /// Whether the slot walkers aim player slot `slot` of `eff` at an opponent
    /// first: a hostile slot (damage, discard, life loss, …) for a seat with
    /// `hostile_player_targets`, in every pod, and in a two-seat Commander game
    /// (caster-first burned its own face: Crackle with Power at four seats,
    /// Jaya's Immolating Inferno for X = 25). Non-Commander duels keep the
    /// control's caster-first pick.
    pub(crate) fn hostile_player_slot(
        &self,
        controller: usize,
        eff: &crate::effect::Effect,
        slot: u8,
        mode: Option<usize>,
    ) -> bool {
        (self.players[controller].hostile_player_targets
            || self.players.len() > 2
            || self.players.iter().any(|p| !p.commanders.is_empty()))
            && eff.player_slot_is_hostile(slot, mode)
    }
}

/// What a counter effect does to a target that has counters: strip one
/// (Etched Slith, Price of Betrayal) or add one of a kind it has (Animation
/// Module).
#[derive(Clone, Copy, PartialEq, Eq)]
enum CounterAim {
    None,
    Strip,
    AddPresent,
}

fn counter_aim(eff: &Effect) -> CounterAim {
    if eff.any_nested(&|e| matches!(e, Effect::RemoveCountersUpTo { .. } | Effect::RemoveAnyCounter { .. })) {
        CounterAim::Strip
    } else if eff.any_nested(&|e| matches!(e, Effect::AddCounterOfPresentKind { .. })) {
        CounterAim::AddPresent
    } else {
        CounterAim::None
    }
}

impl GameState {
    /// CR 122.1 — a player target a counter effect can't use well: a strip
    /// takes poison only (the caster's own, never an opponent's); an add
    /// needs an opponent's poison or a friendly seat's experience / energy.
    fn counter_player_skip(&self, aim: CounterAim, t: &Target, controller: usize) -> bool {
        let Target::Player(p) = *t else { return false };
        let pl = &self.players[p];
        let friendly = p == controller || self.same_team(p, controller);
        match aim {
            CounterAim::None => false,
            CounterAim::Strip => p != controller || pl.poison_counters == 0,
            CounterAim::AddPresent => {
                if friendly { pl.experience == 0 && pl.energy == 0 } else { pl.poison_counters == 0 }
            }
        }
    }
}
